# Brief — F-630 round 4: did r3 land, does the end state close the original defect, and is it proportionate?

Plan at `41671c9b`: `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`.
Prior rounds, all settled and NOT reopened: `f630-plan-r0-opus.md` (1C/7I/4M/2N),
`f630-plan-r0-fold-verify.md` (1C), `f630-plan-r2-boundary-verify.md` (0C/3I),
`f630-plan-r3-implementability.md` (2C/5I/5M/2N).

## Three questions

**A. Did the r3 fold land?** Verdict per finding (C-1, C-2, I-1…I-5, M-1…M-5,
N-1, N-2): FIXED / PARTIAL / NOT-FIXED, with the plan text that settles it.
Pay most attention to **D5g**, which is a new *design* decision (two tiers:
full D1 for vendored records, a pinned-legacy shape for forkbuilt ones) that no
round has reviewed. Is it sound? Can a real defect hide inside "pinned-legacy"?

**B. Does the end state close the ORIGINAL defect? — the question no round has
asked.** F-630 exists because a re-vendor imported a behavioural change and
left `go test ./md/` green, since the conformance test parsed
`.chains[].descriptor` into a field it never asserted. After T1–T4 as written:
simulate a *future* primary change of the same shape — pick a plausible one
(e.g. the primary changes a rendered origin, a derivation suffix, a key order,
or re-points an account), apply it to the vendored corpus, re-run the suite,
and report whether anything goes red. **If a silent import is still possible,
name it — that is the plan failing at its own purpose.** Do this by executing,
in a throwaway worktree, not by reasoning.

**C. Is the plan proportionate?** It began as "re-vendor a corpus and assert a
field" and now carries a fixtures boundary, a two-tier gate, and a structural
scanner with per-directory known-bads. Three real split sites were measured, so
the boundary is not invented — but say plainly whether any part is scaffolding
that buys no defect-detection, and which single part you would cut if one had
to go. A recommendation, not a survey.

## Already established — do not re-derive

- The fork already emits the 0.44.0-correct header; F-630's steps 3 and 4 are
  unnecessary/done.
- Drift: 46 keyed records → 2 identical, 41 descriptor-only (82 chain entries),
  3 semantic (F-529's); 88 across all 44; the gate reds 44 of 46 on the stale
  corpus and 0 of 46 on `b2c5d693`'s.
- T3's literals are 246 files / 50 names, and the 14 added names are listed in
  the plan.
- The three split card/record sites, the 18/2/16 occurrence counts, and the
  284/284 fingerprint agreement.
- D5g's premise: `b2c5d693` ships no post-0.44.0 record of the three
  reuse-bearing policies, because it replaced them.

## Out of scope

Re-verifying earlier rounds. Style and wording. Reviewing an implementation —
none exists. Proposing new features.

## Rules of evidence

Execute; do not reason from prose. Do NOT modify tracked files; use a throwaway
worktree, remove it, and verify both trees clean. Every finding names the state
and the wrong outcome.

## Output

Severity per project standard. **FINAL ACTION: write the report to
`design/agent-reports/f630-plan-r4-endstate.md`** and return ONLY a
one-paragraph summary, that path, the A-verdict tally, your answer to B
(yes/no + what slips through if no), your one-line answer to C, and C/I/M/N
counts.
