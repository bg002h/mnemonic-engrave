# R0 round 1 — IMPLEMENTATION_PLAN_multisig_build_repair.md

Reviewer: independent verification pass (sonnet), 2026-08-13. Plan reviewed at
`design/IMPLEMENTATION_PLAN_multisig_build_repair.md` (current HEAD, commit
`d671d01`); fold isolated via `git diff dcc322f..d671d01`; round 0's report at
`design/agent-reports/multisig-build-repair-plan-R0-round0.md`; source at
`/scratch/code/shibboleth/seedhammer` @ `a10d007`. `./scripts/plan-cite-gate.sh`
re-run this session: **every citation resolves**, including the two new ones
the fold added (`mk/mk.go:5` for the pin-seam note). This is a verification
pass, not a fresh design review — round-0's settled facts (version pins, the
CSPRNG `chunk_set_id`, `address_test.go`'s unattributed fixtures) were not
re-derived.

## Verdict

**NOT GREEN — 0 Critical, 2 Important, 2 Minor.** C1, I1, M1–M4 are cleanly
folded with no new defect. I3 is only half-folded (the S5-side test landed;
the S4-side companion test round 0 asked for did not). I2's fold dropped one
of its three prescribed sentences — the one assigning ownership of the fork's
own mk wire-format re-pin — and M5's fold then papered over that gap by
attributing the re-pin to a stage (S0) that does not deliver it, leaving S5
test 6 dependent on unowned work. Both Importants are narrow, one-paragraph
fixes; neither reopens an operator ruling or touches the GREEN spec.

---

## The implementer-judgement table — the headline question

| # | location (stage / section) | the decision left open | verdict |
| --- | --- | --- | --- |
| 1 | §1a / S0 oracle table, mk1 relation (a) | Exact CLI success criterion for "the primary `mk decode`/`mk inspect` accepts the chunks" (exit code vs. specific stderr/stdout shape). | DETAIL — walk-script plumbing; doesn't change what's engraved. |
| 2 | S0 deliverable 3 | Cite the existing `address_test.go` fixtures' provenance, or replace them with BIP-382 vectors — either is left open. | DETAIL — the new `TestBip382WshMultiAddressesMatchPublishedVectors` (S0) independently proves address correctness either way; this choice is documentation-only. |
| 3 | S0, "the walk script resolves the primary toolchain by version… prints the resolved oracle versions" | Print-only visibility vs. a hard refusal when the resolved (non-vendored) primary is a stale version. | DETAIL/settled, not open — "prints… so a stale oracle is visible **rather than silent**" is repeated twice (§1a, S0) and matches round 0's own I2 fix text verbatim (which asked for visibility, not a version-mismatch refusal). Only vendored testdata gets a hard refusal + dedicated test; that split is deliberate, not a hole. |
| 4 | S2 test 3, `TestBuildRefusesForeignOriginCardBeforeS5` | The test's own **name** says "Refuses"; its **body** says "must be refused **or warned**." Which one? | RULE IT — Minor. Inherits the GREEN spec's own Minor finding M-E ("funds-correct either way, restore fail-closed"), so it doesn't gate — but the plan should still make the name and the body agree before an implementer picks one arbitrarily. |
| 5 | S4 gate, "emulator walk of… one loud failure" | Which of S4's 4 failing-row tests gets the visual walk. | DETAIL — the tests themselves are exhaustive; the walk is a smoke check. |
| 6 | S4 test 5, `TestGateAcceptsSameSeedAtDistinctOrigins` | Exact notice text shown to the operator. | DETAIL/Minor — informational, doesn't change what's engraved. |
| 7 | S5 Implementation item 7, `TestGateStillFiresAfterOriginsDiverge` | No fixture, no PROCEED/FAIL split, no mutation-check — unlike every other S4/S5 gate test, which all name a concrete fixture. Round 0's I3 fix asked for a *second*, S4-side test with this specificity; it is entirely absent. | **RULE IT — Important.** See finding R1-I1 below. |
| 8 | S5 test 6 note, "only sound once **S0's re-pin** includes V19" | Which stage re-pins the fork's own mk wire-format decoder (0.2-era → 0.4.x/V19). S0's three stated deliverables don't include it. | **RULE IT — Important.** See finding R1-I2 below. |
| 9 | S6 gate, "All three restore correctly at an external coordinator" | Doesn't explicitly enumerate item 3's fold-added ms1 readback (restore master B's mnemonic from its plate) as a pass/fail condition of the gate sentence itself. | RULE IT — Minor. One clause: "…and master B's mnemonic restores from its ms1 plate." |
| 10 | S3, "Delete or correct the four `TYPED-ONLY` comments" | Delete vs. correct, per site. | DETAIL — either satisfies "a future reader greps `TYPED-ONLY` and finds nothing misleading." |

**Summary:** mostly a well-specified plan — 6 of 10 rows are DETAIL or settled-not-open. Two rows are genuine specification holes at Important severity, both concentrated in the fold's newest material (S0/S5); two more are Minor and already bounded by the GREEN spec's own risk analysis.

---

## Did the fold work? (round-0 findings)

| finding | fixed? | new defect? |
| --- | --- | --- |
| **C1** — no gate saw a wrong-master ms1 | **YES.** S5 gate rewritten: every ms1 compared byte-for-byte against `ms encode --hex <that master's entropy>`. Test 5 rewritten to decode each ms1 and compare entropy to its claimed master, disjunction removed, mutation named (captured-variable bug on the engrave loop). | No. |
| **I1** — mk1 has no implementable comparison plane | **YES.** New §1a table rules: (a) current primary `mk decode`/`mk inspect` accepts the chunks, AND (b) `canonical_payload_bytes` equality; states explicitly the `chunk_set_id` exclusion is a ruled format property, not a convenience. `--chunk-set-id` flag idea correctly filed, not built. | No. |
| **I2** — oracle never named/pinned | **PARTIAL.** Items 1–2 of round 0's 3-sentence fix landed (oracle versions pinned + printed into every gate record; full input tuple recorded) — expanded into a whole new S0 stage with its own tests and gate. Item 3 — "the vendored-vector re-pin… is filed as a follow-up **owned by the stage that finds drift, or by S6 if none does**" — is **absent**; `grep -n "re-pin"` on the current plan returns only the two lines inside S5 test 6's note. | **Yes — see R1-I2.** The dropped ownership sentence is what leaves S5 test 6 depending on unowned work. |
| **I3** — gate never re-proven after S5 rewires the origins it derives against | **PARTIAL.** S5 gained item 7 (`TestGateStillFiresAfterOriginsDiverge`) — a title + one paragraph of rationale, no fixture, no PROCEED/FAIL split, no mutation mandate. The S4-side companion test round 0 named explicitly (`TestGateDerivesAtDeclaredOriginNotFlowOrigin`, mutation-checked) and the "S4's gate tests are synthetic until S5 by construction" sentence are both **absent** — no hunk touches S4's test list at all. | Not a new defect per se, but the fix is materially thinner than what was asked — see R1-I1. |
| **M1** — S2 "accepts byte for byte" wording | **YES.** Reworded to "production, not acceptance": the primary must *build* an equal md1, not merely decode one. | No. |
| **M2** — S5 test 5 disjunction can't be mutation-checked | **YES** (subsumed into C1's fix — the engrave-both arm is picked, disjunction removed). | No. |
| **M3** — S6 never reads back an ms1 | **YES.** One clause added to S6 item 3: restore master B's mnemonic from its ms1 plate, same flash cycle. | No — see R1-M2 for a related but separate gap (gate sentence doesn't cite it). |
| **M4** — S4-before-S5 conditional not yet settled in plan text | **YES.** Replaced with "RULED by the operator 2026-08-13 ('Agreed. Safety first.')," matching the operator's actual words. | No. |
| **M5** — depth-0 mk1 sits on the pin seam | **YES, a note was added** ("Note the pin seam: … only sound once S0's re-pin includes V19"). | **Yes — see R1-I2.** The added note references a deliverable ("S0's re-pin") that S0's own Deliverables list does not contain, and that I2's dropped ownership sentence would otherwise have assigned elsewhere. |

---

## New findings

### R1-I1 (Important) — S5's gate-reproof test is underspecified relative to what I3 asked for, and I3's S4-side companion test is missing entirely

**Where.** S5 §3 "Implementation," item 7 (`TestGateStillFiresAfterOriginsDiverge`); compare S4's 7 tests, every one of which names a concrete fixture (several explicitly mutation-checked).

**Failure scenario.** Round 0's I3 fix asked for two edits: (1) an S4-side test — a `both` slot whose card declares a non-shared origin, key genuinely derived there, must PROCEED; the same fixture with the key derived at the shared origin instead must FAIL, naming the slot; mutation-checked like S4's other rows; (2) an S5-side regression test re-running the gate through the real post-rewire flow. The fold delivered only (2), and as a single paragraph with no fixture, no PROCEED/FAIL split, and no mutation mandate. An implementer can satisfy "the gate still fires" with `assemble(divergentOriginInput); assertNoError()` — a smoke test that never checks *which* origin the gate derived against, which is exactly the binding M-B exists to protect and exactly what I3's failure scenario described. S4's own implementation is told to reuse `findUserSlot`'s derive-and-compare (`gui/multisig_match.go:34`) — read this session: it derives at each key's own `k.OriginPath`, so it is origin-correct by construction — but the *gate-specific* wrapper built on top of it is new code, and nothing stops it from hardcoding `multisigSharedOrigin()` instead of reading the per-slot origin, since during S4 itself (S2's interim refusal still active) the two values are indistinguishable. That bug would pass every S4 test (fixtures are shared-origin by construction) and, on the current wording, might pass S5 test 7 too.

**Fix.** Add the S4-side test with the explicit PROCEED/FAIL fixture pair I3 specified, mutation-checked (mutate which origin the gate derives at). Give test 7 the same specificity as S4's rows: name the fixture (a `both` slot whose card declares `m/48'/0'/1'/2'`, genuinely derived there vs. genuinely derived at the shared origin instead), state PROCEED/FAIL, and mark it mutation-checked.

### R1-I2 (Important) — S5 test 6 depends on a fork wire-format re-pin that no stage in the plan owns

**Where.** S5 §3, test 6 (`TestDepthZeroCosignerCardIsNamedRefusal`), fold-added note; cross-reference to S0's "Deliverables" (1–3) and §1a's Oracle-1 paragraph.

**Failure scenario.** Round 0's I2 fix had three parts; the third assigned ownership of "the vendored-vector re-pin (0.36→current, mk 0.2→0.4 including V19)… as a follow-up owned by the stage that finds drift, or by S6 if none does." That sentence did not survive the fold — it appears nowhere in the current plan. S5 test 6's fold-added note then says the test's premise (the fork can decode a depth-0 mk1 far enough to see `Path == "m"` and trip `errMultisigEmptyDivergent`) "is only sound once **S0's re-pin** includes V19." But S0's three deliverables (pinned-oracle harness, published-BIP vectors, `address_test.go` provenance) are all about the *external comparison harness* — none re-pins the fork's own decoder from its "mk-codec 0.2" wire pin (`mk/mk.go:5`) to 0.4.x. No stage in the plan lists that re-pin as a deliverable. If it is never built, an implementer reaching S5 test 6 finds the fork cannot parse a depth-0 (V19) card at all — decode fails at an earlier, unnamed point, and the flow shows whatever generic error the current decoder already produces on an unrecognized chunk shape, not the named `errMultisigEmptyDivergent` refusal spec M-1 requires. That silently degrades M-1's "named screen, not a fall-through 'Couldn't assemble'" guarantee back to exactly the fall-through the spec rejected — and S5's gate ("Trace B completes… by test") would have no way to notice, because the test that was supposed to prove it can't be written.

**Fix.** One sentence, restoring what the fold dropped: name which stage owns the fork's mk 0.2→0.4/V19 re-pin — either add it explicitly to S0's Deliverables (if that's the intent) or assign it elsewhere per round 0's original "the stage that finds drift, or S6" language — before S5 test 6 is written against it. Then fix "S0's re-pin" to name whichever stage actually owns it.

### R1-M1 (Minor, recorded) — S2 test 3's name and body disagree on refuse-vs-warn

**Where.** S2 test 3, `TestBuildRefusesForeignOriginCardBeforeS5`. The name says "Refuses"; the body says "must be refused **or warned**." This is inherited from the GREEN spec's own Minor finding M-E ("Fix (optional)… refuse or warn"), which the plan is free to leave as a disjunction — the spec's own analysis is that either arm is fund-safe (addresses derive from keys either way, restore fail-closed). Recorded only: pick one arm so the test's name matches its body.

### R1-M2 (Minor, recorded) — S6's gate sentence doesn't cite the ms1 readback it just added

**Where.** S6 "Gate": "All three restore correctly at an external coordinator." Item 3's fold-added ms1 readback (restore master B's mnemonic from its plate, same flash cycle) is an instruction inside item 3's prose but isn't named as one of the gate's own pass/fail conditions. One clause fixes it: "…and master B's mnemonic restores from its ms1 plate."

---

## Disposition

Not 0C/0I. Both Importants are narrow: R1-I1 needs one S4 test added and one S5 test tightened to match the plan's own house style (concrete fixture, PROCEED/FAIL, mutation); R1-I2 needs one ownership sentence restored and one cross-reference corrected. Neither touches the GREEN spec, reopens an operator ruling, or requires re-deriving C1/I1/M1–M4, which are cleanly closed. Re-review after this fold should scope to: the new S4 test + S5 test 7's rewrite (R1-I1), and the re-pin ownership sentence + S5 test 6's note (R1-I2) — nothing else needs re-touching.
