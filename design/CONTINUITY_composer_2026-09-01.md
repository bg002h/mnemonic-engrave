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
