# F-630 whole-diff fold verification (sonnet, mechanical, pre-push)

**Verifier:** independent agent; did not write the fold or the review it
responds to.
**Question answered:** did fold commit `2cbca02` close the whole-diff review's
Important and two structural Minors without introducing a defect, and is the
build gate real.

Worktree: `/scratch/code/shibboleth/sh-worktrees/f630`, branch
`f630-xpub-header-sync`, verified fold commit `2cbca026e32f4fc39402b46292c4f06fbe271cad`.
All mutations below were applied directly to the worktree and restored by hand
(diff, not `git checkout`) after each one; `git status` and `git diff HEAD`
are empty at the end of every step, confirmed after every restore and once
more at the end of the session.

**Result: fold closes I-1, M-1 and M-2 as claimed. One NEW defect found in the
fold's own code: the "tier arithmetic" assertion is algebraically identical to
the assertion immediately above it and can never fire independently — it is
not a literal tautology (it does depend on data and can report an error), but
it contributes zero net verification. Severity: Minor (no coverage or
guarantee is lost; the invariant it restates is still correctly enforced by
the adjacent clause). 0 Critical / 0 Important / 1 Minor (new) / 0 Nit (new).**

---

## 1. The Important (I-1) — union fix, reproduced and stress-tested

Reran the review's exact reproduction against the fold: deleted
`md/testdata/vectors/keyed_tr_multi_a.*`, ran
`TestKeyedConformanceDescriptorsAgreeWithTheirTemplates -v`.

```
descriptor gate: 46 of 46 vectors pass (43 correct-header, 3 pinned-legacy), 0 fail
```

Coverage preserved — the pinned record is still examined via the `forkbuilt/`
half of the union. Restored; `git diff` empty.

Also machine-checked the commit message's own claimed second mutation (revert
`eachKeyedVector` to vendored-only glob while keeping the new assertions, same
deletion). Patched a scratch copy of `md/vector_fixtures_test.go` back to the
pre-fold glob, reran:

```
conformance_keyed_test.go:293: 2 pinned record(s) reached the legacy arm, want 3
    ([keyed_tr_multi_a keyed_tr_sortedmulti_a keyed_wsh_timelock_hashlock])
descriptor gate: 45 of 45 vectors pass (43 correct-header, 2 pinned-legacy), 0 fail
--- FAIL: TestKeyedConformanceDescriptorsAgreeWithTheirTemplates
```

Exact match to the commit message's claim: the count line still reads
contented, and the new tally assertion is what turns the regression red.
Restored (`git diff` empty after).

**Tried three additional ways to make a pin go unexamined while the gate
stays green** (the brief's instruction to find a *different* hole):

1. Deleted the `forkbuilt/` copy of `keyed_tr_sortedmulti_a` entirely (both
   files) while leaving it named in `forkbuiltRecordPins` and
   `forkbuiltPinnedFiles`. Caught **three ways at once**: the pre-existing
   D5g membership check (`no fork-side record ... nothing is preserving it`),
   the fold's new tally assertion (`2 pinned record(s) ... want 3`), and
   `TestForkbuiltPinsAreAClosedHashedSet` (`which is GONE`, ×2).
2. Emptied the entire `forkbuilt/` directory (all 9 files) — 9 independent
   `t.Errorf` lines from the hashed-set test, one per missing file. Not
   vacuous.
3. Reproduced M-1's consistent `multi_a(2,`→`multi_a(1,` edit directly (not
   checksum-fixed, simpler than the review's version) against the pinned
   `keyed_tr_multi_a.conformance.json`: caught by both the descriptor gate
   (`descriptor gate: 1 vector(s) failed`) and `TestForkbuiltPinsAreAClosedHashedSet`
   (hash mismatch).

No construction found that leaves a pin both present-on-disk and
never-examined while every gate reports green. Every attempt reds, usually on
more than one independent clause.

Confirmed the `forkbuiltRecordPins` names still all overlap the vendored tier
**today** (`ls` both dirs for all three names — present in both), so the union
is currently a no-op on which names any test iterates; it only diverges from
the pre-fold behavior on the day a re-vendor removes one of them, which is
exactly the event the pin exists for. This matches the review's own framing
("latent, not live").

## 2. The counts, asserted not logged — and the tier-arithmetic clause

`failed != 0` is a real, independently reachable assertion — fired correctly
in mutation 3 above (`descriptor gate: 1 vector(s) failed`).

**The "tier arithmetic" clause is not a literal tautology, but it is fully
redundant with the clause directly above it — this is the fold's one new
defect.**

```go
if want := len(forkbuiltRecordPins); passed-legacyPassed+want != passed+failed-failed {
```

`passed+failed-failed` is `passed` for any integer value of `failed` (plain
Go int subtraction, no overflow risk at these magnitudes — verified
exhaustively, not just reasoned about: swept `passed` 0..50, `failed` 0..5,
`legacyPassed` 0..5, `want` 0..5 — 11,016 combinations, **0 mismatches**
between this clause's truth value and `legacyPassed != want` (the clause
immediately above it, `if legacyPassed != len(forkbuiltRecordPins)`). Script
retained at
`/tmp/claude-1000/.../scratchpad/f630verify/algebra.go` (scratch, not
committed).

So the clause reduces algebraically to `want != legacyPassed` — the *exact
same condition* as the preceding `t.Errorf`. It **can** fire (it is
data-dependent, not `false` by construction, so "tautology" in the strict
sense is the wrong word for it) but it **cannot fire independently**: in
every mutation tried above, and in the exhaustive sweep, it agrees with the
clause above it in 100% of cases. It is blind to `passed` and `failed`
individually — despite its error message reading as if it verifies
`43 correct-header + 3 pinned-legacy != examined`, that arithmetic is
identically `passed` regardless of what `failed` or the correct-header count
actually are.

The commit message states *"Now `failed != 0` and the tier arithmetic are
both assertions"* — true as a statement about syntax (two `if` statements
exist), but one of the two is a restatement of an existing check dressed as
an independent one. **No coverage or guarantee is lost** — the real invariant
(every pin reached the legacy arm) is fully and correctly enforced by the
clause above it — so this is a test-hygiene defect, not a correctness
defect: classified **Minor**, not Important. Recorded because the brief
named exactly this risk shape as the cycle's subject matter; the honest
answer is "not vacuous, but not doing any work either."

## 3. `TestForkbuiltPinsAreAClosedHashedSet`

- **All nine hashes verified correct.** Computed `sha256sum` directly on
  `md/testdata/forkbuilt/*` and diffed against `forkbuiltPinnedFiles` —
  exact match, all nine, byte for byte.
- **Directory membership is exhaustive.** `ls md/testdata/forkbuilt/` lists
  exactly the same 9 names as the map's keys, nothing more, nothing less.
- **Catches a smuggled file:** copied
  `keyed_wsh_or_b.phrase.txt` → `forkbuilt/keyed_wsh_or_b.md1.txt` (the
  review's own repro) → `is on disk but not in forkbuiltPinnedFiles`. Fails.
- **Catches an edited pin:** flipped one byte of
  `forkbuilt/keyed_tr_multi_a.md1.txt` → `has CHANGED`, on-disk vs. pinned
  hash both printed. Fails. Also caught the M-1-style consistent
  `multi_a(2→1)` template+descriptor edit (§1.3 above).
- **Cannot pass vacuously:** the source has
  `if len(forkbuiltPinnedFiles) == 0 { t.Fatal(...) }`, and separately,
  emptying the real directory produced 9 explicit failures rather than a
  silent pass (§1, mutation 2).

All claims machine-checked by direct mutation, not by reading the code and
trusting it.

## 4. Did the fold break anything?

- `go vet ./...`: 10 lines, all `testing.ArtifactDir requires go1.26` (go1.25
  file-version notices, pre-existing, unrelated to this diff) — matches the
  established baseline exactly.
- `gofmt -l .`: exactly the five-file baseline
  (`gui/transaction.go`, `gui/transaction_golden_test.go`,
  `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`) — no new
  unformatted file.
- `go test -count=1 ./md/ ./sysw/ ./mk/`: all `ok`.
- Full gui suite via
  `scripts/gui-shard-test.sh ./gui/ 24` from the worktree: **partition
  verified exhaustive: 1374 == 1374; RESULT: ok — all 1374 tests ran across
  24 shards**, wall 21s. Matches the established 1374/1374 baseline exactly.
- Confirmed (§1) the union is a no-op on today's tree — no gui test now
  iterates a name it didn't already iterate before the fold, because all
  three pins still have live vendored twins. No silent-new-name risk exists
  today; it is the future-proofing the fold set out to add.

No regression found anywhere in the surface named in the brief's "already
established" section.

## 5. A defect the fold's own diff introduces in documentation (found, not
   requested, recording for completeness — not counted in the C/I/M/N tally
   since it is cosmetic/doc-only, matching the project's own Nit bar)

Both `eachKeyedVector` doc comments (`md/vector_fixtures_test.go` and
`gui/vector_fixtures_test.go`) now read as two paragraphs that contradict
each other: the pre-existing first paragraph still says *"The glob is over
the VENDORED directory because that is what defines corpus membership"*
(now false), immediately followed by the fold's new paragraph correcting it
(*"eachKeyedVector enumerates the UNION..."*). The fold appended rather than
replaced the stale sentence. No functional effect — call it a **Nit**,
consistent with the residue class below (comments outliving their
condition). Not included in the "new defect" tally above the line because it
carries zero risk; noted here so it doesn't get lost.

---

## Residue adjudication (1 Minor + 4 Nits left unfixed by the fold) — judged, not fixed

All four items were confirmed **structurally untouched** by the fold: `git
show 2cbca02 --unified=0 -- md/conformance_keyed_test.go` shows a single pure
insertion hunk (`@@ -278,0 +279,28 @@`) — nothing in `assertDescriptorsAgree`
(where D1, N-1, N-3, N-4 all live) or in `gui/vector_fixtures_test.go`'s
`loadVectorChunks` (N-2) was touched. All four review claims are exactly as
true today as when the review wrote them.

- **M-3 — D1's parent-fingerprint clause has no mutation row.** Non-blocking.
  This is a gap in the *implementation report's* mutation-coverage table (a
  documentation artifact), not in the shipped test: the review independently
  reproduced that the clause fires, and fires **alone** (six lines, all one
  clause, no D1/D1′/D1″/D2/D3 noise). The code is correct; the report that
  describes it over-claims. Fixing it is a documentation edit to an
  already-persisted report, not a code change — does not block a push.
- **N-1 — the dominated "no longer diverges" clause can never fire alone.**
  Non-blocking. The review's own analysis shows this is *not* a coverage
  gap — clause 1 above it necessarily fires whenever clause 3 would, with a
  message that names the actual change, so nothing goes unreported. Recorded
  because the brief asked whether each clause can fail, not because anything
  is missing. Same shape as this verification's new tier-arithmetic finding,
  and the review correctly triaged it as cosmetic when it did the same
  analysis.
- **N-2 — gui's `loadVectorChunks` lacks md's empty-pin guard.**
  Non-blocking. A real asymmetry between the two mirrored loaders (md fatals
  on an empty pin file, gui silently returns an empty slice), but a
  contentless pin is not on disk today and an empty chunk slice would fail
  loudly downstream (`DecodeChunks`/`Reassemble` on zero chunks), not
  silently pass. Worth conforming eventually; not a push blocker.
- **N-3 — elided-origin bracket pin is case-normalised despite being
  documented "verbatim and complete."** Non-blocking. Cosmetic per the
  review's own finding; `keys[].index`-level correctness is explicitly
  unaffected, and the path-hardening spelling the note was really protecting
  is preserved verbatim.
- **N-4 — a duplicated `keys[].index` defeats the ambiguity refusal in one
  direction.** Non-blocking. The review states this is unreachable from the
  primary's emitter; it is a defensive-depth gap in a test helper reading a
  fixture format the fork does not itself produce with duplicate indices,
  not a path a real input can reach.

None of the five residue items are Critical or Important by the project's
severity rubric (none is a wrong result, data loss, security issue, or unmet
guarantee) and none newly discovered here changes that. Agree with the
review's own triage on all five; none blocks this push.

---

## Tally

| # | Finding | Disposition |
|---|---|---|
| I-1 | eachKeyedVector union + tally assertion | Closed. Reproduced review's repro (46/46 preserved); reproduced commit's own second mutation claim exactly; 3 independent additional attempts to make a pin go unexamined all failed loudly. |
| M-1 | pinned record template/descriptor consistent-edit undetected | Closed by `TestForkbuiltPinsAreAClosedHashedSet`. Reproduced the exact multi_a(2→1) edit; caught by hash and independently by the descriptor gate. |
| M-2 | card-tier membership inferred from existence | Closed by the same hashed-set test. Reproduced the smuggled-file repro; caught by name. |
| M-3 (residue) | no mutation row for D1 parent-fingerprint | Left open, judged non-blocking (documentation gap, not a code gap). |
| N-1..N-4 (residue) | dominated clause / loader asymmetry / case-normalisation / duplicate-index overwrite | All left open, judged non-blocking. |

**New defects introduced by the fold: 0 Critical / 0 Important / 1 Minor / 0
Nit** (the tier-arithmetic clause is fully redundant with its neighbor —
verified by exhaustive 11,016-combination sweep plus every hand-built
mutation agreeing with the reduction; the doc-comment self-contradiction in
§5 is noted but not tallied as it is pure prose with zero behavioral risk).

**Recommendation:** the fold correctly closes I-1, M-1 and M-2, does not
regress `./md/`, `./sysw/`, `./mk/`, or the 1374-test gui suite, and the
residue is genuinely non-blocking. The one new Minor (redundant
tier-arithmetic clause) does not compromise coverage — the invariant it
restates is fully enforced by its neighbor — so it does not block this push;
it is worth a follow-up to either delete the clause or make it check
something the neighbor does not (e.g. `passed+failed == len(eachKeyedVector(...))`
against a captured total, which the current form does not do despite
appearing to).

Worktree left clean: `git status` reports nothing to commit at the end of
this verification.
