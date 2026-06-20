# T6c nits cleanup — whole-diff exec review (round 0) — VERBATIM agent report

**Agent:** `a69c0a218e4b4c5f0` (adversarial opus architect; RAN the Back-nav forward-path probes + the wrapper A3). **Branch:** `chore/t6c-nits`. **Base:** `76ffcdf`. **Final commit:** `c449485`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I). 1 trivial comment-only Minor. Cleared for merge.

---

# T6c nits cleanup — whole-diff exec review (round 0)
**Reviewer:** opus architect (adversarial)  **Branch:** chore/t6c-nits  **Base:** 76ffcdf  **Verdict:** GREEN (0C/0I)

## Back-nav refactor (NIT 2) — RAN
Read the committed stage-loop (`gui/multisig_build.go:265-333`, consts `stageTemplate..stageDone` at 267-274). Forward path verified CORRECT and unchanged in output:
- (a) Each stage assigns its param then advances exactly ONE stage (template→N→K→@S→Fp→Done). k bounded 1..n via `multisigKChoices(p.N)`/`multisigKFor` (idx+1), @S bounded 0..n-1 via `multisigSelfSlotChoices(p.N)` (idx IS slot). k/@S `ChoiceScreen`s constructed fresh inside their `case` from the current `p.N`, so re-entry after an upstream n-change re-derives the ranges. No off-by-one, no skipped stage.
- (b) Back from N/K/@S/Fp sets `stage` back exactly one then `continue` — steps back ONE stage.
- (c) Back from `stageTemplate` is the only `return p, false`. Abandon only from the first stage.
- (d) Loop condition `stage != stageDone`; every non-abandon path strictly advances or steps back to a valid stage; the only `return p, true` is after `stage = stageDone` (reached only after Fp assigns `IncludeFp`). No infinite loop, no default-zero param returned with ok==true.

Commands:
- `go test ./gui/ -run TestBuildParamPickBackNav -v` → PASS (back-from-k→n, back-from-n→template, back-from-template→abandon ok==false).
- `go test ./gui/ -run TestMultisigBuildExperimentalWarningAbort -v` → PASS.
- `go test ./gui/ -run TestMultisigFrontDoorRouting -v` → PASS.
- Two throwaway adversarial probes (run, then deleted; tree clean): `ForwardPathExactParams` (full forward pick, NON-DEFAULT at every stage → exact `{ShWsh, n=4, k=3, @S=3, IncludeFp=true}`) PASS; `ChangeNRebounds` (n=5 → Back→n → n=2; k picker clamps at k=2, range re-derived) PASS — no stale-range/`k>n` regression.

## Wrapper A3 (NIT 3) — RAN
`TestAssembleBuildPolicy_T6bWrapperByteMatch` (`gui/multisig_build_test.go:184-258`) genuinely drives the full wrapper: calls `assembleBuildPolicy(p, selfXpub, selfMasterFP, []mk.Card{card0,card2})` (NOT `md.EncodeMultisig` directly). Foreign slots @0/@2 re-serialized from the decoded fixture's `ExpandedKey.Xpub` (`[65]byte`=cc‖pubkey, `md/expand.go:62`) into base58 mainnet xpubs via `hdkeychain.NewExtendedKey`. `cosignerFromCard`→`decodeXpubBytes` recovers ONLY cc+compressed-pubkey (parentFP discarded), so the synthetic depth=4/childNum=0/parentFP=0 round-trip yields the SAME 65 bytes. Result: 6 chunks byte-equal to the on-disk fixture, stub `7b716421`, `stub == WalletPolicyIDStubChunks(out)`, all 3 slots FpPresent=false under omit, self masterFP `73c5da0a`. → PASS. Original direct `TestAssembleBuildPolicy_T6bByteMatch` retained → PASS.

## errmsg (NIT 1)
`errMultisigBadOriginMode` distinct sentinel (`md/encode_multisig.go:79`), returned ONLY from the OriginMode switch default (`:111`). SCRIPT switch default still returns `errMultisigBadScript` (`:200`). `TestEncodeMultisigRefuse` covers BOTH via `errors.Is` (`encode_multisig_test.go:520`). No behavior change to valid inputs. → PASS (7 subtests).

## No-regression + scope
- `go build ./...` rc=0. `go test ./md/... ./gui/...` all ok. `TestAllocs` PASS (1.54s).
- Named flows green: A3 direct, `_T6bWrapperByteMatch`, `_IncludeFpDiffers`, `_NoXprv`, `TestMultisigSelfSlotChoices`, `TestMultisigFpChoices`, sh-wpkh supply/golden/no-collision, T6b byte-exact (md), classify/render.
- `go vet ./gui/ ./md/` rc=0, NO findings (the pre-existing `gui/op/draw_test.go:176` is out of scope/unrelated).
- Scope: `git diff --name-only main..HEAD` = exactly 5 files (`md/encode_multisig.go`, `md/encode_multisig_test.go`, `gui/multisig_build.go`, `gui/multisig_build_flow_test.go`, `gui/multisig_build_test.go`). Assembler logic, user-decision encoding, warning, security spine untouched.

## Critical / Important
None / None.
## Minor
- (polish, non-blocking) The wrapper test's `card0`/`card2` carry inert `Stubs: [][4]byte{{0,0,0,0}}` + `Fingerprint:""` (unread under fp-omit) — a one-word "unused under omit" comment could preempt confusion. No action required.

## Bottom line
GREEN (0C/0I). The NIT-2 stage-loop refactor preserves the forward-path param set exactly (committed Back-nav test + two adversarial probes), Back steps back exactly one stage, Back-from-template is the sole abandon, no empty/default param reaches `assembleBuildPolicy`. The NIT-3 wrapper test genuinely drives `assembleBuildPolicy` for all slots and byte-matches the T6b fixture + stub `7b716421`; the direct A3 test is retained. NIT-1 adds a distinct, correctly-scoped error. Build/vet/full suites/TestAllocs green; scope exactly the 5 files. Cleared for merge.
