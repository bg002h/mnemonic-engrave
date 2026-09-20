# Brief — verify the F-630 r5 fold (sonnet, mechanical, closing)

**One question: did the r5 fold fix its two Importants and five Minors without
introducing a defect?** Nothing else.

- Report (the input): `design/agent-reports/f630-plan-r5-close.md` (0C/2I/5M/1N)
- Plan before: `git show 7b032747:design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`
- Plan after: `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` at `d9499e7b`
- `git diff eecb1bf7..d9499e7b` is exactly the fold.

Repos: fork `/scratch/code/shibboleth/seedhammer` (`95716e97`), primary
`/scratch/code/shibboleth/descriptor-mnemonic` (`b2c5d693`).
Go: `export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH`.
Do NOT modify tracked files; delete any scratch file; verify both trees clean.

## Per finding: FIXED / PARTIAL / NOT-FIXED, with the plan text

I-1 (D1′'s false justification), I-2 (pinned tier omits D1′), M-1 (membership
assertion severity), M-2 (strict vs normalising), M-3 (xprv — check it is
recorded as non-gating residue and filed as a follow-up), M-4 (T4 firmware
rationale vs D1″'s export), M-5 (ambiguous slot lookup), N-1 (vendored copies
under pinned names).

## Verify these four citations the fold ADDED — a fold's new citations are
ungated by construction

- `crates/md-codec/src/test_vectors.rs:108` — is that a `Vector { … template: … }`
  literal?
- `crates/md-cli/src/cmd/vectors.rs:54` — does it derive the descriptor from
  `v.template`?
- `crates/md-cli/src/cmd/vectors.rs:142` — does it emit `template` into the record?
- That `rec.template` is asserted **nowhere** in the fork (the claim the fold
  rests on).

## Then: did the fold introduce a defect?

The fold's mechanism change is adding D1′/D1″ to the pinned tier. The r5 report
measured that variant (all 46 pass with D1′ applied everywhere, and the M19
regression is caught). Check the fold's text actually says that and does not
over-reach — e.g. does "only the header arm of D1 relaxes" contradict anything
else in the plan, such as D3's allowlist or D2c?

Also check the fold did not break internal consistency: any decision label
referenced but no longer defined, any count that now disagrees with another
part of the document, any task step referring to something cut.

## Out of scope

Re-litigating r5's verdict (B = YES). Earlier rounds. Style. New features.
Proposing the plan do more.

## Output

Severity per project standard; secret-handling is never C/I here.
**FINAL ACTION: write the report to
`design/agent-reports/f630-plan-r5-fold-verify.md`** and return ONLY a
one-paragraph summary, that path, the per-finding tally, and C/I/M/N counts for
any NEW defect.
