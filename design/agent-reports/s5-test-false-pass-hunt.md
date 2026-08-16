# S5 test false-PASS hunt — mechanical verification

Date: 2026-08-15
Scope: `/scratch/code/shibboleth/wt-s5`, `gui/multisig_build_s5_test.go` and
`gui/multisig_build_s5_flow_test.go`. One question only: do the seven named S5
tests assert what their names claim, or can any pass on broken code? No design
opinions, no review of any other test.

Method: static read of each test plus its assertions, then live mutation
testing on the three tests the task flagged as highest-risk (never having gone
red) — `TestLegDerivedAtHeldSlotOrigin`, `TestOneMk1PerHeldSlot`, and, since the
task's item 5 names "test 3, 4 and 7" using the PLAN's own numbering (matched
against the source doc-comments: "plan test 3" / "plan test 4" /
"TestReRunMintsByteIdenticalPlates is plan test 7's ASSERTION half"), the
byte-identical re-run assertion in `TestReRunMintsByteIdenticalPlates` — plus,
for full coverage of the primed table, one live mutation for each of the
remaining four named tests. Every mutation: edited source, ran the target test
with `-v` redirected to a file (never through a pipe), grepped the file for
`--- FAIL` and a printed marker proving the mutated line executed, then
reverted with Edit and re-ran the full green baseline.

**Scope note on item 5.** The task's "Where" section names seven functions,
which does not include `TestReRunMintsByteIdenticalPlates` (it substitutes
`TestGateStillFiresAfterOriginsDiverge`, plan test 8, as the seventh). Item 5's
parenthetical ("tests 3, 4 and 7 ... TestLegDerivedAtHeldSlotOrigin,
TestOneMk1PerHeldSlot, and the byte-identical re-run assertion") uses plan
numbering from the source comments and unambiguously means
`TestReRunMintsByteIdenticalPlates`, which sits outside the named seven. Given
the direct conflict with "Out of scope: do not review any test outside the
seven," I resolved it by keeping the seven as the primary table (below) and
adding `TestReRunMintsByteIdenticalPlates` as a supplementary mutation-proof
item, since it was explicitly named high-risk and proving it is cheap. I did
not otherwise review it (no static per-assertion audit, no table row).

## Primary table — the seven named tests

| Test | Asserts specifically? | Can it fail? | Evidence |
|---|---|---|---|
| `TestMultiSlotSelfAssembles` | Yes — checks per-slot `OriginPath` against exact wanted strings for all 4 slots, asserts `@0 != @1` directly (not just "divergent somewhere"), and separately checks each held key's xpub against the slot it landed on | PROVEN RED | Mutation M1: forced `buildSelfKeys`'s per-slot origin to always use account 0 → `slots @0 and @1 hold the same key` (`gui/multisig_build.go:472`, marker printed 3x) |
| `TestCosignerCardOriginIsHonoured` | Yes — checks the unit claim (`cosignerFromCard`'s parsed `Origin` components match the card's declared path, component-by-component) and the integration claim (`@1`'s decoded origin == `card.Path`, `@0`'s stays at the shared origin) | PROVEN RED | Mutation M2: forced `cosignerFromCard` to always parse the shared origin instead of `card.Path` → `origin component 2 = {...Value:0}, want {...Value:1}` (`gui/multisig_build.go:990`, marker printed) |
| `TestLegDerivedAtHeldSlotOrigin` | Yes — checks the leg's decoded `mk.Card.Path` equals the exact wanted origin string, checks the leg's key bytes equal `keys[1].Xpub` (the descriptor's own bytes, not a value computed by the same function), AND has an explicit non-vacuity check that the leg's key is NOT `keys[0].Xpub` | PROVEN RED | Mutation M3: forced `buildEngraveTail`'s `slotFromSeed` origin to always use account 0 → all three assertions fired: origin mismatch, key does not match `@1`, AND "the @1 leg carries @0's key" (`gui/multisig_build_tail.go:61`, marker printed 3x) |
| `TestOneMk1PerHeldSlot` | Yes — checks cardinality both directions (`len(legs) == held`, `mk1s == held`) computed independently from `sources`, not from the tail's own output, AND a distinctness check that no two legs are byte-identical | PROVEN RED (both directions) | Same M3 mutation → "legs 0 and 1 are byte-identical mk1s". Separate mutation M3b (skip slot 1 entirely) → "2 leg(s) for 3 held slot(s)" and the mk1-count line, proving the undercount direction independently |
| `TestFullModeEngravesMs1ForEveryMaster` | Yes — checks card-kind ordering strictly (`wantKinds` slice compared index-by-index), AND decodes each ms1 with `codex32.DecodeMS1` and compares entropy hex against each master's own `bip39`-derived entropy (not a value produced by the code under test), with an explicit "engraved twice" check | PROVEN RED (two independent mutations) | M4 (drop the per-master `engraved` gate) → wrong count, "engraved 3 ms1 plate(s), want 2". M4b (key the same gate on `Account` instead of `SeedID`, count stays 2) → the entropy-comparison-specific line: "master A's seed was engraved TWICE and the other master not at all" — proves the entropy check, not just the count check, is load-bearing |
| `TestDepthZeroCosignerCardIsNamedRefusal` | Yes — first checks the md-layer premise directly (own subtest, `md.EncodeMultisig` on an empty-origin cosigner, text match on `"non-empty Origin"`), then checks `errors.As` into the specific `errBuildEmptyOrigin` type (not just "any error"), checks `.Slot == 1`, checks `out == nil`, then checks the operator-facing message for the specific slot marker, the "Nothing was engraved" phrase, the escape-hatch command, absence of "scan", and absence of em/en-dash | PROVEN RED | Mutation M5: disabled the empty-Origin check at `md/encode_multisig.go:104-106` → both the premise subtest ("md.EncodeMultisig accepted...") and the outer test ("assembled or failed generically") went red, exactly as the test's own doc comment predicts |
| `TestGateStillFiresAfterOriginsDiverge` | Yes — a PROCEED/FAIL pair on the same fixture differing only in the xpub (asserted explicitly at the end: same Path/Fingerprint, different Xpub, path has the right prefix); PROCEED checks the review screen is reached; FAIL checks the refusal names `@0` AND quotes the card's own origin, explicitly asserts the shared origin is NOT quoted, and checks a raster-ink floor so the screen isn't blank | PROVEN RED (both arms, from one mutation) | Mutation M6: forced `bothSlotKey` to parse the shared origin instead of `card.Path` → PROCEED arm failed ("the gate did not PROCEED..."), FAIL arm failed too ("was ACCEPTED" — the liar card, which carries the shared-origin key, now matches when compared against the shared origin). Exactly the mutation the test's own doc comment names |

## Supplementary — flagged in item 5, outside the named seven

| Test | Can it fail? | Evidence |
|---|---|---|
| `TestReRunMintsByteIdenticalPlates` | PROVEN RED | Mutation M7: added a call-counter to `deriveMultisigLeg` (`gui/multisig_derive.go`) that injects an extra stub on exactly the 2nd invocation across the whole test, simulating hidden per-call state rather than pure randomness → "card 3 plate 0 differs between two runs of the SAME inputs", with two different mk1 strings printed. Marker confirmed 6 calls total (3 per run), and the diverging call landed inside the run as designed |

## False-PASS shapes found

None. Every one of the seven named tests, plus the supplementary byte-identical
test, was proven capable of failing on realistic, targeted breakage of the
exact mechanism its name and doc comment claim to cover — including the
non-vacuity checks (`TestLegDerivedAtHeldSlotOrigin`'s "not @0's key" clause,
`TestOneMk1PerHeldSlot`'s distinctness clause) and the harder-to-fake shapes
(`TestFullModeEngravesMs1ForEveryMaster`'s entropy comparison caught a
same-count-but-wrong-master bug that the count check alone would have missed;
`TestGateStillFiresAfterOriginsDiverge`'s dual-arm structure flipped both ways
from a single mutation, exactly as its own doc comment predicts). No
assertion-free paths, no loops over empty slices, no tautological
self-comparisons, and no skipped/unreachable subtests were found in any of the
eight tests examined.

## Worktree hygiene

- Baseline before any edit: `git status --porcelain` = 3 untracked
  (`gui/multisig_build_s5_flow_test.go`, `gui/multisig_build_s5_test.go`,
  `gui/multisig_build_tail.go`) + 17 modified = 20 entries.
- All 9 mutations (M1, M2, M3, M3b, M4, M4b, M5, M6, M7) were applied one at a
  time via Edit, verified RED with a printed execution marker, then reverted
  via Edit before the next mutation began. `md/encode_multisig.go` (touched
  only by M5) was not in the original modified set and is confirmed
  `git diff`-clean afterward.
- Final `git status --porcelain` (sorted) is byte-identical to the baseline:
  same 3 untracked + 17 modified, same 20 lines.
- `grep -rn "MUTATION-MARKER" gui/ md/` after the last revert: no hits.
- Final full-target re-run (`go test ./gui/ -run '<all 8 names>' -count=1 -v`,
  unpiped): 8/8 `--- PASS`, `ok seedhammer.com/gui 0.269s`, `EXIT=0`.

## Files touched during mutation testing (all reverted)

- `/scratch/code/shibboleth/wt-s5/gui/multisig_build.go` (M1 line ~472, M2 line
  ~990)
- `/scratch/code/shibboleth/wt-s5/gui/multisig_build_tail.go` (M3/M3b line ~61,
  M4/M4b line ~82/87)
- `/scratch/code/shibboleth/wt-s5/gui/multisig_build_slots.go` (M6 line ~306)
- `/scratch/code/shibboleth/wt-s5/md/encode_multisig.go` (M5 line ~104)
- `/scratch/code/shibboleth/wt-s5/gui/multisig_derive.go` (M7, package-level
  counter + call site ~line 48)
