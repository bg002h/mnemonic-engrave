# R0 — SPEC_device_csid_warning.md, round 1

**Artifact:** `design/SPEC_device_csid_warning.md` @ `594c3e3`
**Ground truth:** `/scratch/code/shibboleth/seedhammer` @ `origin/main` `2337ed3` (tree verified clean
before and after; all probe files removed)
**Question asked:** is it sound, complete, and implementable against the REAL fork code — and can
every acceptance gate fail?

## Verdict

**0 Critical / 4 Important / 3 Minor / 2 Nit — does NOT close.**

Every finding below was reproduced by running code against the fork tree, not read off a comment.
Toolchain: `/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go` (repo pins `go 1.25.10`).

---

## What I verified SOUND (do not re-derive)

These were the largest implementability risks in the brief and all of them measured clean. State
them as settled in the next brief.

1. **Contract 1 is implementable with zero risk to `Encode`.** `Encode` does NOT interleave:
   `mk/encode.go:39-45` is literally `bytecode, err := encodeBytecode(card)` then
   `encodeChunks(bytecode)`. `encodeBytecode` is already a standalone function at `mk/encode.go:50`,
   and `top20` at `mk/encode.go:331`. `DerivedChunkSetID` is a wrapper over two existing
   unexported functions.
2. **Contract 1's operand matches the host exactly.** Host (shipped):
   `mk_codec::derive_chunk_set_id(mk_codec::bytecode::encode_bytecode(card))` —
   `crates/me-cli/src/csid_warn.rs:63-69`. Go equivalent: `top20(encodeBytecode(card))`. Same
   semantic, R6 parity holds at the operand level.
3. **The corpus-extension parity test is expressible, and I ran the whole thing.** I decoded all
   21 rows' `strings` through the fork's `mk.Decode`, re-encoded each with `encodeBytecode`, and
   compared to `canonical_bytecode_hex`: **21/21 byte-identical**, and `top20` of each reproduced
   the row's `derived_csid` including `SEED_pinned_12345_ef12f` (declared `12345`, derived
   `ef12f`). Every row parses through `codex32.MKDataSymbols` + `ParseHeader` with no `mk-codec
   0.2` vs `0.5` drift. Contract 1's pin is buildable as written.
4. **The fixture pair exists and is exactly right.** `SEED_pinned_12345_ef12f` and
   `SEED_plate_b_ef12f` carry **identical** `canonical_bytecode_hex` (84 bytes) — same key, one
   mis-stamped, one clean. Every row carries a `strings` array (2 chunks each), so gui tests can
   drive the real scan flow from the corpus.
5. **Contract 2's declared id IS in scope at the call site.** `decodeGathered(ctx, th, g *mk1Gatherer)`
   — `gui/mk1_inspect.go:241` — holds `g.setID uint32` (`gui/mk1_inspect.go:47`), and the two call
   sites (`:186`, `:206`) are both inside `mk1GatherFlow`, upstream of `mk1DisplayFlow`
   (`gui/gui.go:2720-2721`). A notice placed in `decodeGathered` lands before the card display.
6. **The notice idiom exists and answers BACK.** `showNotice` (`gui/slip39_polish.go:44`) →
   `showModal` → `ErrorScreen`. `ErrorScreen.Layout` (`gui/gui.go:388-399`) binds `ok` to Button3
   and `back` to Button1 and **both return the same single `dismissed` boolean** — so BACK
   dismisses and continues, which is exactly the non-blocking semantics R1 requires. Contract 2's
   "every modal answers BACK; proceeding continues to the card" is literally true of this type.
7. **The md1 sibling citation is accurate.** I checked this because the mk-side
   `errChunkSetIDMismatch` (`mk/mk.go:192-194`) is only a chunk-to-chunk equality check and looked
   like a mis-citation. It is not: `md/chunk.go:284-291` genuinely re-derives
   (`deriveChunkSetID(computeEncodingID(d)) != expCsid`) and refuses. The gap statement is correct
   — mk1 has no declared-vs-derived comparison anywhere.
8. **Contract 3's comparison point has both operands.** `offerChunkedMK1(csid uint32, str string)`
   (`gui/bundle.go:177`) holds the declared `csid` from `classify` and `card` from `mk.Decode` at
   `:195`. `bundleCard` (`gui/bundle.go:33-38`) is a plain struct that can take an additive field,
   and the review surface already renders a per-card status token — `fmt.Sprintf("%d. %s OK", ...)`
   at `gui/bundle_flow.go:362`. The affordance is real. (Its *reach* is not — see I1.)
9. **The screenshot gate is scriptable, not a hypothesis.** `cmd/emu/nfc_js.go:43-50` exposes
   `shNFC.present(record)` which **queues**, and three existing drivers already inject mk1/md1
   chunk strings this way (`shots_seating.js:165,181`, `shots_walletpolicy.js:151`,
   `shots_tr_pathological.js:161`). Capture is `screenShot(shotURL, name)` +
   `design/journeys/capture_*.py` (`shots_operator.js:96-108`). No existing driver walks the
   **Inspect key** path specifically, so this cycle writes a new driver — but every primitive it
   needs is present and exercised.
10. **On-device acceptance uses the right format.** I ran the real binary
    (`mnemonic-engrave/target/release/me`) on both mis-stamped chunk strings: it accepts them and
    emits `03 74 D1 01 70 54 00 6d6b31...`. Feeding those exact bytes back through the fork's own
    `ndef.NewMessageReader` → `NewRecordReader` returns the mk1 string verbatim, which
    `gui/scan.go:91` classifies via `codex32.ValidMK` → `mdmkText` → the mk1 flows. The chain is
    end-to-end real. `me`'s converter path is also not a `csid_warn` call site
    (`crates/me-cli/src/csid_warn.rs` is wired only into `bundle.rs:308`, `seal/record.rs:260`,
    `sysw/record.rs:233`), so it will not refuse or interfere with building the fixture tags.

---

## Findings

### I1 — Contract 3 marks ONE surface; the comparison point feeds SIX, and four never show it

**Severity: Important.** Coverage gap; the spec's own guarantee is unmet after implementation.

Contract 3 puts the warning marker "on the bundle review surface". That surface is
`bundleReviewFlow` (`gui/bundle_flow.go:359-366`) and it has exactly **two** callers:

- `gui/bundle_flow.go:45` (`bundleFlow`)
- `gui/wallet_policy.go:125`

But the comparison point Contract 3 specifies — `offerChunkedMK1` (`gui/bundle.go:177`), reached
through `bundleGatherFlow` — has **six** consumers, and the other four never call
`bundleReviewFlow` at all:

| call site | renders `bundleReviewFlow`? |
| --- | --- |
| `gui/bundle_flow.go:45` (Engrave Bundle) | yes |
| `gui/wallet_policy.go:125` | yes |
| **`gui/multisig_build.go:184`** (Build Policy cosigner gather) | **no** — goes to `mk1CosignerCards(cards)` and its own census |
| **`gui/multisig_verify.go:781`** (verify readback) | **no** — `extractReadbackMd1AndMk1s` |
| **`gui/multisig.go:102`** (Engrave Multisig) | **no** — `extractSuppliedMd1` |
| **`gui/singlesig_verify.go:145`** (verify readback) | **no** — `singleSigReadbackCards` |

**Failing scenario:** an operator runs Build Policy and taps the two chunks of
`SEED_pinned_12345_ef12f` as a cosigner card. `offerChunkedMK1` computes the mismatch exactly as
Contract 3 says. The card is then consumed by `mk1CosignerCards` and the build census
(`gui/multisig_build_census.go:53`), neither of which renders the marker. **The operator sees
nothing** — the identical silence the spec exists to remove, on the flow with the most at stake.
Three further `bundleCard` list surfaces exist and are likewise unmarked:
`buildPlateCensusLines` (`multisig_build_census.go:53`), `buildPlateInventoryLines`
(`multisig_build_census.go:89` — the **restore doc**), `buildPayloadCardsLines`
(`multisig_build_payload.go:295`).

The spec's gap section claims covering "the two mk1 reassembly points" closes "the last silent
surface in the constellation". Implemented literally, that claim is false on four of six paths.

**Remedy:** name the surfaces explicitly rather than "the bundle review surface". At minimum
decide, in the spec, for each of the four unmarked consumers: mark, or state why silence is
correct there (e.g. the readback paths compare against an expectation and may not need it; Build
Policy plainly does). If the restore doc and census are excluded by the "any engraving-flow
change" out-of-scope clause, say so — right now the clause and Contract 3 are in unresolved
tension.

---

### I2 — the frozen draft wording contains an em dash, which blanks the ENTIRE modal body

**Severity: Important.** Real defect in the artifact's own frozen text; no ASCII constraint stated.

`SPEC_device_csid_warning.md:40` freezes:

> `... The plate was minted with a pinned id — re-mint it without --chunk-set-id to fix.`

That `—` is U+2014. Per `gui/font_coverage_test.go:31-35`, `font/bitmap/bitmap.go:33` sets
`indexLen = unicode.MaxASCII` and `glyphFor` rejects `int(r) >= indexLen`, so **every non-ASCII
rune is unrenderable on every bitmap face**, and — per the same file and
`gui/bundle_gather_refusal_test.go:19-24` — an unrenderable rune does not blank its line, it
**blanks the whole body**, while `uiContains` still returns true because the text ops were
submitted.

**Measured, via the package's own `errorScreenBody` harness:**

| body | result |
| --- | --- |
| spec draft **as written** (em dash) | **5004 ink px — under the 6000 `buildWalkRasterFloor`; body blank** |
| spec draft with an ASCII `-` | 154 chars drawn in full, headroom **418** (margin 80) — passes |
| host `warning_text` verbatim | 258 chars drawn in full, headroom **302** — passes |

**Failing scenario:** implementer copies the frozen wording verbatim into `gui/mk1_inspect.go`.
The operator meets a notice screen with a title and nothing under it. This *is* caught before
ship by `TestProductionStringsAreDrawable` (`gui/font_coverage_test.go:209`), which is why it is
Important rather than Critical — but the spec is the document that freezes this wording and puts
it through an operator screenshot gate, and it names no constraint.

**Remedy:** replace the em dash with an ASCII `-` (or a full stop), and add an explicit
constraint to the spec: *device strings are ASCII-only; non-ASCII blanks the whole modal body
(`gui/font_coverage_test.go`)*. The spec's prose uses `—` throughout, so the copy-paste risk is
concrete.

---

### I3 — Contract 4's discriminator does not exist, and both available proxies are wrong

**Severity: Important.** Unsound assumption; the natural implementation silently drops a real warning.

Contract 4 says "header type decides, not a value comparison". Correct in intent — but
`mk1Gatherer` stores **no `Chunked` field**:

```go
type mk1Gatherer struct {           // gui/mk1_inspect.go:44-49
	set    map[int]string
	total  int
	setID  uint32
	primed bool
}
```

`offer` reads `h.Chunked` and discards it (`:57-63`). Note it also has **no unprimed `!h.Chunked`
guard**, unlike its md1 twin which returns `gatherIgnored` for exactly this case
(`gui/md1_gather.go:36-38`) — so a single-string mk1 primes the gatherer and reaches
`decodeGathered`. Measured: `offer(single) = gatherAdded; primed=true total=1 setID=0x00000
complete=true`.

An implementer therefore reaches for one of two proxies, and **both are wrong**:

- **`g.setID == 0`** — this is precisely what `mk encode --chunk-set-id 00000` produces on a
  genuinely chunked, genuinely mis-stamped card. **Failing scenario:** operator pins csid `00000`,
  taps both chunks, derived id is some non-zero value, the mismatch is real — and the warning is
  suppressed because the guard mistook it for a single-string.
- **`g.total == 1`** — a *chunked* header can legally declare `total == 1`.
  `mk/mk.go:83-87`: `total := int(syms[6]&0x1f) + 1`, guarded only by `total > maxChunks || index
  >= total`, so `syms[6]=0, syms[7]=0` yields `Chunked: true, TotalChunks: 1`, and
  `reassemble` (`mk/mk.go:175-224`) decodes it. `Encode` never emits that shape, but a foreign or
  hostile minter can.

**Remedy:** state in Contract 4 that `mk1Gatherer` gains a `chunked bool` recorded at prime time,
and that the check keys on it — not on `setID` or `total`. One field; it just has to be in the
spec, because neither proxy is visibly wrong at the call site.

---

### I4 — Contract 4's acceptance gate cannot fail

**Severity: Important.** A gate that passes in both worlds; blocking per the severity rule on
"defects in what a tool claims to have done".

Acceptance asks for a gui test where "single-string is silent". Measured:

- The only single-string mk1 in the tree is `singleMK1Fixture` (`gui/bundle_test.go:38-60`), and
  its own doc comment says *"A real mk1 key card is ALWAYS >=2 chunks (xpub_compact 73B > the 56B
  cap), so a single mk1 is MALFORMED... No single mk1 exists in-tree, so we synthesize one"* — ten
  filler symbols.
- `mk.Decode([]string{singleMK1Fixture})` → **`mk: malformed payload padding`** (run, not inferred).

**Failing scenario:** the test drives the inspect flow with that fixture. `decodeGathered`
(`gui/mk1_inspect.go:242-246`) takes the `err != nil` arm, shows `"Can't decode this key set."`
and returns — **the comparison site is never reached**. The assertion "no csid warning appeared"
therefore passes: with the comparison present, with it deleted, and even with it wrongly applied
to single-strings. It measures the decode failure, not Contract 4.

This also makes the acceptance's own mutation clause misleading: deleting the comparison would
fail the two *mismatch* tests, but the single-string test would stay green either way, so it
contributes nothing to that proof.

**Remedy:** either (a) drop the single-string gui test and pin Contract 4 at the unit level
instead — assert directly that the discriminator from I3 returns "no check" for
`Chunked == false`, which *can* fail; or (b) mint a single-string mk1 that actually decodes (needs
a bytecode ≤ the single-string capacity — note the fixture comment says a real card cannot fit,
so this is likely (a)). Either way the spec must say which, because as written the gate is
unrunnable for its stated reason.

---

### M1 — "condensed for the panel" is unnecessary; the host wording fits verbatim

**Severity: Minor.**

Contract 2 says the content is "condensed for the panel", implying the host string does not fit.
Measured: the host's shipped `warning_text` — 258 normalized chars — is drawn **in full** on the
first frame with **302 characters of headroom** against an 80-char margin. The panel is not the
constraint.

Since R6 ("same warning content") is settled and all three host binaries already print a
byte-identical string guarded by a drift test
(`crates/me-cli/src/csid_warn.rs:29-36` + `wording_pin_matches_the_frozen_r6_text`), the fork could
carry that same string and get byte-exact parity for free instead of a fourth independent
paraphrase to keep in sync. Recommend adopting the host wording verbatim (ASCII already — see I2)
unless the operator redlines it at the screenshot gate.

### M2 — the on-device acceptance undercounts the tags

**Severity: Minor.**

Acceptance says "tap an NDEF tag carrying the pinned corpus card" and "tap the clean-twin tag" —
singular both times. `me`'s NDEF record header is `0xD1` = MB=1 **ME=1**
(`crates/me-cli/src/ndef.rs:5`), i.e. one record per message, and I confirmed a single-record read
back through the fork's parser. Each corpus card is **2 chunk strings** → **2 tags**. The procedure
is 4 tags, 4 taps, and the warning appears on the **second** tap of each pair (set completion), not
the first. An operator following it literally taps once, sees "Captured 1 of 2" and no warning,
and may record a failed acceptance.

**Remedy:** spell out 2 tags per card, 4 total, warning on completion of each pair, order
irrelevant (`reassemble` is order-tolerant, `mk/mk.go:198-206`).

### M3 — "fixtures come FROM the corpus, never hand-minted" is unsatisfiable for Contract 4

**Severity: Minor.**

The vendored corpus has **zero** single-string rows — all 21 rows carry exactly 2 strings — and
`mk.Encode` cannot produce one (`encodeChunks`, `mk/encode.go:236-251`, splits an 84-byte
bytecode at the 53-byte `chunkedFragmentBytes` cap, so always ≥2). The only single-string mk1
that can exist in a test is hand-minted, which is what `singleMK1Fixture` already is. Scope the
rule to the mismatch/clean fixtures where it is both meaningful and satisfiable.

### N1 — Contract 1 overstates the work

**Severity: Nit.** "the encoder's existing bytecode builder, **factored**, not duplicated" implies a
refactor. It is already factored: `encodeBytecode` at `mk/encode.go:50` is a standalone function
and `Encode` (`:39-45`) is a two-line composition. `DerivedChunkSetID` is a wrapper; `Encode` is not
touched at all. Worth saying, so a reviewer of the implementation does not go looking for a
refactor diff that should not exist.

### N2 — single-string mk1 is *refused* in the bundle flow, not merely unwarned

**Severity: Nit.** Contract 4 says "structurally no check, no warning". True in the inspect flow.
In the bundle flow it is refused outright — `classify` returns `clsSingleMK1Refuse`
(`gui/bundle.go:80-82`) and it never reaches a gatherer. Different outcome, worth one clause so
the two flows are not conflated.

---

## Machine-check log (what was executed, not read)

- `go test ./mk/` with a probe decoding all 21 corpus rows' `strings` → `Decode` → `encodeBytecode`
  → compare vs `canonical_bytecode_hex` and `top20` vs `derived_csid`: 21/21 identical.
- `go test ./gui/` with a probe running the spec's frozen body, an ASCII variant, and the host
  `warning_text` through `errorScreenBody` / `bodyDrawnFully` / `modalHeadroom`.
- `go test ./gui/` probe: `mk.Decode([singleMK1Fixture])` and `mk1Gatherer.offer(single)` state.
- `mnemonic-engrave/target/release/me --hex` on both mis-stamped chunk strings.
- `go test ./nfc/ndef/` probe: `me`'s NDEF bytes → `NewMessageReader` → `NewRecordReader` →
  exact mk1 string.
- `go test ./mk/ ./nfc/ndef/` after probe removal: both `ok`. `git status --short` empty.

All probe files were deleted; the fork tree is byte-identical to `2337ed3` as found.
