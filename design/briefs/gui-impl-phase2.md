# Brief — mnemonic-gui phase 2: implement the GREEN design

The design `design/DESIGN_secret_channels_and_new_forms.md` in mnemonic-gui,
on branch `gui-followups` at `220c265`, is **GREEN (0C/0I)** after five review
rounds (R0–R5; reports in `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r*.md`).
**Implement it as written.** It's a frozen spec: if you find it wrong or
unimplementable at some point, stop at that point and report. Don't redesign
it quietly.

## Where

Worktree `/scratch/code/shibboleth/gui-worktrees/followups`, branch
`gui-followups` (clean at `220c265`; it also carries phase 1a's shipped fixes
`5956da2` and `e3a55a4`). Commit in logical steps on this branch.

## Order (the design's Q2: Part B's `ms hashlock` after Part A)

1. **Part A** — private channels, per the design:
   - the Rust planner mirrors `design/measurements/secret-channels/plan.py`
     (the reference); the cached data files the design names are loaded, and
     the policy decisions (the lookalike predicate, the CR/LF refusal, the
     per-OS `private_channels_on`, the reserved `MNEMONIC_GUI_` prefix) apply
     on every path and OS;
   - Preview, Copy command and the confirm dialog as specified;
   - the listed retirements (the help strings, kittest Cell 8, the f679 tests,
     the pass-through helpers, the phase-1a sentinel test).
2. **Tests the design requires** (§A9): T1–T10 and T3′, in Rust, running
   against the pinned release binaries. **The acceptance criterion in §A9 is
   binding:** add the Linux CI job running `secret_channels_t2`, `_t3prime` and
   `_t7` with the pinned binaries, and make T10 pass against the repo's own
   workflows, so "pending" becomes a hard failure. The regen check (T4) runs in
   CI too.
3. **Part B** — the five forms (md `compose`, `shape-key`, `descriptor`,
   `decompose`; then ms `hashlock` on Part A's `StdinToggle` channel), per the
   design's tables: hashlock phrase byte-for-byte, no trim; `--kind` with a
   "not specified" option.

## Test-first, mutation-checked

Every behaviour gets a test that fails first. Mutation-check the planner's
assignment, the lookalike predicate, the CR/LF refusal, and the runner's
delivery (swap two secrets; send the typed text instead of the resolved bytes).
A mutation must turn a test red, and the mutated line must have run.

## Pins

Stay on today's pins: mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0.
F-687 (`-`/`@env:` as channels in the CLIs) is merged but unreleased, and more
CLI work is in flight. The later pin bump is a data change the design already
handles; don't do it here.

## Gates

Full nextest against the release binaries (install them with
`sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`),
never `--release`; clippy `-D warnings` in both feature configs; MSRV 1.88.0;
the tutorial harness; form snapshots (explain every changed PNG); every
measurement script still green; CI on the branch, with the run ids recorded.
Don't tag or merge. Scratch under `/scratch/code/shibboleth/gui-impl2-scratch/`;
delete with `find … -delete`.

## Deliverable

Commits ending with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
**As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-impl-phase2.md`**:
what was implemented per design section, any point where you stopped or
deviated and why, tests and mutations, gates with CI run ids. Return a short
summary plus the path.
