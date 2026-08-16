# S6a R0 — EXECUTABILITY + TEST-FALSIFIABILITY REVIEW

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Code under change:** `/scratch/code/shibboleth/seedhammer` @ `main` = `b8a23bf`
**Lens:** Q1 — can a cold implementer execute §4 literally and land compiling, passing code?
Q2 — can each test in §5 actually fail?
**Out of scope by brief:** line-number citations (already gated 49/49), the fork's
green baseline, the Rust-primary check, cycle scope, prose/markdown.

---

## VERDICT: RED — 0 Critical, 1 Important

---

## I-1 — §4.6's census screen breaks three existing end-to-end walks; the plan lists none of them

**Where:** plan §4.6 (and §5's "Existing tests that must be updated"), against
`gui/singlesig_flow_test.go:51`, `gui/singlesig_flow_test.go:91`,
`gui/template_engrave_test.go:79`.

**The defect.** §4.6 inserts a `confirmReviewScreen(ctx, th, "Plates To Cut", …)`
between the wallet-policy choice and `bundleEngrave`. Three shipped tests drive
`engraveSingleSigFlow` straight across that seam and press nothing there:

| test | file:line | the step that breaks |
| --- | --- | --- |
| `TestEngraveSingleSigFlowFull` | `gui/singlesig_flow_test.go:51` | `:82` `click(Button3)` (wallet policy) → `:83` `pumpUntil(frame, "Card 1 of 3", 64)` |
| `TestEngraveSingleSigFlowWatchOnly` | `gui/singlesig_flow_test.go:91` | `:121` `click(Button3)` → `:122` `pumpUntil(frame, "Card 1 of 2", 64)` |
| `TestEngraveSingleSigFlowTemplate` | `gui/template_engrave_test.go:79` | `:128` `click(Button3)` ("I understand") → `:129` `pumpUntil(frame, "Card 1 of 3", 64)` |

`pumpUntil` (`gui/slip39_polish_test.go:353`) **only pumps frames — it never
presses a button**:

```go
func pumpUntil(frame func() (string, bool), want string, maxFrames int) (string, bool) {
	for i := 0; i < maxFrames; i++ {
		c, ok := frame()
		...
	}
	return content, false
}
```

`confirmReviewScreen` (`gui/multisig_build.go:1720`) loops `for !ctx.Done` until
Button1 (back), Button3/Center (continue) or Button2 (page). So after §4.6 lands,
each of those three walks sits on the census for its whole 64-frame budget,
`"Card 1 of N"` is never drawn (it comes only from `bundleEngrave`'s `ChoiceScreen`,
`gui/bundle_flow.go:404-408`), and each hits its `t.Fatalf`.

The positive control is in-tree and unambiguous: **every existing walk that crosses
a plate census has to press through it.** `TestSupplyAbortIsTheLastScreenOfTheProgram`
(`gui/multisig_verify_report_test.go:1009-1016`) does exactly
`pumpUntil("Plates To Cut")` → `click(Button3)` → `pumpUntil("Choose engraving")`;
so do `gui/multisig_supply_passphrase_test.go:211-215`,
`gui/multisig_supply_multislot_test.go:155`, and
`gui/multisig_engrave_tail_walk_test.go:164`.

**Consequence for the implementer.** Three red tests that the plan's §6 gate
attributes to "this cycle's" edits with no guidance on what the right fix is. §5's
only "must be updated, not weakened" list is the six `buildPlateInventoryLines`
call sites; `gui/template_engrave_test.go` is not mentioned **anywhere** in the
plan. The wrong-but-plausible repairs are live: raise the `pumpUntil` budget
(never converges), move the census after `bundleEngrave` (defeats F-202 entirely
while still satisfying §5's T6 as worded — "reaches `Plates To Cut` before the
engrave picker" is checkable, but nothing in the plan pins the census *before* the
last free moment), or drop §4.6.

**Suggested remedy (UNVERIFIED):** name the three walks in §5 alongside the six
call sites, and state the inserted step as `pumpUntil(frame, "Plates To Cut", N)`
→ `click(&ctx.Router, Button3)` → existing `pumpUntil("Card 1 of N")`, mirroring
`gui/multisig_verify_report_test.go:1009-1013`. I resolved the break and the
prior-art shape against the call graph; I did **not** execute the repaired walks.

**Not a finding, stated because the brief asked:** `gui/singlesig_program_test.go`
is **not** affected. Both of its tests (`:12`, `:79`) walk the start-screen
carousel only and never enter `engraveSingleSigFlow` — `grep -rn
"engraveSingleSigFlow"` returns no hit in that file.

---

## PER-TEST TABLE

| test | can it fail? | vacuity risk | notes |
| --- | --- | --- | --- |
| **T1** label says `NOT passphrase` | **yes** | low | `"NOT passphrase"` is emitted only by `buildFullModeLabel(true)` (`gui/multisig_build_census.go:248-253`); reverting `:80` to the literal removes it from every frame. Route is available: `singlesig.go:71` calls `syswPassphraseFlow`, and `syswPassphraseFlowTitled` (`gui/sysw_source.go:96-110`) takes the payload arm, so `ctx.sysw = sessionHolding(s5PassphraseRecord)` reaches it exactly as `gui/multisig_supply_passphrase_test.go:162,194-200` does. Note `uiContains` (`gui/gui_test.go:527`) strips spaces from the **needle** only. |
| **T2** doc has `BIP-39 passphrase WAS used` + `This backup is` | **yes** | **medium — fails loudly, does not false-pass** | Two unstated costs. (a) `restoreDocScreen` (`gui/singlesig_restore.go:137`) is a **pager**, and §4.2 appends `extra` *after* the descriptor chunks and both addresses, so the inventory lands on the last page(s): a single-frame assertion misses it. Prior art exists and the plan does not name it — `s5PageForNeedle`, `gui/multisig_build_s5_flow_test.go:119`. (b) `restoreDocFlow` sits past `bundleEngrave`, so the walk must cut **every** plate; that needs `p.engraver = newEngraver()` + `p.display = sh2DisplaySize` and a per-plate driver (`s5EngraveOnePlate`, `gui/multisig_supply_passphrase_test.go:110`), none of which the current single-sig walks set up. Both omissions produce a red test, not a green one. |
| **T3** bare run: no `NOT passphrase`, doc says `No BIP-39 passphrase was used` | **yes** | low | The stated mutation (always return the passphrase arm) exercises only the label half; the document half is separable and the prior art (`gui/multisig_supply_passphrase_test.go:305-323`) asserts it on `buildPassphraseInventoryLines` directly rather than through a walk. Both are legitimate — worth stating which, so the mutation proof is unambiguous. |
| **T4** watch-only ⇒ absence line, full ⇒ presence line | **yes** | medium if written at unit level | Fixtures are trivial: `singleSigEngraveCards(b, false)` (`gui/singlesig_engrave.go:20-44`) emits `[mk1, md1]` only, so `bundleSetCarriesASecret` (`gui/bundle_flow.go:482`) is false — the arms really do diverge. But a unit test on `buildSeedInventoryLines` proves nothing about the single-sig *document*; that seam is carried by T2 alone. Fine as designed, worth saying out loud. |
| **T5** abort ends the program | **yes** | low, **given the plan's own §8.3 guard** | Route verified reachable: `bundleEngrave` → per-plate `ChoiceScreen` ("Choose engraving") → Button1 → `bundleAbortWarning` → `showError("Bundle Incomplete")`. `validateMdmk` succeeds on the plain test platform (the shipped walks already reach `"Card 1 of 3"` with `newPlatform()`), so T5 needs **no** engraver. Two notes: the route now has to press through §4.6's census, which T5's row does not mention; and of the three banned strings, `"This backup is"` is absent from the single-sig document until §4.2 lands, so the load-bearing pair against the stated mutation is `"Verify the engraved plates?"` (`singlesig.go:130`) and `"Descriptor:"` (`gui/singlesig_restore.go:108`). |
| **T6** run reaches `Plates To Cut` | **yes** | low | The literal reaches the screen from no other single-sig screen (`grep -rn "Plates To Cut"` → one production site today, `gui/multisig.go:279`). Adding a second production site is safe: `"Plates To Cut"` is **not** in `buildFlowNeedles` or `decoyNeedles` (`cmd/emu/needle_test.go:43-160`), so neither needle gate moves. |
| **T7** capacity arms + ASCII | **yes** | low | The *presence* arm is already double-covered — `gui/multisig_build_prose_test.go:394`, `gui/multisig_build_perseed_passphrase_test.go:273` and `:320` all run `ContainsAny(doc, "—–·''""…")` over joined `buildPlateInventoryLines` output on ms1-bearing cards. The **absence** arm is reached by none of them and is T7's alone; the plan's "every new operator string" wording covers it, but the implementer must build a seedless `cards` fixture on purpose. |
| **T8** `bundle_flow.go` no longer claims `both engraving callers` | **yes** on the stated mutation | **medium** | Pure negative source assertion. Deleting the paragraph outright, or replacing it with a *differently* false claim, satisfies it — while §4.5 states a positive requirement (name all three tail-carrying callers, say why `gui/bundle_flow.go:39` needs none). The pattern it cites, `gui/multisig_build_prose_test.go:402-411`, deliberately pairs its `!Contains` with a positive `Contains("re-decision filed to S5 is now made")` for exactly this reason; T8 has no positive half. `readGuiFile` (`gui/bundle_abort_prose_test.go:275`) does `t.Fatalf` on a bad path, so the read-the-wrong-file vacuity is already closed. **Minor**, recorded below. |

---

## WHAT I CHECKED AND FOUND SOUND

**The 8 call sites — verified independently, the plan's count is CORRECT.**
`grep -rn "buildPlateInventoryLines" --include="*.go" .` → 11 hits: 1 definition
(`gui/multisig_build_census.go:75`), 2 comment mentions
(`gui/multisig_build_census.go:48`, `gui/multisig_restore.go:97`), and exactly
**8 call sites** — production `gui/multisig.go:362`, `gui/multisig_build.go:479`;
test `gui/multisig_build_prose_test.go:369,424,425` and
`gui/multisig_build_perseed_passphrase_test.go:134,246,304`. Every one is listed
in §4.3.

**Every §4 snippet fits the surrounding code.**
- §4.1 — `passphrase` is bound at `singlesig.go:64-74`, ahead of the picker at
  `:77`; `buildFullModeLabel` is package-local. Bare runs are **byte-identical**
  (`buildFullModeLabel(false)` returns `"Full (seed + keys)"`), so no walk that
  selects row 0 by index changes behaviour.
- §4.2 — `restoreDocFlow`'s new `extra []string` is safe: my grep confirms
  **one** production call site (`gui/singlesig.go:136`) and **zero** test call
  sites. `md` and `bip32` are already imported in `gui/singlesig_restore.go:10,16`.
  `restoreDocScreen(ctx, th, append(lines, extra...))` type-checks against
  `restoreDocScreen(ctx, th, lines []string)` at `:137`.
- §4.3 — `type seedCapacity int` + iota consts compile as written; the 3-arg
  `buildPlateInventoryLines` matches §4.2's call.
- §4.4 — `bundleSetCarriesASecret(cards []bundleCard) bool` exists at
  `gui/bundle_flow.go:482` with that exact signature.
- §4.5 — `bundleEngraveResult` / `bundleEngraveDone` are exported-in-package
  (`gui/bundle_flow.go:442-451`).
- §4.6 — `confirmReviewScreen(ctx, th, title string, lines []string) bool` matches;
  `cards` exists from `singlesig.go:126`, so the insertion point (between `:126`
  and `:127`) is forced and unambiguous.
- Ordering: no §4 step depends on a later one.

**§1.9's absence claim re-run and confirmed.** `grep -rn '"Full (seed + keys)"'`
→ 13 hits, all in multisig census/label code, comments, or multisig tests, plus
`gui/singlesig.go:80` itself. The only equality assertion is
`gui/multisig_build_prose_test.go:323` on `buildFullModeLabel(false)`, which §4.1
does not touch. **Nothing pins the single-sig label — §4.1 breaks no test, which
is precisely why T1 is required.**

**§1.5's four `bundleEngrave` production call sites confirmed.**
`gui/bundle_flow.go:39`, `gui/multisig.go:291`, `gui/multisig_build.go:402`,
`gui/singlesig.go:127`. Two gate today; `gui/bundle_flow.go:39` `return`s on the
next line. The comment at `gui/bundle_flow.go:535` is pinned by no test
(`grep -rn "both engraving callers"` → one production hit, zero test hits), so
correcting it breaks nothing and T8 is net-new coverage.

**The F-197 edit does not break the call-site table test.**
`TestMs1ReminderIsTitledForTheProgramThatShowedIt`
(`gui/bundle_abort_prose_test.go:251-268`) does
`strings.Index(src, "bundleEngrave(ctx, th, ")` then reads the next 120 bytes for
`"Engrave Single-Sig"`. Wrapping the call in `if … != bundleEngraveDone {` leaves
the title inside that window.

**The needle gates are unaffected.** Adding `"Plates To Cut"` to `singlesig.go`
makes it two-site, but it is pinned nowhere:
`cmd/emu/needle_test.go`'s `buildFlowNeedles` pins `"Plate Count"` →
`gui/multisig_build.go`, and the decoys are `"Which md1?"` (2, unchanged),
`"First card from where?"`, `"Engrave Bundle"`. Calling `buildPlateCensusLines`
from a second flow does not move `"Plate Count"`'s owner either — under
`cmd/emu/needle_flow_test.go`'s call-graph analysis a literal passed *into* a
shared helper is attributed to the caller.

**`TestEngraveSingleSigFlowTypedOnly_Structural` survives.** It strips `//`
comments from `singlesig.go` and bans `assembleScan`, `act.scan`, `.Scan(`,
`new(scanner)`; none of §4.1/§4.5/§4.6's added text contains any of them.

**The `seedCapacityOne` re-key of the supply path is measured, not assumed.**
`supplyMultisigPolicyFlow` has exactly one `seedEntryFlow` seam
(`gui/multisig.go`, step (3)) and `supplyEngraveTail` emits one ms1 however many
slots match — corroborated by `gui/multisig_supply_multislot_test.go:206`, whose
two-slot expectation is `mk1 key 1 of 2`, `mk1 key 2 of 2`, **`ms1 secret share`**
(unnumbered), `md1 descriptor`. So §3.1.1 holds, and §3.1.7's prefix argument is
exact: `numberedLabel` returns the bare base when `n <= 1`.

**No downstream artifact is coupled to the ruling text.** `"Every seed"` occurs in
exactly two places repo-wide (`gui/multisig_build_census.go:86` and the assertion
at `gui/multisig_build_prose_test.go:382` that §4.3 already schedules for update).
`"Seed handling"` occurs once. No `cmd/emu/walk_*.js` and no
`oracle/gaterecords/*.json` anchors on restore-document prose.

**Watch-only really does take the absence arm.**
`singleSigEngraveCards(b, false)` appends no `cardMS1`, so
`bundleShowMs1Reminder` is true and `bundleSetCarriesASecret` is false —
§4.4's two arms are genuinely distinguishable on the single-sig path.

---

## MINOR / NIT (recorded, does not gate)

- **M-1 — T8 has no positive half.** Add a `Contains` assertion for whatever the
  corrected §4.5 comment must say, so a deletion cannot satisfy it. Mirrors
  `gui/multisig_build_prose_test.go:402-411`. (UNVERIFIED remedy.)
- **M-2 — T2/T3/T4 need machinery the plan does not name.** `s5PageForNeedle`
  (`gui/multisig_build_s5_flow_test.go:119`) for the paged restore doc, and
  `newEngraver()` + `sh2DisplaySize` + `s5EngraveOnePlate`
  (`gui/multisig_supply_passphrase_test.go:110`) to reach it at all. §1.7 points
  at the walk that does both, so this costs a discovery round, not a false green.
- **N-1 — §4.3's header says "all 8" and then enumerates 9.** The 8 is the
  *existing* count (correct); `gui/singlesig.go:136` is the ninth, created by this
  cycle. The enumeration is complete; only the label is off.
- **N-2 — T6's wording does not pin the census *before* the last free moment.**
  "reaches `Plates To Cut` before the engrave picker" is satisfied by a census
  placed anywhere upstream, including before the wallet-policy form. Harmless on
  its own; it becomes load-bearing under I-1, because relocating the census is one
  of the plausible wrong repairs for the three broken walks.

---

## HOW TO READ THIS RESULT

One Important, and it is a **plan omission rather than a design error**: §4.6 is
the right change in the right place, and the three walks it invalidates simply are
not listed. Everything §4 asserts about the code's *shape* — signatures, scope,
imports, call-site counts, the needle gates, the one-seed premises — resolved true
against the tree. The tests in §5 can all fail; the two soft spots are T8's missing
positive half and T2's unstated pager/engraver machinery, neither of which can
produce a green over a broken implementation.
