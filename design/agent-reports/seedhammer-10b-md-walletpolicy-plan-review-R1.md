# R0 GATE RE-REVIEW (Round R1) — IMPLEMENTATION PLAN #10b (md1 wallet-policy display + verify)

**Reviewer:** opus architect (plan R0 gate, re-review after fold). **Date:** 2026-06-19.
**Plan under review (folded):** `design/IMPLEMENTATION_PLAN_seedhammer_10b_md_walletpolicy.md`
**Prior review re-checked:** `design/agent-reports/seedhammer-10b-md-walletpolicy-plan-review-R0.md` (NOT GREEN — 2 Critical + 2 Important + 4 Minors).
**Sources verified against (authoritative, re-checked at file:line for this round):** GO fork `seedhammer` @ `3a55ae5` (#10a merged); RUST `descriptor-mnemonic/crates/md-codec` @ `c85cd49`.
**Posture:** adversarial; every fold claim independently re-checked at `file:line` in the live source, not taken from the plan's prose. Drift-check on everything R0 ruled clean.

---

## VERDICT: GREEN

All four blocking findings (C1, C2, I1, I2) are **CLOSED** with sound, source-grounded fixes that are consistently propagated across the affected tasks (T2/T3/T4/T5). The Minors (M1–M4) are folded or correctly judged harmless. No drift introduced by the fold. The two new tests the fold adds (bare-`sh(sortedmulti)` discriminant test b2; `End≠Index+1` reject test e2) are feasible and pinnable against the live source. The plan is **cleared for single-implementer TDD** in the worktree.

---

## DISPOSITION OF PRIOR FINDINGS

### C1 — unexported chunk-set error referenced cross-package → **CLOSED**
**Fold:** Task 2 Step 3 (plan line 50) now reads: *"EXPORT the chunk-set error sentinels (currently all unexported, `md/chunk.go:13-22`) — at minimum `md.ErrChunkSetIDMismatch` (and `ErrChunkSetIncomplete`) so package `gui` can `errors.Is`-dispatch…"*; Task 2 Step 1 (line 48) adds the failing test *"the csid-mismatch case is matchable via `errors.Is(err, md.ErrChunkSetIDMismatch)`"*; Task 4 Step 3 (line 80) dispatches via *"`on errors.Is(err, md.ErrChunkSetIDMismatch)` (R0-C1 — the EXPORTED sentinel)"*.
**Source re-verification:**
- The sentinel is still unexported today: `md/chunk.go:22 errChunkSetIDMismatch = errors.New("md: chunk set id integrity mismatch")` (lowercase). The full unexported set is `md/chunk.go:13-23`.
- `Reassemble` returns it on the integrity gate: `md/chunk.go:278-279 if deriveChunkSetID(id) != expCsid { return nil, errChunkSetIDMismatch }`. So `DecodeChunks`→`Reassemble` will propagate it; `errors.Is` works after rename (it is the same error value).
- In-package use sites the rename must touch: `md/chunk.go:22` (decl), `md/chunk.go:279` (return), `md/chunk_test.go:247-248` (the existing `!= errChunkSetIDMismatch` assertion). Exactly the three the R0 fix names (line 26 of R0); `grep -rn errChunkSetIDMismatch md/` returns precisely these.
- The only currently-exported md error is `md/md.go:18 ErrChunkedUnsupported`, confirming `gui` could not reach the lowercase form — so the export is genuinely load-bearing for the DISTINCT-csid-UX acceptance item (plan line 113).
**Assessment:** The export + `errors.Is` dispatch is consistent across T2 (export + test) and T4 (dispatch). The fold also opportunistically names `ErrChunkSetIncomplete` for an incomplete-vs-tampered UX split; that is optional and does not over-commit. CLOSED.

### C2 — `(ScriptSh, PolicySortedMulti)` ambiguity (P2SH vs P2SH_P2WSH) → verifies WRONG address → **CLOSED**
**Fold:** Task 2 Step 1 (line 48) now requires the result to *"carry a **nesting discriminant** … assert a `sh(sortedmulti)` vector and a `sh(wsh(sortedmulti))` vector are distinguishable."* Task 2 Step 3 (line 50) requires *"surface a NESTING DISCRIMINANT … Add a precise discriminant to the md result (e.g. a `Template.InnerWsh bool` …) populated from the actual tree nesting in `summarize`/`classifyPolicy`."* Task 3 Step 3 (line 65) maps *"`ScriptSh`+`PolicySortedMulti`+`InnerWsh→P2SH_P2WSH/SortedMulti`** vs **`ScriptSh`+`PolicySortedMulti`+`!InnerWsh→P2SH/SortedMulti`** … do NOT map `ScriptSh`+sortedmulti to P2SH_P2WSH unconditionally."* Task 3 Step 1 (line 62) adds the new test b2: bare `sh(sortedmulti)`→`P2SH` and `sh(wsh(sortedmulti))`→`P2SH_P2WSH`, *"assert they map to DIFFERENT scripts."*
**Source re-verification — the ambiguity is real:**
- `Template` still carries no nesting field: `md/md.go:1190-1197 type Template{N; Root ScriptKind; Policy PolicyKind; K,M; Keys; Renderable}`.
- `classifyPolicy` `tagSh` branch collapses BOTH into `(…, PolicySortedMulti)`: `md/md.go:1263-1278` — line 1267 `if inner.tag == tagWsh { … multiPolicy(gb.children[0]) }` (the `sh(wsh(sortedmulti))` path), and line 1274-1277 `// sh(multi/sortedmulti) legacy P2SH multisig … if pol,k,m,ok := multiPolicy(inner)` (the bare-P2SH path). Both return the same `PolicySortedMulti` and `rootScriptKind(tagSh)=ScriptSh` (`md/md.go:1224-1225`).
- Both targets are address-supported, so no late error saves a wrong mapping: `address/address.go:121-126` — `bip380.P2SH`→`NewAddressScriptHash`; `bip380.P2WSH, bip380.P2SH_P2WSH`→`NewAddressWitnessScriptHash`. `P2SH` and `P2SH_P2WSH` are distinct enum values (`bip380/bip380.go:60-61`) and render distinct addresses.
**Source re-verification — the discriminant IS derivable from the decoded tree (the decisive R1 question):**
- The decoded `*descriptor` tree fully retains the `sh → wsh → multi` nesting. Proof: `canonicalOrigin` (`md/md.go:1110-1118`) ALREADY walks exactly that structure today — `case tagSh:` → `inner := b.children[0]` → `if inner.tag == tagWsh { if gb,ok := inner.body.(childrenBody); … && isWshInnerMulti(gb.children[0].tag) }`. The same `inner.tag == tagWsh` test that `classifyPolicy` uses at `md/md.go:1267` is exactly the `InnerWsh` predicate. So populating `Template.InnerWsh` in `summarize`/`classifyPolicy` is a one-line read of already-present tree state — NOT under-specified.
- The chunked path retains the identical tree: `Reassemble` (`md/chunk.go:268`) calls the same `decodePayloadValidated(full, …)` that `Decode` uses (`md/md.go:1211`), returning the same `*descriptor`. So `DecodeChunks`→`summarize` produces the same `InnerWsh` faithfully.
**Assessment:** The fold picks R0's preferred option (b): surface a real discriminant from the tree, map `InnerWsh→P2SH_P2WSH` and `!InnerWsh→P2SH` (NOT display-only — bare `sh(sortedmulti)` is genuinely a renderable, address-supported P2SH multisig and can be faithfully verified, so refusing it would be an unnecessary capability loss; mapping it to `P2SH` is correct and the test b2 pins it). The discriminant is derivable, the mapping is precise, and the new test would catch a regression. CLOSED.

*(One non-blocking observation, NOT a finding: the plan's prose at line 62 hand-builds a bare-`sh(sortedmulti)` vector because the corpus lacks one. That vector decodes as renderable via the explicit-origin path — `canonicalOrigin` returns `(…, false)` for bare-sh (`md/md.go:1110-1119` has no bare-sh case), so `validateExplicitOriginRequired` (`md/md.go:1032-1063`) requires an explicit origin on the wire for it. The hand-built vector must therefore carry an explicit origin/path_decl, exactly as the implementer would discover at TDD-RED. Feasible and pinnable; the fold's "hand-build both" instruction is sufficient.)*

### I1 — empty `OriginPath` for canonical-default descriptors → wrong serialized key-origin → **CLOSED**
**Fold:** Task 2 Step 1 (line 48) and Step 3 (line 50) now state origin precedence as *"override (`d.tlv.originOverrides[idx]`) > `d.pathDecl` (Divergent[idx] / Shared) > **canonicalOrigin(d.tree) when both are empty**"* with the rationale spelled out (elided origin → empty `OriginPath` → `ExtendedKey()` serializes depth/childnum=0 → wrong displayed key-origin).
**Source re-verification:**
- `canonicalOrigin(d.tree) (originPath, bool)` exists and is reachable in package `md` (`md/md.go:1089`), and the validator accepts an elided origin precisely when it returns `true` (`md/md.go:1032-1034 if _, ok := canonicalOrigin(d.tree); ok { return nil }`). So a decoded descriptor CAN have empty `originOverrides` + empty `pathDecl` yet be valid — the exact case I1 targets.
- The damage path is confirmed: `bip380.Key.ExtendedKey()` (`bip380/bip380.go:97-108`) sets `depth = uint8(len(k.DerivationPath))` and `childNum = DerivationPath[last]` (0 when empty) → an empty path serializes depth=0/childnum=0, a wrong key-origin in the displayed/serialized descriptor.
- The precedence inputs all exist with the cited names: `tlvSection.originOverrides`/`originPresent` (`md/md.go:529-530`), `pathDecl{shared *originPath; divergent []originPath}` (`md/md.go:208-212`).
**Assessment:** The fold adds `canonicalOrigin(d.tree)` as the third tier, matching the validator's own acceptance rule. Note correctly that derivation parity is unaffected (derivation uses KeyData+ChainCode+Children only, `address/address.go:174-214`); the fix is about the displayed key-origin. The fold makes this a stated, tested decision rather than a silent omission. CLOSED.

### I2 — `OriginHardened []bool` wrong shape; `<a;b>` with `b≠a+1` not early-rejected → **CLOSED**
**Fold:** Task 2 Step 1 (line 48) now specifies `ExpandedKey{… OriginPath bip32.Path …}` and explicitly *"DROP the `OriginHardened []bool` field — it's the wrong shape"*, with `OriginPath` a `bip32.Path` *"with hardening IN-BAND as value + hdkeychain.HardenedKeyStart, `bip32/bip32.go:18`."* Task 2 Step 3 (line 50) repeats the in-band-hardening requirement and the no-parallel-bool rule. Task 3 Step 3 (line 67) adds *"`expandUnsupported` on: hardened wildcard, hardened multipath alt (D5), OR a multipath range with `End != Index+1`** (R0-I2 — `address.derivePubKey` only supports `End==Index+1`, `address/address.go:196-198`)."* Task 3 Step 1 (line 62) adds test e2: an exotic `<a;b>/*` with `b≠a+1` → `expandUnsupported`.
**Source re-verification:**
- `bip32.Path = []uint32` with in-band hardening: `bip32/bip32.go:18 type Path []uint32`; `Path.String()` masks `>= hdkeychain.HardenedKeyStart` (`bip32/bip32.go:26-31`); every standard path in `bip380.Script.DerivationPath()` is built as `hdkeychain.HardenedKeyStart + N` (`bip380/bip380.go:124-165`). So there is no parallel `[]bool` — dropping `OriginHardened` is correct.
- The decoder component to fold: `md/md.go:172-175 type pathComponent{hardened bool; value uint32}` → fold to `value + hdkeychain.HardenedKeyStart` when `hardened`. Sound.
- `bip380.Key.DerivationPath` is `bip32.Path` (`bip380/bip380.go:31`), so the projection feeds it directly.
- The `End != Index+1` constraint is real and late-failing if not pre-rejected: `address/address.go:196-198 case bip380.RangeDerivation: if c.End != c.Index+1 { return nil, errors.New("unsupported range path element") }` — this fires inside `derivePubKey`, i.e. at verify time (after the descriptor is already built and routed into `DescriptorScreen`), so early-rejecting it in `expandedToDescriptor` to `expandUnsupported` is the correct placement (matches D5's "don't let derivePubKey fail late"). `bip380.Derivation{Index, End, Hardened, Type}` (`bip380/bip380.go:38-46`) has the fields needed to detect `End != Index+1`.
**Assessment:** Both halves closed: field dropped + in-band fold specified (T2/T3), and the exotic-range early-reject specified with a test (T3.e2). CLOSED.

### Minors
- **M1 (stale route line) — FOLDED.** Task 5 (lines 88/92) now says *"the `errors.Is(err, md.ErrChunkedUnsupported)` arm — ~`:1975-1985`; anchor on the CONDITION, not the line number"*. Verified the live arm is `gui/gui.go:1975-1976` (`case errors.Is(err, md.ErrChunkedUnsupported): showError(…, "Multi-part descriptor — not yet supported.")`). The citation is now correct and condition-anchored.
- **M2 (dead sh-wpkh branch) — FOLDED.** Task 3 Step 3 (line 65) now states *"there is NO `ScriptSh`+singlesig (`P2SH_P2WPKH`) case — `classifyPolicy` never renders sh-wpkh on the Go side; omit that dead branch."* Verified: `classifyPolicy` (`md/md.go:1237-1281`) has no path that returns `PolicySingle` under `tagSh` — `tagSh` only reaches `multiPolicy` (`md/md.go:1275`), which returns only `PolicyMulti`/`PolicySortedMulti`/`PolicyComplex` (`md/md.go:1283-1293`). So sh-wpkh/sh-pkh classify `PolicyComplex`→`!Renderable`→display-only; the mapping branch is correctly removed.
- **M3 / M4** — R0 ruled these harmless redundancy / low-risk wording; the fold's D2 exclusion list (line 65) still lists `PolicyMultiA`/`PolicySortedMultiA` (never produced for a renderable Template per `md/md.go:1283-1293`) and the precedence wording is now tier-correct (`override > d.pathDecl (Divergent[idx]/Shared) > canonicalOrigin`, matching the `shared XOR divergent` semantics at `md/md.go:210-211`). No action.

---

## DRIFT-CHECK ON WHAT R0 RULED CLEAN (still holds after the fold)

- **D1 mainnet-only — still GREEN-acceptable.** Plan line 15 / Task 3 line 67 `bip380.Key.Network = &chaincfg.MainNetParams`. Faithful to Rust hardcoded `Main` (`derive.rs:57`, per R0). `bip380.Key.Network *chaincfg.Params` field present (`bip380/bip380.go:29`). No drift.
- **D3 no-pubkeys → template-only.** Plan line 17 / Task 3 line 66 `If any !XpubPresent → expandTemplateOnly`. `tlvSection.pubPresent` exists (`md/md.go:528`). No drift.
- **D4 secp256k1 in `md.validateXpubBytes`.** Task 1 (lines 30-38) unchanged: `secp256k1.ParsePubKey(p.xpub[32:65])`. Verified the validator is still a no-op today (`md/md.go:1071-1077`) on the SHARED decode path (`decodePayloadValidated` calls it at `md/md.go:1148`, used by both `Decode` and `Reassemble`). The byte-split `xpub[32:65]`=pubkey is the correct slice (R0 ruling 3). No drift.
- **D5 hardened-wildcard reject.** Task 3 line 67 retains the hardened-wildcard + hardened-multipath-alt early reject; now also covers the `End != Index+1` range (I2). No drift.
- **D6 build-before-`DescriptorScreen` (alloc gate).** Plan line 20 / Task 4 line 80 (completion handler does reassemble→expand→build before `descriptorFlow`) / Task 6 line 102 (`TestAllocs` gate confirmation). No drift — the fold did not move the build off the completion handler.
- **xpub byte-split** `ChainCode=Xpub[0:32]` / `KeyData=Xpub[32:65]` — Task 3 line 67 unchanged and correct (R0 ruling 3).
- **Gather clone fidelity** — Task 4 line 80 clones `mk1Gatherer`/`mk1GatherFlow` with `mk.ParseHeader`→`md.ParseChunkHeader`, `!h.Chunked`→foreign guard. No drift.
- **`mdmkFlow` route** — Task 5 unchanged in mechanism (replace the single arm). No drift.
- **TDD order + executability** — every new type (`md.ExpandedKey` with `OriginPath bip32.Path`, `Template.InnerWsh`, `expandStatus`) is defined in its task before use; the cross-package error (C1) and the discriminant (C2) are now defined in T2 before T3/T4 consume them. No forward-reference left open.
- **No new top-level `program`** — md1 inspect still reached via `mdmkFlow`, no StartScreen enum lockstep. No drift.
- **New tests feasible/pinnable** — b2 (bare `sh(sortedmulti)` + `sh(wsh(sortedmulti))` → different scripts): discriminant derivable from tree (see C2), hand-built vectors decode via explicit-origin path; the two scripts (`P2SH` vs `P2SH_P2WSH`) are distinct enum values (`bip380/bip380.go:60-61`) producing different addresses (`address/address.go:122-126`) — pinnable. e2 (`<a;b>/*`, `b≠a+1` → `expandUnsupported`): the reject condition is a static `End != Index+1` check derivable in `useSiteToChildren`/`expandedToDescriptor` without reaching `derivePubKey` — pinnable.

---

## NEW ISSUES INTRODUCED BY THE FOLD

None. The fold added a `Template.InnerWsh`-class discriminant, a third origin tier, a field drop, two early-rejects, and two test cases — all additive and source-consistent. No type/precedence/route regression was introduced, and no previously-clean ruling was disturbed.

---

## CLEARANCE

**VERDICT: GREEN (0 Critical / 0 Important).** C1, C2, I1, I2 all CLOSED with source-grounded, internally-consistent fixes; Minors folded or correctly judged harmless; no fold-induced drift. The plan is **cleared for single-implementer TDD** in the worktree (Task 0 → Task 6), followed by the mandatory whole-diff adversarial execution review before merge. Reviewer modified nothing.
