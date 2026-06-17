# CLAUDE.md — mnemonic-engrave repo notes

This file is auto-loaded by Claude Code when starting a session in this repository.

## What this is

`mnemonic-engrave` is the m-format constellation sibling that gets constellation strings (`md1`/`mk1`) engraved on a **SeedHammer II** machine. It provides the Rust CLI `me` (crate `crates/me-cli`): a converter (single `md1`/`mk1` → NDEF; refuses `ms1`), plus `me bundle` / `me bundle --preview` (the latter via the `me-preview` Go sidecar that reuses upstream SeedHammer curve math, pinned via the `third_party/seedhammer` submodule). SeedHammer firmware planning/design docs also live here, in `design/` — the fork itself (`bg002h/seedhammer`) is kept clean. Dual-licensed **MIT OR Unlicense**.

## Conventions

- **Default to ultracode (multi-agent workflow orchestration).** Standing user directive (2026-06-17), project-wide across the m-format constellation and the seedhammer fork — does NOT require the per-turn `ultracode` keyword. For any *substantial* task (research, design, implementation, review — anything beyond a conversational answer or a trivial mechanical edit), author and run a **Workflow** by default: decompose, fan out parallel subagents, **adversarially verify** findings before relying on them, and prefer exhaustiveness over token thrift; chain several workflows for multi-phase work. Token cost is not a constraint. Plain Q&A and trivial one-line edits still run solo. Composes with — does not replace — the R0 gate.
- **MANDATORY pre-implementation R0 gate — NO code before GREEN (0C/0I).** Every brainstorm spec and implementation plan-doc MUST pass an opus architect R0 review and converge to 0 Critical / 0 Important before any implementation begins. Fold findings → persist the review verbatim to `design/agent-reports/` → re-dispatch until GREEN. Re-dispatch after every fold (folds can introduce drift). Proceeding past any gate (start coding, advance phase, tag, ship) with an open Critical/Important is prohibited. (Project standard, shared with `mnemonic-toolkit`.)
- Design artifacts in `design/`: `RECON_*`, `SPEC_*`, `IMPLEMENTATION_PLAN_*`, `FOLLOWUPS.md`; per-phase opus reviews persist verbatim to `design/agent-reports/`.
- Per-phase TDD: tests before impl; reviewer-loop until 0C/0I after every fold.
- SeedHammer firmware work: planning docs live here; upstream PRs branch off `upstream/main`, commits signed + DCO, authored Brian Goss; keep PRs small and focused.
- Stage paths explicitly (no `git add -A`).
