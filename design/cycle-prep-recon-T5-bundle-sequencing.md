# cycle-prep recon — 2026-06-19 — T5 guided bundle sequencing

**Fork HEAD:** `e4ca173` (T4 merged + pushed bg002h). **Recon agents (parallel, source-verified):** `a01d37bccb47bf61a` (device-state gap, vs fork `e4ca173`), `aa63020b3ad9a1c13` (bundle-set protocol facts, vs `me-cli` v0.3.0 / `mk-codec` v0.4.0 / `md-codec` v0.36.0). Verbatim reports: `design/agent-reports/seedhammer-T5-recon-device-state.md`, `…-bundle-protocol.md`.

T5 roadmap line: "multi-chunk reassembly + integrity + guided bundle sequencing (M; needs T2): confirm a complete chunked md1/mk1 SET before engraving (device-side `me bundle`)."

## HEADLINE — the roadmap line conflates two things, one already DONE
- **"Multi-chunk reassembly + integrity" for a SINGLE card is already shipped:** mk1 read-side gather/reassembly/cross-chunk-hash (T2b: `mk1Gatherer`/`mk1GatherFlow`/`mk.Decode`) + mk1 write-side multi-plate engrave (T4: `multiPlateEngrave`). NOT residual T5 scope.
- **The genuine residual = guided BUNDLE sequencing across MULTIPLE DISTINCT cards** (an md1 descriptor card + N mk1 key cards [+ ms1 placeholder] from one wallet) — the device analogue of host `me bundle`. **No cross-card orchestration, no bundle data model, no bundle entry point exists on the device** (grep: zero non-test hits for bundle/cardSet/walletSet).

## THE GATING SCOPE DECISION (md1 ⇄ #10 entanglement) — needs a user call
The pivotal source fact (verified, `md-codec` v0.36.0 @ `c85cd49`):
- **mk1 set-verification (completeness + consistency + cross-chunk integrity) is header/stream-level — NEEDS NO ENCODER.** Cross-chunk integrity = recompute `SHA-256(bytecode)[0..4]` over the reassembled bytes (`mk-codec/string_layer/chunk.rs:195-201`); `reassemble_from_chunks` makes no `encode_*` call.
- **md1 set-INTEGRITY REQUIRES the full md DECODER + RE-ENCODER (the deferred #10 dependency).** `md-codec/chunk.rs:376` `decode_payload` → `:379` `compute_md1_encoding_id` (which re-runs `encode_payload`, `identity.rs:39-44`) → `:380-382` compare derived-csid to header-csid. No trailing-hash shortcut; md1 csid is *derived from the re-encoded payload* (`chunk.rs:175-179`), so the integrity comparison value cannot be reconstructed without re-encoding.
- md1 **completeness + consistency** (all indices 0..total-1 once; same version/csid/total) ARE header-only (`md-codec/chunk.rs:343-367`) — only the cross-chunk INTEGRITY gate needs the encoder.
- Device today: `md.Decode` only detects chunked md1 (bit 0 of sym 0) and REFUSES (`md/md.go:1207`, `ErrChunkedUnsupported`); chunked md1 descriptors **cannot be engraved on-device at all** right now. There is no md1 chunk-header parser and no md `Encode`.

**Implication:** T6 (flagship — derive ms1+mk1+md1 from one seed, engrave all three) needs the full md encoder REGARDLESS, so #10 is on T6's critical path. T5's md1 portion is the entanglement point. → Three coherent scopings (see "scope decision" below).

## Verified facts (cite source)
### Chunk-header models (both 20-bit csid; verified vs codec source)
- **mk1** `StringLayerHeader::Chunked`, 8 five-bit symbols (`mk-codec/string_layer/header.rs`): version(5b), type(5b; 0x01=chunked), **chunk_set_id 20b/4sym**, **total_chunks 5b stored value−1** (1..=32), **chunk_index 5b verbatim 0-based**; validity index<total≤32. csid is RANDOM (CSPRNG `pipeline.rs:45-49`) or caller-pinned.
- **md1** `ChunkHeader`, 37 bits (`md-codec/chunk.rs`): version(4b=4), chunked-flag(1b, bit0 of sym0), **chunk_set_id 20b**, **count 6b stored count−1** (1..=64), **index 6b verbatim 0-based**; index<count. csid is DERIVED (top-20 of payload SHA-256). Single-vs-chunked probed by `sym0 & 0x01` BEFORE `ChunkHeader::read` (md-codec 0.36 quirk: pristine single md1 → spurious `WireVersionMismatch{got:2}`; `me bundle` works around at `bundle.rs:135-151`).
### Host `me bundle` semantics to mirror (`me-cli` v0.3.0 @ `2ee44ad`)
- Classify by HRP (case-insensitive `classify.rs:40-52`); **refuse ms1 up front via a classify-only pre-scan over ALL lines before validating any** (`bundle.rs:188-192`; exit 3 RefusedSecret). Maps to device secret-spine: ms1 hand-entered only, never the bundle/NFC channel.
- Group by chunk_set_id (BTreeMap), hand whole group to codec: mk1 `mk_codec::decode` (`bundle.rs:273`), md1 `md_codec::chunk::reassemble` (`:246`). Drop/dup/mismatch/foreign → SetIncomplete*/inconsistent → exit 4. A foreign chunk with a DIFFERENT csid lands in its own singleton group → incomplete (not a within-group mismatch). A foreign chunk re-stamped with the SAME csid is caught ONLY by the cross-chunk integrity hash (mk1) / payload re-derivation (md1).
### Device reuse inventory (fork `e4ca173`)
- mk1 per-card completeness+integrity: `gui/mk1_inspect.go:48-83` (`mk1Gatherer`), `:156-256` (`mk1GatherFlow`), `mk/mk.go:148-224` (`mk.Decode`/reassemble + cross-chunk hash). **Read-only today (gathers to display, not engrave).**
- mk1 per-card N-plate engrave + set-abort: `gui/derive_xpub.go:263-293` (`multiPlateEngrave`). Wired only into `deriveXpubFlow`.
- mk1 encoder: `mk/encode.go` (only multi-string encoder in-tree). md1 encoder: DOES NOT EXIST (#10).
- Program-enum lockstep (8 sites, all `gui/gui.go`): enum 148-150, dispatch 1489-1496, left-wrap 1630, right-wrap 1637, title 1655-1660, npage 1834, layoutMainPlates 1842-1849 (panic if missed), npages 1852-1853; nav test `gui/derive_xpub_program_test.go` hard-codes engraveXpub upper bound (needs update).

## SCOPE DECISION (the spec must lock; user call) — three options
1. **mk1-only bundle** (cleanest, zero #10): bundle = N mk1 key cards; reuse mk1Gatherer/mk.Decode/multiPlateEngrave end-to-end; group-by-csid across cards; cross-card completeness + guided sequencing. md1 descriptor stays un-engravable-if-chunked until #10. **~650-950 LOC.** Lowest risk; defers the md-encoder to be built ONCE for T6.
2. **mk1 + thin md1 header-completeness**: also gather/engrave the (possibly chunked) md1 descriptor card with header-level completeness + consistency, engraving chunks VERBATIM, but WITHOUT the full md1 cross-chunk integrity gate (needs the encoder). **Unlocks chunked-descriptor engraving now** (impossible today); narrow integrity caveat (a deliberate same-derived-csid forgery slips through — but the operator scans own cards, and a corrupt descriptor is caught at wallet-restore time). The thin md1 chunk-header parser is reusable; only the "skip integrity" compromise is temporary. **+250-400 LOC (~900-1350 total).**
3. **Build the md encoder (#10) first, then full-fidelity T5+T6**: front-load the shared hard dependency; T5 and T6 both get full md1 fidelity; no throwaway. Reorders #10 ahead of T5; larger upfront cycle (the md encoder is the constellation's hardest deferred piece).

## Effort + phasing
One cycle (after the scope call). Phase A: bundle data model + cross-card grouping (+ thin md1 header parser iff option 2). Phase B: new `engraveBundle` program (8-site lockstep + nav test) + bundle gather/sequence flow (incremental scan accumulating distinct cards; "card X of Y, plate P of Q"; bundle-complete gate; reuse multiPlateEngrave per card; set-level abort).

## Biggest risks (for the spec to lock)
1. **Bundle definition** — must state T5 = multi-DISTINCT-card orchestration (not the single multi-chunk card, already done) or risk re-implementing T2b.
2. **mk1-only vs md1-inclusive** (the scope decision above) — locks #10 contact.
3. **Bundle completeness semantics** — cards self-declare TotalChunks per set, but NOTHING declares how many distinct CARDS a wallet needs; the device scans incrementally with no "expected card count." Define the "done adding cards" affordance + whether the md1 descriptor card is required vs optional.
4. **Set-level abort across cards** (extends T4 R0-I3 `abortWarning`): partial-bundle abort must not record completed state.
5. **Program-lockstep drift** (8 gui.go sites + nav-test hard-coded bound) — T4's exact failure surface; a missed `layoutMainPlates` case panics.
6. **No hardware** to validate the incremental multi-card scan/engrave UX (highest-uncertainty UX in the roadmap).

## Gate reminder
`SPEC_seedhammer_T5_*` MUST pass opus R0 to 0C/0I before code; fold → persist verbatim → re-dispatch until GREEN. Fork-side only; no upstream PR. ms1 secret-spine carries (refuse ms1 in the bundle channel).
