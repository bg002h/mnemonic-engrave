# F-694 part 1, round 2 — the controller's rulings 1–4 and v0.63.0 prep (INCOMPLETE; handed back mid-run)

Agent: Opus 5.5, 2026-09-25. Worktree `/scratch/code/shibboleth/gui-worktrees/repin063`, branch `repin-063`.
The harness forced this handback while the Rust mutation harness was still running, so this
report records the state at handback. It is not a finished round.

## Done and committed (`bf88ab4`, tests only; no planner or production code change)

1. **Ruling 1, source order.** `shapes.py` now lists every shape's sources in the form's
   order. T6 `t6_every_shape…` asserts that order, not the sorted set, and lists every
   drift at once. Four shapes had drifted:
   - bundle slot+passphrase;
   - bundle wsh-multi 2 slots+passphrase;
   - silent-payment secret+passphrase;
   - verify-bundle ms1+slot+passphrase.

   I regenerated `plans_pure.json` (T8), `plans.json`, `a5.md`, `t3.md` and
   `run_plans.out` (`FAILURES: none`), and the DESIGN §A5 and §A9 blocks.
   `check_design_tables.py` is GREEN.
2. **Ruling 2, the surviving mutation.** `test_plan.py` now pins the leading-dash refusal on
   `ms hashlock --hashlock-phrase` (macOS and Windows). It also checks that this input is
   still measured not argv-exact, so the leg goes red loudly if a future re-pin makes it
   exact, rather than going vacuous. `mutations.py`: **20/20 killed, control survived**, and
   Nm13 is killed by a test_plan assertion.
3. **Ruling 3, the two t6 Copy tests.** Both now derive their expectation, which was small:
   - `t6_copy_env_provenance…` branches on the measured `cli_env_rule` of
     `mnemonic restore --passphrase`. A verbatim or absent rule expects the `printf` pipe;
     otherwise it expects the CLI's own `--passphrase @env:MY_PW`, with no printf.
   - The shell leg reads which recipe Copy printed and runs that one: the `read -rs
     MNEMONIC_GUI_S1` recipe for a clean value, the stdin row for an edge-whitespace value.
     A non-vacuity assert requires that the read recipe ran at least once.
   - The second leg's label changed from "printf recipe" to "$VAR recipe".
   - Why: on 0.105.1 the passphrase EnvRef cell has terminator `null`, because strip-one
     makes no terminator exact. The planner therefore treats it as lenient and keeps
     non-clean values off it (§A3c).
   - Observation, not acted on: this is conservative but coarser than necessary. The CR/LF
     refusal already makes strip-one harmless, so `run_bytes.py` could compute the
     terminator after applying `cli_env_rule`. That would be a measurement refinement for
     later.
   - T1/T3'/T6/T7/T8: 31/31.

## In the working tree, NOT committed

- DESIGN §A6 step 4 prose now says: exact on 40 inputs; only `ms hashlock
  --hashlock-phrase` is not; `ms derive --passphrase` was not exact on 0.19.1.
  `check_design_tables.py` is GREEN with this edit.
- CHANGELOG: `## mnemonic-gui [0.63.0] — 2026-09-25`. It covers the secret channels (Part
  A), the five forms (Part B), the existing Unreleased bullets (restore masking, F-685),
  the re-pin effects of F-687, the CLI prompt/drain/empty-warning behaviour versus the GUI,
  the `--separator` mirror and the tutorial label. `## Unreleased` is left empty above it.
- README: Latest release, install tag, and a v0.63.0 "Recent releases" bullet.
- `src/form/channels/mod.rs` shows as modified **only because the Rust mutation harness
  (`scripts/secret-channels-mutations.py`, PID 1400190) was mid-mutation at handback**. The
  harness restores each file by sha256 when it finishes. **Do not commit that file.** Check
  `git diff src/` is empty after the harness exits.

## NOT done

- Rust mutation harness result. The log `/scratch/code/shibboleth/gui-repin-scratch/logs/rustmut.log`
  was still empty, because output comes at the end.
- `Cargo.toml` `version = "0.63.0"` and the `Cargo.lock` update. I held these back so as not
  to disturb the harness.
- Full gates after these changes: nextest full suite, clippy (both configs), MSRV, tutorial
  harness, form snapshots, T10, and CI on the branch.
- Commit and push of the CHANGELOG/README/DESIGN edits.
- Deleting `/scratch/code/shibboleth/gui-repin-scratch/`. The running harness uses its
  target dir.

Resume: wait for the harness to exit and confirm `git diff src/` is empty. Then set the
version and run `cargo update -p mnemonic-gui --offline`. Run the full gates, commit, push
to `repin-063` and `ci/repin-063`, and watch CI. Finally delete the scratch dir with
`find … -delete`.
