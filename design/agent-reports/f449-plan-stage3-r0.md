# F-449 stage 3 plan — R0 architect review

**Artifact:** `design/IMPLEMENTATION_PLAN_f449_stage3_go_port.md` (engrave master; plan baselines dm `cf35d61a`, fork `7b6f2fb`).
**Reviewer:** independent R0 (opus). **Date:** 2026-09-23.

**VERDICT: GREEN — 0 Critical / 0 Important / 5 Minor / 2 Nit.**

## How this was checked

The plan's blocks were applied **independently** to fresh `git archive` exports under
`/scratch/code/shibboleth/.tmp/r0-s3/{dm,fork}`, task by task, each on the previous task's tree. Neither real checkout was touched.

- **Task 1 (dm):** `md vectors` emitted exactly 14 new files and no `M` line. `liana_taproot` = `md1gzfdsssj5qqcreqygvtam2lm5fenw4v` / `4092d84212a00181e4044300`. The kofn ids match the plan byte for byte (`116659077988624006ef630ddf96c982`, `f99cc42e…963f`, `cb913ed2…88d5`). After Steps 5–6 the only REDs were the two snapshot tests, whose `.snap.new` files carry `is_nums: true`, `unspendable_kind: liana_unspendable` and `schema: md-cli/2`. After Step 7: `phase-gate.sh` **exit 0**. The Step 9 survivor reproduced (`want == xkey` → `true`: 2/2 green).
- **Fork, Tasks 3–6:** md **167 → 169 → 179**. Task 5 Step 2 gave exactly the five REDs listed. Vendoring gave `260 files, 53 vectors`, with 14 new files. Whole gate: gofmt = the five-file baseline, **55 non-gui `ok`**, `RESULT: ok -- all 1380 tests ran across 24 shards`.
- **Mutations re-run by me:** M2a, M5a, M7, X3 (Liana grouped with **Slot**, a G6 variant) and X4 (`needsV8` ignores the root) are all caught. Two new survivors, X1 and X2, are below.
- Rust's `md-cli/tests/vector_corpus.rs::vectors_output_matches_committed_corpus` regenerates the corpus and diffs it. So the new goldens bind **Rust** as well as Go.
- **Journey probes** (scratch gui tests, since deleted) are recorded under Q5.
- **Not run:** the TinyGo size measurement, the device, and the emulator.

## Findings

### Critical — none. Important — none.

### Minor

**m1. Decoding kind 0 at version 8 is pinned by no gate in either language (a §8.10-class blind spot on the read side).**
Counterexample, measured:
- **X1 (Go).** In `readNodeDepth`'s Tr arm, `if kind {` becomes `if kind || true {`. The reader still consumes the bit but always yields `InternalKeyLianaUnspendable` at v8. **md 179/179 pass.**
- **The same mutation in Rust** (`tree.rs` read arm: `let _ = r.read_bits(1)?; InternalKey::LianaUnspendable`) passes **1536/1536**.
- **X2 (Go).** A kind bit is also read for a *Slot* key at v8, which Rust reads only iff `is_nums`. md 179/179 pass.

Both gates pin the **write** side only (`TestKindBitPolarityIsPinnedOnTheWire`, and Rust `the_kind_bit_polarity_is_pinned_on_the_wire_not_just_round_tripped`). Every vector read at v8 is kind 1. The affected input is a non-minimal v8 card (kind 0, or a slot key, at v8). No encoder mints one, but SPEC §6 row 6 makes it *accepted at decode*. A Go reader with X1 would read such a card as a different wallet from Rust, and every suite would stay green. There is no current wrong result, hence Minor.

Remedy, **Rust first** (Rust-primary rule):
1. In Task 1, extend the Rust polarity test to read each golden back through `read_node`: `[0x06,0x00]`@8 → `NumsPoint`, `[0x07,0x00]`@8 → `LianaUnspendable`, `[0x06]`@4 → `NumsPoint`. Add one Slot-at-v8 golden (no kind bit).
2. Then give `TestKindBitPolarityIsPinnedOnTheWire` the same read-back via `readNode(c.want, 0, c.version)` → `c.ik`.

This takes about ten lines per side, and both X1 and X2 then fail. If it is not folded in now, file it with F-655 (next dm release) plus a Go convergence item.

**m2. The plan's description of the stage-3 inspect screen is wrong** (Review Focus 2, and "What no gate covers" bullet 6).
The plan says a kind-1 wallet on inspect "still shows the NUMS key-path line". Measured on the scratch end state, `gatheredDescriptorFlow(keyed_tr_liana_kofn_recovery)` shows:
1. `Complex policy - display only.`
2. then `md1 descriptor / Complex policy - cannot display safely. / Keys: 4 @0 73c5da0a m/48h/0h/0h/3h<0;1>/* …`

There is **no key-path line, no Policy id and no address.** Its kind-0 twin (`keyed_compose_preset_kofn_recovery`) gets `Policy id: c5788d70…` and the address flow. The key-path line exists only on the template and composer consent screens (`policySummaryLines`, `gui/template_engrave.go:159-164`), and there it reads `Key-path: none (script paths only)`, which is true of kind 1.

Nothing on screen is false, so this is not a device finding. Remedy: correct both bullets, and add to Task 8's sweep that kind 1 loses the inspect `Policy id:` line until stage 4. That line is the one §7 says distinguishes the two kinds.

**m3. `TestVersion8CardsAreReadOnEveryRoute`'s single-card half asserts only absences.**
A flow that exits without drawing the display passes it. Measured today, the screen does reach `Complex policy - cannot display safely. Keys: 3 @0 - m/48h/0h/0h/2h<0;1>/*`. Remedy: add a positive `uiContains(all.String(), "Keys: 3")` (or the display title).

**m4. A stage-3 fork `main` can be flashed before stage 4 lands, and on it the engrave half of §7a.3 does not exist yet.**
Other cycles flash fork main, so "no flash in this stage" does not keep this build off a board. On such a build, the Wallet Policy program accepts a kind-1 card set. It shows the version-derived id and `This device can't derive addresses for this policy.` (`noAddressLines`), then `bundleEngrave`s the **supplied strings verbatim**.

That is not worse than silence: the statement is true and the steel is a copy of Rust-minted cards. But it is the SPEC §7a row-2 outcome, and nothing records it. Remedy: add one line to Task 8's §9 status ("stage 3 alone lets the device copy kind-1 cards without an address; the engrave refusal is stage 4"). No code change.

**m5. The citation count in Task 5 Step 7 is wrong.**
"Every recursive `readNodeDepth(r, kiw, depth+1)` (13 sites)" measures **10** at `7b6f2fb`. The compiler catches any site that is missed, so the only fix needed is the count.

### Nit

- **n1.** Task 3 Step 4's new `case md.InternalKeyLianaUnspendable:` Skip arm in `gui/taproot_script_path_test.go` is unreachable for both kind-1 vectors. `md.TapLeavesChunks` errors first (`tap leaf shape not supported for address derivation`), so they skip at `:50`. The real gate is the `stillUnsupported` entries, which work (G6 and X3 are caught). Soften the comment, or accept the arm as future-proofing.
- **n2.** Engrave master is now `255cb976`, not `bebb532a` (stage 4a's F-656 filing). F-654 and F-655 are still unused, so Task 8's IDs hold. Re-grep at filing time as the plan says.

## Rulings on the author's scope calls

- **(a) Porting §2's recipe into `md/`: SOUND.** It is pure and network-free, a semantic port of `nums.rs` `liana_unspendable_xpub`. It reproduces all 9 Liana goldens. In stage 3 it is test-only and dead-code eliminated. It is what makes D1' a byte-equality check rather than §8.10's structural match (M7 is caught only by `TestLianaReductionRefusesANearMiss`, re-measured).
- **(b) The bundle surface: SOUND.** `classify` → `clsDrop` → "Not an md1/mk1 card." is the same false sentence on a second channel. The fix reuses the one helper. Its tests assert the positive message and pass on my tree.
- **(c) The `md1_encoding_id` assertion: SOUND.** It is reached by all **48** keyed records (48 PASS subtests, and all 48 fail when the record value is perturbed). It is the only id that hashes the header, and so the only one that sees the version.
- **(d) The address half of §7a.3 in stage 3: SOUND, and forced.** The signature change leaves the one production caller no silent option. Kind 1 grouped with NUMS (G6) and kind 1 grouped with Slot (X3, which would derive from `@0`) are both caught by `TestEveryKeyedVectorReachesAnAddress`. The journey probe shows no address.
- **(e) The §6 mint refusals move to stage 4 (F-654): SOUND.** Go `Reassemble` → `computeEncodingID` → `encodePayload` (`md/identity.go:11-17`). `encodePayload` carries only the structural checks that Rust's `encode_payload_for_identity` keeps (`encode.rs` `Admission::SkipPolicy`). The kind-1 refusals sit under `Admission::Enforce` only. No device path mints kind 1 in stage 3:
  - `StripToTemplate` is fed only `EncodeSingleSig` output (`gui/singlesig.go:204`) or the device-authored sortedmulti (`gui/multisig_build.go:445`);
  - the supply and wallet-policy programs engrave supplied strings verbatim.
- **(f) F-643, "no Go counterpart, wording only": SOUND.** Device correction goes `codex32.Correct` (`gui/gui.go:1296`) → `confirmCorrectionFlow` (`gui/codex32_polish.go:336`) → full decode. A structural check past the header is unimplementable for an unknown version. One consequence the ruling should name: stage 3 *creates* the device's own analogue. A miscorrection that lands on v12 now shows `This firmware cannot read md1 version 12.` That is why the observation-only wording in Global Constraints is correct, and it supports the ruling.

## The five questions

1. **Cross-language correctness.**
   - The Go reader and writer follow `tree.rs` line for line; I compared them by reading.
   - Identities are version-derived at all four writers. Go equals Rust on `wallet_policy_id`, `wallet_descriptor_template_id` and `md1_encoding_id` for both kind-1 records.
   - Rust's decoder has no kind-1 validation that Go lacks.
   - The new vectors pin kind 1 in both directions:
     - encode: `TestEncodePayloadGoldens` / `TestEncodeMD1StringGoldens` / `split` equality;
     - decode: `ik` asserted in the dispatch tests, and the wire_version population check (`v8 > 0`) in Rust;
     - and in Rust by corpus freshness.
   - The mutations the gates miss are X1 and X2 (m1). Both are confined to non-minimal v8 input.
2. **Funds and restore.**
   - No v4 card becomes unreadable: the v4 goldens are unchanged, 46 prior keyed records still agree, and `errWireVersion` had no identity-matching caller outside `md_test.go`.
   - Kind 1 is never treated as NUMS or as a Slot key (G6 and X3 are caught).
   - Kind 1 never gets an address in stage 3.
   - The mint-refusal-reaches-decode class is avoided (ruling e).
3. **False passes.** The conformance gate asserts and does not only parse: M7 and the encoding-id perturbation both go red. Weak spots are m3 and n1. Every other new test was shown to fail under a mutation, either by the plan (re-measured where listed above) or by me.
4. **Execution order.**
   - Task N runs on Task N-1's tree. I executed them sequentially, and every boundary count matched.
   - Real dm is at `cf35d61a` and real fork at `7b6f2fb`, both clean.
   - The Go side pins the Rust merge commit in two places: `md/testdata/compose_vectors.provenance.json` (commit + per-file sha256, where the sha256 is what the test enforces) and the `md/bits.go` package doc (`<TASK-1 MERGE SHA>`, which Task 7 greps for).
   - No downstream repo reads dm's corpus live, so Task 1's push reddens nothing else.
5. **Operator journey on a stage-3 board.**
   - *Load a v8 Liana plate set:* every chunk is added. The set completes and the screen shows "Complex policy - display only." then the keys.
   - *View the policy:* key list only, with no Policy id (m2).
   - *Check an address:* none offered. Through the Wallet Policy program the device says "This device can't derive addresses for this policy." (m4).
   - *Supply the policy to derive a cosigner leg:* the mk1 stub is version-derived and equals Rust's.
   - *Scan a v12 card:* gather (first chunk or later), inspect single, and bundle all show `This firmware cannot read md1 version 12.` A v12 record seeded from the payload is silently not added, the same as before.
   - No divergence produces a wrong outcome worse than silence.

## Verified sound (beyond the above)

- The Task 1 fixes are test-only, and the Rust harness recogniser is byte-equality.
- The loader's `unspendable_kind` mapping fails on an unrepresentable value.
- The pin test's widened `liana_` prefix brings directory-scan coverage.
- `sysw/confirm.go` is left unchanged, which matches Rust `chunk_key`.
- `gatherUnsupportedVersion` joins a shared enum, but every `default:` arm that consumes it maps it to a drop, never to success.

ready for implementation: yes
