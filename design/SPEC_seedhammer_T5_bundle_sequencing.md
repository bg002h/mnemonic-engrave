# SPEC — T5: guided bundle sequencing (multi-card md1/mk1 → confirm complete → engrave)

**Status:** for opus R0 gate (must reach 0C/0I before any code).
**Fork base:** `bb0e506` (#10 complete). **Fork-side only; no upstream PR.**
**Feeds from:** `design/cycle-prep-recon-T5-bundle-sequencing.md` + `design/agent-reports/seedhammer-T5-recon-refresh-postT10.md` (the post-#10 device-state + bundle-model investigation). **Mirrors host `me bundle`** (`crates/me-cli/src/bundle.rs`, the constellation tool the user directed us to mirror).

## 1. Why / context
A multisig wallet backup = one **md1 descriptor card** + N **mk1 key cards** (each a cosigner account xpub) [+ optionally an **ms1** secret placeholder, which is HAND-TYPED, never NFC]. The device today engraves/inspects ONE card at a time (T2b mk1 gather, #10b md1 gather, T4 mk1 multi-plate engrave). T5 adds **guided sequencing of the whole SET**: gather a bundle of public md1/mk1 strings over NFC, confirm each card's chunk-set is complete + consistent + integral, then engrave all cards' plates with cross-card "card X of Y / plate P of Q" guidance and a set-level abort. This is the device analogue of host `me bundle` (which the user directed us to mirror) and the engraving spine T6 (flagship: derive ms1+mk1+md1 from a seed) will reuse.

**Bundle-completeness model = OPTION A (operator-driven), locked.** The recon established host `me bundle` has NO notion of "all the cards a wallet needs" — it groups input strings by `chunk_set_id` and proves each chunk-set internally complete/consistent/integral, trusting the operator's batch (`bundle.rs:200-273`). Mirroring the constellation ⇒ Option A: the operator scans cards until they hit "Done adding cards"; each card auto-verifies on completion; nothing on the wire declares an expected card count. (Option B — descriptor-driven md1↔mk1 cross-matching — is feasible but conditional on the md1 carrying xpubs + a new normalization step + hardware-untestable UX; documented as a deferred follow-on, OUT of T5.)

## 2. Scope

### IN
- **A new top-level `engraveBundle` program**, inserted in the enum **BEFORE `qaProgram`** (R0-I-A: sites `npage`/`npages` are derived consts `int(engraveXpub)+1` `gui/gui.go:1834,1853` and the wrap bounds key off `engraveXpub` — ALL of these + both consts must be rewritten off `engraveBundle`, the new navigable upper bound; inserting after `qaProgram` would miscount). The 8-site lockstep (enum/dispatch/wrap×2/title/npage/`layoutMainPlates`/npages) must be coherent with **BOTH the `title` switch arm AND the `layoutMainPlates` arm present** (R0-I-B: only `layoutMainPlates` panics on a missing case `:1849`; the `title` switch fails OPEN to a blank title `:1655-1660` — a silent defect). The nav-test (`gui/derive_xpub_program_test.go:30-31`) must update the navigable bound AND land ON `engraveBundle` asserting its non-blank title.
- **Bundle gather (Phase 1) — explicit per-scan classification (R0-C1/C2):** an NFC scan loop accumulating MULTIPLE DISTINCT cards. Each `scan.Object` is classified BEFORE accumulation:
  - **ms1 / any `codex32.String` secret (HRP `ms`)** → **refuse** ("Type the ms1 share on-device — never over NFC") + continue. **R0-C2: ms1 arrives as a `codex32.String`, NOT `mdmkText`** (`gui/scan.go:70-73`) — the loop MUST add an explicit `codex32.String`/HRP-`ms` case; the `mdmkText`-only assertion would otherwise SILENTLY DROP it (no refusal). Also drop+note any other non-md/mk scan object (e.g. a bip39 mnemonic, a bare address).
  - **single-string (non-chunked) mk1** → **REFUSE** (R0-C1, host parity `bundle.rs:128`): a valid mk1 is ALWAYS ≥2 chunks (xpub_compact 73B > 56B cap), so a single mk1 string is malformed and has NO cross-chunk integrity (`mk.Decode` would take the `!Chunked` no-hash path `mk/mk.go:177-181`). Refuse with "Incomplete key card — scan all its chunks."
  - **single-string (non-chunked) md1** → accept as a **standalone 1-plate card** (a small descriptor legitimately fits one string): validate via `md.Decode` (BCH + full structural decode = its integrity; there is no chunk set to reassemble) and add it as its own card record. Do NOT key it by csid (it has none, zero-value `md/chunk.go:193-196`) — give standalone cards their own bucket/unique key so two single md1 cards never collide.
  - **chunked md1 / chunked mk1** → route to a `bundleGatherer` keyed by `chunk_set_id` → one per-card sub-gatherer (reuse `mk1Gatherer`/`md1Gatherer` UNCHANGED). A scanned chunk routes to its csid's sub-gatherer; **a NEW csid starts a NEW card** (NOT a foreign-rejection — the key inversion vs single-card gather). On a card's chunk-set completing, verify it (`mk.Decode` / `md.DecodeChunks` — full reassembly + integrity gate) and add a verified card record.
  - Running on-screen tally ("md1 descriptor ✓ · mk1 key ×2"); duplicate of an already-complete card → ignore with feedback.
- **Bundle review/confirm (Phase 2):** show the accumulated bundle (count + per-card type/summary, each marked verified); operator confirms to proceed or keeps adding / aborts.
- **Guided engrave (Phase 3):** for each card in turn, engrave its strings **VERBATIM** as N plates (single-string md1 → 1 plate) via `validateMdmk`, with cross-card progress "Card X of Y · Plate P of Q" and a **set-level abort** (`abortWarning`-style: a partial bundle can't be restored; record no completed state). Append an **ms1 reminder** prompt at the end (mirror `bundle.rs:296-306`): "Also hand-engrave your ms1 share(s)." **R0-I-C:** generalize `multiPlateEngrave`/`abortWarning` (`gui/derive_xpub.go:263-302`) by ADDING parameters (the card label + plate-count context) with defaults that leave `deriveXpubFlow`'s existing call site (`derive_xpub.go:269,300-301`) BYTE-UNCHANGED — or add a thin bundle-specific sequencer alongside it; do NOT mutate the mk1 copy/behavior `deriveXpubFlow` depends on (I-7).

### OUT (explicitly deferred)
- **Option B descriptor-driven cross-matching** (md1 @N keys ↔ mk1 cards, "N cosigners, all matched") → follow-on (`seedhammer-bundle-cross-match` in FOLLOWUPS); needs the md1 to carry xpubs + a normalization helper.
- **Producing bundle strings on-device** (derive ms1+mk1+md1 from a seed) → T6 (uses T5's Phase-3 sequencing).
- **md1 re-encoding** — md1 has no public encoder; the device engraves the SCANNED md1 chunk strings verbatim (matches host, `bundle.rs:243-267`). No encoder on T5's path.
- **verify-bundle / restore-doc** (read cards back + cross-check parity + watch-only restore document) → T6.

## 3. Verified facts (cite source; full detail in the recon-refresh)
- Reusable single-card primitives: `mk1Gatherer` (`gui/mk1_inspect.go:48-83`), `md1Gatherer` (`gui/md1_gather.go:23-63`), `mk.Decode` (`mk/mk.go:148-224`, SHA-256[0:4] integrity), `md.DecodeChunks`/`md.Reassemble` (`md/expand.go:25`, `md/chunk.go:207`, csid integrity gate), `multiPlateEngrave` (`gui/derive_xpub.go:263-293`, format-agnostic mechanism via `validateMdmk` `gui/gui.go:1897`).
- `mk.Card` (`mk/mk.go:132-139`) and `md.Template`/`md.ExpandWalletPolicy` (`md/expand.go`) for the per-card review summary.
- ms1 = BIP-93 codex32 secret, HRP `ms`; classify by HRP before `1` (host `classify.rs:40-52`); refuse early (host `bundle.rs:188-192`).
- Program lockstep: enum `{backupWallet,engraveXpub,qaProgram}` `gui/gui.go:147-151`; the 8 sites at `:1489-1502,1628-1631,1636-1639,1655-1660,1834,1842-1850,1852-1853`; nav-test bound `gui/derive_xpub_program_test.go:30-31`. **`npage`/`npages` are derived consts `int(engraveXpub)+1` (`:1834,1853`) and the wrap bounds key off `engraveXpub`** → `engraveBundle` MUST be inserted BEFORE `qaProgram` and become the new bound in all of these (R0-I-A). **`layoutMainPlates` PANICS on a missing case (`:1849`) but the `title` switch FAILS OPEN to a blank title (`:1655-1660`)** — both arms required (R0-I-B).
- **ms1 scan classification (R0-C2):** the scanner yields `mdmkText` for md1/mk1 but a `codex32.String` for an `ms`-HRP secret (`gui/scan.go:70-73`); the gather loop's `mdmkText`-only type-assert (`gui/mk1_inspect.go:218`, `gui/md1_gather.go:135`) silently drops anything else → the bundle loop needs an explicit `codex32.String` refusal case.
- **single-string vs chunked:** mk1 is ALWAYS chunked (single mk1 malformed → host refuses, `bundle.rs:128`); md1 single-string is valid (a small descriptor); `mk.Decode([single])` takes the `!Chunked` no-integrity path (`mk/mk.go:177-181`); single cards have zero-value csid (`mk/mk.go:74-76`, `md/chunk.go:193-196`) → must NOT be csid-keyed.
- Golden fixtures for tests already in-tree: `wshSortedmultiChunks` (#10b) + the mk1 chunk goldens (`v1c0`/`v1c1`/`v3c1` per `mk` tests).
- No bundle/cardSet model exists today (grep-confirmed). No NFC writer exists (output = engraved plates + QR; input = NFC scan of PUBLIC strings).

## 4. Faithfulness / security spine
- **md1/mk1 are PUBLIC → NFC-gather OK.** ms1/seed are SECRET → REFUSED in the bundle/NFC channel (hand-typed only); a clear refusal + the end-of-bundle ms1 reminder. No secret material is ever gathered, displayed, or engraved by T5.
- **Each card is engraved VERBATIM** from its scanned, integrity-verified chunk strings — no re-encode, no transformation (matches host `me bundle`). A card that fails reassembly/integrity is never added to the bundle (never engraved).
- **Faithful to host `me bundle`:** per-chunk-set completeness/consistency/integrity, operator-supplied set, ms1 refusal + reminder. No invented "expected card count."

## 5. Acceptance gate (TDD; `NFCReader()==nil` in tests → drive the accumulator + flows directly via `runUI`/`click`/`runes`, like the mk1/#10b gather tests)
1. **Multi-card gather:** feed a 2-card bundle (1 chunked md1 + 1 chunked mk1, embedded golden chunk sets) chunk-by-chunk in interleaved order → both cards reach complete + verified; the bundle holds exactly 2 verified cards; a 3rd card (different csid) adds a 3rd.
2. **New-csid = new card (not foreign):** a chunk with a csid different from the in-progress card starts a new card, not a `gatherForeign` rejection.
3. **Per-card integrity:** a dropped/tampered chunk in one card → that card never completes (stays pending, never added/engraved); the other cards are unaffected.
4. **ms1 refusal (R0-C2):** an `ms1…` scan (delivered as a `codex32.String`) → refused with the "type on-device" message via the explicit classification case, bundle unchanged; the end reminder appears. Assert it is NOT silently dropped.
5. **Single-string cases (R0-C1):** a single-string (non-chunked) **mk1** → REFUSED ("scan all its chunks"), never engraved; a single-string **md1** → accepted as a standalone 1-plate card (BCH+structural-validated), engraved as 1 plate; two distinct single md1 cards do NOT collide.
6. **Guided engrave sequence:** confirming a 2-card bundle drives "Card 1 of 2 · Plate P of Q" then "Card 2 of 2 · …" via `validateMdmk`, engraving each card's strings verbatim; both md1 and mk1 cards sequence correctly.
7. **Set-level abort:** aborting mid-bundle shows the partial-bundle warning and records no completed state.
8. **Empty/degenerate bundle:** "Done adding" with 0 complete cards → a no-op/warn (nothing to engrave), not a crash; "Done" while a card is mid-chunk-set → warn that card is incomplete (drop it, don't engrave a partial).
9. **Program nav (R0-I-A/B):** `engraveBundle` (before `qaProgram`) reachable/titled (non-blank)/wrap-correct across all 8 lockstep sites with NO panic; nav-test lands on it asserting the title; `TestAllocs` re-run green.
10. **No-regression + no-panic:** full suite + `TestAllocs` green; single-card mk1/md1/ms1 flows + `deriveXpubFlow` (byte-unchanged call site, I-C) + `backupWalletFlow` unchanged; fuzz the `bundleGatherer` accumulator (arbitrary scan sequences → no panic, only verified-complete cards added).

## 6. Invariants (R0 must confirm)
- **I-1:** a card is added ONLY after it passes its integrity gate — chunked: full reassembly + integrity (`mk.Decode`/`md.DecodeChunks`); single-string md1: BCH + full structural `md.Decode`. A failed/partial card is never added, never engraved. **Single-string mk1 is REFUSED** (R0-C1, host parity `bundle.rs:128`; no integrity exists for it).
- **I-2:** only CHUNKED cards are keyed by `chunk_set_id`; a chunk whose csid differs from any in-progress card starts a NEW card (NOT a foreign-rejection — the inversion vs single-card gather). Single-string md1 cards get a unique non-csid bucket (zero-value csid would otherwise collide them, R0-C1).
- **I-3:** ms1 — arriving as a `codex32.String`, NOT `mdmkText` (R0-C2) — is refused in the NFC/bundle channel via an EXPLICIT classification case with a clear hand-type message (never silently dropped); no secret is ever gathered/engraved; an ms1 reminder is shown at bundle end.
- **I-4:** every engraved plate is a VERBATIM scanned chunk string (md1 + mk1); no re-encode/transform.
- **I-5:** set-level abort records no completed state and warns the bundle is unusable partial.
- **I-6:** the new `engraveBundle` program (inserted BEFORE `qaProgram`) is coherent across all 8 lockstep sites — BOTH the `title` arm (no blank-title fail-open, R0-I-B) AND the `layoutMainPlates` arm present, and `npage`/`npages`/both wrap bounds rewritten off `engraveBundle` not `engraveXpub` (R0-I-A); nav-test updated to the new bound AND landing on `engraveBundle` asserting a non-blank title; `TestAllocs` intact + re-run after the enum change.
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
