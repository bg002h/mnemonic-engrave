# T7b IMPLEMENTATION — whole-diff exec review (round 0) — VERBATIM agent report

**Agent:** `a8f6a7b032fad1cfa` (adversarial opus architect; RAN an independent golden re-derivation + a scrub-test non-vacuity probe). **Branch:** `feat/t7b-bip85-derive`. **Base:** `82d46b3`. **Final feature commit:** `3b10ce4`. **Date:** 2026-06-19.
**Verdict:** GREEN (0C/0I). 1 non-blocking Minor → FOLLOWUPS. Mandatory post-implementation gate per CLAUDE.md phase (4). T7b cleared for merge.

---

# T7b IMPLEMENTATION — whole-diff exec review (round 0)
**Reviewer:** opus architect (adversarial)  **Branch:** feat/t7b-bip85-derive  **Base:** 82d46b3  **Verdict:** GREEN (0C/0I)

## Derive re-run (MANDATE #1) — RAN it
Wrote a throwaway `gui/zz_review_throwaway_test.go` that drove the COMMITTED `deriveBip85Child` against every golden AND an *independent* in-test reference derive (separate `hdkeychain` walk + `bip85.Entropy` + `bip39.New`) + an independent fp recompute (`MnemonicSeed`→`NewMaster`→`ECPubKey`→`bip32.Fingerprint`). All MATCH (committed == reference == pinned want):
- abandon `m/…/12'/0'` → `prosper short ramp…fold`, ent `ac98dac5…`, child fp **02e8bff2** ✓
- abandon `18'/0'` → `winter brother…fox`, ent `fc039f51…`, fp **3bb5fd0c** ✓
- abandon `24'/0'` → `stick exact…jaguar`, ent `d5a9cb46…`, fp **c2f2dd51** ✓
- abandon `12'/1'` → `sing slogan bar…desert` ✓; `12'/9'` → `earth ice square…bubble` ✓
- canonical master (`install scatter…usage`) `12'/0'` → `girl mad pet…nose`, ent **6250b68daf746d12a24d58b4787a714b** (byte-identical to BIP-85 spec §5.1) ✓
- master fps: abandon **73c5da0a** ✓, canonical **627ef3a6** ✓

Path is fully hardened (`{PathRoot, 39+h, 0+h, words+h, index+h}`, gui/bip85.go:49-55), `ECPrivKey()` error propagated (`:67-71`, never `.Serialize()`s nil), entropy = **leading** `hmacOut[:entLen]` (`:84`). Throwaway removed; worktree pristine (`git status` clean).

## Scrub non-vacuity probe (MANDATE #3)
Temporarily neutered the top-level scrub defer in a copy of `gui/bip85.go` (restored after):
- Both scrubs disabled → `TestBip85DeriveFlow_ScrubsBothMnemonics` **FAILS**: `master[11] = 3, not scrubbed on exit (I-3)`.
- Master-only re-enabled, child disabled → **FAILS**: `child[0] = 1380, not scrubbed on exit`.

The test is **NON-vacuous on both branches** and exercises the SUCCESS path (engrave completes → top-level defer, not an in-loop abort scrub). File byte-restored (`git diff HEAD` empty). All 7 exit paths traced: derive-error/picker-cancel/warning-abort/engrave-build-error/engrave-backout all set `child=nil` after an in-loop scrub; success `return` is covered by the single top-level LIFO defer; privkey+HMAC+seed wiped by 3 `defer wipeBytes` inside `deriveBip85Child`. `bip85SeedHook` is SYNCHRONOUS (`:230-232`, not deferred); exactly ONE top-level scrub defer. Master is typed-only — `seedEntryFlow` (derive_xpub.go:82) uses `inputWordsFlow` only, no scan/NFC branch.

## Test/probe results
- Full suite `go test ./gui/... ./bip85/... ./bip39/... ./backup/... ./codex32/... ./mk/... ./md/...` → all **ok**, no FAIL.
- `go test ./gui/ -count=1` (uncached) → **ok 11.247s**.
- `TestAllocs` → **PASS**.
- All committed T7b tests PASS, incl. `TestBip85DeriveFlow_ScrubsBothMnemonics` (3.79s).
- Nav family (Bip85Derive/Multisig/SingleSig/Bundle/Xpub Program+LeftWrap) → all **PASS**.
- `FuzzDeriveBip85Child` seed corpus PASS; 15s fuzz = ~2.08M execs, **0 crashes**, no `testdata/fuzz` artifact.
- xprv-grep on `gui/bip85.go` → **CLEAN** (no `.String()`/Neuter/xprv/tprv/xpub/NewExtendedKey); artifact is words+SeedQR via `engraveSeed`/`backup.EngraveSeed` only.
- `go vet ./gui/` clean; `go build ./...` OK.

## Critical
None.

## Important
None.

## Minor (→ FOLLOWUPS, non-blocking)
- The dispatch `case bip85Derive:` was placed after `case engraveMultisig:` (gui.go:1509-1511) rather than the plan's "just before `case backupWallet:`". Functionally identical — Go switch cases are unordered, no fallthrough, each `continue`s. No action needed; noting only as a documented divergence from the plan's literal placement.

## Verified-correct
- **I-1 derive:** byte-identical to biptool/canonical BIP-85 across all 6 vectors + 2 master fps (independent re-derive RAN). Fully-hardened path; leading-bytes truncation; `entLen` guard keeps `bip39.New` un-panicable.
- **I-A child fp on plate:** `engraveBip85Child` (`:94-104`) computes `masterFingerprintFor(child, MainNet, "")` (err propagated) and passes the SAME child + mfp to `engraveSeed`, which stamps `MasterFingerprint: mfp` (gui.go:475). Test asserts `gotFP==wantChildFP`, `gotFP!=masterFP`, pinned `0x02e8bff2` — and `02e8bff2 != 73c5da0a` confirmed distinct (non-vacuous). `backupWalletFlow` never called (passphrase-fp picker bypassed; only a comment mentions it).
- **I-2 picker bounds:** `bip85WordChoices=[12 18 24]`, `bip85IndexChoices=[0..9]`, app fixed BIP-39, no free-form entry; `TestBip85ParamBounds` pins them and derives a valid child for all 30 pairs.
- **I-3 secrets:** two-secret scrub proven non-vacuous; typed-only master; deterministic.
- **I-4 channel:** child engraved steel-only via `engraveSeed`; xprv-grep clean.
- **I-7 lockstep:** all 8 sites repointed to `bip85Derive` (enum, dispatch, left/right wrap, title `"BIP-85 Child Seed"`, npage, layoutMainPlates case + `panic("invalid page")` default, npages); 2 new nav-tests + 2 repointed prior nav-tests + 2 comment-only updates all pass; `qaProgram` stays the last non-navigable sentinel (wrap bound is `bip85Derive`; qaProgram reached only via the unchanged `FOREVERLAURA!` debug path).
- **I-8 no-regression:** diff touches exactly `gui/bip85*.go` (3 new) + 8 gui.go lockstep lines + 4 nav-test repoints; no Backup-Wallet/T4/T6/codec source changed; full suite green.

## Bottom line
**GREEN — 0 Critical, 0 Important.** Every BIP-85 golden child and fingerprint is byte-identical to biptool/canonical BIP-85 under an *independent* re-derivation I ran; the child's own bare fp (never the master's) is what lands on the plate; the two-secret scrub test is provably non-vacuous on both the master and child branches and exercises the success path; the child is engraved steel-only with no extended-key in the artifact; all 8 lockstep sites are coherent with `qaProgram` still non-navigable; the full suite + `TestAllocs` + a 2M-exec fuzz are green with no regression to the prior flows or codecs. The one Minor (dispatch case placement) is a functionally-inert divergence from the plan's literal wording. **This clears T7b for merge.**
