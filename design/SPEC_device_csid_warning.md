# SPEC — SH2 device: mk1 chunk-set-id mismatch warning (scan flow)

**Status: DRAFT for R0.** Implements FOLLOWUPS `device-csid-mismatch-warning`
(operator rulings W11 "Same warning everywhere" + W12 scheduling; host legs
shipped 2026-08-31/09-01). Fork-native UI work (Rust-primary exempt class b);
**no codec lockstep** — the derivation is already pinned cross-language by the
vendored corpus + parity test (`mk/chunk_set_id_parity_test.go`,
`mk/testdata/csid_ext_v0.1.json`, SHA-gated). Baseline: seedhammer fork
`origin/main` (`2337ed3` at drafting).

## The gap (measured)

The device's two mk1 reassembly points decode via `mk.Decode` and never
compare the DECLARED set id (the gatherer's accumulator key) against the
DERIVED id: `gui/mk1_inspect.go` (`decodeGathered`) and `gui/bundle.go`
(the mk sub-gatherer completion). A mis-stamped plate scans, decodes and
displays with no signal — the last silent surface in the constellation.
The md1 sibling already routes its (REFUSING) mismatch to a distinct screen
(`gui/md1_gather.go`, R0-C1); mk1's version is a WARNING per host ruling R1
(the id is opaque to content; mk warns, never refuses).

## Contracts

1. **`mk.DerivedChunkSetID(card Card) (uint32, error)`** — new additive
   export in the fork's `mk/` package: a thin wrapper over the ALREADY
   factored `encodeBytecode` (`mk/encode.go:50`; `Encode` is untouched —
   r1 N1) returning `top20(bytecode)`. This is the HOST operand
   (derive over the canonical re-encode of the decoded card) — R6 parity at
   the semantic level. Pinned by extending the parity test: for every clean
   vendored-corpus row, `DerivedChunkSetID(Decode(strings))` equals the
   row's `derived_csid`.
2. **Inspect flow** (`decodeGathered` success): the comparison is gated
   on an explicit `chunked bool` field ADDED to `mk1Gatherer`, set from
   the first accepted header (r1 I3 — `setID == 0` is a real mis-stamp
   value and `total == 1` is representable, so neither is a proxy). On
   mismatch, a NON-BLOCKING notice screen before the card display (every
   modal answers BACK; proceeding continues). **Content: the HOST WARNING
   VERBATIM** — measured to fit the panel with 302 chars headroom and
   ASCII-safe (r1 I2/M1; the earlier condensed draft's em dash blanks
   the modal, 5004 ink px vs the 6000 floor, and is DROPPED):
   > `warning: this key card's stamped chunk-set id (12345) was not derived from its content, which computes ef12f. The card decodes fine, but diagnostics that name plates by id will call it 12345. To fix it, re-mint: run mk encode again without --chunk-set-id and the id is derived from the key data automatically.`
   Byte-exact R6 parity, stronger than the spec first asked.
3. **Bundle-gatherer flow — ALL SIX consumers, enumerated (r1 I1):** the
   comparison runs once at `offerChunkedMK1` set completion and the
   result travels ON `bundleCard` as data (declared + derived), so no
   downstream surface can be silent by omission. Per surface:
   - Engrave Bundle (`bundle_flow.go:45`) and Wallet Policy
     (`wallet_policy.go:125`): interactive gather → the contract-2
     notice modal at set completion, PLUS the review-list marker.
   - **Build Policy cosigner gather (`multisig_build.go:184`) — the
     funds-most path: notice modal at set completion, and the marker in
     `buildPlateCensusLines`, `buildPlateInventoryLines` (the RESTORE
     DOC — a mis-stamped id archived there is the name-drift hazard the
     host cycle documented) and `buildPayloadCardsLines`.**
   - Engrave Multisig (`multisig.go:102`): NO marker, NO modal — its
     `extractSuppliedMd1` refuses unconditionally on ANY mk1 presence
     before a card could render (verified r2), so a csid warning is
     unreachable there; silence is correct by prior refusal, and a test
     pins that refusal so the reason cannot rot silently.
   - Verify readbacks (`multisig_verify.go:781`,
     `singlesig_verify.go:145`): line-marker only, NO modal — these
     screens are verdict-shaped and the verify verdict itself is
     content-based, unaffected by the stamped id; a mid-verify modal
     would blur the pass/fail reading. (Stated reason, per R0; the
     operator may overrule at the screenshot gate.)
   The marker's compact form (e.g. an id rendered `12345!ef12f` or a
   flag glyph) is the implementer's proposal, frozen at the screenshot
   gate. Every tap is still ACCEPTED (R1 — warning, never refusal).
4. **Single-string mk1 — UNREACHABLE, stated with the measurement (r1
   I4/N2/M3):** a single-string mk1 KeyCard is structurally impossible
   (host-measured: 56-byte single-string capacity < 80-byte minimum card
   bytecode), so no comparison site exists to guard. In the bundle flow
   single-strings are refused at classify (`clsSingleMK1Refuse`,
   `gui/bundle.go:80-82`) before any gatherer; in the inspect flow one
   fails decode as malformed. NO gate is claimed for "silent on
   single-string" (the earlier claim could not fail); instead a test
   pins `clsSingleMK1Refuse` — a real, failable assertion about the
   adjacent behavior.

## Acceptance

- gui tests mirror the md1 R0-C1 pattern: the corpus's pinned row strings
  fire the warning in the inspect flow AND in each of the four
  modal/marker-bearing bundle consumers (Engrave Bundle, Wallet Policy,
  Build Policy incl. census/inventory/payload lines, and the two verify
  readbacks' line-markers); the clean twin is silent everywhere;
  `clsSingleMK1Refuse` pinned; the Engrave-Multisig unconditional mk1
  refusal pinned (contract 3's unreachability reason — the existing
  TestExtractSuppliedMd1 subtest counts, r3 Minor); the notice answers
  BACK and proceeds.
- Parity: the contract-1 corpus extension test (every clean row).
- Mutation: deleting the comparison fails the warning tests in both flows;
  mutated-line-RAN evidence.
- **Simulator screenshot** (`cmd/emu` `shots_` driver, NFC-injected
  mis-stamped set): a PNG of the warning screen — the OPERATOR GATE. The
  wording/layout freezes only after operator approval.
- Suite: `go test ./mk/` + `./gui/` via `scripts/gui-shard-test.sh` (24);
  gofmt -l empty; go vet clean.
- **On-device acceptance, DEFINED here (the S6b lesson), operator-executed
  (r1 M2 — precise tap counts):** FOUR tags, two per card (each card is 2
  chunks; one tag per chunk; `me`-prepared payloads committed alongside,
  end-to-end parse verified by r1). Flash (sh2-flash), then: tap the
  pinned card's two tags in either order (reassembly is order-tolerant)
  → after the SECOND tap (set completion) the warning modal appears — the
  first tap correctly shows only capture progress; tap the clean twin's
  two tags → no warning at completion. Flashing and taps are the
  operator's; the ready build + four tag payloads are this cycle's
  deliverable.

## Out of scope

Refusing on mismatch (R1); the md1 flow (already refuses, correct);
host tools (shipped); any engraving-flow change; OTP/signing/flashing
(operator, risk-set irreversible class); mk/md codec changes beyond the
one additive `mk/` export.
