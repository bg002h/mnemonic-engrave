# T6b IMPLEMENTATION — whole-diff exec review (round 0) — VERBATIM agent report

**Agent:** `a091f6a4b8b89cbc8` (adversarial opus architect; RAN the code + a dedicated 6-case ms1 probe + a D14 partial-match probe + fuzz). **Branch:** `feat/t6b-multisig-supplied-md1`. **Base:** `072461a`. **Final feature commit:** `38209c1`. **Date:** 2026-06-19.
**Verdict:** GREEN (0C/0I). 3 non-blocking Minors → FOLLOWUPS. Mandatory post-implementation gate per CLAUDE.md phase (4). T6b cleared for merge.

---

# T6b IMPLEMENTATION — whole-diff exec review (round 0)
**Reviewer:** opus architect (adversarial)  **Branch:** feat/t6b-multisig-supplied-md1  **Base:** 072461a  **Verdict:** GREEN (0C/0I)

## Deviation #1 (verifyMultisig ms1) — independent verdict
RAN a dedicated 6-case probe against the COMMITTED `verifyMultisig` (gui/multisig_verify.go:26-29, which delegates straight to `bundle.Verify` with NO `d.MS1=""` mask) + the native ms1 semantics in bundle/verify.go:74-79. Result — the deviation is CORRECT and strictly SUPERIOR for the multisig path:
- CASE (a) full bundle + correct re-type → both sides carry ms1 → entropy compared → PASS. ✔
- CASE (b) watch-only → `deriveMultisigLeg(..., full=false)` natively yields `reDerived.MS1==""` AND `ms1Readback==""` → both-empty → ms1 leg SKIPPED → PASS. The fix did NOT break watch-only into a spurious FAIL. ✔
- CASE (c1) full derived + EMPTY readback ms1 → `verify: ms1 presence mismatch` → FAIL. This is the exact hole the plan's `d.MS1=""` mask WOULD HAVE OPENED (it would have collapsed a full bundle with a missing ms1 readback into a both-empty skip = silent PASS). The implementer's removal CLOSES it. ✔
- CASE (c2) full derived + WRONG ms1 (valid codex32, different entropy) → `verify: ms1 entropy mismatch` → FAIL. ✔
- CASE (c3) watch-only derived + ms1 typed at readback → presence mismatch → FAIL. ✔
- CASE (c4) wrong md1 → FAIL. ✔

ROOT CAUSE the deviation is right: single-sig (`verifySingleSig`, gui/singlesig_verify.go:49-58) MUST keep the mask because `deriveSingleSigBundle` ALWAYS re-derives a full ms1 (no `full` param), so watch-only needs the mask to reach both-empty. Multisig's `deriveMultisigLeg` takes `full` and is called in `multisigVerifyFlow` (gui/multisig_verify.go:75) with the SAME `full` flag that gates the ms1 readback (gui/multisig_verify.go:83) — derived-side ms1 presence and readback presence are perfectly lockstepped, so the mask is unnecessary AND removing it restores the presence-mismatch guard. NO path where a wrong/missing ms1 verifies as PASS.

## Test/probe results
- Full no-regression suite (`go test -count=1 ./gui/... ./md/... ./bundle/... ./mk/... ./codex32/...`): ALL `ok`, no FAIL. T6a single-sig logic byte-unchanged (the only pre-existing files modified are gui.go lockstep + the 3 nav-tests; every changed gui.go line is a program-const insertion).
- T6b targeted tests + nav-tests + TestAllocs: all PASS (TestFindUserSlot, TestExtractSuppliedMd1, TestAllSlotsHaveXpub, TestSuppliedMultisigFixtureIsFullPolicy, TestDeriveMultisigLeg, TestMultisigEngraveCards, TestVerifyMultisig, TestMultisigRestoreLines, TestEngraveMultisigProgramNavigable, TestEngraveMultisigLeftWrap, TestAllocs).
- Fuzz: FuzzExtractSuppliedMd1 (~1.8M execs/5s) and FuzzFindUserSlot (~68k execs/5s) — 0 crashes.
- D14 partial-match probe (gui/multisig_match.go:48): a slot with the real chain code but a corrupted pubkey byte at index 64 → NO match (full 33-byte `[32:65]` compared, no off-by-one); corrupted chain code + real pubkey → NO match; genuine pair → match; a slot whose OriginPath disagrees with the embedded key's actual origin → NO match (derives at each slot's OWN origin). `bytes.Equal(cc[:], k.Xpub[0:32]) && bytes.Equal(pk[:], k.Xpub[32:65])` — refuses on zero (returns ok=false), skips `XpubPresent==false`, first-by-index + `reused` on ≥2. No non-cosigner reaches an engrave.

## Critical
(none)

## Important
(none)

## Minor (→ FOLLOWUPS, non-blocking)
- gui/multisig.go:91-92: the reused-key notice names only `reused[0]`/`reused[1]`; with ≥3 matched slots the 3rd+ index is not shown (the FIRST-by-index slot is still engraved deterministically and correctly). Cosmetic UX only.
- gui/multisig.go:57 `_ = tpl`: `tpl` is decoded in step 2 then discarded and re-decoded inside `multisigRestoreDocFlow` (gui/multisig_restore.go:58). Harmless redundant decode (public data, display-only); could thread the already-decoded `tpl/keys` through. Polish only.
- gui/multisig_fuzz_test.go:59 `TestMultisigSeedHookSeamExists` only asserts the hook seam compiles; the behavioral scrub-on-exit is structurally guaranteed by the `defer` (gui/multisig.go:69-73, mirrors the proven single-sig pattern) but is not driven headlessly. Matches the single-sig test posture; acceptable.

## Verified-correct
- I-1 (D14, Critical): canonical 32+33-byte `bytes.Equal` compare, per-slot own-origin derive, refuse-on-zero, first-by-index + reused notice. Probe-confirmed no partial/truncated/wrong-origin leak.
- I-2 (Critical): supplied md1 engraved VERBATIM — `deriveMultisigLeg` clones `suppliedMd1` unmodified (gui/multisig_derive.go:60), `multisigEngraveCards` clones again (gui/multisig_engrave.go:31); no re-encode anywhere; device builds no multisig md1.
- I-3 / I-11: full-policy gate (`allSlotsHaveXpub`, empty→refuse, any-missing-xpub→refuse) + single-md1 supply filter (0→refuse, ≥2→refuse, any cardMK1/cardMS1→refuse). Tests + fuzz pass.
- I-4: mk1 stub == `WalletPolicyIDStubChunks(suppliedMd1)`, Path == matched origin, fp == 73c5da0a — golden-asserted.
- I-5: verify reuses `bundle.Verify` UNCHANGED, user-slot-only.
- I-6 (faithful-or-refuse): `multisigRestoreLines` derives addresses ONLY on `expandOK && desc!=nil`; non-bip380/template-only → nil desc → display-only "addresses unavailable", NO `address.*` call (golden + template-only test pass; wrong-address verify structurally impossible per md1_expand.go:42-48).
- I-7 (Critical, security spine): seed typed-only via `seedEntryFlow` (no gui/scan.go scan→derive in any multisig file); `deriveAccountXpub` uses `.Neuter()` + scrubs seed/master/intermediates internally; `m.Entropy()` guarded by `m.Valid()`; entropy `wipeBytes`'d after EncodeMS1; mnemonic scrubbed via `defer` on every exit path in both `engraveMultisigFlow` and `multisigVerifyFlow`; ms1 engraved-only never NFC; grep clean of xprv/PrivKey serialization in new code; restore doc carries no secret.
- I-8: mainnet-only (`&chaincfg.MainNetParams` everywhere).
- I-9 (lockstep): all 8 gui.go sites retargeted to `engraveMultisig`; NO stale `engraveSingleSig` wrap bound remains (the 4 surviving refs are enum const, dispatch case, title case, layoutMainPlates case — all legitimate per-program entries); both wrap bounds now `engraveMultisig`; `qaProgram` stays the highest enum → non-navigable; layoutMainPlates lists the new program (no render panic); 3 nav-tests + new multisig_program_test all PASS; TestAllocs green.
- I-10 (no-regression): full suite green; only gui.go lockstep + 3 nav-tests touch pre-existing files; T4/T5/T6a/codecs byte-unchanged.
- Process: 11 commits, all SSH-signed (gpgsig header present), DCO Signed-off-by + correct author Brian Goss + Co-Authored-By trailer. (`sign=N` in `git log` is only the local allowedSignersFile verification config, not a missing signature.)
- Build clean (`go build ./...` exit 0); `go vet ./gui/...` exit 0 (the lone note is an unrelated go1.26 `testing.ArtifactDir` in a pre-existing op/draw_test.go, not from this diff).

## Bottom line
GREEN (0 Critical / 0 Important). Deviation #1 is correct and CLOSES a silent-pass hole the original plan code would have opened; independently probe-verified across all three cases plus the symmetric one-sided and wrong-ms1/wrong-md1 cases. The D14 cross-match is robust against partial/off-by-one/wrong-origin leaks. Security spine intact (typed-only, neutered, per-leg scrub, no secret leak, verbatim engrave). Lockstep coherent, no regressions, fuzz clean. **T6b is cleared for merge.** Three non-blocking Minors → FOLLOWUPS.
