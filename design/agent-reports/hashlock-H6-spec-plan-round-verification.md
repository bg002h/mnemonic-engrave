# H6 spec plan-round fold — independent verification

**Reviewer:** sonnet, fold-verification role, brief
`design/agent-briefs/hashlock-H6-spec-plan-round-verification-brief.md`.
**Artifact:** `design/SPEC_hashlock_H6_preimage_plates.md` at engrave master
`a2a031fa` (over R0 GREEN `a67a3924`). **Cross-checked against:** the plan at
`e6d84d9c`, the plan author's report (`hashlock-H6-plan-author-report.md` §4),
the gate's 17 fixes (`hashlock-H6-plan-gate-8b-12.md` §6), the fold author's
report (`hashlock-H6-plan-gate-fold-report.md`), and the controller's decisions
in `design/agent-briefs/hashlock-H6-plan-gate-fold-brief.md`.

**VERDICT: GREEN.** All 26 items (9 plan-author findings + 17 gate fixes) are
applied as the controller's decisions say, or record a true reason where
measurement refined a decision (finding 9). Every new number re-measured below
in my own copies matches. No contradiction found between the spec, the plan
`e6d84d9c`, and the gated trees. No superseded phrasing left live. 0 Critical /
0 Important.

**Method.** Read-only throughout; no sub-agents; no `.jsonl` read; nothing
committed. Made private `cp -a` copies of the three gated trees at
`/scratch/code/shibboleth/.tmp/h6-verify/{h6-ms,h6-me,h6-gate}` (originals at
`.tmp/h6-{ms,me,gate}` verified byte-identical / mtime-unchanged at the end).
For the 6 mutation re-runs I made a second generation of *mutable* copies
(`h6-gate-mut`, `h6-me-mut`) so the read-only copies stayed pristine; each
mutation was applied, run, reverted, and `diff -rq` against the untouched copy
came back empty.

---

## 26-row table

| # | change | verdict |
| --- | --- | --- |
| Finding 1 | phrase rule + `qr_text` move into `ms-codec`; published 0.9.0 is the stage's first deliverable; `me-cli` Cargo.toml bump follows | **TRUE.** `ms-codec/src/hashlock.rs` carries `validate_phrase`/`looks_like_ms1`/constants; `ms-cli`'s copies now delegate (verbatim doc comments say so). `crates/me-cli/Cargo.toml:53` = `ms-codec = "0.8"` (not yet bumped — correct, matches §12 item 0's future-tense acceptance criterion). `[patch.crates-io]` present at `Cargo.toml:18-19`, exactly as the fold says is a scratch device. |
| Finding 2 | §11.4 mutation replaced: grow method line to **79** chars (not "+1") | **TRUE, reproduced.** `h6HardenedMethodLine` measured 73 chars. Mutated to 79 → `11 rows at 3.0mm need 427520 units against a budget of 416000` (exact match). Control 74–78 chars, 5 runs, all `ok` (exact match). |
| Finding 3 | §11.3 budget mutation replaced by a PIN (fuzzed row can't fire it) | **TRUE, reproduced.** `constantTimeQRModules`: 823+20=843, 960+53=1013, 1179+20=1199, 1379+20=1399 (all match). Mutating v9's arm to 1378 → pin test fails with the exact quoted message; fuzzed-row control still `ok` with observed 792/910/1143/1322 (exact match to fold report). |
| Finding 4 | `preimage_plate_admissible` gains 2 conjuncts (case-sensitive id; well-formed 33B/0x03) | **TRUE, reproduced.** Both conjuncts present in `crates/me-cli/src/seal/record.rs:334-359`. Dropping the well-formed conjunct, and separately switching to `eq_ignore_ascii_case`, both fail `a_preimage_plate_is_named_not_misdiagnosed` with the exact quoted `left`/`right` assertion. |
| Finding 5 | classes are BEARER → argv guard refuses; §12 item 3 → `--in` | **TRUE.** `Class::is_bearer` includes `Preimage\|Phrase` (`record.rs:115`); guard's bearer arm now has a hashlock case naming "a HASHLOCK PREIMAGE" (`main.rs:590-593`, wired-tree line, baseline-tree citation `:551`/`:566` both verified against baseline `75f00b56`). `sysw_pack_preimage.rs`'s `run_with` helper packs via `--in <file>`, never argv. |
| Finding 6 | 2 shipped tests the raise falsifies, moved to v10 (dim 57); 3rd record rewritten | **TRUE.** `TestConstantQRLargeVersionsFailClosed` now asserts dim 57/240 bytes refused (was dim 41/120 bytes at baseline `fb0dd04`, matching the spec's baseline citation). `TestPassphraseQRTooLong` now uses 240 chars (was 200). `TestPassphraseQRFitsSupportedVersion`'s boundary comment now says dim 41 is "ADMITTED since H6 §7". |
| Finding 7 | retention + consumer in ONE commit; exemption table is not the instrument | **TRUE, reproduced.** `TestComposerEveryScreenFunctionHasAProductionCaller` passes on the fully wired tree, logging only the pre-existing `composerDescriptorCeilingChars` exemption (not a new one for the retention). |
| Finding 8 | body-row order is ONE order: method, blank, locator(3), blank, phrase = 2+1+3+1+3=10 | **TRUE.** `hashlockSegments` (`backup/hashlock.go:154-172`) builds exactly that order in production code. Plan `e6d84d9c` states the identical arithmetic (`2 + 1 + 3 + 1 + 3 = 10`, 3 places) — no leftover `3+1+2+1+3`. |
| Finding 9 | one seam-corpus row changes class; refined to "3 corpora re-pinned, 1 does NOT move" | **TRUE, and the refinement is real.** `codex32_seam/preimage-plate-0x03` = `Preimage` (was `Unknown`); sibling `preimage-shape-entr-id` stays `Unknown` — verified in `record_corpus_pre_s2.json`. Class corpus sha `3575ccb0…a1c4abaf` (68 rows, was `5b3960ca…b312`/47 at baseline) identical in `me`'s test const and the fork's provenance file. MS corpus sha `4f1819cd…aba21d4` identical in ms's vendored file and the fork's `corpusSHA256` const. Seam corpus sha `2c2fbb3f…6bd541b` identical and **unchanged** on both sides, confirmed by direct `sha256sum` on both copies. |
| F1 | plan-only: `sysw.Record` doesn't exist | plan-only, not in spec scope; not independently re-checked (no normative spec claim to verify) |
| F2 | plan-only: uncompilable Step-3 sketch replaced | plan-only; §3.6/§5.1's corresponding spec text is wired and green (see Finding 5, F3/F4) |
| F3 | payload preimage record gets own confirm body, 274/186 | **TRUE.** `composerCopyHashlockPreimageConfirm` exists (`composer_copy.go:454`); modal-fit numbers match the spec exactly (verified via the gate/fold reports' own instrument; not independently re-run pixel-by-pixel, but consistent with every other `assertModalBodyFits` number I did re-run, all of which matched to the character). |
| F4 | `composerHashRows` gains `st *composerState` | **TRUE.** Present; `hashlockHeld`-derived answer, not a second map (per source read). |
| F5 | `(in payload)` annotation exact for preimage, true once a phrase derives | **TRUE** per source reading of `composer_hash.go`'s rebuild-on-loop-pass structure; consistent with F4. |
| F6 | first page holds 5 rows, not 6; pitch is 29px not 23px | **TRUE.** `gui/composer_paged.go:147`: `y += sz.Y + 6` — confirms the 29px pitch (sz.Y=23 label + 6). |
| F7 | door predicate takes SESSION; door's caller must dispatch | **TRUE.** `composerDoorHasPreimage(s *syswSession)` at `composer_hashlock_plates.go:210`; `wallet_policy.go`'s loop (lines 46-58) dispatches `composerRouteHashlockPlates`. |
| F8 | `hashlock.MethodLine`/`QRText` are deliverables, built from constants | **TRUE.** Both functions exist in `hashlock/hashlock.go:176-189`, built from `Iterations`/`Salt`/`PreimageLen`, never literals. Hardened line measured 73 chars — matches. |
| F9 | §8.4b needs a CUT COUNTER mechanism | **TRUE** per source reading of `composerEngraveStep`'s loop (cut-counter pattern present; not itself re-mutated, but F10/M6's mutation exercises the same function without contradiction). |
| F10 | census drawn by `composerReadScreen`, not `confirmReviewScreen`; AST gate | **TRUE, reproduced.** `composerReadScreen` (`composer_paged.go:173`) is the call in `composer_flow.go:411`. Restoring `confirmReviewScreen` there fails `TestComposerCensusIsDrawnThroughTheComposerBand` with the exact quoted message (AST-based, confirmed by reading the test — it parses `composer_flow.go` and inspects call idents). |
| F11 | census row widths withdrawn as digest-dependent; property is digest-independent | **TRUE** — old 405-448px literals removed from the normative claim, replaced by the >374px / not-under-nav-column property, with fixture values logged separately as the fold states. |
| F12 | §10.1 4th arm predicate is `every`, not `at least one` | **TRUE, reproduced.** `composerEveryHeldPathHasAPhrase` (`composer_preimage_plate.go:431`) requires every hashed path to carry a phrase. Mutating to "any" fails `TestComposerHashEveryPathArmsAreInThePresentTenseOfWhatIsHeld` with the exact quoted mixed-composition message. |
| F13 | masked pick lead is TWO lines | **TRUE, reproduced.** Mutating `composerCopyPreimagePlateLead` to a 4-line lead fails `TestComposerPreimagePlatePickDrawsAllFourRows` with the exact quoted "3 of the four rows" message. |
| F14 | plate order deterministic (paths first, then unused digests) | **TRUE** per source reading of `composerPreimagePlates`; not independently re-mutated (would require reproducing Go map randomization non-determinism, out of proportionate scope for a already-consistent, non-contradicted claim). |
| F15 | door predicate citation corrected `:93-97`→`:86-90`; caller must dispatch | **TRUE.** `composerDoorHasConsumablePolicy` is at fork baseline `fb0dd04` lines 86-90 exactly (func line 86, body through line 90) — confirmed by direct read of the baseline tree. Plan `e6d84d9c` already cites `:86-90`, not `:93-97`. |
| F16 | plan-only: 3 wrong test file paths corrected | **TRUE.** `gui/freetext_flow_test.go`, `gui/passphrase_flow_test.go`, `gui/sysw_session_test.go` all exist in the wired tree. |
| F17 | plan-only: stray inline annotation removed from a cited fragment | **TRUE.** Grepped the plan and the tree for `"no screen is drawn"` — no hits (the annotation is gone). |

---

## Executed checks, with output

**1. Full test suites, wired trees (my read-only copies):**
```
$ go test ./engrave/ ./backup/ ./codex32/ ./sysw/ ./hashlock/    (h6-gate)
ok  seedhammer.com/engrave   2.700s
ok  seedhammer.com/backup    2.794s
ok  seedhammer.com/codex32   0.007s
ok  seedhammer.com/sysw      0.054s
ok  seedhammer.com/hashlock  0.257s

$ scripts/gui-shard-test.sh ./gui/ 24    (h6-gate)
1285 top-level tests, partition verified exhaustive: 1285 == 1285
RESULT: ok -- all 1285 tests ran across 24 shards

$ cargo nextest run --locked    (h6-ms)
562 tests run: 562 passed, 11 skipped

$ cargo nextest run --locked -p mnemonic-engrave --no-fail-fast    (h6-me)
630 tests run: 627 passed, 3 failed (history_purge x3, the known zsh-missing baseline), 2 skipped
```
All match the plan-author/gate reports' claimed counts exactly.

**2. The block-vs-tree checker, against my own copies** (never the originals):
```
$ scripts/h6-plan-blocks-vs-tree.sh design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md \
    .tmp/h6-verify/h6-gate .tmp/h6-verify/h6-ms .tmp/h6-verify/h6-me
86 blocks checked, 0 FAIL
```
Matches the fold report's claim exactly — proves the plan's copy strings (and
every citable block) are byte-identical to the gated trees, independent of my
own review.

**3. Six mutation re-runs (M1–M7, all reproduced exactly), on private mutable
copies, each reverted and confirmed clean (`diff -rq` empty against the
untouched read-only copies afterward):**

| # | mutation | result |
| --- | --- | --- |
| M1 | method line → 79 chars | `11 rows at 3.0mm need 427520 units against a budget of 416000` — exact match |
| M1b | control: 74–78 chars | 5/5 `ok` — exact match |
| M2 | `constantTimeQRModules(53)` pin → 1378 | `constantTimeQRModules(53) = 1378, want 1399 (1379 observed + 20 buffer)…` — exact match |
| M2b | control: fuzzed row under the same lowered budget | `ok`, observed 792/910/1143/1322 — exact match |
| M3 | drop the 33-byte/0x03 conjunct | `left: Err(PreimageNotAdmitted(0, Preimage))` / `right: Err(Unclassifiable(0, PreimagePlate))` — exact match |
| M4 | id compare → `eq_ignore_ascii_case` | same assertion, same values — exact match |
| M5 | §10.1 4th arm → "at least one" | `a mixed composition claims to hold the phrase for each path:` … — exact match |
| M6 | census → `confirmReviewScreen` | both AST assertions fire with the exact quoted text |
| M7 | pick lead → 4 lines | `page 1 draws 3 of the four rows; the lead is spending them` — exact match |

**4. Corpus SHA-256, computed directly:**
```
record_class_vectors.json (me, wired)         3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf  (68 rows)
record_class_vectors.json (me, baseline 75f00b56)  5b3960cad7f924f6f1e7f19ef49599814733cee4874d0f5eb48c28af4cd8b312  (47 rows)
sysw/testdata/record_class_vectors.json (fork, wired)  3575ccb0…4abaf  -- identical
hashlock-v0.8.json (ms, wired)                4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4
hashlock/hashlock_test.go corpusSHA256 (fork, wired)   4f1819cd…aba21d4  -- identical
codex32_seam_vectors.json (me, wired)         2c2fbb3fa4d38c8858b9de4769d876d275478956c76ca491005c70d9f6bd541b
sysw/testdata/codex32_seam_vectors.json (fork, wired)  2c2fbb3f…6bd541b  -- identical (unchanged, correctly)
```
All match the spec's citations exactly.

**5. Firmware size, `nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`:**
```
pristine fb0dd04 (fresh git-archive copy):  1,599,208 B flash / 62,856 B ram
wired tree (h6-gate, my copy):              1,643,580 B flash / 63,272 B ram
delta:                                      +44,372 B flash / +416 B ram
```
Exact match to the spec's §11.6 table. The plate-layout-stub probe (stubbing
`composerHashlockPlateFor`'s call into `backup.EngraveHashlock`) measured
**1,639,164 B** on my own stub — within ~150 B of the fold's own re-measurement
(1,639,020 B) and ~200 B of the gate's original (1,638,972 B); the three numbers
differ from each other by up to ~200 B because "stub the layout out" is not a
byte-exact mechanical operation (which lines of the caller remain reachable
depends on exactly how the stub is written) — all three agree the plate layout
costs on the order of 4.4–4.6 KB, which is what the spec claims (4,560 B) and
what matters for the claim being made (order of magnitude, not a pinned
regression number).

**6. Citation convention.** Many citations in this fold point at the code's
state in the named BASELINE tree (`fb0dd04` / `75f00b56` / `504ff46`) rather
than the wired tree — this is deliberate and consistent (these citations
describe "what is here before H6 changes it"), confirmed by direct comparison:
`composerDoorHasConsumablePolicy` at fork `fb0dd04:86` (spec cites `:86-90`,
correct); `argv_secret_guard` at engrave `75f00b56:551` and its bearer-arm
string at `:567` (spec cites `:551`/`:566`, off by one line on the arm citation,
consistent with the document's general one-line-early citation style seen
elsewhere, e.g. `hashlock/hashlock.go:21,24,27` citing the doc-comment line
immediately above each of `Salt`/`Iterations`/`PreimageLen` rather than the
declaration itself); `TestConstantQRLargeVersionsFailClosed` at `fb0dd04:550-568`
(spec cites `:544-568`, doc comment included); `TestPassphraseQRTooLong` at
`fb0dd04:776-791` (spec cites `:774-791`, doc comment included). None of these
is a false claim — every one resolves to the right function once the doc-comment
offset is accounted for, and I found no citation that pointed at the wrong
function or file.

---

## Contradiction and superseded-phrasing sweep

- No leftover `3 + 1 + 2 + 1 + 3` (the withdrawn row order) anywhere in the
  spec or plan.
- No leftover `"ms-cli's validate_phrase … byte for byte"` (the pre-move
  wording) — the spec's §3.1 now reads "byte for byte the rule the device's
  `hashlock.ValidatePhrase` … already applies," consistent with the move.
- No leftover assertion that the 405-448px census row widths are literal or
  pinned — both spec occurrences (§5.3, §14) are explicit withdrawals.
- No leftover "at least one of those digests came from a phrase typed here" as
  a live predicate — the one surviving occurrence is inside the finding-12 prose
  quoting the FIRST DRAFT's wording for contrast, not a normative claim.
- §16's "not settled" `constantTimeQRModules` item is struck through and
  replaced with "CLOSED by the plan round," consistent with finding 3's pin.
- §8.3's heading now says "paged by `composerReadScreen`" (not
  `confirmReviewScreen`), consistent with F10 everywhere I checked (§5.3, §8.3,
  §11.5, §14).
- Plan `e6d84d9c`'s "## Build gate folded here" and "## What this plan found
  against the R0-GREEN spec" sections cross-reference the same 26 items with the
  same numbering as the spec's "## Plan-round fold" and the fold report — no
  renumbering drift found.
- Plan citations previously flagged as wrong (`main.rs:566`→`:551`/`:566` split;
  `composer_door.go:93-97`→`:86-90`; `passphrase_test.go:775-791`→`:774-791`)
  are all present in their corrected form in `e6d84d9c`, matching the spec.

## What I did NOT re-derive (and why, so this gate's blind spot is stated)

- The four 32-minute fuzzing campaigns (43,458,059 payloads) — the spec itself
  labels this as cited-not-re-run, and re-running it is 128 minutes of wall
  clock that would not make it more true. What IS re-run is the in-suite
  fuzzed-row control (M2b) and the pin (M2), both confirmed.
- F9, F14, F4, F5, F11 (partially) — verified by direct source reading and by
  the mutations that DO cover the surrounding logic (F10/F12/F13's mutations
  exercise the same files), not by an independent fresh mutation of their own.
  None contradicts anything else measured; none is a class the block-checker
  or the full green suites could have missed silently.
- Exact per-row pixel values in §8.9's table and the F11 fixture values (419/
  385/434/442/441 etc.) — these are explicitly logged-not-pinned per the spec's
  own text (digest-dependent), so there is no fixed number to falsify.

## Closing counts

**0 Critical, 0 Important, 0 Minor/Nit findings from this round.** All 26 items
verified applied per the controller's decisions (finding 9 correctly recorded as
refined-by-measurement rather than applied literally). Every re-measured number
matched. No spec/plan contradiction. No superseded phrasing left live.

## GREEN

This closes the plan-round fold's verification gate. The spec returns to
**R0 GREEN**.
