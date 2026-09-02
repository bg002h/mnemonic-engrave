# F-453 plan-author report — Composer Stage 0b (`md compose --preset` + six archetype vectors)

**Plan:** `design/IMPLEMENTATION_PLAN_composer_S0b_presets.md` (new, 1562 lines, 3 tasks). Status: DRAFT, R0 not yet run — this report is the authoring record, not a review.

## Grammar chosen, and why

`--preset <name>[,<k>of<n>]*[,<param>=<value>]*`, mutually exclusive with `--path` via a `clap::ArgGroup` (`required(true)`, default `multiple(false)`), still under one required `--wrapper`. Names are the six archetypes in kebab-case (`plain-multisig`, `simple-timelocked-inheritance`, `kofn-recovery`, `tiered-recovery`, `hashlock-gated`, `decaying-multisig`) — kebab-case because every other multi-word CLI value in this surface already is (`sh-wsh`). `<k>of<n>` tokens are POSITIONAL (tier 1 before tier 2, the only unambiguous way to fill two key-set parameters); every other parameter is a named `key=value` token, matched by name so order doesn't matter and a duplicate is refused. `older`/`older1`/`older2`/`after` are always plain numbers (never `--path`'s `Nu`/`Ht` unit suffixes) because every `presets::*` constructor's own lock argument is a bare `u32` blocks/height count — there is no unit ambiguity to disambiguate. `unsorted` is never a preset parameter: `presets::ks` hardcodes `sorted: true`, so there's nothing to toggle. No CLI-side special case was written for §4d's "under sh/sh(wsh) only plain-multisig" rule — every non-plain constructor already fails `validate`'s `ComposeError::LegacyWrapperShape` there, so the existing propagation path covers it for free.

## Default parameters, and their source

§4d fixes no numeric defaults, so this plan sets them: `2-of-3` and `older=26280` everywhere free — the journey's own canonical values, already used repeatedly in `family()` and in S0's own CLI test. Shrunk from S0 Task 7's illustrative examples where those would exceed the four-journey-xpub budget every other `keyed_compose_*` vector obeys (`tiered_recovery`: `2of2,1of2` not `2of2,2of3`; `decaying_multisig`: `2of2,1of1`, `older1=13140`/`older2=26280`/`after=1_000_000`, reusing an existing family() `AfterHeight` value, not `2of3,1of2`). All six chosen parameter sets were verified end to end against the REAL shipped `md compose`/`encode`/`decode`/`address` binary (66bdf2f4) before being written into the plan — not hand-derived from the lowering table.

## Singular-tag decision

`preset:<name>` tags (six of them) are added to `SINGULAR_TAGS`, exempting them from the two-vector rule. Justification: F-453's explicit deliverable is ONE MANIFEST vector per archetype, so each tag has exactly one legal vector by construction — the same structural reason `spine:0` is already singular. They are NOT added to `compose_vectors.rs`'s hardcoded required-tag list, because §12 item 1's own required-axis list (read at HEAD) does not name presets as a required coverage axis.

## Plan size and verification depth

3 tasks (vs. S0's 9 — the constructors, types and lowering already ship; this plan is the CLI surface and the corpus). Every Rust block was written into a real scratch copy of descriptor-mnemonic and built: `cargo build`/`clippy`/`nextest` all green on both crates (md-codec 52/52 compose tests with the six new family rows; md-cli 23/23 on the new+existing compose CLI tests, 775/775 on the full crate after corpus regeneration), `cargo fmt --all -- --check` clean. The exporter was run for real (30 new files, matching 6 vectors × 5 file kinds). `scripts/plan-build-gate-md.sh` was run against the finished plan file twice (reproducibly): steps 1-5 pass exactly as documented; step 6 halts on a real compile error (arity mismatch) until `main.rs`'s fragment is hand-wired, which is now stated precisely in the plan rather than glossed over — this was a genuine finding from actually running the gate, not something inferred from reading its source, and the plan's own first draft had understated it before this was caught and corrected.

## Left open for the operator (sparing)

1. **R0 has not run.** This plan must clear the architect R0 loop (0C/0I) before implementation, per CLAUDE.md.
2. **The fork's two hand-edits** (`compose_vectors_pin_test.go`'s hardcoded 26-name list and `126` count) are named as Stage-3-or-later follow-on work in Task 3, not built here — confirm that ownership assignment is acceptable, or file it as its own follow-up before this plan ships.
