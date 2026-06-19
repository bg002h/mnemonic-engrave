# Plan R0 Gate RE-REVIEW (R1, post-fold) — T6a-2 (GUI) single-sig flagship on-device flow

**Reviewer:** opus architect (adversarial plan R0 gate re-review, round R1, after the R0 fold)
**Date:** 2026-06-19
**Plan under review (folded):** `design/IMPLEMENTATION_PLAN_seedhammer_T6a2_gui.md`
**Prior R0 review (NOT GREEN, 1C/2I/5m):** `design/agent-reports/seedhammer-T6a2-gui-plan-review-R0.md`
**Recon (Topic 6 corrected):** `design/agent-reports/seedhammer-T6a2-gui-recon.md`
**Spec (GREEN, Phase B):** `design/SPEC_seedhammer_T6a_singlesig_flagship.md`
**Fork verified @:** `bfff857` (the working fork checkout `/scratch/code/shibboleth/seedhammer`, `git rev-parse HEAD = bfff857c5a30ff82fa5eddbd52452cf70e9116de`; all cited files present at this HEAD). The `third_party/seedhammer` submodule is pinned to upstream `713aee2` and does NOT contain the fork surface — verification used the fork checkout directly (see Process note).

---

## VERDICT: GREEN

**0 Critical · 0 Important · 0 Minor (blocking).**

All three blocking findings from R0 (C1, I1, I2) are CLOSED, and all five minors (m1–m4 + the Task-0 fork-fetch prerequisite) are CLOSED, verified against `bfff857`. The fold introduced no drift: every R0-confirmed-sound part (enum lockstep, bound-stub wiring, cardMS1 append safety, watch-only Verify extension, sh-wpkh direct descriptor, the security spine) still holds at source. The parentFP threading (T3 → T7 → T6) is internally consistent across the three tasks and the orchestrator passes it explicitly. The plan is **cleared for single-implementer TDD** in the worktree.

One non-blocking observation (a stale leftover in the *recon's* "REUSE as-is" one-liner, OBS-1 below) does not affect the gated artifact (the plan) — the plan's Task 5 and the recon's authoritative Topic 6 are both correct and unambiguous. Noted for hygiene only; not a finding.

---

## Status of each R0 finding

### C1 — read-back API (was CRITICAL) → **CLOSED**

R0 found Task 5 named a non-existent API shape: `mk1GatherFlow`/`md1GatherFlow` → `.collected()` to assemble the readback `bundle.Bundle`, which is impossible because those flows do not yield `[]string`.

**Fold verified in the plan:** Task 5 Step 1 (line 87) + Step 3 (line 89) now read back mk1/md1 **via the T5 `bundleGatherer`** (`gui/bundle.go`, csid-keyed, yields `bundleCard.strings` for both kinds, handles first/next-chunk priming, refuses ms1), pulling `cardMK1.strings`/`cardMD1.strings`, ms1 hand-typed (`inputCodex32Flow` → `codex32.DecodeMS1`), then assembling `bundle.Bundle{MS1, MK1, MD1}` → `bundle.Verify`. The plan EXPLICITLY forbids `mk1GatherFlow`/`md1GatherFlow`/`.collected()` and states the correct return types inline ("`mk1GatherFlow` returns a decoded `mk.Card`… `md1GatherFlow` returns only `bool`… `.collected()` is private to the gatherer").

**Source re-confirmation @ bfff857:**
- `gui/mk1_inspect.go:156` — `func mk1GatherFlow(ctx *Context, th *Colors, first string) (mk.Card, bool)` — returns a decoded `mk.Card`, NOT `[]string`. ✓ (still true)
- `gui/md1_gather.go:72` — `func md1GatherFlow(ctx *Context, th *Colors, first string) bool` — returns only `bool`; on completion it calls `gatheredDescriptorFlow(ctx, th, g.collected())` internally (`md1_gather.go:76`, `:140`) and discards the strings. ✓ (still true)
- `.collected()` is a method on the PRIVATE gatherers — `func (g *mk1Gatherer) collected() []string` (`gui/mk1_inspect.go:77`); never returned by the `*GatherFlow` wrappers. ✓
- The C1 fix-(a) target is real and yields verbatim `[]string`: `bundleGatherer.offerChunkedMK1` appends `bundleCard{kind: cardMK1, strings: collected, …}` where `collected := sub.collected()` (`gui/bundle.go:193,201-206`); `offerChunkedMD1` appends `bundleCard{kind: cardMD1, strings: collected, …}` (`gui/bundle.go:233,239-244`). `bundleCard.strings []string` is the verbatim chunk strings (`gui/bundle.go:32-35`). ✓
- The assembled-bundle contract holds: `bundle.Verify(derived, readback Bundle)` (`bundle/verify.go:32`) consumes `Bundle{MS1 string; MK1 []string; MD1 []string}` (`bundle/verify.go:19-23`) — exactly what `cardMK1.strings`/`cardMD1.strings` + the typed ms1 produce. ✓
- First/next-chunk priming is accounted for: the plan's Task 5 description names "handles the first/next-chunk priming" and the `bundleGatherer` is csid-keyed (it owns the chunk-set first/next plumbing internally), so the implementer does not need the `first`-argument dance that the bare `*GatherFlow` wrappers required. ✓

**Recon Topic 6 corrected:** `seedhammer-T6a2-gui-recon.md` Topic 6 (lines 31-32) now reads "**Read-back gather — CORRECTED (plan-R0 C1):** … must use the T5 `bundleGatherer` … yields `bundleCard.strings []string` for both kinds … **Do NOT use `mk1GatherFlow`/`md1GatherFlow`/`.collected()`** …" with the correct return types and source lines, and ends "(The earlier shorthand here was wrong.)" The wrong "→ `.collected()`" shorthand is gone from Topic 6. ✓

**Ruling: C1 CLOSED.**

### I1 — parentFP (was IMPORTANT) → **CLOSED**

R0 found Task 6 built the restore-doc `bip380.Key` with `ParentFingerprint` unspecified/`0`, producing a non-canonical xpub (correct addresses, but the descriptor string would not byte-match the engraved mk1 xpub).

**Fold verified in the plan:**
- Task 3 (line 61) captures `parentFP := key.ParentFingerprint()` in the SAME `hdkeychain.NewKeyFromString` decode used to get `(chainCode, compressedPubkey)`, and `deriveSingleSigBundle` now RETURNS `(b bundle.Bundle, masterFP uint32, parentFP uint32, xpub string, err error)` (line 60 signature) — parentFP + xpub threaded out for Task 6. ✓
- Task 6 (line 99) sets `bip380.Key.ParentFingerprint: parentFP` with the explicit annotation "the REAL non-zero parent fp from Task 3's decode, NOT 0; else Key.String()/desc.Encode() re-serializes a non-canonical xpub that doesn't byte-match the engraved mk1." ✓
- Task 6 Step 1 (line 101) asserts "`desc.Encode()` (the `Key.String()` xpub) BYTE-MATCHES the engraved mk1 card's xpub" for the abandon-test seed (a real golden), with the rationale "address-match alone would hide a dropped parentFP." ✓ (byte-match, not just address-match — exactly the strengthened assertion R0 required)
- Task 7 (line 113) threads `parentFP + xpub` from `deriveSingleSigBundle` into `restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path)` — the orchestrator passes it. ✓
- Task 3 Step 1 (line 63) also asserts "the returned `parentFP` is the real non-zero parent fingerprint (for Task 6)." ✓

**Source re-confirmation @ bfff857:**
- `bip380.Key.ExtendedKey()` (`bip380/bip380.go:97-109`) builds `fp` from `k.ParentFingerprint` (`binary.BigEndian.PutUint32(fp[:], k.ParentFingerprint)`, `:98-99`) and `Key.String()` = `k.ExtendedKey().String()` (`bip380/bip380.go:111-113`) → the parentFP bytes ARE part of the serialized xpub; `Descriptor.encode` writes `k.String()` per key (`bip380/bip380.go:215-222`). So a `0` parentFP genuinely yields a different base58 string. The fix root-cause is real and the fold addresses it. ✓
- `compactFromXpub` exposes `key.ParentFingerprint()` in the same decode it already does for chainCode/pubkey: `key, err := hdkeychain.NewKeyFromString(xpub)` (`mk/encode.go:117`), then `key.ParentFingerprint()` (`mk/encode.go:161-164`). So Task 3 capturing parentFP from the SAME `NewKeyFromString` is directly feasible against the verified pattern. ✓
- The account xpub from `deriveAccountXpub` is depth-3 (m/purpose'/coin'/account', `gui/derive.go:50` serializes `acct.String()` after a 3-component path) → its parent fingerprint is the non-zero fingerprint of the m/purpose'/coin' key. The "real non-zero parentFP" assumption is correct. ✓
- masterFP-vs-parentFP discipline preserved: `deriveAccountXpub` returns `masterFP = bip32.Fingerprint(pk)` of the MASTER (`gui/derive.go:31`), which is what `EncodeSingleSig.fp` and `bip380.Key.MasterFingerprint` use; parentFP is the distinct account-xpub parent. The plan keeps these two correctly separate (Task 3 line 63: "`EncodeSingleSig.fp` uses masterFP… not the xpub parent fp"; Task 6 line 99: `MasterFingerprint: masterFP, ParentFingerprint: parentFP`). ✓

**Ruling: I1 CLOSED.**

### I2 — reminder gate (was IMPORTANT) → **CLOSED**

R0 required the ms1-reminder suppression to be `cards`-derived inside `bundleEngrave` with NO signature/param change, to preserve T5's byte-unchanged call site.

**Fold verified in the plan:** Task 4 Step 3 (line 77): "**R0-I2: gate the end-of-engrave `bundleMs1ReminderText` on a `cards`-DERIVED signal — `any(card.kind == cardMS1)` over the `cards` slice INSIDE `bundleEngrave` (suppress iff an ms1 card was engraved). Do NOT add a parameter / change the `bundleEngrave` signature** (T5's call site `bundle_flow.go:36` must stay byte-unchanged; T5 gather never produces `cardMS1` → its reminder still shows). Keep `bundleMs1ReminderText()` defined." ✓ — exactly the contract-preserving mechanism R0 mandated.

**Source re-confirmation @ bfff857:**
- `func bundleEngrave(ctx *Context, th *Colors, cards []bundleCard)` (`gui/bundle_flow.go:327`); reminder unconditional at `bundle_flow.go:360` (`showError(ctx, th, "Engrave Bundle", bundleMs1ReminderText())`). Gating on `any(card.kind==cardMS1)` over `cards` keeps the signature identical. ✓
- T5's call site `bundle_flow.go:36` is `bundleEngrave(ctx, th, cards)` — a `cards`-only call, byte-unchanged under this approach. ✓
- T5 gather never emits `cardMS1`: `bundleCardKind` is `cardMK1`/`cardMD1` only (`gui/bundle.go:24-27`); the gatherer appends only those two kinds (`gui/bundle.go:201,239`), and `classify` never produces a third (`gui/bundle.go:63-98`). So T5 bundles always satisfy `any(card.kind==cardMS1)==false` → reminder still shown → T5 behaviour unchanged. ✓
- `bundleMs1ReminderText()` stays defined (Task 4 line 77 says "Keep `bundleMs1ReminderText()` defined" — `TestBundleEngraveMs1Reminder` calls it directly). ✓

**Ruling: I2 CLOSED.**

### Minors (m1–m4 + Task-0 prerequisite) → **all CLOSED**

- **m1 (decoded-fields + bound-stub, NOT raw-string vs T4):** Task 3 Step 1 (line 63) now reads "**R0-m1 — decode mk1 via `mk.Decode` and assert the DECODED FIELDS (network/path/fingerprint/xpub) match T4's known card AND the stub == `WalletPolicyIDStubChunks(md1)` (NON-zero, NOT `[0,0,0,0]`)** — do NOT assert raw-string-equality vs T4's golden (the bound stub changes the bytes)." ✓ Source: `WalletPolicyIDStubChunks(strs []string) ([4]byte, error)` (`md/walletpolicyid.go:126`) — non-zero bound stub changes `mk.Encode` bytes vs T4's `[0,0,0,0]`, so decoded-field comparison is the correct test. **CLOSED.**
- **m2 (24-word seed; longest ms1 fits; no whole-bundle abort):** Task 4 Step 1 (line 75) now reads "**R0-m2: include a 24-word seed** (ms1 = 75 chars) to prove the LONGEST ms1 engraves (fits a plate via `validateMdmk`) and does NOT trip the whole-bundle abort (`bundle_flow.go:331-337`)." ✓ Source: the whole-bundle abort is real at `bundle_flow.go:331-337` (`if err != nil || len(plates) == 0 { bundleAbortWarning…; return }`). **CLOSED.**
- **m3 (LOCAL single-sig table, not the shared `var scriptTypePurpose`):** Task 2 Step 3 (line 51) now reads "**R0-m3:** define a NEW local/unexported single-sig table (BIP-84 first, then 44/49/86) in this file; do NOT mutate or index-couple to the shared package-level `var scriptTypePurpose` (`gui/derive_xpub.go:32-42`, order load-bearing for the 6-entry picker)." ✓ **CLOSED.**
- **m4 (explicit `<0;1>/*` Children):** Task 6 (line 99) now sets `Children: []bip380.Derivation{{Type: RangeDerivation, Index: 0, End: 1}, {Type: WildcardDerivation}}` with the annotation "**R0-m4 — set `<0;1>/*` EXPLICITLY, matching `useSiteToChildren` `md1_expand.go:144-147`; don't rely on the empty-default." ✓ Source re-confirmed: `useSiteToChildren` returns exactly `[]bip380.Derivation{{Type: bip380.RangeDerivation, Index: idx, End: end}, {Type: bip380.WildcardDerivation}}` (`gui/md1_expand.go:144-147`). **CLOSED.**
- **Task-0 fork-fetch prerequisite:** Task 0 Step 1 (line 26) now reads "Ensure `bfff857` is reachable first — `git -C /scratch/code/shibboleth/seedhammer fetch origin` … the work happens in the `bg002h` fork checkout, where `bfff857` is HEAD … Then `git worktree add … bfff857`." ✓ — the R0 process-note prerequisite is folded in. **CLOSED.**

(R0's m5 was explicitly "No action" — the Task-0 baseline set covers all touched packages; still true.)

---

## No-drift re-confirmation (R0-confirmed-sound parts still hold @ bfff857)

1. **Enum insertion + lockstep:** Enum at `gui/gui.go:147-152` is `backupWallet=0, engraveXpub=1, engraveBundle=2, qaProgram=3`; plan inserts `engraveSingleSig` between `engraveBundle` and `qaProgram` → `…engraveBundle=2, engraveSingleSig=3, qaProgram=4`. All FOUR navigable bounds keyed on `engraveBundle` verified present: left-wrap `gui/gui.go:1634` (`m.prog = engraveBundle`), right-wrap `:1641` (`if m.prog > engraveBundle`), `npage` `:1840` (`const npage = int(engraveBundle)+1`), `npages` `:1859` (`const npages = int(engraveBundle)+1`). Dispatch arm `:1497` (`case engraveBundle: bundleFlow`), title arm `:1664`, `layoutMainPlates` arm `:1850` (`case backupWallet, engraveXpub, engraveBundle:`). Both nav-tests hardcode `engraveBundle` as the wrap upper bound (`gui/bundle_program_test.go:34,43`; `gui/derive_xpub_program_test.go:30,39`). Plan Task 1 (line 37) moves all four bounds + adds all three arms + updates both nav-tests. Consistent, no drift. ✓
2. **Bound-stub wiring (masterFP-not-parentFP; `WalletPolicyIDStubChunks([]string)`; drop `stubZeroWarning`):** `WalletPolicyIDStubChunks(strs []string) ([4]byte, error)` is the gui-callable strings form (`md/walletpolicyid.go:126`). `deriveAccountXpub` returns `masterFP = bip32.Fingerprint(pk)` (master, `gui/derive.go:31`). `stubZeroWarning` exists at `gui/derive_xpub.go:237` (called `:157`) — the plan correctly drops it for the bound (non-zero) stub. ✓
3. **cardMS1 append safety + `validateMdmk` format-agnostic:** Appending `cardMS1` after `cardMD1` (`gui/bundle.go:27`) is safe — the gather classify never emits it (`gui/bundle.go:63-98`) and the gather tally switch counts only `cardMD1`/`cardMK1`, ignoring others (`gui/bundle_flow.go:73-82`), so no tally arm change is forced. `validateMdmk` is format-agnostic and `bundleEngrave` engraves any card's `strings` uniformly (`bundle_flow.go:328-357`). ✓
4. **Watch-only `bundle.Verify` extension:** `Verify(derived, readback Bundle)` currently always decodes ms1 entropy on both (`bundle/verify.go:70-78`). The plan's extension (empty-both → skip ms1, one-sided → error, stub+mk1+md1 legs always run) sits cleanly before `:68`; stub-binding (`:36-41`), mk1 fp/xpub/path (`:44-60`), md1-exact (`:64-66`) all run regardless. ✓
5. **sh-wpkh direct-descriptor (`address.addressAt` handles P2SH_P2WPKH, bypass classifier):** `address.addressAt` for `Singlesig` handles `P2SH_P2WPKH` at `address/address.go:144` (witness-pubkey-hash) then wraps to P2SH at `:160-170`; script map 44→P2PKH/49→P2SH_P2WPKH/84→P2WPKH/86→P2TR matches the bip380 enum. ✓
6. **Typed-only seed (D12) + per-leg scrub (D11):** `seedEntryFlow` at `gui/derive_xpub.go:82` (typed entry); `deriveAccountXpub` scrubs seed/master/intermediates with the verified serialize-before-Zero ordering (`gui/derive.go:21,28,35,43,46-51`). Plan Task 7 (line 113) gates `m.Entropy()`, defers `wipeBytes`, scrubs the mnemonic `[]Word`. ✓
7. **No-DescriptorScreen for restore-doc (alloc gate):** Plan Task 6 (line 99) uses `address.Receive/Change` + a plain screen, NOT `DescriptorScreen`. ✓
8. **TDD order:** Every task is fail → run-fail → impl → run-pass → commit; `TestAllocs` re-run after the enum change (Task 1 line 38). ✓

**parentFP threading consistency (T3 → T7 → T6):** Task 3 captures+returns `parentFP` (line 60-63); Task 7 receives it from `deriveSingleSigBundle` and passes it into `restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path)` (line 113); Task 6's `restoreDocFlow` signature takes `parentFP uint32` (line 99) and sets `bip380.Key.ParentFingerprint: parentFP`. The three signatures agree end-to-end; the orchestrator passes it. No drift. ✓

---

## Non-blocking observation (NOT a finding)

**OBS-1 — recon "REUSE as-is" one-liner still lists `mk1/md1GatherFlow+collected()`.** `seedhammer-T6a2-gui-recon.md:41` (the "Classification → REUSE as-is" summary line) still names "`mk1/md1GatherFlow+collected()`" among reusable surfaces. This is a stale pre-fold leftover that *reads* as contradicting the corrected Topic 6 (`:31-32`), which explicitly forbids those three for the read-back. It is non-blocking because (a) the gated artifact is the PLAN, and the plan's Task 5 (lines 87-89) is unambiguous and correct (uses the `bundleGatherer`, forbids the three with correct return types); (b) recon Topic 6 — the authoritative read-back surface map — is corrected and explicit. The `:41` line is a generic "these functions exist and are reusable" inventory, not a wiring instruction, so it doesn't mislead an implementer who follows the plan. Recommend (optional hygiene, not gating) striking "`+collected()`" / qualifying the `:41` mention to point at Topic 6, to remove the apparent contradiction. Does not affect GREEN.

**OBS-2 (informational) — `EncodeMS1` returns `(string, error)`.** `codex32.EncodeMS1(entropy []byte) (string, error)` (`codex32/msencode.go:17`). Task 3 step (6) writes `ms1 := codex32.EncodeMS1(m.Entropy())` with the gloss "gate validity first"; the implementer must bind both returns (handle the error). This is implicit in TDD + the "gate `m.Entropy()` validity" instruction and is not a plan defect. Noted only so the single-implementer doesn't transcribe a single-value assignment.

---

## Cleared

The plan is **GREEN — cleared for single-implementer TDD** in the `feat/t6a2-gui` worktree off `bfff857`. Execute the GREEN plan as written (single subagent, strict TDD, fail → run-fail → impl → run-pass → commit per task), then the mandatory whole-diff adversarial execution review before merge. C1, I1, I2 and all minors are CLOSED; no new findings; no drift from the fold.

---

## Process note (not a plan finding)

The local `third_party/seedhammer` submodule is pinned to upstream `713aee2`, which does NOT contain the fork's T4/T5/T6a-1 surface. Verification used the fork checkout at `/scratch/code/shibboleth/seedhammer` (HEAD = `bfff857c5a30ff82fa5eddbd52452cf70e9116de`), where all cited files are present. Task 0's folded fork-fetch prerequisite (`git -C /scratch/code/shibboleth/seedhammer fetch origin` before `git worktree add … bfff857`) correctly addresses the R0 process note. This reviewer modified nothing; no throwaway build was needed (all facts were verifiable by reading source @ the fork HEAD).
