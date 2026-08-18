# S6b P7 implementation report — GATE 4 modal-fit sweep, the REST of the firmware (F-192)

Worktree: `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`, on top of P6
(`87e5a5e`, "S6b P6 (4): the modal-fit sweep, GATE 4 -- every long modal this
cycle touched"). P7 exists because P6's own scope was narrower than spec §4:
P6 swept "every modal S6b added or changed" (`IMPLEMENTATION_PLAN_s6b.md` §1);
F-192's filed text and spec §4 both say "every long modal in the firmware" /
"**Every** long modal body is gated". P6 noticed the gap and documented it
rather than picking a side. The operator ruled: sweep the rest. This phase is
that sweep.

Commit: `12a428d` "S6b P7 (4): the modal-fit sweep, the REST of the firmware --
F-192 closes" — two files: `gui/multisig_build_slots.go` (the one copy fold,
see the finding below) and `gui/s6b_p7_modal_fit_sweep_test.go` (new, 452
lines).

## How this was enumerated

A `go/ast` scan (written for this phase, not committed — a throwaway `go run`
over `gui/`, per the brief) walked every `gui/*.go` **non-test** file for
`ast.CallExpr` nodes whose `Fun` is the identifier `showError` or
`showNotice` — the only two functions that reach `showModal` → `ErrorScreen`,
the `Warning.Layout` body F-185's check targets. `ConfirmWarningScreen` is a
different call shape (a struct literal, not one of these two calls); the scan
did not find any *additional* composed `ConfirmWarningScreen` body outside the
one F-185/P6 already gates (`multisigBuildExperimentalWarningBody`, asserted
by `gui/multisig_build_prose_test.go:98`).

For each call's 4th argument, the AST node was classified: `*ast.BasicLit` is
a **literal**; `*ast.CallExpr`, `*ast.BinaryExpr` (string concatenation), and
`*ast.Ident` are **composed**.

**Verified counts** (machine-checked, not hand-counted):

- **131 total call sites** across 73 non-test `.go` files in `gui/`.
  Cross-checked: `grep -c 'showError(\|showNotice('` over the same files =
  **133** — the `+2` is exactly the two function *definition* lines
  (`func showError(...)`, `func showNotice(...)`, `gui/slip39_polish.go:36,44`),
  which also match that substring. 131 = 133 − 2, confirming the AST walk
  found every call and nothing extra.
- **87 `*ast.BasicLit` literals.**
- **44 composed** by AST shape. Of these, **5 are `*ast.BinaryExpr` string
  concatenations of two *adjacent string literals* with no interpolation** —
  compile-time constants (`multisig_build.go:57`, `multisig_build.go:317`,
  `singlesig_verify.go:195`, `unlock_flow.go:153`, `unlock_kdf.go:93`). These
  are one fixed value each, identical in kind to a `BasicLit` for this check's
  purposes (no worst-case search applies), but the AST shape still marks them
  composed. Net: **87 + 5 = 92 literal-valued bodies, 39 genuinely
  runtime-composed** — which reconciles with the plan's own "92 literal / ~36
  composed" bound (`design/SPEC_s6b_pre_flash_cycle.md` §4,
  `IMPLEMENTATION_PLAN_s6b.md` §1): 44 composed − 8 excluded as trivially
  short (see below) = **36**, matching the plan's own number once the
  trivial-exclusion bucket is subtracted the same way.

### Where this sweep differs from the dispatch brief's list

The brief said "re-derive this list yourself... report any producer this list
missed." Three mismatches found:

1. **`gui/address_polish.go:60` is `err.Error()`, not `Describe(...)`.** The
   brief's "Describe(...) in address_polish.go" does not match this file's
   content — no `Describe` call exists there.
2. **`gui/slip39_polish.go` has THREE `slip39words.Describe(err)` call sites**
   (lines 244, 249, 303), a whole file the brief's "Describe(...) in
   address_polish.go, seedxor_polish.go, gui.go:2674" line did not name.
3. **`gui/freetext_flow.go` has FOUR composed sites, not "(x3)".** Lines 949,
   955 and 1175 are `fmt.Sprintf`; line 1171 is a string concatenation
   (`"The " + strings.ToLower(what) + " is a single line."`) the brief's
   Sprintf-only wording missed.

None of these change what got gated — the AST walk already covered all of
them — but they are reported as instructed.

## Coverage table

Every one of the 44 composed call sites, by producer. Call sites of the same
producer/identifier are grouped into one row.

### GATED IN THIS PHASE (P7) — 25 producers, 27 call sites, `gui/s6b_p7_modal_fit_sweep_test.go`

| producer | site(s) | worst case chosen | headroom (margin 80) |
| --- | --- | --- | --- |
| `buildSupplyRefusal` | `multisig_build.go:133,158` | all 4 text shapes; N≤5 so digits are single-digit and negligible; `incomplete=true` is the longest default-branch sentence | 418 / 455 / 378 / 436 |
| `buildSeedKeyMismatchMessage` | `multisig_build_slots.go` via `multisig_build.go:246` | `who` non-empty (longer branch); `Declared` = `multisigSharedOrigin().String()`, the real path this firmware issues | 107 |
| `buildFingerprintContradictsMessage` | `multisig_build.go:252` | `who` non-empty; Declared/Derived are fixed 8-hex (no length variance) | **42 → FAIL, then 128 after shortening (see Finding below)** |
| `buildDuplicateKeyMessage` | `multisig_build.go:287` | both slots carrying provenance (`SelfFromCard`, both in `origins`) — the longest `buildSlotProvenance` combination reachable | 302 |
| `buildEmptyOriginMessage` | `multisig_build.go:295` | `who` non-empty, `Declared` non-empty (empty falls back to shorter `"m"`) | 223 |
| `multisigVerifyNoExpectationBody` (const) | `multisig_verify.go:745,902` | fixed value, one string, two sites | 513 |
| `multisigVerifyNoPolicyBody` (const) | `multisig_verify.go:755` | fixed value | 476 |
| `multisigVerifyForeignPolicyBody` (const) | `multisig_verify.go:811` | fixed value | 436 |
| readback-count Sprintf | `multisig_verify.go:832` | both counts at N≤5 plural form | 476 |
| `multisigVerifyOKMessage` | `multisig_verify.go:1112` | all 4 branches (legs≤1/legs>1 × full/not); legs≤5 | 494 / 494 / 494 / 476 |
| reused-key notices (2 Sprintf) | `multisig.go:186,193` | 4 slots — the max one seed can hold in an N≤5 policy (`len(slots)>=2` required to reach this code) | 339 / 418 |
| `abortWarning` Sprintf | `derive_xpub.go:527` | double-digit `done`/`total` (production census docs "6-9 plates over hours" for this kind of count) | 455 |
| 3 freetext refusals | `freetext_flow.go:949,955,1175` | line counts generously double/triple-digit; entered-character count at 200 (keystrokes are "always accepted", uncapped before this refusal) | 494 / 494 / 476 |
| Done-adding-cards messages (`msg`, `pendingMsg`) | `bundle_flow.go:199,215` | the `!hasReader` literal variant of each (strictly longer than the `hasReader` variant) | 476 / 455 |
| 3 constant-concat bodies | `multisig_build.go:57`, `unlock_flow.go:153`, `unlock_kdf.go:93` | fixed value each, no search | 476 / 397 / 397 |
| codex32-too-long refusal | `unlock_kdf.go:448` | `seal.MaxEngraveableCodex32Len` is a build constant (90); no runtime variance | 455 |
| `unlockHashBody` | `unlock_flow.go:92` | 24 public records (§6.4's documented cap), UNSEALED (longer than SEALED) | 360 |
| `unlockRetryBody` | `unlock_kdf.go:425` | `HasHash=true` branch (the `false` branch is a shorter plain literal); same 24-record/UNSEALED worst case | 244 |
| `err.Error()` from `address.Change`/`Receive` | `address_polish.go:60` | reconstructed string matching the producing package's documented format + longest `bip380.Script.String()` ("Nested Segwit (P2SH-P2WPKH)"); see caveat below | 476 |

**Caveat on `address_polish.go:60`**: the real error (`address` package's
unexported `errUnsupported`, wrapped `"address: multisig script: %s: %w"`) is
not constructible from `package gui` (unexported identifier, different
package) and the call site's own precondition
(`descriptorAddressFlow`'s doc comment: "the caller opens this only when
`address.Supported(desc)`") means this branch is not normally reachable in
production at all. The test reproduces the **documented format string**
verbatim with the longest `Script.String()` value as a representative worst
case, rather than exercising the (currently unreachable) real error path.
Stated explicitly rather than silently assumed.

### ALREADY GATED, before P7 — 9 call sites across 8 producers

| producer | site(s) | existing test | note |
| --- | --- | --- | --- |
| `bundleAbortWarningText` | `bundle_flow.go:505` | `gui/bundle_abort_prose_test.go:188` `TestAbortWarningsAreDrawnInFull` (both arms) | pre-S6b |
| `bundleMs1ReminderText` | `bundle_flow.go:466` | P6's `TestModalsThisBlockTouchesAreDrawnInFull` (`gui/modal_fits_test.go:304`) | S6b, P6 |
| `multisigVerifyNoSlotBody` | `multisig_verify.go:931` | P1/P6's sweep, plus `gui/multisig_verify_passphrase_test.go:103-105` (all 3 arms) | explicitly excluded from P7's scope by the dispatch brief |
| `multisigVerifyIncompleteText` | `multisig_verify.go:1091` | `gui/multisig_verify_report_test.go:297` — **but only over 2 cases totalling 3 slots, not this policy's N=5 worst case** | **supplemented** by P7's `TestMultisigVerifyIncompleteTextWorstCase` (4 checked + 1 outstanding); headroom 156 |
| `multisigVerifyFailureText` | `multisig_verify.go:1075,1100` | `gui/multisig_verify_report_test.go:812` (all 3 error shapes) | pre-S6b |
| `multisigVerifyCoveredSeedBody` | `multisig_verify.go:943` | `gui/multisig_verify_report_test.go:899` (all 4 boolean combinations) | pre-S6b |
| literal-concat body | `multisig_build.go:317` | P5/P6's `TestModalsThisBlockTouchesAreDrawnInFull` | S6b |
| literal-concat body | `singlesig_verify.go:195` | P6's `TestS6bModalFitSweep` (F-204 "passphrase entered" arm) | S6b, P1 |

### UNGATED, WITH REASON — trivially short, 8 call sites

Pinned by `TestS6bP7TriviallyShortBodiesAreNotCandidates`, which fails if any
of them reaches P6's own established 87-character floor (the shortest body
`TestModalsThisBlockTouchesAreDrawnInFull` treats as worth checking), so
future growth has something to turn red rather than relying on this table.

| producer | site(s) | longest value | chars |
| --- | --- | --- | --- |
| `ppEntryError` | `passphrase_flow.go:104` | "Too long. At most 100 characters fit on one plate." | 52 |
| `verifyProvenanceLine` | `plate_verify.go:335` | "device-compared (%d of %d)" | ~30 |
| `slip39words.Describe` | `gui.go:2674`, `slip39_polish.go:244,249,303` | "member threshold mismatch" | 26 |
| `seedxor.Describe` | `seedxor_polish.go:63` | "unsupported length (use 12/18/24 words)" | 40 |
| single-line refusal concat | `freetext_flow.go:1171` | "The footer is a single line." | 29 |

**Total accounting**: 27 (P7-gated call sites) + 9 (already-gated call sites)
+ 8 (trivially-short, excluded) = **44**, matching the AST scan's composed
count exactly. Nothing is unaccounted for.

## The finding: `buildFingerprintContradictsMessage` was over budget

Reported before it was touched, per the phase's instruction. It was covered
only by `gui/multisig_build_gate_test.go`'s
`TestGateRefusalsAreDrawnWithoutScrolling`, which **looks like** a fit check —
it renders `showError`'s first frame and asserts an ink floor — but only
asserts that three **named substrings** appear on that frame
(`uiContains` per phrase: `"@0"`, `"Nothing was engraved"`, `"me sysw pack"`),
never that the **whole body** does. That is exactly the seam F-185 exists to
close: a body cut mid-sentence still passes a substring check as long as the
substrings it happens to look for land before the cut. This is a genuine gap,
not a hypothetical — `buildSeedKeyMismatchMessage` and `buildDuplicateKeyMessage`
sit under the same look-alike test and (as it turned out) do pass the real
check; `buildFingerprintContradictsMessage` did not.

Under the real class check at its worst case (both fingerprints present,
`who` non-empty), the body **drew in full** (not truncated today) but at only
**42 characters of headroom** against the 80-character margin — under F-185's
own standard, `t.Errorf`'d by `assertModalBodyFits`.

**TDD, red → green:** first run reported "fits today with only 42 characters
to spare... Shorten this body rather than lowering the margin" (test FAIL).
The copy was shortened — three sentences tightened, **no assertion removed**:
still names the slot and `who`, quotes both fingerprints, states "Nothing was
engraved", explains a fingerprint is not bound to the key (written by whoever
made the card), states the likely cause (a different passphrase), and gives
the `me sysw pack` route:

```
Slot @%d%s carries the key your seed derives, but the card says its master
fingerprint is %s and this seed's is %s. Nothing was engraved.

A fingerprint is written on a card by whoever made it and is not bound to the
key, so the two can disagree. Most often the card was made under a different
passphrase.

Re-enter the seed with that passphrase. Reassigning the slot only stops the
check. If the card is stale, rewrite the payload with `me sysw pack`.
```
→
```
Slot @%d%s's key derives from your seed, but the card's fingerprint is %s and
your seed's is %s. Nothing was engraved.

A fingerprint is written by whoever made the card and is not bound to the
key, so they can disagree, often from a different passphrase.

Re-enter the seed with that passphrase. Reassigning only stops the check. If
the card is stale, rewrite the payload with `me sysw pack`.
```

Re-run: **128 characters of headroom** (PASS). Verified no existing test
depended on the exact old wording beyond the three `uiContains` substrings
above (all three preserved) — checked
`TestGateRefusalsAreDrawnWithoutScrolling`, `TestGateErrorDispatchRoutesEveryArm`,
`TestGateRefusesContradictingFingerprint`, `TestGateNeverPrintsSeedOrPassphrase`,
`TestMultisigVerifyNoticeIsHonest`; all still pass.

Every other body in the coverage table above is a **regression pin, green
throughout** (drew in full with margin to spare on first measurement) — stated
per-case in the table, not dressed as red→green.

## Build gate

- `go build ./gui/...` — clean. `gofmt -l gui/s6b_p7_modal_fit_sweep_test.go` —
  clean (one fix applied after initial write).
- `go vet ./gui/...` — only the two pre-existing go1.26 `t.ArtifactDir()`
  findings the plan already names (`gui/op/draw_test.go:176`,
  `gui/freetext_sizeproof_golden_test.go:111`); nothing new from this phase.
- `go test ./gui/ -run <new tests> -v`: all 29 `TestS6bP7ModalFitSweep`
  subtests PASS, `TestMultisigVerifyIncompleteTextWorstCase` PASS,
  `TestUnlockHashAndRetryBodiesWorstCase` PASS (2 assertions),
  `TestS6bP7TriviallyShortBodiesAreNotCandidates` PASS. Regression check on
  the five tests neighboring the `buildFingerprintContradictsMessage` fold
  (named above) — all PASS.
- **Sharded gate**: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 6 20m`
  (cwd = the worktree). Exhaustiveness line: `partition verified exhaustive:
  858 == 858`. Wall: **127s**. Result: **exactly one failure**, shard 2 —
  `TestGate51bMaxScrollAgreesWithVisibility`, reporting "22 diverge" over
  `bodysz.Y` in `[239,260]` against a 321-value probe range (`[0,320]`),
  logged "EXPECTED (R-E)". Matches the brief's prediction (22 of 321 values in
  `[239,260]`) exactly. **All 6 shards' stderr files are 0 bytes** —
  `err0.txt` through `err5.txt` confirmed individually. No other failure
  anywhere.
- **Non-`gui` packages**: `go test $(go list ./... | grep -v '/gui$') -count=1 -timeout 20m`
  — every listed package `ok` or `[no test files]`, zero failures (68
  packages, including `seal`, `bundle`, `md`, `mk`, `address`, `passphrase`).
- **Goldens**: `git status --short` before this commit showed exactly
  `M gui/multisig_build_slots.go` and `?? gui/s6b_p7_modal_fit_sweep_test.go`
  — no `backup/testdata/*.bin` or any other tracked golden moved.

## The occlusion boundary (stated, not extended)

Same boundary P6 states, restated rather than re-derived. `bodyDrawnFully`
(`gui/modal_fits_test.go:81-100`) compares the drawn frame's **op tree**
against the source string; `ExtractText` has no notion of what visually sits
on top of what, so a glyph the panel draws **underneath** P5b's opaque
scroll-arrow chip is still "on the frame" as far as this check can see — the
text reached the compositor, whether or not a chip was painted over it
afterwards. That is **occlusion**, not truncation, and it is **GATE 5.3's**
job (`gui/scroll_arrows_test.go`), not this sweep's. P7 does not attempt to
extend to cover it and makes no claim that it does.

## What the spec/plan got wrong, or was ambiguous about

Nothing required a STOP. The spec (§4, "Every long modal body is gated") and
the plan's P7 addition (`design/IMPLEMENTATION_PLAN_s6b.md` §1, added
2026-08-18 explaining P6's narrowing) were both unambiguous about this phase's
scope. The dispatch brief's own bounding scan had the three inaccuracies
listed above (address_polish.go, slip39_polish.go, freetext_flow.go) — the
brief anticipated this ("a starting point, not an authority") and asked that
any miss be reported, which this does. The one substantive discovery is not a
spec/plan defect but a **pre-existing test-suite gap**: two look-alike tests
(`TestGateRefusalsAreDrawnWithoutScrolling`'s substring/ink-floor check) had
stood in for the real F-185 class check on 4 producers since before S6b,
without ever being run against it — which is precisely the class of exposure
F-192 was filed to close, now closed for the whole firmware.

## Result

GATE 4 is now firmware-wide green: 44 composed bodies fully accounted for (27
newly gated, 9 already gated — one supplemented to its true worst case, 8
excluded with a stated, pinned reason), one genuine over-budget finding fixed
by shortening copy (no assertion removed, no margin lowered), zero
regressions, zero moved goldens, and the sweep stays inside the occlusion
boundary GATE 5.3 owns. This closes F-192 and P7, the last phase of the S6b
implementation plan's modal-fit work. The whole-diff independent adversarial
review (mandated, non-deferrable per `IMPLEMENTATION_PLAN_s6b.md` §4) is next.
