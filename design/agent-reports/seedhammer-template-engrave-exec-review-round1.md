# Exec Review — SeedHammer template-engrave — Round 1 (focused C1 re-review)

- **Type:** FOCUSED re-review of the single-Critical fold (C1) on top of the completed implementation. The prior whole-diff exec review (`seedhammer-template-engrave-exec-review.md`) returned NOT GREEN — 1C/0I and verified everything else clean; this round confirms C1 is correctly resolved, the fold introduced no drift, and gives the final verdict. Resolved-clean items are NOT re-litigated (no regression evidence found).
- **Reviewer:** opus architect (independent).
- **Date:** 2026-06-20.
- **Under review:** the C1 fix commit `6583e5f` ("fix(md): template guard admits legacy multi-in-combinator (exec-review C1)") on top of branch HEAD `3d328d6` (the 8 implementation commits), worktree `/tmp/seedhammer-wt-template`, branch `feat/template-engrave`.
- **SPEC:** `design/SPEC_seedhammer_template_engrave.md` (DD3/DD7 keep the §5 general-miniscript wallet in scope).

---

## VERDICT

**GREEN — 0 Critical / 0 Important.** C1 is correctly RESOLVED; the fold is comment-/test-/one-arm-only with no drift; the full suite is GREEN; M2 deferral is acceptable.

---

## C1 — RESOLVED

**Finding (prior review):** the guard refused BOTH `tagSortedMulti` AND `tagMulti` under a combinator (`case tagSortedMulti, tagMulti:` + `if inCombinator { refuse }`), over-refusing the §5 degrade2 wallet (`wsh(or_i(and_v(...multi(3,...)),...))`) that the authoritative toolkit `template_admissible` ADMITS — contradicting DD3/DD7 and the Task-4 strip golden.

**Resolution evidence (all four guard-correctness checks pass):**

1. **Guard now ADMITS legacy `multi`-in-combinator.** `md/template_guard.go:61-65` is a standalone `case tagMulti:` that returns `nil` **unconditionally** (no `inCombinator` test). The `tagSortedMulti` arm (`:55-60`) is now separate and still `return errTemplateUnsupportedShape` only `if inCombinator`, else `nil`. The split is exactly as the C1 fix prescribed.

2. **Render-gap shapes still REFUSED (correct half not broken).**
   - `tagSortedMultiA` (`:49-51`) → unconditional refuse (any tap leaf). The `tagTr` arm (`:89-97`) descends into the tap tree with `inCombinator=true`, and `tagTapTree` (`:98-106`) likewise — so a `sortedmulti_a` on any leaf is reached by its own always-refusing arm; a `sortedmulti` (non-`_a`) leaf in a tap tree is refused via the in-combinator `tagSortedMulti` arm. `tr(NUMS, multi_a)` is admitted (`tagMultiA` `:52-54`).
   - `sortedmulti`-in-combinator → refused: combinator tags (`or_i`/`and_v`/`thresh`/…) have NO named case (confirmed: the only `case tag…` labels are SortedMultiA, MultiA, SortedMulti, Multi, Wsh, Sh, Tr, TapTree), so they fall to the `default` arm (`:107-124`) which descends with `inCombinator=true`; a `tagSortedMulti` reached there refuses. Verified live: `TestTemplateEngraveShapeGuard/refuse/wsh(or_i(sortedmulti,...))` and `/refuse/tr(sortedmulti_a)` both PASS (each first asserts the shape still `split()`-encodes — a non-vacuous precondition — then asserts the guard refuses).

3. **§5 degrade2 wallet ADMITTED — and the regression test is non-vacuous.**
   - `TestTemplateGuardAdmitsDegrade2` exists (`md/template_guard_test.go:145-150`), loads the **real committed golden** via `loadTemplateMD1(t, "degrade2_11key.tmpl.md1.txt")` (helper at `md/template_strip_test.go:18-25`, `os.ReadFile` of `testdata/template/…`; golden present, 342 bytes), reassembles it through the production `TemplateEngraveShapeGuardChunks`, and asserts `nil`. PASS.
   - **Non-vacuity of the golden:** an in-review throwaway probe (added, run, then removed — `git status --porcelain` clean afterward) walked the decoded golden tree: it contains **4 `tagMulti` (0x06) legacy-multi nodes, 0 `tagSortedMulti`, and `tagOrI` (0x19) combinators**, all nested below the single `wsh` under combinator spine. So the test genuinely exercises legacy-multi-under-combinator (the exact shape C1 was about), not a degenerate tree.
   - **Non-vacuity of the constructed admit case:** the new `wshOrILegacyMultiGuard` (`:58-69`) builds `wsh(or_i(multi(2,@0,@1), multi(1,@0,@1)))` and is registered in the `admitted` map as `"wsh(or_i(multi,...)) legacy"`. Because `or_i` (`tagOrI`) has no named case it descends via `default` with `inCombinator=true`, so the `multi` leaves are evaluated under `inCombinator==true` — i.e. the test would FAIL if the old combined-arm refusal were still present. Verified live: `TestTemplateEngraveShapeGuard/admit/wsh(or_i(multi,...))_legacy` PASS. (The companion `wshOrISortedMultiGuard` REFUSE case is preserved, so the guard isn't trivially admit-all.)

4. **Tests run (not skipped), ALL GREEN.**
   - `go build ./...` → exit 0.
   - `go test -count=1 ./md/... ./bundle/... ./gui/...` → ALL PASS: `md` 0.022s, `bundle` 0.010s, `gui` 12.9s, `gui/op|saver|text|widget` PASS; `gui/assets`,`gui/layout` no test files.
   - Verbose guard run confirms the relevant tests EXECUTE (no SKIP): `TestTemplateEngraveShapeGuard` (2 refuse + 4 admit subtests, all RUN/PASS), `TestTemplateGuardAdmitsDegrade2` PASS, `TestTemplateGuardHardenedUseSiteStrips` PASS, `TestStripToTemplateGolden` (incl. the `degrade2_11-key general (non-canonical → keep origins)` subtest) PASS.

**C1: RESOLVED.** The guard now matches the verified authoritative toolkit semantics (refuse only `tr(sortedmulti_a)` and `sortedmulti`-in-combinator; admit legacy `multi` everywhere), the §5 wallet is admitted with a non-vacuous golden-pinned regression test, and the SortedMulti/SortedMultiA refusals are intact.

---

## Drift / scope of the fold — NONE

`git show 6583e5f --stat` shows exactly three files, all in-scope for the prescribed fold:

- `md/template_guard.go` (+14/-8): the one-arm split (`tagSortedMulti` / `tagMulti` separated) plus comment corrections (file-header `:7-11`, `guardNode` doc `:43-46`, `tagTr` doc `:89-93`, `default` doc `:107-109`, and `tagMulti` inline). No other logic touched; `templateEngraveShapeGuard`, `TemplateEngraveShapeGuardChunks`, the wsh/sh spine handling, and the error var are unchanged in behavior.
- `md/template_guard_test.go` (+30): the new `wshOrILegacyMultiGuard` constructor, its `admitted`-map entry, and `TestTemplateGuardAdmitsDegrade2`. Pure additions; no existing case removed or weakened (the `refuse` map still holds `tr(sortedmulti_a)` and `wsh(or_i(sortedmulti,...))`).
- `md/template_strip.go` (+3/-1): **M1 — comment-only.** The doc text changed from "on a decoded clone" to "IN PLACE on the freshly-decoded descriptor (M1: Reassemble returns a NEW descriptor per call, …)". The `StripToTemplate` function body (`:27-53`) is byte-for-byte unchanged — same `Reassemble`, same four mutations (pubkeys/pubPresent, fingerprints/fpPresent), same C1-conditional `canonicalOrigin` origin elision, same `split(d)` re-emit. StripToTemplate behavior is unchanged; the M1 edit only corrects the stale "clone" wording. Confirmed independently by `TestStripToTemplateGolden` still GREEN across all three goldens (including degrade2 keep-origins).

No source outside these three files was modified. No prior-review PASS item shows any regression. The throwaway probe used to confirm golden non-vacuity was removed; tree left clean.

---

## M2 deferral — ACCEPTABLE (not a blocker)

M2 (WDT-Id override-TLV branch exercised only structurally, no golden for a present-`UseSitePathOverrides` template) is a Minor and was filed as the followup `seedhammer-wdt-id-override-tlv-golden`. The override branch cannot diverge any pinned golden (no override in the vectors) and the byte order matches Rust on inspection. Deferring it does not block this gate.

---

## What was run
- `go build ./...` → exit 0.
- `go test -count=1 ./md/... ./bundle/... ./gui/...` → ALL PASS.
- `go test -v -run 'TestTemplateEngraveShapeGuard|TestTemplateGuardAdmitsDegrade2|TestTemplateGuardHardenedUseSiteStrips|TestStripToTemplateGolden' ./md/` → all RUN + PASS, no SKIP.
- `git show 6583e5f --stat` (3 files: guard + guard_test + strip-doc) and `git show 6583e5f` (confirmed the strip change is comment-only and the guard arm split matches the prescribed fix).
- Throwaway tree-walk probe over the decoded `degrade2_11key.tmpl.md1.txt` golden (4× legacy `multi`, 0× sortedmulti, under `or_i` combinators) to prove the regression test is non-vacuous; probe removed, `git status --porcelain` clean.

---

**VERDICT: GREEN — 0 Critical / 0 Important.**
