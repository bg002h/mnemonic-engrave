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
- BOOT CONFIRMED bg6fb90cb; W-1/W-2/W-3 confirmed on the device (9c96b78).
  Part A WALKED with the operator through the census and the variant pick:
  no divergence; Template-ID/stub on the device = the host's (100ff8d). The
  plate is DEFERRED: no blank available. Open: the door's lead line wording
  on this machine (payload region present, not loaded). NEXT when a blank is
  on hand: repeat from the door (under a minute), TEXT + QR, hold to cut,
  read the plate back = md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3
  byte for byte; then Task 5 at the operator's call; then close the records
  and push (two record commits unpushed since 950f42e).
- W-4 FOUND by the operator on the device (walk record ae9bf9c, cause
  5ef61ae): composerDigitEntry overprints its prompt and its range/echo line
  (per-line clamp to the band above the keyboard); all four pads. The
  implementer is RESUMED on fork branch composer-s4d (worktree
  wt-composer-s4d, off 6fb90cb): one vertically centred group, a rasterising
  geometry test over the four pads failing first, emulator proof via the
  operator's route, then capture --arm both -> composer-S4-W4-fix-report.md.
  Briefs ready: composer-S4-W4-verification-brief.md, -merge-push-brief.md,
  -merge-message.txt (<W4_FIX_SHA>). NEXT: persist -> controller gates ->
  sonnet verification -> merge -> flash at the operator's word -> the
  operator re-checks the blocks and date pads; the plate still waits for a
  blank. The device walk otherwise continues on bg6fb90cb.
- W-4 FIXED on fork composer-s4d bb50775 (report fcc661b): the digit pad's
  box + prompt + echo are one centred group in the 86 px band; rasterising
  test 12/12 red on 6fb90cb, green after; gui 1192; capture --arm both 0;
  the diff also extracts the date validator in composer_lock.go (for the
  test); firmware +624 B -> 1,581,204 / 62,800 (implementer; controller
  measuring, .tmp/fw-size-s4d.log). Controller fast gates: gofmt clean,
  TestComposer ok, cmd/emu ok, js vet 0. Sonnet W-4 verification DISPATCHED
  (brief 34ff2c9) -> composer-S4-W4-verification.md. Also this session:
  W-5 recorded (hashlock entry: the payload route is invisible without a
  payload; F-465 `ms hashlock` proposed, F-466 on-device entry for a
  ruling; the term "hashlock phrase" agreed and in memory). NEXT: persist
  the verification -> merge composer-s4d (brief composer-S4-W4-merge-push-
  brief.md) -> plan §4 pin to the measured size -> flash at the operator's
  word -> the operator re-checks the blocks and date pads -> the plate when
  a blank is on hand.
- FLASHED 2026-09-03 at the operator's word (device in BOOTSEL) the UNMERGED
  W-4 branch build composer-s4d bb50775 (built in the worktree with
  `nix run .#build-firmware`, signed and loaded by sh2-flash, verify 100%;
  log .tmp/sh2-flash-bb50775.log) so the operator can check the pads while
  the verification runs; expected version line `bgbb50775 (UNLOCKED)`. A
  second flash of fork main follows the merge. Operator rulings recorded
  ce9e5c5 (F-465 home = ms; F-466 on-device hashlock-phrase entry REQUIRED,
  Rust first). Measured: a 32-byte preimage encodes as an ms1 string and
  decodes back (indistinguishable from a seed backup -- a labelling
  question for the ms hashlock design).

## RESUME POINT 2026-09-03 (evening) -- W-4 in verification; device on the branch build

State: fork main 6fb90cb (S3 + W-1 + W-2 + W-3 + the S4 driver); branch
composer-s4d bb50775 = the W-4 digit-pad fix (worktree
/scratch/code/shibboleth/wt-composer-s4d), verified by the implementer,
sonnet verification IN FLIGHT -> design/agent-reports/composer-S4-W4-verification.md
(if that file exists on resume, persist it; if not, re-dispatch with
design/agent-briefs/composer-S4-W4-verification-brief.md). The DEVICE runs
the branch build bgbb50775 (flashed for the pad check). Firmware on bb50775:
1,581,204 B flash / 62,800 B RAM (+624 over 6fb90cb). Records on engrave
master are committed; unpushed since 6db0545 (push next).

Steps, in order:
1. Persist the W-4 verification. If 0C/0I: merge agent (sonnet) with
   design/agent-briefs/composer-S4-W4-merge-push-brief.md (tip bb50775,
   message composer-S4-W4-merge-message.txt) -> fork main; remove
   wt-composer-s4d; plan §4 firmware pin -> 1,581,204 / 62,800 with the
   merge SHA; walk record W-4 "shipped".
2. Push engrave master (sonnet push agent, ci/staging, FREEZE).
3. At the operator's word: sh2-flash fork main (replaces the branch build;
   same bytes as bb50775 plus the merge commit).
4. The device walk continues: the operator confirms the blocks and date
   pads show two lines; the plate (Taproot 2-of-3, string
   md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3) when a blank is
   on hand; Task 5 at the operator's call.
5. Next cycle (ruled 2026-09-03): F-465 `ms hashlock` (Rust first; the
   preimage as an ms1 backup, labelling open) and F-466 on-device
   hashlock-phrase entry (REQUIRED, Rust first; spec §6c/§14/C25 fold with
   its own R0). Term: "hashlock phrase" (memory).
Command to resume: /resume-composer
- W-4 SHIPPED: fork main 70008da5 (merge of composer-s4d bb50775; CI run
  33826231689 green; verification 0C/0I persisted 4f24459; push report
  2d8487c). Plan §4 pin -> 1,581,204 / 62,800 (e405b12). wt-composer-s4d
  removed. The device runs bgbb50775 (the branch build; identical gui bytes
  to main 70008da5 apart from the merge commit) -- re-flashing main is
  optional hygiene at the operator's word. Steps 1 of the evening RESUME
  POINT DONE; step 2 (push) dispatched next; then CLEAR. On resume: step 3
  (optional re-flash), step 4 (the operator's pad re-check -> record it; the
  plate when a blank is on hand; Task 5 at the operator's call), step 5
  (next cycle: F-465 ms hashlock, F-466 on-device hashlock-phrase entry).

## RESUME POINT 2026-09-03 (night) -- HASHLOCK PHRASE cycle opened; brainstorm under opus review

State, all measured:
- Engrave push a8af7a0 DONE (report 51b7c69). The device still runs the
  W-4 branch build bgbb50775 (same gui bytes as fork main 70008da5); the
  operator's pad re-check, the Taproot 2-of-3 plate (blank pending) and Task 5
  remain the S4 residue, all at the operator's word. S4 is otherwise closed.
- The operator pivoted to the hashlock phrase (F-465/F-466). Brainstorm
  record `design/BRAINSTORM_hashlock_phrase.md` (fb64091, 72081c5): rulings
  L1-L11 verbatim -- two methods, the operator's choice: hardened =
  PBKDF2-HMAC-SHA256, 100,000 iterations, salt "ms-hashlock-v1", 32 bytes,
  or plain sha256; a new ms1 kind byte 0x03 for the preimage, derivation in
  ms-codec (`ms_codec::hashlock`), `ms hashlock` a thin verb; device = digest
  only; 32 bytes = 64 hex. Sections 4.1-4.3 agreed; 4.4 (device leg), 4.5
  (process/homes), 4.6 (testing) still to walk WITH the operator after the
  review. Defaults for veto in section 5 (method default hardened; salt; 20-char
  warning floor; --out = preimage; --json advisory; --random; method asked
  after the phrase; the reuse line from 3.7). F-467 (hashvault journey
  hashlocks unspendable: 40/38/34-byte phrases hashed once) and F-468 (ms
  split has no preimage source) filed.
- R0 round 0 = ONE opus reviewer, cryptography + Bitcoin programmer lens,
  DISPATCHED against 72081c5 (brief
  `design/agent-briefs/hashlock-brainstorm-R0-crypto-review-brief.md`) ->
  `design/agent-reports/hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md`.
  If that file exists on resume, persist it (own commit); if not, re-dispatch
  with the brief.

Steps, in order:
1. Persist the review (own commit) -> read it -> machine-check every
   measurable claim -> walk its findings WITH the operator (rulings are the
   operator's; a Critical on the KDF/salt/method rule changes section 2) ->
   fold the brainstorm record (own commit) -> re-review only if the fold is
   non-trivial (sonnet verification of the fold; a new lens only if a question
   remains unasked).
2. Walk sections 4.4-4.6 with the operator, one at a time; record; then the
   writing-plans path per the constellation workflow: spec(s) first --
   `mnemonic-secret/design/SPEC_ms_hashlock.md` (kind 0x03 + `ms_codec::hashlock`
   + `ms hashlock`; own R0) and the composer spec fold (§6c, §8, §12, §14,
   C25; own R0 with a journey lens) -- then plans per stage (H1 ms, H2 fork,
   H3 records, H4 device), each build-gated and R0'd immediately before its
   implementer. Implementation UC off, one implementer per stage.
3. Push engrave master (sonnet push agent, ci/staging, FREEZE) whenever the
   tree is clean and no commit is imminent.
4. Owed to the operator, unchanged: the S4 residue above; F-454 (me 0.8.1,
   now also carrying me's ms1 classifier learning kind 0x03); F-455/F-457/
   F-459/F-462/F-463/F-464 polish.
Command to resume: /resume-composer

## RESUME POINT 2026-09-03 (late) -- brainstorm R0 closed; security-software review in flight; PAUSE BEFORE SPEC

State, all measured:
- Brainstorm `design/BRAINSTORM_hashlock_phrase.md` at 82433fd: rulings
  L1-L19. R0 round 0 (opus crypto lens, 1C/6I/6M/2N, report d13819e) folded
  d2e8f68 with three rulings in response (L12 sha256 warns always, never
  refuses; L13 no --salt this cycle, F-469; L14 preimage singles carry id
  `hash`); r1 sonnet fold verification (95e7423: 16 FIXED / 1 PARTIAL / 1 new
  Important) folded d31d595 (decode/combine PRINT a preimage; only
  verify/derive refuse). L15 no scrub discipline on the device; L16 4.4
  agreed; L17 4.5 agreed; 4.6 (testing) PRESENTED, not ruled.
- L18: a second lens, security software engineering (opus, single agent),
  DISPATCHED against 82433fd (brief
  `design/agent-briefs/hashlock-brainstorm-R0-r2-security-software-brief.md`)
  -> `design/agent-reports/hashlock-brainstorm-R0-r2-security-software-expert.md`.
  If that file exists on resume, persist it (own commit); if not, re-dispatch.
- **L19 (operator, verbatim): "Pause before spec."** After the L18 review is
  persisted, folded and walked with the operator, and 4.6 is ruled, STOP. No
  `SPEC_ms_hashlock.md`, no composer spec fold, until the operator says so.
- Engrave master pushed through d31d595 (report d7ff513); commits after it
  are records only.

Steps, in order:
1. Persist the L18 report (own commit) -> machine-check its measurable claims
   -> bring the findings to the operator (rulings are theirs; C/I on 4.6 or on
   an agreed section get an operator disposition) -> fold (own commit) ->
   sonnet fold verification if non-trivial -> record the operator's ruling on
   4.6.
2. STOP (L19). Report the state; wait for the operator's word on the spec.
3. Push engrave master (sonnet push agent, ci/staging, FREEZE) whenever the
   tree is clean and no commit is imminent.
Command to resume: /resume-composer
- 2026-09-03 (after): L18 review LANDED (opus security-software lens,
  4C/6I/7M/3N, persisted e9d7895; every measurable claim reproduced by the
  controller); rulings in response L20 (`--in` = the ms1; two phrase
  channels; ms1-shaped phrases refused), L21 (`--random` requires `--out` or
  `--json`), L22 (`me`'s classifier first as a new stage H1b; `DecodeMS1`
  unchanged with a separate preimage decoder; no new class); fold c20ec9e
  (checks in its message); r3 sonnet fold verification DISPATCHED (brief
  f292f83) -> `hashlock-brainstorm-R0-r3-fold-verification.md`. **The
  operator RULED 4.6 stands ("Yes, 4.6 stands") = L23; NOT yet written into
  the record because the verifier is reading it -- write L23 and flip 4.6's
  header from PRESENTED to agreed as soon as the r3 report is persisted.**
  Then fold any r3 finding, commit, push, and STOP (L19).
- **W-6 (operator, 2026-09-03, on the device bgbb50775; walk record):** in
  Wallet Policy -> Build a new policy -> script (tr/wsh) -> `Start from?`,
  any selection made there cannot be returned to with Back: Back jumps to the
  script choice, and picking a script again SKIPS `Start from?`. Recorded
  verbatim in `design/S4_journey_walk_2026-09-02.md` W-6 with a controller
  note (likely the shape flow's re-entry after W-1's Back change,
  `gui/composer_shape.go`; NOT measured, NOT fixed). Next for W-6: reproduce
  on the emulator with the shipped driver, classify, and if it is a change,
  a fork branch `composer-s4e` with a failing-first test, sonnet-verified,
  merged, flashed at the operator's word. W-6 is S4 residue, independent of
  the hashlock cycle's pause (L19).
- **BRAINSTORM COMPLETE (2026-09-03, late):** r3 sonnet verification of the
  r2 fold (e06dd15: FIXED 20 / PARTIAL 1 / NOT 0; two Importants + one Minor,
  all wording: the 4.6 fork bullet still described the pre-review H2, a
  length-row test named `DecodeMS1` instead of `DecodeMS1Preimage`, the
  byte-exact row named one phrase channel) folded 0f9bb99 with L23 ("Yes,
  4.6 stands") written and the record's STATUS set to COMPLETE; PAUSED
  BEFORE SPEC (L19). Engrave master pushed through 203f3bb (report 643a119);
  the commits after it (643a119, e06dd15, 0f9bb99, this one) are records
  only -- push next. NOTHING is in flight. On resume: (1) if the operator has
  given the word, start `mnemonic-secret/design/SPEC_ms_hashlock.md` per
  brainstorm 4.5 (spec first, own R0: correctness + adversarial + tests-vector
  lenses; then the H1 plan with a `plan-build-gate-ms.sh`); (2) otherwise the
  S4 residue: W-6 (reproduce on the emulator, classify), the pad re-check,
  the plate, Task 5 -- all at the operator's word.
- **HAND-OFF NOTE (the operator is switching to opus; fable budget low):**
  everything above is on disk and committed except L23 in the brainstorm
  record (pending the r3 verifier). An opus session resumes with
  `/resume-composer`; the rules that bind hardest: persist and fold are two
  commits; the agent writes its own report; push only via
  `scripts/push-via-staging.sh` with master frozen; sonnet verifies folds;
  no spec until the operator says so (L19).

## RESUME POINT 2026-09-04 -- W-6 MEASURED, W-7 (Critical) FOUND AND FIXED; verification in flight

State, all measured this session:
- The hashlock cycle is UNCHANGED and still PAUSED BEFORE SPEC (L19). Nothing
  was started there. This session took the S4 residue instead, which L19 does
  not gate.
- **W-6 reproduced** on fork main `70008da` by a flow-level walk that drives
  the real `composerFlow` (no emulator needed to see it; the operator had
  already seen it on the device). Both halves: Back at the path list draws
  `Which script?`, and re-picking a script draws the path list with
  `Start from?` never drawn. The site is `gui/composer_flow.go`'s Back leg,
  NOT `gui/composer_shape.go` as the earlier controller note guessed. Class:
  **W-6 CHANGE (Important)** -- the preset screen (six archetypes, F-453/S0b)
  is reachable once per composition and the only route back was to discard.
- **W-7 found while measuring it (Critical, funds-relevant).** The same leg
  assigned `st.list.Wrapper` directly, bypassing `composerShapeGuard` (§8j)
  and `composerApplyShapeEdit` (the discard). Measured with
  `md.Composed.Slots()` on [Path 1: 2-of-2, Path 2: a single key]:
  `wsh -> [{@0 p0 o0} {@1 p0 o1} {@2 p1 o0}]`,
  `tr -> [{@0 p1 o0} {@1 p0 o0} {@2 p0 o1}]` -- equal COUNT, permuted mapping,
  so `composerSizeAssignments` kept every seat and the key seated as "Path 1
  key 1 of 2" became Path 2's sole spending key. Reachable via §8p's "What
  now?" -> "Back to the paths", which lands on the path list with seats held.
  No screen says so (`composerMappingLines` prints index and origin, never the
  path). §7d's rule was met by the path list's own "Change the script" row and
  unmet one function away.
- **Both FIXED** on fork branch `composer-s4e` = `05466727` (worktree
  `/scratch/code/shibboleth/wt-composer-s4e`, off `70008da`): `composerStartStep`
  walks §7b's opening pair and IS the Back leg, entered at the preset screen.
  Back is now the inverse of the way in; the blank row KEEPS the paths on the
  second pass; the choice goes through `composerApplyShapeEdit` after §8j is
  asked whenever the shape signature would move.
- Failing-first tests in `gui/composer_backleg_test.go`; four mutations of the
  fix each caught by their own named assertion (drop §8j; blank the list on the
  blank row; wrapper picker alone; assign without `composerApplyShapeEdit`).
  Audited mechanically: every other production assignment to `st.list` goes
  through `composerApplyShapeEdit` except `composerMoveUp`, which discards
  unconditionally on purpose -- the Back leg was the ONLY unguarded one.
- Controller gate GREEN on `05466727` (`.tmp/s4e-gate.log`): gofmt -l cmd/
  clean (gui/ residue = the three pre-existing transaction*.go, review r0 N-2);
  vet = the two pre-existing go1.25 ArtifactDir lines only; `go test ./...`
  0 FAIL; gui shards **1195** (1192 + 3 new); 32-bit exit 0 both; `go build
  ./cmd/...` 0; firmware **1,581,428 B flash / 62,800 B RAM = +224 / +0** over
  `70008da`.
- Engrave records `e930ee7`: walk record (W-6 measured + classified, W-7 its
  own entry), spec §7b folded with the Back rule (it was SILENT about Back at
  the path list, which is why no gate caught either), F-470 filed (a preset row
  replacing hand-built paths with nothing seated is unconfirmed -- the
  operator's call, deliberately not invented in a fix branch). Spec gates:
  structure OK 56/49, glyph 114/0, cite ok.
- **IN FLIGHT:** the independent verification, **opus** (not sonnet: W-7 is a
  Critical about which key seats into which slot, and the controller authored
  the fix with no independent implementer, so this is design-level adversarial
  review on risk-set work). Brief
  `design/agent-briefs/composer-S4-W6-verification-brief.md` ->
  `design/agent-reports/composer-S4-W6-verification.md`. If that file exists on
  resume, persist it (own commit); if not, re-dispatch with the brief.

Steps, in order:
1. Persist the verification report (own commit, verbatim). If 0C/0I: dispatch
   the merge agent (sonnet) with
   `design/agent-briefs/composer-S4-W6-merge-push-brief.md` (tip `05466727`,
   message `composer-S4-W6-merge-message.txt`) -> fork main; then remove the
   worktree (`git -C /scratch/code/shibboleth/seedhammer worktree remove
   wt-composer-s4e`) and mark W-6/W-7 shipped in the walk record. If it is not
   0C/0I: fold on `composer-s4e`, re-gate, re-verify.
2. Push engrave master (sonnet push agent, `scripts/push-via-staging.sh
   master`, FREEZE) -- unpushed since `9bebe05`: `aeb29c8`, `e930ee7` and the
   records after them.
3. Flash at the operator's word only: after the merge, fork main carries W-2 +
   W-3 + W-4 + W-6 + W-7 and is the build for the rest of the device walk
   (`~/bin/sh/sh2-flash -y`, device in BOOTSEL, expected version line
   `bg<sha>` (UNLOCKED)). The device currently runs `bgbb50775` (W-4 branch
   build), on which the Back leg is still the shipped one.
4. The rest of the S4 residue, unchanged, all at the operator's word: the pad
   re-check on the blocks/date pads, the Taproot 2-of-3 plate when a blank is
   on hand (string
   `md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3`, byte for byte),
   Task 5 (Part B on the device).
5. The hashlock cycle stays PAUSED (L19) until the operator gives the word for
   `mnemonic-secret/design/SPEC_ms_hashlock.md`.
6. Owed to the operator: F-470's ruling (above), F-454 (me 0.8.1), F-455/F-457/
   F-459/F-462/F-463/F-464 polish, the three S3 defaults (implemented).
Command to resume: /resume-composer

- **VERIFICATION ROUND 1 (opus) SAID DO NOT MERGE: 1C/1I/1M** (report persisted
  `cb6b337`). W-6 closed; W-7 closed for the wrapper; **C-1 was a Critical the
  W-6 fix INTRODUCED** -- the preset rows it newly made reachable from the path
  list carried every seat across a tr renumbering with no §8j, because
  `composerShapeSignature` re-derived md's numbering rule (§7d's enumeration)
  while `lowerTr` picks the internal key with `isBareSingle()` and numbers it
  ahead of listed order. Controller re-derived it independently: hand-built
  `[2-of-2, 1 key, 1 key]` and the `decaying-multisig` preset both sign
  `w0/2,1,1,` and disagree on three of four slots. I-1 (Important,
  pre-existing at `70008da`) is the same root cause on the path editor's lock
  arm. M-1: the leg's sole exit was untested.
- **FOLDED at fork `818220d8`** (records `ca66d0b`): `composerShapeSignature`
  now carries `md.Composed.Slots()` itself, with the structural terms kept only
  as the fallback for a list the codec refuses (an edit into or out of a refused
  shape reads as a move, which discards -- the safe direction);
  `composerEditCanRenumber` asks the CODEC whether a lock/hash edit on a path
  can move the mapping, rather than restating `isBareSingle()`; the lock and
  hash arms are wrapped in `composerApplyShapeEdit` behind that answer, so §8j
  still does not fire on a wsh lock edit (§7g calls it DEFAULT, and the shipped
  test pins it); M-1 pinned by a test. Spec §7d's enumeration replaced by the
  codec's answer; §7b refined.
- Gate on `818220d8` (`.tmp/s4e-gate2.log`): gofmt cmd/ clean, vet = the two
  pre-existing lines, `go test ./...` 0 FAIL, **shards 1199** (1195 + 4 new),
  32-bit exit 0 both, `go build ./cmd/...` 0, firmware **1,582,564 / 62,800 =
  +1,360 B flash / +0 RAM over 70008da**. Four mutations of this fold each
  caught by their own named assertion, including "ALWAYS ask on the lock arm",
  which the shipped wsh test catches -- the guard must not over-fire.
- **IN FLIGHT: verification round 2 (opus)**, brief
  `design/agent-briefs/composer-S4-W6-fold-verification-brief.md` ->
  `design/agent-reports/composer-S4-W6-fold-verification.md`. Its highest-value
  item is the THIRD DOOR: the class is "the GUI decides something the codec
  decides", now seen twice. If that report exists on resume, persist it (own
  commit); if not, re-dispatch with the brief.
- **THE OPERATOR PUT THE SH2 IN BOOTSEL 2026-09-04 and was told to take it out
  again**: there was nothing safe to flash. `composer-s4e` carried the open C-1
  at that moment, and C-1's route (Back -> `Start from?` -> a preset) is exactly
  what the operator would walk to check W-6; fork main `70008da` is the same gui
  bytes the device already runs as `bgbb50775`. Flash only after the merge, at
  the operator's word.
- Steps unchanged from the resume point above, except that step 1's merge waits
  on ROUND 2 being 0C/0I, and the merge brief now points at tip `818220d8`.

- **VERIFICATION ROUND 2 (opus) ALSO SAID DO NOT MERGE: 0C/2I/2M/1N** (report
  persisted `da1f812`). C-1, I-1 and M-1 all FIXED, and the signature's root
  cause closed STRUCTURALLY (the reviewer: 0 equal-signature renumbering pairs
  over 4,828 composable lists; 0 over 28,948 preset x hand-built pairs). But
  both Importants were inside the fold's own new `composerEditCanRenumber` --
  **the defect class had moved into the remedy.** It cleared the HASH in both
  variants while varying only the lock, so it answered a question about a path
  it had already changed: I-2, on a key-less path both variants collapse to the
  same refused shape, so the hash arm asked nothing and the new
  `composerApplyShapeEdit` wrapper discarded EVERY SEAT with no §8j and no
  chance to decline (a regression against `05466727`); I-3, on a tr path
  carrying a hash no lock can affect `isBareSingle`, so §8j fired, cleared
  nothing, and declining left the lock uneditable. Measured over 14,092 pairs:
  1,200 false negatives, 288 false positives.
- **FOLDED at fork `177b4906`** (records `8553ce3`): the probe varies ONLY the
  field its arm edits (`composerFieldLock`/`composerFieldHash`) and each arm
  passes its own. The reviewer's census is COMMITTED as
  `TestComposerEditCanRenumberIsExactOverEveryReachableShape` -- 3,708
  (list, path, field) cases with an oracle independent of the probe (it sweeps
  the values each SCREEN produces rather than comparing two points): 0/0 on the
  fix, 156 false negatives / 288 false positives on the probe it replaced (the
  288 match the reviewer's count exactly). The call-site wiring, which the
  census cannot see, is pinned by
  `TestComposerHashEditOnAKeylessPathAsksBeforeItDiscards`; both field-swap
  mutations are caught. M-2/M-3 comment corrections;
  `composerMoveUp`'s premise re-measured (`w1/1,1,|0.0/1.0/` before and after a
  swap, so its unconditional discard is still load-bearing). N-1 filed as F-471.
  Spec §7d refined again (the probe varies that field alone; a refused shape has
  no mapping and an edit into or out of one counts as a move).
- Gate on `177b4906` (`.tmp/s4e-gate3.log`): gofmt cmd/ clean, vet = the two
  pre-existing lines, `go test ./...` 0 FAIL, **shards 1201**, 32-bit exit 0
  both, `go build ./cmd/...` 0, firmware **1,582,628 / 62,800 = +1,424 B flash
  / +0 RAM over 70008da**.
- **IN FLIGHT: verification round 3 (SONNET, targeted)**, brief
  `design/agent-briefs/composer-S4-W6-fold-r2-verification-brief.md` ->
  `design/agent-reports/composer-S4-W6-fold-r2-verification.md`. Sonnet and not
  opus because what is left is mechanical false-PASS hunting: the controller
  wrote both the probe and its census test, so the decisive experiment is
  running that test against round 2's probe restored in a copy -- it must FAIL
  there. A clean round CLOSES this loop (do not keep looping for reassurance);
  then merge with `composer-S4-W6-merge-push-brief.md` (tip `177b4906`).
- Round tally, for the record: r1 opus 1C (introduced by the W-6 fix), r2 opus
  2I (introduced by r1's fold), r3 sonnet pending. Each round found the defect
  class one level inside the previous remedy. That is the argument for the
  committed census test: it turns "is the probe exact?" from a review question
  into a command.

- **ROUND 3 (sonnet, targeted): MERGE, 0C/0I/1M** (report persisted `0605cae`).
  I-2 and I-3 closed. The false-PASS hunt ran the experiment the brief named:
  the census test FAILS against round 2's exact probe restored in a copy (156
  false negatives / 288 false positives, matching the fold commit's figure) and
  passes 0/0 on the fix; the oracle is structurally independent of the probe;
  both call sites are pinned by tests that fail on either field swap. Its one
  Minor was the M-2 comment fold listing the Move arm among the routes through
  `composerApplyShapeEdit` -- it is not, `composerMoveUp` discards
  unconditionally -- folded inline at `618f86f1` (a wording fold does not
  re-trigger the gate).
- **W-6 and W-7 CLOSED at fork `composer-s4e` `618f86f1`**, four commits over
  `70008da`. Merge agent (sonnet) DISPATCHED with
  `design/agent-briefs/composer-S4-W6-merge-push-brief.md` (message
  `composer-S4-W6-merge-message.txt`) -> `composer-S4-W6-push-report.md`. If
  that report exists on resume, persist it and verify `origin/main` yourself.
- AFTER THE MERGE, in order: (1) persist the push report; (2) remove the
  worktree `git -C /scratch/code/shibboleth/seedhammer worktree remove
  wt-composer-s4e`; (3) push engrave master via
  `scripts/push-via-staging.sh master` with the tree FROZEN (unpushed since
  `9bebe05`: aeb29c8, e930ee7, e90b3c0, cb6b337, da1f812, 0605cae and the rest
  of this session's records); (4) TELL THE OPERATOR the build is ready and
  flash fork main at their word -- `~/bin/sh/sh2-flash -y`, device in BOOTSEL,
  expected version line `bg<merge sha>` (UNLOCKED). That build carries W-2, W-3,
  W-4, W-6 and W-7 and is the one the rest of the device walk needs; the device
  is on `bgbb50775`, where the Back leg is still the shipped one.
- Review tally for this fix, worth keeping: r1 (opus) 1C introduced by the W-6
  fix; r2 (opus) 2I introduced by r1's fold; r3 (sonnet) 1M introduced by r2's
  fold. Each round found the class one level inside the previous remedy and
  each remedy was smaller than the last. The durable artifact is
  `TestComposerEditCanRenumberIsExactOverEveryReachableShape`, which turns "is
  the probe exact?" into a command.

- **W-6/W-7 SHIPPED 2026-09-04: fork main `839fa5aa719b8ec6970655530b74e1e3a3b73a36`**
  (merge --no-ff of `composer-s4e` `618f86f1`; runs 33896299380 Test and
  33896299483 Build image both success; report persisted). Controller verified
  `origin/main` == local main independently. Worktree `wt-composer-s4e` removed
  (branch kept, merged). Walk record marks W-6 and W-7 SHIPPED.
- **FLASH TARGET is now fork main `839fa5aa`** -- it carries W-2, W-3, W-4, W-6
  and W-7 and is the build the rest of the device walk needs. The device runs
  `bgbb50775`, on which the Back leg is still the shipped (defective) one.
  `~/bin/sh/sh2-flash -y` with the device in BOOTSEL, at the OPERATOR'S WORD
  only; expected version line `bg839fa5a (UNLOCKED)`. Firmware on this main:
  1,582,628 B flash / 62,800 B RAM (+1,424 B flash / +0 RAM over 70008da).
  After the boot is confirmed, the device checks worth making are: Back at the
  path list lands on `Start from?`; Back again on the script choice; picking a
  script shows `Start from?` again; and the blank row keeps the paths already
  built.
- NEXT: push engrave master via `scripts/push-via-staging.sh master` (FREEZE),
  then the device walk resumes -- the pad re-check, the Taproot 2-of-3 plate
  when a blank is on hand (string
  `md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3`, byte for byte),
  Task 5 at the operator's call. The hashlock cycle stays PAUSED (L19).

- Engrave master pushed **`3d55279`** via ci/staging (required context green,
  no bypass, staging ref deleted; report persisted). Controller verified
  `origin/master` itself. The commits after `3d55279` are records only.
- **NOTHING IS IN FLIGHT.** The session's remaining work all needs the operator
  or is paused: (a) flash fork main `839fa5aa` at their word (device in
  BOOTSEL, `~/bin/sh/sh2-flash -y`, expected `bg839fa5a (UNLOCKED)`), then the
  four W-6/W-7 device checks -- Back at the path list lands on `Start from?`;
  Back again on the script choice; re-picking a script shows `Start from?`
  again; the blank row keeps the built paths -- plus the adversarial one: seat
  a key, Back, pick a PRESET, and confirm §8j fires and the seats are cleared;
  (b) the pad re-check and the Taproot 2-of-3 plate when a blank is on hand;
  (c) Task 5 at the operator's call; (d) the hashlock cycle, PAUSED before spec
  (L19).

- **FLASHED 2026-09-04 at the operator's word** (device confirmed in BOOTSEL by
  `lsusb`: `2e8a:000f Raspberry Pi RP2350 Boot`): `~/bin/sh/sh2-flash -y` built
  fork main `839fa5a` in the devshell, signed it (key matched the burned OTP
  fingerprint `846aa289…`), and loaded
  `seedhammerii-v0.0.0-bg839fa5a.signed.uf2`, sha256
  `986bf0ce98e1c1220c27a3dfadf462e316bb2e6693d0602589bfe97dc5d86955`;
  `picotool load --verify` 100%, exit 0. Log
  `.tmp/sh2-flash-839fa5a.log`.
  **Boot judgement is on MACHINE power and is the operator's** -- Init() wants a
  20-28 V USB-PD contract before it configures the LCD and reboots to BOOTSEL
  without one, so a laptop port gives a dark screen that is indistinguishable
  from a signature rejection. Expected version line `bg839fa5a (UNLOCKED)`.
  Device checks once it boots: (1) Back at the path list lands on `Start from?`;
  (2) Back again on the script choice; (3) re-picking a script shows
  `Start from?` again; (4) the blank row keeps paths already built; and the
  adversarial one -- seat a key, Back, pick a PRESET, and §8j must fire and
  clear the seats (before this build it kept them silently, on slots serving
  different paths).

## RESUME POINT 2026-09-04 -- L19 LIFTED; SPEC_ms_hashlock DRAFTED; R0 round 0 (three opus lenses) IN FLIGHT

State, all measured:
- **The operator lifted L19**: "I want to pivot to the spec for ms1 string for
  preimage backup" (2026-09-04, after W-6/W-7 shipped and `bg839fa5a` was
  flashed). Then: "Switch to fable for coordination" -- the controller is Fable
  from that message on; subagent tiering is unchanged (opus for spec/plan lenses
  and whole-diff reviews, sonnet for fold verification and pushes).
- **S4 residue, unchanged and at the operator's word**: the `bg839fa5a` boot
  judgement on machine power; the four W-6 device checks and the W-7
  adversarial one (seat, Back, pick a preset -> §8j fires and clears); the pad
  re-check; the Taproot 2-of-3 plate when a blank is on hand; Task 5. Engrave
  master pushed through `107839c` (S4 plan §4 firmware pin -> 1,582,628 /
  62,800 at `839fa5aa`); the commits after it are records only.
- **`mnemonic-secret/design/SPEC_ms_hashlock.md` DRAFTED at `5ba61ca`** (ms
  master; briefs `e128be4`), fourteen sections in the house style of
  `SPEC_ms_v0_2_kofn.md`, written by the controller from brainstorm 4.1-4.6 and
  L1-L23. Machine-checked while writing, at `7fc1e58`: all fourteen §14
  citations re-grepped; the four derivation values recomputed in `python3
  hashlib` and cross-checked in `openssl kdf` (hardened X byte-identical; the
  brainstorm's truncated sha256 pair now written in full: X
  `c4bbcb1fbec99d65bf59d85c8cb62ee2db963f0fe106f483d9afa73bd4e39a8a`, H
  `b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb`); the
  entr-32 single's shape (`ms10entrsq...`, 75 chars, first payload char `q`)
  measured with the shipped `ms`. One brainstorm claim deliberately NOT
  inherited: the >=9-character distance between the `entr` and `hash` id
  codewords is marked MEASURE AT PLAN TIME. Engrave's spec-structure-check does
  not parse `§`-style headings (0 sections), so it is not a gate for this spec;
  it did surface two real things, both fixed before the commit (ambiguous
  cross-document `§8i` references, now "the composer spec's §8i"; two table
  rows with escaped pipes).
- **IN FLIGHT: R0 round 0 = three opus lenses in parallel**, per brainstorm
  4.5 (L17), briefs in `mnemonic-secret/design/agent-briefs/`:
  `ms-hashlock-spec-R0-r0-{correctness,adversarial,tests}-brief.md` ->
  `mnemonic-secret/design/agent-reports/ms-hashlock-spec-R0-r0-{correctness,adversarial,tests}.md`.
  Each brief: one question, the machine-checked facts listed, the other two
  lenses' scope forbidden, the report filename fixed. If a report exists on
  resume, persist it (own commit, in mnemonic-secret); if not, re-dispatch
  with its brief.

Steps, in order:
1. Persist each lens report as it lands (own commit each, verbatim, in
   mnemonic-secret) -> read all three -> machine-check every measurable claim
   -> dedupe across lenses -> bring any finding that touches a RULING to the
   operator (rulings are theirs) -> ONE fold of the spec (own commit; re-run the
   citation re-grep and the derivation recomputation, output in the message)
   -> sonnet fold verification if the fold is non-trivial -> a new lens only
   if a question remains unasked (the journey lens is assigned to the composer
   spec fold, not this spec; enumerate before closing). Closure is lens-closure.
2. Then the H1 plan (`mnemonic-secret/design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md`)
   with a `plan-build-gate-ms.sh` sibling of the me/md gates on the pinned
   toolchain, its own R0 (fidelity opus + tests sonnet), re-validated
   immediately before ONE opus implementer (UC off, a mnemonic-secret
   worktree). The plan's build gate must MEASURE the id-codeword distance (§1).
3. Push engrave master (sonnet, `scripts/push-via-staging.sh master`, FREEZE)
   whenever the tree is clean and no commit is imminent; ms master has its own
   staging ritual (four required contexts) for when the spec closes.
4. The composer spec fold (§6c, §8, §12, §14, C25) under its own R0 with a
   journey lens re-walking W-5 -- AFTER the ms spec closes, per 4.5's order.
Command to resume: /resume-composer
- **R0 round 0 LANDED and FOLDED (2026-09-04).** Three opus lenses on
  `SPEC_ms_hashlock.md`: tests 1C/11I/11M/4N (`d02185e`), correctness
  1C/7I/6M/2N (`e6ef0a0`), adversarial 4C/4I/5M/1N (`4c59d8e`) -- all in
  mnemonic-secret. Every measurable claim re-derived by the controller before
  the fold (the fold commit lists the sites). The six Criticals, one line each:
  the single-string accept set never admitted `hash`, so no preimage plate was
  readable; `is_ms1_shaped` does not case-fold, so an UPPERCASE plate string
  passed the phrase channel; `--random --json | jq` lost the only copy of X at
  exit 0; `--out` truncates, so a second `--random` clobbered an irreproducible
  preimage; **the flashed SH2 CUTS a preimage plate as a seed** (`isStrictMs1`
  has no prefix test; `unlockEngraveCodex32` never calls `DecodeMS1`) and `me`
  will classify one as a secret seed record on the ms-codec bump -- the
  brainstorm's "older readers refuse" premise is measured false; `--hex`
  accepts a seed's entropy as X with no warning able to fire. ONE fold at ms
  `1a14a4d` (message = the machine-check record).
- **r1 fold verification (sonnet) GREEN** (ms `afb5714`): 28/28 C+I fixed in
  the text, three spot-checks reproduced, no new contradiction; its one Nit
  (N-2 wording) folded inline with the STATUS line at ms `d4d6771`.
  **SPEC_ms_hashlock R0 GREEN under lens-closure** (correctness, adversarial
  with journeys, tests/vectors, fold-verification; the journey lens belongs to
  the composer spec fold per 4.5).
- **THREE CONTROLLER DEFAULTS AWAIT THE OPERATOR** (rulings are theirs; each
  is labelled in the spec): (1) §1 rule 2 -- a single whose id and prefix
  disagree is REFUSED (`TagKindMismatch`) rather than dispatched on the
  prefix; (2) §4.1 -- `--random` requires `--out FILE`; `--json` alone no
  longer satisfies the gate (narrows L21); (3) §9 -- **H0**: the fork's
  `isStrictMs1` prefix test (flashed) and `me`'s classifier guard ship BEFORE
  ms-cli 0.18.0 is released (reorders 4.5). A veto folds the section back and
  re-verifies.
- Also measured, unrelated to a ruling: `me sysw pack --in -` is NOT a stdin
  sentinel (exit 2); the no-argument form reads stdin (exit 0) -- the spec had
  it wrong three times. A stray empty `-` directory (Aug 26) was removed from
  the engrave checkout. Process slip, corrected: the first draft of THIS entry
  was appended to mnemonic-secret's hand-off COPY of the continuity file
  (shell cwd) and reverted there (ms `53f6fd0`); this file is authoritative.
- ms master has ~12 unpushed design-only commits (spec, briefs, reports, the
  revert) -- push via the ms hand ritual (four required contexts) when the H1
  plan's first commit is not imminent; engrave master unpushed since
  `107839c`.
- NEXT: the H1 plan, `mnemonic-secret/design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md`
  + `scripts/plan-build-gate-ms.sh` (measures §1's codeword distance), own
  R0 (fidelity opus + tests sonnet, mutation beside every test), re-validated
  immediately before ONE opus implementer (UC off, an ms worktree). H0's fork
  and `me` guards are their own small plans in their own repos and must SHIP
  before 0.18.0.
- **H1 PLAN DRAFTED** (`mnemonic-secret/design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md`,
  committed as DRAFT at ms `dbccbe8` with `scripts/plan-build-gate-ms.sh` and
  `scripts/plan-handwire-ms-hashlock.py`): Task 0 (the gate) + Tasks 1-11
  (codec constants/errors; the kind, dispatch, accept set and tag/kind check;
  the hashlock module; the corpus + the literal-constant three-way repro test;
  the six-part guard; the byte-verbatim reader and phrase rule; the verb; the
  other verbs + the catch-all count; the CLI test matrix; MIGRATION/CHANGELOG;
  release with H0 first). Written with the writing-plans skill: files,
  interfaces, failing test first, run + expected, commit, a mutation beside
  every test; new behaviour in NEW files so the gate extracts them, edits to
  existing files as FRAGMENTS carried verbatim in the hand-wire script.
  **Gate iterating**: runs 1-3 fixed only anchors (envelope doc-comment
  alignment, inspect.rs's real import/match, the two helper call shapes) and
  the lockfile rule for new deps; run 4 (in flight) after adding the
  `validate()` arm the fragments had missed -- a real plan gap the gate
  caught, which is what it exists for. NEXT: gate green -> commit with the
  gate output -> R0 (fidelity opus + tests sonnet, mutation beside every test)
  -> fold -> verify -> re-validate immediately before ONE opus implementer in
  an ms worktree (UC off). Both repos' records are pushed (engrave `2302b9c`,
  ms `d4d6771`); commits after them are records + the plan draft.
- **H1 PLAN R0 ROUND 0 LANDED**: fidelity (opus) 2C/10I/9M/3N (ms `95f417c`),
  tests (sonnet) 0C/4I/3M/1N (ms `2f4a93b`), both persisted verbatim. Both
  Criticals sat where the gate cannot look, in fragment semantics: the three
  new codec errors had no `From<ms_codec::Error>` arm (a `TagKindMismatch`
  surfaced as `unhandled ms_codec::Error variant` at exit 1), and inspect's
  rule-6 relaxation had no tag/kind check outside the `Entr` arm (a forged
  `hash`-over-`0x00` string printed "OK: would decode"). The tests lens found
  18/18 declared mutations caught and the gaps in DEPTH: no case-folding test,
  9/10 corpus rows unfilled with nothing loading the file, a word-blocklist
  "never words" check, the hex-looking-longer-phrase acceptance untested.
  Every measurable claim re-derived by the controller. **ONE fold applied**
  (parts A/B/C: the error arms; inspect's rule-6b check outside the arms; both
  Cargo bumps in Task 1; the forward_compat loop; split.rs's arm; `create_new`
  via O_EXCL; one shape predicate; `--hex` parsed by the verb with §8i and both
  spellings; `--hashlock-phrase -` refused = a FOURTH controller default for
  the operator; the corpus's eleven rows MEASURED (python3, two cross-checked
  in openssl) plus a loader test that re-derives every row; the mixed-case
  row; the structural three-line decode check; honest RED-step wording with
  message assertions). Gate re-runs 12-13 in flight (r12: 75/75 tests, one
  unused import at clippy; r13 after removing it). NEXT: r13 green -> fold
  commit (gate output in the message) -> fill and commit
  `ms-hashlock-H1-plan-R0-r1-fold-verification-brief.md` (sonnet) -> dispatch
  -> STATUS R0 GREEN -> re-validate immediately before ONE opus implementer.
- **H1 PLAN R0 FOLD COMMITTED (ms `3592532`) after gate run 13 GREEN** (75/75
  hashlock tests, clippy + fmt clean, codeword distance 17, downgrade row exit
  2, `MS-GATE EXIT: 0`); the fold message maps every C/I of both round-0
  reports to its change and quotes the gate tail. Round-1 brief committed at
  ms `11fb612`; sonnet fold-verification DISPATCHED (report will land at
  `design/agent-reports/ms-hashlock-H1-plan-R0-r1-fold-verification.md`, its
  own commit). NEXT: persist -> if GREEN, STATUS R0 GREEN on the plan (one
  commit) -> push ms via the hand ritual -> re-validate immediately before
  ONE opus implementer in an ms worktree (UC off), H0 first.
- **H1 PLAN R0 GREEN (ms `4dbff0b`).** r1 sonnet fold verification (ms
  `0c9efa1`): 16/16 C+I FIXED with both Criticals and all four tests-lens
  Importants EXECUTED (mutations reverted), corpus rows re-derived (openssl
  cross-check), the loader test fails on a placeholder; two new Importants,
  both RECORDS — the fold message's Minor ledger was wrong for five items
  (they WERE folded), and the C-1 arms were attributed to three different
  tasks — folded as wording only (no gate re-run; lens-closure: fidelity,
  tests/mutation, fold-verification). Open Minors after both rounds: M-4
  (half), N-1, tests M-3, tests N-1.
- **H0 PLAN DRAFTED + GATE GREEN (engrave `b0af794`)**:
  `design/IMPLEMENTATION_PLAN_hashlock_H0_reader_guards.md`. Hand-wired into
  both repos (no script recognises its paths): Rust 2/2 tests, fmt clean, the
  pin test fails under the mutation; Go codex32/sysw/seal ok, gui TestUnlock
  ok, three mutations each fail the right test — the gui one ends on the
  "Engrave Plate" frame for a preimage plate, which is the whole reason H0
  exists. Engrave pushed `917d4e3` (run 33927403159, no bypass; report
  `e06e29d`). NEXT: push ms (11 commits, hand ritual, master frozen) -> H0 R0
  (opus fidelity + sonnet tests) -> ONE opus H1 implementer in an ms worktree
  (UC off; plan re-validated: staleness check below) -> H0 implementer after
  its GREEN; flash H0 at the operator's word.
- **H1 IMPLEMENTED on ms branch `hashlock-h1` (tip `a150ba7`, worktree
  `ms-worktrees/hashlock-h1`), ONE opus implementer, Tasks 1-10 (Task 11 =
  release, deliberately not run: H0 first). Controller re-ran the gates:
  nextest 554/554, cargo test 555/0, clippy + fmt clean. Eight deviations
  recorded in the branch's `ms-hashlock-H1-implementation-report.md`; D2 is a
  real plan defect (encode.rs check order broke two shipped tests; moved
  below the reserved-tag check, script moved with it), D5 a GUI-schema pin
  the plan never mentioned (55 -> 67). Post-impl adversarial review (opus)
  DISPATCHED against `design/agent-briefs/ms-hashlock-H1-post-impl-brief.md`
  (ms `d9c1e36`); report lands in the MAIN ms checkout. ms master pushed
  `4dbff0b` (four contexts, no bypass; report `c985f02`).
- **H0 PLAN R0 ROUND 0 LANDED AND FOLDED** (fidelity opus 2C/5I/2M
  `1b254c9`; tests sonnet 0C/1I/3M `a7aebdc`; fold `fdfb040`, gate re-run
  green in both repos). The two Criticals were real and the gate could not
  see either: (C-1) the NFC scan door and the typed M*1 STRING door both
  reach `engraveCodex32` -> `EngraveSeedString` titled HASH with no guard --
  the plan had guarded only the sealed path; (C-2) `IsPreimage` read ANY
  string's first payload byte, so ~1/256 of legitimate shares and plain
  BIP-93 secrets would go inert and one such share refuses a whole sealed
  payload -- and none of the 82 ms1 fixtures in the fork has payload[0]==0x03,
  so every gate stayed green. Fold: predicate = unshared && 33-byte payload
  && 0x03 (id not consulted); Scan mirror narrowed; named refusal at the
  choke point; corpus 9 -> 12 rows (sha f1f2fa6b...391c) built with
  `codex32.NewSeed` -- a plain secret and a share beginning 0x03 (device
  true), the shape under id `entr` (device false); a 33-byte 0x31 fixture
  added when the `!= msPrefixEntr` mutation SURVIVED the first table; me's
  diagnosis names the kind (`UnknownReason::PreimagePlate`). r1 sonnet fold
  verification DISPATCHED. NEXT: H1 post-impl -> fold -> merge hashlock-h1
  -> push ms; H0 r1 -> STATUS GREEN -> ONE opus implementer (fork branch
  hashlock-h0 + engrave) -> post-impl -> flash at the operator's word.
- **H1 POST-IMPL REVIEW (opus) NOT GREEN: 2C/3I/6M/2N (ms `b776253`).** C-1:
  `json_both_variants`'s advisory assertion is satisfied by the engraving
  card's own "the secret" line, so deleting the `PrivateKeyMaterial` advisory
  leaves 554/554 green (a test that cannot fail on §4.4/§11). C-2: the
  implementer's report claimed nextest 535 (Task 8's number) at the final
  gate, measured 554, with a false explanation. I-1: the terminal prompt says
  "then Enter" but `read_to_end` needs EOF (pty transcript: hangs). I-2:
  `--separator` has no `parse_separator`. I-3: under `--allow-argv-secret`,
  `--hashlock-phrase` with a missing value swallows `--json` and derives a
  preimage from the string `--json` at exit 0. All eight deviations verified
  TRUE; 11/12 mutations killed. The SAME implementer was RESUMED with the
  controller's decisions (advisory asserted under `--no-engraving-card`; the
  terminal branch reads one line, prompt text unchanged; `parse_separator`;
  the guard refuses a missing/flag-shaped value at exit 64; report counts
  re-measured at the tip). NEXT: sonnet fold verification on the branch diff
  -> merge -> push ms.
- **H0 PLAN r1 NOT GREEN -> r1 FOLD (`64a6e0d`) -> r2 DISPATCHED.** The r1
  verifier (`97dab8c`) confirmed all 8 C+I fixed and found ONE new Important
  that was MINE: the round-0 fold message claimed "whole crate 615/616" when
  the true state was 610/616 -- the four seam rows are enumerated by
  `tests/record_corpus.rs` (S2's invariant-2 capture, 33 records) and red
  three of its tests; nextest without `--no-fail-fast` stopped at the first
  failing binary and my grep cut the output at three lines. Fold: Task 1 Step
  1b extends the capture 33 -> 37 (class Unknown, consult record-refusal, the
  invariant-2 argument: added, not moved); a Global Constraint fixes the
  measurement method (`--no-fail-fast`, OWN target dir per worktree -- a
  shared target handed the run seven vector-test binaries compiled from a
  reviewer's deleted worktree -- `touch` after restoring a backup). Measured
  clean: 616 run, 613 passed, the 3 failures all `history_purge` and
  identical on untouched master (no /usr/bin/zsh). Memory:
  shared-target-dir-across-worktrees-bakes-paths. r2 (sonnet, narrow)
  dispatched against `hashlock-H0-plan-R0-r2-fold-verification-brief.md`.
- **H1 FOLD DONE (ms `hashlock-h1` tip `447eb09`, nine commits, same
  implementer):** every C/I reproduced before its fix; C-1's advisory test
  now fails when the advisory is deleted; I-1's terminal branch reads one
  line (prompt text unchanged); I-2 binds `parse_separator`; I-3's guard
  refuses a flag-shaped value at exit 64 (`Decision::Usage`); M-3(c) closed a
  real hole (`mode(0o600)` mutation now fails one test); M-1/M-2 filed as
  follow-ups (secret-handling ruling). Controller re-ran CI's commands at the
  tip in an ISOLATED target dir: nextest 559/559, cargo test 560/0, clippy
  0, fmt 0 on both toolchains. Sonnet fold verification DISPATCHED (brief ms
  `81beaec`). The implementer also hit the shared-target trap (three tests
  red at a150ba7 from a reviewer's deleted worktree; `cargo clean -p` fixed it).
- **H0 PLAN R0 GREEN (engrave `e7af98a`)**: r2 sonnet GREEN (`b4c4090`), one
  wording fold (the PreimagePlate message no longer asserts `id hash`).
  Lens-closure: fidelity, tests/mutation, fold-verification x2. ONE opus H0
  IMPLEMENTER DISPATCHED (brief `60b4cfb`): branches `hashlock-h0` in
  `me-worktrees/hashlock-h0` (own target dir) and
  `.tmp/seedhammer-hashlock-h0`; stops after the firmware size; the emulator
  walk, merge and flash are the controller's/operator's. NEXT: H1 r1 ->
  merge hashlock-h1 -> push ms; H0 impl -> post-impl (opus) -> emulator walk
  -> fork PR/merge -> flash at the operator's word -> then ms-cli 0.18.0.
- **H1 MERGED to ms master (`7d12102`, --no-ff; plan record `1e3d6df`).**
  r1 sonnet fold verification GREEN (ms `576fae9`: 5/5 C+I fixed, every
  added test fails on its defect, no argv_guard regression). Controller
  gated the MERGED tip in an isolated target dir: nextest 559/559, clippy 0,
  fmt 0 on both toolchains. NOT RELEASED: ms-codec 0.8.0 / ms-cli 0.18.0 stay
  unpublished until H0 is merged and flashed (spec §9 default). The H1 plan
  carries an IMPLEMENTATION RECORD (D2 encode-order and D5 schema-pin plan
  defects). ms push of 27 commits DISPATCHED (hand ritual, four contexts;
  master frozen). Engrave pushed `423b276` (run 33932528219, no bypass;
  report `92f2ec1`). Worktree `ms-worktrees/hashlock-h1` removed; branch
  kept. H0 implementer still running.
- **ms PUSHED `1e3d6df` (27 commits; four required contexts green, no
  bypass; report `351a75e`) -- and the NON-required `vendor/ satisfies
  Cargo.lock (offline)` check went RED** (`no matching package named
  pbkdf2`): H1's dependency bump rewrote Cargo.lock and nobody re-vendored.
  Reproduced locally with `bash ci/repro/vendor-freshness.sh`; fixed with
  `cargo vendor vendor/` (eleven new crate dirs, nothing modified), gate OK,
  committed as its own commit (ms `8796d69`); second push DISPATCHED with
  the instruction that vendor-freshness must ALSO be green on the staged
  SHA. Memory: ms-cargo-lock-change-needs-cargo-vendor (run the script
  BEFORE a push; the ritual does not stop on a non-required check). The H1
  plan's Task 1 never said "re-vendor" -- records commit after the push
  window.
- **H0 IMPLEMENTED (engrave `hashlock-h0` tip `265dc8e`: be72e75 corpus +
  pin + diagnosis, 19b2a58 records F-472/473/474 + CHANGELOG, 265dc8e the
  report; fork `hashlock-h0` tip `14afdff`, ONE commit).** Every plan RED
  reproduced verbatim; all nine MUTATION claims re-run and failed on their
  line; engrave 616/613 (the 3 box-local history_purge), fork vet clean,
  three packages ok, 1204 gui tests via the shard script; firmware
  1,583,100 flash (+472 B), 62,800 RAM. Five minor deviations (absolute
  vendoring path; gofmt column realignment; trailer order after `-s`; a
  whitespace-only frame quote; the local clippy lint). NOT done, by design:
  emulator walk, merge, flash, post-impl review. Controller re-run of both
  gates and the emulator build from the fork branch IN FLIGHT; post-impl
  brief committed (`fbc66bc`); walk script drafted at
  `.tmp/h0/walk_h0_preimage.js` (typed door -> "Hashlock preimage"; NFC ->
  "Unknown format"; guarded against any engrave screen). NEXT: dispatch
  the opus post-impl; run the walk (serve cmd/emu on a FRESH port, drive
  via playwright); fold; merge both branches; push engrave; fork PR/merge;
  flash at the operator's word; then ms-cli 0.18.0 (Task 11).
- **H0 EMULATOR WALK PASSED (Task 3 Step 2).** emu.wasm built from fork
  `hashlock-h0` (14afdff), served on a fresh port, driven via playwright
  with `cmd/emu/walk_h0_preimage.js` (committed on the fork branch as its
  second commit). Typed M*1 STRING door with the plate -> frame
  "This record is a hashlock preimage, not a seed. It is not engraved as
  one." / "Hashlock preimage"; NFC door (`shNFC.present(plate)`) -> "Unknown
  format" on the start screen; no engrave/confirm frame in either trail;
  ok:true. (First run's ok:false was the script comparing squashed text to
  a spaced needle -- fixed; the device's behaviour was identical.) CONTROL
  in flight: the same walk against an emulator built from unguarded main
  839fa5aa must reach "Confirm Codex32 Secret". Controller gate re-run at
  both tips: engrave 613/616 (history_purge x3, box-local), fmt clean,
  seam sha f1f2fa6b...391c; fork vet + three packages + gui subset ok,
  vendored corpus identical. Post-impl (opus) IN FLIGHT.
- **H0 WALK CONTROL DONE -- the walk can fail.** Same `walk_h0_preimage.js`
  against emu.wasm built from unguarded fork main 839fa5aa: the typed door
  ends on "Confirm Codex32 Secret / id HASH / Unshared secret (S) / 75
  chars" (the walk fails with waitFor("Hashlock preimage") timed out, screen
  reads exactly that), and the NFC door on main goes from the start screen
  straight to the SAME confirm frame (trail: SeedHammer -> Confirm Codex32
  Secret id HASH Unshared secret (S) 75 chars). That is the spec §9 reader
  table's "NOT a refusal", now measured at both doors: on the flashed
  device today a preimage plate is one button from being cut as a seed.
  Fork branch: 14afdff + 45f3d4c (the walk). Servers stopped. NEXT:
  post-impl (opus, in flight) -> fold -> merge engrave hashlock-h0 (merge
  commit; master moved) + fork hashlock-h0 -> push engrave -> flash at the
  operator's word (sh2-flash -y, BOOTSEL) -> boot judgement -> ms-cli
  0.18.0 (Task 11).
- **H0 POST-IMPL (opus) NOT GREEN: 1C/2I/4M/1N (`87771f6`).** C-1 is real
  and the emulator walk could not see it: `engraveCodex32`'s guard runs
  ONCE before the loop, and the Recover arm reassigns `scan` from
  `Interpolate(shares, 'S')` inside it -- a preimage single recovered from
  K-of-N shares walks to "Confirm Codex32 Secret" and "Engrave Plate"
  (reproduced in the gui harness; no test drove Recover). I-1: a plain
  BIP-93 33-byte seed beginning 0x03 is refused while the doc comment says
  16..32-byte seeds are untouched -- the BEHAVIOUR stands (that shape IS a
  preimage plate by construction; me refuses the same string), the WORDS
  and a corpus row change. I-2: `me seal` still refuses with the raw codec
  error (the named diagnosis was wired only into `sysw pack`). All 13
  mutations killed their tests; firmware byte-identical; corpus identical;
  the 0.8-bump simulation gave five witnesses (plan claimed three). SAME
  implementer RESUMED with the decisions (guard at the top of the loop +
  a Recover test; honest wording + the collision row + capture 38; a
  `RecordError::PreimagePlate` arm for `me seal`; N-1 pin-test
  discrimination). NEXT: sonnet fold verification -> merge both branches
  -> push engrave -> flash at the operator's word.
- **H0 FOLD DONE (engrave `hashlock-h0` 95cd48a; fork `hashlock-h0`
  83fbc17; seven commits, same implementer).** C-1 closed at the root (the
  guard at the top of `engraveCodex32`'s loop; a Recover-arm test that
  reproduced the reviewer's frame byte for byte, shares driven UPPERCASE
  because codex32 needs one case across a set); I-1 words + the 13th
  corpus row `bip93-plain-33-byte-payload-0x03` (host false / device
  false; capture 38; sha re-pinned both sides); I-2 `RecordError::
  PreimagePlate` so `me seal` names the kind (N-1 closed with it); M-1/
  M-3/M-4 wording. 14 mutations, no survivors. Gates: engrave 617/614
  (history_purge trio), fork green with 1205 gui tests; firmware
  1,583,132 / 62,800 (+504 B vs main). Implementer's correction: the
  corpus flip that bites the RUST seam test is `host_admits`, not
  `device_admits` (the Rust test cannot call Go). Sonnet fold verification
  brief committed; dispatch next; controller gate re-run in flight.
- Controller gate re-run at the folded tips (isolated target dir): engrave
  95cd48a 617 run / 614 passed (history_purge trio), fmt clean, seam sha
  bb703f608215bb00ccc677de4a282772016e774dd2d1d0f5c828ea38f5eac78b pinned
  in BOTH repos, 13 rows, capture 38; fork 83fbc17 vet ok, three packages
  ok, gui subset ok, gofmt clean, corpus byte-identical. Sonnet fold
  verification in flight.
- **H0 MERGED. READY TO FLASH AT THE OPERATOR'S WORD.** r1 sonnet fold
  verification GREEN (`7c68daf`). Engrave: `hashlock-h0` merged to master
  `024dd08` (--no-ff), plan record `0c9d005`. Fork: `hashlock-h0` merged to
  main `c4a64fc` (--no-ff; tree identical to the tested branch 83fbc17) and
  PUSHED to origin (unprotected, plain push). Firmware at fork main
  1,583,132 flash / 62,800 RAM (+504 B vs 839fa5aa). Worktrees and scratch
  removed. NEXT: push engrave master via the ritual (dispatching); then
  `~/bin/sh/sh2-flash -y` with the SH2 in BOOTSEL -- ONLY at the operator's
  word; boot judgement is the operator's; then ms-cli 0.18.0 (H1 Task 11:
  ms-codec 0.8.0 + ms-cli 0.18.0 release) and the FOUR controller defaults
  still await the operator (TagKindMismatch refusal; --random requires
  --out; H0-before-0.18.0 ordering, now honoured; --hashlock-phrase -
  refused).
- **OPERATOR: "Let's assume it booted."** (2026-09-05, after the engrave push
  `364b864` landed; report `bc913d2`.) Recorded as stated: the controller
  has NOT flashed anything; the H0 firmware is merged and pushed at fork
  main `c4a64fc` (1,583,132 / 62,800), and the boot is ASSUMED at the
  operator's direction. The H0-before-0.18.0 ordering is therefore
  satisfied by the operator's word, not by a measured boot; if the device
  is later found on `bg839fa5a`, flash before any preimage plate exists.
  Proceeding to H1 Task 11 (release) prep: the Step 1 gate evidence, the
  release gate, the publish dry run; the outward steps (release commit
  push, tags, publish) follow.
- **OPERATOR RULINGS L24-L27 (2026-09-05, asked one by one at the
  operator's request; ms `a1e0a6f`):** L24 TagKindMismatch refused (kept);
  L25 `--random` requires `--out FILE` (kept); L26 release order:
  "Release regardless of the device" -- 0.18.0 does not wait for a measured
  flash/boot (H0 is merged and pushed in both repos; the controller flashed
  nothing; a device still on 839fa5aa cuts a preimage plate as a seed until
  flashed); L27 `--hashlock-phrase -` refused (kept). H1 Task 11 in
  progress: release gate (build/nextest/clippy/fmt/vendor-freshness/publish
  dry-run, isolated target) and the `me` H0 evidence build in flight; then
  CHANGELOG date + corpus sha -> release commit -> staging ritual -> tags
  `ms-codec-v0.8.0` + `ms-cli-v0.18.0` -> man-release -> acceptance report.
- **H1 RELEASE COMMIT ms `cd0a60f`** ("release: ms-codec 0.8.0 + ms-cli
  0.18.0 -- corpus SHA pinned; H0 merged; released regardless of the device
  (ruling L26)"): both CHANGELOG entries dated 2026-09-05, corpus sha
  a46c197a...1d30 pinned, Step 1 evidence in the message (fork c4a64fc; me
  6d8ef65 refuses the plate by name at exit 4 on `sysw pack` AND `seal`;
  the plan's Step 1 wording superseded by H0's actual diagnosis -- recorded,
  not re-planned), Step 2 gate at a1e0a6f (559/559, clippy, fmt x2, vendor
  OK, publish dry-run OK). ONE sonnet agent DISPATCHED to: staging ritual
  (five contexts incl. vendor-freshness) -> push master -> tags
  ms-codec-v0.8.0 + ms-cli-v0.18.0 on the pushed SHA (by name, never
  --tags) -> watch man-release -> list assets -> report
  `push-ms-cd0a60f-release.md`. NOT dispatched: `cargo publish -p
  ms-codec` to crates.io (0.7.0 is there; ms-cli last published 0.14.0) --
  the operator's call; H1b (me's bump) needs 0.8.0 published. Acceptance
  (Step 3, spec §12 items 1-6) being run against a local build at cd0a60f;
  re-checked against the released musl binary once the assets exist.
- **ms-cli 0.18.0 + ms-codec 0.8.0 RELEASED (2026-09-05).** ms master
  `cd0a60f` pushed via the ritual (four contexts, no bypass); tags
  `ms-codec-v0.8.0` and `ms-cli-v0.18.0` on that SHA; `man-release` success
  with 7 assets (two musl tarballs, man tarball, PROVENANCE x2, SHA256SUMS
  x2); released x86_64 musl binary re-checked: checksum OK, digests and
  re-derivation identical to the local build. Acceptance report
  `ms-hashlock-H1-acceptance.md` committed with the release report (ms
  master, one commit after cd0a60f). NOT DONE: `cargo publish -p ms-codec`
  (crates.io still 0.7.0) -- the operator's call, asked; H1b (me's 0.8
  bump, F-473) waits on it. NOT DONE: the flash (operator assumed the boot;
  ruling L26). The hashlock-phrase cycle's H1 is SHIPPED; H0 merged; H2
  (device: 0x03 arm + "Type a hashlock phrase" row + method pick) is the
  next stage, unplanned.
- **ms-codec 0.8.0 PUBLISHED to crates.io (2026-09-05T03:28:43Z).** The
  operator said "Proceed autonomously, asking fable architect for blocking
  questions"; the publish was the one blocking decision, routed to a fable
  architect (decision report ms `a4e3b4a`: YES, from the tag's tree,
  --locked, ms-codec only, verify via the API). Done exactly so, from a
  detached worktree at `ms-codec-v0.8.0` (cd0a60f); `max_version 0.8.0`
  confirmed. Follow-up filed in ms: RELEASE_PROCESS.md gets the real
  publish step. Records pushed: engrave `1b0ec7e`, ms `1990648` (+3 record
  commits since, unpushed). Toolkit manual: `ms hashlock` section written,
  `cli-subcommands.list` gains the verb, mk chapter mirrors four post-0.13.0
  encode flags the lint flagged; lint passes flag-coverage; one cspell word
  (`sysw`) being added; toolkit push next. UNBLOCKED: H1b (me bumps to
  ms-codec 0.8 with an explicit Payload::Preimage refusal arm and
  preimage_plate re-pointed; F-473) -- planning next, risk set.
- **H1b PLANNED** (`design/IMPLEMENTATION_PLAN_hashlock_H1b_me_bump.md`,
  uncommitted until its gate runs): me's pin 0.7 -> 0.8 (crates.io);
  `validate_record` matches the decoded payload (`Payload::Preimage` ->
  `RecordError::PreimagePlate` on the SUCCESS path; wildcard arm for the
  `#[non_exhaustive]` enum); `preimage_plate` pin-independent (Ok Preimage
  | ReservedPrefixViolation 0x03); a `TagKindMismatch` diagnosis arm
  (ruling L24) for the `preimage-shape-entr-id` row; me 0.8.1 + CHANGELOG;
  F-473 closes. RED = H0's three tripwires + the seam row going red at the
  bump. Gate (worktree me-worktrees/h1b-gate, own target) IN FLIGHT. Toolkit:
  local master had diverged from origin (two F-324 ci commits from
  2026-09-02 absent locally); the push agent STOPPED correctly (report
  committed); rebased, lint OK, re-dispatched.
- **H1b PLAN COMMITTED, GATE GREEN (`e672194`)**: RED at the bare bump =
  five failures (the three H0 tripwires, the sysw unit test, the seam row);
  Tasks 2-3 green; mutations a/b and the mismatch-arm removal each fail
  their tests; 619/616 whole crate. R0 round 0 DISPATCHED (opus fidelity +
  sonnet tests, briefs committed). Toolkit docs push: the SHA cannot earn
  its required contexts (all three workflows are path-filtered to crates/
  etc.; docs-only commits trigger none), the only job that ran
  (sibling-pin-check, non-required) fails on a PRE-EXISTING deliberate pin
  exception (cross-tool-differential.yml:80, commented); the push agent
  stopped correctly twice. Decision (bypass per the repo's own docs-only
  comments / widen paths / gated touch) routed to the fable architect.
- **H2 SPEC DRAFTED (`design/SPEC_hashlock_H2_device.md`, engrave `bfd042e`):**
  the device leg -- `Type a hashlock phrase` row (label-keyed switch, r2
  C-4), the phrase screen (NewPassphraseKeyboard, a NEW flow, counter n/100
  unclamped), the method pick with both modals (L12), the derivation on the
  countdown screen through a NEW 14-byte-salt driver (r2 M-5), the confirm
  modal (§8i + the reuse lines), a `hashlock` Go package porting
  ms_codec::hashlock with the 0.8.0 corpus vendored + pinned, and
  `DecodeMS1Preimage` with `DecodeMS1` unchanged (r2 C-2). Citations
  re-grepped at fork c4a64fc / ms cd0a60f (one fixed: the phrase cap is
  ms-cli's HASHLOCK_PHRASE_MAX_CHARS). R0 round 0 DISPATCHED: fidelity +
  journey lenses (opus; briefs `a3798e4`). Toolkit docs push: the fable
  architect's decision = the staging-PR form (PR #68 precedent; the
  pull_request triggers are unfiltered) -- reports + decision + two
  follow-ups committed in the toolkit; push agent next.
- **H1b R0 round 0 FOLDED (`b7ced42`)**: fidelity 0C/2I (the re-pointed
  predicate was NARROWER than 0.7's -- a 0x03 single under an unknown id or
  with a wrong X length regressed to "outside the profile"; the wildcard arm
  failed OPEN on a future payload kind) + tests 0C/4I (RED narrative said
  "profile arm" where the cause was ADMISSION; a 109-column literal broke fmt;
  the PreimageLengthMismatch family unnamed; six bare-bump failures not five).
  Fold: preimage_plate by SHAPE (unshared, 33 bytes, 0x03) + the codec's
  PreimageLengthMismatch, with a new id_kind_mismatch helper (L24 excluded);
  positive Entr|Mnem arms + a REFUSING wildcard; RecordError::TagKindMismatch
  so me seal names the mismatch; doc/message fixes; Task 4 keeps one
  [Unreleased] and closes F-473 + advances F-454; M-6 (seam prose) filed with
  H2. Gate re-run in the worktree: 9/9 targeted, both verbs name the 50-char
  malformed plate, 619/616. r1 sonnet fold verification DISPATCHED.
- **H2 SPEC R0 ROUND 0 LANDED: fidelity 3C/5I/6M/2N (`3f88280`), journey
  2C/6I/5M/1N (`a70f950`).** Fidelity Criticals: the spec never forbids a
  screen-layer normalisation and its tests use a phrase that is a fixed point
  of seal.NormalisePassphrase (a fold ships green); §2's ms1-shape predicate
  (codex32.New, checksum) is stricter than the host's shape-only
  looks_like_ms1 (a grouped/mistyped plate the host refuses is derived from on
  the device); the sha256 acceptance literal is seven hex chars. Journey
  Criticals: the phrase is the only key to the path and no screen says to
  write it and the method down (§8h names "the preimage", which this route
  cannot produce); nothing relates the derived digest to any preimage the
  operator holds (device/host method mismatch discovered at spend time).
  ONE fold of the spec next.
- **H2 SPEC R0 ROUND 0 FOLDED (`60a86f6`)**: §2 forbids the screen-layer fold
  by mechanism (seal.NormalisePassphrase named) and the lockstep drives the
  three non-fixed-point corpus rows; rule 3 is the host's shape-only
  looks_like_ms1; the sha256 literal corrected (b867db87..edbc96cb); §4.5 gets
  the backup line, the payload relation line and the host-reconciliation line;
  §4.6 states the Back contract once (a loop; false only at Which hash?;
  tests through composerAddPath); §4.7 keeps §8h at Done with a phrase-route
  form; the method modals get a verb and honest numbers; the composer's "never
  derives a preimage" records are named for folding. r1 sonnet fold
  verification DISPATCHED. Also in flight: H1b r1 verification; the toolkit
  staging-PR push.
- **TOOLKIT MANUAL PUSHED (`7e07088c`, PR #69, staging-PR ritual):** the
  `ms hashlock` section, `cli-subcommands.list`, the mk mirror rows and the
  two follow-ups are on origin; `examples`/`test (ubuntu-latest)`/`clippy`
  green on the SHA via the unfiltered pull_request triggers, master
  fast-forwarded with no bypass, PR merged by the fast-forward, ci/staging
  deleted; report committed in the toolkit (`67090e2a`, unpushed record).
  The H1b implementer brief is drafted (`hashlock-H1b-implementer-brief.md`,
  uncommitted, `<PLAN_SHA>` pending r1 GREEN).
- **H1b PLAN R0 GREEN (`eece8a3`)**: r1 sonnet GREEN (`65043f6`; six Importants
  reproduce as FIXED from the plan's text; 2M/1N folded as wording). Staleness
  check vs 0f5ce23: 0 drifted. ONE opus IMPLEMENTER DISPATCHED (brief
  `cc1f34e`): branch `hashlock-h1b` in `me-worktrees/hashlock-h1b`, own target
  dir; stops before the me 0.8.1 release. Gate worktree h1b-gate removed.
  NEXT: implementation -> opus post-impl -> fold -> merge -> push engrave;
  the me 0.8.1 release is a fable/operator decision. H2 spec r1 in flight.
- **H2 SPEC r1 NOT GREEN -> r1 FOLD (`c06a760`) -> r2 DISPATCHED.** The r1
  verifier (`040e85f`) confirmed 5/5 C and 10/11 I fixed and found two
  Importants the round-0 fold introduced: a false mutation claim (the
  separators row is a fixed point of seal.NormalisePassphrase =
  ToLower(Join(Fields))) and a wrong fit-gate citation (the "588" is a
  historical filler measurement in a comment of modal_fits_test.go; the real
  gate is assertModalBodyFits, per-body render + headroom, margin 80, no
  capacity constant). Fold: §2/§7.1 credit the normaliser mutation to the
  case and whitespace rows and give the separators row its own fold; §4/§4.5/
  §7.2/§10 cite the real gate and require every new body in its table with a
  drop order; I-5 completed (the fork comment is this stage's, the composer
  spec sentences are H3's); HOLD not CONTINUE on the confirm surface. Engrave
  pushed `d723cac` (report `79a05a6`). Lesson: two of my own folds now put a
  number or a claim into a spec that a grep would have falsified -- re-grep
  every NEW citation a fold adds, not only the ones the draft carried.
- **H2 SPEC R0 GREEN (`55ee7a4`)**: r2 sonnet found one leftover word (§3
  named the old confirm gesture; §4.5 is HOLD) -- folded as one word with a
  grep as the check (and the STATUS reworded once so its own grep claim held).
  Lens-closure: fidelity/design, journey/adversarial, fold-verification x2.
  NEXT: the H2 implementation plan (fork `hashlock` package + corpus vendoring
  + Which hash? row + phrase/method/derive/confirm screens + DecodeMS1Preimage
  + walk), its build gate (Go, hand-wired), R0, ONE implementer, post-impl,
  emulator walk, fork merge, flash at the operator's word (H4). H1b
  implementer still running.
- **H1b IMPLEMENTED (engrave branch `hashlock-h1b`, tip `278a0e4`: 51f25c9
  Tasks 1+2, 2bb3f3b Task 3, 6f4edf8 records, 278a0e4 report).** RED seen as
  predicted (six F-473 failures + the history trio); four mutations
  reproduced; me 0.8.1 with ms-codec 0.8.0; F-473 closed, F-454 advanced,
  F-475 filed (the seam-prose correction, owning phase H2; measured:
  UnknownTag, not TagKindMismatch). Two recorded deviations (a witness
  comment corrected to the measured fact; Task 2's git add widened to the
  gated tree). Observation: main.rs's UnknownReason match is exhaustive, so
  a new refusal reason without operator words does not compile -- a free
  structural guard. Controller re-run at 278a0e4: 619/616, fmt clean,
  clippy only the pre-existing lint, lockfile moved ms-codec only, both
  verbs exit 4 on the plate. Opus post-impl DISPATCHED (brief 38509a9).
- **H2 PLAN DRAFTED (`38509a9`)**: six tasks (hashlock package on
  seal.NewDeriver + vendored corpus; DecodeMS1Preimage; label-keyed Which
  hash?; the phrase route as one loop with both copy gates; the emulator
  arm; records). Build gate delegated to a sonnet gate agent (hand-wire
  into .tmp/h2-gate, run, report every fix as a finding). OPERATOR (mid-turn):
  "You may use up to 5 agents to help with questions or folds or anything"
  -- concurrency budget 5; two in flight now.
- **OPERATOR (mid-turn, before sleeping): "You may use up to 10 agents and
  have all permissions"; "Use ultracode effort"; "Proceed autonomously and
  try hard not to wait on me. I must sleep"; "Consult fable if needed";
  "Fable may answer for me".** Budget 10 concurrent agents; pushes,
  publishes, tags and releases proceed without a confirmation round; every
  operator-shaped decision goes to a fable decision agent (report in
  agent-reports); the flash still needs a device state only the operator can
  establish unless fable rules otherwise. Ultracode here = five DISJOINT
  lenses in parallel on the H2 plan (fidelity, tests/mutation, journey walk,
  adversarial failure states, coverage/comprehension) then a refute pass,
  then ONE fold -- via Agent dispatches, not the Workflow tool.
- **H1b GREEN AND MERGED (engrave master `8c83e4e`, record `bf80390`).**
  Opus post-impl (`hashlock-H1b-post-impl.md`, persisted `5f4b634`):
  0C/0I/3M/2N; 19 kind-space families through both verbs (6 place, 13
  refuse exit 4), 6 killing mutations, 4 wording-only survivors. Minors
  folded on the branch at `4d5b6b7` (controller, inline): M-1 the CHANGELOG
  and `preimage_plate` doc over-claimed -- `PreimageLengthMismatch` is
  reached only when the string length sits in the profile's length sets
  (X in {16,17,20,21,24,25,28,29,32,33}); X = 18 is
  `Bip93OutsideTheProfile(53)`; claim narrowed, behaviour unchanged. M-3 two
  assertions added (0x03 2-of-N share is NOT a plate; UPPERCASE plate IS).
  N-1 `:137`. N-2 the widened profile sentence says a `hash` id is refused
  for its kind. M-2 (argv guard does not cover a plate; secret-handling)
  filed as F-476. Gate at 4d5b6b7: fmt clean; 619/616/3 box-local
  history_purge/2 skipped; clippy only `is_multiple_of`. me 0.8.1 is
  UNRELEASED on master; the release order is a fable decision in flight
  (`decision-me-0.8.1-release-and-flash-rule.md`, also rules on unattended
  flashing).
- **Pushes**: ms `e96676c` pushed via ci/staging (13 contexts green, run
  33946142767, no bypass; report commit 504ff46 local). Toolkit `67090e2a`
  push (staging-PR form) in flight. Engrave master NOT pushed since
  `d723cac` -- batch with the me 0.8.1 release window (freeze applies).
- **H2 R0 r0 briefs committed** (`da8d9ba`, `fede50f`): fidelity, tests,
  journey, adversarial, coverage, refute -- all with `<PLAN_SHA>` pending the
  gate fold. NEXT: gate report -> fold plan -> commit with gate output ->
  fill SHAs -> dispatch five lenses -> refute -> fold -> sonnet verification
  -> ONE opus implementer on fork branch `hashlock-h2` -> controller
  emulator walk -> opus post-impl -> merge fork main -> push -> flash per the
  fable rule -> H4.
- **me v0.8.1 RELEASED (`f94c903`)** on the fable decision (a30f7c3: release
  now; no unattended flash). Sonnet release agent: master pushed via
  ci/staging (no bypass), tag at f94c903, run 33946853992 all 8 jobs success,
  7/7 assets, minisign + sha256 verified, downloaded me 0.8.1 refuses the
  plate by name. F-454 CLOSED. Engrave origin/master = f94c903; the records
  since are unpushed (batch with the next window).
- **OPERATOR (mid-turn): "Not in boot sel but don't wait on device." and
  "Proceed as if device tests passed for now."** Device acceptances (H0 boot,
  H4 walk) are recorded as ASSUMED at the operator's word, never measured; the
  emulator walk is the measured device gate. Signed H0 image pre-built per
  fable's allowance: `seedhammer/seedhammerii-v0.0.0-bgc4a64fc.signed.uf2`
  (sha256 3df72ebc22b61c5112c02989514f186f7c1d060ac66aa4649894f0d7a382b67c
  -- a signed sha is not an identity across signings). Operator's command
  when ready: `~/bin/sh/sh2-flash /scratch/code/shibboleth/seedhammer/seedhammerii-v0.0.0-bgc4a64fc.signed.uf2`
  (or the H2 merge tip's image, whichever is newest; one flash, not two).
- **H2 PLAN BUILD GATE: GREEN WITH FIXES (12)** (`hashlock-H2-plan-build-gate.md`,
  persisted 6597832). Real findings: the copy-table AST gate needs a row per
  new composerCopy* func (the plan under-counted its own by 4); the confirm
  modal needed spec §4.5's drop-order steps 1 AND 2 (headroom 64 -> 186);
  the §8i modal's title is "Path N hash", not "Hash lock"; a zero-payload
  session swaps the lead, so "Which hash?" is absent from that frame; the
  harness's hold() never releases and EventRouter is ONE global pointer, so
  sequential holds stall silently (new holdConfirm with an explicit release;
  memory `harness-hold-never-releases`); fix #12 a pre-existing test
  (composer_gates_test.go) asserting the old lead -- found only by the whole
  shard set. Firmware 1,595,236 / 62,856 (+12,104 / +56). Fold delegated to
  an opus fold agent (brief `hashlock-H2-plan-gate-fold-brief.md`) with a
  checker script `scripts/h2-plan-blocks-vs-tree.sh` (plan blocks == gated
  tree) as the re-gate. NEXT: fold commit -> fill PLAN_SHA in six briefs ->
  five lenses + refute -> fold -> ONE implementer.
- **H2 PLAN GATE FOLDED (`02abee6`)** by an opus fold agent: 12 gate fixes + a
  13th the gate applied but never logged (`hashHex` -> `hashlockHashHex`: gui
  already declares `hashHex` at seal_fixture_test.go:172, so round 0's block
  was a redeclaration) + 3 prose corrections + 5 import lines. Re-gate is
  `scripts/h2-plan-blocks-vs-tree.sh` (38e21db): 25 blocks (5 whole files by
  diff, 20 fragments verbatim), 0 FAIL, controller re-run in the fold commit.
  Un-gated residue named: Task 5 Step 1's walk is prose, never executed (the
  implementer writes and runs it; the controller re-runs). Fold report
  persisted 23974d3. **R0 ROUND 0 DISPATCHED (five lenses in parallel, briefs
  5685258): fidelity (opus), tests/mutation (sonnet, own scratch .tmp/h2-tests),
  journey walk (opus), adversarial failure states (opus), coverage +
  comprehension (sonnet).** NEXT: persist five reports -> refute pass (sonnet)
  -> persist -> ONE fold -> checker + sonnet fold verification -> GREEN ->
  ONE opus implementer (brief hashlock-H2-implementer-brief.md, fill PLAN_SHA).
- **Rate limit hit (session limit, reset 23:40 Phoenix):** all five round-0
  lens agents were terminated mid-work; two had already written their reports
  (coverage 0C/1I/3M/1N persisted d4d5861; journey 0C/5I/3M/2N persisted
  636c671, counts from headers -- no closing block). The other three
  (fidelity, tests, adversarial) were RESUMED from their transcripts at
  04:55 on 2026-09-05 after the operator's "Proceed"; reports pending. Engrave
  origin/master = 1e61916 (pushed, no bypass). Pre-written: r1 fold-verification
  brief (cda41ac, FOLD_SHA pending).
- **H2 R0 ROUND 0 -- all five reports persisted** (fidelity 2b2bee5 0C/6I/7M/4N;
  tests e2b5735 1C/6I/2M/1N; journey 636c671 0C/5I/3M/2N; adversarial f9d3921
  2C/4I/7M/5N NOT GREEN; coverage d4d5861 0C/1I/3M/1N). Headline Criticals:
  the hardened derive flow never arms a wakeup (idleWakeup = 3 min blocks
  AppendEvents; the touch harness is blind to it; unlock_kdf.go:295-336 fixed
  the same thing once); the DecodeMS1Preimage / kind-row tests cannot fail on
  a wrong preimage or digest (corpus values unread). Recurring Importants
  across lenses: reconciliation line moved behind composerEveryPathHashed is
  unreachable for a mixed wallet; hashByPhrase never reset; Type-64-hex Back
  change untested though the plan claims a test; Deriving zero-state lead
  dead. Refute pass (sonnet) in flight over all five. NEXT: persist refute ->
  ONE opus fold agent (brief hashlock-H2-plan-R0-r0-fold-brief.md: plan AND
  gated tree, RED/GREEN, shards, checker) -> persist fold report -> fold commit
  -> r1 sonnet fold verification (brief cda41ac).
- **Refute pass persisted (7e698de):** 25 C/I findings -> 24 CONFIRMED, 1 PARTIAL
  (fit-gate renderer LABEL wrong, capacity delta measured 0 -- journey's
  "re-run the drop order" remedy refuted), 16 distinct defects, three severity
  disputes for the fold. **Round-0 FOLD DISPATCHED** (opus, brief 35310dd):
  plan + gated tree together, RED/GREEN per new test, shards, checker.
  Memory: `harness-ignores-the-wakeup-deadline`.
- **H2 PLAN R0 ROUND 0 FOLDED (`f60c2df`)**: 16/16 confirmed defects, both
  Criticals closed by EXECUTED tests (the derive flow keeps awake under the
  screensaver, proven on a Run-level synctest harness -- not deferred to the
  walk; decoder/digest tests compare the corpus's full 32 bytes and digest);
  21 mutations RED->GREEN; TrimSpace declined-as-redundant REFUTED by
  measurement (\v \f U+0085 U+00A0 U+2003 flip; matches the host's
  raw.trim()); 2 spec departures recorded as H3 items with replacement
  sentences (no spec edit). Controller re-run: checker 26 blocks / 0 FAIL;
  four packages ok; gui hashlock selection ok. Fold report 2234f04. **r1
  fold verification DISPATCHED** (sonnet, brief f43a8ad; executes both
  Criticals' RED itself). If GREEN: fill PLAN_SHA in the implementer brief ->
  ONE opus implementer on fork branch hashlock-h2.
- **H2 PLAN R0 GREEN (`1cb05b8`).** r1 fold verification (sonnet, 536fa79) GREEN:
  both Criticals' RED/GREEN reproduced in a fresh scratch, declined remedies
  verified by execution, whole suite + vet + gofmt + checker re-run (1220/1220,
  26/0). Two Nits folded (a citation off by one -- in a Go comment, so the
  gated tree's composer_hashlock.go got the same edit; the embedded checker
  line number). STATUS carries the lens list and names the one un-executed
  gate (the walk). **ONE opus implementer DISPATCHED** (brief
  hashlock-H2-implementer-brief.md at 1cb05b8): fork worktree
  .tmp/seedhammer-hashlock-h2 (branch hashlock-h2 off c4a64fc) + engrave
  records worktree me-worktrees/hashlock-h2. NEXT: implementation report ->
  controller gate re-run + emulator walk (fresh port, playwright) -> opus
  post-impl (brief hashlock-H2-post-impl-brief.md, fill tips) -> fold ->
  merge fork main --no-ff -> push -> pre-build signed image -> H4 ASSUMED at
  the operator's word; flash at their word only.
