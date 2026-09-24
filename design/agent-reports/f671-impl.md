# F-671 implementation report

**Status: DONE.** Fork `bc7cbe7` (branch `f671-consent`, on `f3586db`). Engrave `80735588` (branch `f673-evidence`, touching only the spec and FOLLOWUPS). Nothing has been pushed or merged.

## The fix

- **One predicate.** `composerLianaRefusesSeating(st)` in `gui/composer_review.go` returns `len(composerSharedSeedInPath(st)) > 0`. `composerSharedSeedInPath` is the finder that puts §8g ("Liana will refuse it") on the mapping review. The rule is written once, and nothing restates it.
- **Consent.** `composerConsentLinesFor` gained a fourth parameter, `lianaRefusesSeating`. Its only production caller is `composerConsentFlow`, which passes `composerLianaRefusesSeating(st)`. When the flag is true, the kind-1 arm draws `composerCopyLianaKeyPathSameSeed`. That body replaces "Liana (as of v15.0) and Bitcoin Core import this form" with "SAME SEED, SAME PATH: one seed fills more than one slot of one path, so Liana will refuse it. Bitcoin Core imports this form." The rest of the body is unchanged.
  - This is the one consent input that is not read from the md1. That is safe because `composerSelfCheck` runs first and refuses unless every decoded slot fingerprint, and the shape, match the seating. The doc comment says so.
  - All 26 test call sites now pass `false`.
- **Key-path choice.** `composerUnspendableRows(st)` draws `composerCopyUnspendableRowLianaSameSeed` when the predicate fires: "Liana key: Bitcoin Core imports it. SAME SEED, SAME PATH: Liana will refuse it." On first entry nothing is seated, so the original row stands. On re-entry (Back from the stub, mapping or consent) the seating is kept, and the old row was false for this wallet.
- **Other print sites checked:**
  - The inspect screen (`md1_inspect.go`: "Key path: Liana key") and the engrave summary (`template_engrave.go`: "Key-path: none (Liana key)") name only the kind and make no import claim. Unchanged.
  - The drop-cause text "Liana would not import this policy (…)" is a refusal. Unchanged.
- **NUMS.** No body claims Liana imports a NUMS wallet: §8f says "Liana needs its own unspendable key instead", the NUMS row says "Liana and Nunchuk do not", and §8x class 2 applies. This is now pinned by `TestComposerNoConsentClaimsLianaImportsANUMSWallet`.
- **Copy table and spec.** Both new bodies are in `composerCopyTable` as §8y rows, and the declared-body count went from 92 to 94. SPEC §8y quotes both verbatim. That was checked mechanically: the normalized spec blockquotes contain all four §8y Liana bodies.

## Tests (TDD: red first)

All are in `gui/composer_f671_test.go`.

- **`TestComposerConsentDoesNotClaimLianaImportsASameSeedWallet`** walks the real `composerFlow` by touch and nav-slot taps only: tr, kofn-recovery, Liana key, then Type a seed, FROM PAYLOAD and Skip, with the payload seed seated in all four slots, through the mapping review to consent.
  - The mapping review must carry §8g.
  - The consent must not carry either Liana import claim, and must carry the same-seed body.
  - Row taps use hit targets read from the drawn frame. Nav uses `tapNavSlot`. No Up/Down events are used.
  - It was RED before the fix, failing on "the consent still says 'Liana (as of v15.0) and Bitcoin Core import this form'", with the mapping control passing.
- **`TestComposerKeyPathChoiceDoesNotClaimLianaImportsASameSeedSeating`** checks the rows for an unseated state (the control) and a seated state.
- **`TestComposerKeyPathChoiceSameSeedRowFitsTheFirstPage`** checks that both rows are on page 1 under the picker's own layout.
- **`TestComposerNoConsentClaimsLianaImportsANUMSWallet`** covers the NUMS half.
- **Mutations.** Each was verified to have applied (grep count 1), then restored and touched.
  - Predicate forced false: the flow test and the chooser test go red.
  - Consent arm forced off: the flow test goes red.
  - Chooser arm forced off: the chooser test goes red.
- **Whole gui package.** `gui-shard-test.sh ./gui/ 24` ran 1403/1403, and the partition was verified exhaustive. The first run failed on `TestEmulatorWalksQuoteCopyThatStillExists`, because the new walk waits for "Slots @0, @1 and @2 are the same seed.", which is argument-dependent. I added `composerCopySameSeedThreshold([0,1,2],2,3)` to that gate's corpus.
- **Rest of the module:** green.
- **`gofmt -l .`:** exactly the five-file baseline.
- **`go vet ./gui/`:** reports only the pre-existing `testing.ArtifactDir` go1.25 notes, in files this change does not touch.

## Emulator walk

- **The gate as given:** `capture_composer.py --arm both --emu …/f671/cmd/emu` printed "all legs matched the host" (keyed-A, keyed-B, keyless, liana).
- **New arm.** `cmd/emu/shots_composer.js` has a new `liana-same-seed` arm. It loads the composer payload, builds tr kofn-recovery with the Liana key, and seats the payload seed (b8688df1) into all four slots.
  - It asserts §8g on the mapping review.
  - It asserts the consent has the same-seed body and not the claim.
  - It then goes Back to the path list, takes Done, and asserts that the re-entered key-path choice shows the same-seed row with 2 tappable rows.
  - As a control, the unseated first entry still shows the original row.
  - Result: PASS. With `composerLianaRefusesSeating` forced false and the wasm rebuilt, it FAILS with "F-671: the consent claims Liana imports…". After restoring and rebuilding, it passes again.
- **Not wired into `capture_composer.py`'s `--arm` choices.** That file is outside my permitted file set. I drove the arm through capture_composer's own `drive`/`serve` harness from a scratchpad script (`walk_same_seed.py`, 25 lines). Wiring it in means adding a `liana-same-seed` choice, an `EXPECTED` entry and a leg whose `expect` is `read_keyed()["A"]`.

## Screenshots examined

All are under `design/journeys/shots/` (git-ignored):

- `l04-consent-p1.png` is the claim case (the Liana arm, nothing seated). It shows "Liana (as of v15.0) and Bitcoin Core import this form."
- `s05-consent-p1.png` is the same-seed case. It shows "SAME SEED, SAME PATH: one seed fills more than one slot of one path, so Liana will refuse it. Bitcoin Core imports this form." The body fits the page.
- `s06-key-path-same-seed.png` shows the re-entered choice. Both rows are visible and left-aligned, and the Liana row reads "Bitcoin Core imports it. SAME SEED, SAME PATH: Liana will refuse it."

## Concerns

- The first-page geometry test for the new row was not mutation-tested.
- The Nunchuk sentence in the same-seed body is carried over unmeasured for same-seed seatings. Leaf keys stay distinct because each slot is at a different account, so the existing sorted-order claim should still hold.
- The plan-3 verdict table (F-671's owning-phase remedy) will supersede these hand-written bodies. Until then, this is the one predicate to reuse.
