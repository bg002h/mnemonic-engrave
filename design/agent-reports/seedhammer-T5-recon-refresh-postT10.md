# RECON-REFRESH (T5, agent a171406b5a2b142d5, 2026-06-19) — post-#10 device state + bundle-completeness model (source-verified vs fork `bb0e506`)

Recon only. Supersedes the pre-#10 portions of `design/cycle-prep-recon-T5-bundle-sequencing.md`. All citations live at `bb0e506`.

**Headline:** #10 (10a+10b) landed and resolved the md-encoder gate — but added a full md1 DECODER + chunked gather + wallet-policy expansion + an INTERNAL re-encoder (for the integrity gate), NOT a public md1 string encoder. The device can decode/reassemble/verify/display a scanned chunked md1 set, but cannot PRODUCE md1 chunks. T5's residual (multi-DISTINCT-card orchestration) is unchanged and still entirely net-new. **md1 cards engrave VERBATIM from scanned chunk strings — no encoder on T5's path** (matches host `me bundle`, which carries the input string into each plate, `bundle.rs:243-267`).

## Job 1 — Post-#10 inventory
### Single-card primitives that EXIST (reusable)
**mk1 — read+write complete:** `mk1Gatherer` (`gui/mk1_inspect.go:48-83`, `complete()`=`primed&&len(set)==total`); `mk1GatherFlow→(mk.Card,bool)` (`:156`); `mk.Decode([]string)(Card,error)` reassemble+SHA-256[0:4] integrity (`mk/mk.go:148-224`); `mk.Card{Network,Path,Fingerprint,Stubs [][4]byte,Xpub(base58)}` (`mk/mk.go:132-139`); `mk.Encode(card)([]string,error)` the ONLY public string encoder in-tree (`mk/encode.go:38`); `multiPlateEngrave(ctx,th,strs)` "Plate i of N"+set-abort (`gui/derive_xpub.go:263-293`, wired only into `deriveXpubFlow:162`).
**md1 — read/verify ONLY:** `md1Gatherer` (`gui/md1_gather.go:23-63`); `md1GatherFlow(ctx,th,first)bool` (`:72-174`)→`gatheredDescriptorFlow` (`:189-210`)→`md.ExpandWalletPolicyChunks`→`expandedToDescriptor`→`descriptorFlow`/`md1DisplayFlow`/error; `md.Reassemble([]string)(*descriptor,error)` (`md/chunk.go:207`); `md.DecodeChunks`/`ExpandWalletPolicy`/`ExpandWalletPolicyChunks` (`md/expand.go:25,83,102`); `md.ParseChunkHeader` (`md/chunk.go:185`), `ChunkHeader{Version,Chunked,ChunkSetID,TotalChunks,ChunkIndex}` (`:49-55`).
**CORRECTION:** NO exported `md.Encode`/`md.split` — `encodeMD1String`/`encodePayload`/`split` unexported, take unexported `*descriptor` (`md/encode.go:451,373`; `md/chunk.go:121`). NO md1 engrave path exists (grep: zero Engrave/multiPlate in `gui/md1_gather.go`/`md1_inspect.go`). The #10 encoder exists solely to recompute the derived csid in the integrity gate.

### Net-new for T5 (confirmed absent — grep bundle/cardSet/walletSet = 0 non-test hits)
Bundle data model (multiple distinct cards, each a chunk-set); cross-card accumulation/grouping; "whole-set complete?" gate; plate-tracking across cards; bundle entry point. ALSO net-new: an **md1 verbatim multi-plate engrave** — `multiPlateEngrave` mechanism (engrave list of strings as N plates via `validateMdmk` `gui/gui.go:1897`) is format-agnostic + reusable, but its copy is mk1-specific ("Account Xpub" etc.) → generalize.

### GUI program + lockstep (current lines)
`program` enum `{backupWallet,engraveXpub,qaProgram}` `gui/gui.go:147-151` (qaProgram debug-only, not navigable). A new `engraveBundle` program touches the 8 T4-lockstep sites: (1) enum `:147-151`; (2) dispatch switch `:1489-1502`; (3) left-wrap `:1628-1631`; (4) right-wrap `:1636-1639`; (5) title switch `:1655-1660`; (6) `npage` const `:1834`; (7) `layoutMainPlates` switch — **panics on missing case** `:1842-1850`; (8) `layoutMainPager` npages `:1852-1853`. Nav-test hard-codes the navigable upper bound at `engraveXpub` (`gui/derive_xpub_program_test.go:30-31`) — MUST update. `mdmkFlow` (`gui/gui.go:1939-1994`) is strictly one-card → T5 bundle loop is a SIBLING flow, not an extension.

## Job 2 — Bundle-completeness model
### Option B feasibility (md1 @N keys ↔ mk1 cards): TECHNICALLY FEASIBLE, conditional, NOT a reliable gate alone.
- Descriptor side `md.ExpandedKey` (`md/expand.go:56-64`): `Index, OriginPath, UseSite, Fingerprint [4]byte+present, Xpub [65]byte(=32B chaincode‖33B pubkey)+present`. **xpub TLV is SPARSE/OPTIONAL** (`md/md.go:509-512,744-779`; gated `md/expand.go:208-218`) — a descriptor can carry ZERO xpubs (→template-only, `gui/md1_expand.go:42-49`). Wire stores only chaincode+pubkey; parentFP/depth/childnum NOT on wire (`expandedToDescriptor` hardcodes ParentFingerprint:0 `gui/md1_expand.go:65-67`).
- Key side `mk.Card` (`mk/mk.go:132-139`): `Path string`, `Fingerprint string`(8-hex origin/master, may be ""), `Xpub string`(base58). base58 reconstructed from version(4)‖parentFP(4)‖chainCode(32)‖pubKey(33), depth=len(comps), childNum=last comp (`mk/mk.go:378-404`).
- **Comparable invariant = (32B chaincode ‖ 33B compressed pubkey)** — same EC point+chaincode on both sides. Must base58-decode mk1's Xpub + extract chaincode+pubkey, compare to ExpandedKey.Xpub[0:32]/[32:65]. Exact+reliable WHEN both present. NOT the same serialization (md1 zeroes parentFP/depth/childnum; mk1 carries them) → naive string/65-byte compare FALSE-mismatches; MUST normalize to chaincode+pubkey only. Master-fp also comparable but both optional → unreliable alone.
- **Hard precondition:** Option B's "this wallet needs N keys + here are their xpubs" signal exists only if the md1 carries per-@N xpubs. A pubkey-less descriptor gives `tpl.N` (count) but no key material to match → "N cosigners" but can't verify WHICH mk1 belongs. Whether real md1 cards embed all xpubs is open (a multisig-backup descriptor typically would). NO cross-match helper exists today (grep-confirmed).

### Host `me bundle` model (`crates/me-cli/src/bundle.rs`)
Groups input strings by chunk_set_id into BTreeMaps (`:200-223`), verifies EACH chunk-set's own completeness/consistency/integrity via `mk_codec::decode`(`:273`)/`md_codec::chunk::reassemble`(`:246`). **NO notion of "all the cards a wallet needs," NO md1↔mk1 cross-match, NO expected-cosigner-count.** Refuses ms1 up front (`:188-192`), appends a trailing ms1 reminder plate (`:296-306`). Engraves md1 VERBATIM (carries input string into each PlateEntry, `:243-267`). **⇒ host me bundle IS Option A in spirit.**

### RECOMMENDATION: Option A (operator-driven), Option B hybrid documented/deferred.
Option A: scan cards one at a time (each card's chunk-set auto-completeness via the gatherers); operator hits "done adding cards"; device engraves/confirms all accumulated cards. **Mirrors the shipped host tool exactly** (the user's "mirror m* / bundle behavior" directive → Option A is the faithful mirror); reuses every primitive; zero dependency on md1 carrying xpubs; works for every wallet; NO encoder needed (md1 verbatim). Tradeoff: can't say "missing cosigner #2" or catch a wrong-wallet mk1 — trusts the operator's set.
Hybrid (DEFER as follow-on, additive on an Option-A core): if md1 scanned first AND carries xpubs, show "N cosigners" (from tpl.N) + tick off each mk1 by normalized chaincode+pubkey match → auto "all N matched" gate. Only place Option B adds value; conditional + brand-new normalization/match code + hardware-untestable UX → out of T5 core.

## Open questions / risks for the T5 spec to lock
1. Bundle = multi-DISTINCT-card orchestration, NOT single multi-chunk card (already shipped). Don't re-implement gatherers.
2. md1 engrave net-new + VERBATIM (no encoder); generalize `multiPlateEngrave`'s mk1-specific copy.
3. "Done adding cards" affordance — the central Option-A completeness gate (nothing on wire declares card count).
4. md1 descriptor required vs optional in the bundle (Option A → optional, just another card; decide if any count/guidance wanted).
5. Set-level abort across cards — extend `abortWarning` (`gui/derive_xpub.go:298-302`) to partial-bundle; record no completed state.
6. Program-lockstep drift — 8 `gui.go` sites + nav-test bound (`derive_xpub_program_test.go:30-31`); missed `layoutMainPlates` case panics.
7. ms1 spine — refuse ms1 in bundle/NFC channel (hand-typed only); append ms1 reminder (mirror `bundle.rs:296-306`).
8. NO hardware to validate incremental multi-card scan/engrave UX — highest-uncertainty.

**Sizing:** new `engraveBundle` program + bundle data model + cross-card grouping + gather/sequence flow + generalized verbatim engrave. Fork-side only. ~lower than the prior "Option 1" ~650-950 LOC since md1 read path is done. `SPEC_seedhammer_T5_*` MUST pass opus R0 to 0C/0I before code.
