# Track A (verify-cluster) — whole-diff execution review (round 0) — VERBATIM agent report

**Agent:** `a3a2261d060002d0d` (adversarial opus architect; mandatory post-implementation gate; reproduced all 3 masking-proofs by real negative-control reverts in throwaway worktrees off the tip). **Branch:** `feat/fix-verify-cluster`. **Base:** `3a23dbb`. **Tip:** `d45a4af`. **Date:** 2026-06-20.
**Verdict:** GREEN (0 Critical / 0 Important). MERGE recommended. 2 cosmetic Minors.

---

# R0 EXEC REVIEW — feat/fix-verify-cluster (Track A) — whole-diff post-implementation gate

**Reviewer:** adversarial opus architect (mandatory non-deferrable post-implementation exec gate; applied real negative-control reverts + captured production-routed error strings in a throwaway worktree off the tip).
**Branch:** `feat/fix-verify-cluster` | **Base:** `3a23dbb` (fork main) | **Tip:** `d45a4af`
**Commits:** M1 `ec3f231`, H2 `883d764`, H1 `b2c3231`, L1 `cae68bf`, L2 `2a57307`, relabel `d45a4af`
**Verdict: GREEN — 0 Critical / 0 Important. MERGE.**

The diff faithfully implements the GREEN plan (`00e1530`) + spec (`803296a`) + both R0 folds. The three flow-level tests fail-before/pass-after through the production functions; I reproduced all three masking-proofs by real reverts, including the decodable-but-wrong foreign-mk1 discrimination via the actual stub-binding leg. No scope creep, `gui/multisig.go` untouched, zero new vet findings, full suite green, commit hygiene satisfied.

## Per-mandate findings (with observed output)

### Mandate 1 — Diff fidelity / scope: PASS
- `git diff 3a23dbb..d45a4af` touches exactly the 9 planned files (5 production + 4 test): `bundle/verify.go`(+test), `gui/md1_gather.go`(+test), `gui/multisig_supply.go`(+test), `gui/multisig_verify.go`(+test), `gui/singlesig_verify.go`. Matches the plan's "Files touched" table verbatim.
- **`gui/multisig.go` UNTOUCHED:** `git diff 3a23dbb..d45a4af -- gui/multisig.go` is empty. H1 added a NEW `extractSuppliedMd1AndMk1` and did NOT widen `extractSuppliedMd1`; bare `extractSuppliedMd1` now has its only non-test/non-fuzz caller at `gui/multisig.go:71` (the live engrave/supply flow) — unaffected.
- **`wipeBytes`** (`gui/slip39_polish.go`) called, not edited (no diff).
- **No Track-B file touched:** `slip39/combine.go`, `seedxor/seedxor.go`, `gui/bip85.go`, `gui/codex32_polish.go`, `gui/slip39_polish.go` all empty diffs. (Track B's L1 codex32 scrub is on the separate `feat/fix-scrub-batch` branch — correctly NOT in this diff.)
- No unplanned hunk / no signature ripple beyond the planned `ms1Entropy` 2-extra-returns.

### Mandate 2 — Verification-fix correctness: PASS (all 5)
- **H1:** `multisig_verify.go:118` now calls `verifyMultisig(reDerived, ms1Readback, suppliedMk1, suppliedMd1)` — the real read-back mk1 from `extractSuppliedMd1AndMk1` (`:73`), not `reDerived.MK1`. The `derived` param is kept as the comparator baseline. Helper return order `(md1, mk1, ok)` consistent across def/tests/wiring.
- **H2:** `collected()` walks `for i := 0; i < g.total; i++` (index order). No signature change → `equalStrings` stays positional-by-contract. All 3 sites (`md1_gather.go:83,147` + `bundle.go:234`) are `complete()`-guarded; the distinct `mk1Gatherer.collected()` (`bundle.go:194`, `mk1_inspect.go:259`) is unaffected.
- **M1:** `Verify` compares `dLang != rLang` (LANGUAGE, not raw prefix). `TestVerifyBundleLanguageEnglishNotOverRejected` confirms English/`entr` is NOT over-rejected.
- **L2:** named constants `multisigVerifyOKTitle="Verify OK"` / `multisigVerifyOKBody="Operator key and secret verified. Other cosigners' keys are taken as supplied."` Single-sig flow's own `verifySingleSig` success copy untouched.
- **L1:** both verify-flow probes (`multisig_verify.go:109`, `singlesig_verify.go:119`) now `_, _, ent, err := codex32.DecodeMS1(s)` + `wipeBytes(ent)`. Inner-scope `err`/`ent` freshly declared with `:=`, both used — no shadow/unused (vet clean).

### Mandate 3 — Negative controls / masking-proof: PASS (all reproduced by real reverts)
- **T-H1** (production-routed, captured directly):
  - FIXED wiring (real readback foreign mk1): `err=verify: readback mk1/md1 stub mismatch (key card does not bind to this policy)` → FAIL via the stub-binding leg (decodable-but-wrong discriminator, not a trivial decode failure).
  - MASKED wiring (self-compare `verifyMultisig(derived, …, derived.MK1, derived.MD1)`): `err=<nil>` → the wrong plate silently PASSES (the bug).
  - Routes the production `extractSuppliedMd1AndMk1` + `verifyMultisig`, not a stub. All 4 subtests + the extraction tests PASS.
- **T-M1** (reverted the `if dLang != rLang` block): `TestVerifyBundleLanguageMismatch` → `verify_test.go:259: language-differ readback (same entropy) accepted, want FAIL` → FAIL (bug reproduced); `TestVerifyBundleLanguageEnglishNotOverRejected` stays PASS. After restoring the fix both PASS (error `verify: ms1 wordlist/language mismatch`). Routes production `Verify`/`ms1Entropy`.
- **T-H2** (reverted `collected()` to map-range): `TestMD1GathererCollectedIndexOrder` → `collected()[0]="md1f9k2szsl3..." want index order "md1f9k2szspqj..."` → FAIL (the discriminator); `TestVerifyBundleMd1PositionalContract` (relabeled) stays green (it is a `bundle` comparator test, unaffected by the gather-layer revert). After restoring, the index-order test PASSES across 4 orders × 10 trials.
- Old masked tests not relied upon: `TestVerifyMultisig` (`:30` self-feeds `derived.MK1`) still PASSES but is NOT the discriminator; `TestVerifyBundleMd1Reordered` is gone (`go test -run` reports "no tests to run").

### Mandate 4 — Build / test / vet / no-regression: PASS
- `go build ./...` → clean (exit 0).
- `go test ./...` → every package `ok`, 0 FAIL (incl. `bundle`, `gui`, the pre-existing `TestVerifyMultisig`).
- `go vet ./gui/ ./bundle/` → clean (exit 0).
- `go vet ./...` at tip vs cold `3a23dbb`: both 38 lines, byte-identical after path normalization → zero new findings (all 38 are pre-existing go1.25/1.26 `testing.ArtifactDir` + `bspline` unkeyed-fields notes).

### Mandate 5 — TinyGo + secret-hygiene: PASS (by inspection)
- No build tags on any touched file. Production diff contains no reflection/unsafe/goroutine/generics/cgo. Changes are: loop reorder (H2), two extra `int` returns (M1), two pure helpers, string constants, `wipeBytes` calls. `tinygo` NOT in PATH → device build correctly deferred to the controller's integration gate (consistent with plan Task 7 Step 3 + both R0 reviews). L1 scrubs improve hygiene with no behavior change.

### Mandate 6 — Commit hygiene: PASS
- 6 commits, no merge. Each carries a raw `gpgsig -----BEGIN SSH SIGNATURE-----` (confirmed via `git cat-file commit`; `%G?`=`N` only because this env has no allowed-signers file — expected). Each: `Signed-off-by: Brian Goss <goss.brian@gmail.com>` (DCO), `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`, author `Brian Goss <goss.brian@gmail.com>`.
- Clean per-commit file boundaries (explicit-path staging): M1=verify.go(+test); H2=md1_gather.go(+test); H1=multisig_supply.go(+test)+multisig_verify.go(+test); L1=the two scrub sites only; L2=multisig_verify.go copy(+test); relabel=verify_test.go only.
- Each of the 6 commits builds (`go build ./...` OK) and tests green (`bundle`+`gui` `ok`) at that commit — self-contained.

## Critical
None.

## Important
None.

## Minor
1. **`extractSuppliedMd1AndMk1` has no fuzz coverage**, whereas its sibling `extractSuppliedMd1` is fuzzed (`gui/multisig_fuzz_test.go:49`). The new helper is a simple non-panicking switch with explicit unit tests covering all 6 acceptance/rejection cases, so this is cosmetic/non-blocking — the plan did not mandate a fuzz target. (Optional future polish.)
2. **L2 body wording** is "Operator key and secret verified" vs the spec sketch's "Operator key + secret verified" — semantically identical; the regression test asserts the load-bearing substrings (`"taken as supplied"` present, `"matches the seed"` absent). Non-blocking (already noted in plan-R0 Minor 2).

## Verified-correct list
- Production diff is exactly the 5 fixes; `gui/multisig.go` empty diff; no Track-B file touched; `wipeBytes` called not edited.
- All OLD-side anchors matched live `3a23dbb` source (every hunk applied without fuzz).
- H1 masking-proof: FIXED→`stub mismatch` FAIL, MASKED→`<nil>` PASS, captured from the production `extractSuppliedMd1AndMk1`+`verifyMultisig` path.
- M1 negative control reproduces the false-PASS; English not over-rejected.
- H2 negative control reproduces the false-FAIL discriminator; positional-contract guard stays green.
- Fan-out re-confirmed: `ms1Entropy` only `bundle/verify.go:83,87`(+def); `md1Gatherer.collected()` at `md1_gather.go:83,147`+`bundle.go:234`; bare `extractSuppliedMd1` only at engrave `multisig.go:71`; `extractSuppliedMd1AndMk1` sole caller `multisig_verify.go:73`.
- Zero new vet findings; full `go test ./...` green; each commit self-contained, SSH-signed, DCO'd, correctly trailered.

## Bottom line
The implementation is a faithful, surgical realization of the GREEN plan. The test-masking defeat — the core purpose of this gate — holds under adversarial revert: each of the three flow-level tests is genuinely load-bearing (fails when its fix is reverted) and routes the named production function, with the H1 case discriminating via the real stub-binding leg rather than a trivial decode failure. No scope creep, no shared-verify-path regression, no over-correction. **MERGE.**

**Fork hygiene:** left clean on `main @ 3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`, working tree empty; branch `feat/fix-verify-cluster` intact at `d45a4af9d9fb4ac33c8a8b58d0670e7ccfedf984`. Throwaway worktrees (`/tmp/exec-A`, `/tmp/exec-A-base`, `/tmp/exec-A-neg`) removed + pruned. The implementer's worktree, Track B's, and `seedhammer-wt-bip39` left untouched.
