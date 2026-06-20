# Own-code fix — combined-diff integration exec review (final pre-push gate) — VERBATIM agent report

**Agent:** `a37738279fa835faf` (adversarial opus architect; final pre-push integration gate; reproduced all 4 load-bearing negative controls by real reverts in a throwaway worktree off the merged tip). **Fork HEAD:** `39cb5cf` (main, pre-push). **Base:** `3a23dbb`. **Date:** 2026-06-20.
**Verdict:** GREEN (0 Critical / 0 Important). PUSH recommended. Remaining post-push gate: the TinyGo device build in CI. 1 cosmetic Minor (brief-phrasing, not a code issue).

---

# Combined-Diff Integration Execution Review — SeedHammer Own-Code Bug-Hunt Remediation

**Reviewer:** Adversarial opus architect (final pre-push integration gate; reproduced all 4 load-bearing negative controls by real reverts in a throwaway worktree off the merged tip)
**Fork HEAD:** `39cb5cf` (`main`, NOT yet pushed) · **Base:** `3a23dbb` · **History:** `3a23dbb` → `2ea7754` (merge Track B) → `39cb5cf` (merge Track A)
**VERDICT: GREEN — 0 Critical / 0 Important. PUSH.**

The merge combined the two independently-GREEN tracks correctly: true file-disjointness, no merge artifacts, no evil-merge content, all 8 findings present and correct in the integrated tree, full build/test/vet green together, and all 4 masking-proofs reproduce in the merged tree exactly as in the per-track reviews. One documented Minor (a phrasing mismatch in the brief, not a code defect).

## Per-mandate findings (with observed output)

### Mandate 1 — Combined diff fidelity / no drift: PASS
- `git diff --stat 3a23dbb..39cb5cf` touches exactly the 17 expected files (9 Track A + 8 Track B), nothing else.
- **True file-disjointness:** `comm -12` of the two tracks' file sets is EMPTY; the union exactly equals the combined-diff file set. No file touched by both tracks.
- **No merge-conflict artifacts:** grep for `<<<<<<<`/`=======`/`>>>>>>>` in the added lines → NONE.
- `gui/multisig.go` UNTOUCHED; `gui/slip39_polish.go` (`wipeBytes` helper) UNTOUCHED — both empty diffs.
- **New production symbols:** Track B's only new production symbol is `bip85PkeyHook` (test-only, nil-guarded). Track A additionally introduces its planned symbols `ms1Entropy` (bundle/verify.go), `extractSuppliedMd1AndMk1` (gui/multisig_supply.go), and the L2 const block (`multisigVerifyOKTitle`/`Body`) — all explicitly documented in Track A's GREEN plan + exec review (Minor 1 below).

### Mandate 2 — All 8 findings closed in the MERGED tree: PASS
- **H1** — `gui/multisig_verify.go:118`: `verifyMultisig(reDerived, ms1Readback, suppliedMk1, suppliedMd1)` — real operator read-back from `extractSuppliedMd1AndMk1` (`:73`), NOT `reDerived.MK1`.
- **H2** — `gui/md1_gather.go:64-70`: `collected()` walks `for i := 0; i < g.total; i++` (index order over the `map[int]string`).
- **M1** — `bundle/verify.go:103`: `Verify` compares `dLang != rLang` (codex32 LANGUAGE via `ms1Entropy`), not raw prefix.
- **L2** — `gui/multisig_verify.go:17-18,122`: honest copy "Operator key and secret verified. Other cosigners' keys are taken as supplied." — no "matches the seed" over-claim.
- **L1 (all 3 sites)** — `DecodeMS1` probe entropy scrubbed: `singlesig_verify.go:119/124`, `multisig_verify.go:109/114`, `codex32_polish.go:103/104` (all `wipeBytes(ent)`).
- **M2** — `slip39/combine.go:87-92`: `defer` scrubs all group-share secrets + `ems` on success + all 3 error returns; `wipe(d)` on both the digest-fail branch (`:148`) and success (`:151`).
- **M3** — `seedxor/seedxor.go`: `wipe(e0)` (`:40`), `wipe(out)`+`wipe(e)` on mismatch (`:48-49`), `wipe(e)` on success (`:55`), `wipe(out)` on bad-length and final.
- **M4** — `gui/bip85.go:110`: `defer pkey.Zero()` immediately after `priv := pkey.Serialize()`.

### Mandate 3 — Load-bearing negative controls re-run IN THE MERGED TREE: PASS (all 4 behave exactly as in the per-track reviews)
- **(a) H1 masking-proof:** FIXED wiring → "decodable-but-wrong foreign mk1 → FAIL via stub binding" subtest PASSES (rejects via the real `stub mismatch` leg, not a trivial decode failure); "masking proof: self-compare PASSES the foreign mk1 (the bug)" subtest PASSES (demonstrates the bug). **Stronger-than-test finding:** reverting the production call site (`:118`) to the masked `reDerived.MK1`/`reDerived.MD1` form makes `suppliedMk1` unused → the Go compiler refuses to build (`declared and not used: suppliedMk1`). The H1 fix is structurally compiler-protected against the self-compare regression, on top of the production-routed unit subtests.
- **(b) M4:** strip `defer pkey.Zero()` → `TestDeriveBip85Child_ScrubsPkey` FAILS (`bip85_test.go:526: pkey.Key not zeroed after deriveBip85Child returned`); restore → PASS.
- **(c) T-M1:** drop the language compare → `TestVerifyBundleLanguageMismatch` FAILS (`verify_test.go:259: language-differ readback (same entropy) accepted, want FAIL` — false-PASS reproduced); `TestVerifyBundleLanguageEnglishNotOverRejected` stays PASS; restore → both PASS.
- **(d) T-H2:** revert `collected()` to map-range over `g.set` → `TestMD1GathererCollectedIndexOrder` FAILS (wrong index-0 chunk); restore → PASS.

### Mandate 4 — Full integration build/test/vet on the merged tree: PASS
- `go build ./...` → clean (exit 0).
- `go test ./...` → every package `ok`, 0 FAIL (Track A flow tests + Track B scrub tests + all inherited tests pass together; `bundle`, `gui`, `slip39`, `seedxor` all green).
- `go vet ./bundle/ ./gui/ ./slip39/ ./seedxor/` → clean on the merged tree AND on cold `3a23dbb` → zero new vet findings (pre-existing `bspline`/`backup`/`engrave`/`gui/op` go1.25/1.26 notes correctly out of scope). go1.26.4.

### Mandate 5 — Merge + commit hygiene: PASS
- Both merges `2ea7754` and `39cb5cf` are `--no-ff` (2 parents each), SSH-signed (raw `gpgsig … BEGIN SSH SIGNATURE`), DCO `Signed-off-by: Brian Goss`, `Co-Authored-By: Claude Opus 4.8 (1M context)`, author Brian Goss.
- All 10 underlying fix commits: same — signed + DCO + co-authored + author Brian Goss.
- No stray/unsigned commit in `3a23dbb..39cb5cf` (12/12 carry all four attributes).
- **No evil merge:** `diff(2ea7754, a74c1bc)` is EMPTY (merge-1 tree == Track-B tip); `diff(2ea7754..39cb5cf)` == Track-A's 9-file branch diff; `diff(39cb5cf, d45a4af)` == Track-B's 8 files. The integrated tree is exactly Track-A ∪ Track-B applied to the common base — no merge-introduced edits.

### Mandate 6 — TinyGo: PASS (by inspection; CI device build is the remaining post-push gate)
- No build tags on any touched production file. No reflection/unsafe/cgo/goroutine/generics introduced. Only new production import: `github.com/btcsuite/btcd/btcec/v2` in `gui/bip85.go` (already in the dep tree; required by the `bip85PkeyHook` signature). Changes are loop reorder, extra int returns, pure helpers, string constants, and `wipe`/`Zero`/`wipeBytes` calls.

## Critical
None.

## Important
None.

## Minor
1. **Brief-phrasing mismatch (not a code defect).** Mandate 1's clause "the only new production symbol is `bip85PkeyHook` (test-only)" is true for Track B; in the integrated tree Track A also adds its planned symbols (`ms1Entropy`, `extractSuppliedMd1AndMk1`, the L2 consts), all sanctioned by Track A's GREEN plan + exec review. No unexpected/cross-track symbol exists.
2. (Inherited, informational) `%G?` would report `N` in this env due to no `gpg.ssh.allowedSignersFile`; the raw SSH-signature blocks are physically present on all 12 commits.

## Verified-correct list
- 17-file combined diff = exact union of the two tracks; intersection empty; no file double-touched.
- No merge-conflict markers; `gui/multisig.go` and `gui/slip39_polish.go` untouched.
- All 8 findings (H1/H2/M1/L2/L1×3/M2/M3/M4) present and correct in the merged source.
- 4 negative controls reproduce in the merged tree identically to the per-track reviews; H1 additionally compiler-protected at the call site.
- `go build`/`go test ./...` (0 FAIL)/`go vet` all green together; zero new vet findings vs cold base.
- 12/12 commits signed + DCO + co-authored + author Brian Goss; both merges `--no-ff`; no evil-merge content.
- TinyGo-safe by inspection.

## Bottom line
The merge is a faithful, drift-free integration of two file-disjoint tracks. Nothing regressed across the combine: both tracks' tests pass together, all 8 findings are closed in the integrated tree, every masking-proof holds (H1 is even compiler-enforced), and merge/commit hygiene is fully satisfied with no evil-merge content. The only open item is the cosmetic brief-phrasing note (Minor 1), which is not a code issue.

**RECOMMENDATION: PUSH.** Remaining post-push gate: the TinyGo device build in CI (not a pre-push blocker).

**Fork state on exit:** `main @ 39cb5cf6351f6c87ab779e5688a60186f65412e0`, working tree clean. Throwaway worktrees (`/tmp/exec-combined`, `/tmp/exec-base`) removed + pruned. The three pre-existing worktrees (`seedhammer-wt-bip39`, `/tmp/scrub-batch-wt`, `/tmp/seedhammer-verify-cluster`) left untouched.
