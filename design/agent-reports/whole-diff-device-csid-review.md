# Whole-diff adversarial review — SH2 device csid-mismatch warning

**Target:** `git -C /scratch/code/shibboleth/sh-worktrees/dev-warn diff 2337ed3..HEAD`
(one commit `952712a`; 9 modified `gui/` + 2 `mk/` files, 1 new test file,
1 new emu driver). **Spec:** `design/SPEC_device_csid_warning.md` (as amended
2026-09-01: screenshot gate CLOSED, marker form FROZEN). **Implementer report:**
`design/agent-reports/impl-device-csid-warning.md`.

**VERDICT: 1 Critical / 1 Important / 4 Minor / 3 Nit — DOES NOT SHIP AS IS.**
Both blocking findings are text-only remedies (a committed operator README, a
test docstring, one spec acceptance row); no production behaviour needs to
change. Everything the diff actually *does* on the device is correct: no
clean-card or md1-flow regression exists anywhere in the diff, and the R6
warning text is byte-identical to the Rust host.

---

## What I machine-checked before writing a word (do not re-derive)

Toolchain: `/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go`
(`go` is not on PATH on this box; the repo is nix-provisioned).

| check | result |
| --- | --- |
| `gui-shard-test.sh ./gui/ 24` | **ok — all 1056 tests** across an asserted 24-way partition, 22s wall |
| all 72 non-`gui` packages (`go test`, incl. `cmd/emu`) | green, exit 0 |
| `go test ./mk/...` | ok 0.036s |
| `go vet ./mk/...` | clean |
| **Mutation A** — deleted the `offerChunkedMK1` comparison block (`gui/bundle.go:298-303`) | **9 FAIL / 13 PASS** on `-run 'CSID\|Csid\|ChunkedField\|DecodeGathered\|BuildPolicyGather'`; every clean-twin control correctly stayed green; restored byte-identical, `git status` clean, `grep -rn MUTATION-GATE gui/ mk/` empty |
| **Mutation B** — deleted BOTH census/inventory markers | **exactly 1 of 1056 tests fails** (see C1) |
| warning text vs Rust host | `csidMismatchWarningText` == `crates/me-cli/src/csid_warn.rs:28 chunk_set_id_mismatch_warning` **byte-identical**, and == the corpus `warning_text` (309 chars, ASCII-only) — checked in Python against both sources independently |
| 4 NDEF tags | TLV `0x03…0xFE`, TNF=1, type `T`, MB=1 ME=1, zero-length lang; payloads unwrap to the corpus `strings` **byte-exact** for all four |
| README regeneration commands | re-run under bash — all four files **reproduced byte-identically** (`cmp -s`) |
| every line citation in the implementer report | resolves exactly (`mk/encode.go:64`, `gui/mk1_inspect.go:57`, `gui/bundle.go:49/63/78/91/115`, `bundle_flow.go:46,364`, `wallet_policy.go:108`, `multisig_build.go:201`, `multisig_build_census.go:53,89`, `multisig_build_payload.go:295`, `multisig_verify.go:793`, `singlesig_verify.go:153`); `grep -c '^func Test' gui/csid_warning_test.go` = **21** |
| `mk.Encode` untouched | confirmed — the diff inserts `DerivedChunkSetID` *before* `encodeBytecode`; `Encode` bytes unchanged |
| verify verdict + note renders | ink **8185 → 11555** px with the note appended (floor 6000, no blanking); `gui/widget/label.go:41` handles `'\n'` explicitly |
| emu driver | drives the **real** flow (home → NFC chunk 0 → `mk1 key` chooser → Inspect key → "Captured 1 of 2" → chunk 1 → modal), *and* a real Engrave Bundle gather → Done → modal → review marker. **No shortcut; no labelling problem.** It asserts and throws rather than only capturing. |

---

## C1 (Critical) — the Build-Policy restore-doc / plate-census marker is unreachable, and a passing test plus a committed operator README both claim it works

**Scenario.** The operator follows
`design/journeys/csid-tags/README.md`'s closing paragraph at the hardware gate:

> "Same two tag pairs also exercise the bundle-gatherer surfaces … plus the
> `[csid 12345!ef12f]` marker on the review list and (Build Policy only) the
> plate census / restore doc / payload-cards lines"

They gather the mis-stamped cosigner card in Build Policy, walk to "Plate Count"
and then to the restore document — and **no marker appears on either**, because
neither screen can ever receive a gathered card.

**Mechanism (complete call-graph proof).** `csidMismatch` is set in exactly one
place — `gui/bundle.go:298` in `offerChunkedMK1`. `buildPlateCensusLines` and
`buildPlateInventoryLines` have exactly three call sites between them:

- `gui/multisig_build.go:477` and `:578` — `cardsOut` from
  `buildEngraveTail` (`gui/multisig_build_tail.go:129`), whose return is
  `multisigEngraveCardsMulti(ms1s, mk1s, engraveMd1)` — **device-minted**
  `bundleCard` literals (`gui/multisig_engrave.go:35,43,50`);
- `gui/multisig.go:274` and `:381` — `cardsOut` from `supplyEngraveTail`,
  same `multisigEngraveCardsMulti` constructor;
- `gui/singlesig.go:236` and `:349` — `cards` from `singleSigEngraveCards`
  (`gui/singlesig_engrave.go:20`), also literals.

None of the three constructors sets `csidMismatch`, and none of the three flows
routes gathered cards there — Build Policy's cosigner cards supply *keys to the
policy*, they are never plates this device cuts. So `csidMarker(c)` at
`gui/multisig_build_census.go:53` and `:89` returns `""` on every production
path, always.

**Mutation confirmation.** Deleting both `csidMarker(c)` calls and running the
full 1056-test suite fails **exactly one** test —
`TestBuildPlateCensusLinesMarksCSIDMismatch` (`gui/csid_warning_test.go:387`),
which constructs the mismatched card itself and calls the two helpers directly.
Nothing that drives a flow notices.

**Why this is Critical and not Minor.** The row is the one the spec **bolds** as
"the funds-most path" and names as "`buildPlateInventoryLines` (the RESTORE DOC —
a mis-stamped id archived there is the name-drift hazard the host cycle
documented)", and the acceptance row "Build Policy incl. census/inventory/payload
lines" is satisfied by a test that cannot fail for the reason it claims. It is a
false coverage claim on a load-bearing row, and it has already propagated into a
**committed, operator-facing document** that will be read during the imminent
flash gate — where an unmet expectation either fails the gate spuriously or gets
rationalised away.

**Root cause worth recording.** The host cycle's hazard was "diagnostics that
name plates by id". `grep -rn 'ChunkSetID\|chunk-set id\|csid' gui/*.go | grep -v
_test.go` shows that **before this diff no device surface named a plate by
chunk-set id at all** — the premise did not transfer from host to device, and the
spec imported it unexamined across three R0 rounds.

**Remedy (text only, no behaviour change).**
1. Delete the closing "(Build Policy only) the plate census / restore doc"
   clause from `design/journeys/csid-tags/README.md`; keep payload-cards and the
   review list, which *are* reachable and *are* proven live.
2. Correct `TestBuildPlateCensusLinesMarksCSIDMismatch`'s docstring
   (`gui/csid_warning_test.go:382-386`) — it currently asserts it covers "the
   restore-doc consumer named 'the funds-most path'". State that it is a
   helper-level pin and that no production flow feeds these two functions a
   gathered card.
3. Either drop the two `csidMarker(c)` calls at `multisig_build_census.go:53,89`
   or keep them with a comment saying they are defensive-only today. **Do not**
   "fix" this by routing gathered cosigner cards into the plate inventory — a
   restore doc must list only plates this device cut.
4. Amend the spec's Contract 3 bullet and the matching Acceptance clause, and
   file a FOLLOWUP noting the host→device premise gap so the next cycle does not
   re-import it.

---

## I1 (Important) — the on-device acceptance procedure omits two navigation steps, and the failure mode is a silent no-op

**Scenario.** The operator, at the flash gate, follows
`design/journeys/csid-tags/README.md` "Tap order" literally:

> "At the mk1 'Inspect key' door: the FIRST tap shows only 'Captured 1 of 2.
> Scan the next chunk.'"

It does not. Tapping tag1 at the home screen dispatches through
`StartScreen.Flow → engraveObjectFlow → mdmkFlow` (`gui/gui.go`), which shows a
**`mk1 key` / "Choose action" chooser** (`mdmkFlow`, `choices = ["Inspect key",
…]`). "Captured 1 of 2" appears only after selecting row 0. The README's step 1
skips that tap entirely.

Worse, step 2 ("Tap tag3 and tag4") gives no way back. After the pinned card,
`mk1DisplayFlow`'s Back returns to `mdmkFlow`'s **own chooser loop**, not to the
home screen — a **second** Back is required to leave `mdmkFlow`. Neither
`mk1DisplayFlow` (documented "Read-only: no engrave, no NFC") nor `ChoiceScreen`
runs a scanner, so a tag tapped from either screen is **silently ignored**: no
message, no progress, nothing. That is indistinguishable from a dead build, and
it is the classic "looks like a hang" shape this constellation has been bitten by
before.

**This is not speculative.** The implementer's own emulator run hit exactly this
and fixed it *in the driver* — `cmd/emu/shots_csid_warning.js:145-150` carries
the comment "TWO Backs to reach home … a SECOND Back there is what leaves
mdmkFlow itself" — and the fix never reached the operator-facing document. The
same imprecision sits in the spec's Acceptance section ("after the SECOND tap …
the first tap correctly shows only capture progress").

**Remedy.** In `README.md`'s "Tap order", make the inspect walk explicit:
tap tag1 → the `mk1 key` / "Choose action" chooser appears → confirm "Inspect
key" → "Captured 1 of 2. Scan the next chunk." → tap tag2 → warning modal. Then:
dismiss the modal, Back out of the card display, **Back again** out of the
chooser to the home screen, and only then tap tag3. Add one sentence stating that
a tag tapped from the chooser or the card display does nothing at all, so a
non-response there is expected rather than a fault.

---

## Minor

**M1 — the "byte-exact / host-verbatim" assertion is case- and whitespace-blind.**
Every warning-text assertion goes through `uiContains(content, row.WarningText)`,
and `uiContains` (`gui/gui_test.go`) lower-cases both sides and strips spaces
from the **needle only**, while `op.Drawer.ExtractText` returns content with all
spaces already removed. Probed directly: extracted content is
`"warning:thiskeycard'sstamped…"`, and `uiContains(content,
strings.ToUpper(WarningText))` returns **true**. So an uppercased or
differently-spaced warning would pass every test in this cycle, and Contract 2's
"Byte-exact R6 parity" is not what is gated. *(I byte-checked the string against
both the Rust host and the corpus independently — the code is correct today; the
test is weaker than its claim.)* **Remedy:** one pure line, e.g. in
`TestCSIDFixturePairIsWhatTheSpecClaims`:
`if got := csidMismatchWarningText(wantDeclared, wantDerived); got !=
pinned.WarningText { t.Errorf(...) }`.

**M2 — nothing guards the consumer set; a seventh consumer would be silent.**
Contract 3's stated mechanism is "the result travels ON `bundleCard` as data, so
no downstream surface can be silent by omission" — but the data riding along does
not render anything; each surface must opt in, and no test enumerates the
`bundleGatherFlow`/`bundleGatherFlowResume` call sites (verified: no such test
exists in `gui/*_test.go`). `TestMultisigVerifyFlowWiresCSIDNoteIntoVerdicts`
covers one function only. It is also (a) comment-blind-spotted — `funcBody`
returns raw source, so a `+csidNote` inside a comment satisfies it — and (b)
brittle in the wrong direction: `strings.Count(body, "multisigVerifyFailureText(err)+csidNote") != 2`
**fails on a legitimate third comparator-FAIL site**. **Remedy:** a `funcBody`
guard that enumerates the six gather call sites and asserts each caller either
calls `showBundleCSIDMismatchNotices` or appears on a named exempt list (Engrave
Multisig, the two verify readbacks), so a seventh consumer trips it.

**M3 — the notice re-fires on every re-entry into a gather.** The modal is called
after the gather *returns*, and all three callers loop back into the gather:
`bundleFlow` on review-Back (`gui/bundle_flow.go:56 gathered = cards; continue`),
`walletPolicyFlow` on **five** `continue` paths (`gui/wallet_policy.go:112,117,
122,125,128`), and `multisig_build.go`'s `buildStepGather` ("this step can run
more than once now"). `bundleGatherFlowResume` re-offers `prev` cards' strings
through `offer()`, so the mismatch is recomputed and the notice fires again —
once per mismatched card, every round trip. An operator bouncing off Wallet
Policy's "Supply exactly one wallet policy (md1) card" refusal re-reads the same
309-char warning on each attempt. Warning-only and non-blocking, so not a
correctness defect; worth a one-line "already shown this session" latch if it
irritates at the gate.

**M4 — the clean-twin negative controls are weaker than their positive twins.**
`TestBuildPolicyGatherSilentOnCleanTwinLive` (`gui/csid_warning_test.go:548-561`)
pumps **8** frames after Done looking for `"warning:"`, while its mismatch twin
uses `pumpUntil(..., 64)`. A notice that fired on frame 9 would be missed. Same
shape, smaller: the positive live test proves the modal is reachable within 64
frames, so the control should use the same budget.

---

## Nit

**N1 — miscount in the implementer report.** Deviation 2 says the verify note was
applied at "5 sites total". The actual count is **7**: `singlesig_verify.go:225,
229,231,235` (three comparator-FAIL variants + PASS) and
`multisig_verify.go:1171,1196,1208` (two comparator-FAIL + PASS). The wiring is
correct; only the number in the record is wrong.

**N2 — `gofmt -l` is not empty on go1.26.7.** It reports `gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go` — none
touched by this diff (they are absent from `git diff --stat 2337ed3..HEAD`).
Pre-existing / toolchain-version artefact against the repo's pinned `go 1.25.10`,
same class as the two `go vet ./gui/` `testing.ArtifactDir` findings the
implementer already documented. Not this cycle's, but the report's "gofmt -l on
every touched/new file: empty" should say *touched* files explicitly, since a
bare `gofmt -l gui/` is not empty.

**N3 — README regeneration snippet is bash-only.** It uses herestrings (`<<<`),
which this operator's `fish` does not have. Labelled ```sh, so cosmetic — and I
confirmed all four files reproduce byte-identically when run under `bash`.

---

## Per-deviation verdicts (implementer report §"Deviations, and why")

1. **Modal-firing granularity — fires once, after the whole gather returns.**
   **ACCEPTED.** Verified live (`TestBuildPolicyGatherShowsCSIDMismatchNoticeLive`
   drives `buildMultisigPolicyFlow` end to end off a real payload and I re-ran
   it). It is the only shape available: all six consumers share
   `bundleGatherFlowResume`, so a notice inside the loop would also fire for
   Engrave Multisig and the two verify readbacks, which Contract 3 forbids. The
   inspect flow (`decodeGathered`) still fires at true set completion — i.e. on
   the second tap — so the spec's on-device acceptance wording holds for the
   flow it describes. See M3 for the re-entry consequence.
2. **Verify-note scope (PASS + comparator-FAIL only).** **ACCEPTED as a scope
   call**, count corrected to 7 (N1). The readback-accounting refusals
   (`singlesig_verify.go:166` "Need one key card (mk1) and one descriptor (md1)
   read back."; `multisig_verify.go`'s `extractReadbackMd1AndMk1s` failure) carry
   no note. Defensible — those refusals are about a different problem — and the
   note would be additive information, not a wrong answer, so it does not gate.
3. **Marker form `" [csid 12345!ef12f]"`.** **ACCEPTED and now moot** — the spec
   as amended 2026-09-01 records the operator's "Screenshots perfect!" and
   freezes the wording and marker form as rendered. Pinned by
   `TestCSIDMarkerForm`.
4. **Contract 4 required no new production code.** **ACCEPTED.**
   `TestClassifySingleMK1Refuse` confirmed present and unmodified at
   `gui/bundle_test.go:113`; `TestExtractSuppliedMd1`'s "any mk1 present ->
   refuse" subtest confirmed at `gui/multisig_supply_test.go:42`.

---

## Explicitly cleared (do not re-review)

- **No clean-card or md1-flow regression anywhere in the diff.** Every list
  surface appends `csidMarker(c)`, which returns `""` for a matched card and for
  every non-`cardMK1` kind, so `bundleReviewFlow` (`bundle_flow.go:364`),
  `buildPlateCensusLines`/`buildPlateInventoryLines`
  (`multisig_build_census.go:53,89`) and `buildPayloadCardsLines`
  (`multisig_build_payload.go:295`) emit byte-identical output for clean sets.
  No index arithmetic changed (`i+1` / `perCard[i]` untouched). Both verify flows
  append `csidNote`, which is `""` when nothing mismatches. The archived
  restore-doc rendering for clean cards is unchanged. 1056/1056 green confirms it.
- **`mk/` is additive.** `DerivedChunkSetID` is a new export over the
  already-factored `encodeBytecode`; `Encode` is byte-unchanged. The
  `DerivedChunkSetID` error path is a silent no-op, and it is **unreachable in
  practice**: `decodeBytecode` (`mk/decode.go`) rejects `stubCount == 0` and the
  count is a byte, so `encodeBytecode`'s two stub guards cannot fire on a decoded
  card, and path/xpub/fingerprint all round-trip (pinned by
  `TestEncodeGoldenRoundTrip`). Rust's `derived_chunk_set_id` returns `Option`
  and skips identically — semantic parity, not just textual.
- **Consumer enumeration is complete for scan-derived cards.** The only other
  `mk.Decode` sites are the verify comparator (`multisig_verify.go:573,581`,
  content comparison, no display), `reStubMk1` (`template_engrave.go:43`,
  device-derived), `buildCosignerCards`/`walletPolicyKeyCards` (downstream of a
  gather already covered), and `sysw/confirm.go:89` (computes confirmed/
  unconfirmed indices only; a mis-stamped card is correctly still *confirmed*).
- **Modal semantics.** `showNotice` == `showError` == `showModal`, which returns
  on any button (`ErrorScreen.Layout` treats Button1 and Button3 identically) or
  on `ctx.Done`. It mutates no state, does not touch the gatherer, and cannot
  swallow a flow — `decodeGathered` still returns `(card, true)` afterwards
  (`TestDecodeGatheredWarnsOnCSIDMismatch` asserts exactly that). Long bodies
  scroll (`Warning.Layout` arrow chips); the note is appended **after** the
  verdict, so a clip can only ever hide the note, never the verdict.
- **Engrave Multisig's silence is correct by prior refusal.**
  `gui/multisig.go:102` gathers, then `extractSuppliedMd1(cards)` at `:106`
  refuses on any mk1 *before* anything renders. Pinned twice.
- **Tag payloads and their README procedure are otherwise sound** — four files,
  two per card, byte-exact after NDEF unwrap, regeneration reproducible, mode
  0600, order-tolerance claim correct (`mk.reassemble` slots by index). Only the
  navigation steps (I1) and the census/restore-doc claim (C1) are wrong.
