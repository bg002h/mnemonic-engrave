# R0 review, round 3 (scoped fold re-check): IMPLEMENTATION_PLAN_fork_refugium_F5_F7.md

- **Reviewer:** independent adversarial architect (opus), R0 gate, round 3
- **Date:** 2026-10-05
- **Plan revision:** fc73529 (draft 3)
- **Earlier rounds:** `refugium-F5-F7-plan-R0.md` (0C/8I/10M/3N), `refugium-F5-F7-plan-R0-round2.md` (0C/3I/8M/3N)
- **Code baseline:** bg002h/seedhammer main be00ef8 (read-only; `git status` clean before and after)
- **Counts:** 0 Critical / 0 Important / 7 Minor / 3 Nits
- **Verdict:** GREEN (0C/0I). The gate closes. The Minors below are corrections the implementer should take with the plan. None of them produces a false GREEN: each one either turns a test RED that the implementer then has to resolve, or strengthens a gate that already holds without it.

Scope, following the proportional re-review rule: (1) whether draft 3 fixed each round-2 finding; (2) whether the fold introduced new defects. Facts settled in rounds 1 and 2 were not re-derived.

## What I executed

- **`default.elf` (be00ef8, TinyGo 0.41.1):**
  - `nm`: 11,620 symbols.
  - `grep -a -o -F` for each §4.5 marker.
  - A Python section map that locates each data-literal hit in its ELF section, with surrounding bytes.
- **`go list`:**
  - `go list -e -tags tinygo,rp,refugium -f '{{.GoFiles}}' ./cmd/controller` returns `[engraver.go main.go platform_sh2.go]`, rc 0, `.Error` nil. It runs on host Go 1.25.10, which matches go.mod's `go 1.25.10`.
  - I also diffed the gui file set host-tagged (`-tags refugium`) against device-tagged (`-tags tinygo,rp,refugium`).
- **Emulator, headless:**
  - Built `cmd/emu` to a scratch dir with `GOOS=js GOARCH=wasm go build` (10 s). The repo was not touched.
  - Copied `index.html`, the walk/shots `.js` files and `wasm_exec.js` next to the build, and served the directory with `python3 -m http.server`.
  - Drove it with Playwright 1.56.1 (`/opt/node-tools/node_modules/playwright`) on `/opt/pw-browsers` Chromium, headless.
  - It boots. `window.shTap/shPress/shRelease/shSysw/shPace/shNFC/shScreen/shScreenSeq/shTargets/shWaitFor/shStep` are all installed. `shScreen()` returned the boot prompt (`"A systemwide payload is present. Load it? LOAD SKIP"`). `shNFC.present("hello-not-a-format")` moved `presented()` from 0 to 1.

## Fold table check (round 2, finding by finding)

| R2 | Folded? | Note |
|---|---|---|
| I-1 emulator walk | **Yes** | `deliveredCount` in `Read`, an emulator `refugium` twin, a named end screen, a headless Playwright run with the default walk first, and the CI vet line are all specified. I confirmed the route runs headless from a static server. The walk still needs a new script: no single-sig walk exists today (`grep -il single walk_*.js` finds only comments). See M-6 for three walk-design hazards I hit by trying it. Its instrument (`deliveredCount`) is new code, so "default walk first, a failing control stops the phase" is the right way to execute it. |
| I-2 ELF gate never run | **Yes** | The default ELF is measured. Every marker the plan claims present is present, and `FOREVERLAURA` occurs 0 times as claimed. Two of the marker attributions are wrong (M-1), and the "no symbol of its own" claim is false (M-2). Neither is a false GREEN. |
| I-3 NFC-off has no executed test | **Yes** | Two layers. On the gui side: the `startScanner` gate, `nfcAvailable()`, and a counting-reader test over the seven sites with an untagged positive control. On the device side: `Poller.Read` in the ELF absence set (present in the default ELF at `0x1014042d`). |
| M-1 comment hits; `""` trap | Yes | The plan uses an AST `BasicLit` scan and `proofTriggersEnabled`. The literal list is short by three triggers; see M-3. |
| M-2 guard and `go list` form | Yes | The command runs as written. The gui file set is the host one; see M-4. |
| M-3 unkeyed scan offers | Yes | |
| M-4 Engrave Text; `validateMdmkStrings` kind | Yes | The predicate wording is self-contradictory; see M-5. |
| M-5 ms-codec source | Yes | git rev `e5dff6f`. |
| M-6 fingerprint wipes | Yes | |
| M-7 `Close()` and fault screen | **Mostly** | `Close()` writes `regOpCtrl=0` (`driver/st25r3916/st25r3916.go:297-301`), as stated. The fault mechanism is not named and has no test; see M-7. |
| M-8 tagged emulator CI | Yes | |
| Nits (untyped nil, emulator twin, whole-payload `pass:`) | Yes | The `pass:` refusal is now stated. Placement and secrecy detail are in N-1. |

## Minor

### M-1. §4.5: two default-ELF "positive control" markers are not the things they are said to control.

Section map of the hits (all in `.text`, the merged rodata):

- **`lock-boot`** (one hit, `0x192910`): `"lock-boot: %vunknown debug command: %q"`. This is the `log.Printf("lock-boot: %v", err)` format string at `gui/gui.go:2271`, not the `case "lock-boot":` comparison at `:2268`. The comparison compiled to integer compares, like `FOREVERLAURA!`. The marker still works for what §4.1 does, because the whole arm moves to `debugcmd_default.go`. But it proves the *log line* is gone, not the trigger.
- **`PASSPROOF!`** (two hits): both are inside the help text `ppPassProofKeepPassphrase` and `ppPassProofKeepFingerprint` (`gui/passphrase_passproof.go:127,130`): `"Back = no: continue with PASSPROOF! exactly as typed"` and `"keep PASSPROOF! in this field"`. The trigger `ppPassProofTrigger = "PASSPROOF!"` (`:62`) has **no** standalone occurrence in the ELF. So the ELF "positive control" for the passphrase trigger is the help text, and the trigger's absence rests on the source check alone, exactly as `FOREVERLAURA!` does.
- `TEXTPROOF!` and `CONSTPROOF!` are standalone entries (`…shTEXTPROOF!SH 3.0mm 44x26…`), from the `Trigger:` fields of the proof table (`freetext_proof.go:536,544`). Those attributions are correct.

**Fix.** In §4.5, state what each marker is:

- `lock-boot` is the log format string of the debug arm.
- `PASSPROOF` is the passproof help text.

Say that the `lock-boot`/`PASSPROOF!` *comparisons* have no binary form and are covered at source level only. Add to §4.1 that the two `ppPassProofKeep*` help strings (and any other literal containing a forbidden token) move to `!refugium` files with the trigger. The AST check will otherwise turn RED on them; that is correct behaviour, but the plan should not leave it to be discovered.

### M-2. §4.5: "The OTP writer's absence rests on the redirect literal … since it has no symbol of its own" is false.

The default ELF has `seedhammer.com/driver/otp.writeECC` (`0x100507f5`) and `seedhammer.com/driver/otp.writeOrRow` (`0x10050885`). In `driver/otp/otp.go`, these are reached only from writers:

- `EnableSecureBoot` (`:94`)
- `WriteBootKey` (`:166,169`)
- `WriteWhiteLabelString` (`:273,276`)
- `writeECCRow` (`:332`)

The readers kept for `isSecureBootEnabled` (`readOrRow`, `readECC`, `ReadBootKey`, `ReadWhiteLabelString`) do not call them. They are therefore the binary-level marker for "no OTP writer is linked". They are also stronger than the redirect literal, which is tied only to `writeOTPValues`. A Refugium `LockBoot` that called `otp.EnableSecureBoot()` directly would leave the literal absent and still link `writeOrRow`.

**Fix.** Add `otp.writeECC` and `otp.writeOrRow` to the absence set, with the default ELF as the positive control (measured present). Keep the literal and the source call check.

### M-3. §4.5's forbidden-literal list omits three QA triggers that §4.1 says must be absent.

§4.1 says "the other `ftProofTrigger*` triggers are absent". §4.5's list has only `PASSPROOF!`, `TEXTPROOF!` and `CONSTPROOF!`. Missing:

- `BOTHPROOF!` (`freetext_proof.go:69`)
- `SIZEPROOF!FRONT` and `SIZEPROOF!BACK` (`:86-87`)

All three are present as data in the default ELF: `BOTHPROOF` once, `SIZEPROOF` twice. So they are measurable markers on both layers.

Also, `gui/preview.go:100-107` (`//go:build !tinygo`, so it is in the host `-tags refugium` build) references all five `ftProofTrigger*` constants. Moving the constants to `!refugium` files breaks the tagged host build unless those `previewBuilders` entries move as well.

**Fix.** Add the three literals to both lists, and name `preview.go`'s proof entries in §4.1.

### M-4. §4.5: the gui source-check file set is the host set, not the device set.

`go list -tags refugium ./gui` and `go list -e -tags tinygo,rp,refugium ./gui` differ:

- The host set has `composer_state_hook.go`, `engraved_hook.go`, `frame_hook.go`, `plate_hook.go` and `preview.go`.
- The device set has the four `*_tinygo.go` hook twins instead, and no `preview.go`.

So the device-only gui files are never scanned. They are small hooks today, so the risk is low. But the check claims to cover "the Refugium file set", and the controller line already uses the device tags.

**Fix.** Scan the union of `-tags refugium` and `-tags tinygo,rp,refugium` for gui. Make the positive control the union of the matching default sets.

### M-5. §4.4: the predicate is named two incompatible ways.

The plan says the gate "tests each string's HRP (`ms`, case-insensitive, the same predicate as the free-text warning)". The free-text warning's predicate is `hashlock.IsMS1Shaped` (`hashlock/hashlock.go:190-215`, via `sysw_session.go:309`). That predicate:

- trims the string;
- strips whitespace, `-` and `,`;
- requires at least **48** characters, an `ms1` prefix and bech32 characters only.

An HRP test has neither the length floor nor the separator stripping.

Neither choice is unsafe for real cards: a 16-byte ms1 is 50 characters, and cards carry verbatim codec strings. But the implementer has to choose, and the tagged test has to match the choice.

**Fix.** Pick one. Recommended:

- In `validateMdmkStrings`, use the plain case-insensitive `ms1` prefix on each card string. Note that the QR is built only on the `len(strs) == 1` branch (`gui.go:2675-2682`), so that branch is the only one to gate.
- Keep `IsMS1Shaped` for free text, where separators matter.
- Alternatively, gate in `bundleEngrave`, which already holds `cardMS1` (`bundle_flow.go:618`), and keep the HRP check only for non-bundle callers.

### M-6. §4.2's walk: three hazards found by running the route.

The route itself works; see "What I executed".

- **(a) The emulator's default payload holds a `pass:` record.** It boots with the `records` systemwide payload and prompts LOAD/SKIP. `sysw_test_payload.bin` contains `pass:636f…` (`cmd/emu/sysw_test_payload.go:28`). Under §4.3 the Refugium build refuses that payload whole, so a walk that takes the default LOAD meets the refusal screen, not the start screen. The walk must tap SKIP, or call `shSysw("cards")`/`shSysw("none")` first. The same applies to any gui test that loads the records payload under the tag; add those to §4.5's exclusion list.
- **(b) The presented record's content is unspecified.** In the default build the start-screen scanner (`gui.go:2236`) dispatches a delivered record: a valid md1/mk1 record would navigate away and derail the walk from the single-sig flow. Records presented while no scanner runs only queue (`cmd/emu/nfc.go:91-102`). Specify a record that parses to nothing actionable, such as an unrecognised plain string, which yields a status and no navigation. Specify also where `delivered() > 0` is expected to happen: at the start screen, before the flow is entered.
- **(c) In the Refugium build the walk is vacuous about gui.** The emulator twin returns a nil reader, so `delivered() == 0` holds by the twin's construction and never exercises the new gui `startScanner` gate in a real flow.

**Fix.** Address (a) and (b) in §4.2. For (c), add a second Refugium walk: same build, with the twin's reader left attached (for example, a walk-only `shNFC.attach()` that the twin honours, still with `FeatureNFC` unset). There, `presented() > 0 && delivered() == 0` is evidence that gui's gate holds across a real flow, not just in the unit test.

Name the new walk script file. No single-sig walk exists; `walk_trace_b.js` exports `keyPoint` for typing.

### M-7. §4.2: the fault-screen mechanism is unnamed and untested.

`gui.Platform` (`gui/gui.go:3608-3638`) has no fault accessor. Its three implementers are:

- `testPlatform` (`gui_test.go:464`)
- `cmd/emu` `platform` (`platform.go:350`)
- `cmd/controller` `Platform` (`platform_sh2.go:568`)

`run_flow.go:88` `runWithFlow` runs a session loop that re-enters `uiFlow` after every wipe. `uiFlow` (`gui.go:2084`) starts with the systemwide `syswLoadFlow` prompt. So "before anything else" means the top of `uiFlow`, ahead of `syswLoadFlow`, re-shown on every session re-entry.

**Fix.** Name the mechanism. Options:

- an optional interface `interface{ BootFault() error }` asserted in `uiFlow`, which needs no change to the other two platforms;
- a `Features` bit.

Add a test: a `testPlatform` reporting the fault draws the screen, a tap or button press does not advance, and no `syswLoadFlow` or start screen is drawn.

## Nits

- **N-1 (§4.3).** Place the `pass:` refusal **after** `sysw.Open` (`sysw_load.go:131`), because a sealed payload's records are only visible once it is opened. Place it **before** `ctx.sysw.load` (`:173`), so no session holding the secret is ever created. Drop `p` on refusal.
  - "A message naming the record" must name it by position and class, never by body. `ClassPassphrase.IsSecret()` is true (`sysw/record.go:84-86`).
  - Note that this departs from the load flow's stated rule "None of them refuses anything (§13)" (`sysw_load.go:186-187`). The better justification is "a secret the build cannot use is never held in RAM", not the digest argument; an inert loaded record would not change what the digest covers.
- **N-2 (§4.5).** Two fixes to the device-side markers:
  - `st25r3916` `Detect` and `reset` have **no** symbols even in the default ELF (inlined), so "record any that disappears" records nothing. Measured symbols that should vanish with the reader: `(*st25r3916.Device).configureProtocol`, `.enable`, `.Read`, `.commandAndWait`, `.waitForInterrupt`. `(*Device).Close` and `.writeReg` must stay.
  - Scan data literals within allocated sections only (`.text`/`.data`). The ELF carries DWARF (`with debug_info`), and a whole-file `grep -a` could in future match a debug string.
- **N-3 (§4.1 vs §4.5).** §4.1 says whether `qaEngraveFlow` still links "is measured … and reported". §4.5 puts `gui.qaEngraveFlow` in the marker set that must be **absent**. The `case qaProgram: qaEngraveFlow(ctx)` arm at `gui.go:2148` remains reachable as far as the compiler can tell, so it will link unless the arm itself is profile-gated. Make §4.1 say the arm is gated (`if !refugiumProfile`), consistent with §4.5's absence requirement.

## New-defect check on the fold

Apart from the attribution errors (M-1, M-2) and the walk details (M-6), the fold introduced no new control-flow defect:

- The two-layer NFC design is sound.
- `startScanner` with a nil reader returns a channel that never delivers (`nfc_scan.go:56-59`).
- `FeatureNFC` is read at exactly the three sites the plan lists (`bundle_flow.go:195`, `transaction.go:316`, `derive_xpub.go:287`).

The source-parse test would be harder to evade if it asserted that the *identifier* `FeatureNFC` appears only in `nfcAvailable` and its declaration (`gui.go:3656`), rather than matching the call spelling (nit-level, folded into M-4's spirit).

## Process

- The device ELF gate has run on the default build. I re-measured it here, and the markers agree with the plan except for the attributions in M-1 and M-2.
- The emulator walk's route has run headless, as far as boot, the systemwide prompt and tag presentation. Its full form needs the new `deliveredCount` and a new walk script, and the plan orders the default-build run first as a stop condition.
- 0C/0I: the R0 loop for this plan closes. The Minors are wording and coverage corrections that do not re-trigger a gate under the proportional rule.
