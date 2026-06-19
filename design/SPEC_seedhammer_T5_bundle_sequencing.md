# SPEC — T5: guided bundle sequencing (multi-card md1/mk1 → confirm complete → engrave)

**Status:** for opus R0 gate (must reach 0C/0I before any code).
**Fork base:** `bb0e506` (#10 complete). **Fork-side only; no upstream PR.**
**Feeds from:** `design/cycle-prep-recon-T5-bundle-sequencing.md` + `design/agent-reports/seedhammer-T5-recon-refresh-postT10.md` (the post-#10 device-state + bundle-model investigation). **Mirrors host `me bundle`** (`crates/me-cli/src/bundle.rs`, the constellation tool the user directed us to mirror).

## 1. Why / context
A multisig wallet backup = one **md1 descriptor card** + N **mk1 key cards** (each a cosigner account xpub) [+ optionally an **ms1** secret placeholder, which is HAND-TYPED, never NFC]. The device today engraves/inspects ONE card at a time (T2b mk1 gather, #10b md1 gather, T4 mk1 multi-plate engrave). T5 adds **guided sequencing of the whole SET**: gather a bundle of public md1/mk1 strings over NFC, confirm each card's chunk-set is complete + consistent + integral, then engrave all cards' plates with cross-card "card X of Y / plate P of Q" guidance and a set-level abort. This is the device analogue of host `me bundle` (which the user directed us to mirror) and the engraving spine T6 (flagship: derive ms1+mk1+md1 from a seed) will reuse.

**Bundle-completeness model = OPTION A (operator-driven), locked.** The recon established host `me bundle` has NO notion of "all the cards a wallet needs" — it groups input strings by `chunk_set_id` and proves each chunk-set internally complete/consistent/integral, trusting the operator's batch (`bundle.rs:200-273`). Mirroring the constellation ⇒ Option A: the operator scans cards until they hit "Done adding cards"; each card auto-verifies on completion; nothing on the wire declares an expected card count. (Option B — descriptor-driven md1↔mk1 cross-matching — is feasible but conditional on the md1 carrying xpubs + a new normalization step + hardware-untestable UX; documented as a deferred follow-on, OUT of T5.)

## 2. Scope

### IN
- **A new top-level `engraveBundle` program** (parallel to `backupWallet`/`engraveXpub`), with the 8-site program lockstep (`gui/gui.go` enum/dispatch/wrap×2/title/npage/`layoutMainPlates`/npages) coherent + no reachable panic, and the nav-test bound (`gui/derive_xpub_program_test.go:30-31`) updated.
- **Bundle gather (Phase 1):** an NFC scan loop accumulating MULTIPLE DISTINCT cards. A `bundleGatherer` keyed by `chunk_set_id` → one per-card sub-gatherer (reuse `mk1Gatherer`/`md1Gatherer`). A scanned chunk routes to its csid's sub-gatherer; **a NEW csid starts a NEW card** (NOT a foreign-rejection — the key difference from single-card gather). On a card's chunk-set completing, verify it (`mk.Decode` / `md.DecodeChunks` — full reassembly + integrity gate) and add a verified card record to the bundle. Running on-screen tally ("md1 descriptor ✓ · mk1 key ×2"). ms1 (HRP `ms`) → **refuse in this channel** ("Type the ms1 share on-device — never over NFC") + continue.
- **Bundle review/confirm (Phase 2):** show the accumulated bundle (count + per-card type/summary, each marked verified); operator confirms to proceed or keeps adding / aborts.
- **Guided engrave (Phase 3):** for each card in turn, engrave its chunk strings **VERBATIM** as N plates (generalize the existing `multiPlateEngrave` mechanism — engrave a list of strings via `validateMdmk`; the mk1-specific copy is parameterized), with cross-card progress "Card X of Y · Plate P of Q" and a **set-level abort** (`abortWarning`-style: a partial bundle can't be restored; record no completed state). Append an **ms1 reminder** plate/prompt at the end (mirror `bundle.rs:296-306`): "Also hand-engrave your ms1 share(s)."

### OUT (explicitly deferred)
- **Option B descriptor-driven cross-matching** (md1 @N keys ↔ mk1 cards, "N cosigners, all matched") → follow-on (`seedhammer-bundle-cross-match` in FOLLOWUPS); needs the md1 to carry xpubs + a normalization helper.
- **Producing bundle strings on-device** (derive ms1+mk1+md1 from a seed) → T6 (uses T5's Phase-3 sequencing).
- **md1 re-encoding** — md1 has no public encoder; the device engraves the SCANNED md1 chunk strings verbatim (matches host, `bundle.rs:243-267`). No encoder on T5's path.
- **verify-bundle / restore-doc** (read cards back + cross-check parity + watch-only restore document) → T6.

## 3. Verified facts (cite source; full detail in the recon-refresh)
- Reusable single-card primitives: `mk1Gatherer` (`gui/mk1_inspect.go:48-83`), `md1Gatherer` (`gui/md1_gather.go:23-63`), `mk.Decode` (`mk/mk.go:148-224`, SHA-256[0:4] integrity), `md.DecodeChunks`/`md.Reassemble` (`md/expand.go:25`, `md/chunk.go:207`, csid integrity gate), `multiPlateEngrave` (`gui/derive_xpub.go:263-293`, format-agnostic mechanism via `validateMdmk` `gui/gui.go:1897`).
- `mk.Card` (`mk/mk.go:132-139`) and `md.Template`/`md.ExpandWalletPolicy` (`md/expand.go`) for the per-card review summary.
- ms1 = BIP-93 codex32 secret, HRP `ms`; classify by HRP before `1` (host `classify.rs:40-52`); refuse early (host `bundle.rs:188-192`).
- Program lockstep: enum `{backupWallet,engraveXpub,qaProgram}` `gui/gui.go:147-151`; the 8 sites at `:1489-1502,1628-1631,1636-1639,1655-1660,1834,1842-1850,1852-1853`; nav-test bound `gui/derive_xpub_program_test.go:30-31`; `layoutMainPlates` panics on a missing case (`:1842-1850`).
- No bundle/cardSet model exists today (grep-confirmed). No NFC writer exists (output = engraved plates + QR; input = NFC scan of PUBLIC strings).

## 4. Faithfulness / security spine
- **md1/mk1 are PUBLIC → NFC-gather OK.** ms1/seed are SECRET → REFUSED in the bundle/NFC channel (hand-typed only); a clear refusal + the end-of-bundle ms1 reminder. No secret material is ever gathered, displayed, or engraved by T5.
- **Each card is engraved VERBATIM** from its scanned, integrity-verified chunk strings — no re-encode, no transformation (matches host `me bundle`). A card that fails reassembly/integrity is never added to the bundle (never engraved).
- **Faithful to host `me bundle`:** per-chunk-set completeness/consistency/integrity, operator-supplied set, ms1 refusal + reminder. No invented "expected card count."

## 5. Acceptance gate (TDD; `NFCReader()==nil` in tests → drive the accumulator + flows directly via `runUI`/`click`/`runes`, like the mk1/#10b gather tests)
1. **Multi-card gather:** feed a 2-card bundle (1 chunked md1 + 1 chunked mk1, embedded golden chunk sets) chunk-by-chunk in interleaved order → both cards reach complete + verified; the bundle holds exactly 2 verified cards; a 3rd card (different csid) adds a 3rd.
2. **New-csid = new card (not foreign):** a chunk with a csid different from the in-progress card starts a new card, not a `gatherForeign` rejection.
3. **Per-card integrity:** a dropped/tampered chunk in one card → that card never completes (stays pending, never added/engraved); the other cards are unaffected.
4. **ms1 refusal:** an `ms1…` scan → refused with the "type on-device" message, bundle unchanged; the end reminder appears.
5. **Guided engrave sequence:** confirming a 2-card bundle drives "Card 1 of 2 · Plate P of Q" then "Card 2 of 2 · …" via `validateMdmk`, engraving each card's strings verbatim; both md1 and mk1 cards sequence correctly.
6. **Set-level abort:** aborting mid-bundle shows the partial-bundle warning and records no completed state.
7. **Program nav:** the new `engraveBundle` program is reachable/titled/wrap-correct across all 8 lockstep sites with NO panic; the nav-test bound is updated; `TestAllocs` stays green.
8. **No-regression + no-panic:** full suite + `TestAllocs` green; single-card mk1/md1/ms1 flows + `deriveXpubFlow`/`backupWalletFlow` unchanged; fuzz the `bundleGatherer` accumulator (arbitrary scan sequences → no panic, only verified-complete cards added).

## 6. Invariants (R0 must confirm)
- **I-1:** a card is added to the bundle ONLY after its chunk-set passes full reassembly + integrity (`mk.Decode`/`md.DecodeChunks`); a failed/partial card is never added, never engraved.
- **I-2:** a chunk whose csid differs from any in-progress card starts a NEW card (bundle-level), NOT a foreign-rejection (the semantic inversion vs single-card gather).
- **I-3:** ms1 (HRP `ms`) is refused in the NFC/bundle channel with a clear hand-type message; no secret is ever gathered/engraved; an ms1 reminder is shown at bundle end.
- **I-4:** every engraved plate is a VERBATIM scanned chunk string (md1 + mk1); no re-encode/transform.
- **I-5:** set-level abort records no completed state and warns the bundle is unusable partial.
- **I-6:** the new `engraveBundle` program is coherent across all 8 lockstep sites (no reachable `panic("invalid page")`); nav-test updated; `TestAllocs` intact (any bundle build happens off the alloc-gated screens).
- **I-7:** single-card flows (mk1/md1 inspect, `deriveXpubFlow`, `backupWalletFlow`) and the codecs are byte-unchanged.
- **I-8 (faithfulness):** the completeness model is Option A — per-chunk-set, operator-driven — matching host `me bundle` (no invented expected-card-count); cross-matching is explicitly deferred.

## 7. Biggest risks (lock in R0)
1. **The new-csid-=-new-card inversion** (I-2): the single-card gatherers treat a foreign csid as an error; the bundle gatherer must treat it as a new card. Reuse the per-card gatherers UNCHANGED but wrap them in a csid-keyed map; do not edit the single-card foreign semantics.
2. **"Done adding cards" UX** — the only completeness gate (Option A); must be unambiguous and not strand a half-scanned card (a card mid-chunk-set when the operator hits done → warn it's incomplete / drop it).
3. **Program-lockstep drift** (8 sites + nav-test bound) — a missed `layoutMainPlates` case panics (T4's exact failure surface).
4. **Set-level abort across cards** — partial-bundle warning; no completed state recorded.
5. **Generalizing `multiPlateEngrave`** — its copy is mk1-specific; parameterize without regressing `deriveXpubFlow`'s use.
6. **No hardware** to validate the incremental multi-card scan/engrave UX (highest-uncertainty; the spec defines behavior, hardware tuning later).

## 8. Gate reminder
This spec MUST pass opus R0 to 0C/0I before code; fold → persist verbatim to `design/agent-reports/` → re-dispatch after every fold until GREEN. Then implementation plan → its own R0 → GREEN → single-implementer TDD in a worktree → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h. ms1 secret-spine carries.
