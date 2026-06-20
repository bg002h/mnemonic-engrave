# sh(wpkh) plan — R0 review (round 0) — VERBATIM agent report

**Agent:** `a46fd1bd0a09480fb` (adversarial opus architect; applied the full two-touch-point fix in a throwaway worktree + RAN the BIP-49 golden / no-collision / A6-coupling / no-regression probes). **Fork HEAD:** `65a1018`. **Plan commit:** `933c822`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I). 2 non-blocking Minors. Cleared for single-implementer TDD.

---

# sh(wpkh) plan — R0 review (round 0)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 65a1018  **Plan commit:** 933c822  **Verdict:** GREEN (0C/0I)

## BIP-49 golden re-verification (MANDATE #1) — RAN
RAN in an isolated detached worktree off `65a1018`. Applied the plan's BOTH touch-points verbatim (md.go: `InnerWpkh` field + `classifyPolicy sh(wpkh)→PolicySingle` arm + `innerWpkhNesting` helper + `summarize` set; md1_expand.go: `ScriptSh+PolicySingle+InnerWpkh→P2SH_P2WPKH` arm) and drove the REAL projection path `EncodeSingleSig(ScriptShWpkh) → ExpandWalletPolicyChunks → expandedToDescriptor → address.Receive/Change`. Probe output (abandon seed, `m/49'/0'/0'`):
```
account xpub = xpub6C6nQwHaWbSrzs5tZ1q7m5R9cPK9eYpNMFesiXsYrgc1P8bvLLAet9JfHjYXKjToD8cBRswJXXbbFpXgwsswVPAZzKMa1jUp2kVkGVUaJa7   ✓
masterFP     = 0x73c5da0a   ✓
len(strs)    = 3            ✓ (chunked, VF3)
tpl          = {Root:2(ScriptSh) Policy:0(PolicySingle) Renderable:true InnerWpkh:true InnerWsh:false}   ✓ (real summarize/classifyPolicy path, NOT a bypass)
receive[0]   = 37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf   ✓ MATCH
change[0]    = 34K56kSjgUCUSD8GTtuF7c9Zzwokbs6uZ7   ✓ MATCH
```
Every load-bearing value MATCHES the plan's Task-3 golden byte-exact. No fund-loss-class divergence.

## No-collision verdict (MANDATE #2) — RAN, PASS
Both touch-points confirmed present and correct (Task 1 decoder + Task 2 projection). RAN the no-collision probe with the SAME key material across the three `sh` shapes:
```
P2SH_P2WPKH = 3KNzwtfNVKGJaiGGgJYeSUvcRV4swBGzJE
P2SH(bare)  = 39fiayD2eNRLSVCwvzuyMcxNFABRHfNSU3
P2SH_P2WSH  = 35tek545ZwPexwHBCGtgGkgoKSZnyRw7kd
```
Pairwise-distinct. The new arm keys on `case md.PolicySingle:` (md1_expand.go:87); every multisig `sh` arm lives under `case md.PolicySortedMulti:` (md1_expand.go:98). `PolicySingle(0) ≠ PolicySortedMulti(2)` → disjoint switch cases, impossible to cross-map. `TestExpandedToDescriptorShNesting` (md1_expand_test.go:124-149, P2SH↔P2SH-P2WSH pin) RAN green UNCHANGED. A bare `sh` under `PolicySingle` with `InnerWpkh==false` falls through to `expandUnsupported`/nil — RAN, confirmed (safe display-only fallback, I3).

## A6 fuzz two-halves (MANDATE #3) — RAN, both halves confirmed coupled
- Applied BOTH halves (synthesize `InnerWpkh = at(5)&8` + thread into `Template`; add `ScriptSh` to `isBip380ExpressibleShape`'s `PolicySingle` arm) + the seed entry `{2,0,1,1,1,10}`. Seed corpus PASS (5/5); 20s active fuzz PASS (45,818 execs, 0 crashes, 0 invariant trips, no testdata/fuzz file written).
- Proved the coupling empirically: with the `InnerWpkh` half but WITHOUT the `isBip380ExpressibleShape` half, seed#3 trips line-74 `expandOK for non-bip380 shape root=2 policy=0` — exactly VF6. The plan mandates BOTH (Task 4 Step 1 (a)+(b)); correct.
- The unconditional `ScriptSh` in the expressible arm is SOUND: when `InnerWpkh==false`, `scriptForTemplate` returns `!ok`→`expandUnsupported` (not `expandOK`), so `expandOK ⇒ expressible` is never violated; 20s fuzz confirms.
- R3 stale-comment cleanup applied + grep gate RAN: `BUILD OK / NOTE GONE / STALE-CLAIM GONE / NO PRIVATE MATERIAL`. The plan's quoted comment text (md.go:1173-1178, singlesig_restore.go:25-29) matches source verbatim.
- No-regression RAN: `wpkh→bc1q…` PASS; `pkh`/`tr` PASS; `sh-wsh`/`bare-sh` unchanged; BIP-84 golden (`TestSingleSigRestoreWpkhKnownAddress`) PASS; restore `sh-wpkh` PASS; alloc gate (`TestAllocs`) PASS; full sweep `./md ./gui ./address ./bip380` green; `go vet` rc=0; `go build` rc=0.

## Critical
None.
## Important
None.
## Minor
- **m1 (benign, plan already flags).** Task 0 branches off `8eb51d7`; current `main` is `65a1018`. Verified `git diff 8eb51d7..65a1018` EMPTY (CI-trigger commit), so all `8eb51d7`-pinned line numbers valid and the tree is identical. Branching off current `main` is marginally cleaner (no stale base); benign. Not blocking.
- **m2 (cosmetic).** `go vet` emits one pre-existing note on `gui/op/draw_test.go:176` (`testing.ArtifactDir requires go1.26`); confirmed identical on the pristine tree (rc=0), unrelated. The plan's Task 5 "no output" expectation is slightly optimistic — treat rc=0 as the gate, not literal empty output. Not blocking.

## Verified-correct
- Both touch-points present & correct; mirror the `InnerWsh` precedent exactly (md.go:1218/1322/1355 field/helper/summarize; consumed md1_expand.go).
- Internal symbols the appended `md/md_test.go` references all exist: `tagWpkh`(0x00), `tagWsh`(0x02), `tagSortedMulti`(0x07), `keyArgBody{index uint8}`, `childrenBody{children []node}`, `multiKeysBody{k uint8, indices []uint8}`, `node{tag, body}`.
- `md/md_test.go` EXISTS (`package md`) → Task 1 correctly APPENDS, does not "create".
- Helper signatures (VF8) verified: `EncodeSingleSig`(encode_singlesig.go:36), `ExpandWalletPolicyChunks`(expand.go:102), `deriveAccountXpub`(derive.go:19), `decodeXpubBytes`(singlesig_derive.go:99), `originComponents`(singlesig_derive.go:128), `singleSigPath`(singlesig_pick.go:81), `abandonAboutMnemonic`(derive_test.go:13), `address.Receive/Change/Supported/Find`(address.go:24/20/28/53). `address.Find` 4-return `(chain,index,found,err)` matches Task 2 usage — RAN, `found=true`.
- R1 fixture is non-bypass: `ExpandWalletPolicyChunks` routes through the changed `summarize`/`classifyPolicy`/`InnerWpkh` (confirmed by the golden probe's `tpl` assertion).
- Diff scope = exactly `md/md.go`, `gui/md1_expand.go`, `gui/md1_expand_fuzz_test.go`, `gui/singlesig_restore.go` (+ the two test files extended); NOTHING in `address/` or `bip380/`.
- Throwaway worktree removed; pristine fork left clean at `65a1018`.

## Bottom line
**GREEN — 0 Critical / 0 Important.** Re-derived the load-bearing BIP-49 `sh(wpkh)` golden by RUNNING the plan's exact two-touch-point fix through the real `ExpandWalletPolicyChunks` projection path: `receive[0]=37Vuc…` / `change[0]=34K56…` / acct-xpub / masterFP `0x73c5da0a` all byte-exact. No P2SH-P2WPKH↔P2SH-P2WSH collision (disjoint switch, pairwise-distinct). The A6 fuzz two-halves are genuinely coupled and the plan mandates both (half-A6 self-break proved); 20s fuzz clean. No-regression across wpkh/pkh/tr/sh-wsh/bare-sh + BIP-84 golden + alloc gate + vet + build. Plan fully-specified — no placeholders, type/signature-consistent, correct FAIL→PASS, correct APPEND-vs-create. Two non-blocking Minors. Cleared for implementation (single subagent, TDD, worktree).
