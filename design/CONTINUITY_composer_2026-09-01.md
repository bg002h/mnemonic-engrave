# CONTINUITY — Wallet Policy COMPOSER cycle (arbitrary tr/wsh authoring on the SH2), 2026-09-01

**Resume here (updated later on 2026-09-01, after "Proceed autonomously").**
The spec's R0 is CLOSED under lens-closure at `49d2dae` (+ wording `0b56ed4`).
The STAGED plan and the Stage 0 detailed plan are committed at `3a799fa` with
the build gate GREEN (`scripts/plan-build-gate-md.sh`, toolchain 1.85.0: 48
compose tests, 47 pass, one PINNED red for the un-assembled MANIFEST fragment;
clippy clean; md-cli compiles). Gate tooling fixes: `e9c42c9`, `8382d69`.
**Plan R0 round 0 DONE:** fidelity lens (opus, 0C/4I/5M/3N, `b820b64`) and tests
lens (sonnet, 0C/2I/4M/2N, 20 mutations / 19 caught, `f531cff`) persisted; folded
at `891b17d` (six Importants: tag list, presets pinned + decaying signature,
family-wide 5b cross-check, md encode parity = new Task 8, two-digit inlining,
Task 2 sequencing) and `fb65f2c` (controller hand-check: Task 8 gate is
MINTING-only after an unconditional draft broke two n1 reading-verb tests; the
two keyless-wsh vectors are `no-corpus`; wired scratch copy ran the whole md-cli
suite 761/761). Build gate at both: 52 compose tests, 51 pass, 1 pinned red;
clippy clean; md-cli compiles. Plan is now 9 tasks.
**Rounds 1 and 2 DONE (2026-09-02):** r1 verification 20/20 FIXED, one new Minor
(`5aa340b`) folded at `761ded7`; r2 verification of `fb65f2c`+`761ded7` 0 new
defects / 0 false claims (`1827e1b`), range nit folded at the closure commit.
**PLAN R0 GREEN.** Status line in the plan header.
**S0 IMPLEMENTED (2026-09-02):** worktree `/scratch/code/shibboleth/wt-composer-s0`,
branch `composer-s0`, nine commits `b19dca7b..9820e618`; report persisted
`db0d729`; controller re-ran the gates independently: fmt + clippy clean,
nextest 1318/1318, doctests ok, 126 new vector files / 22 conformance files,
tree clean. Plan's Expected lines corrected to the measured facts (see `git log`).
**Whole-diff execution review of `composer-s0`:** 0C/1I/2M/3N (`976cc45`);
folded ON THE BRANCH at `7c9b4fd7` (encode's signature refusal names
--experimental; CHANGELOG states the gate's blast radius: descriptor/address/
vectors --template share it, the first two without an opt-out; malleability +
mixed timelocks newly enforced under wsh/sh; the round-trip test's `if let Ok`
guard removed; --json documented) and `66bdf2f4` (follow-up
`md-descriptor-address-template-lack-experimental`). Plan record updated.
**S0 WHOLE-DIFF REVIEW CLOSED 0C/0I:** gate re-run on the folded worktree
(fmt/clippy clean, 1318/1318, doctests ok); sonnet verification 4 FIXED / 1
DECLINED (reason holds) / 0 regressions (`agent-reports/composer-S0-exec-review-r1-fold-verification.md`).
**S0 SHIPPED TO MAIN (2026-09-02):** descriptor-mnemonic `origin/main` =
`66bdf2f4` (staging run 33607451817: cargo test ubuntu + cargo clippy success,
no bypass); mnemonic-engrave `origin/master` = `46fc91b8` (test (rust + go)
success, no bypass); `agent-reports/composer-S0-push-report.md` (`8eda7c2`).
No tag, no version bump, no publish (blocked, see the staged plan's S0 exit;
the operator's decision). Worktree `wt-composer-s0` left in place.
**S1 `now:` default — STAND-IN RULING (fable, revocable):** (c) narrowed —
auto-append only when the payload holds a `key:` or `hash:` record; `--now`
opt-in; `--no-now` opt-out; supplied `now:` wins (`agent-reports/composer-S1-decision-now-default.md`, `7612066`).
Its fold (spec §6a ×2, §10 item 2, §7g row; brainstorm §3.12 item 21; S1 plan
Task 4 + header) is applied together with the S1 R0 round-0 fold.
**S1 R0 round 0 DONE:** fidelity (opus, 0C/6I/10M/3N, `85493a5`) and tests
(sonnet, 0C/3I/2M, `343f17a`) persisted; folded together with the ruling at
`e20eae1` (spec 6a/10 item 2/7g + brainstorm item 21) and `3919f1f` (+ cite
fix after it). Machine-checked in the gate scratch copy with Task 2 + ruled
Task 4 wired: 612/612, clippy clean, threaded runner clean; fixture 40 rows,
sha256 `a894e619…46c3`. Two of the controller's own fold slips were caught by
that run (a digits-only "uppercase hex" row; the F-246 pre-check placed after
the ceremony) and are recorded in the plan.
**S1 PLAN R0 GREEN (2026-09-02):** r1 verification 29 FIXED / 1 PARTIAL / 0
regressions (`ec29be8`), the partial + one prose slip folded at the closure
commit (see `git log`). Records pushed to origin/master `b8e19ebb` before it.
**IN FLIGHT:** ONE implementer (opus, UC off) in two worktrees (mnemonic-engrave
`wt-composer-s1` off master, branch `composer-s1`; mnemonic-secret
`wt-ms-bip48-p2tr` off `5f37b43`, branch `bip48-p2tr`), Tasks 1-5 and 7 (Task 6
is DONE); report → `agent-reports/composer-S1-implementation-report.md`.
**THEN:** persist → verify gates independently in both worktrees → whole-diff
opus execution review (both diffs) → fold → sonnet verification → merge both
via each repo's staging ritual → release `me` 0.8.0 and `ms` 0.17.0 per each
repo's release process → Stage 2 plan (fork Go: builder, pk_h arm, PolicyShape,
Classify lockstep, vendoring of both fixtures).
**Payload-spec fold (S1 Task 6) CLOSED under its own R0:** r0 0C/3I/4M/5N
(`bb49953`) folded at `44765d7` (+ F-450 `72ac66d`); r1 verification 11 FIXED /
1 PARTIAL (`de34664`), the partial folded at `fdf7671`.
**After (1) is 0C/0I:** merge `composer-s0` into descriptor-mnemonic `main`
(fast-forward), version bump + publish per `design/RELEASE_PROCESS.md` there,
vendor nothing yet (S2 does), then S1's R0 (plan `273f414`+) right before its
implementer.
**Stage 1 DRAFT written meanwhile (`273f414`, NOT R0-reviewed):**
`design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md` + `scripts/plan-build-gate-me.sh`
(`b44fb61`). Machine-checked in scratch copies (me suite 599/609 with the six
named auto-append casualties; ms Task 5 check in flight). It carries an OPEN
QUESTION for the operator: spec §6a's DEFAULT `now:` auto-append changes every
payload's identity and adds a Wallet-Policy-only record to every other
program's payload; controller recommends appending only when the payload holds
a composer-relevant record. Oracle for bip48-p2tr measured by two independent
BIP-32 implementations (scratchpad `oracle-bip48-p2tr.txt`; in the plan, Task 5).
Its R0 runs right before its implementer, after S0 ships; Task 6 (payload-spec
fold, controller, own R0) may start earlier.
**NEXT after the implementer:** persist its report → verify its gate output
independently (nextest workspace, clippy, fmt, vectors diff) → whole-diff opus
execution review over `git diff b19dca7b..composer-s0` (report
`composer-S0-exec-review-r0.md`) → fold → sonnet verification → merge to main,
version bump + publish per descriptor-mnemonic `design/RELEASE_PROCESS.md` →
Stage 1 plan. Tasks 1-9 in order
(Task 5 pastes 26 MANIFEST entries from the printer; Task 8 changes
`parse_template_ext` for minting verbs only; Task 9 is the whole-workspace gate
+ corpus regeneration + release note), then the whole-diff opus execution review,
then publish per the repo's release checklist. Not done yet: a LIVE operator walk
of the spec's journeys (the one lens the operator can run). Repo side-commits
this session: descriptor-mnemonic `480e54fe` + `b19dca7b` (follow-up
`md-encode-keyless-template-sigless-path-not-gated`, owned by Task 8).

Original note: usage ran low with R0 round 0 in flight; the operator ruled
"finish what is in flight but don't launch anything new", later lifted.

## State

| artifact | where | commit |
| --- | --- | --- |
| Brainstorm record, 29 operator rulings C1..C29, measurements §3.1-3.11 | `design/BRAINSTORM_wallet_policy_composer.md` | `9100230`, folded `b452a79` |
| SPEC, DRAFT, R0 round 0 dispatched, no PENDING sections | `design/SPEC_wallet_policy_composer.md` | `f68134b`, fold `b452a79` |
| Lowering reviews (fable ×2, operator-directed) | `agent-reports/composer-lowering-rules-bitcoin-expert-review.md`, `composer-lowering-i1-single-key-head-miniscript-review.md` | `e9f2ba0`, `e0375f1` |
| Recon (opus ×2, operator-approved fan-out) | `agent-reports/composer-recon-taproot-multisig-origin-convention.md`; `composer-recon-same-fingerprint-two-accounts-import.md` + `-core-` + `-sparrow-` | `9f55eb6`, `f7f0b27` |
| Follow-ups filed | here: F-448, F-449; descriptor-mnemonic: `md-older-zero-time-units-not-refused` (790fc224), `md-descriptor-depth0-xpub-ledger-registration` (3b0944fb); mnemonic-secret: `ms-derive-taproot-justifications-stale` (5f37b43) | committed in each repo |

Heads at spec time: fork `169073c`, engrave `b452a79`, descriptor-mnemonic `3b0944fb` (md 0.14.0), toolkit `d8f06483`, mnemonic-secret `5f37b43`.

## R0 ROUND 0 FOLDED (`bc1c07c`); ROUND 1 RUN AND FOLDED (see `git log`: reports aa022ae/12abf0f/1630465, fold commit after them; gates structure 0, glyph 87/0, cites 61/61). Lenses run so far: correctness, adversarial, journey x2, coverage, feasibility, fold-verification x1. Round-1 fold verified (44bb06b) and its gaps folded at 99463ac (gates structure 0, glyph 93/0, cites 64/64). Lenses run: correctness, adversarial (on the ORIGINAL §4-§14 only), journey x2, coverage, feasibility, fold-verification x2. Adversarial-on-folds (073bd7f) found 1C/7I, folded at the round-3 fold (see `git log`; gates structure 0, glyph 103/0, cites 68/68). NEXT: sonnet verification of the round-3 fold; if clean, closure (lenses exhausted: correctness, adversarial x2, journey x2, coverage, feasibility, fold-verification x3) and the writing-plans skill: a STAGED plan with Stage 0 (md-codec `compose` + vectors + `md compose`) in full detail.

Each agent writes its own report (the controller never transcribes):

| lens | report path |
| --- | --- |
| correctness + internal consistency: **1C/11I/10M/2N** | `design/agent-reports/composer-spec-R0-r0-correctness.md` |
| adversarial funds safety (counterexamples): **5C/5I/2M/1N** | `design/agent-reports/composer-spec-R0-r0-adversarial.md` |
| operator journey walk (J1 two-path tr, J2 RCW, J3 no-payload template): **6C/10I/8M/2N** | `design/agent-reports/composer-spec-R0-r0-journey.md` |
| coverage + traceability (rulings→spec, rules→acceptance, work items→sections): **2C/8I/11M/6N** | `design/agent-reports/composer-spec-R0-r0-coverage.md` |

Persist commits: journey `4201b56`, coverage `85ec239`, adversarial `1154edc`,
correctness (see `git log`). Controller spot-checks held on every top finding
(consent copy for non-renderable shapes; single-stub re-mint vs seating; sh(wsh)
at script_type 1'; the 1985-11-05 midnight boundary encodes as a HEIGHT; wsh
summarised as one branch; `sysw.Classify` vs `seal.Classify`; single-leaf `{P1}`).
Recurring themes across lenses: the consent surface cannot state a composed shape;
the stub re-mint must carry BOTH stubs (C9); the origin table row for sh(wsh);
no acceptance item for refusals or lock ranges; date floor missing.

**Fold done 2026-09-01.** Controller defaults taken in the fold are listed in the
brainstorm record section 3.12 for the operator's veto; Minors/Nits from the four
reports are recorded there and NOT folded. **On resume, when launches are allowed:**
(1) re-review the fold — sonnet: did the fold fix each C/I and introduce nothing
(compare `git diff 80e6a72..bc1c07c -- design/SPEC_wallet_policy_composer.md`
against the four reports); opus: one NEW lens not yet run (e.g. implementation
feasibility of the Go builder + `md compose` against md-codec admission, or a
second journey walk on the regenerated §7); (2) fold, gate, commit; (3) then fold
the Minors/Nits; (4) the writing-plans skill. Original resume text follows.

(1) read all four persisted reports
(never the JSONL task output); dedupe findings across lenses; (2) fold all four into the spec together; run the gates and put
their output in the fold commit message:

    scripts/spec-structure-check.sh design/SPEC_wallet_policy_composer.md
    scripts/plan-glyph-check.sh     design/SPEC_wallet_policy_composer.md
    CITE_FORK_ROOT=/scratch/code/shibboleth/seedhammer scripts/plan-cite-check.sh design/SPEC_wallet_policy_composer.md

(`scripts/spec-check.py` is the systemwide-payload spec's own gate; its
remaining failures do not apply.) (3) Re-review only what the fold changed
(non-trivial folds), sonnet for fold-verification, opus for a new lens; a clean
round closes the LENS, not the spec — enumerate remaining lenses before closing.
(4) Then `superpowers:writing-plans` for the plan; implementation UC OFF, one
implementer; Rust first (md-codec `compose` + vectors) before the Go builder.

## Rulings that shape everything (verbatim in the brainstorm §2)

C1 spend-path list; C2 archetypes as presets; C3 firmware vocabulary, Rust-first;
C4 template first; C5 one slot one path, reuse the master via hardened accounts;
C6 Build inside Wallet Policy (door becomes a choice in every state); C7 Multisig
Build deprecated by COMMENT only; C8 payload-first seating (pick list, "remaining"
keys); C9 teach the mk1 stub unconditionally; C10 engraved FORM is the operator's
choice (concrete text/QR/keyed md1, or template + mk1); C11 timelock UI respects
ranges; C12 seeds (words/ms1, unsealed payload or typed) are a key source; C13
Full/Watch-only, secret as words/SeedQR/ms1; C14 no Sealed-Payload memory
treatment; C15 lowering lives in md-codec `compose` / `md compose`; C16 grammar
bounds; C17 pkh in wsh, pk in tr (F-448); C18 raw-H NUMS this cycle (F-449); C19-
C23 review findings (first-appearance numbering, or_i(pkh) head, or_d for bare
multi head, keyless wsh-only, lock-only refused, ≥1 keyed path, first-listed
single key as internal key, lock ranges sourced); C24 pack time via `now:` record
= lower bound; C25 entry UX (digit pad; kind→unit→digits→echo; hash: record);
C26 Build with no payload = keyless template; C27 recon fan-out (3 deferred);
C28 taproot seed-derived origin `m/48'/coin'/account'/3'`; C29 same seed twice in
ONE path = warning.

## Lessons this session recorded (also in memory)

- Line counts are not a fold check: five rulings silently failed to land because
  an anchor differed by one character; grep the inserted row afterwards.
- A regtest node rejects mainnet xpubs: a "refusal" can be the network, not the
  descriptor. Match the agent's environment before calling its claim wrong.
- The compiler is a validity oracle, not a byte oracle; byte identity with the
  toolkit archetypes is impossible under any uniform rule.

## State at 2026-09-02 (S1 implemented, S2 plan drafted)

- **S1 implemented** in worktrees `wt-composer-s1` (`composer-s1`, 5 commits off
  59e6f12) and `wt-ms-bip48-p2tr` (`bip48-p2tr`, 1 commit off 5f37b43). Report
  persisted 97275c4. Controller re-ran independently: engrave fmt/clippy clean,
  nextest 621/621, threaded `cargo test` all ok; ms 477/477. Whole-diff opus
  execution review DISPATCHED (report → `composer-S1-exec-review-r0.md`).
  THEN: persist → fold → sonnet verification → merge both via each repo's
  `push-via-staging.sh` → release `me` 0.8.0 / `ms` 0.17.0.
- **S2 plan drafted and build-gated** (`IMPLEMENTATION_PLAN_composer_S2_fork_codec.md`,
  b95df91 + glyph fold): 9 tasks. New gate `scripts/plan-build-gate-go.sh`
  (4659452) extracts the anchored Go into a scratch copy of the fork at
  `/scratch/code/shibboleth/.plan-build-gate-go/seedhammer`; fragments of
  existing files are hand-wired by `scratchpad/handwire_s2.py`. Measured with
  fragments wired: `go test ./md/ ./mk/ ./sysw/` ok, 73 new sub-tests pass, all
  28 family vectors byte- and chunk-identical (incl. the two no-corpus chunk
  sets produced by `md compose`/`md encode` at 66bdf2f4), 5 pkh vectors' P2WSH
  addresses equal Rust's, 40/40 record-class rows lockstep, keyed conformance
  gate 14→36 sub-tests. Two latent LOADER bugs found by the gate (hex-string
  pubkeys and hash bodies read as base64) → plan Task 2 Step 2a.
  R0 round 0 DISPATCHED (fidelity opus + tests sonnet →
  `composer-S2-plan-R0-r0-{fidelity,tests}.md`). GREEN expires: re-validate
  immediately before dispatching the S2 implementer (after S1 ships).
- Transcription lesson re-learned: one of the two no-corpus chunk strings was
  mis-pasted (a 5-char group dropped); the machine-generated file was right.
  Read literals from files, never from a terminal echo.

## State at 2026-09-02, later (both reviews folded, verifications running)

- **S1**: opus whole-diff review `composer-S1-exec-review-r0` (0C/1I/3M/5N)
  persisted 6183d94; folded on `composer-s1` at 5720e3c (spec 4.4 cell; five
  fixture rows → 45 rows, sha `eed6b177d1a3406a69c4a0102635f5d59c6412fa65e106f85b831c4736ac464e`;
  suffix detail; rename; changelog); gates re-run green (fmt, clippy,
  nextest 622/622, threaded 622/0, propagation clean). Records on master:
  S1 plan pin + F-451/F-452/F-453 (d36d7f4). Sonnet fold verification →
  `composer-S1-exec-review-r1-fold-verification.md` DISPATCHED. THEN: bump
  me 0.8.0 (`crates/me-cli/Cargo.toml`, CHANGELOG `## [0.8.0] - date`) and
  ms-cli 0.17.0 (`crates/ms-cli/Cargo.toml`, CHANGELOG `## ms-cli [0.17.0] — date`)
  on the branches → ff-merge → `scripts/push-via-staging.sh` per repo (sonnet
  push agent) → annotated tags `v0.8.0` / `ms-cli-v0.17.0` on the pushed SHAs.
- **S2 plan**: R0 r0 reports persisted (df78058 fidelity, a0bf11a tests);
  fold e2dc7e4 (typed `Locks []Lock` + `Sorted` on `Branch`, `lockFromWire`,
  Task 6 precondition + 45-row pin, shipped or_* cards pinned, change chain,
  dir-scanning pin test, aliasing probe, digit-count test); gate green in the
  hand-wired scratch (80 new sub-tests). Sonnet r1 verification →
  `composer-S2-plan-R0-r1-fold-verification.md` DISPATCHED. A clean r1 closes
  R0; re-validate ("what did S1's merge falsify?") immediately before
  dispatching the S2 implementer, with Task 6's precondition satisfied.

## S1 SHIPPED 2026-09-02

- mnemonic-engrave `master` 38e3ed13eb0d903ae2d24e64edc830a9484dcc6e pushed via
  `ci/staging` (required check green, no bypass); tag `v0.8.0`; `assemble +
  sign + release` green, release published with 7 assets:
  https://github.com/bg002h/mnemonic-engrave/releases/tag/v0.8.0
- mnemonic-secret `master` 1068f389116928e4cd22e5b0658749d09b06611d pushed via
  the hand ritual (four required contexts green, no bypass); tag
  `ms-cli-v0.17.0`; release published (`ms-man.tar.gz`) but `man-release.yml`'s
  reproducibility jobs (`repro-substrate`, `repro-x86_64-musl`; NOT required
  contexts) are RED: the repro Docker image's vendored cache predates the
  `mnemonic-io-lib` git pin 6c24e628 and `cargo build --offline` cannot fetch
  it; the musl-binary jobs were skipped in consequence. Filed in
  mnemonic-secret `design/FOLLOWUPS.md` (see that repo); not this cycle's code.
- Report: `composer-S1-push-report.md` (e155ca4). Worktrees `wt-composer-s1`
  and `wt-ms-bip48-p2tr` removed (branches `composer-s1`, `bip48-p2tr` kept).
- **S2 implementer running** (opus) in fork worktree `wt-composer-s2`, plan
  38e3ed1; **S3 GUI recon** (sonnet, read-only) running → `composer-S3-recon-gui.md`.

## In flight at 2026-09-02 (late)

- **S2 implementer** (opus) in `wt-composer-s2` → `composer-S2-implementation-report.md`.
  THEN: persist → re-run gates in the worktree (vet, go test ./md/ ./mk/ ./sysw/,
  gui -run TestComposer, test-32bit, oraclelive build, js vet, gofmt, firmware
  size) → opus whole-diff execution review → fold → sonnet verification →
  merge into the fork main → fork push (its own CI: test.yml) → flash? (the
  operator's call; S2 has no screens, nothing to see on device).
- **F-324 fix** (sonnet) in mnemonic-toolkit worktree `wt-toolkit-f324` →
  `f324-toolkit-git-source-report.md`. THEN: review diff → push toolkit master via
  its staging ritual (contexts `examples`, `test (ubuntu-latest)`, `clippy`) →
  re-pin `toolkit_ref` in ms `man-release.yml` (+ pass `git_source_url`/`git_source_rev`
  from Cargo.lock) → ms push via staging → `workflow_dispatch` the gate once →
  `gh run rerun 33621228397 --repo bg002h/mnemonic-secret --failed` to publish the
  0.17.0 musl binaries.
- **S3 plan author** (opus) → `design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`
  + `composer-S3-plan-author-report.md`. THEN: build gate (now also runs
  `gui -run ^TestComposer`), cite/glyph/stepref checks, R0 lenses (fidelity opus,
  tests sonnet, + a JOURNEY-WALK lens: the operator is not present, so an agent
  walks §7's journey against the plan; the live walk with the operator remains
  owed), fold, verification, then implementer after S2 ships.

## S2 implemented + folded, review dispatched (2026-09-02)

- Implementer report persisted ae70d04 (tasks 1-8 green; Task 9 RED on three
  pre-existing gui tests). Controller fold on `composer-s2` = 489d52e: the
  `v:multi_a` verify-fold defect (pre-existing, WRONG taproot address for a
  verify-wrapped multi_a leaf; Go-only convergence fix), the pk_h tripwire
  converted to a positive test, the consent-absence test re-aimed at a new
  `gap_wsh_andor` fixture. Gates after the fold: md/mk/sysw ok, gui 1059/1059
  (24 shards, 38 s), 32-bit ok, oraclelive ok, js vet ok, gofmt clean. Plan
  record folded on master 9f011ae (second address glob, Step 2a order,
  `-mod=readonly`, gofmt baseline nit, Task 9 note).
- Opus whole-diff execution review DISPATCHED → `composer-S2-exec-review-r0.md`.
  THEN: persist → fold → sonnet verification → `git merge --no-ff composer-s2`
  into the fork `main` → push (fork main is unprotected; CI test.yml runs on
  push; watch it) → firmware size already measured +3,168 B flash (1,506,820 B),
  RAM unchanged; flashing is the operator's call (no screens in S2).
- F-324 close agent (sonnet) DISPATCHED: toolkit push via staging → ms re-pin
  (`pins` job derives the io-lib rev) → ms push via staging → `workflow_dispatch`
  gate → conclusion on the 0.17.0 binaries (likely needs a 0.17.1 tag; the
  controller decides) → `f324-close-report.md`.

## S2 review loop CLOSED; merge dispatched (2026-09-02)

- r1 fold verification (sonnet): 6/7 VERIFIED, V-1 = README note placement,
  folded 7a4eeb5 (doc only). `composer-s2` tip 7a4eeb572ed9ea6a7fda0d6e0201a5df29a61fe8.
  Sonnet merge+push agent dispatched (brief scratchpad/push-brief-s2.md, merge
  message scratchpad/s2-merge.msg) → `composer-S2-push-report.md`. Fork main is
  unprotected: merge --no-ff, push, watch test.yml.
- S3 plan draft 2e61a98 + author report 236b1e7; mechanical checks clean (222/222
  cites vs the S2 worktree, glyph 0, table 0, stepref 0); gate: new files compile,
  md/mk/sysw ok, gui composer tests red until the six fragments are wired -- the
  author is writing scratchpad/handwire_s3.py. THEN: green gate run → R0 lenses
  (fidelity opus, tests sonnet, journey opus; briefs in scratchpad) → fold →
  verification → implementer after S2 merges. Three operator questions in the
  author report §5 (Part A ships alone? §7f two forms (F-455)? presets wait on
  F-453?) -- surfaced to the operator; defaults if silent: yes / two forms /
  blank-shape first.

## S3 plan gated green; R0 round 0 dispatched (2026-09-02)

- Author's `scratchpad/handwire_s3.py <scratch-root>` applies the plan's six
  fragments + four shipped-test updates (10 files; refuses to run twice). Wired
  run: composer tests 118 sub-tests ok, whole gui 1125/1125, md/mk/sysw ok.
  The script surfaced one plan defect (the 8e comment wrapped across two lines,
  failing its own coverage test) -- fixed in the plan by the author; that hunk
  rode the STATUS commit 39f381b (an author self-fold before any review).
- R0 round 0 lenses DISPATCHED: fidelity (opus) → `composer-S3-plan-R0-r0-fidelity.md`,
  tests (sonnet) → `…-tests.md`, journey (opus) → `…-journey.md`. Briefs in
  scratchpad (`fidelity-lens-brief-s3.md`, `tests-lens-brief-s3.md`,
  `journey-lens-brief-s3.md`). Scratch: `.plan-build-gate-go-s3/{seedhammer,wired}`.
- Engrave master pushed 88b4a4aa via staging (report persisted). Fork merge of
  `composer-s2` (7a4eeb5) in progress → `composer-S2-push-report.md`.

## S2 SHIPPED (2026-09-02)

- Fork `main` 321acb56f74ff60e81abcfa511b2013f3aeb0abc = merge --no-ff of
  `composer-s2` (7a4eeb5); CI runs Test (tests + tinygo-device-build) and Build
  image all success; origin/main confirmed. Report `composer-S2-push-report.md`
  (e44fa12). Staged plan S2 marked SHIPPED (0b9a478). Worktree removed.
- S3 gate scratch copies (`.plan-build-gate-go-s3/{seedhammer,wired}`) were made
  from the S2 worktree; future gate runs use the default FORK_REPO (fork main now
  carries S2). At the r0 fold, re-baseline the S3 plan on fork main 321acb56.

## S3 R0 r0: journey lens landed (2026-09-02)

- `composer-S3-plan-R0-r0-journey.md` (opus): 2C/6I/6M/4N (7501564). C-1: Part B's
  seating half is never joined to any flow (14 dead functions; plan line 4437
  promises a `Replace gui/composer_flow.go` no task supplies) -- the
  "can-a-user-do-the-thing" class. C-2: the Key-order question is asked while the
  path is sole; adding a second path silently lowers sortedmulti to multi.
- Controller's own structural finding: Part B tasks are compressed (no
  Run/Expected; B8/B10 no steps). The author agent is preparing the expansion
  in scratchpad/s3-partB-expansion.md. Fold plan: wait for fidelity + tests
  lenses, then ONE author-driven fold (expansion + wiring + all findings),
  re-gate wired, r1 verification lenses.
- Engrave master pushed 67ffa3e1 (report 9a7939f).
- Fidelity lens landed: `composer-S3-plan-R0-r0-fidelity.md` (opus) 1C/12I/11M/3N
  (7195a7c); C-1 = the same unjoined Part B. Forwarded to the author for the
  combined fold; tests lens still running.
- Gate hardened (2f94f0d): `plan-build-gate-go.sh` step 8 counts DEAD-IN-PROD
  functions among the plan's new production files (comments stripped); proven
  on the wired S3 scratch: exactly the 14 the two lenses named. Prints, does not
  fail (a later-stage API is a legitimate hit); the reviewer decides.
- F-453 (presets, Rust first) mini-plan author DISPATCHED (sonnet) →
  `design/IMPLEMENTATION_PLAN_composer_S0b_presets.md` + `composer-S0b-plan-author-report.md`.
  THEN: plan-build-gate-md.sh, R0 (fidelity opus + tests sonnet), fold, implementer
  in a descriptor-mnemonic worktree, whole-diff review, push via staging, then
  re-vendor into the fork (`scripts/vendor-compose-vectors.sh`, 126 → 126+N files)
  so S3's preset task (A10) unblocks.
- Tests lens landed: `composer-S3-plan-R0-r0-tests.md` (sonnet) 15C/1I -- 19 of 27
  mutations NOT caught, two test files missing; "does not close under the tests
  lens". Forwarded to the author for the combined fold. All three r0 reports are
  in; the fold is ONE author-driven rewrite (Part B expansion + wiring + every
  finding), then a fresh wired gate (DEAD-IN-PROD must read 0), then the r1
  verification (brief: scratchpad/verify-brief-s3-r1.md).

## S3 R0 r0 fold applied by the author (2026-09-02, uncommitted until the controller's gate re-run)

- The author applied the combined fold to the plan in the working tree: Part B
  expanded to the step standard; Task B11 joins Part B to the flow; a
  flow-level walk from a keyed payload to the engrave screen; every mutation
  survivor's test replaced; three declines with citations (journey C-2 second
  shape; fidelity I-10 → F-457; fidelity N-1 → spec fold); F-456 (date-ceiling
  body) and F-457 (text/QR plates need a Rust-first renderer; form A = keyed
  md1 this stage, C10 narrowed -- operator's call) filed on master (c418930).
  Author-reported gate: DEAD-IN-PROD gui = 1 (`composerDescriptorCeilingChars`,
  justified), composer tests 100 top-level / 81 sub-tests, gui 1158/1158, cites
  238/238, staleness 0 vs 321acb56.
- Controller's independent re-gate RUNNING (log scratchpad/s3-gate-r1.log).
  THEN: commit the fold (message = gate output) + the report update; dispatch
  r1 verification (brief scratchpad/verify-brief-s3-r1.md; PRE_FOLD_SHA = 39f381b).
- /tmp is a 32 GB tmpfs at ~76%: the session's scratchpad build cache
  (649 MB) was removed; run sharded gui tests with TMPDIR on /scratch.
- Fold COMMITTED 3820a6a (message = the controller's independent re-gate: cite
  238/238, staleness 0, glyph 0, tables 89/0, composer 181 PASS lines, gui
  1158/1158, DEAD-IN-PROD 1 justified); author report update 5fbc5d8. r1 fold
  verification (sonnet) DISPATCHED → `composer-S3-plan-R0-r1-fold-verification.md`.
  A clean r1 closes R0 for S3 (lens-closure: fidelity, tests/mutation, journey,
  fold verification -- the same set S2 closed on plus the journey walk).
  Still owed to the operator before implementation: the three §5 questions
  (defaults stand if silent) and F-457's narrowing of C10.
- S0b (F-453 presets) plan DRAFT 96dfff7 (sonnet author; report aaf13cc): 3 tasks,
  `--preset <name>[,<k>of<n>]*[,<param>=<value>]*` over the six shipped
  constructors, six `keyed_compose_preset_*` vectors. Controller's md gate
  RUNNING (log scratchpad/s0b-gate-r0.log); the author warns step 6 halts on
  main.rs's un-wired dispatch arm (run() gains a `preset` parameter) -- hand-wire
  the main.rs fragment, re-run, then R0 (fidelity opus, tests sonnet).
- S0b plan: mechanical fixes b1a3225 (five bare-file cites qualified → 23/23;
  the clap ArgGroup row reworded; glyph 0; tables 19/0). md gate halted at
  step 6 as the author predicted (main.rs arm is a fragment); controller is
  hand-wiring main.rs (scratchpad/handwire_s0b.py) + the MANIFEST fragment
  into /tmp/plan-build-gate-md, exporting the six vectors and running the
  compose suites (log scratchpad/s0b-gate-wired.log). THEN: R0 (fidelity opus
  + tests sonnet; briefs in scratchpad), fold, verification, implementer in a
  descriptor-mnemonic worktree, whole-diff review, staging push, re-vendor.
- S0b wired gate GREEN (scratchpad/s0b-gate-wired.log): main.rs + MANIFEST
  fragments wired, md-codec compose 52/52, md-cli compose 23/23, clippy + fmt
  clean; the vector export + full suites running (s0b-export.log). Fidelity
  lens (opus) DISPATCHED → `composer-S0b-plan-R0-r0-fidelity.md`; tests lens
  (sonnet) next, after the export lands.
- tmpfs hygiene: the session's stale scratch build targets (11 GB: f453-probe,
  ms-gate-target, gate-target, lianaprobe, mscheck, liana-src) removed; /tmp
  back to 58%. `/tmp/plan-build-gate-md-target` (6.3 GB) stays until the S0b
  gate rounds end; future gate runs should set TMPDIR=/scratch/code/shibboleth/.tmp.
- S0b export in the wired scratch: 6 conformance files / 30 files for the six
  `keyed_compose_preset_*` vectors; md-cli full suite 775/775; md-codec 518/519
  -- the one red, `display_grouping_conformance::conformance_vectors_pass`, is a
  GATE ARTEFACT (the md gate's copy omits `design/`, which that test reads via
  `../../design/display-grouping-vectors.tsv`); it passes in the real checkout
  and CI on main 66bdf2f4 is green. Tests lens (sonnet) DISPATCHED →
  `composer-S0b-plan-R0-r0-tests.md`. Both S0b lenses running.
- S0b fidelity lens landed: `composer-S0b-plan-R0-r0-fidelity.md` (opus) 0C/4I/4M/2N
  (32ba3f1): all four Importants are cross-plan statement gaps with S3 task A10
  (defaults = the device's shape? wrapper coverage per archetype; refusal table
  subset; fork pin-test ownership). Fold both S0b lenses into S0b AND a small
  A10 fold in S3 -- after S3's r1 verification lands (S3 must not move under
  its reviewer).
- S0b tests lens landed: `composer-S0b-plan-R0-r0-tests.md` (sonnet) 0C/0I/0M/2N,
  12/12 mutations caught (8a0f8bb). Both S0b lenses forwarded to the S0b author
  (sonnet, resumed) who is folding per scratchpad/fold-brief-s0b-r0.md; then
  the controller commits the fold with the gate output and dispatches the r1
  verification (brief scratchpad/verify-brief-s0b-r1.md). Master pushed 3611ca25.
- S0b r0 fold APPLIED by the author (uncommitted): all fidelity findings folded
  (N-2 declined: additive --json field, schema never bumped for that), both
  tests-lens nits folded; author's gate: workspace nextest 1340/1340, md-cli
  compose 31/31, md-codec compose 52/52, corpus 156 files, cite 25/25.
  Controller's independent re-gate RUNNING on /scratch
  (scratchpad/s0b-gate-r1.log; TMPDIR=/scratch/code/shibboleth/.tmp). THEN:
  commit fold (message scratchpad/s0b-fold-r0.msg + gate lines) → r1
  verification (brief scratchpad/verify-brief-s0b-r1.md) → implementer in a
  descriptor-mnemonic worktree → whole-diff review → staging push → re-vendor
  (S3 A10: scratchpad/s3-a10-fold-notes.md).
- tmpfs: old /tmp md gate scratch + 6.3 GB target removed; /tmp at 39%.
- S0b fold COMMITTED 6c308b6 (controller re-gate in the message: cite 25/25,
  md-codec compose 52/52, md-cli compose 31/31, workspace 1302/1302 in the
  scratch vs the plan's Expected 1340 -- r1 reconciles). r1 verification
  (sonnet) DISPATCHED → `composer-S0b-plan-R0-r1-fold-verification.md`. A clean
  r1 closes S0b's R0 (lenses: fidelity, tests/mutation, fold verification);
  then: implementer (single, descriptor-mnemonic worktree, UC off) → whole-diff
  review → staging push (contexts `cargo test (ubuntu-latest)` + `cargo clippy`)
  → re-vendor into the fork (S3 A10).
- S3 r1 fold verification landed (8aeb6a7): NOT GREEN -- both r0 Criticals
  VERIFIED, 12/15 tests-lens Criticals closed, but the fold overwrote Task
  A11's fence with B11's joined body (Part A no longer builds alone: NEW
  Critical), Task C2's counts went stale (NEW Important), fidelity I-2's fix
  unreachable, journey I-6's fix a `u == 0` tautology (F-458 filed), B5/B6/B9
  lack closing steps, six Importants unguarded by tests, two stale Produces
  lines, N-1's destination missing. Round-1 fold DISPATCHED to the S3 author
  with the controller's decisions (incl. a Part-A-only build step in A11).
- Gate: `GATE_UNTIL='^### Task B1'` mode added to plan-build-gate-go.sh; on the
  current S3 plan it reproduces the r1 Critical (`undefined: composerKeySources`
  in Part A's composer_flow.go). The S3 author was told to make Task A11's
  Part-A-only gate step use it. Scratch for that mode:
  `.plan-build-gate-go-s3-partA`.
- S0b r1 verification landed (1e96196): 0C/1I/1M -- I-1..I-4, M-1..M-3 VERIFIED
  (mutations live); the Important is M-4's tautological test (a 7th unmatched
  PRESET_NAMES entry passes every test and panics `unreachable!()` at runtime);
  the plan's 1340 workspace count is right (the fold commit's 1302 was
  transient). Round-1 fold DISPATCHED to the S0b author (iterate PRESET_NAMES;
  no `unreachable!` -- a CliError instead). THEN: commit + r2 verification
  (brief scratchpad/verify-brief-s0b-r2.md, targeted) → implementer.
- S3 round-1 fold APPLIED by the author (uncommitted): A11 self-contained again
  + a Part-A-only gate step (GATE_UNTIL + `handwire_s3.py --part-a`), which
  found composer_discard.go mis-assigned (B3 → A5); I-2 wired; F-458 fixed;
  C2 counts; Produces lines; N-1 destination; B5/B6/B9 steps; guards for the
  six unguarded Importants and C-9/C-12/6b/8d; C-15 moot. Author's gate:
  Part A 47 PASS with Part B absent; whole 110/95, gui 1168/1168, DEAD 1,
  cite 241/241, staleness 0. Controller's independent re-gate RUNNING
  (scratchpad/s3-gate-r2.log). THEN: commit (message scratchpad/s3-fold-r1.msg)
  → r2 verification (brief scratchpad/verify-brief-s3-r2.md).
- S3 round-1 fold COMMITTED (controller re-gate in the message: Part A alone
  89 PASS lines; whole 205; gui 1168/1168; DEAD-IN-PROD 1 justified; cite
  241/241; staleness 0). r2 verification (sonnet) dispatched next
  (brief scratchpad/verify-brief-s3-r2.md).
- S0b round-1 fold APPLIED by the author (uncommitted): the PRESET_NAMES drift
  test is a unit test that iterates the list (a 7th unmatched name fails it);
  `unreachable!()` → CliError (phantom preset exits 1); M-3 wording. Author's
  gate: md-codec compose 52/52, md-cli compose 31/31 (+1 unit test), workspace
  1340/1340, corpus 156, cite 25/25. Controller re-gate RUNNING
  (scratchpad/s0b-gate-r2.log). THEN: commit (scratchpad/s0b-fold-r1.msg) → r2
  verification (scratchpad/verify-brief-s0b-r2.md, targeted) → implementer
  (scratchpad/implementer-brief-s0b.md).
- S0b round-1 fold COMMITTED (controller re-gate in the message; the
  1302-vs-1340 count is the --all-features flag). r2 verification (sonnet,
  targeted) dispatched next.
- S0b r2 verification (a209572): 0C/1I -- one table row with the superseded
  "is read as" wording. Folded by the controller as a one-word change,
  propagation-checked (0 sites), STATUS R0 GREEN. Baseline descriptor-mnemonic
  main 66bdf2f4 unmoved. NEXT: dispatch the S0b implementer (opus, worktree
  wt-composer-s0b, brief scratchpad/implementer-brief-s0b.md), then whole-diff
  review (scratchpad/review-brief-s0b.md), staging push, re-vendor (S3 A10).
- S3 r2 verification (d407109): NOT GREEN -- A11 standalone VERIFIED (independent
  Part-A-only gate), I-2/C-12/6b/8d VERIFIED, but three claimed guards cannot
  fail their own named mutation (F-458's guard never calls composerLockEdit; the
  hex-bound guard uses 63 chars so the decoder masks the bound; C-9 rests on a
  standalone test not exercising composerFlow) + one new Minor (Task C1).
  Round-2 fold DISPATCHED to the S3 author: drive the real function/screen,
  paste the failing mutation output into Expected. THEN: controller re-gate →
  commit → r3 targeted verification (brief scratchpad/verify-brief-s3-r3.md).
- S3 round-2 fold APPLIED by the author (uncommitted): the three guards now
  drive the real surface with the failing mutation output pasted into
  Expected; Task C0 makes "mutate, see it fail, revert" an explicit step; C1
  template; I-5 recorded as a structural no-op. Author's gate: whole 112/100,
  gui 1170/1170, DEAD 1, Part A alone 47 PASS, cite 241/241, staleness 0.
  Controller re-gate RUNNING (scratchpad/s3-gate-r3.log). THEN: commit
  (scratchpad/s3-fold-r2.msg) → r3 targeted verification
  (scratchpad/verify-brief-s3-r3.md).
- S0b IMPLEMENTED: `composer-s0b` (wt-composer-s0b) = 4793619b (vectors), 5002ebac
  (--preset), 87bc10ff (corpus + notes) over 66bdf2f4; report d3d31ff (workspace
  1340/1340; +30 files / 6 conformance; two Expected-line deviations, both
  wording). Controller's independent CI-form gates RUNNING
  (scratchpad/s0b-impl-gate.log). THEN: opus whole-diff review
  (scratchpad/review-brief-s0b.md) → fold → sonnet verification → ff-merge into
  descriptor-mnemonic main → `scripts/push-via-staging.sh main` (contexts
  `cargo test (ubuntu-latest)` + `cargo clippy`; no tag, no publish) → re-vendor
  into the fork (S3 A10; 126 → 156 files, 26 → 32 names).
- S3 round-2 fold COMMITTED (controller re-gate in the message: Part A alone 89,
  whole 212, gui 1170/1170, DEAD 1, cite 241/241, staleness 0). r3 targeted
  verification (sonnet) dispatched next (scratchpad/verify-brief-s3-r3.md).
- S0b implementation gate GREEN in wt-composer-s0b (fmt/clippy/nextest 1340/
  threaded 1340/checksum/30 preset corpus files/probes). Opus whole-diff review
  dispatched (scratchpad/review-brief-s0b.md -> design/agent-reports/
  composer-S0b-exec-review-r0.md). S3 r3 verification still running.
- S3 plan R0 GREEN: r3 (sonnet, targeted) 0C/0I persisted 3c8254f; STATUS set
  9081222. Task A10 fold (opus, scratchpad/fold-brief-s3-a10.md, facts settled
  by the controller: six keyed_compose_preset_* vectors, five wsh + kofn tr,
  vendoring glob already covers them, pin test 26->32 names / 126->156 files)
  IN PROGRESS; then sonnet verification (scratchpad/verify-brief-s3-a10.md,
  <A10_FOLD_SHA>), then the implementer waits for S0b to ship (A10 unblocked).
  S0b whole-diff review (opus) still running.
- PROCESS EXIT mid-session wiped the scratchpad (all briefs, handwire_*.py,
  gate logs). A10 fold had completed: report c677c6d, fold 0051be7 (checks in
  its message). Briefs now live in design/agent-briefs/ (recreated from
  context: S3 implementer, S0b push, A10 notes + verification brief). The S0b
  reviewer was resumed from its transcript with the brief resent inline.
  NEXT: A10 verification (sonnet) -> fill <S3_GREEN_SHA>; S0b review report ->
  fold -> verify -> push (<S0B_TIP>) -> record the S0b merge commit in A10 ->
  dispatch the S3 implementer. The hand-wire scripts are lost; S3's plan gate
  needs none further (A10 carries no go fence).
- A10 fold VERIFIED (sonnet 0C/0I/0M/0N, report 084d7da). S0b whole-diff review
  0C/0I/4M/4N (report 1736aca): M-1/M-2/M-3/N-3 folded inline in
  wt-composer-s0b (uncommitted; gate running to .tmp/s0b-fold-gate.log), N-1/
  N-2/N-4 filed F-459 (cffdccd), M-4 already closed by 0051be7. NEXT: commit
  the S0b fold -> sonnet verification (design/agent-briefs/
  composer-S0b-fold-verification-brief.md, <S0B_FOLD_SHA>) -> push
  (composer-S0b-push-brief.md, <S0B_TIP>) -> record the merge commit in S3 A10
  (that commit is <S3_GREEN_SHA>) -> dispatch the S3 implementer.

## RESUME POINT 2026-09-02 (context cleared here; resume with /resume-composer)

State, all measured:
- mnemonic-engrave master: S3 plan R0 GREEN (STATUS 9081222), Task A10 fold
  0051be7 verified 0C/0I/0M/0N (084d7da). Briefs live in design/agent-briefs/.
  master pushed through 1c7aac4; commits after it are records only.
- descriptor-mnemonic: branch `composer-s0b` in worktree
  /scratch/code/shibboleth/wt-composer-s0b, tip 1dc8d40 = whole-diff review
  fold (M-1/M-2/M-3/N-3) on 87bc10ff (three implementer commits). Gate on the
  fold: fmt/clippy/doc exit 0; nextest 1342/1342; threaded 1342/0; cli_compose*
  32; compose_* 52. Review 0C/0I/4M/4N (report 1736aca); F-459 filed (cffdccd)
  for N-1/N-2/N-4; M-4 closed by 0051be7. main = 66bdf2f4, untouched.
- seedhammer fork main 321acb56 (S2). No S3 code exists yet.
- S0b fold verification (sonnet): 0C/0I/0M/0N (report 361369b); the pre-fold cli_compose count was 30, not 31 -- the brief's figure was stale, the fold added two tests -> 32. Step 1 below is therefore DONE; start at step 2.

Steps, in order:
1. If the verification report (design/agent-reports/
   composer-S0b-exec-review-r1-fold-verification.md) is not 0C/0I: fold on
   composer-s0b, re-run the gate in the fold commit's form, re-verify.
2. PUSH S0b: fill <S0B_TIP> (= `git -C /scratch/code/shibboleth/wt-composer-s0b
   rev-parse HEAD`) in design/agent-briefs/composer-S0b-push-brief.md; dispatch
   a sonnet push agent with it (ff-merge composer-s0b into main, then
   scripts/push-via-staging.sh main; FREEZE main during the window; report to
   design/agent-reports/composer-S0b-push-report.md). Verify origin/main
   yourself; persist the report; note the merge commit.
3. RECORD the S0b merge commit in S3 Task A10 (the blockquote that says the
   controller records it; also FOLLOWUPS F-453 -> CLOSED with the commit).
   One-line fold; plan checks (cite/glyph/table/stepref) in the message. That
   commit is <S3_GREEN_SHA>.
4. DISPATCH the S3 implementer (opus; UC off; ONE agent):
   design/agent-briefs/composer-S3-implementer-brief.md with <S3_GREEN_SHA>
   filled and the dispatch message stating "S0b has shipped at <merge commit>;
   Task A10 is unblocked". Worktree /scratch/code/shibboleth/wt-composer-s3,
   branch composer-s3. Expect hours; it reports to
   design/agent-reports/composer-S3-implementation-report.md.
5. After the implementer: persist its report; controller re-runs the fork
   gates (scripts/gui-shard-test.sh ./gui/ 24, ./md ./mk ./sysw, gofmt, vet,
   test-32bit.sh, build-firmware size line) on the worktree; then an opus
   whole-diff execution review (brief to write in design/agent-briefs/, same
   shape as composer-S2's: counterexamples, mutation-test the tests, what the
   diff made false elsewhere, CI gates as CI runs them); fold; sonnet verify;
   merge --no-ff into fork main, push, watch test.yml; flash via
   ~/bin/sh/sh2-flash only when the operator says so.
6. Engrave master: push the record commits via a sonnet push agent
   (scripts/push-via-staging.sh master; freeze) whenever the tree is clean and
   no commit is imminent.
7. Owed to the operator (surface, do not block): three S3 defaults stand if
   silent (Part A ships alone; §7f offers the device's two plate forms
   (F-455); presets follow F-453's Rust half); F-457 narrows C10 this stage;
   me 0.8.1 owed (F-454); the live journey walk with the operator.

- 2026-09-02 (after the resume point): step 2 DONE -- S0b on descriptor-
  mnemonic main 1dc8d409 (ff; run 33698441737 green; report 1b3c159). Step 3
  DONE -- A10 records the merge commit, F-453 CLOSED (plan revision 722edbd
  = <S3_GREEN_SHA>; brief 3725e7f). Step 4 IN PROGRESS -- S3 implementer
  (opus, one agent) dispatched against 722edbd with A10 unblocked; worktree
  /scratch/code/shibboleth/wt-composer-s3, branch composer-s3; reports to
  design/agent-reports/composer-S3-implementation-report.md. Step 6 next
  while it runs (engrave master push), then step 5 when it reports.
- S3 IMPLEMENTED: composer-s3 tip b300a84 (25 commits; report d409c37). Two
  facts from the run: (1) /nix is GONE from this machine -- Go 1.26.7 restored
  at /scratch/code/shibboleth/.toolchain/go/bin/go (sha verified); TinyGo and
  `nix run .#build-firmware` unavailable, so Task C2 Step 4 (firmware size
  delta vs 1,503,652 B flash / 62,592 B RAM at 169073c) is UNRUN -- the plan
  cannot close and nothing may be flashed until Nix is reinstalled (operator).
  (2) CI's go test ./... failed two cmd/emu needle tests: the B11 fence titled
  the composer census "Plate Count" (the Build walk's single-site anchor).
  Controller fold a63fd1e on composer-s3: the census takes the supply paths'
  title "Plates To Cut" (unpinned, 2 sites already); cmd/emu + gui ok. The
  plan's B11 fence folded to match (this commit). NEXT: controller gate re-run
  on a63fd1e -> opus whole-diff review (brief composer-S3-exec-review-brief.md;
  Go path is now the .toolchain one) -> fold -> verify -> merge to fork main.
- Controller gates on a63fd1e GREEN (go test ./... 54 ok; gui 1174/1174;
  composer 228 PASS lines; 32-bit; oraclelive; js vet; gofmt residue is
  pre-existing on main). Opus whole-diff review DISPATCHED (brief 3f70311 ->
  design/agent-reports/composer-S3-exec-review-r0.md). Firmware size gate
  still UNRUN (no nix). NEXT: persist report -> fold on composer-s3 -> sonnet
  verify -> merge --no-ff to fork main + push + watch test.yml; NO flash.
- S3 whole-diff review (opus) 1C/2I/5M/3N persisted 077c5f4. C-1: Back past
  seating re-asks every slot with all sources filtered as used (keyed policy
  unreachable); I-1: Move up leaves the shape signature unchanged so §8j clears
  nothing; I-2: six self-check arms with no failing test. Fold brief 26a16e0
  (controller decisions: resume seating + release sources; Move up discards
  unconditionally; six fault rows). The implementer agent is RESUMED to fold
  on composer-s3 -> report composer-S3-fold-r0-report.md. NEXT: persist that
  report -> controller gate -> sonnet verification (brief
  composer-S3-fold-verification-brief.md, fill <S3_FOLD_SHAS>/<S3_REVIEWED_TIP>
  = a63fd1e) -> merge/push brief -> fork main. Still no flash (no /nix).
- NIX REINSTALLED by the operator (Omarchy; nixos.org multi-user, flakes on;
  `nix run .#build-firmware` builds). The Bash tool needs
  PATH=/nix/var/nix/profiles/default/bin. Firmware size BASELINE measured on
  fork main 321acb56 via `nix develop -c tinygo build -size short ...`:
  flash 1,506,884 B / RAM 62,592 B (plan's 169073c figure was 1,503,652 /
  62,592; S2 accounts for the difference). Take the S3 measurement on the
  fold's tip; the delta must be non-zero (plan C2 Step 4).
- S3 FOLD r0 DONE by the resumed implementer: fork 7edc863 (C-1/I-1), 83e932a
  (I-2 + a SECOND CRITICAL found by the control: self-check read K/N outside
  their domain, 4/12 preset pairs unbuildable -- fixed), 27afa9f (Minors/Nits);
  engrave db53513 (spec M-2/M-3), b1a1985 (F-461). Report 477b8ee. Implementer
  measured firmware 1,579,924 B flash / 62,800 B RAM (delta +73,040 / +208);
  controller re-measuring. NEXT: sonnet verification (brief filled) -> merge/
  push brief (<S3_TIP> = 27afa9f or the verified tip) -> fork main.
- Controller firmware measurement on 27afa9f (nix develop -c tinygo build
  -size short ...): flash 1,579,924 B / RAM 62,800 B = the implementer's
  figure; delta vs 321acb56 baseline +73,040 B flash / +208 B RAM (non-zero:
  plan C2 Step 4 discharged). Log .tmp/fw-size-s3-27afa9f.log.
- S3 fold VERIFIED 0C/0I (sonnet, report 33420b0; both firmware numbers
  reproduced by the verifier too). Merge/push agent DISPATCHED: merge --no-ff
  composer-s3 (27afa9f, 29 commits) into fork main, push, watch test.yml;
  report composer-S3-push-report.md. THEN: plan STATUS closing note (merge
  SHA, gates, firmware delta), CLAUDE.md toolchain note (Go via the flake is
  1.26.3 in /nix/store now; tests run on /scratch/code/shibboleth/.toolchain
  go1.26.7), push engrave master, S4 (on-device acceptance + the journey walk
  WITH the operator; flash only at the operator's word via ~/bin/sh/sh2-flash).

## RESUME POINT 2026-09-02 (evening) -- S3 SHIPPED; S4 next

- S3 MERGED: fork main b77449db (Test run 33709139231 green; report 6cdeb27);
  plan STATUS closed (b551da3); CLAUDE.md toolchain note added. NOT FLASHED.
- Worktree /scratch/code/shibboleth/wt-composer-s3 (branch composer-s3,
  27afa9f) can be removed: `git -C /scratch/code/shibboleth/seedhammer
  worktree remove wt-composer-s3` (the branch is merged).
- Open records: F-461 (self-check use-site arm unreachable), F-459 (S0b
  nits), F-460 (Multisig Build comment-only deprecation), F-454 (me 0.8.1
  owed), F-455/F-457 as the plan left them; the 57-vs-55 count mis-transcribed
  in two reports (reports are verbatim; note only).

Steps, in order:
1. Push engrave master (sonnet push agent, ci/staging ritual, FREEZE) --
   records only since ea64f86.
2. S4 = on-device acceptance. It has NO plan yet. Before any flash: (a) walk
   the Part-A journey WITH the operator on the emulator (cmd/emu walks +
   shots) and classify every divergence per the journey method; (b) write a
   short S4 acceptance plan (what is checked on the device, in what order,
   with the abort criteria) and R0 it with one journey lens; (c) flash ONLY
   at the operator's explicit word, via ~/bin/sh/sh2-flash, never picotool;
   the device boots the operator's own key (slot 1) -- the build is
   `nix run .#build-firmware` -> seedhammerii-v0.0.0-bg<sha>.uf2.
3. Owed to the operator, unchanged: the three S3 defaults (Part A ships
   alone; §7f offers the device's two plate forms (F-455); presets follow
   F-453's Rust half) -- all now IMPLEMENTED as defaults, so a different
   choice is a change request; F-457's C10 narrowing; me 0.8.1 (F-454); the
   live journey walk (now step 2a).
- FLASHED 2026-09-02 at the operator's word (device in BOOTSEL): sh2-flash -y
  built fork main b77449d, signed it (signed uf2 sha256 f85bb1619a06...),
  picotool load --verify 100%, reboot issued (log .tmp/sh2-flash-b77449d.log).
  Boot judgement on MACHINE power is the operator's; expected version line
  `bgb77449d (UNLOCKED)`. Engrave master pushed through 401697f.
  S4 (device acceptance + journey walk WITH the operator) starts when the
  boot is confirmed.
- BOOT CONFIRMED by the operator 2026-09-02: firmware bgb77449d boots on
  machine power. S4 BEGINS: the journey walk WITH the operator on the device
  (Part A first: no payload -> Wallet Policy -> Build a new policy -> shape ->
  keyless template -> census -> engrave -> decode the plate back). Findings go
  to design/S4_journey_walk_2026-09-02.md as they are made (refusal / warning
  / default / not-our-concern / documentation-only, and a change only when
  the wrong outcome is worse than saying nothing).
- S4 walk paused at step 3 (operator tired). W-1 FIXED on fork branch
  composer-s4 bc9dd63 (blank row first; Back returns to the wrapper choice;
  five walks retargeted; new test fails both mutations). Spec §7b + walk
  record 34c92bf. Controller gates running (.tmp/s4-gate.log); sonnet
  verification dispatched (brief composer-S4-W1-verification-brief.md).
  THEN: merge --no-ff composer-s4 into fork main, push, watch CI, sh2-flash
  (the device is in BOOTSEL, at the operator's word), record; resume the walk
  at "Add a spend path" when the operator is back.
- Controller gates on composer-s4 bc9dd63 GREEN: go test ./... 54 ok; gui
  1186/1186 (24 shards); 32-bit; oraclelive; js vet; firmware 1,579,940 B
  flash / 62,800 B RAM (+16 B over b77449db). Log .tmp/s4-gate.log.
- W-1 SHIPPED: fork main 60bee002 (merge of composer-s4 bc9dd63; CI run
  33711458384 green; report 64675a2). FLASHED at the operator's word (device
  in BOOTSEL): sh2-flash -y built + signed seedhammerii-v0.0.0-bg60bee00,
  load verified, reboot issued (log .tmp/sh2-flash-60bee00.log). Boot
  judgement pending on machine power: version line `bg60bee00 (UNLOCKED)`;
  W-1 check: Build a new policy -> script -> "Start from?" first row "Build my
  own paths", Back returns to the script choice. Walk resumes at step 3.

## RESUME POINT 2026-09-03 -- S4 plan drafted, journey lens in flight; walk resumes at step 3

State, all measured:
- Fork main 60bee002 = origin/main = the flashed bg60bee00; boot judgement on
  machine power still the operator's (W-1 check: Build a new policy -> script
  -> "Start from?" first row "Build my own paths", Back returns to the
  script choice). Worktree wt-composer-s4 already removed; branch composer-s4
  merged.
- The three shipped Wallet Policy emulator drivers (capture_walletpolicy /
  _seating / _tr_pathological) RUN against a fresh emu.wasm of 60bee002: all
  exit 0 (8/8/9 shots) -- the S3 plan's Task C2 Step 5 door edit, executed.
  Walk record 7a008a6; runner /scratch/code/shibboleth/.tmp/s4-emu-regression.sh.
- S4 plan DRAFT `design/IMPLEMENTATION_PLAN_composer_S4_acceptance.md` d640875:
  Task 0 done; Tasks 1-3 = the emulator journey (third payload blob,
  transcript_composer.sh, shots_composer.js + capture_composer.py, two arms +
  negative control); Task 4 = the live device walk + ONE keyless tr 2-of-3
  plate decoded back; Task 5 = Part B on the device (operator's call); Task 6
  records. Host oracle measured and pinned in plan §2 (md 1dc8d409, me 0.8.0
  at target/debug/me, ms 0.16.0): keyed wsh policy id 4dd749a8..., template
  id 531ab9e1..., four addresses; keyless tr chunk
  md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc, template id e0863d3c...;
  payload digest dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b. Plan checks in the
  commit message. Scratch values in /scratch/code/shibboleth/.tmp/s4-*.
- R0 = ONE journey lens (opus) DISPATCHED (brief 1e57f19,
  design/agent-briefs/composer-S4-plan-R0-journey-brief.md) -> report
  design/agent-reports/composer-S4-plan-R0-r0-journey.md.

Steps, in order:
1. When the lens lands: persist its report (own commit) -> fold the plan
   (pin the ? cells; a Critical on the Policy-ID / seed-account rule changes
   §2's oracle) -> plan checks in the fold commit -> sonnet fold verification
   only if the fold is non-trivial -> STATUS R0 GREEN.
2. Dispatch ONE implementer (opus, UC off) for Tasks 1-3: write
   design/agent-briefs/composer-S4-implementer-brief.md (two worktrees:
   /scratch/code/shibboleth/wt-composer-s4-emu off fork main, branch
   composer-s4-emu; /scratch/code/shibboleth/wt-engrave-s4-emu off master,
   same branch name); report composer-S4-implementation-report.md. Then:
   controller re-runs the gates (plan §4) -> opus whole-diff review (both
   diffs) -> fold -> sonnet verify -> fork merge --no-ff + push + watch
   test.yml; engrave push via staging.
3. The live walk WITH the operator (plan Task 4) whenever the operator is at
   the machine: confirm the bg60bee00 boot first, then resume the walk record
   at "Add a spend path" on the keyless tr 2-of-3 shape; then ONE plate with
   the plan's abort criteria; decode back on the host.
4. Push engrave master (sonnet push agent, ci/staging ritual, FREEZE) whenever
   the tree is clean and no commit is imminent.
5. Owed to the operator, unchanged: the three S3 defaults (implemented);
   F-457's C10 narrowing; me 0.8.1 (F-454); Task 5's payload write and plate
   count.
- 2026-09-03 (after the resume point): engrave master pushed 746e7b5 via
  staging (report c2424ba). S4 plan R0 r0 journey lens (opus) LANDED:
  1C/12I/5M/2N, all in the plan's expected values, none in 60bee002; the
  keyed oracle (Policy-ID 4dd749a8..., stubs 531ab9e1/4dd749a8, four
  addresses, seed B at its own account 0') CONFIRMED on the harness. C-1: the
  keyless plate string is the CHUNKED form
  md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3 (56 chars, 0xb0884;
  the device is chunk-form-always; md verify accepts both forms, so only the
  byte comparison catches it). Report ac2014e; controller re-ran every
  corrected value (keyless chunked, fingerprinted template 2 chunks 0x34c51,
  cards 2/3/2, stdin pack identical); fold fda1d9e (checks in its message);
  F-462/F-463 filed b431406; r1 verification (sonnet, targeted) DISPATCHED
  (brief 43e37eb) -> composer-S4-plan-R0-r1-fold-verification.md. Implementer
  brief drafted 54da69b with <S4_GREEN_SHA> to fill. NEXT: persist r1 -> if
  clean, STATUS R0 GREEN (closure lenses: journey on the shipped code, fold
  verification) -> fill the brief -> dispatch the implementer (Tasks 1-3).
  Task 4 (the device plate) now cuts the 56-char string; Task 4 reads the
  device door's lead first (the Load Payload region may still be on the
  machine).
- S4 PLAN R0 GREEN (2026-09-03): r1 fold verification (sonnet, 206807b)
  20/20 folded, 0C/0I/0M/0N; STATUS set 5a5f3df (plan checks: cite 0/0,
  glyph 47/0 -- the STATUS commit message says 46, the STATUS line added one
  quoted string --, tables 29/0, stepref 23 cross-document/round-label/UI
  indices). Implementer brief filled 7c475e5. S4 IMPLEMENTER (opus, ONE
  agent, UC off) DISPATCHED against 5a5f3df for Tasks 1-3: worktrees
  /scratch/code/shibboleth/wt-composer-s4-emu (fork, off 60bee002) and
  /scratch/code/shibboleth/wt-engrave-s4-emu (engrave, off master), both on
  branch composer-s4-emu; reports to composer-S4-implementation-report.md.
  Whole-diff review brief drafted 3cb0b1f (placeholders <S4_FORK_TIP>,
  <S4_ENGRAVE_TIP>, <S4_ENGRAVE_BASE>, <S4_GREEN_SHA>=5a5f3df,
  <CONTROLLER_GATES>). NEXT when the implementer reports: persist -> controller
  re-runs plan §4's gates on both worktrees (go test ./cmd/emu/, js vet, gofmt,
  capture_composer.py --arm both + --prove-it-can-fail with the EMU override,
  the three shipped drivers, firmware size unchanged 1,579,940/62,800) ->
  fill the review brief -> opus whole-diff review -> fold -> sonnet verify ->
  fork merge --no-ff + push + watch test.yml; engrave: the journeys diff
  merges into master and pushes via staging. Engrave master push (records)
  next, while the implementer runs.
- S4 IMPLEMENTER REPORTED (9be1977): Tasks 1-2 DONE (fork composer-s4-emu
  05d903b: third emulator payload, digest dbe9 e774 ... matched on the
  emulator's digest screen; engrave composer-s4-emu 5040bb2:
  transcript_composer.sh, 27 gates, every §2 oracle byte-identical; the three
  shipped drivers exit 0 against the worktree). Task 3 STOPPED on a shipped
  Critical: composerPickScreen (gui/composer_paged.go) has no per-row touch
  target, so on the SH2 (touch only, no directional buttons) only a page's
  first row is selectable -- n=2, n=3, Done, hash rows, seating rows all
  unreachable; every composer test drives synthetic Down events. Recorded as
  W-2 in the walk record + fix brief (bbb852f). The implementer is RESUMED
  (same agent) on fork branch composer-s4b (worktree wt-composer-s4b, off
  60bee002): per-row Clickable hit areas as ChoiceScreen has; regression test
  on the touch harness through the real flow, failing first; emulator proof;
  report composer-S4-W2-fix-report.md. NEXT: persist that report -> controller
  gates on composer-s4b (gui shards, cmd/emu, js vet, firmware size delta) ->
  sonnet verification (targeted; brief to write) -> merge --no-ff into fork
  main + push + watch test.yml -> sh2-flash at the operator's word (the device
  walk cannot pass step 3 without it) -> the implementer merges main into
  composer-s4-emu and resumes Task 3 (the driver taps rows by geometry, as the
  shipped ChoiceScreen walks do). Plan fold owed: Task 3 rows note that taps
  select rows; §4 coverage line; the review brief's <CONTROLLER_GATES>.
  Implementer additions to review at the whole-diff round: test.yml gained
  ./cmd/emu/ in the oraclelive compile step; `me sysw show` prints digest: on
  STDERR (transcript reads both streams).
- W-2 FIXED on fork composer-s4b 2dff0ee (report 1daed7e): per-row Clickable
  hit areas in composerPickScreen (composerPageLines now returns the row
  bands; one Clickable per visible row, cap 24); touch-harness test
  composer_pick_touch_test.go fails on 60bee002 ("how many must sign? 1"),
  passes after, fails under the op.Input mutation; gui 1187/1187; emulator
  paired control tap-3 -> "1 2 3", tap-Done -> "Sorted keys". Briefs filled
  0caef53; sonnet targeted verification DISPATCHED ->
  composer-S4-W2-verification.md; controller gates running
  (.tmp/s4b-gate.log: gofmt/vet/TestComposer/cmd/emu/js vet done and clean,
  shards/32-bit/oraclelive/firmware size pending). Plan folded fdce82f (rows
  tapped by geometry; Task 4 prerequisite = the fix flashed). NEXT: persist
  the verification -> merge/push agent (brief composer-S4-W2-merge-push-brief.md,
  message composer-S4-W2-merge-message.txt) -> sh2-flash at the operator's
  word -> resume the implementer for Task 3 (merge fork main into
  composer-s4-emu first) -> the device walk resumes at step 3 on the new
  build.
- W-2 SHIPPED: fork main 3cc71d9b (merge of composer-s4b 2dff0ee; Test run
  33735918679 green; report persisted); verification 0C/0I/0M/0N (cc1ba56);
  controller firmware 1,580,580 B flash / 62,800 B RAM (+640 / +0 over
  60bee002, .tmp/s4b-gate.log). NOT FLASHED -- the operator's word is owed;
  the device walk cannot pass step 3 on bg60bee00. Implementer RESUMED for
  Task 3 (brief composer-S4-task3-resume-brief.md: merge fork main into
  composer-s4-emu, write shots_composer.js + capture_composer.py, run both
  arms, the negative control, the named unchunked-string mutation, the
  shipped drivers; append "Task 3 -- DONE" to the implementation report).
  NEXT: sh2-flash 3cc71d9b at the operator's word (expected version line
  bg3cc71d9 (UNLOCKED)); persist the Task 3 report -> controller gates ->
  fill the review brief (composer-S4-exec-review-brief.md) -> opus whole-diff
  review -> fold -> verify -> merge both; engrave push.
- S4 TASK 3 DONE (report appended 2becbcd): fork composer-s4-emu 86cec95
  (= 05d903b + merge of main 3cc71d9b + shots_composer.js/shTargets), engrave
  composer-s4-emu c6adac2 (capture_composer.py). --arm both exit 0 (three
  legs, 50 shots; digest, both ids, both stubs, four addresses, every
  engraved string byte for byte incl. the 56-char keyless plate); negative
  control exit 0; the named unchunked mutation exit 1 ("56 chars" vs "47
  chars"); shipped drivers exit 0; gui 1188/1188. New emulator surface:
  window.shTargets() (read-only hit-region reader; zero rows on a pre-W-2
  build). Firmware 1,580,580/62,800 = W-2 alone; plan §4's pin wants
  updating to that number (Task 6). Controller gate run on both worktrees
  RUNNING (.tmp/s4-impl-gate.sh -> s4-impl-gate.log: fork gates + firmware;
  staged tree .tmp/s4run-ctl: transcript with FORK override, --arm both,
  control, the named mutation, the three shipped drivers). Review brief
  composer-S4-exec-review-brief.md filled except <CONTROLLER_GATES> (+ a
  shTargets lens). NEXT: fill that from the log -> commit brief -> opus
  whole-diff review -> persist -> fold -> sonnet verify -> fork merge --no-ff
  (composer-s4-emu) + push + watch; engrave: merge composer-s4-emu into
  master + staging push -> Task 6 records. Flash of 3cc71d9b still at the
  operator's word.
- CONTROLLER GATES on the S4 implementation (logs .tmp/s4-impl-gate{,2,3}.log;
  staged tree .tmp/s4run-ctl with seedhammer -> wt-composer-s4-emu): fork
  86cec95 gofmt/vet/cmd/emu/js vet/gui 1188 across 24 shards/32-bit/
  oraclelive/firmware 1,580,580/62,800 all ok; engrave c6adac2 transcript
  exit 0, --arm both exit 0 (21+21+8 shots, all legs matched), control exit 0,
  the unchunked mutation exit 1 naming 56 vs 47 chars, three shipped drivers
  exit 0. Two harness lessons on the way (memory): exit codes through a tail
  pipe are tail's; a nohup child of a run_in_background Bash dies with it
  (launch with setsid from a foreground call, wait with Monitor). Review brief
  filled 9700ebf; OPUS WHOLE-DIFF REVIEW DISPATCHED ->
  composer-S4-exec-review-r0.md. NEXT: persist -> fold (implementer resumed
  or inline) -> sonnet verify -> fork merge --no-ff composer-s4-emu + push +
  watch; engrave: merge composer-s4-emu into master + staging push -> Task 6
  (spec §12 items 2/3/9 EXECUTED lines, staged plan §S4, README row +
  build_pdf_composer.py, F-460 checked present, F-461 note). Flash of
  3cc71d9b still at the operator's word.
- S4 WHOLE-DIFF REVIEW (opus) 0C/1I/4M/4N persisted 93988cd. I-1: the
  negative control accepted ANY driver failure as the address comparison
  catching the corruption (a corrupted payload digest killed the walk at row
  2 in 8 s and it printed PASSED). M-1 dead post-condition; M-2 "TEXT ONLY
  alone" unasserted; M-3 empty address list compares nothing; M-4 port
  collision with capture_seating.py; N-1 unreachable guard; N-3 transcript
  rev line; N-4 no composer needle pinned. N-2 CORRECTS THE CONTROLLER'S
  RECORD: my review-brief gate line said `gofmt -l cmd/ gui/` clean -- the
  gate log (s4-impl-gate.log lines 3-5) lists gui/transaction.go,
  transaction_golden_test.go, transaction_txrecord_test.go, unformatted at
  60bee002 already and outside the diff; the plan's gate is `gofmt -l cmd/`,
  which is clean. Recorded here; the brief stays as committed. Fold brief
  composer-S4-fold-r0-brief.md; the implementer is RESUMED to fold ->
  composer-S4-fold-r0-report.md. NEXT: persist -> controller re-runs the
  changed gates -> sonnet fold verification (brief to write) -> merge both.
- S4 REVIEW FOLDED by the implementer (report 095ee81): fork composer-s4-emu
  a6eb44e (M-1, M-2, N-1, N-4 needle pins), engrave composer-s4-emu 651fa0e
  (05a066a: I-1 control attribution INCONCLUSIVE arm proven both ways, M-3,
  M-4 ports 8803/8744; 651fa0e: transcript regenerated at the tip). Controller
  fast gates on a6eb44e: gofmt cmd/ clean, cmd/emu ok, js vet exit 0. Sonnet
  fold verification DISPATCHED (brief bb795d7) ->
  composer-S4-fold-r0-verification.md. Merge briefs ready:
  composer-S4-merge-push-brief.md (fork, tip a6eb44e, message
  composer-S4-merge-message.txt) and engrave-merge-push-brief-s4.md (engrave,
  tip 651fa0e, message engrave-merge-message-s4.txt, <ENGRAVE_MASTER_TIP> to
  fill at dispatch). NEXT: persist the verification -> if 0C/0I: fork merge
  agent -> engrave merge agent (FREEZE master) -> Task 6 records (spec §12
  items 2/3/9 EXECUTED, staged plan §S4, journeys README row +
  build_pdf_composer.py, F-460 present (measured: gui/multisig_build.go:22-24
  + a pinning test), F-461 note) -> the walk record -> flash 3cc71d9b (or the
  S4 merge, which changes no firmware bytes) at the operator's word -> the
  device walk from step 3 -> Task 4's plate -> Task 5 at the operator's call.
- W-3 FOUND (walk record 325a164) by LOOKING at the S4 capture's shots while
  drafting the Task 6 PDF: the composer's paged widgets (composerPageLines)
  centre lines across the full panel, so the Template-ID's 32nd hex digit is
  under the Back button and the keyless mk encode tails under the page button;
  the consent (confirmReviewScreen) is fine; shScreen() sees text under a
  button, so the driver, the review and the verification all passed. Fix
  brief composer-S4-W3-fix-brief.md; the implementer is RESUMED on fork branch
  composer-s4c (worktree wt-composer-s4c, off 3cc71d9b): wrap+centre inside
  the band left of the nav column, re-measure the §13 capacity pins, a
  GEOMETRY test failing first, emulator proof -> composer-S4-W3-fix-report.md.
  Task 6 PDF builder DRAFTED: design/journeys/build_pdf_composer.py
  (untracked on master; developed and run in .tmp/s4run-ctl, 17 pages, no
  missing assets) -- commits with Task 6 after the engrave merge. Memory:
  text-extraction-cannot-see-clipping. NEXT unchanged, plus: W-3 verification
  (sonnet, targeted; brief to write) -> merge composer-s4c -> flash the
  resulting main at the operator's word (W-2 + W-3 both matter for the
  device walk's step 3 and the stub screen).
- W-3 FIXED on fork composer-s4c 0b49f66 (report e0c1d3d): composerPageLines
  wraps+centres inside x 8..419; rasterising geometry test red on 3cc71d9b,
  green after; gui 1189/1189; firmware unchanged 1,580,580/62,800. Spec §13
  folded on master (stub screen 6 rows/frame, controller re-measured). Sonnet
  W-3 verification DISPATCHED (brief bda683b) -> composer-S4-W3-verification.md.
  MERGE ORDER FORCED by a dependency: W-3 changes the stub screens' page
  counts, and shots_composer.js pins them (`pages.length !== 2`), so:
  (1) W-3 verified -> merge composer-s4c into fork main (brief
  composer-S4-W3-merge-push-brief.md) -> (2) resume the implementer with
  composer-S4-fold-r1-brief.md (<W3_MERGE_SHA>): merge main into
  composer-s4-emu, pin the measured page counts, re-run the captures ->
  (3) targeted sonnet check of that fold (small) -> (4) merge composer-s4-emu
  into fork main (composer-S4-merge-push-brief.md: its tip moves past
  a6eb44e; refill) -> (5) engrave merge (engrave-merge-push-brief-s4.md) ->
  Task 6. The driver's fold-r0 verification (sonnet, on a6eb44e/651fa0e) is
  still running and stands for those tips. Flash: the main after (1) carries
  W-2 + W-3 and is the build for the device walk -- at the operator's word.
- S4 driver fold-r0 VERIFIED (sonnet, 8/8, 0I/0M/0N; report persisted): the
  driver branches a6eb44e / 651fa0e are review-closed. They still wait on the
  merge order above (W-3 merge -> page-count re-pin -> driver merge).
- W-3 VERIFIED (sonnet 0C/0I/2M, report c9b8194; F-464 filed for the test
  coverage gap; the fix report's shard count is 1187->1189 = +2, not +1).
  Fork merge of composer-s4c DISPATCHED (brief composer-S4-W3-merge-push-
  brief.md) -> composer-S4-W3-push-report.md. Fold-r1 verification brief
  drafted (composer-S4-fold-r1-verification-brief.md, placeholders). NEXT
  when the merge lands: fill <W3_MERGE_SHA> in composer-S4-fold-r1-brief.md
  -> resume the implementer -> persist -> fill + dispatch the r1 verification
  -> refill composer-S4-merge-push-brief.md's tip -> driver merge -> engrave
  merge -> Task 6 -> flash at the operator's word.
- W-3 SHIPPED: fork main 1ae0ffcb (merge of composer-s4c 0b49f66; CI run
  33750030577 green; report 6cddca0). Worktrees wt-composer-s4b and
  wt-composer-s4c removed (branches merged). FLASH TARGET for the device walk
  is now fork main 1ae0ffcb (W-2 + W-3; expected version line bg1ae0ffc
  (UNLOCKED)) -- at the operator's word only. Implementer RESUMED for fold r1
  (brief cf0acd5): merge main into composer-s4-emu, re-pin the stub page
  counts -> composer-S4-fold-r1-report.md; then the r1 verification (brief
  filled except the two tips) -> driver merge (refill
  composer-S4-merge-push-brief.md's tip) -> engrave merge -> Task 6.
- FOLD r1 was TRIVIAL (report a3138f2): the merge of main (W-3) into
  composer-s4-emu = b481be7 changed no page count, no pin, no assertion;
  only transcript_composer.txt's rev line (55db8e5). Per the proportional
  re-review rule no r1 verification round is dispatched (the brief
  composer-S4-fold-r1-verification-brief.md stays on record, unused); the
  controller re-runs the capture on the merged tips instead
  (.tmp/s4-impl-gate4.log: --arm both + the control) and looks at the stub
  shots. Merge briefs refilled (2529b61): fork tip b481be7 on 1ae0ffcb,
  engrave tip 55db8e5. NEXT: gate4 green + shots legible -> fork driver merge
  -> engrave merge (fill <ENGRAVE_MASTER_TIP>) -> Task 6.
- GATE4 GREEN on the merged driver tip (fork b481be7 / engrave 55db8e5;
  .tmp/s4-impl-gate4.log): --arm both exit 0 (21+21+8 shots, all legs
  matched), control exit 0; the controller LOOKED at k02-stub-p0.png on the
  merged build: all 32 hex digits and both mk encode lines clear of the
  buttons (W-3 holds). Fork driver merge DISPATCHED (brief 2529b61) ->
  composer-S4-push-report.md. The post-W-3 capture artifacts (shots c*/k*,
  composer-result.json, out/composer/) are copied into the main checkout's
  design/journeys (untracked, as every journey's are) for the Task 6 PDF.
  NEXT: persist the fork push report -> engrave merge (fill
  <ENGRAVE_MASTER_TIP>; FREEZE) -> Task 6 (run build_pdf_composer.py in the
  main checkout, commit builder + PDF + README row; spec §12 EXECUTED lines;
  staged plan §S4; FOLLOWUPS F-460/F-461 notes) -> push -> flash at the
  operator's word (target = fork main after the driver merge; firmware bytes
  unchanged from 1ae0ffcb since cmd/emu is outside cmd/controller).

## RESUME POINT 2026-09-03 -- S4 EMULATOR JOURNEY SHIPPED; the device walk is what remains

State, all measured:
- Fork main 6fb90cb18b3ec24050251a3cc01143bf8c022efd = origin/main: S3 +
  W-1 (60bee002) + W-2 (3cc71d9b, pick lists tappable) + W-3 (1ae0ffcb, paged
  lines clear of the buttons) + the S4 driver (composer-s4-emu: third
  emulator payload, shots_composer.js + shTargets(), needle pins). Firmware
  1,580,580 B flash / 62,800 B RAM (cmd/emu is outside cmd/controller, so
  the driver merge changed no firmware byte). CI green on every merge.
- The DEVICE still runs bg60bee00 (S3 + W-1). NOT FLASHED since: on it, step
  3's "how many keys?" cannot take 3 (W-2) and the Template-ID is not fully
  legible (W-3). Flash target = fork main (6fb90cb; version line
  bg6fb90cb (UNLOCKED)) via ~/bin/sh/sh2-flash, ONLY at the operator's word.
- mnemonic-engrave master: S4 plan STATUS closed for Tasks 0-3, 6; spec §12
  items 2/3/9 EXECUTED; staged plan §S4 STATUS; journey PDF + README;
  FOLLOWUPS F-462 (h vs ' notation), F-463 (ms1 reminder), F-464 (geometry
  test coverage) filed; F-460 checked present; F-461 revisited unchanged.
  All worktrees removed. Records pushed through 789a411 + the s4 merge
  e3ee51c9; the commits after e3ee51c9 are records only (push next).
- Walk record design/S4_journey_walk_2026-09-02.md: W-1 (shipped), W-2
  (shipped), W-3 (shipped); paused at step 3.

Steps, in order:
1. Push engrave master (sonnet push agent, ci/staging, FREEZE) -- records only.
2. At the operator's word: `~/bin/sh/sh2-flash -y` on fork main 6fb90cb with
   the device in BOOTSEL; boot judgement on machine power is the operator's;
   expected version line `bg6fb90cb (UNLOCKED)`. Check W-1 (Start from? row
   0 = Build my own paths), W-2 (tap 3 on "how many keys?" selects it), W-3
   (the Template screen's id shows all 32 hex digits) on the device.
3. The live walk WITH the operator (plan Task 4): resume the walk record at
   "Add a spend path" on Taproot 2-of-3; three questions per step; classify;
   read the door's Lead first. Then ONE plate with the plan's abort criteria;
   the string must equal md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3
   byte for byte (md verify accepts the unchunked form too, so bytes are the
   check). Fixes, if any, batch on a fork branch composer-s4d with a test
   that fails first, sonnet-verified, merged, flashed at the operator's word.
4. Task 5 (Part B on the device) at the operator's call: me sysw pack
   --region from design/journeys/out/composer/records.txt (regenerate with
   transcript_composer.sh), picotool load at 0x10D00000 in BOOTSEL, digest
   dbe9 e774 ..., the keyed itinerary by hand, form B (4 plates) or A (2).
5. Close: walk record Part A/B closed -> staged plan §S4 OPEN items -> plan
   STATUS -> continuity -> push. Then the residue: F-454 (me 0.8.1), F-455,
   F-457, F-459, F-462/F-463/F-464 (post-S4 polish), the operator's three S3
   defaults (implemented; a different choice is a change request).
- FLASHED 2026-09-03 at the operator's word (device in BOOTSEL, picotool saw
  RP2350 with the previous image's signature verified): sh2-flash -y built
  fork main 6fb90cb, signed it (seedhammerii-v0.0.0-bg6fb90cb.signed.uf2,
  sha256 dc5fd3cf59839209d222acc7e7420f5ecd8fe2181a2c94190b48034a01087887),
  picotool load --verify 100%, reboot issued (log .tmp/sh2-flash-6fb90cb.log).
  Boot judgement on MACHINE power is the operator's; expected version line
  `bg6fb90cb (UNLOCKED)`. Device checks before the walk: W-1 (Start from? row
  0 = Build my own paths), W-2 (a tap on 3 in "how many keys?" selects it),
  W-3 (the Template screen shows all 32 hex digits). Then the walk from
  "Add a spend path" (Taproot 2-of-3), the door's Lead read first.
