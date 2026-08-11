# Overdue follow-up burndown verification — F-77, F-80, F-84, F-87, F-89, F-93

**Date:** 2026-08-10. **Scope:** independent re-verification of six FOLLOWUPS.md
items whose owning phase has shipped, against code — not against register text
or any prior report's conclusions. Checked read-only against
`/scratch/code/shibboleth/seedhammer-b2b` @ `75233b8` (branch `b2b`, working
tree clean at start and end). Every mutation described below was applied with
Python via a heredoc, verified with `git diff --stat` before and after, and
restored from a pre-mutation copy before moving to the next item — the tree is
clean now (`git status --porcelain` empty). All `go test` invocations ran via
`/nix/var/nix/profiles/default/bin/nix develop /scratch/code/shibboleth/seedhammer --command go test ...`
from inside `seedhammer-b2b`; PASS/FAIL output below is pasted, not
paraphrased. Whole-package `go test ./gui/... ./seal/...` was green
(`ok seedhammer.com/gui 41.092s`, `ok seedhammer.com/seal 18.406s`) after every
restore.

A prior same-day report,
`design/agent-reports/2026-08-10-b2b-followup-reconciliation.md`, already
covers F-87/F-89/F-93 (plus F-96/F-105) and reached the same verdicts on
F-87/F-89/F-93 reported below. It was used as a pointer to relevant files, not
as a source of truth — every claim below was re-derived from the code and
re-run, and F-89/F-93 additionally got fresh mutations that report did not
apply (it explicitly declined to re-mutate F-93's commit-time-verified rows;
this pass did, and both still kill).

---

## F-77 — the encrypted section's md1/mk1 cards have no grouping

```
F-77  DONE
  what the entry asked for:   extend pass 3's §6.3 card grouping over the encrypted section's ClassMDMK subset, reusing groupCards/cardKey, not re-derived in gui.
  what is in the tree:        seal/label_encrypted.go:28 func labelEncryptedCards(out []AdmittedRecord) — filters to ClassMDMK, calls the SAME groupRecords/labelCards as the public path; wired at seal/record.go:266-268 (`if section == SectionEncrypted { labelEncryptedCards(out) }`) inside AdmitSection, which is called for real at seal/unlock_key.go:102 (`AdmitSection(recs, SectionEncrypted)`) — the production Unlock path, not a test-only entry point.
  behavioural proof:          Deleted the `labelEncryptedCards(out)` call (replaced with a no-op comment). Reran seal's three F-77 tests:
                                 TestEncryptedSectionCardsAreLabelled          -> FAIL (record 1-5 report HRP '\x00', plate 0/0, card 0/0)
                                 TestEncryptedMultisigCardsAreDistinguishable  -> FAIL ("vector F carries 0 mk1 records, want 6")
                                 TestUnreadableEncryptedCardDoesNotReject      -> PASS (unaffected, as expected -- it asserts non-rejection, which the mutation doesn't touch)
                               Restored; `git diff --stat seal/record.go` empty; all three PASS again.
  residue:                    none against the entry's text.
```

**Proposed closure line:**
`**CLOSED 2026-08-10 — labelEncryptedCards at seal/label_encrypted.go:28, wired into AdmitSection at seal/record.go:266-268 and reached from production via seal/unlock_key.go:102; killed by deleting the wiring, which fails TestEncryptedSectionCardsAreLabelled and TestEncryptedMultisigCardsAreDistinguishable.**`

---

## F-80 — residue from the B1 whole-diff review (the three "owning phase: B2" bullets)

Per the entry's own 2026-08-08 amendment, of the seven bullets only three carry
an explicit `owning phase: B2` tag, and the operator split them: the
"already cut" marks and the Back-is-Lock affordance go to B2a-ii; the
`layoutMainPager` pixel pin does **not** (deferred to F-78's font/rasterising
work). Verified all three.

```
F-80a (layoutMainPager pin)  NOT DONE (correctly — still ownerless/deferred to F-78)
  what the entry asked for:   pagerDots must measure the drawn screen, not call layoutMainPager directly with a test-supplied constant.
  what is in the tree:        gui/unlock_program_test.go:157-167 -- pagerDots(t, ctx, lastNav program) still calls `layoutMainPager(&ctx.B, &descriptorTheme, backupWallet, lastNav)` directly, with `lastNav` a caller-supplied constant (bip85Derive / unlockPayload), exactly the "measures the function, not the screen" shape the entry describes.
  behavioural proof:          not applicable -- confirmed by reading, matches the entry's original description verbatim. No mutation needed since nothing changed.
  residue:                    entirely open, as expected. This is correctly NOT B2a-ii's to close (needs a rasterising/pixel check, F-78's territory).

F-80b (Back-is-Lock affordance)  DONE
  what the entry asked for:   Back on the screen that exits a decrypted session must not read as assets.IconBack ("step back"); needs a distinct lock/exit affordance.
  what is in the tree:        gui/unlock_platelist.go:172-179 -- the plate-list nav uses `Icon: assets.IconDiscard` instead of assets.IconBack, with a comment naming F-80's B2 item explicitly and explaining why IconDiscard (not a new lock glyph) was chosen: it already carries "discard this session" elsewhere in the codebase.
  behavioural proof:          gui/unlock_plates_test.go:312 TestPlateListBackIconIsDiscardNotBack pixel-compares the rendered Back slot against reference renders of both IconDiscard and IconBack (uiContains can't see icons, so this is a real behavioural check). Ran PASS. Mutated the icon back to assets.IconBack at unlock_platelist.go:179 -> TestPlateListBackIconIsDiscardNotBack FAILs. Restored; diff empty; PASS again.
  residue:                    none.

F-80c ("already cut" marks)  DONE
  what the entry asked for:   §10.2.2's "records already cut this session are marked" -- the plate list must show a mark after a completed engrave, and the plate list is B1/B2's to add.
  what is in the tree:        gui/unlock_plates.go:45-62 unlockPlateLabel appends " (cut)" (a WORD, not the F-78-poisoned "·" glyph) when `cut` is true; gui/unlock_platelist.go:112-117 sets `plates[sel].cut = true` only on a COMPLETED engrave (unlockEngraveFlow returning true), never on cancel.
  behavioural proof:          gui/unlock_plates_test.go:241 TestPlateListMarksCutAfterACompletedEngraveAndNotAfterACancelledOne (subtests cancelled/completed) -- ran PASS (9.24s, drives a real ~29,500-frame engrave to completion). Mutated unlockPlateLabel to drop the `cut` branch -> TestPlateListMarksCutAfterACompletedEngraveAndNotAfterACancelledOne/completed FAILs ("a COMPLETED engrave did not mark the plate as cut") and TestUnlockPlateLabelWrapsPlateLabel FAILs (2 cases). Restored; diff empty; both PASS again.
  residue:                    none.
```

**Proposed closure lines:**
- F-80b: `**CLOSED 2026-08-10 — assets.IconDiscard at gui/unlock_platelist.go:179; pixel-pinned by TestPlateListBackIconIsDiscardNotBack, killed by reverting to assets.IconBack.**`
- F-80c: `**CLOSED 2026-08-10 — " (cut)" mark at gui/unlock_plates.go:59, set on completion only at gui/unlock_platelist.go:116; killed by dropping the cut branch, which fails TestPlateListMarksCutAfterACompletedEngraveAndNotAfterACancelledOne/completed and TestUnlockPlateLabelWrapsPlateLabel.**`
- The `layoutMainPager` bullet stays OPEN, correctly, under F-78/its own steam — do not close it as part of this sweep.

---

## F-84 — `SeedScreen` gains `NoEdit`

```
F-84  DONE
  what the entry asked for:   SeedScreen needs a NoEdit mode (zero value = editable, so existing callers are unaffected) so a payload-sourced seed can't be "typo-fixed" into a self-consistent but wrong plate; the guard must be on the CLICK HANDLER, not just the layout, because Filter.matches gates on button identity with no bounds check.
  what is in the tree:        gui/gui.go:2341 `NoEdit bool` field; gui/gui.go:2388 `if !s.NoEdit && editBtn.Clicked(ctx) {` (handler guard); gui/gui.go:2464 `if !s.NoEdit { navBtns = append(...) }` (layout guard, icon omitted rather than dead). Production wiring: gui/unlock_session.go:291 `ss := &SeedScreen{NoEdit: true}` inside unlockEngraveMnemonic, the payload-sourced-seed flow.
  behavioural proof:          gui/unlock_session_test.go:1053 TestPayloadSeedScreenRefusesEditing drives the REAL production call site (via unlockedPayload/runSecretSession, not a direct SeedScreen construction) and checks BOTH routes: no touch target at the edit nav slot, AND Button2 does not reach word entry. Ran PASS, alongside TestSeedScreenNoEditClosesBOTHRoutes (widget-level, both NoEdit states) -- PASS. Mutated the production call site `ss := &SeedScreen{NoEdit: true}` -> `ss := &SeedScreen{}` (exactly the mutation the test's own comment names as the one thing nothing else covers) -> TestPayloadSeedScreenRefusesEditing FAILs. Restored; diff empty; PASS again.
  residue:                    none.
```

**Proposed closure line:**
`**CLOSED 2026-08-10 (already recorded as implemented, not deferred, in the entry's own header) — SeedScreen.NoEdit at gui/gui.go:2341,2388,2464, wired at gui/unlock_session.go:291; killed by reverting the production call site to &SeedScreen{}, which fails TestPayloadSeedScreenRefusesEditing.**`

---

## F-87 — nothing pins `unlockEngraveMnemonic`'s deferred wipe

```
F-87  PARTIAL
  what the entry asked for:   drive EACH of unlockEngraveMnemonic's three early returns (!ss.Confirm, masterFingerprintFor error, engraveSeed error) with a hook that fires right after `defer clear(m)` is registered, and assert m is zero after each return.
  what is in the tree:        gui/unlock_session.go:270-311. `defer clear(m)` at :279; `unlockMnemonicParsedHook` fires immediately after (:284-286, gui/unlock_mnemonic_seam.go). Two of the three early returns are exercised: :292 (!ss.Confirm -> return) and :309-area (engraveSeed err -> return). The THIRD (masterFingerprintFor err, :300-303) has no test anywhere in the tree -- confirmed by `grep -rn "Couldn't derive the fingerprint" --include="*.go" .` (only the production showError call, zero test hits) and `grep -rln "masterFingerprintFor" --include="*_test.go" .` (bip85_test.go, seedxor_polish_test.go, gui_test.go, unlock_session_test.go -- none drives THIS call site's error branch; unlock_session_test.go's only mention is a comment).
  behavioural proof:          Ran the two existing tests: TestUnlockEngraveMnemonicZeroesMOnConfirmDiscard PASS, TestUnlockEngraveMnemonicZeroesMOnEngraveSeedError PASS. Mutated `defer clear(m)` away (commented out) -> BOTH tests FAIL: ConfirmDiscard reports word N still 138 (etc.) and EngraveSeedError reports the same for words 14-23+. Restored; diff empty; both PASS again. The third leg was not mutated because there is nothing to mutate -- no test exists to kill, which is the finding itself.
  residue:                    the masterFingerprintFor-error early return (gui/unlock_session.go ~:300-303) has no test. This is not silent: gui/unlock_session.go's own comment (lines ~260-269) and the file's history both explain that adding a seam requires touching masterFingerprintFor/deriveMasterKey, which are shared funds-path code explicitly scoped OUT of this phase and re-filed as F-94 (B2c). The branch is also plausibly unreachable in practice: SeedScreen.Confirm's own validity probe (gui/gui.go:2426, `deriveMasterKey(mnemonic, &chaincfg.MainNetParams, "")`) already runs the identical deterministic derivation (same m, same network, same empty password) and must have succeeded before Confirm returns true -- so masterFingerprintFor's later, identical call cannot then fail. That argument is not itself pinned by a test; it's a code-reading argument, not a machine-checked one.
```

**No closure line proposed — F-87 does not meet the entry's own bar.** Its
text says "drive EACH of the three early returns"; two of three are driven.
This is a judgment call for the register owner: either (a) accept 2-of-3 with
the unreachability argument recorded and reword the entry to say so
explicitly (it currently doesn't), or (b) file the third leg's seam under
F-94 (B2c, which already owns `masterFingerprintFor`/`deriveMasterKey`) and
leave F-87 open until that lands. Do not mark this CLOSED as-is.

---

## F-89 — B2b's idle wipe MUST unwind the flow, not just call `p.Wipe()`

```
F-89  DONE
  what the entry asked for:   the §10.2.4 idle-wipe timer must make the flow RETURN (so deferred wipes like clear(m) actually fire), not wipe p.Secret/p.Wipe() in place while the flow stays parked mid-function.
  what is in the tree:        gui/run_flow.go:282-288 -- on the armed-wipe deadline, sets `wiping = true; ctx.Done = true; break`. ctx.Done propagates through every `for !ctx.Done { ...; ctx.Frame(...) }` loop in the flow (Context.FrameCallback at run_flow.go:143-148 stops yielding once ctx.Done is true), unwinding the whole call stack and firing every registered defer -- including unlockEngraveMnemonic's `defer clear(m)` -- before the session loop restarts with a fresh Context. `RecordsResident` (seal/session.go:20-51, renamed from SecretsResident) carries a doc comment disclaiming the wide "secrets are gone" reading; wipe_guard.go's `armed()` never touches it, and run_flow.go's own comment states the timer keys on the session bracket's lifetime, not the predicate.
  behavioural proof:          gui/wipe_inventory_audit_test.go:200 TestWipeZeroesEveryPinnedBufferAtRunLevel (both subtests: vectorA parked on SeedScreen.Confirm, vectorF parked on ms1 Cut/Skip) drives the REAL Run loop through the REAL 3:00+30s timer via synctest and asserts m/rec are zero after the wipe. Ran PASS (0.40s). Mutated the armed-wipe branch to remove `ctx.Done = true` (keeping `wiping = true; break`, simulating "wipe fires but the flow is never told to unwind") -> BOTH subtests FAIL with "the wipe never restarted the session (sessions=1)" -- the session hangs forever rather than wiping, which is worse than F-89's own described defect (a live seed with a clean-reading predicate) and confirms the unwind is load-bearing, not decorative. Restored; diff empty; PASS again.
  residue:                    none against the entry's stated constraint.
```

**Proposed closure line:**
`**CLOSED 2026-08-10 — unwind via ctx.Done at gui/run_flow.go:282-288, RecordsResident's narrowed contract at seal/session.go:20-51; killed by removing ctx.Done=true from the armed-wipe branch, which fails TestWipeZeroesEveryPinnedBufferAtRunLevel (both subtests, "the wipe never restarted the session").**`

---

## F-93 — the screensaver still PARKS a spec-legal derivation

```
F-93  DONE
  what the entry asked for:   treat an in-progress derivation as activity so the screensaver's non-breaking `continue` cannot permanently stall it -- a Run-side fix, reconciled with F-89 so it can never postpone an ARMED wipe.
  what is in the tree:        gui/unlock_kdf.go:334-335 -- unlockDerive calls `ctx.KeepAwake()` then `ctx.WakeupAt(time.Now())` before every ctx.Frame in its per-slice loop. gui/run_flow.go:251 -- `if len(evts) > 0 || (ctx.keepAwake && !armed) { a.idle.start = now }`: the `&& !armed` term is what stops KeepAwake from postponing an armed §10.2.4 wipe (F-93's own comment at unlock_kdf.go:326-333 states this reconciliation explicitly).
  behavioural proof:          Ran TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver (PASS, drives a real 100,000-iteration seal.Deriver past idleTimeout under synctest with a 1s tick floor) and TestRunKeepAwakeCannotPostponeAnArmedWipe (PASS). TWO independent mutations, both killed:
                                 (1) removed `ctx.KeepAwake()` from unlockDerive -> TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver FAILs: "Run exceeded 100000 ticks without terminating -- flow is probably parked (screensaver?)", last frame mid-derivation at 90%. Restored; diff empty; PASS again.
                                 (2) changed `(ctx.keepAwake && !armed)` to `(ctx.keepAwake)` in run_flow.go -> TestRunKeepAwakeCannotPostponeAnArmedWipe FAILs (12.39s, times out waiting for the armed session to ever restart). Restored; diff empty; PASS again.
  residue:                    none against the entry's stated constraint. (Note: this mutation was independently re-applied and re-killed by this pass, not merely re-cited from the prior same-day reconciliation report, which had relied on the fold commit's own message for this specific row.)
```

**Proposed closure line:**
`**CLOSED 2026-08-10 — ctx.KeepAwake() at gui/unlock_kdf.go:334, reconciled with F-89 via the "&& !armed" term at gui/run_flow.go:251; killed independently by (a) removing KeepAwake (TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver fails, derivation parks under the screensaver) and (b) removing "&& !armed" (TestRunKeepAwakeCannotPostponeAnArmedWipe fails, armed wipe never fires).**`

---

## Summary table

| item | verdict |
| --- | --- |
| F-77 | DONE |
| F-80 (layoutMainPager pin) | NOT DONE — correctly still open/ownerless, deferred to F-78 |
| F-80 (Back-is-Lock) | DONE |
| F-80 ("already cut" marks) | DONE |
| F-84 | DONE |
| F-87 | PARTIAL — 2 of 3 early returns pinned; masterFingerprintFor-error leg has no test and no mutation could be run against it |
| F-89 | DONE |
| F-93 | DONE |

Tree state at end: `/scratch/code/shibboleth/seedhammer-b2b` clean, `git status
--porcelain` empty, `go test ./gui/... ./seal/...` green. No file was left
modified. No commit was made in the firmware worktree or in this repo as part
of this verification (per instruction: report only, no fixes, no commits).
