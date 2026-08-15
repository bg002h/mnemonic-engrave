# Adversarial review — session decisions across four repos (2026-08-15)

Reviewer: fable (independent context). Scope: rule BLOCKER / NOT A BLOCKER on D1–D8
as briefed. S0b not re-reviewed; bip48 reversal not re-litigated. Every claim below
cites a file/line read or a command run in this review.

---

## 1. BLOCKERS

### B1 (= D1) — the BIP deletion removed true, load-bearing information and replaced it with a false general claim

**The decision.** `mnemonic-key/bip/bip-mnemonic-key.mediawiki` (commit a38a908) deleted the
"toolkit slot-XOR" note and added: *"Deriving from the whole payload is what gives distinct
cards distinct chunk_set_ids with no further help."*

**Why it blocks.**
- The toolkit still does slot-XOR, everywhere: `derive_mk1_chunk_set_id_for_slot(stub, slot)` at
  `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs:94-96`,
  called with per-slot indices at synthesize.rs:543, :708, :880, :1271 (grep run this review).
- It is not merely cosmetic: `verify_bundle.rs:977` **binds** on it — the `mk1_template_stub_bind`
  check requires each supplied card's csi to equal `derive_mk1_chunk_set_id_for_slot(stub, slot)`.
- The replacement claim is **false for the toolkit's own card class**. The self-multisig path
  (synthesize.rs ~695-710, read this review) builds N `KeyCard`s that are byte-identical across
  slots — same stubs, same fingerprint, same path, same xpub; the comment states "all xpubs are
  identical, so the old per-fingerprint scheme collided ALL cosigners". The csi is the ONLY
  per-slot distinction. Payload-hash derivation gives all N cards the SAME id, and verify-bundle's
  csi-grouping (audit I10, synthesize.rs:82-86) collapses. "Distinct cards" with identical
  payloads exist, and the payload hash does not distinguish them — that is exactly what the
  deleted paragraph documented and the new text denies.
- A reader reproducing a toolkit-emitted card from the BIP's formula now gets a silent mismatch
  with no explanation anywhere in the document.

**Authority half of D1: NOT a blocker.** The file is the project's own pre-submission draft;
editing it is within the project's authority. The lapse is process — a normative-doc edit folded
into a code commit with no review round. This review serves as that round; no separate remedy
needed beyond the content fix.

**Minimal resolution** (one edit to the BIP, one to the SPEC; no code):
1. In `bip-mnemonic-key.mediawiki` §chunked-header, restore a corrected note after the MAY
   paragraph, e.g.:
   > *Note (toolkit slot-XOR): the reference bundling toolkit derives per-card
   > `chunk_set_id` values as `base XOR slot`, where `base` comes from the leading 20 bits
   > of the first `policy_id_stub` — not from the payload hash above. Cards for different
   > cosigner slots can be payload-identical (e.g. self-multisig, or keyless template
   > stubs), so a payload-derived id would collide across slots; the slot XOR keeps
   > per-cosigner chunk groups distinct, and the toolkit's verify surface binds on it.
   > Byte-for-byte reproduction of a toolkit-emitted card therefore requires the toolkit's
   > derivation. Interop is unaffected: the field is opaque and only mismatch-checked
   > within a single card's chunk set.*
2. Weaken the false sentence in both the BIP and `design/SPEC_mk_v0_1.md:124` ("two distinct
   cards get distinct ids without further help") to "two cards with distinct payloads get
   distinct ids"; payload-identical cards need an external distinguisher.

No other decision reviewed rises to blocker.

---

## 2. NOT BLOCKERS, but fix when convenient

- **D2 privacy note**: add one sentence to SPEC §2.5 + BIP: chunk_set_id is a deterministic
  function of the payload (leaks 20 bits of SHA-256(payload) in chars 2-5) and is not a privacy
  mechanism; mk1 strings are plaintext. Can ride the B1 edit.
- **D5 doctrine**: write the discrimination rule into `seedhammer/oracle/record.go` (header doc +
  the record.go:366-368 refusal text): *re-pin that can reach the device path (payload, firmware,
  walk script) → re-walk; comparison-source-only re-pin → rebuild via gaterecord, valid only
  because the tier-2 derived-census comparison re-runs under the new pin.* Today that rule exists
  only in the session that applied it, and the committed message prescribes the opposite remedy.
- **D5 mechanism**: have `cmd/gaterecord` run `DeriveExpected`+`CompareCensus` at write time when
  the inputs file carries an `expect` block, refusing on mismatch — moves an existing check
  earlier so a `-force` rebuild cannot outrun the comparison. ~15 lines in main.go's `run()`.
- **D8a pins.json `_comment` is self-contradictory**: it still says mnemonic-secret "had 3
  modified files… weaker attestation… marked", but the ms entry now says
  `checkout_clean_when_recorded: true` (re-pin to ddfa497). The comment describes the superseded
  0.14.0 pin. Fix the comment before this evidence file is next cited.
- **D6 argv**: `ms derive` has no `--phrase-stdin` (help output read this review; only
  `--passphrase-stdin` and stdin for the ms1 positional exist), so piping a raw phrase is not
  currently possible. Add `--phrase-stdin` upstream in ms (matches the constellation's
  "seed NEVER on argv" convention, cf. mnemonic-gui wire_shape_snapshot.rs:196), then switch
  `expect.go:msDerive` to pipe. Until then the comment at expect.go:281-283 is honest and the
  exposure is structurally bounded (see §3/D6).
- **D3 naming**: add a one-line comment or micro-test in mk-codec naming the cross-version
  property ("a 0.4.x card is a card with an arbitrary csi; the pinned-csi corpus vectors prove
  decode accepts them") so the existing coverage can't be refactored away unnoticed.
- **D7 policy**: state the additive-fields-within-schema_version policy explicitly in ms
  (format.rs doc or SPEC), so the next additive field doesn't need this argument re-made.
- **D8b record schema**: GateRecord binds oracle commits but nothing binds the device-under-test
  (no firmware/emu-build field in WalkResult or GateRecord — walk JSON head read this review).
  Pre-existing, not introduced by any D; consider a field at the next record-schema bump.
- **D8c fork provenance**: `seedhammer/mk/mk.go:5` still pins "mk-codec 0.2"; refresh on next
  sync per the Rust-primary rule.

---

## 3. CORRECT AS DECIDED

- **D2 (ship without a privacy analysis → no reversal needed).** The "unlinkable on the wire"
  premise was wrong: mk1 payloads are plaintext, so two 0.4.x encodings of one card differed
  only in the 4 csi chars (+checksum) and were already trivially linkable by payload comparison.
  The only new channel is header-only confirmation of a *guessed* payload (20 bits), and for
  engraved steel whoever reads chars 2-5 reads the whole string. SPEC §2.5's pre-existing
  reuse-MUST already specified identical re-encodings; the CSPRNG was the defect (settled).
  A note suffices (§2); no reversal.
- **D3 (no explicit cross-version test).** The gap is already covered de facto: 19 of 41 corpus
  vectors pin arbitrary csis — vector[0] pins 0x12345 where the payload-derived value is 0x83bb2
  (both computed this review) — and `tests/vectors.rs:220` decodes the pinned strings. Decoders
  MUST accept any 20-bit value (SPEC line 114), and a 0.4.x card is wire-identical to an
  arbitrary-csi card. Real coverage, wrong label; naming it is polish (§2).
- **D4 (GENERATOR_FAMILY roll + V0_1_SHA256 re-pin).** `V0_1_SHA256` is consumed only by
  mk-codec's own `tests/vectors.rs:41` self-check; searched all four repos plus gui/toolkit —
  no external consumer pins that hash (the fork vendors no mk corpus; mk.go carries a comment
  pin only). NOT rolling the token would have broken corpus regeneration reproducibility
  (generator output ≠ disk). Correct.
- **D5 (rebuild the record from the saved walk after the ms re-pin).** Sound as executed, and
  this was the question I probed hardest:
  - It was a *rebuild through `NewRecord`*, not an edit: walk invariants re-checked, fresh pin
    resolution; `git diff 04f2716..c94c135 -- oracle/gaterecords/` shows only `recorded_at` and
    the ms oracle entry changed — the walk binding (sha 2dc9c525…) was inherited byte-identical.
  - The ms 0.14→0.15 re-pin cannot reach the device path: nothing in the emulator/payload path
    invokes ms; `cmd/emu/sysw_cards_payload.bin` is unchanged since 3ea08f9 (git log run).
  - Both dangerous misuse axes are machine-guarded and the guards RUN: payload drift → NewRecord
    refuses (record.go:214-217); derivation drift → `TestS0CensusMatchesTheDerivedExpectation`
    (expect_test.go:74-80) re-derives via the pinned binaries and byte-compares the record's
    census on every suite run. It skips only when a binary is absent (expect_test.go:28-31);
    all three are installed at ~/.cargo/bin with sha256 exactly matching pins.json (sha256sum
    run this review: md 9ef480ad…, mk 030ca218…, ms e63d9cb5…), so the settled-green fork suite
    executed this comparison under ms 0.15.0.
  - What the decision defeated is only the refusal *message's* over-strict remedy — hence the §2
    doctrine fix, so the real rule outlives the session.
- **D7 (`script_type_defaulted`, schema_version "1").** Verified non-optional in `DeriveJson`
  (format.rs:110-124). Consumers: seedhammer expect.go parses 3 fields via Go json (unknown
  fields ignored, expect.go:272-277); mnemonic-gui does not parse `ms derive --json` at all —
  its drift gates target gui-schema flag surfaces and the `mnemonic` toolkit's wire shapes, and
  pinned-upstream.toml pins ms at ms-cli-v0.13.0, so the gui sees nothing until a deliberate pin
  bump, where its choice/flag drift gates fire by design. Additive-within-version matches the
  constellation's stated policy (schema_check.rs GuiSchemaRoot comment). Correct.
- **D6 (comment, for now).** The exposure is structurally bounded beyond the comment: seed words
  reach `msDerive` only from a committed inputs file whose documented policy is
  words-for-published-vectors, digest-for-real (cmd/gaterecord/main.go:54-58), and
  `SeedWords()` refuses digest-only seeds from derivation (inputsfile.go:89-100) — real material
  is pushed off this path by design. The stdin switch (§2) is right but is currently impossible
  without the upstream ms flag, so it cannot gate S1.
- **D8 invited review of `expect.go` / `inputsfile.go`**: no blockers found. Notable *good*
  decisions: closed `ExpectKind` set refusing unknown kinds (expect.go:126-129); recorded origin
  CHECKED against the template-derived path (expect.go:155-159); `CompareCensus` refuses a
  vacuous pass (expect.go:202-205); absolute oracle paths with the shell-alias rationale
  (expect.go:88-92); `DisallowUnknownFields` on the inputs file (inputsfile.go:50).
- Rust-primary optics on mk-codec 0.5.0: the Go port already had the behavior Rust adopted, but
  the change landed in Rust first with vectors and Go needed no edit — compliant in letter and
  spirit (convergence toward the SPEC, not a Go-led change).

---

## 4. RANKED ACTION LIST

1. **B1** — restore the corrected slot-XOR note in the BIP and fix the false "no further help"
   sentence in BIP + SPEC §2.5 (blocker; one paragraph + one sentence, text supplied above).
2. Write the D5 re-pin discrimination rule into record.go's doc + refusal message
   (the rule must outlive the session that invented it), and fix the stale pins.json `_comment`
   (separate small commits).
3. Wire the expect-block census comparison into `cmd/gaterecord` before S1 mints new records.
4. Add `--phrase-stdin` to `ms derive` upstream; switch `expect.go` to pipe.
5. D2 one-sentence privacy note (ride the B1 edit).
6. D3 naming comment/test in mk-codec; D7 additive-policy sentence in ms.
7. On next fork sync: refresh mk.go's provenance pin; consider a device-identity field at the
   next gate-record schema bump.
