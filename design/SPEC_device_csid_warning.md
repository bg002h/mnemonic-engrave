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
   export in the fork's `mk/` package: canonically re-encode the decoded
   card's bytecode (the encoder's existing bytecode builder, factored, not
   duplicated) and return `top20(bytecode)`. This is the HOST operand
   (derive over the canonical re-encode of the decoded card) — R6 parity at
   the semantic level. Pinned by extending the parity test: for every clean
   vendored-corpus row, `DerivedChunkSetID(Decode(strings))` equals the
   row's `derived_csid`.
2. **Inspect flow** (`decodeGathered` success, chunked input only): compare
   derived vs the gatherer's declared id. On mismatch, show a NON-BLOCKING
   notice screen before the card display — the fork's modal conventions
   bind (every modal answers BACK; proceeding continues to the card).
   Content parity per R6: both ids as bare 5-digit lowercase hex + the
   remedy gist, condensed for the panel. Draft (frozen by test + operator
   screenshot approval — the wording may be redlined at the screenshot
   gate):
   > `Stamped set id 12345 was not derived from this key's content (computes ef12f). The key itself is intact. The plate was minted with a pinned id — re-mint it without --chunk-set-id to fix.`
3. **Bundle flow** (mk sub-gatherer completion): same comparison; the tap
   is still ACCEPTED (R1 — warning, never refusal). The card's bundle
   entry carries a visible warning marker on the bundle review surface;
   exact affordance mirrors the existing bundle status idioms
   (implementer proposes; screenshot gate approves).
4. **Single-string mk1**: carries no set id — structurally no check, no
   warning (header type decides, not a value comparison).

## Acceptance

- gui tests mirror the md1 R0-C1 pattern: a mis-stamped set (strings from
  the vendored corpus's pinned row — fixtures come FROM the corpus, never
  hand-minted) fires the warning in BOTH flows; the clean twin is silent;
  single-string is silent; the notice answers BACK and proceeds.
- Parity: the contract-1 corpus extension test (every clean row).
- Mutation: deleting the comparison fails the warning tests in both flows;
  mutated-line-RAN evidence.
- **Simulator screenshot** (`cmd/emu` `shots_` driver, NFC-injected
  mis-stamped set): a PNG of the warning screen — the OPERATOR GATE. The
  wording/layout freezes only after operator approval.
- Suite: `go test ./mk/` + `./gui/` via `scripts/gui-shard-test.sh` (24);
  gofmt -l empty; go vet clean.
- **On-device acceptance, DEFINED here (the S6b lesson — no more shipping
  without one), operator-executed:** flash the build (sh2-flash); tap an
  NDEF tag carrying the pinned corpus card (payload prepared host-side via
  `me`, committed alongside) → the warning appears; tap the clean-twin
  tag → no warning. Flashing and taps are the operator's; everything up to
  the ready build + tag payloads is this cycle's deliverable.

## Out of scope

Refusing on mismatch (R1); the md1 flow (already refuses, correct);
host tools (shipped); any engraving-flow change; OTP/signing/flashing
(operator, risk-set irreversible class); mk/md codec changes beyond the
one additive `mk/` export.
