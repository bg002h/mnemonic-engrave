# sh(wpkh) verify-projection SPEC — R0 review (round 0) — VERBATIM agent report

**Agent:** `a410cc12946e5bb7d` (adversarial opus architect; RAN classify-premise + BIP-49 golden + no-collision probes). **Fork HEAD:** `8eb51d7`. **Spec commit:** `ede563a`. **Date:** 2026-06-19.
**Verdict:** GREEN (0C/0I). 2 non-blocking plan-stage Minors. Cleared for the implementation plan.

---

# sh(wpkh) verify-projection SPEC — R0 review (round 0)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 8eb51d7  **Spec commit:** ede563a  **Verdict:** GREEN (0C / 0I)

## Address-golden + classify-premise probe (MANDATE #1) — RAN

**Premise TRUE — `sh(wpkh)` is non-renderable today.** Ran `classifyPolicy` on the canonical `node{tagSh,[node{tagWpkh,keyArgBody{0}}]}` tree (F4 shape) at HEAD `8eb51d7`:
```
classifyPolicy(sh(wpkh))         => policy=5 k=0 m=0   (PolicyComplex=5, PolicySingle=0)
innerWshNesting(sh(wpkh))        => false
classifyPolicy(sh(wsh(sortedmulti))) => policy=2 k=1 m=1   (PolicySortedMulti=2)
innerWshNesting(sh(wsh(sortedmulti))) => true
```
So `sh(wpkh)` → `PolicyComplex` → `Renderable=false` (summarize, `md.go:1336`) → routes to display-only (`md1_gather.go:206-208`). Touch-point #1 (the `classifyPolicy` arm) is genuinely needed. The spec's F3 is exact: `classifyPolicy`'s `tagSh` case (`md/md.go:1285-1300`) handles only `sh(wsh(multi/sortedmulti))` and bare `sh(multi/sortedmulti)`; no inner-`tagWpkh` arm.

**Golden MATCHES byte-exact.** Derived the abandon seed at `m/49'/0'/0'` through the real `address.Receive/Change` path:
```
account xpub = xpub6C6nQwHaWbSrzs5tZ1q7m5R9cPK9eYpNMFesiXsYrgc1P8bvLLAet9JfHjYXKjToD8cBRswJXXbbFpXgwsswVPAZzKMa1jUp2kVkGVUaJa7
receive[0]   = 37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf   ✓ (matches spec golden)
change[0]    = 34K56kSjgUCUSD8GTtuF7c9Zzwokbs6uZ7   ✓ (matches spec golden)
```

**`address` derives P2SH-P2WPKH with NO new code.** `address/address.go:144-146` (`P2WPKH, P2SH_P2WPKH` → witness-pubkey-hash) and `:160-170` (wraps `P2SH_P2WPKH`/`P2SH_P2WSH` via `PayToAddrScript`→`NewAddressScriptHash`→`3…`). `bip380.P2SH_P2WPKH` exists (`bip380/bip380.go:62`), in `Singlesig()` (`:116-117`). `address.Supported` = `Receive(desc,0)` succeeds → verify lights up (F6) with no GUI change. **`InnerWpkh` does not yet exist** (grep empty); the `InnerWsh` precedent it mirrors is exact (`md.go:1211-1218,1322-1331,1355`; consumed `md1_expand.go:106-109`).

## No-collision verdict (MANDATE #2)
**PASS — no P2SH-P2WPKH ↔ P2SH-P2WSH collision.** Same key material through `address.Receive`:
```
P2SH_P2WPKH = 37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf
P2SH_P2WSH  = 3Kdr7CoTcx8UaGuzD7aqQxXi1dxUmBdph2
P2SH(bare)  = 39kh1g5VzX7eEEzAnbNZsG2w1WCYNQVu3G
```
All three pairwise-distinct. The `scriptForTemplate` switch (`md1_expand.go:86-111`) is disjoint: the new arm keys on `case md.PolicySingle:` (line 87); every multisig `sh` arm lives under `case md.PolicySortedMulti:` (line 98). `PolicySingle(0) ≠ PolicySortedMulti(2)`, so an `sh(wpkh)` can never reach a P2SH-P2WSH/bare-P2SH arm, and vice-versa. The regression pin is real: `gui/md1_expand_test.go:124-149` (`TestExpandedToDescriptorShNesting`) asserts `sh(wsh(sortedmulti))→P2SH_P2WSH ≠ sh(sortedmulti)→P2SH` and stays unchanged.

## Critical
None.
## Important
None.
## Minor
- **m1 (plan-stage, not blocking).** A6 (`FuzzExpandedToDescriptor`): `isBip380ExpressibleShape`'s `PolicySingle` arm (`md1_expand_fuzz_test.go:18-19`) omits `ScriptSh`, and `root := …%5` (line 43) draws 0..4 — without the update the new `expandOK` trips the line-74 invariant. Mechanical detail the plan should pin: the new arm keys on `InnerWpkh`, but the harness builds `md.Template` with only `InnerWsh` (`at(5)` bits 1/2/4 spent on innerWsh/xpubPresent/wildcardHardened); the harness must also synthesize `InnerWpkh` (a free bit, e.g. `at(5)&8`) or the new arm is never fuzzed.
- **m2.** F8 cites `gui/derive_test.go:26` `knownAccountXpub84` correctly; the spec's golden table correctly derives a *fresh* BIP-49 account xpub (not reusing BIP-84). No defect.

## Ambiguity adjudication (R1 test fixture)
**Deferring the fixture-construction method to the plan is ACCEPTABLE.** Confirmed empirically: `EncodeSingleSig(ScriptShWpkh)` → 3 chunk strings; single-string `Decode(chunk)` → `ErrChunkedUnsupported` (`md.go:1229-1230`). **Recommended fixture:** (1) BEST: `EncodeSingleSig(real BIP-49 xpub, ScriptShWpkh)` → `md.ExpandWalletPolicyChunks(strs)` — `ExpandWalletPolicyChunks` (`md/expand.go:102-112`) calls `Reassemble` then **`summarize(d)`**, i.e. routes through the SAME `classifyPolicy`/`summarize`/`InnerWpkh` code the fix changes (NOT a bypass). (2) Acceptable: hand-built `md.Template{Root:ScriptSh, Policy:PolicySingle, InnerWpkh:true, Renderable:true}` → `expandedToDescriptor`, paired with a direct `classifyPolicy`/`summarize` unit test on the node. Acceptance gate (A1–A7) + invariants (I1–I5) cover all four mandate risks. IN/OUT coherent: `sh(wpkh)` single-sig only.

## Verified-correct
F1 (`address` P2SH-P2WPKH net-zero), F2 (projection arm deliberately absent `md1_expand.go:96-97`), F3 (decoder non-renderable, ran), F4 (wire shape `Root==ScriptSh`, `ScriptShWpkh` encode-only), F5 (`InnerWsh` precedent, ran), F6 (verify lights up automatically `gui.go:2405-2421` etc.), F7 (`expandedToDescriptor` shape-agnostic, mainnet pin `md1_expand.go:61`), F8 (golden infra). A5 regression refs exact. `go build ./...` rc=0; `go test ./md ./gui ./address ./bip380` green; fork left clean at `8eb51d7`.

## Bottom line
**GREEN — 0 Critical / 0 Important.** The two-touch-point analysis is real and empirically confirmed; the pinned BIP-49 golden is byte-exact through the real `address` path; `address`/`bip380` derive P2SH-P2WPKH with zero new code; no P2SH-P2WPKH↔P2SH-P2WSH collision (disjoint switch, pairwise-distinct addresses). A6 fuzz update + R3 stale-comment cleanup correctly mandated. R1 fixture deferral sound (`ExpandWalletPolicyChunks` routes through `summarize`). The two Minors are plan-stage polish. Spec cleared for the implementation plan / TDD.
