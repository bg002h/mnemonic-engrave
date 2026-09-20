# Brief — verification of the F-630 plan fold (sonnet, mechanical)

**Two questions, nothing else:**

1. **Did the fold fix each finding?** For every finding in
   `design/agent-reports/f630-plan-r0-opus.md` (1C/7I/4M/2N), say
   FIXED / PARTIAL / NOT-FIXED / DECLINED and quote the plan text that
   settles it.
2. **Did the fold introduce a new defect?** Review-response edits are the
   text nobody has read yet. Check the NEW material specifically.

**Artifacts.**
- Report (the input): `design/agent-reports/f630-plan-r0-opus.md`
- Plan before the fold: `git show 5cc0a0a2:design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`
- Plan after the fold: `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` (at `4b7894c9`)
- `git diff 03f368d1..4b7894c9` is exactly the fold and nothing else.

Repos: fork `/scratch/code/shibboleth/seedhammer` main `95716e97`; primary
`/scratch/code/shibboleth/descriptor-mnemonic` main `b2c5d693`.
Go: `export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH`. Scope any
Go run with `-run`; a full `./gui/` run is ~493s.

## ALREADY MACHINE-CHECKED by the controller — do not re-derive

Every one of these was re-grepped or re-computed at `95716e97` after the fold.
If you think one is wrong, say so in one line with the command, and move on.

- Citations resolve: `gui/policy_address_test.go:125,174,77,143-147`;
  `md/f533_pinned_vectors_test.go:82-86`;
  `md/compose_vectors_pin_test.go:79-84,103,110-111,134`;
  `md/duplicate_keys_test.go:16,84`; `md/compose_shape_test.go:111`;
  `md/expand.go:56-64,76-81`; `scripts/vendor-compose-vectors.sh:16,18,20-22`;
  `gui/md1_expand.go:143`; `bip380/bip380.go:97-109`.
- 82 descriptor entries move across the 41 descriptor-only records; 88 across
  all 44 drifted. The gate reds 44 of 46. 284/284 bracket fingerprints agree
  with `ExpandedKey.Fingerprint`. `composeVectorNames` holds 36 names and the
  pin 176 files; the widened pattern would give 51/247.
- The fork already emits the 0.44.0-correct header; step 3 of F-630 is
  unnecessary and step 4 is already done. Not open for re-litigation.

## What to look at hardest

The fold's new material is D2c, D3's two-sided allowlist, D5 (the pin carries
the record), D6 (pin a divergence, not a deletion), the merged T3, and the new
dependent-sites table. Specifically:

- **D5's remedy.** A pin now carries `<name>.conformance.json` beside
  `<name>.md1.txt`, and `loadVectorRecord` prefers it. Does that actually make
  record and card one policy at every site that pairs them, or is there a site
  that still pairs a pinned card with a vendored record? Check
  `gui/policy_address_test.go` and anything else globbing
  `keyed_*.conformance.json`.
- **D6's replacement check.** It asserts the vendored file's
  `wallet_descriptor_template_id` equals a recorded value (`8c1c0566`,
  `09903620`, `71ff3b74`). Verify those three are what `b2c5d693` actually
  ships. Does the check still detect a *third* policy arriving under these
  names?
- **D3 two-sided.** Does pinning the record's expected bracket actually close
  the account-`9h` mutation the report constructed, or does the allowlist's
  wording still leave the record half loose?
- **T1's acceptance number** is now 44. Confirm against the report's measured
  `STALE corpus: 44 vectors FAIL, 2 PASS`.
- **The merged T3.** Does it now name everything that must change together
  (script pattern, the two literals, the `compose_refusal_` exclusion), and is
  its "capture the pre-state first" instruction actually sufficient to check
  the diff expectation after the script has overwritten the corpus?

## Out of scope

Style, wording, doc polish. Proposing the plan do MORE. Re-auditing the fork.
Re-measuring anything in the machine-checked list. Do not review the
implementation — none exists yet.

## Rules of evidence

A new-defect finding must name a concrete failure the plan-as-written now
admits: the state, and the wrong outcome. "Could be clearer" is a Nit.

## Output

Severity per project standard: Critical / Important / Minor / Nit.
Secret-handling defects are never Critical or Important here.

**FINAL ACTION: write the full report to
`design/agent-reports/f630-plan-r0-fold-verify.md`** (in
`/scratch/code/shibboleth/mnemonic-engrave`) and return ONLY a one-paragraph
summary, that path, the per-finding verdict tally, and your C/I/M/N counts for
any NEW defects. Do not return the report inline.
