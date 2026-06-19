# Plan R0 Gate Review — T6a-2 (GUI) single-sig flagship on-device flow

**Reviewer:** opus architect (adversarial plan R0 gate, pre-implementation)
**Date:** 2026-06-19
**Plan under review:** `design/IMPLEMENTATION_PLAN_seedhammer_T6a2_gui.md`
**Recon:** `design/agent-reports/seedhammer-T6a2-gui-recon.md`
**Spec (GREEN, Phase B):** `design/SPEC_seedhammer_T6a_singlesig_flagship.md`
**Fork verified @:** `bfff857` (fetched from `bg002h/seedhammer`, checked out into a throwaway worktree; the local submodule is pinned to upstream `713aee2` which does NOT contain the fork's T4/T5/T6a-1 surface — see Process note).

---

## VERDICT: NOT GREEN

**1 Critical · 2 Important · 5 Minor.**

The plan's spine is sound and most facts check out against `bfff857`: the enum-insertion lockstep is complete and correctly enumerated, the bound-stub wiring (incl. masterFP-not-parent-fp) is right, `validateMdmk` is genuinely format-agnostic so the `cardMS1` engrave is mechanically sound, the watch-only `bundle.Verify` extension is clean and has zero downstream callers, and the sh-wpkh direct-descriptor path is verified to produce correct addresses. But two findings block: (C1) Task 5's read-back wiring names an API shape that does not exist — `mk1GatherFlow`/`md1GatherFlow` do NOT return `.collected()` strings, and `md1GatherFlow` returns only `bool`, so the readback `bundle.Bundle` cannot be assembled as specified; and (I1) Task 6 builds the restore-doc `bip380.Key` from the account xpub but drops the real `ParentFingerprint`, so the displayed/exported descriptor's xpub is non-canonical (does not byte-match the engraved mk1 xpub) even though derived addresses are correct.

Fold C1 + I1 (+ ideally I2), re-dispatch.

---

## CRITICAL

### C1 — Task 5 read-back wiring uses a non-existent API: `mk1GatherFlow`/`md1GatherFlow` do NOT expose `.collected()`, and `md1GatherFlow` returns only `bool` (the verbatim strings are discarded)

**Plan location:** Task 5, Step 1 + Step 3 (lines 87, 89): "read back mk1/md1 over NFC (`mk1GatherFlow`/`md1GatherFlow` → `.collected()`) … assemble the readback `bundle.Bundle`". Recon Topic 6 (line 32) propagates the same wrong shape: "`mk1GatherFlow`→`mk1Gatherer.collected()[]string`; `md1GatherFlow`→`md1Gatherer.collected()`".

**Source (verified @ bfff857):**
- `gui/mk1_inspect.go:156` — `func mk1GatherFlow(ctx *Context, th *Colors, first string) (mk.Card, bool)`. It owns its scanner, collects the set, and returns a **decoded `mk.Card`** — NOT `[]string`, NOT a `*mk1Gatherer`. `.collected()` (`mk1_inspect.go:77`, `func (g *mk1Gatherer) collected() []string`) is a method on the **private gatherer**, which `mk1GatherFlow` constructs internally (`g := &mk1Gatherer{}`, `:157`) and never returns.
- `gui/md1_gather.go:72` — `func md1GatherFlow(ctx *Context, th *Colors, first string) bool`. It returns **only a bool**. On completion it calls `gatheredDescriptorFlow(ctx, th, g.collected())` internally (`:76`, `:140`) and **discards the strings** — there is no path for a caller to obtain the verbatim `[]string`.
- `bundle.Verify(derived, readback bundle.Bundle)` needs `bundle.Bundle{MS1 string; MK1 []string; MD1 []string}` (`bundle/verify.go:19,32`) — i.e. the verbatim chunk strings. Neither gather-flow yields them.

**Why it blocks:** Task 5's central act — "assemble the readback `bundle.Bundle` → `bundle.Verify`" — is impossible with the cited functions. `md1GatherFlow` gives the implementer a `bool` and nothing else; `mk1GatherFlow` gives a `mk.Card`. The plan's verify flow has no way to populate `readback.MK1`/`readback.MD1`. An implementer following Task 5 verbatim hits a compile/design wall, and (the real R0 risk) may paper over it by comparing decoded `mk.Card`/`md.Template` fields ad-hoc instead of routing through the shipped, tested `bundle.Verify` — losing the md1-exact-string and stub-binding legs the comparator guarantees (`verify.go:62-66,88-105`). Both flows also require a `first` chunk argument primed from an initial NFC scan, which the plan's "read back" step doesn't account for.

**Exact fix (pick one, state it in the plan):**
- **(a) Preferred — reuse the T5 `bundleGatherer`.** It already yields `bundleCard{strings []string}` for both kinds (`gui/bundle.go:32-37,201-206,239-244`), is csid-keyed (handles the chunk-set first/next plumbing), and refuses an ms1 over NFC. The verify flow gathers the read-back cards via the bundle gatherer (or a thin wrapper around it), pulls `cardMK1.strings`/`cardMD1.strings`, hand-types the ms1 (`inputCodex32Flow` → `codex32.New`), assembles `bundle.Bundle{MS1, MK1, MD1}`, and calls `bundle.Verify`. This keeps the verbatim-string contract intact.
- **(b) Drive the low-level gatherers directly.** Build the verify-read-back screen around `mk1Gatherer`/`md1Gatherer` + `.collected()` (the methods that DO return `[]string`), not the `*GatherFlow` wrappers. More net-new UI than (a).
- **(c) Re-encode from the decoded card.** Use `mk1GatherFlow`'s returned `mk.Card` and `mk.Encode(card)` to reproduce `MK1 []string` (deterministic; `mk.Decode` round-trips stubs). For md1 there is no equivalent because `md1GatherFlow` returns nothing — you'd still need `md.DecodeChunks`/re-encode plumbing. Messy; not recommended.

Whichever is chosen, update both the plan (Task 5 Step 1/Step 3) **and** recon Topic 6 so the "→ `.collected()`" shorthand is replaced with the real path, and add the `first`-chunk priming to the flow description.

---

## IMPORTANT

### I1 — Task 6 restore-doc descriptor drops the real `ParentFingerprint`, so the displayed/exported descriptor xpub is non-canonical (≠ the engraved mk1 xpub)

**Plan location:** Task 6 (line 99): builds `*bip380.Descriptor` with `Key{… KeyData: xpub[32:65], ChainCode: xpub[0:32]}` and "Display … the descriptor". `ParentFingerprint` is unspecified; the cited template (`gui/md1_expand.go:60-69`) sets `ParentFingerprint: 0`.

**Source (verified @ bfff857):**
- `bip380.Key.ExtendedKey()` (`bip380/bip380.go:96-109`) reconstructs the xpub from `KeyData`, `ChainCode`, `ParentFingerprint` (→ `fp[:]`), `depth = uint8(len(DerivationPath))`, and `childNum = DerivationPath[last]`.
- `bip380.Descriptor.Encode()` (`bip380/bip380.go:171`, `encode` `:183-224`) writes, per key, the origin `[<MasterFingerprint><DerivationPath>]` **and** `k.String()` = `k.ExtendedKey().String()` (`:215-222`, `Key.String` `:111-113`).
- The account xpub returned by `deriveAccountXpub` (`gui/derive.go:50`, `acct.String()`) carries a **real non-zero parent fingerprint** (the fingerprint of the m/purpose'/coin' key). Building the `Key` with `ParentFingerprint: 0` makes `ExtendedKey().String()` re-serialize an xpub whose parent-fp bytes are `00000000` → a **different base58 string** than the engraved mk1 xpub, even though depth (3) and childNum (0') match.
- The real parentFP IS recoverable: `compactFromXpub` already does exactly this — `hdkeychain.NewKeyFromString(xpub)` then `key.ParentFingerprint()` (`mk/encode.go:117,161-164`, verified). So the Task-3 `xpub→(chainCode,pubkey)` decode glue can capture `key.ParentFingerprint()` in the same step and thread it to Task 6.

**Why it blocks:** This is the security-critical restore document. Derived **addresses** are unaffected (BIP-32 CKD uses only `KeyData`+`ChainCode`), so the address-match tests (Task 6 Step 1) pass and silently hide the defect. But the **descriptor string the user records/imports** would contain a non-canonical xpub that does not byte-match the mk1 key card the user just engraved, breaking the "this descriptor restores this exact wallet" contract and tripping wallets that canonicalize/validate the key-origin on import. The acceptance bar (line 137, "restore doc addresses correct … display-only, no secret") doesn't assert xpub canonicality, so the defect ships unless caught here.

**Exact fix:** In Task 3's xpub-decode glue, additionally capture `parentFP := key.ParentFingerprint()` (uint32) from the same `hdkeychain.NewKeyFromString`, return it from `deriveSingleSigBundle`, and in Task 6 set `bip380.Key.ParentFingerprint: parentFP`. Add a Task-6 assertion: `desc.Encode()` (or `Key.String()`) **byte-equals the mk1 card's xpub** for the abandon-test seed (a real golden). (Note: this is strictly stronger than what md1_expand needs, because md1_expand only ever has the md1 wire, which carries no parentFP — there `0` is unavoidable and addresses-only is the contract. Here the canonical xpub is in hand, so dropping it is a needless regression.)

### I2 — Task 4 reminder-gating must derive from `cards` (not a new param), or it silently changes T5's `bundleEngrave` contract / breaks the byte-unchanged claim

**Plan location:** Task 4, Step 3 (line 77): "Gate the end-of-engrave `bundleMs1ReminderText` on 'did we engrave an ms1 card?' — suppress when full, show when watch-only." Task 8 (line 124) + Acceptance (line 139) require T5 `bundleFlow`/`bundleEngrave` **byte-unchanged**.

**Source (verified @ bfff857):**
- `gui/bundle_flow.go:327` — `func bundleEngrave(ctx *Context, th *Colors, cards []bundleCard)`; the reminder is **unconditional** at `:360` (`showError(ctx, th, "Engrave Bundle", bundleMs1ReminderText())`).
- T5's `bundleFlow` (`bundle_flow.go:36`) calls `bundleEngrave(ctx, th, cards)` with PUBLIC-only cards (gather never produces `cardMS1`, `gui/bundle.go:24-27`). T5 expects the reminder **shown** (its bundles never engrave the secret ms1).
- T5 tests: `gui/bundle_engrave_test.go` — `TestBundleEngraveGuidedTitles`/`SetAbort` (`:64,:84`) don't pump to the reminder; `TestBundleEngraveMs1Reminder` (`:105`) calls `bundleMs1ReminderText()` **directly** (just asserts the string content), not via `bundleEngrave`.

**Why it matters:** The plan must gate the reminder **without** changing the `bundleEngrave` signature or T5's call site (or Task 8's "byte-unchanged" + the "single-card/T4/T5 flows pass verbatim" claim is false). The clean, contract-preserving approach is: inside `bundleEngrave`, detect `any(card.kind == cardMS1)` over `cards` and suppress the reminder iff an ms1 card was engraved. Because T5 gather never produces `cardMS1`, T5's path keeps the reminder shown and its behaviour is unchanged. The plan should state this explicitly: **the "did we engrave ms1?" signal is `cards`-derived, NOT a new function parameter.** If instead the plan threads a bool param, it changes the signature and `bundleFlow`'s call site → contradicts the byte-unchanged acceptance bar. Minor-adjacent but Important because the plan as worded leaves the mechanism ambiguous and one obvious reading (add a param) violates an explicit acceptance criterion. (Also: keep `bundleMs1ReminderText()` defined — `TestBundleEngraveMs1Reminder` calls it directly.)

---

## MINOR (non-blocking)

### m1 — Task 3 "mk1 == T4's known card" will NOT be string-equal (different stub → different bytes)
Plan Task 3 Step 1 (line 63): "mk1 == T4's known card BUT with stub == `WalletPolicyIDStubChunks(md1)`". T4 uses `Stubs:[][4]byte{{0,0,0,0}}` (`gui/derive_xpub.go:142`); the bound stub is non-zero, so `mk.Encode` emits **different bytes** than T4's golden (`mk_test.go` confirms non-zero stubs like `c0ffee00` encode fine). The assertion must compare **decoded fields** (network/path/fp/xpub via `mk.Decode`) + assert the stub is the bound non-zero value, NOT raw-string-equal T4's card. Reword to avoid a brittle golden-string test.

### m2 — verify the 24-word ms1 (75 chars) fits a plate before relying on `cardMS1` engrave
Measured @ bfff857: `EncodeMS1` yields **50 / 62 / 75** chars for 16/24/32-byte entropy (12/18/24 words). `bundleEngrave` aborts the WHOLE bundle if `validateMdmk` returns no fitting plate (`bundle_flow.go:331-337`). 75 chars is comparable to mk1 xpub chunks (which fit), so it almost certainly fits TEXT+QR, but the Task 4 test should include a **24-word** seed to prove the longest ms1 engraves (not just abort-on-no-fit). Otherwise a long-seed user hits a full-bundle abort.

### m3 — `scriptTypePurpose` is a package-level mutable `var`, not a func
Plan Task 2 (line 51) says "clone the `scriptTypePurpose`/path-build idea". `scriptTypePurpose` is a package-level `var []struct{...}` (`gui/derive_xpub.go:32-42`), order load-bearing (indexed by the stage-1 choice). The clone should be a **local/unexported** single-sig table to avoid mutating or index-coupling to the shared 6-entry one. Trivial, just flag so the implementer doesn't reorder the shared var.

### m4 — confirm `Children` choice for restore-doc recv/change is explicit `<0;1>/*`
`address.derivePubKey` defaults to `<0;1>/*` only when `Children` is **empty** (`address/address.go:176-188`); for change it uses `End`(=1), receive uses `Index`(=0) on the `RangeDerivation`. Task 6's `Children: [RangeDerivation{0,1}? or recv/change]` is correct either way (empty → default, or explicit `[{Range 0,1},{Wildcard}]`). Recommend setting it **explicitly** to `[]bip380.Derivation{{Type:RangeDerivation,Index:0,End:1},{Type:WildcardDerivation}}` (matches `useSiteToChildren`'s standard output, `md1_expand.go:144-147`) for clarity and to not depend on the empty-default.

### m5 — Task 0 baseline test set omits a package the plan touches indirectly
Task 0 Step 2 baselines `./gui/... ./md/... ./codex32/... ./bundle/... ./mk/... ./bip380/... ./address/...` — good and complete for the touched packages. No action; noting it covers all packages the plan modifies/depends on (the only headless modify is `bundle/`, correctly included).

---

## Explicit rulings requested

1. **Enum-insertion completeness (Task 1):** **CORRECT and complete.** Verified @ bfff857: the navigable bounds keyed on `engraveBundle` are exactly `:1634` (left-wrap `m.prog = engraveBundle`), `:1641` (right-wrap `if m.prog > engraveBundle`), `:1840` (`const npage = int(engraveBundle)+1`), `:1859` (`const npages = int(engraveBundle)+1`) — the plan moves all four to `engraveSingleSig`. Dispatch (`:1490-1500`), title (`:1659-1666`, no qaProgram arm → blank if missed), and `layoutMainPlates` (`:1848-1856`, `panic("invalid page")` if missed) arms must be added — plan does all three. `qaProgram` is reached ONLY via the `"FOREVERLAURA!"` debug command (`:1601-1602`, symbolic) → renumbering 3→4 is safe; stays non-navigable. Both nav-tests hardcode `engraveBundle` as the wrap bound (`gui/bundle_program_test.go:43-44`, `gui/derive_xpub_program_test.go:30-32`) → plan updates BOTH. `TestAllocs` exercises only `StartScreen.Flow` + `DescriptorScreen.Confirm` (`gui_test.go:64-71`); the new title/plate arms are string-literal/image-op only and the pager just loops one more iteration → alloc-free; plan's re-run is the right guard. **No finding.**

2. **Bound-stub wiring incl. masterFP-not-parent-fp (Task 3):** **CORRECT.** `WalletPolicyIDStubChunks([]string)([4]byte,error)` takes the md1 **strings** and is gui-callable (`md/walletpolicyid.go:126`); `WalletPolicyId`/`WalletPolicyIDStub` take an unexported `*descriptor` and are NOT (`:30,:103`). `EncodeSingleSig.fp` = the account `masterFP` from `deriveAccountXpub` (the **master** fingerprint, captured before zeroing, `gui/derive.go:31`), NOT `xpub.ParentFingerprint()` — plan's md1-embedded-fp assertion is the right check. The xpub→(chainCode[32],compressedPubkey[33]) decode is feasible via the verified `compactFromXpub` pattern (`mk/encode.go:117,147,151-157`). Order xpub→`EncodeSingleSig`→`WalletPolicyIDStubChunks`→`mk.Encode(Stubs:[][4]byte{stub})`, drop `stubZeroWarning` — all sound. (See m1 for the golden-string nit.)

3. **cardMS1 addition safety (Task 4):** **SAFE, with I2.** `validateMdmk` (`gui/gui.go:1903`) is genuinely **format-agnostic** — it QR-encodes + lays out ANY string as TEXT/TEXT+QR with no md/mk validation → engraving an ms1's codex32 string through it works. `bundlePlatePlan` (`bundle_flow.go:303-318`) iterates `c.strings` for any kind → a `cardMS1` engraves uniformly. Appending `cardMS1` after `cardMD1` (`gui/bundle.go:27`) leaves T5 gather/classify untouched (classify never emits `cardMS1`; the gatherer's tally switch `bundle_flow.go:75-82` only counts cardMD1/cardMK1 and ignores others — verify the implementer adds a tally arm only if the verify-readback path reuses the tally, else fine). **Leak vector check:** engraving the SECRET ms1 onto owner-held steel via the same sequencer is acceptable — it goes to a plate, never NFC (the NFC channel still refuses ms1, `gui/bundle.go:69`, `bundle_flow.go:55-56`); the only residual is the un-wipeable immutable `string`, already accepted by the spec (§4d). The reminder gating is I2.

4. **Watch-only `bundle.Verify` extension (Task 5):** **CLEAN, low-risk, in-scope.** `bundle/verify.go:32` currently always decodes ms1 entropy on both bundles (`:70-78`). The extension is a ~6-line guard before `:68`: `if derived.MS1=="" && readback.MS1=="" { return nil-after-mk1/md1/stub-legs }`; `if (derived.MS1=="") != (readback.MS1=="") { error("ms1 presence mismatch") }`; else unchanged. Stub-binding (`:36-41`) + mk1 fp/xpub/path (`:44-60`) + md1-exact (`:64-66`) legs run regardless → full-mode behaviour preserved. There are **zero** callers of `bundle.Verify` outside its own tests (grep clean) → no downstream breakage; the only test surface is `bundle/verify_test.go`. The headless touch is correctly isolated to Task 5 and tested headlessly. **No finding** (the GUI wiring that consumes it is C1).

5. **sh-wpkh direct-descriptor path (Task 6):** **CORRECT — addresses are right for all 4 incl. sh-wpkh.** `address.addressAt` for `Type==Singlesig` (`address/address.go:133-155`) switches on `desc.Script` and handles `P2SH_P2WPKH` at `:144` (witness-pubkey-hash), then wraps to P2SH at `:160-170` → a directly-built `{Type:Singlesig, Script:P2SH_P2WPKH}` yields a correct nested-segwit address, bypassing `classifyPolicy` (which sends single-key sh-wpkh to `PolicyComplex`, `md/md.go:1283-1300`, verified) and `scriptForTemplate` (no sh-wpkh+single arm, `md1_expand.go:96-97`). The `KeyData=xpub[32:65]`/`ChainCode=xpub[0:32]` mapping matches the verified template (`md1_expand.go:65-66`). The plan correctly does NOT route through `DescriptorScreen` (the alloc gate exercises `DescriptorScreen.Confirm`, `gui_test.go:60-71`). **The descriptor-string canonicality is I1** (addresses correct, but the displayed/exported xpub must carry the real parentFP). Script map 44→P2PKH/49→P2SH_P2WPKH/84→P2WPKH/86→P2TR matches the bip380 enum (`bip380.go:39-48`).

6. **Security spine:** **Sound.** Typed-only seed (D12) is correctly anchored on `seedEntryFlow` (`gui/derive_xpub.go:82`, 12/24-word typed entry, zero scan refs — verified) and NEVER `scan` — the footgun is real (`gui/scan.go:28,61-62,70-73`: the scanner returns a `bip39.Mnemonic` AND a `codex32.String` from NFC). The D12 structural test ("references `seedEntryFlow`, NOT `scan`/`assembleScan` for the secret") + behavioural test ("a scanned bip39/codex32 cannot reach the derive entrypoint") is **falsifiable as specified** — a regression that routes a scanned object to `deriveSingleSigBundle` fails the behavioural test. Per-leg scrub (D11): entropy `defer wipeBytes` after `EncodeMS1` (gate `m.Entropy()` validity first — it panics on invalid), seed/master/intermediates scrubbed inside `deriveAccountXpub` (verified `gui/derive.go:21,28,35,43,51` + the serialize-before-Zero ordering at `:46-51`), mnemonic `[]Word` zeroed after the last derive consumer (`derive_xpub.go:113-117` pattern), restore-doc public. ms1 engraved-only/never-NFC: confirmed. **No finding** (subject to C1's verify-flow being rebuilt without leaking strings, and I1's restore-doc carrying no secret — both already public material).

7. **TDD order + executability + no-regression:** Tasks are bite-sized and test-first; the `NFCReader()==nil` approach is feasible (`gui/gui_test.go:408` is overridable; `runUI`/`click`/`pumpUntil`/`newPlatform` all exist — `gui_test.go:467`, `event_test.go:42`, `slip39_polish_test.go:329`, `gui_test.go:452`). No undefined helper is referenced before definition **except** the C1 case (Task 5 references `.collected()` off the flow, which doesn't exist). The headless `bundle.Verify` touch is correctly isolated in Task 5. Task split is otherwise sound. The byte-unchanged claim is jeopardised only by I2 (the reminder-gating mechanism).

---

## Process note (not a plan finding)

The local submodule `third_party/seedhammer` is pinned to **upstream** `713aee2`, which does NOT contain the fork's T4/T5/T6a-1 commits — most files the plan cites (`gui/derive_xpub.go`, `gui/bundle*.go`, `md/encode_singlesig.go`, `codex32/msencode.go`, `bundle/verify.go`, etc.) are absent at that HEAD. Verification required fetching `bg002h/seedhammer` and checking out `bfff857` into a throwaway worktree (`/tmp/sh-bfff857`, since removed). This is expected per CLAUDE.md (the submodule tracks upstream; the fork lives on `bg002h`), but Task 0's `git worktree add … bfff857` will **fail in a fresh clone** unless the fork remote is fetched first. Recommend Task 0 Step 1 explicitly fetch `bg002h/seedhammer` (or document that the executing environment already has `bfff857` reachable) before `git worktree add … bfff857`.

---

## Re-dispatch checklist (must reach 0C/0I)
- [ ] **C1:** Rewrite Task 5's read-back to use a real API that yields verbatim `[]string` (preferred: the T5 `bundleGatherer` → `bundleCard.strings`); correct recon Topic 6's "→ `.collected()`" shorthand; account for the `first`-chunk priming. Keep the assembled `bundle.Bundle` → `bundle.Verify` contract.
- [ ] **I1:** Capture `key.ParentFingerprint()` in Task 3's xpub decode, thread it to Task 6's `bip380.Key.ParentFingerprint`, and add a Task-6 assertion that `desc.Encode()`/`Key.String()` xpub byte-matches the engraved mk1 xpub.
- [ ] **I2:** State that the ms1-reminder suppression is **`cards`-derived** inside `bundleEngrave` (no signature/param change), preserving T5's byte-unchanged call site; keep `bundleMs1ReminderText()` defined.
- [ ] (recommended) Fold m1–m4; add the Task-0 fork-fetch prerequisite.
- [ ] Re-dispatch this R0 after folding (folds can drift).
