# R0 — SPEC_device_csid_warning.md, round 2 (fold-check)

**Artifact:** `design/SPEC_device_csid_warning.md` @ `0a4b545` (fold of r1)
**Diff checked:** `git diff 594c3e3..0a4b545 -- design/SPEC_device_csid_warning.md`
**Prior review:** `design/agent-reports/R0-device-csid-warning-r1.md` (0C/4I/3M/2N) @ `594c3e3`
**Ground truth:** `/scratch/code/shibboleth/seedhammer` @ `origin/main` `2337ed3` (tree verified
clean before and after; one probe test file added and deleted, `git status --short` empty,
`git rev-parse HEAD` unchanged at `2337ed3cdeed03c1bc689e2c986919d72e0907ff`)
**Scope:** proportional fold-check per project rule — NOT a fresh audit. r1's verified-sound
items (Contract 1's wrapper-over-`encodeBytecode` with 21/21 corpus proof; `me`'s NDEF
end-to-end parse) taken as settled and not re-derived.

## Verdict

**0 Critical / 1 Important / 0 Minor / 0 Nit — does NOT close.**

I1 is only 5/6 discharged: the fold enumerates and disposes five of r1's six named
`offerChunkedMK1` consumers, and silently drops the sixth. Every other r1 finding (I2, I3, I4,
M1, M2, M3, N1, N2) is fully and correctly discharged, independently re-verified against the
fork tree and, for I2/M1, against the real rasterizer.

---

## Per-finding disposition

### I1 — 5 of 6 consumers discharged; `gui/multisig.go:102` (Engrave Multisig) dropped entirely

**Not closed. Important — same class r1 raised, now on one surface instead of four.**

r1's table named SIX callers of the shared `offerChunkedMK1` comparison point (all reached via
`bundleGatherFlow`): `bundle_flow.go:45`, `wallet_policy.go:125`, `multisig_build.go:184`,
`multisig_verify.go:781`, `multisig.go:102`, `singlesig_verify.go:145`. The fold's Contract 3
names and disposes exactly five:

> "Engrave Bundle (`bundle_flow.go:45`) and Wallet Policy (`wallet_policy.go:125`): interactive
> gather → the contract-2 notice modal at set completion, PLUS the review-list marker."
> "**Build Policy cosigner gather (`multisig_build.go:184`)** — the funds-most path: notice
> modal at set completion, and the marker in `buildPlateCensusLines`, `buildPlateInventoryLines`
> ... and `buildPayloadCardsLines`."
> "Verify readbacks (`multisig_verify.go:781`, `singlesig_verify.go:145`): line-marker only, NO
> modal ... (Stated reason, per R0; the operator may overrule at the screenshot gate.)"

`gui/multisig.go:102` — "Engrave Multisig" — never appears. Grepped the whole file:
`grep -n "multisig.go:102\|Engrave Multisig\|extractSuppliedMd1" design/SPEC_device_csid_warning.md`
→ zero hits. No modal decision, no marker decision, and no stated "silence is correct" reasoning
of the kind given for the verify readbacks.

Confirmed this is a live call site, not stale: `gui/multisig.go:102` still calls
`bundleGatherFlow(ctx, th, "Engrave Bundle")`, and `offer()` (`gui/bundle.go:120-141`) is generic
— it doesn't discriminate by caller, so a chunked mk1 offered during this specific gather still
routes through `offerChunkedMK1` and can compute a real mismatch.

I traced the eventual outcome: `extractSuppliedMd1` (`gui/multisig_supply.go:24`) unconditionally
refuses when any `cardMK1` is present —

> `case cardMK1, cardMS1: return nil, false // a stray key/secret card pollutes the supply.`

— regardless of csid match. So a plausible "silence is correct" argument exists (the card is
refused wholesale here, matched or mismatched, unlike the funds-critical Build Policy path where
a mismatched card is silently *accepted*) — but the fold never makes that argument, and r1's own
remedy for this class was explicit: *"decide, in the spec, for each of the... unmarked
consumers: mark, or state why silence is correct there."* Three of the four were decided; this
one — arguably the easiest of the four to justify — was simply omitted, leaving the same
ambiguity r1 flagged: an implementer has no spec text to follow for this surface, and Contract
3's own preamble ("the result travels ON `bundleCard` as data... so no downstream surface can be
silent by omission") is literally false as written, since this surface's disposition is absent
rather than "stated non-modal by design."

Not Critical: I confirmed the refusal is unconditional on mk1 presence, so no wrong result or
funds-safety break follows from the omission — this is a spec-completeness gap, not a silently
accepted mismatch. Importance is unchanged from r1's own classification of the same defect class.

**Remedy:** add one clause to Contract 3 (or Out of scope) naming `gui/multisig.go:102` and
stating the disposition — most likely "no marker/modal: any mk1 is refused here regardless of
csid status (`extractSuppliedMd1`), so the mismatch is moot" — mirroring the reasoning already
given for the verify readbacks.

### I2 + M1 — host wording verbatim, ASCII, byte-exact — CLOSED

Fetched `SEED_pinned_12345_ef12f`'s `warning_text` from
`/scratch/code/shibboleth/seedhammer/mk/testdata/csid_ext_v0.1.json` and diffed byte-for-byte
against the spec's quoted string (`design/SPEC_device_csid_warning.md:41`) in Python:
**`EQUAL: True`**, 309 raw chars, ASCII-only both (`ascii-only corpus: True`,
`ascii-only spec: True`). Also confirmed it matches `crates/me-cli/src/csid_warn.rs`'s
`chunk_set_id_mismatch_warning` format template verbatim (same literal, `{declared:05x}` /
`{derived:05x}` substituted with `12345`/`ef12f`).

Independently re-ran r1's raster measurement rather than trusting the citation: added a probe
test in `gui/` calling the real `assertModalBodyFits(t, ..., errorScreenBody, body)` harness
against this exact 309-char string. Result: **`258 chars drawn in full, headroom 302 chars
(margin 80)`** — PASS, reproducing r1's own cited numbers exactly (309 raw − 51 whitespace = 258
normalized chars, which is what the harness's whitespace-stripping `normalizeDrawn` measures —
resolves what first looked like a 309-vs-258 discrepancy). Probe file deleted; fork tree
confirmed byte-identical to `2337ed3` after (`git status --short` empty).

### I3 — explicit `chunked bool`, both proxies rejected in text — CLOSED

`gui/mk1_inspect.go:44-49` (struct) and `:51-67` (`offer`) confirmed unchanged from r1's
citation: the `!g.primed` branch sets `total`/`setID`/`primed` together from the first accepted
header — exactly the point the fold's "set from the first accepted header" describes. The fold's
Contract 2 text states the field and both rejected proxies verbatim from r1's own reasoning
(`setID == 0` a real mis-stamp value; `total == 1` representable for a genuinely chunked
header). r1's remedy bar was explicit ("it just has to be in the spec") — met.

### I4 + N2 + M3 — single-string reframed unreachable-with-measurement — CLOSED

Contract 4 drops the unfailable "silent on single-string" claim and states the structural
measurement (56-byte single-string capacity < 80-byte minimum card bytecode, host-measured) plus
two distinct, correctly separated outcomes: bundle flow refuses at `classify`
(`clsSingleMK1Refuse`, confirmed real code at `gui/bundle.go:80-82`, part of the existing
`classify` switch I read in full); inspect flow fails decode as malformed. This directly answers
N2 (flows not conflated) and I4 (a real, failable assertion — `clsSingleMK1Refuse` pinned —
replaces the gate that could not fail). M3: the old "fixtures come FROM the corpus, never
hand-minted" sentence is gone from the acceptance section entirely (`grep` for that phrase
against the current file: zero hits), consistent with scoping the corpus-only rule away from the
single-string case as M3 asked.

### M2 — 4 tags, 2 per card, warning on second tap, order-irrelevant — CLOSED

Acceptance now reads: "FOUR tags, two per card... tap the pinned card's two tags in either order
(reassembly is order-tolerant) → after the SECOND tap (set completion) the warning modal appears
— the first tap correctly shows only capture progress; tap the clean twin's two tags → no
warning at completion." Matches r1's remedy precisely. Spot-checked `mk/mk.go`'s reassembly loop
(`~190-210`): chunks are written into a `slots` array by explicit `idx`, independent of arrival
order — confirms "order-tolerant."

### N1 — wrapper wording — CLOSED

Contract 1 now reads "a thin wrapper over the ALREADY factored `encodeBytecode`
(`mk/encode.go:50`; `Encode` is untouched — r1 N1)" — matches N1 exactly.

---

## New-defect / contradiction checks (as asked)

1. **Does Acceptance's test list match Contract 3's per-surface decisions exactly?** Yes for the
   five surfaces both sections name — no surface promised a marker/modal in Contract 3 but
   missing from Acceptance, and no acceptance-only surface. But this consistency is because
   *both* sections consistently omit the same sixth consumer (`gui/multisig.go:102`) — matching
   sections don't rescue the I1 gap above.
2. **Is the corpus `warning_text` byte-identical to the quoted spec text?** Yes — confirmed above
   under I2/M1, `EQUAL: True`, ASCII-only, independently diffed in Python and independently
   raster-measured against the real harness.
3. **Contradiction between "every modal answers BACK" and "non-blocking"?** None. Both describe
   the *same* modal instance's two exits: r1's pre-verified-sound item 6 established
   `ErrorScreen.Layout` binds Button1 (back) and Button3 (ok) to the same single `dismissed`
   bool, so BACK and OK are equivalent, both dismiss, both proceed. "Answers BACK" and
   "non-blocking" are the same fact stated twice, not competing claims.

## Machine-check log

- Diffed spec-quoted `warning_text` against `mk/testdata/csid_ext_v0.1.json`'s
  `SEED_pinned_12345_ef12f` row and against `crates/me-cli/src/csid_warn.rs`'s format template,
  in Python: byte-identical, ASCII-only.
- Added `gui/probe_r2_csid_warning_test.go` calling the real `assertModalBodyFits` /
  `errorScreenBody` harness (`/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go test
  ./gui/ -run TestProbeR2CsidWarningBodyFits -v`) against the exact 309-char string: PASS, 258
  chars drawn in full, headroom 302 (margin 80). Deleted; `git status --short` empty after,
  `HEAD` unchanged at `2337ed3`.
- `grep -rn "offerChunkedMK1"` / `bundleGatherFlow` / `extractSuppliedMd1` across `gui/*.go`:
  confirmed all six r1-named call sites still exist at (or near) their cited lines, and that
  `gui/multisig.go:102` is a live, reachable sixth caller not mentioned anywhere in the current
  spec text (full-file grep for "multisig.go:102", "Engrave Multisig", "extractSuppliedMd1":
  zero hits).
- Read `gui/multisig_supply.go:12-32` (`extractSuppliedMd1`) in full: confirmed unconditional
  refusal on any `cardMK1`/`cardMS1` presence, regardless of csid status.
- Spot-checked `buildPlateCensusLines`, `buildPlateInventoryLines` (`multisig_build_census.go`),
  `buildPayloadCardsLines` (`multisig_build_payload.go`) exist as real functions; `mk/mk.go`
  reassembly loop confirmed order-tolerant (slot-indexed, not append-ordered).
