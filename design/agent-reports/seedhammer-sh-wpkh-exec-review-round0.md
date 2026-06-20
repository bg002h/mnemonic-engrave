# sh(wpkh) projection — whole-diff exec review (round 0) — VERBATIM agent report

**Agent:** `a959e45485908cbf3` (adversarial opus architect; independently re-derived the BIP-49 golden + reproduced/restored the half-A6 self-break + 7.1M-exec fuzz). **Branch:** `feat/sh-wpkh-verify-projection`. **Base:** `65a1018`. **Final commit:** `8c7dfb5`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I). 1 cosmetic Minor → FOLLOWUPS. Mandatory post-implementation gate per CLAUDE.md phase (4). Cleared for merge.

---

# sh(wpkh) projection — whole-diff exec review (round 0)
**Reviewer:** opus architect (adversarial)  **Branch:** feat/sh-wpkh-verify-projection  **Base:** 65a1018  **Verdict:** GREEN (0C/0I)

## BIP-49 golden re-run (MANDATE #1) — RAN
Re-derived INDEPENDENTLY through the REAL path (`deriveAccountXpub → EncodeSingleSig(ScriptShWpkh) → ExpandWalletPolicyChunks → expandedToDescriptor → address.Receive/Change`), abandon seed @ `m/49'/0'/0'` (own probe, then committed `TestShWpkhGoldenAddress` at `-count=1`):
```
acct xpub = xpub6C6nQwHaWbSrzs5tZ1q7m5R9cPK9eYpNMFesiXsYrgc1P8bvLLAet9JfHjYXKjToD8cBRswJXXbbFpXgwsswVPAZzKMa1jUp2kVkGVUaJa7   ✓
masterFP  = 0x73c5da0a   ✓     len(strs) = 3 (chunked)   ✓
tpl = {Root:2(ScriptSh) Policy:0(PolicySingle) Renderable:true InnerWpkh:true InnerWsh:false}   ✓ (REAL summarize/classifyPolicy, NOT a stub)
recv[0]=37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf   ✓ MATCH      change[0]=34K56kSjgUCUSD8GTtuF7c9Zzwokbs6uZ7   ✓ MATCH
```
`37Vuc…` matches the widely-published BIP-49 abandon-seed receive#0 reference. `singleSigPath(49)={49',0',0'}` — correct account path, no off-by-one. `--- PASS: TestShWpkhGoldenAddress`. No fund-loss-class divergence. (`md/md.go:1296-1298`, `gui/md1_expand.go:95-102`, commit `36bf818`/`b5741c5`.)

## No-collision (MANDATE #2) — RAN
New arm keys on `case md.PolicySingle:` (`gui/md1_expand.go:87`); all multisig sh arms key on `case md.PolicySortedMulti:` (`:104`). `PolicySingle(0) ≠ PolicySortedMulti(2)` → disjoint switch cases, structurally impossible to cross-map. SAME key material, `address.Receive[0]` for all three sh shapes (committed `TestShWpkhNoCollision`, RAN PASS): `dWpkh.Script==P2SH_P2WPKH`, `dBare.Script==P2SH`, `dNested.Script==P2SH_P2WSH`, addresses pairwise-distinct. `address/` derives P2SH-P2WPKH unchanged (`address.go:144-146` pubkey-hash → `:160-166` `PayToAddrScript`→`NewAddressScriptHash`→`3…`). `TestExpandedToDescriptorShNesting` UNCHANGED (diff shows no edit) and green.

## A6 fuzz both-halves (MANDATE #3)
BOTH halves present: (a) `innerWpkh := at(5)&8 == 8` synthesized + threaded into `Template` (`md1_expand_fuzz_test.go:52,56`); (b) `ScriptSh` added to `isBip380ExpressibleShape`'s `PolicySingle` arm (`:19`). Seed corpus 5/5 PASS (incl. the new `{2,0,1,1,1,10}` entry); 20s active fuzz = 7,116,269 execs, 0 new interesting, 0 crashers, no `testdata/fuzz/` written. Half-A6 self-break PROVEN: reverting only the `isBip380ExpressibleShape` half (keeping the InnerWpkh synth) trips `md1_expand_fuzz_test.go:76 expandOK for non-bip380 shape root=2 policy=0` at seed#3 — coupling genuine, both landed; restored byte-identical.

## Test/regression results
Fresh `-count=1` sweep `./md/... ./gui/... ./address/... ./bip380/...` → all `ok`, rc=0. Regression goldens PASS: `wpkh`/`pkh`/`tr` singlesig (`TestExpandedToDescriptorSinglesig`), BIP-84 `TestSingleSigRestoreWpkhKnownAddress`, `TestSingleSigRestoreDescriptorScripts/{sh-wpkh,wpkh,pkh,tr}`, wsh-sortedmulti round-trip, `TestExpandedToDescriptorShNesting`. `TestAllocs` PASS (1.55s) — projection is pure switch logic, no new heap on the verify hot path. Display-only fallback verified by probe: bare-sh `PolicySingle` `InnerWpkh==false` → `expandUnsupported`/nil (I3 safe). Mainnet pinned (`md1_expand.go:61`).

## Critical / Important
None / None.

## Minor
- (→FOLLOWUPS, cosmetic) The fuzz invariant `isBip380ExpressibleShape(root, policy, renderable)` (`md1_expand_fuzz_test.go:75`) deliberately ignores `innerWpkh` and reports `ScriptSh+PolicySingle` as expressible regardless. SOUND because the only obligation is `expandOK ⇒ expressible`, and for `InnerWpkh==false` the projection returns `!ok`→`expandUnsupported` (never `expandOK`) — proven by the 20s fuzz with both bit values exercised. The harness is intentionally over-permissive on the expressible side; the comment at `:10-12` is slightly understated (lists `innerWsh` as an input it no longer takes). Non-blocking polish.

## Verified-correct
- Decoder arm (`md/md.go:1295-1300`) requires `inner.tag==tagWpkh` AND `inner.body` is `keyArgBody`; placed before the `sh(wsh)`/`sh(multi)` arms; mutually-exclusive inner tags ⇒ no shadowing. `TestClassifyPolicyShWpkhRenders`/`TestInnerWpkhNesting` RAN green.
- `innerWpkhNesting` (`:1350-1362`) symmetric with `innerWshNesting`; false for bare `wpkh`, `sh(wsh(...))`, any non-sh root. `summarize` sets `InnerWpkh` on the root tree, independent of `InnerWsh` — discriminant on the correct node.
- Diff touches EXACTLY the 6 contracted files; nothing in `address/`/`bip380/`. Secret grep clean. R3 stale-comment cleanup landed.
- `go vet`: ONLY finding is the pre-existing `gui/op/draw_test.go:176` go1.26 note — file UNTOUCHED, byte-identical to base (rc=1 on pristine base too). No NEW finding.
- `go build ./...` clean (go1.26.4).

## Bottom line
GREEN — 0 Critical / 0 Important. RAN the load-bearing BIP-49 `sh(wpkh)` golden by independently driving the real `ExpandWalletPolicyChunks` projection: `receive[0]=37Vuc…` / `change[0]=34K56…` / acct-xpub / masterFP `0x73c5da0a` all byte-exact. No P2SH-P2WPKH↔P2SH-P2WSH collision (disjoint switch, pairwise-distinct). A6 fuzz both-halves coupled (self-break reproduced + restored); 20s/7.1M-exec fuzz clean. No regression across wpkh/pkh/tr/sh-wsh/bare-sh + BIP-84 golden + alloc gate + vet + build. Decoder arm additive, body-guarded, cannot mis-tag a non-sh(wpkh) shape; display-only fallback preserved. One non-blocking cosmetic Minor (→FOLLOWUPS). **Cleared for merge.**
