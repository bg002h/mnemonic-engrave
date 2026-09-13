You are the INDEPENDENT adversarial execution reviewer (opus tier) for two
follow-on fixes from the composer journey walk in the SeedHammer fork.

Repo `/scratch/code/shibboleth/seedhammer`, branch `composer-review-names-script`,
tip `1dab84a`. YOUR RANGE IS `git diff 6728c22..1dab84a` — exactly two commits:

1. `c4d8527` the Review names the script, in the picker's words (journey I-7)
2. `1dab84a` the changed-id banner compares the IDS (journey I-5)

`6728c22` (the script picker's preselect, journey C-1) is under review by a
DIFFERENT agent and is OUT OF YOUR SCOPE. Treat it as given. The findings are in
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-policy-journey.md`
as I-7 and I-5.

ONE QUESTION: can you construct a policy or a sequence for which either change
puts a FALSE statement on a screen the operator copies onto steel — a wrong
script name, or a changed-id banner that is wrong in either direction?

Read-only on the fork; commit nothing; no sub-agents; never read any `.jsonl`.
Work in your OWN scratch copy:
`rm -rf /scratch/code/shibboleth/.tmp/followons-review && git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/followons-review 1dab84a`.
Go is `/scratch/code/shibboleth/.toolchain/go/bin/go`, first on PATH;
`TMPDIR=/scratch/code/shibboleth/.tmp`. Revert every mutation; remove the
worktree when done.

## Already settled — do not re-derive
- gui is 1296/1296 across 24 shards, partition verified exhaustive.
- Firmware: 1,645,292 -> 1,645,980 flash across the two commits, RAM unchanged.
- Five mutations already run and RED: the Review line removed (4 sub-tests);
  sh-wsh collapsed into sh (Nested alone); the id predicate constant-false,
  constant-true, and unreadable-id-swallowed.
- `composerTemplateChunksFor` is deterministic over one unchanged state
  (measured, byte-identical). The LEG that produced differing chunks for an
  unchanged shape is still unidentified and is deliberately left open in F-520 —
  do not treat its absence as a defect in the fix, but DO say if you find it.
- Secret-handling never gates.

## Construct counterexamples — an assessment without one is Minor at most
1. **The script line's truth.** `composerScriptLine` maps a DECODED
   `md.Template` to a label. Find a template it names wrongly. Specifically:
   `sh(wsh(...))` vs bare `sh(...)` differ only by `InnerWsh` and hash to
   different addresses — is `InnerWsh` populated on every path that reaches the
   Review, including a consent rendered from chunks alone? What does a
   single-sig root print, and is the fallback honest rather than confidently
   wrong? Is there a wrapper the composer CAN build that lands in the default
   arm?
2. **Is it the card's script or the composition's?** The fix reads the decoded
   template deliberately. Verify the Review really is rendered from the chunks
   that will be engraved on every path that reaches it, and that no caller
   passes chunks from a different composition than the one on screen.
3. **The shared labels.** `composerWrapperLabels` and `composerWrapperOrder` are
   now shared between the picker and the Review, index-aligned. Prove the
   alignment holds, and that nothing else indexes into either by a stale
   assumption. What happens if a fifth wrapper is appended to one and not the
   other — is that a compile error or a silent mislabel?
4. **The id predicate, both directions.** `composerIdChanged` must not be silent
   when the id really moved, and must not fire when it did not. Construct: two
   shapes whose ids collide (say so if infeasible and why); a keyed vs keyless
   chunk set for the same policy; an empty `current`; a `shown` that is
   non-empty but unreadable. Check the call site in `composer_flow.go` too — is
   `shown` assigned on every leg that reaches the stub screen, and can it hold a
   set from a DIFFERENT shape than the one the operator last saw?
5. **The tests.** For each test the two commits add, mutate the guarantee it
   names (not its navigation) and quote the failure. A survivor on a normative
   guarantee is Critical.

## Severity
Critical: a false statement on the Review or the stub screen; a banner silent
when the id really changed; a test that cannot fail on what it names. Important:
a mislabelled wrapper in a reachable case, an unsound assumption, a missing
case. Minor/Nit: wording, records.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-journey-followons-review.md`
(create; must not exist): findings as `### C-n / I-n / M-n / N-n — title` with
the command, verbatim output and the line violated; a table of every wrapper and
root you exercised with the label it produced; the mutation table; closing counts
and GREEN / NOT GREEN. Return a two-line summary plus the path.
