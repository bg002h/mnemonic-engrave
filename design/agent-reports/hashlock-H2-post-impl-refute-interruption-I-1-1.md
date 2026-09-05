# Hashlock H2 — refute pass (skeptic 2/2) on lens `interruption` I-1

Target: `hashlock-h2` @ **17b3979**, detached worktree
`/scratch/code/shibboleth/.tmp/h2-wf-refute-interruption-I-1-1` (removed after
this report; nothing committed, nothing pushed, no sub-agents). Go 1.26.7 at
`/scratch/code/shibboleth/.toolchain/go`.

**Claim under test (I-1, Important):** the hashlock phrase screen draws no
readout at all (masked or revealed) because `hashlockPhraseFlow` cuts a lead
band that `passphraseEntryFlow` does not, driving the keyboard's readout budget
below one line, so `PassphraseKeyboard.Layout`'s clamp binary-searches away
every rune — and this is an interruption finding because §4.6 preserves the
phrase across 4 of 5 interruptions while the operator can never see what
survived.

**Verdict: CONFIRMED**, as stated (Important, not Critical).

## What I checked independently

1. **Re-derived the mechanism from source**, not from the report's prose:
   - `gui/composer_hashlock.go:167-175` — `hashlockPhraseFlow` cuts a **lead
     band** (`content.CutTop(leadSz.Y)`, line 169) *and* a counter band before
     `kbd.MaxHeight = content.Dy()` at line 175.
   - `gui/passphrase_flow.go:129,137` — `passphraseEntryFlow` cuts **only** the
     counter band before its own `kbd.MaxHeight = content.Dy()` at line 137. No
     lead band exists in this sibling flow's layout at all.
   - `gui/passphrase_keyboard.go:448-472` — `PassphraseKeyboard.Layout` clamps
     `avail := k.MaxHeight - k.size[k.page].Y - readoutGap` and, if the shown
     string measures taller than `avail`, binary-searches the number of
     leading runes to drop until it fits — a monotonic clamp with no floor
     that keeps at least one character.
   All three citations match the branch exactly; nothing is stale.

2. **Confirmed the copy departure from spec**, not just from the plan's own
   restatement of it:
   - `SPEC_hashlock_H2_device.md:190` — §4.2's lead, verbatim: *"Use a phrase
     you have never used anywhere else."*
   - `gui/composer_copy.go:367-369` — shipped
     `composerCopyHashlockPhraseLead()` returns *"This screen does that
     hashing for you. Use a phrase you have never used anywhere else."*
   - Traced the extra sentence to **`IMPLEMENTATION_PLAN_hashlock_H2_device.md:3179`**,
     fold item 11, answering a *different* finding (journey I-5, "§8i rule
     modal confusing ahead of the phrase route") — a legitimate, spec-directed
     copy change that was never cross-checked against the layout geometry
     it collided with. This is not a fabricated scenario; it is a real
     interaction between two independently-justified changes.

3. **Reproduced both pinned tests from the report's own test file**, dropped
   into my worktree unmodified, against the real 480×320 panel
   (`sh2DisplaySize = image.Pt(480, 320)`, confirmed at `gui_test.go:405`) and
   the real production styles/theme (`ctx.Styles.lead`, `.subtitle`, `.word`,
   `descriptorTheme`) — not a synthetic stand-in:

   ```
   $ go test ./gui/ -run 'TestHashlockPhraseScreenReadoutBudget|TestHashlockPhraseScreenDrawsNoReadout' -count=1 -v
   === RUN   TestHashlockPhraseScreenDrawsNoReadout
       composer_hashlock_interruption_test.go:613: PINNED DEFECT: 4 characters are in kbd.Fragment and the frame draws no '****': "qwertyuiopasdfghjklzxcvbnmABCspaceshowThisscreendoesthathashingforyou.Useaphraseyouhaveneverusedanywhereelse.4/100Hashlockphrase"
       composer_hashlock_interruption_test.go:633: PINNED DEFECT: revealed=true, the cap reads `hide`, and the frame still draws nothing: "qwertyuiopasdfghjklzxcvbnmABCspacehideThisscreendoesthathashingforyou.Useaphraseyouhaveneverusedanywhereelse.4/100Hashlockphrase"
   --- PASS: TestHashlockPhraseScreenDrawsNoReadout (0.01s)
   === RUN   TestHashlockPhraseScreenReadoutBudget
       composer_hashlock_interruption_test.go:660: panel (480,320), content 268 px; lead band 44, counter band 23, grid (340,182), one readout line 19
       composer_hashlock_interruption_test.go:662: hashlockPhraseFlow readout budget = 11 px; passphraseEntryFlow = 55 px
       composer_hashlock_interruption_test.go:672: PINNED DEFECT: 11 px of readout budget against a 19 px line
   --- PASS: TestHashlockPhraseScreenReadoutBudget (0.00s)
   PASS
   ok  	seedhammer.com/gui	0.017s
   ```

   Byte-for-byte identical to the numbers and frames quoted in
   `hashlock-H2-post-impl-lens-interruption.md`. Both tests drive the *actual*
   `hashlockPhraseFlow`/`composerHashEdit` harness (`runComposerHashEdit`,
   `hashlockEnterPhraseRow`, `typeOnPassphraseKeyboard`) — not a mock of the
   layout — and the budget test recomputes the arithmetic from the same
   constants (`leadingSize`, `CutBottom(8)`, `readoutGap = 8`) the production
   function uses, at the real panel size.

4. **Ran the control** the report cites as proof the keyboard widget itself is
   not at fault:

   ```
   $ go test ./gui/ -run 'TestPassphraseMaskReveal' -count=1 -v
   --- PASS: TestPassphraseMaskReveal (0.00s)
   ```

   `passphrase_keyboard_test.go:134-142` asserts `****` renders for the same
   widget with no `MaxHeight` clamp applied — confirming the defect is the
   caller's band budget, not the widget's masking/reveal logic.

5. **Checked for a pre-existing false-PASS** on this exact claim (which would
   move it toward Critical under this cycle's severity rubric): `grep -n
   "reveal\|mask\|\*\*\*\*\|readout" gui/composer_hashlock_test.go` returns
   nothing. No shipped test claimed the hashlock phrase screen's readout or
   reveal worked before this lens's own new tests — so there is no
   "false-PASS test on a normative guarantee" here, only an untested gap.

6. **Checked the interruption/preservation claims against §4.6** verbatim
   (`SPEC_hashlock_H2_device.md:289-303`): Back from the confirm modal, a
   declined method modal, the method pick, and during derivation all preserve
   the phrase (`composerHashEdit` returns `false` **only** for Back at the
   phrase screen itself) — matching "four of the five interruptions… preserve"
   exactly. Run's screensaver as the fifth is outside this file but is not
   contested here.

7. **Checked the Important-not-Critical call** against this cycle's severity
   rubric (Critical = digest divergence path / lost operator work / hash
   assigned before HOLD / false-PASS on a normative guarantee / false record
   claim). None apply: the derivation algorithm itself is unaffected (same
   bytes still produce the same digest as the host); §4.5's confirm-modal body
   (`SPEC_hashlock_H2_device.md:257-259`, "Before you fund this wallet, run ms
   hashlock…") is drawn before HOLD assigns the hash and is unconditional per
   spec; and the `n/100` counter (verified live in the frame dumps above:
   `"4/100"`) still reports true length. So the wrong outcome is exactly what
   the report says — a mistyped phrase can reach a held digest — but a
   funds-safety catch exists before spending, matching "Important" rather than
   "Critical."

## Conclusion

Every citation resolves, the mechanism reproduces byte-for-byte on the real
harness at the real panel size, the root cause (an added lead sentence,
itself a legitimate fold answering a different finding) is traced to its exact
source line, the control test proves the widget itself is sound, and the
severity classification survives this cycle's own rubric. I could not find any
line of attack that weakens the claim.

**Verdict: CONFIRMED.**
