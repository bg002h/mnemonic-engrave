# F-694 part 1, round 3 — mnemonic-gui v0.63.0 prepared, all gates and CI green

Agent: Opus 5.5, 2026-09-25. This round finishes the resume steps from `f694-gui-repin2.md`.
Branch `repin-063` in worktree `/scratch/code/shibboleth/gui-worktrees/repin063`, tip
`1d3e49692fffea776fa431e19f7786d562a42b4f`. Not tagged, not merged.

## Outcome

v0.63.0 is prepared: every local gate and every CI job is green. The re-pin needed no
planner or production code change. The only code changes are the test-side ones from
rulings 1–3.

## Commits on `repin-063` (off origin/master `3ca1d60`)

| sha | what |
|---|---|
| `45e6153` | pins: toolkit v0.104.0 -> v0.105.1, ms v0.19.1 -> v0.20.1 (+ `--separator` mirror comments) |
| `245a43d` | measurement caches re-derived; DESIGN generated blocks; CI installer SHA -> toolkit `1402d547` |
| `f209244` | 50 tutorial PNGs re-baselined; the only change is the `Pinned: mnemonic 0.105.1` label |
| `bf88ab4` | rulings 1–3 (test side): shapes.py in form order, the Nm13 leg re-pointed at `ms hashlock --hashlock-phrase`, the t6 Copy tests derive their expectation |
| `fbc1f91` | `rust_mutations.out` re-run; §A6 step 4 prose now reflects ms 0.20.1 `argv_eq_exact` |
| `1d3e496` | release prep: Cargo.toml 0.63.0 (+ lock), CHANGELOG `[0.63.0]`, README |

## Ruling 4 — Rust mutation harness

`scripts/secret-channels-mutations.py`: **31/31 killed by assertion with the mutated line
run; control survived; problems: none.** In round 1 the three red t6 tests killed the
control. `src/form/channels/mod.rs` was restored after the run; `git status` was clean
before any commit.

## Gates, at `1d3e496`, against the pinned release binaries

| gate | result |
|---|---|
| `cargo nextest run --locked --workspace` (no `--release`) | 773/773 passed, 6 skipped |
| clippy `--all-targets -D warnings` / `--no-default-features -D warnings` | clean / clean |
| MSRV 1.88.0 `cargo check --locked` | clean |
| tutorial harness `GUI_TUTORIAL_SNAPSHOTS=1 … --include-ignored` | 12/12 |
| form snapshots, T10 (in the suite); `os_gate.py` | pass; exit 0 |
| `regen_check.py --plans` | GREEN |
| `check_design_tables.py` | GREEN (test_plan.py 0 failures) |
| `mutations.py` (Python T5) | 20/20, control survived |
| Rust mutation harness | 31/31, control survived |
| CI, `ci/repin-063` @ `1d3e496` | schema-mirror run **36207415543: success**; build run **36207415492: success** on every job (clippy, headless, msrv, snapshots, tutorial-snapshots, 8 targets); release skipped (tag-gated) |

## CHANGELOG [0.63.0] covers

- Secret channels (Part A) and the five new forms (Part B): both are unreleased since
  0.62.0.
- The existing Unreleased bullets: restore `--from` masking and F-685.
- The measured F-687 effects of the re-pin:
  - the 17 former WRONG cells are now OK;
  - strip-one on 16 inputs;
  - Copy spelling `@env:VAR`;
  - edge-whitespace passphrases now go over stdin;
  - `ms derive --passphrase=` is exact.
- Why the CLIs' terminal prompt, paste drain and empty-value warning do not affect the GUI.
- `--separator` is now covered by the choices gate.
- The tutorial label.

## Cleanup

- `/scratch/code/shibboleth/gui-repin-scratch/` deleted with `find … -delete`.
- Remote `ci/repin-063` deleted.
- Kept for the merge: remote `repin-063` and the worktree.

## Open, not acted on

- The EnvRef cells of the 16 strip-one inputs are measured with terminator `null`, so the
  planner treats them as lenient and keeps edge-whitespace values off them. This is safe
  but coarser than needed, because the CR/LF refusal already makes strip-one harmless.
  `run_bytes.py` could derive the terminator after applying `cli_env_rule`. That is a
  possible follow-up, not a defect.
- The `c1_evidence.sh` line labelled "= the literal passphrase '-' on argv" is out of date:
  argv `-` is now stdin. The data is correct.
