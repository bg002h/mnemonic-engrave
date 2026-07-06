# AUDIT_FUNDS_SAFETY — mnemonic-engrave user-funds-safety audit

Started 2026-07-06. Controller: Claude (fable), finders/verifiers: opus. Scope per user
directive: **this repo only** (`/scratch/code/shibboleth/mnemonic-engrave` and subfolders);
the constellation codec crates one level up are audited separately; `third_party/seedhammer`
upstream internals are out of scope except this repo's *use* of its APIs.

## Goal

1. Find user-funds-safety bugs in the engraving code paths.
2. Produce a concrete automated-testing improvement plan that would reveal such bugs
   (seeds next cycle's SPEC → plan → R0 gate → implementation).

**Funds-safety bug classes:** (a) engraved output ≠ validated input (wrong/dropped/
reordered/truncated characters or shares, glyph-mapping errors, preview-vs-bundle
divergence); (b) validation bypass (checksum not enforced, ms1 admitted, errors
swallowed); (c) secret exposure (argv/logs/error text/temp files/permissions/leftover
previews); (d) silent partial failure (non-atomic writes, ignored errors, wrong exit codes).

## Resume protocol (for a fresh session)

Kickoff/resume command: **`/resume-audit`** (defined in `.claude/commands/resume-audit.md`),
or equivalently prompt: *"Resume the funds-safety audit: read design/AUDIT_FUNDS_SAFETY.md
and continue from the first incomplete step in the state tracker."*

Steps for the resuming controller:
1. Read this file's state tracker below.
2. A dimension is DONE iff its report file exists in `design/agent-reports/`. Re-dispatch
   any missing dimension (opus finder, prompt template recorded in the workflow script under
   the session dir — or reconstruct from the dimension descriptions below).
3. A finding is VERIFIED iff verdict files exist in `design/agent-reports/funds-audit-verify/`
   (named `<dim>-<findingId>-r<n>.md`). Dispatch opus refuters for unverified
   critical/important (2 votes) and moderate (1 vote) findings.
4. Synthesis: dedupe confirmed findings across dimensions, rank by severity, write
   `design/agent-reports/funds-audit-SYNTHESIS.md` + update the summary section here.
5. Draft `design/SPEC_me_testing_hardening.md` from confirmed findings + the D6 report;
   then the standard R0 gate before any test implementation (per CLAUDE.md).

## State tracker

| Step | Status | Artifact |
|---|---|---|
| D1 admission finder | DONE 2026-07-06 | `design/agent-reports/funds-audit-D1-admission-round0.md` |
| D2 NDEF/manifest finder | DONE 2026-07-06 | `design/agent-reports/funds-audit-D2-ndef-round0.md` |
| D3 bundle pipeline finder | DONE 2026-07-06 | `design/agent-reports/funds-audit-D3-bundle-round0.md` |
| D4 Go sidecar/boundary finder | DONE 2026-07-06 | `design/agent-reports/funds-audit-D4-sidecar-round0.md` |
| D5 secret hygiene finder | DONE 2026-07-06 | `design/agent-reports/funds-audit-D5-hygiene-round0.md` |
| D6 test adequacy finder | DONE 2026-07-06 | `design/agent-reports/funds-audit-D6-tests-round0.md` |
| Adversarial verification | DONE 2026-07-06 (20 refuter verdicts, all crit/imp/mod findings covered) | `design/agent-reports/funds-audit-verify/*.md` |
| Synthesis (controller, fable) | DONE 2026-07-06 | `design/agent-reports/funds-audit-SYNTHESIS.md` |
| Test-hardening SPEC draft | DONE 2026-07-06 | `design/SPEC_me_testing_hardening.md` |
| R0 round 0 | DONE 2026-07-06 — NOT GREEN (0C/2I/6L), ALL findings folded into SPEC; descoped items recorded in FOLLOWUPS.md | `design/agent-reports/me-testing-hardening-spec-R0-round0.md` |
| R0 round 1 | DONE 2026-07-06 — **GREEN (0C/0I/5 nit)**; both round-0 Importants verified closed against codec source; 5 nits folded inline (per GREEN+fold-nit precedent) | `design/agent-reports/me-testing-hardening-spec-R0-round1.md` |
| Implementation plan doc | DRAFTED 2026-07-06 | `design/IMPLEMENTATION_PLAN_me_testing_hardening.md` |
| Plan R0 round 0 | DONE 2026-07-06 — NOT GREEN (0C/1I/6L), ALL folded (I1 unreachable-arm test retargeted to direct ChunkHeader::read; guard located in validate.rs; CI triggers + branch-protection note; perturb-then-revert rule for drift guards; STOP condition extended to admission regressions; scratch-crate fixture path) | `design/agent-reports/me-testing-hardening-plan-R0-round0.md` |
| Plan R0 round 1 | DONE 2026-07-06 — **GREEN (0C/0I/3 nit)**, nits folded inline; every fold verified against codec source | `design/agent-reports/me-testing-hardening-plan-R0-round1.md` |
| Implementation (fixes + tests) | DONE 2026-07-06 — all plan steps 0–11 green, NO STOP conditions; 82 Rust + 16 Go tests, 0 skips with ME_REQUIRE_GO=1; hermetic clean-clone + end-to-end verified; 12 commits on branch `me-testing-hardening` in worktree `/scratch/code/shibboleth/mnemonic-engrave-testing-hardening` (NOT merged/pushed). Go toolchain: `/home/bcg/.local/go/bin` (not on default PATH) | worktree + `design/agent-reports/me-testing-hardening-impl-log.md` (on branch) |
| Post-impl adversarial exec review (mandatory) | round 0 DONE 2026-07-06 — **GREEN (0C/0I/2L/1N)**; reviewer re-performed 5 perturbations (no test theater), regenerated fixtures byte-identical, mk-codec 0.4.0→0.4.1 diff = zero normative change | `design/agent-reports/me-testing-hardening-exec-review-round0.md` |
| Fold of exec-review L1/L2/N1 | DONE 2026-07-06 — controller inline fold, TDD (red→green), commit `7565f62` on the branch: mk-codec `InvalidHrp` pass-through redacted at ValidateError Display + B8 canary variant; `.gitignore` artifact. Full suite re-verified green (82 Rust, 0 skips; Go ok) | worktree commit `7565f62` + impl-log fold section |
| Exec review round 1 (scoped to fold `7565f62`) | DONE 2026-07-06 — **GREEN (0C/0I, zero new findings)**; L1/L2/N1 closed with perturbation proof; InvalidHrp confirmed the ONLY input-carrying mk-codec variant; no other branch drift | `design/agent-reports/me-testing-hardening-exec-review-round1.md` |
| Branch protection | DONE 2026-07-06 (user-approved) — master requires status check `test (rust + go)` (exact check-run name, verified resolved to the Actions app after the maiden run; initially set to job-id `test`, corrected). strict=false, enforce_admins=false: admin direct pushes allowed, paired with the local-merge workflow; PRs and non-admin pushes are gated | GitHub repo setting via API |
| Push + maiden CI run | DONE 2026-07-06 — `master → e4e6e27` pushed; run 28825720713 ALL GREEN: `test (rust + go)` passed first try (submodules + Go setup clean), 6/6 build-matrix jobs green, `assemble` correctly skipped (non-tag push) | GitHub Actions run 28825720713 |
| Merge | DONE 2026-07-06 (user decision) — fast-forward `master` → `7565f62` (13 commits); full suite re-verified green on merged master (82 Rust, 0 skips, ME_REQUIRE_GO=1; Go ok). NOT pushed — user pushes when ready (first push exercises the new CI `test` job) | `git log fdc11aa..7565f62` |

**CYCLE COMPLETE 2026-07-06.** Descoped follow-ups live in `design/FOLLOWUPS.md`
(`me-*` entries F8–F18 subset). Scratch fixture generators kept at
`/scratch/code/shibboleth/me-impl-scratch/` for reproducibility.
| Post-impl adversarial exec review | PENDING (mandatory, after implementation) | `design/agent-reports/` |

## Dimensions

- **D1 admission** — `crates/me-cli/src/{classify.rs,validate.rs,lib.rs,main.rs}`:
  checksum enforcement on every path, ms1 refusal robustness, malformed-input rejection,
  case/whitespace normalization altering payloads, md-codec/mk-codec API misuse.
- **D2 NDEF/manifest** — `crates/me-cli/src/{ndef.rs,manifest.rs}`,
  `firmware/ndef-roundtrip/`: NDEF record construction vs NFC Forum spec (TNF/SR/length
  boundaries), manifest ↔ payload integrity, roundtrip tool correctness.
- **D3 bundle pipeline** — `crates/me-cli/src/{bundle.rs,preview.rs}`: share
  ordering/sequencing, overwrite/atomicity, single-parse consistency (bundle engraves the
  same bytes that were validated), sidecar invocation and error propagation.
- **D4 Go sidecar/boundary** — `preview/*.go`: parameter fidelity vs upstream
  `third_party/seedhammer` v1.4.2 (the "replicated VERBATIM from platform_sh2.go" claim),
  glyph coverage of the bech32 charset in font/sh, QR generation, Rust↔Go protocol framing,
  version handshake, the ndef-roundtrip `replace` pointing outside the repo.
- **D5 secret hygiene** — whole first-party tree: argv/env/log/error-text exposure,
  zeroize correctness (reallocation copies), output file permissions, leftover preview
  PNG/SVG containing the payload, panic messages.
- **D6 test adequacy** — all tests (`crates/me-cli/tests/*`, `preview/*_test.go`, CI):
  golden-vector coverage (currently ONE `.ndef` golden), differential Rust↔Go testing,
  property/fuzz gaps, mutation sensitivity ("would the suite catch a one-word swap?").
  Output = prioritized concrete test proposals.

## Findings summary (2026-07-06 — full detail in funds-audit-SYNTHESIS.md)

**No confirmed CRITICAL** — no path found where `me` engraves bytes ≠ validated input,
drops/reorders shares, or admits a checksum-invalid string.

Confirmed **IMPORTANT** (2 high-confidence refuter votes each):
- **F1** `me bundle` echoes full input to stderr → mangled-HRP ms1 secret leaked verbatim
  (`bundle.rs:54-60`; convert path is hardened, bundle path regressed).
- **F2** No CI runs any tests — release.yml is build-only; red suite can be tag-released.
- **F3** Cross-language differential tests silently PASS when `go` is absent.

Confirmed **MODERATE**: **F4** md1 separators/newlines validated-then-emitted uncovered
by checksum (lib.rs:57-63); **F5** stale md-codec 0.36 admits >93-symbol md1 that 0.40's
fail-closed guard rejects (bump needed); **F6** NDEF TLV 255-boundary untested (mutation
→ device misparse).

Confirmed **LOW** F7–F18 and four refuted findings: see SYNTHESIS. Next session: R0 gate
on `SPEC_me_testing_hardening.md`, then single-implementer TDD cycle (fixes F1–F7 + test
hardening), then mandatory post-impl adversarial execution review.
