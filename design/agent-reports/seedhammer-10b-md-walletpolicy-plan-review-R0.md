# R0 GATE REVIEW — IMPLEMENTATION PLAN #10b (md1 wallet-policy display + verify)

**Reviewer:** opus architect (plan R0 gate, pre-implementation). **Date:** 2026-06-19.
**Plan under review:** `design/IMPLEMENTATION_PLAN_seedhammer_10b_md_walletpolicy.md`
**Sources verified against:** GO fork `seedhammer` @ `3a55ae5` (#10a merged — confirmed via `git rev-parse HEAD`); RUST `descriptor-mnemonic/crates/md-codec` @ `c85cd49` (v0.36.0, confirmed).
**Posture:** adversarial; every protocol/wire/API fact independently checked at `file:line`, not taken from the plan's or recon's prose.

---

## VERDICT: NOT GREEN

**2 Critical, 2 Important.** The plan is ~90% sound — D1, D4, D5, D6, the xpub byte-split direction, the gather clone, and the route change all verify clean. But two findings would make a faithful single-implementer TDD pass produce either a non-compiling build (C1) or a descriptor that **verifies against the wrong address** (C2 — a direct I-6 / §4 "no mis-rendering of spend paths" violation, the exact failure class the spec's faithful-or-refuse discipline exists to prevent). Fold, persist, re-dispatch.

---

## CRITICAL

### C1 — Task 4 references the unexported `md.errChunkSetIDMismatch` from package `gui`; it will not compile.
**Plan location:** Task 4, Step 3 (line 80): *"on `errChunkSetIDMismatch`→`showError(...,"Chunks don't match…")`; on other err→…"*.
**Source:** `md/chunk.go:13-22` — every chunk-set error is **unexported**:
```
errChunkSetEmpty, errChunkSetInconsist, errChunkSetIncomplete,
errChunkIndexGap, errChunkSetIDMismatch
```
The **only** exported md error is `md/md.go:18 var ErrChunkedUnsupported`. The completion handler in Task 4 lives in package `gui` and cannot name `md.errChunkSetIDMismatch` (lowercase → package-private). The plan gives no mechanism to surface it across the package boundary, so the csid-mismatch DISTINCT-error UX (an explicit acceptance item, line 113) is unbuildable as written.
**Fix:** In Task 2, when adding `md.DecodeChunks`, **export** the integrity-mismatch error as `md.ErrChunkSetIDMismatch` (rename the var at `md/chunk.go:22` and update its in-package uses + `md/chunk_test.go:247-248`), and have `DecodeChunks` return it (or `fmt.Errorf("…: %w", ErrChunkSetIDMismatch)`). Then Task 4 dispatches with `errors.Is(err, md.ErrChunkSetIDMismatch)`. (Optionally also export `ErrChunkSetIncomplete`/`ErrChunkSetInconsist` if the UX wants to distinguish "incomplete" from "tampered"; at minimum the integrity-mismatch one is load-bearing for the locked DISTINCT-UX acceptance criterion.) Add an explicit Task-2 sub-step for the rename so the re-export is test-covered.

### C2 — `(ScriptSh, PolicySortedMulti)` is ambiguous between `sh(sortedmulti)` (P2SH) and `sh(wsh(sortedmulti))` (P2SH_P2WSH); the Template carries no nesting, so the Task 3 mapping cannot disambiguate and would verify against a WRONG address.
**Plan location:** Task 3, Step 3 (line 65): *"`ScriptSh`+wsh-sortedmulti→`P2SH_P2WSH/SortedMulti`"* — but `expandedToDescriptor(tpl md.Template, keys …)` (line 60) operates on the **Template only**.
**Source (decisive):**
- `md/md.go:1190-1197 type Template{N; Root ScriptKind; Policy PolicyKind; K,M; Keys; Renderable}` — **no nesting / no inner-script field.**
- `md/md.go:1263-1278 classifyPolicy` `tagSh` branch: tries `sh(wsh(multi/sortedmulti))` first (1267-1273), then **falls through to bare `sh(multi/sortedmulti)` legacy P2SH** (1274-1277). **Both return `(ScriptSh, PolicySortedMulti)`** — indistinguishable in the Template.
- Both are real, reachable, renderable md1 shapes (Rust confirms the legacy variant: `canonical_origin.rs:223,229` "sh(sortedmulti) — legacy P2SH multi, not nested in wsh"; distinct canonical origins `m/48'/0'/0'/1'` for nested vs `m/45'` for bare in `canonicalOrigin` `md/md.go` — note the Go `canonicalOrigin` only emits `48'…1'` for `sh(wsh)` and has no bare-sh case, but the shape still decodes + classifies as renderable via the explicit-origin path).
- The address layer supports BOTH targets (`address/address.go:121-126`: `bip380.P2SH`→`NewAddressScriptHash`; `bip380.P2WSH, bip380.P2SH_P2WSH`→witness-script-hash), so a wrong mapping silently produces a structurally valid but WRONG address — no late error to save you.

If the implementer follows line 65 literally and maps `(ScriptSh, PolicySortedMulti)`→`P2SH_P2WSH`, then a bare `sh(sortedmulti)` set verifies the operator's funds against a nested-segwit address that the descriptor does not control. This is precisely the I-6 / spec §4.1 "never silently verify against a wrong address" hazard.
**Fix (pick one, must be explicit in the plan):**
- **(a) Preferred — refuse the ambiguous case.** Since the locked D2 expressible set is *"singlesig + `wsh(sortedmulti)` + `sh(wsh(sortedmulti))`"* and **bare `sh(sortedmulti)` is NOT in it**, `expandedToDescriptor` cannot faithfully build `(ScriptSh, PolicySortedMulti)` from the Template alone → map it to `expandUnsupported` (display-only). But to do that *correctly* you still must distinguish nested from bare — which the Template can't — so option (a) alone forces ALL `(ScriptSh, PolicySortedMulti)` to display-only, dropping the in-scope `sh(wsh(sortedmulti))`.
- **(b) Correct — give the projection the structure it needs.** Have `expandedToDescriptor` (or a new `md`-side helper) consult the **decoded `*descriptor` tree** (or have `summarize`/the expansion surface an explicit `Nested bool` / inner-script discriminant on the Template) so `sh(wsh(sortedmulti))`→`P2SH_P2WSH` and bare `sh(sortedmulti)`→`expandUnsupported` (out of D2 scope) are separable. Given Task 3 already takes `tpl md.Template`, the cleanest is to extend the NET-NEW `md` surface (Task 2) to carry the wrapper discriminant, since the tree is unexported and `gui` cannot inspect it.
Either way, **add a Task 3 test** for a bare-`sh(sortedmulti)` golden asserting it does NOT become `expandOK`/`P2SH_P2WSH`. The current Task 3 corpus (line 62) has no bare-sh-sortedmulti case, so TDD would not catch this regression.

---

## IMPORTANT

### I1 — `ExpandWalletPolicy` omits the `MissingExplicitOrigin` / canonical-origin handling; structured `OriginPath` will be empty for canonical-default descriptors, producing a wrong serialized key-origin (depth/childnum).
**Plan location:** Task 2, Step 3 (line 50): *"resolving structured origin (from `d.tlv.originOverrides` / `d.pathDecl`)"* — full stop. No mention of the empty-baseline / canonical-origin fallback.
**Source:** Rust `expand_per_at_n` (`canonicalize.rs:437-455`) resolves `origin_path = override ? baseline(shared|divergent[idx])` and **gates** on `MissingExplicitOrigin` when baseline is empty AND no override AND `canonical_origin(&d.tree).is_none()`. The Go decoder ALREADY enforces this at decode time (`md/md.go:1032-1064 validateExplicitOriginRequired`, called from `decodePayloadValidated`), so a *decoded* descriptor can't raise it — **but** that gate *passes* when the baseline is empty and a `canonicalOrigin(d.tree)` default exists (`md/md.go:1089-1120`, e.g. `wpkh`→`m/84'/0'/0'`, `wsh(sortedmulti)`→`m/48'/0'/0'/2'`). In that case the wire carries NO origin, `d.tlv.originOverrides`/`d.pathDecl` are empty, and a literal "structured origin from those fields" yields an **empty** `OriginPath`.
Consequence: `bip380.Key.DerivationPath` empty → `ExtendedKey()` (`bip380/bip380.go:100-107`) serializes `depth=0, childNum=0` → the displayed/serialized xpub key-origin is wrong for every canonical-default descriptor. (Address *derivation* itself only uses KeyData+ChainCode+Children — `address/address.go:174-214` — so `address.Find` still verifies correctly; the damage is confined to the displayed key origin / descriptor string. Rust's `xpub_from_tlv_bytes` also uses placeholder depth/childnum at `derive.rs:58-60`, so derivation parity holds.)
**Fix:** State in Task 2 Step 3 that when the resolved baseline+override origin is empty, `ExpandWalletPolicy` must substitute `canonicalOrigin(d.tree)` (the already-shipped helper at `md/md.go:1089`) for the structured `OriginPath` — matching the validator's own acceptance rule — so the displayed key origin is the canonical `m/84'/0'/0'`-class path rather than bare `m`. (This also fixes a latent #10a display defect where `originPathStringFor` returns `"m"` for the canonical case, but that is out of #10b scope; the #10b accessor must not propagate it.) If the team decides display-of-empty-origin matches Rust closely enough, that is a defensible product call — but it must be a *stated, tested* decision, not a silent omission.

### I2 — The `ExpandedKey.OriginHardened []bool` field is the wrong shape: `bip32.Path` encodes hardening in-band; Task 3 must fold `value | HardenedKeyStart`, and exotic `<a;b>` use-sites with `b != a+1` are not early-rejected.
**Plan location:** Task 2 Step 3 (line 50) proposes `ExpandedKey{… OriginPath bip32.Path; OriginHardened []bool …}` "if `bip32.Path` doesn't encode hardening"; Self-review (line 120) defers the question to the implementer.
**Source (resolves the deferred question authoritatively):** `bip32/bip32.go:18 type Path []uint32`; hardening is encoded in-band as `value + hdkeychain.HardenedKeyStart` (= +0x80000000), proven by `bip380/bip380.go:124-165` building every standard path that way and `bip32/bip32.go:26-30,72,107` masking it off. The md decoder's component is `md/md.go:172 pathComponent{hardened bool; value uint32}`. So **there is no parallel `[]bool` in the consumer** — the `OriginHardened []bool` field is redundant/mis-shaped; the implementer must fold each component to `value | hdkeychain.HardenedKeyStart` when `hardened` and build a single `bip32.Path`. The plan's "implementer must check `bip32` and adapt" is acceptable as a *deferral*, but since R0 is now resolving it: **drop `OriginHardened []bool`; specify the offset-fold in Task 2/Task 3.**
**Secondary (same finding):** Task 3 (line 67) maps `<a;b>/*`→`RangeDerivation{Index:a,End:b}` and notes `address.derivePubKey requires End==Index+1` (`address/address.go:196-198`) but does NOT early-reject `b != a+1`. Rust's `use_site_to_derivation_path` (`to_miniscript.rs:116-130`) accepts arbitrary `multipath[chain]` values, so an exotic `<0;2>/*` would build `expandOK`, route into `DescriptorScreen`, and only fail when the operator attempts a verify — a late, confusing failure rather than the intended display-only. **Fix:** in `useSiteToChildren`/`expandedToDescriptor`, treat a range whose `End != Index+1` (or alt_count != 2) as `expandUnsupported`, consistent with D5's "don't let `derivePubKey` fail late" rationale. Add a Task 3 test case.

---

## MINORS (non-blocking; fold opportunistically)

- **M1 — Route-point line citations are stale.** The actual `ErrChunkedUnsupported` arm is `gui/gui.go:1975-1976` (verified). The plan (Task 5, lines 88/92) says `:1984-1985`; the recon says `:1979-1987`/`:1984-1985`; the spec says `:1971-1979`. All off by 9-11 lines. Harmless because the plan anchors on the `errors.Is(err, md.ErrChunkedUnsupported)` string, but correct the citation.
- **M2 — D2 over-claims `sh-wpkh` as expressible on the Go side.** Rust supports `sh(wpkh)` (`to_miniscript.rs:239 new_sh_wpkh`, vector `sh_wpkh_nested`), but the Go `classifyPolicy` `tagSh` branch (`md/md.go:1263-1278`) never renders `sh(wpkh)`/`sh(pkh)` singlesig → they classify `PolicyComplex` → `Renderable=false` → display-only. So the plan's `ScriptSh+singlesig→P2SH_P2WPKH` mapping branch (line 65) is **dead code** (unreachable from a Go-decoded Template), and D2's "sh-wpkh" expressible claim is not deliverable in #10b. Not a bug (fails safe to display-only, and Task 3.b's corpus correctly omits sh-wpkh), but note it so the implementer doesn't waste effort wiring an unreachable branch.
- **M3 — D2 exclusion list lists already-excluded policies.** `classifyPolicy` can only return `PolicySingle`/`PolicyMulti`/`PolicySortedMulti`/`PolicyComplex` (`multiPolicy` handles only `tagMulti`/`tagSortedMulti`, `md/md.go:1283-1293`). `PolicyMultiA`/`PolicySortedMultiA` are **never produced** for a renderable Template — they fall to `PolicyComplex`→`!Renderable`. So the only load-bearing renderable-but-not-bip380 exclusion is **`PolicyMulti` (unsorted)**, which the plan correctly handles. Listing `PolicyMultiA`/`PolicySortedMultiA` in the exclusion (line 65) is harmless redundancy.
- **M4 — Precedence wording.** Task 2 (line 48) writes origin precedence as "override>divergent[idx]>shared", implying a 3-tier fallback. `pathDecl{shared *originPath; divergent []originPath}` are mutually exclusive ("set iff !divergent", `md/md.go:208-212`), matching Rust's `PathDeclPaths::Shared|Divergent` enum. Correct semantics is `override > (shared XOR divergent[idx])`. Reword to avoid an implementer building a spurious 3-level chain; the cited mirror (`canonicalize.rs:437-444` / `originPathStringFor`) is correct, so risk is low.

---

## RULINGS ON THE EXPLICITLY-REQUESTED QUESTIONS

1. **D1 mainnet-only — GREEN-acceptable.** md1 carries no network on the wire and Rust hardcodes `NetworkKind::Main` (verified `derive.rs:57`). `bip380.Key.Network = &chaincfg.MainNetParams` is faithful. A testnet picker is not required for GREEN — it is a clean, isolated follow-up (single field), and forcing a picker now would *diverge* from the m\* constellation. Acceptable as locked; log the testnet-picker seam to FOLLOWUPS.

2. **D2 subset mapping — INCOMPLETE / unsafe as written → C2 + M2/M3.** The expressible set is right in principle, but (C2) the `(ScriptSh, PolicySortedMulti)` Template ambiguity makes the `sh(wsh(sortedmulti))`→P2SH_P2WSH mapping unsafe, and (M2) the `sh(wpkh)` branch is unreachable on Go. Must fix C2 before GREEN.

3. **xpub byte-split direction — CORRECT.** Verified authoritative: `chain_code = bytes[0..32]`, `public_key = bytes[32..65]` (`derive.rs:50-55`). Plan's `ChainCode:Xpub[0:32], KeyData:Xpub[32:65]` (line 67) is the right direction; the secp256k1 check is on `xpub[32:65]` (`validate.rs:221`), matching the plan's Task 1 `secp256k1.ParsePubKey(p.xpub[32:65])` (line 36). No off-by-one.

4. **`bip32.Path` hardening — RESOLVED (in-band, no parallel bool) → I2.** `bip32.Path = []uint32` with hardening as `value + HardenedKeyStart` (`bip32/bip32.go:18`, `bip380/bip380.go:124-165`). The `OriginHardened []bool` field is unnecessary; the implementer must fold the offset. The plan's deferral is acceptable but R0 now resolves it concretely (see I2).

---

## WHAT VERIFIED CLEAN (no action)

- **#10a substrate present + correct:** fork HEAD `3a55ae5`; `md.Reassemble` returns unexported `*descriptor` (`md/chunk.go:196`); `md.ParseChunkHeader` returns `md.ChunkHeader{Version,Chunked,ChunkSetID,TotalChunks,ChunkIndex}` field-compatible with `mk.Header` (`md/chunk.go:38-44,174-189`); the `syms[0]&1` bit-0 discriminator is consulted first (`md/chunk.go:182-185`).
- **D4 placement faithful:** `validateXpubBytes` is a no-op (`md/md.go:1071-1077`), `errInvalidXpubBytes` exists (`md/md.go:898`), and it is invoked by `decodePayloadValidated` (`md/md.go:1148`) — the SHARED path for both `Decode` and `Reassemble` (`md/chunk.go:268`), so the new secp256k1 check correctly gates the chunked path too. Rust has the symmetric `validate_xpub_bytes` validator (`validate.rs:216-226`). The `secp256k1/v4` dep is in-tree (used by `address`/`bip32`). Chain-code prefix intentionally unvalidated — matches Rust (`validate.rs:211-212`).
- **D5 faithful:** Rust rejects hardened wildcard (`derive.rs:99-101`) and hardened multipath alt (`to_miniscript.rs:126-128`) with `HardenedPublicDerivation`. Early-reject in `expandedToDescriptor` is the right placement.
- **D6 required + correct:** `DescriptorScreen.Confirm` hoists `address.Supported(s.Descriptor)`→`Receive(desc,0)` (secp256k1, allocating) out of the frame loop (`gui/gui.go:2369-2372`). The reassemble→expand→build MUST complete before entering the screen — the gather-completion-handler placement (Task 4) satisfies this. `TestAllocs` (`BenchmarkAllocs`, `gui/gui_test.go`) exercises only `StartScreen.Flow`+`DescriptorScreen.Confirm`, so gather/build is off the alloc-gated path.
- **Gather clone faithful:** `mk1Gatherer` (`gui/mk1_inspect.go:48-83`) + `mk1GatherFlow` (`:156-256`) are a clean clone source; the `!h.Chunked`→foreign guard (`:65`) and `offer/complete/collected` shape port verbatim with `mk.ParseHeader`→`md.ParseChunkHeader`.
- **Route + no-regression:** the single-arm change at the `ErrChunkedUnsupported` case leaves `err==nil`→`md1DisplayFlow` (`gui/gui.go:1974`) and the default arm unchanged; single-md1/mk1/ms1 paths untouched. No new `program` enum (md1 inspect reached via `mdmkFlow`, not a StartScreen program) → no 6/8-site lockstep.
- **Reused signatures all match:** `descriptorFlow(ctx,th,desc)` (`gui/gui.go:2058`), `verifyAddressFlow(ctx,th,desc)` (`gui/verify_address.go:22`), `md1DisplayFlow(ctx,th,tpl)` (`gui/md1_inspect.go:77`), `showError(ctx,th,title,msg)` (`gui/slip39_polish.go:22`).
- **Task 3.a round-trip feasible:** `address.Receive`/`Find`/`Supported`/`Change` all exist (`address/address.go:20-53`); `address` supports `SortedMulti` for P2SH/P2WSH/P2SH_P2WSH and `Singlesig` for P2PKH/P2WPKH/P2SH_P2WPKH/P2TR (`address/address.go:96-158`); standard `<0;1>/*`→`Range{0,1}` satisfies the `End==Index+1` constraint (`:196-198`).
- **`ExpandWalletPolicy` access:** lives in package `md`, so the unexported `idxPub{idx,xpub[65]byte}` (`md/md.go:508-511`), `tlvSection` (`:522-532`), `pathDecl`/`useSitePath`, `canonicalOrigin` (`:1089`) are all reachable. (Contrast C1: the *gui* side cannot reach unexported `md` errors.)
- **TDD order / executability:** tasks are bite-sized, test-first, with pinned golden expected values from the vendored corpus; no type is referenced before its defining task except the C1 cross-package error and the C2/I1/I2 underspecs noted above.

---

## REQUIRED TO REACH GREEN
Fold **C1** (export the integrity-mismatch error + Task-2 sub-step + test) and **C2** (disambiguate or refuse `(ScriptSh, PolicySortedMulti)`; add the bare-sh test). Fold **I1** (canonical-origin substitution in `ExpandWalletPolicy`, stated+tested) and **I2** (drop `OriginHardened []bool`, specify the in-band offset fold; early-reject exotic `<a;b>` with `b≠a+1`). Minors M1-M4 may be folded opportunistically. Then re-dispatch this R0 (re-dispatch after EVERY fold per the project gate). No code before GREEN (0C/0I).
