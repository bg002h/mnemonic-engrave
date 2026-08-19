# Operator-stand-in decisions — DoNextList items 8 and 9

**2026-08-18. Decided by a stand-in agent while the operator was asleep; the
operator can overrule either call in one line.** No tracked source file was
changed and nothing was committed. Inputs: `design/DoNextList.md` items 8/9,
`design/agent-reports/f210-journey-capture-exec-review.md` (I-1),
`scripts/plan-cite-check.sh`, `scripts/plan-cite-gate.sh`,
`scripts/plan-build-gate.sh`, `scripts/push-master.sh`,
`.github/workflows/release.yml`, `design/journeys/*`, and
`design/DRAFT_round_trip_journey_definition.md`.

---

## DECISION 1 (item 8) — the doc gates and CI

**THE CALL: (d), with (c)'s scoping as the mechanism — wire
`plan-cite-check.sh` into `scripts/push-master.sh` as a blocking,
changed-docs-only step; CI is the wrong home for the cross-repo gate, and
`plan-build-gate.sh` stays a fold-time local tool.**

### Reasoning

- **The push ritual is already the one mandatory gate `master` passes
  through.** Branch protection's chicken-and-egg makes `push-master.sh` the
  only path that doesn't print a bypass line, the push agent runs it as a
  matter of course, and the script already enforces preconditions (clean tree,
  freeze check). A gate there is "impossible to forget" without any new
  enforcement machinery — and unlike a pre-commit hook it is tracked, shared,
  and not bypassable with `--no-verify`.
- **Only the developer machine can check what actually needs checking.** The
  constellation's docs cite across repos heavily (the DoNextList repair table
  was mostly cross-repo cites). CI structurally lacks four of the six roots.
- **(a) — sibling checkout in the workflow — is declined, and not for
  speed.** All five siblings are public under `bg002h` (verified via
  `gh api`; depth-1 clones would be cheap, no secrets needed). The real defect
  is semantic: CI would resolve citations against sibling **origin HEAD**,
  while authors write docs against the **local** sibling state, often in the
  same session as unpushed sibling work. Push-ordering races would turn the
  gate red on *correct* docs — and by this repo's own measured words, a gate
  that reds on correct work trains the reader to ignore it exactly as fast as
  one that is always green. The local roots are, by construction, the state
  the doc was written against.
- **(b) — in-repo-only CI — is declined as the primary move** because it
  gates the minority slice while wearing the appearance of coverage ("a gate
  that hides its own blind spot is worse than no gate"). It is acceptable
  later as a backstop for external PRs *only* if it prints an explicit
  "N cross-repo citations NOT checked here" coverage line and excludes
  `design/agent-reports/`. Filed as an ownerless follow-up, not part of this
  decision's critical path.
- **(e) — do nothing — is declined on measured yield**: the repaired gate
  found 13 dangling citations in a 12-doc sample and a real error in this
  session's own docs. Discipline decays; the yield says enforce.
- **Scope is changed-docs-only** (`origin/master..HEAD`), matching "a fold is
  authorship and re-earns the gate". The 201 + 633 corpus is NOT cleaned in a
  campaign — it converges opportunistically, one doc at a time as docs are
  touched. **`design/agent-reports/` is excluded permanently and the
  exclusion is named in the gate's output**: reports are persisted verbatim
  and never edited afterwards, so a red gate on a report would demand a
  forbidden edit. A dangling citation inside a report is information about
  the review, not a defect to fix.
- **`plan-build-gate.sh` stays out of CI** because it is not generic: it is
  hardcoded to one plan's files (`src/seal/*.rs`, `tests/seal_cli.rs`) —
  per the standing rule, each repeatedly-folded plan commits its *own*
  extractor. The fold rule already binds it to fold commits. Trigger to
  revisit: a fold ever reaching `master` uncompiled.

### First concrete step

Add ~10 guarded lines to `scripts/push-master.sh` after the dirty-tree
precondition (before the staging push):

```
DOCS=$(git diff --name-only --diff-filter=d "origin/$BRANCH..HEAD" -- \
        'design/*.md' 'design/**/*.md' ':(exclude)design/agent-reports/**')
if [ -n "$DOCS" ] && [ -x scripts/plan-cite-check.sh ]; then
  ./scripts/plan-cite-check.sh $DOCS || die "citation gate failed on changed docs"
fi
```

Guarded on the script existing, so `push-master.sh` stays repo-agnostic for
siblings that lack the checker. Then push that change through the ritual
itself — the gate's first execution is on the commit that introduces it,
satisfying "a gate that has never been executed is a hypothesis".

### Traded away

Continuous whole-corpus verification (known dirt stays until touched); any
coverage for pushes made outside the ritual (mitigated: a direct push already
prints a bypass line, which the project treats as failure); and CI-visible
enforcement for external PR contributors (filed as the optional (b) backstop,
not built now).

---

## DECISION 2 (item 9) — `backup-strings.txt` has no producer

**THE CALL: (a) — both journeys generate `out/backup-strings.txt` from their
own inputs and encodes, `me bundle` engraves that file, the tracked
`inputs*/backup-strings.txt` fixtures are deleted, and transcripts + PDFs are
re-recorded; do it BEFORE the constellation round-trip audit dispatches.**

### Reasoning

- **(b) recreates the defect by construction** — regenerate-and-pin is the
  current state with a newer timestamp, and the project has measured, twice
  in one week (F-210, then I-1), what happens to artifacts that vouch for a
  process nobody re-runs.
- **(c) collapses into (b) plus a tripwire**: the consistency check is red
  *today* (the fixture is already stale against `mk 0.13.0`), so it forces a
  regeneration anyway and still leaves a file nothing produces. Its only
  virtue — detecting print/engrave divergence — is delivered *structurally*
  by (a): same run, same tool, same inputs, so divergence becomes impossible
  rather than merely detected. (a) is the only option that removes the class,
  and it is the exact rule the DRAFT definition §5 already codifies ("must
  not read an intermediate that nothing in the journey writes" — this file is
  an intermediate wearing an input's clothing; the journey's true origins are
  the seeds, xpubs and policy).
- **Feasibility was verified, not assumed.** Operator journey: every
  `inputs/keys/cosigner-*.xpub` carries its origin in a header comment
  (checked 00, 01, 11: e.g. `# cosigner 1 — origin [ff4bdd8b/48h/0h/0h/2h]`),
  and the fixture composition is 1 md1 + 12×2 mk1 = 25 lines — so a loop over
  the key files plus the already-captured md1 reproduces the file exactly.
  Pathological: 3 md1 chunks + 11×2 mk1 = 25, but the key files carry **no**
  origin headers (the transcript hardcodes key-00's `73c5da0a` /
  `m/84'/0'/0'`) — the one named prep step is adding origin headers to those
  11 files, values extracted mechanically via `mk decode` of the current
  fixture (field-semantics proven stable across all three string generations
  by the I-1 review).
- **On the operator's caution about changing engraving fixtures**: nothing
  key-material changes — the new strings are re-encodings of the same xpubs,
  field-identical under `mk decode` (review-proven). Any physical steel cut
  from the old fixture remains valid and decodable; the old strings stay in
  git history, and the commit message should name all three superseded
  generations. Because this changes what gets engraved it stays in the risk
  set: one implementer, one independent execution review over the diff — the
  F-210 pattern that just worked.
- **Timing: before the audit dispatches.** The audit is blocked on the
  operator ruling the DRAFT's §8 open items anyway, so this fills that gap
  rather than delaying anything. Landing it first stops the audit from
  re-purchasing a known finding (a journey that prints one card and engraves
  another is exactly what its §5 lens will flag), and gives the audit a
  clean exemplar of the anti-requirements instead of a counterexample.
- **Downstream simplifications**, both free: the review's open question
  "which artifact is canonical" resolves — the *generator* is canonical, and
  committed transcripts/PDFs are re-recordings refreshed whenever
  regeneration changes them; and item 5's pending fixture-key regeneration
  (depth-4 xpubs) becomes edit-inputs-and-re-run instead of another hand-move
  of a pinned file — so land this before item 5.

### First concrete step

In `design/journeys/transcript.sh` (the operator journey — no missing
metadata), insert before step 4: a loop over `inputs/keys/cosigner-*.xpub`
parsing each header's `[fp/path]`, running `mk encode --from-md1 "$MD1"` per
cosigner, and assembling `out/backup-strings.txt` from the captured md1 plus
all 24 mk1 chunks; point `me bundle --in` at it; delete
`inputs/backup-strings.txt`; re-record `transcript.txt` and rebuild the PDF.
Pathological journey follows in a second commit after the 11 origin headers
are added.

### Traded away

The engraved plate set stops being a pinned fixture — it tracks the installed
toolchain, so every future `mk` release turns journey regeneration into a
reviewable diff plus a transcript/PDF re-record instead of a no-op. That cost
is the point: drift becomes a visible diff instead of a silent contradiction
between step 2 and step 4. The transcripts also grow (12 encode blocks
instead of 1), lengthening the PDFs.

---

## Facts machine-checked for these decisions

| fact | how checked |
| --- | --- |
| all 5 siblings public on GitHub (7–64 MB repos) | `gh api repos/bg002h/<r>` |
| `push-master.sh` is one tracked command, repo-agnostic, with die-on-bypass | read the script |
| `plan-build-gate.sh` is hardcoded to the seal plan's files | read the script header |
| operator xpub headers carry origins; pathological ones do not | `head -1` on 00/01/11 and key-00/key-10 |
| fixture composition 1+24 / 3+22 = 25/25 | `grep -c '^md1' / '^mk1'` |
| `backup-strings-tr.txt` has 0 consumers (positive control: 4 hits for `backup-strings`) | grep with control |
| `transcript.sh` already carries the I-2 fold (loud CAPTURE FAILED path) | read the script |

## Loose end filed (not decided here)

`inputs-pathological/backup-strings-tr.txt` and `wallet-policy-tr.txt` have
**zero consumers** — a fixture pair nothing reads and nothing produces, the
same class one level down. Fold its disposition into the item-9 change
(default: delete; alternative: give the tr variant its own journey, which the
LOOSE ENDS entry on the balanced-tr fixture already gestures at). Operator's
pick.
