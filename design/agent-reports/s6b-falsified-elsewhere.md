# S6b — statements falsified elsewhere in the repository

Sweep scope: the 20-commit diff `b1479a1..HEAD` on the fork worktree
`/scratch/code/shibboleth/wt-s6b` (branch `s6b-pre-flash`), read via
`git log --oneline` and each commit's full message. For every named mechanism
in the dispatch brief (`singleSigVerifyFlow`'s bool return + retry loop, its
`engravedWithPassphrase` parameter, F-199's `verifyRefused`→`verifyIncomplete`,
`multisigVerifyNoSlotBody`'s `provedInnocent` arm, the plate-marking predicate,
the preloaded passphrase-plate program + restore document, `backup.Text`'s
`Title`/`Footer`), I named the old invariant in my own words and grepped for it
both in the fork tree (`gui/`, `backup/`, excluding `_test.go` files and
`third_party/`) and in `mnemonic-engrave/design/*.md` (excluding
`design/agent-reports/`, which is exempt as a historical-record directory per
the brief). The already-swept scroll/arrows/`Warning`/`maxScroll` mechanism from
commit `1cec141` was treated as closed and not re-checked. Every candidate below
was verified against the current tree or a real test run before being reported;
one is backed by a `go test -v` transcript pasted in place.

## Findings

### 1. `statusVerifiedOnRetry` is asserted "unreachable" in four places; the diff made it reachable and now tests reaching it — Important

**Mechanism falsified:** commit `511f7f3` (S6b P9, F2) turned `singleSigVerifyFlow`'s
only caller from a one-shot `if` into a `for` loop, and its own doc comment says
so ("Before P9 this flow was void and its only caller was a one-shot `if`").
That is exactly the "written at most once" / "unreachable by construction"
class the dispatch brief names. Four passages in two design docs assert the old
unreachability as a checked, PASS'd gate, and none has been updated:

- `design/S6A_STEP1_EXIT_MAPPING.md:198-207`:
  > `### The §4.8 consequence check — statusVerifiedOnRetry must be unreachable`
  > … `statusVerifiedOnRetry` requires `adverse && pass` in one call. Both adverse
  > sites are **terminal `return` statements** — `:117` and `:146` — with no loop
  > around them (F3: no retry loop; the single call site is a one-shot `if`). So no
  > control path writes `adverseRecorded` and then reaches `:149`.
  > **`statusVerifiedOnRetry` is unreachable. PASS.**

- `design/S6A_STEP1_EXIT_MAPPING.md:543-550` (the doc's own self-check section,
  repeating the claim as `C5`):
  > **C5 — `statusVerifiedOnRetry` unreachable: adverse sites are terminal.**
  > … Two adverse rows (`:117`, `:146`), both `return` statements per F2, with no
  > loop in the flow and a one-shot call site per F3. No path writes adverse then
  > falls through to `:149`. **PASS.**

- `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md:1172-1179`:
  > A consequence worth checking the mapping against. Exactly **one** of the four
  > states is unreachable from inside the eleven exits: **`statusVerifiedOnRetry`**,
  > which needs a prior adverse write *and* a later pass within one call, and
  > single-sig has **no retry loop** — `gui/singlesig.go:131` is a one-shot
  > `if sel, ok := verifyChoice.Choose(ctx, th); ok && sel == 0` with no re-offer,
  > so the flow runs at most once per engrave (§5.2). **A proposed mapping that
  > reaches `statusVerifiedOnRetry` from within the eleven exits is wrong**, and
  > that is checkable without judgement.

- `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md:1324-1329` (a test-planning
  directive, not just narration):
  > **T23 AND T24 CANNOT RUN ON THE SINGLE-SIG PATH, and §5 put every test there
  > (R3 I-3).** Single-sig has **no retry loop** — its verify is a one-shot
  > `if sel == 0 { ... }` — so `failed → abandoned` and `incomplete → complete`
  > are unreachable by construction. Those rows must be driven on a **multisig**
  > flow, which is the only place a second attempt exists.

**Why it is now false, verified by running the code:** `gui/singlesig.go` now
wraps the verify offer in a `for` loop (see the commit's own diff), and
`gui/singlesig_verify.go:96` returns a `bool`. The new test
`TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine`
(`gui/s6b_p9_failure_states_test.go:217`) drives the *real* single-sig flow
through a failed attempt then a passing one and asserts the rendered
`statusVerifiedOnRetry` line. Ran it directly:

```
$ go test ./gui/ -run TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine -v
=== RUN   TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine
    s6b_p9_failure_states_test.go:348: statusVerifiedOnRetry line for a retried single-sig pass:
        1 key plate was read back and matched what this run engraved. No secret seed share was
        read back or compared. An earlier check did not pass; a later full check passed.
--- PASS: TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine (0.13s)
PASS
```

`statusVerifiedOnRetry` is reached from the single-sig path today — the exact
outcome all four passages declare structurally impossible. The fourth passage
is the most load-bearing: it is a test-placement directive telling a future
implementer that T23/T24-shaped assertions "must be driven on a multisig flow,"
when the diff's own new test already drives that exact transition on
single-sig.

**Severity: Important.** These are stated as *checked gates* ("PASS", "checkable
without judgement"), not narrative — a reader relying on either
`S6A_STEP1_EXIT_MAPPING.md` passage to reason about what the restore document
can say, or relying on the test-placement directive to decide where new
single-sig retry coverage belongs, would conclude a live, tested, funds-adjacent
code path is dead.

### 2. The multisig twin's verdict at the readback-accounting failure is quoted as `verifyRefused`; it is `verifyIncomplete` since this diff — Minor

`design/S6A_STEP1_EXIT_MAPPING.md:368-371`, arguing single-sig's own
`singleSigReadbackCards` failure should classify as adverse:

> §4.7b binds it. The multisig twin is `gui/multisig_verify.go:701`
> (`extractReadbackMd1AndMk1s` fails → `verifyRefused`), listed in §4.7b's
> **adverse** column as *"readback filter drops cards"*. Same position in the
> flow, same kind of check, same shape of failure.

Commit `c95dd23` (F-199, part of this diff) changed exactly this site. Current
code, `gui/multisig_verify.go:779-796`:

```go
readbackMd1, readbackMk1s, ok := extractReadbackMd1AndMk1s(cards)
if !ok {
    ...
    // verifyIncomplete, NOT verifyRefused (F-199, S6b spec §3.1). ...
    rec.adverse = true
    showError(ctx, th, "Verify Bundle", "Read back one wallet-policy md1 AND the operator key card(s) (mk1).")
    return verifyIncomplete
}
```

The site now returns `verifyIncomplete`, which both multisig engrave callers
re-offer on — the opposite of the terminal `verifyRefused` the doc cites as
supporting precedent. (`design/SPEC_s6b_pre_flash_cycle.md` §3.1 documents the
change correctly; this is the one place the old classification survives as a
present-tense fact rather than as "this is what F-199 fixed.")

**Severity: Minor.** The passage is reasoning-support for a different
(single-sig) classification question that was answered correctly and is not
itself gated on the stale fact; a reader would get the multisig site's current
behavior wrong, but nothing here directs code or a safety decision.

### 3. `IMPLEMENTATION_PLAN_s6a_singlesig_truth.md` calls its clause table "the sole authority" for the pass-line text; the diff changed the text it does not know about — Minor

`design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md:973`, clause **B** of the
"four lines" table:

> | **B** | `The ms1 secret you typed matched this seed.` | iff `rec.pass.full` |

and the authority claim at line 991:

> **§4.7c IS THE SOLE AUTHORITY FOR WHAT THE BUILDER PRINTS, and it must
> therefore carry every clause.**

Commit `2c18a6f` (S6b P1, F-206, part of this diff) changed the production
string. Current code, `gui/verify_status.go:169`:

```go
verifyStatusMS1Clause = "The ms1 you typed for each seed matched."
```

confirmed by `grep -n "verifyStatusMS1Clause ="  gui/verify_status.go` and by
the in-tree regression comment at `gui/singlesig_truth_test.go:43`: `// t22MS1Clause
was "The ms1 secret you typed matched this seed." until F-206`. The clause-B row
in the "sole authority" table is the pre-F-206 string; the table was never
touched by this diff (the file lives in `mnemonic-engrave/design/`, a different
repo from the fork commits that changed the string, and `git log` on it shows no
commit in this window).

**Severity: Minor.** Clauses A, B2, C, D are unaffected and still correct; only
clause B's literal text is stale, in a document about a phase (S6a) that has
already shipped. The risk is a future reader trusting the "sole authority"
framing to look up current pass-line wording instead of the source.

## Mechanisms swept and found CLEAN

- **`singleSigVerifyFlow`'s new `engravedWithPassphrase` parameter / the
  failure-copy switch going from two arms to three (F3).** Grepped
  `design/*.md` for the three arms' wording ("Check the engraved plates",
  "This set was engraved WITH a passphrase"); the only hits are inside
  `SPEC_s6b_pre_flash_cycle.md` and `REQUIREMENTS_s6b_pre_flash_cycle.md`,
  both properly framed as the fix's own rationale, and `FOLLOWUPS.md`'s F-204
  filing (historical, describes what was found, not a live claim).
- **`multisigVerifyNoSlotBody`'s `provedInnocent` arm / R-M's wording change**
  ("Your plates are fine. Try again and skip the passphrase." →  the new
  adopted text). Grepped for the old string tree-wide: the two hits are both in
  `SPEC_s6b_pre_flash_cycle.md`, both explicitly marked "currently ends" as the
  motivating problem statement immediately preceding the adopted replacement —
  not a live claim. The in-code doc comment on `multisigVerifyNoSlotBody`
  itself (`gui/multisig_verify.go:151-170`) correctly narrates the change in
  past tense ("the operator struck the original's 'skip the passphrase'
  advice").
- **Plate-marking predicate ("the set contains a seed"; watch-only sets are
  NOT marked).** Grepped for "every plate is marked", "marking is
  unconditional", and the old call-site line number (`gui/singlesig.go:177`,
  which P2's own fixup commit `0166be4` already swept in the same diff — a
  fifth site missed by that fixup was checked and does not exist). Clean.
  `REQUIREMENTS_s6b_pre_flash_cycle.md`'s R-A section states "watch-only sets
  are NOT marked" and matches `gui/singlesig.go:297-306`
  (`singleSigPlateMark`) exactly.
- **`backup.Text` gaining optional `Title`/`Footer`.** No design doc outside
  the S6b set describes `backup.Text` as having only a body/paragraphs field;
  the only other `Text` struct hits in `design/*.md` belong to the unrelated
  `me` CLI (`mnemonic-engrave`'s own crate), a different package entirely.
- **The preloaded passphrase-plate program and its restore-document wording**
  ("nothing this device engraves carries a passphrase" retraction, P4/commit
  `639e1b2`). Grepped the exact retracted sentence tree-wide: the only hit
  outside the fix's own commit is `SPEC_s6b_pre_flash_cycle.md`, which quotes
  it as the defect being fixed, correctly.
- **F-199's `verifyRefused`→`verifyIncomplete` change, checked against
  `IMPLEMENTATION_PLAN_multisig_build_repair.md` and
  `SPEC_multisig_build_repair.md`** (the docs that originally introduced the
  five-way verdict enum): no hits for `verifyRefused` or
  `extractReadbackMd1AndMk1s` in either file, so nothing there to falsify.
- **`IMPLEMENTATION_PLAN_s6b.md`'s own "must not" list** ("P1 — must not widen
  `verifyRefused`; only `:753` re-offers, and the gate asserts the other three
  do not loop"). Still true: F-199 changed exactly one site and the other
  three `verifyRefused` returns are untouched and still terminal, per the
  fold commit's own gate output.

## Not swept

- The scroll/arrow/`Warning`/`maxScroll` mechanism, per the dispatch brief
  (already fully swept in commit `1cec141`).
- `design/agent-reports/`, excluded per the brief as historical record.
- Superseded `CONTINUITY_*.md` snapshots (e.g. `CONTINUITY_2026-08-16c.md:59`,
  which repeats the "one-shot `if`" claim): treated as point-in-time session
  notes analogous to agent-reports — the newest, `CONTINUITY_2026-08-17.md`,
  is the one anyone would actually resume from, and every earlier one is
  routinely superseded by design. Not reported, but flagged here in case that
  judgment call is wrong: if continuity docs are meant to stay individually
  accurate rather than merely superseded, `CONTINUITY_2026-08-16c.md:59` needs
  the same correction as Finding 1.
- I did not exhaustively grep every one of the ~90 other `design/*.md` files
  for incidental mentions of the mechanisms above (e.g. old T6a/T6b spike docs
  that predate the verify-tail work by months); I targeted the files the
  commit messages and `FOLLOWUPS.md` cross-reference, plus direct string
  greps for the specific old wordings and function/variable names named in
  the dispatch brief and found in the diff. A broader pass over docs with no
  textual link to these mechanisms was not attempted.
- I did not re-review `gui/*_test.go` files beyond spot-checking the ones the
  commit messages named, on the assumption (stated in the brief) that the
  scroll-mechanism sweep and the P9 fold's own test updates already covered
  test-file assertions for the mechanisms P9 touched; I did not independently
  grep every `t.Errorf`/`t.Fatalf` string in the ~900-test `gui` package
  against every mechanism.
