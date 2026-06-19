# RECON (T5, agent a01d37bccb47bf61a, 2026-06-19) — device-state gap analysis (source-verified vs fork `e4ca173`)

Recon only; nothing implemented. Baseline builds green; mk/md tests pass.

## Headline finding (resolves the roadmap ambiguity)
**The roadmap's one-line T5 conflates two things, one already DONE.**
- "Multi-chunk reassembly + integrity" for a **single mk1 card** is **already shipped** (T2b read-side + T4 write-side). NOT residual T5 scope.
- The genuinely-missing piece is **guided BUNDLE sequencing = a MULTI-CARD set** (md1 descriptor + several mk1 key cards [+ ms1 placeholder], one wallet) — the device-side analogue of host `me bundle`. That cross-card orchestration **does not exist on the device today.**
→ T5's real residual = **BUNDLE = multiple distinct cards**, not the single multi-chunk card.

## 1. Inventory (what EXISTS)
### mk1 read/gather/reassembly (T2b) — COMPLETE for a single card
- `mk1Gatherer` `gui/mk1_inspect.go:48-83`: primes on first chunk (`total`,`setID`); rejects foreign (`!Chunked || ChunkSetID!=setID || TotalChunks!=total`→gatherForeign 65-67); dedups by ChunkIndex (gatherDup 68-70); `complete()`=`primed && len(set)==total` (75).
- `mk1GatherFlow` `gui/mk1_inspect.go:156-256`: own scanner goroutine, "Captured N of M" (235), foreign/dup feedback (225-231); complete→decodeGathered→mk.Decode (258-265). **Read-only: gathers to DISPLAY, never to engrave the set.**
- `mk.Decode`/`reassemble` `mk/mk.go:148-224`: slot-index by ChunkIndex, reject mixed/csid-mismatch/dup/missing (188-213), verify 4-byte cross-chunk SHA-256 (214-222, errCrossChunkHash).
- `mk.ParseHeader`/`parseHeaderSyms` `mk/mk.go:56-92`; symbols `codex32.MKDataSymbols` `codex32/mkdata.go:17-42`.
→ **mk1 read-side completeness/integrity for ONE card fully solved. Reuse.**

### mk1 engrave sequencing (T4) — single card, N plates; NO bundle notion
- `multiPlateEngrave(ctx,th,strs)` `gui/derive_xpub.go:263-293`: loops N chunk strings of ONE card "Plate i of N" (273-277), per-plate variant picker + NewEngraveScreen.Engrave (286). Set-level abort `abortWarning` (283,298-302).
- Producer `mk.Encode`/`encodeChunks` `mk/encode.go:38-44,235-269`: deterministic csid=top-20 SHA-256 (316-319), ≥2 chunks, per-chunk BCH. **Only multi-string ENCODER in the fork.**
- Wired ONLY into `deriveXpubFlow` (`gui/derive_xpub.go:162`); strictly one card at a time, no "next card."

### md1 handling (T2c) — single-string decode ONLY; chunked REFUSED, not parsed
- `md.Decode` `md/md.go:1202-1216`: detects chunking by bit 0 of symbol 0 (1207)→`ErrChunkedUnsupported` (15-18). **NO md1 chunk-header parse, NO ChunkSetID/Total/Index, NO reassembly, NO md Encode** (grep confirmed empty).
- GUI `gui/gui.go:1971-1979` (mdmkFlow): chunked md1→"Multi-part descriptor — not yet supported." Inspect single-template-only `gui/md1_inspect.go:77-133`.
- Deferred as ledger **#10** (`md/md.go:17`).

### Scan path & set notion
- `scanner.Scan` `gui/scan.go:28-81`: single-object recognizer; md1/mk1→mdmkText (72-73). **No accumulation, no set.**
- `engraveObjectFlow` `gui/gui.go:1875-1891`: routes one object to one per-card flow. No multi-object case.
- `mdmkFlow` `gui/gui.go:1939-1991`: ENGRAVE path engraves the **single scanned str verbatim** (1983-1989). Full-set gather (mk1GatherFlow 1967) invoked ONLY for "Inspect key" display — **a scanned mk1 chunk does NOT trigger multi-plate set engraving today.**
- **No bundle/multi-card data model anywhere** (grep zero non-test hits). No T5 FOLLOWUPS entry yet.

## 2. The GAP (what T5 must add)
T5 targets **a BUNDLE = multiple DISTINCT cards** (md1 descriptor + N mk1 keys [+ ms1 placeholder]). Missing:
1. **Bundle data model** — none. Track which cards belong to one wallet, each card's completeness (a chunk set), which plates engraved-vs-pending across the WHOLE set. Host manifest (`DESIGN_me_bundle_preview.md:35-44`) is the template.
2. **Cross-card accumulation/grouping** — scan collects ONE object; need "group scanned strings by csid, distinguish md1-set vs mk1-set, track multiple sets." mk1Gatherer groups within ONE set and rejects foreign csid as ERROR — for a bundle a foreign csid is a DIFFERENT card to track. Net-new.
3. **"Confirm whole set complete before engraving + track plates done" UX** — none. T4 tracks plates within one card; no across-card progress, no bundle completeness gate.
4. **Bundle entry point / program** — none.

**Reuse wholesale:** mk1Gatherer/mk1GatherFlow, mk.Decode, multiPlateEngrave, scanner-shell idiom (mk1_inspect.go:162-208, verify_address.go:77-89), ChoiceScreen, showError, paged flows.

## 3. GUI surface
Bundle is multi-card/multi-step → doesn't fit single-scan dispatch. Two hooks:
- **(Recommended) New StartScreen program `engraveBundle`** between engraveXpub and qaProgram → the program-enum lockstep (~8 sites T4 touched, all gui.go): enum 148-150; dispatch switch 1489-1496; left-wrap 1630; right-wrap 1637; title switch 1655-1660; npage 1834; layoutMainPlates switch (panic if missed) 1842-1849; npages 1852-1853. Plus nav test `gui/derive_xpub_program_test.go` (hard-codes engraveXpub upper bound — needs update).
- (Alt) Extend mdmkFlow with "Add to bundle" — lighter lockstep but muddies per-card flow, no natural "done scanning, engrave all." Weaker.

## 4. The #10 boundary
**T5 can — and should — be scoped to NOT pull in #10.** Clean cut: T5 = bundle-set COMPLETENESS + guided cross-card sequencing on chunk HEADERS + per-card completeness, NOT md1 PAYLOAD. Device engraves md1 chunk strings VERBATIM (already does). Obstacle: md1 completeness still needs to PARSE the md1 chunk header (csid/total/index) + cross-chunk integrity — and md.Decode today only detects bit-0 and refuses. So md1-inclusive T5 needs a **thin read-side md1 chunk-header parser + reassembly-completeness check** (sibling to mk's, NOT the full payload decoder, NOT the encoder). [NOTE: per the companion bundle-protocol recon, md1 cross-chunk INTEGRITY (csid-vs-derived) DOES need the encoder; header-only completeness+consistency does not.]
Three options (cheapest first):
- **mk1-only** (cleanest, zero #10): bundle=N mk1 cards; reuse mk1Gatherer/mk.Decode/multiPlateEngrave end-to-end. Smallest, lowest risk.
- **mk1 + md1-header-completeness**: thin md1 chunk-header parse + reassembly-completeness (no payload decode, no encoder). Most faithful to host `me bundle`. Moderate.
- **Full md1 (decode+display descriptor)** = IS #10. Avoid in T5.

## 5. Effort / risk
Mostly orchestration glue + new GUI program + bundle data model, not new crypto.
**LOC (mk1-only):** model+grouping ~120-180; engraveBundle program+lockstep+nav test ~80-120; bundle gather/sequence flow ~200-300; tests ~250-350. **~650-950.** Option 2 (+md1 header) **+250-400** (~900-1350 total).
**Risks for spec to lock:** (1) **Bundle definition** — must state T5=multi-DISTINCT-card orchestration or risk re-implementing T2b. Resolve first. (2) **mk1-only vs md1-inclusive** — locks #10 contact. (3) **Bundle completeness semantics** — mk1 cards self-declare TotalChunks per set, but NOTHING declares how many distinct CARDS a wallet needs; device scans incrementally with no "expected card count" → must define the "done adding cards" affordance + whether md1 descriptor is required vs optional. (4) Set-level abort across cards (extends T4 R0-I3). (5) Lockstep drift (8 sites + nav test hard-coded bound). (6) No hardware to validate multi-card scan/engrave UX (highest-uncertainty UX in roadmap).

**Key files:** gui/mk1_inspect.go, gui/derive_xpub.go, mk/mk.go + mk/encode.go, md/md.go:1199-1216 (#10 boundary), gui/gui.go:145-151/1479-1508/1655-1660/1812-1853/1875-1991, gui/scan.go, design/DESIGN_me_bundle_preview.md.
