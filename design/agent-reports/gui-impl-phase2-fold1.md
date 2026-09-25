# mnemonic-gui phase 2, fold 1: answering the whole-diff review

**Outcome:** I fixed M1, M2, N1 and N2. M2 was small, so I fixed it here rather than filing it. N3 needs nothing here, since it is fixed upstream (ms F-683). `gui-followups` moves `ee26ef1` → `27f45f6` in 2 commits, pushed. CI is green on `27f45f6`: build `36181207304`, schema-mirror `36181207291`. Nothing was merged or tagged.

Review: `design/agent-reports/gui-impl-phase2-review.md` (0C/0I/2M/3N).

| sha | what |
|---|---|
| 864160c | fold 1: M1, M2, N1, N2 + `tests/fold1_review.rs` |
| 27f45f6 | `rust_mutations.out` re-run: 31/31 killed, control survived |

## Per finding

- **M1: hashlock Copy at "(choose)".**
  - Fix: `copy::copy_commands_with` now asks `conditional::run_blocker` first. If Run is blocked, both Copy buttons are `Disabled("Copy disabled — choose a --kind first …")`. This is the same gate, with the same wording, as Run.
  - Tests:
    - `m1_hashlock_copy_is_gated_like_run_at_choose` (pure): "(choose)" disables both buttons; `sha256` copies `--kind sha256`; "all kinds" copies.
    - `m1_window_copy_buttons_are_disabled_at_choose` (window): both Copy buttons and Run are disabled at "(choose)" and enabled with a kind.
  - Mutation: "M1: Copy no longer gated on (choose)" was **killed** by both tests.
- **M2: a pasted xprv in the B2–B4 md fields (secret handling, non-gating).**
  - New `copy::copy_reveals_secret` is true when the Copy plan carries a masked token. That can only be a content-masked non-source, i.e. these md fields; planner sources are never on the Copy argv.
  - When it is true:
    - both Copy buttons read "— reveals secret";
    - Run opens the confirm dialog (`needs_confirm` now includes a masked token in the plan);
    - the dialog uses the argv wording "passes secret-bearing arguments to" instead of "privately";
    - the key stays `••••` in the dialog.
  - Tests:
    - `m2_copy_reveals_a_pasted_private_key_and_says_so`: the xprv gives true, an xpub gives false, and a hashlock planner source gives false.
    - `m2_window_labels_copy_and_confirms_run_without_claiming_privacy`: covers the labels, Run stopping at the dialog, the wording, no xprv in the dialog, and the xpub case with no label.
  - Mutation: "M2: Copy reveal label never raised" was **killed**.
- **N1: nested labels.**
  - New `Binding::field_label()` takes the key from its first `--…` or `<…>` word on. Preview, the dialog and Copy comments now read `--share ← …` and `# --share: …`.
  - `source_label` keeps `plan.describe`'s spelling, so §A5 / T8 parity holds and T8 still passes.
  - Tested by `n1_binding_lines_drop_a_nested_subcommands_child_word`. Swapping `line()` back to `source_label` turns it red (hand mutation).
- **N2: mutation count.**
  - The denominator now counts the mutations actually run.
  - A filtered run prints "control not run (filtered)". Verified: a filtered run prints "2/2 … control not run (filtered)".
- **N3:** upstream (F-683); no change.

## Gates

All run against the pinned binaries, reinstalled with `install.sh`; sha256 == `measured_with.json`.

- nextest: **773 passed**, 6 skipped (768 + 5 new).
- clippy `-D warnings`, default and `--no-default-features`: clean.
- MSRV 1.88.0 `cargo check --locked`: clean.
- Tutorial harness: 12/12, with no transcript or PNG change. N1 does not touch J1/J2, whose keys are not nested.
- Form snapshots: 2/2, with no PNG change.
- `check_design_tables.py`: no stale blocks; `test_plan.py`: 0 failures.
- Full mutation harness: **31/31** killed by assertion with the mutated line run; the control survived.
- CI: build `36181207304` and schema-mirror `36181207291` succeeded on `27f45f6`. The `ci/gui-followups` ref was deleted afterwards.

Scratch `/scratch/code/shibboleth/gui-impl2-scratch/` was deleted with `find … -delete`.
