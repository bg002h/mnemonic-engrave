# Brief — F-630 round 5: did r4 land, and does the end state close the class NOW?

Plan at `7b032747`: `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`.
Prior rounds, settled and NOT reopened: `f630-plan-r0-opus.md`,
`f630-plan-r0-fold-verify.md`, `f630-plan-r2-boundary-verify.md`,
`f630-plan-r3-implementability.md`, `f630-plan-r4-endstate.md`.

This is intended as the closing round. Two questions.

**A. Did the r4 fold land?** Verdict per finding (C-1, C-2, I-1, M-1, M-2, M-3,
N-1): FIXED / PARTIAL / NOT-FIXED, with the plan text that settles it. Note
that M-2 was closed by *deletion* — the structural scanner is cut — so check
that nothing still depends on it.

**B. Does the end state close the class now? Re-run r4's question B against the
new plan.** Build the end state in a throwaway worktree — T2's pins, D6, D5b's
card fix and routed loaders, T3's three edits with the real widened re-vendor,
and T1's gate now implementing D1 + **D1′** + **D1″** + D2a/D2b/D2c + D3 + D5g
including its membership assertion — then re-run r4's ten mutations plus any
you think of. Report per mutation: CAUGHT or SILENT, and by which clause.

D1′ is the new mechanism and the one to attack. It reduces each
`chains[c].descriptor` back to the record's `template`: replace every
`[fp/origin]xpub` with `@N/origin` where N is the slot whose 65 bytes match,
then compare against `template` with `<0;1>` resolved to `c`.

The controller machine-checked it before it was written into the plan: exact on
**46 of 46** records at `b2c5d693`, and catching suffix / quorum / script-type /
chain-swap / key-order while being silent on the header defect (which D1
catches) and the checksum (which D1″ catches). **Do not merely re-confirm
that.** Attack it:

1. **Can a descriptor be wrong in a way that still reduces to the template?**
   Construct one. Think about slots that appear more than once, a `@N` whose
   material matches two slots, an origin containing a `]` or a `/`, a key
   rendered with a `'`-vs-`h` hardening spelling, an xprv where an xpub belongs.
2. **Does D1′ hold for the pinned tier?** D5g exempts pinned records from D1's
   header rule. Does the plan say whether D1′ also relaxes there — and if it
   does not relax, do the three pinned records actually satisfy it?
3. **Is `template` itself trustworthy?** The plan argues `template` is bound to
   the card by the existing `wallet_descriptor_template_id` assertion, making
   D1′ cross-language rather than self-consistent. Verify that: mutate
   `template` alone and see whether anything reds.
4. **D1″'s export.** Exporting `validChecksum` from `bip380` — does that compile,
   and does the checksum clause actually fire on a bad checksum?

## Already established — do not re-derive

- The fork emits the 0.44.0-correct header; F-630 steps 3 and 4 are
  unnecessary/done.
- Drift: 2 identical / 41 descriptor-only (82 chain entries) / 3 semantic; 88
  across all 44; 44 of 46 red on the stale corpus, 5 of 46 after T2's pins.
- T3's literals: 246 files / 50 names, with the 14 added names in the plan.
- `go vet` exits 1 at baseline (ten ArtifactDir diagnostics); `gofmt -l .` is
  the five-file baseline; T1–T3 edit zero non-test Go files.

## Out of scope

Re-verifying earlier rounds. Style and wording. Proposing new features. Arguing
the scanner back in — that was a recommendation this round's author made and the
controller took.

## Rules of evidence

Execute; do not reason from prose. Assert that each mutation APPLIED before
reporting its verdict. Do NOT modify tracked files; use a throwaway worktree,
remove and prune it, verify both trees clean.

## Output

Severity per project standard. **FINAL ACTION: write the report to
`design/agent-reports/f630-plan-r5-close.md`** and return ONLY a one-paragraph
summary, that path, the A-verdict tally, your B answer (yes/no; if no, exactly
what slips through), and C/I/M/N counts.
