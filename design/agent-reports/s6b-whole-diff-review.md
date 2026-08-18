# S6b whole-diff adversarial review — the last gate before the flash

**Scope:** `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`, 14 commits
`b1479a1..12a428d` (tip verified = `12a428d6fe250fa235cc0dbc7943cd74084a7ba1`), all seven
phases P1–P7. Worktree verified clean. Every claim below marked *[run]* was executed by this
review, not inherited from a phase report.

**Machine-checked before judging:** full `gui` suite via
`scripts/gui-shard-test.sh ./gui/ 6 20m` *[run]* — 858 tests enumerated, partition verified
exhaustive, **exactly one failure: `TestGate51bMaxScrollAgreesWithVisibility`** (the
designed-red probe, see queued item 3); all shard stderr empty. `backup` and `passphrase`
packages ok *[run]*. Goldens: the frozen sixteen byte-identical; exactly one new file,
`backup/testdata/passphrase-4-preloaded.bin` *[run: `git diff --stat -- backup/testdata/`]*.
No codec package touched — the diff is confined to `backup/` and `gui/`, so
`md1`/`mk1`/`ms1` stay byte-identical and the Rust-primary rule is not triggered *[run:
diffstat over the whole range]*.

---

## 1. Findings table

| id | sev | phase(s) | one line |
| --- | --- | --- | --- |
| C1 | **Critical** | P3 | The preloaded plate flow re-presents the passphrase in a fully **editable** keyboard; an edit engraves the edited passphrase under the *original* derivation's fingerprints + policy id, stamped DERIVED |
| C2 | **Critical** | P3 (× pre-existing entry surface) | A wallet passphrase >100 chars derives the wallet but is **silently truncated** by the preloaded flow's `copy`; the truncated passphrase engraves with the full passphrase's derived fingerprints, stamped DERIVED |
| I1 | **Important** | P5/P5b (× all modal screens) | Hold-to-scroll to the bottom leaves the hidden down arrow's press state **stale**; it then auto-repeats with no finger on the panel, pinning every overflowing safety modal at full scroll — *empirically confirmed* |
| I2 | **Important** | P3 / process | Spec §2.3a's **GATE 2.3e** ("confirm screen's provenance clause agrees with the footer's on both paths") never became a test — silently dropped from the spec's own §7 table and the plan's P3 row; the typed path's confirm clause has no pin at all |
| M1 | Minor | P4 | The cut-arm passphrase paragraph drops "Without it, these plates do not reach the money." — the money consequence survives only on steel (P2's title) and only in full mode |
| M2 | Minor | P2/P3 drift | `gui/singlesig.go:111` "The mnemonic is consumed for the LAST time here." is now false — S6b's own lazy derivation (`:336`) consumes it later |
| M3 | Minor | P5 drift | "no production path on SeedHammer II emits [Up/Down]" (`gui/passphrase_flow_test.go:772-775` + its error strings) falsified by P5's arrows; quoted approvingly into `gui/s6b_modal_fit_sweep_test.go:91-93` in the same cycle |
| M4 | Minor/Nit | P3 | `Derived:true, PolicyID:""` renders footer `POLICY   DERIVED` — the defensive `serr == nil` arm at `gui/singlesig.go:348` swallows the stub error; unreachable today, malformed if ever reached |
| M5 | Nit | P4 | "nothing this device engraves carries a passphrase" was already false as a *device-wide* claim pre-S6b (the standalone passphrase program); S6b's declined-offer runs make the tension proximate. Sentence is frozen by ruling for not-cut runs — record only |

**Verdict: RED 2C/2I.**

---

## 2. Finding blocks

### C1 (Critical, P3) — the preloaded entry step is editable, and nothing re-checks it

**The defect.** `engravePassphraseFlowPreloaded` (`gui/passphrase_flow.go:804`) copies the
session passphrase into `secret` (`:813`) and then hands it to the standard **editable**
keyboard: `passphraseEntryFlow(ctx, th, secret, n, nil)` (`:831`), which seeds
`kbd.Fragment = string(dst[:n])` (`gui/passphrase_flow.go:76-78`) and lets the operator
add/delete/replace anything before OK. The flow then proceeds with the **original** derived
`seedFP`/`combinedFP`/`policyID` and `Derived: true` — no comparison of `secret[:n]` against
`body` exists anywhere.

**The reachable case.** Operator accepts the plate offer; on the entry screen (a live
keyboard whose whole function is editing) they "fix" a character they believe was mistyped,
press OK, confirm, engrave. Every screen along the way tells them the values are
derivation-grade: the acceptance screen says *"Source: this session's own derivation"*, the
confirm screen says *"Fingerprints are derived by this device"*, the plate footer says
`POLICY <id>  DERIVED`.

**Why it is wrong.** The steel now binds passphrase P′ to `EXPECTED SEED FP` / `EXPECTED
COMB FP` values derived from P, under the device's own authority. At restore, P′ produces a
fingerprint mismatch against the plate's own EXPECTED lines — R4's self-diagnosis fires, but
it diagnoses *the wrong thing* ("wrong wallet") — and the wallet's true passphrase P is
recorded **nowhere**. That is permanent funds loss discovered years later. It is also the
exact guarantee the spec rests the whole preload design on: spec §2.1 — *"the preloaded
passphrase is the one the device actually derived with, so the plate records the passphrase
belonging to the wallet that was engraved. Re-typing adds a second chance to disagree, not a
check."* The implementation reintroduced the second chance to disagree, and upgraded its
label from the standalone path's honest `FINGERPRINTS TYPED, NOT VERIFIED` to `DERIVED`.
Note the provenance claim is rendered in **three** places (acceptance screen, confirm
screen, footer) — the brief's "assume there is a third" — and one unchecked assumption
falsifies all three at once.

**Smallest fix.** After the entry step returns on the preloaded path, compare and refuse:
if `!bytes.Equal(secret[:n], body)`, show a truthful message (e.g. *"The passphrase was
changed. A passphrase plate must record the exact passphrase this wallet was derived
with."*), re-seed the buffer from `body`, and stay on the entry step. ~8 lines, one file.
(Alternative: skip the entry step entirely on the preloaded path — the confirm screen
already reveals the passphrase for proofreading, spec 5.1 — but that changes the step
machine; the comparison is smaller.)

### C2 (Critical, P3) — silent truncation of a >100-char wallet passphrase, then DERIVED steel

**The defect.** The wallet-derivation passphrase is unbounded: `passphraseFlowTitled`
(`gui/gui.go:806-833`) returns `kbd.Fragment` with **no** `ValidatePassphrase` call, and the
payload branch of `syswPassphraseFlowTitled` (`gui/sysw_source.go:98-113`) returns
`string(raw)` unbounded. `deriveSingleSigBundle` derives with the full string. The preloaded
flow then does `secret := make([]byte, passphrase.MaxLen)` (100) and
`n := copy(secret, body)` (`gui/passphrase_flow.go:813`) — silent truncation.
`ValidatePassphrase` passes the truncated 100 chars (`passphrase/passphrase.go:23-40`;
exactly-100 is legal), so no screen refuses.

**The reachable case.** Operator enters a 101+-char printable-ASCII passphrase at "Add a
BIP-39 passphrase?" (accepted without complaint), engraves, accepts the plate offer. The
entry readout is **masked by default**; the counter shows `100/100`, which reads as fine.
OK → confirm → engrave.

**Why it is wrong.** The plate carries the truncated passphrase plus the **full**
passphrase's combined fingerprint and policy id, stamped `DERIVED` — the same
false-vouching steel as C1, reachable without any edit. (The identical `copy` in
`engravePassphraseFlowFrom:653` is pre-existing, but on that path the plate says `TYPED,
NOT VERIFIED` and the operator typed the passphrase *for* the plate; only the preloaded path
attaches derivation-grade claims to a value it silently mutilated.)

**Smallest fix.** Validate the *actual* wallet passphrase before preloading: in
`singleSigPassphrasePlateOffer` (`gui/singlesig.go:318`), before or instead of the offer,
`if err := passphrase.ValidatePassphrase(passphrase); err != nil { showError(ctx, th,
"Passphrase Plate", ppEntryError(err)); return passphrasePlateNotCut }`. ~4 lines; also
handles a non-ASCII payload passphrase earlier and more truthfully than the entry step's
refusal loop. (C1's byte-compare then keeps the entry step from re-introducing the
mismatch.)

### I1 (Important, P5/P5b) — a hidden arrow's press state goes stale; the arrow then scrolls by itself

**The defect.** `Warning.Layout` gates both the input region *and* the event pull on the
visibility predicate: the region is only emitted inside `if showDown { … }` and events are
only pulled by `if showDown && w.arrowDown.Clicked(ctx)` (`gui/gui.go:424-429`). GATE 5.1
itself *requires* the down arrow to disappear at full scroll — which press-and-hold
auto-repeat (a designed feature, `gui/widget.go:48-68`) reaches with the finger still down.
The router's capture then looks up the pressed tag in the **current** frame
(`gui/event.go:307-309`), finds the region gone, nils the tag, and the release event is
discarded at `Reset()` — `arrowDown.Pressed` stays `true` forever. When the arrow
re-appears, `Clicked` hits the auto-repeat branch first and fires with **no finger on the
panel**.

**Empirically confirmed** *[run]*, via a scratch test driving real pointer events with
production frame semantics (`ctx.Reset()` per frame): hold-to-bottom → arrow hides →
release dropped → `Pressed` stale → one tap on the up arrow scrolls 91→0 → the down arrow
re-appears and **ghost-auto-repeats scroll 0→91** with zero input. Scratch test deleted
after use; worktree clean.

**The reachable case.** Any overflowing modal — the blast radius is `ErrorScreen` (every
`showError`) and `ConfirmWarningScreen` (`gui/gui.go:314-341`), i.e. exactly the safety
screens the arrows exist for. The operator holds the down arrow to read to the end (the
natural gesture the feature invites), then tries to scroll back up: the page snaps back to
the bottom on every attempt for the life of that screen instance. The up arrow becomes the
precise false affordance spec §5.1 was written to forbid — *"a control that visibly does
nothing on a safety screen teaches the operator that the arrows do not work"* — except
worse: it visibly does the opposite. No committed gate can see it: GATE 5.1's interaction
test drives synthetic Button events (`click(&ctx.Router, Down)`) that no production path
emits — the identical blind spot `gui/start_screen_touch_test.go`'s own header documents
from the pager bug.

**Smallest fix — verified** *[run]*: clear the press state whenever a direction is hidden —
in `Warning.Layout`, after the two handlers: `if !showUp { w.arrowUp.Pressed = false }` /
`if !showDown { w.arrowDown.Pressed = false }`. Two lines. Applied experimentally: the
stale press clears, up-tap works (91→0), no ghost repeat, and **all GATE 5.1 tests stay
green**; then reverted. The regression test for it should drive **touch** at the chip
coordinates (my probe confirmed touch routing itself works today: a tap at (214,306)
scrolls), which also closes the Button-path-only gap.

### I2 (Important, P3/process) — GATE 2.3e was specified, then silently vanished

**The defect.** Spec §2.3a is normative: *"GATE 2.3e: the confirm screen's provenance
clause agrees with the footer's on both paths. They are two renderings of one fact and must
not be able to disagree."* That gate appears in neither the spec's own §7 gate table, nor
the plan's P3 row, nor any test — `grep -rn "2.3e"` over `gui/` and `backup/` returns
nothing *[run]*. This is the same scope-narrowing failure P7 was created to correct (a
requirement shrunk while keeping its name), one layer down. Current coverage: the derived
path's two surfaces are asserted separately
(`TestPreloadedConfirmScreenNamesDerivedProvenance`, `TestPassphraseFooterProvenance`); the
**typed path's confirm clause ("Fingerprints are typed, not verified") has no pin at all**
*[run: grep]* — a regression could make the typed confirm screen claim derivation with no
red test — and no assertion couples the two renderings. Structurally they *can* disagree:
`ppConfirmFlow` and `ppBuildPlate` take independent `derived` parameters
(`gui/passphrase_flow.go:497,569`), agreeing today only because each caller passes literals.
The cycle's own closure rule applies: a gate that has never executed is a hypothesis.

**Smallest fix.** One test: for `derived ∈ {false, true}`, drive `ppConfirmBody` and
`passphraseLayoutFor` from the same inputs and assert the screen's provenance clause and the
footer's provenance form agree (typed↔typed, derived↔derived). ~25 lines, no production
change.

### Minors (recorded, not gating)

- **M1 (P4).** The not-cut arm says *"Without it, these plates do not reach the money."*;
  the cut arm (`gui/multisig_build_census.go:301-309`) replaces that sentence with
  plate-handling instructions and never states the money consequence. Mitigation exists on
  full-mode steel (P2's `PASSWORD REQUIRED` title) but not in the document itself, and not
  at all on a watch-only+passphrase run (unmarked per R-A). Smallest fix: append the
  sentence to the cut arm's first line. Wording is spec-unsettled (§8), so this is a
  content-omission note, not a rewording proposal.
- **M2 (drift).** `gui/singlesig.go:111` still says the mnemonic is consumed for the last
  time at `:112`; S6b's lazy derivation at `:336` reads it later (sound — the mnemonic is
  caller-scrubbed at return, `singlesig_derive.go:40-43` confirms it is not wiped inside).
  The stale comment is the exact "comments outlive their conditions" class: a reader moving
  the scrub earlier on its authority would make the lazy derivation produce *valid-looking
  wrong fingerprints*. One-line comment fix.
- **M3 (drift).** `gui/passphrase_flow_test.go:772-775` and its two error strings claim no
  production path emits Up/Down; P5's arrows made that false, and
  `gui/s6b_modal_fit_sweep_test.go:91-93` quotes the stale text in the same cycle that
  falsified it. The load-bearing conclusion (ppConfirm has no scroller; the screen must fit)
  survives. Comment fix, two files.
- **M4.** `passphraseFooterFor` with `Derived:true, PolicyID:""` renders
  `POLICY   DERIVED`. Unreachable today — `md.FormAwareStubChunks(b.MD1)` already succeeded
  on the same input during derive (step 4 of `deriveSingleSigBundle`) and again if
  templateized, so the `serr == nil` guard at `gui/singlesig.go:348` cannot fail in
  practice — but the defensive arm should refuse rather than render a malformed claim.
- **M5.** Recorded only; the not-cut sentence is frozen by ruling (§6/6a) and remains true
  of "these plates".

---

## 3. The three queued items

### 3.1 — P1's GATE 3.1 spec amendment (source assertion replacing a behavioural walk): **CONFIRMED sound**

The amendment's core claim — whether a verdict re-offers is decided *only* by the callers'
loop condition — is a source fact and holds, re-verified here rather than inherited:
the loop `if res != verifyIncomplete && res != verifyFailed { break }` is verbatim-identical
at `gui/multisig.go:346` and `gui/multisig_build.go:461` *[run: read both]*; both dispatch
through `multisigVerifyFn = multisigVerifyFlow` (`gui/multisig_verify.go:695`), which has no
other production caller *[run: grep]*; no retry construct exists at any refusal site. The
`:854`-unreachable argument holds: `verifyFreshSlots` has exactly **one** error return, on
`len(expected) == 0` *[run: read the function body]*; `multisigVerifyFlow:744` refuses that
exact condition before the derive loop; `expectedSlots` is never reassigned (comment-stripped
source scan in `TestExpectedSlotsNeverReassignedInVerifyFlow`, plus my own grep of all 15
occurrences *[run]*). Residual weakness — a substring pin proves presence, not that the
pinned line is *the* governing loop — is disclosed inside the test file itself, and the
behavioural walk `TestBothEngraveFlowsDriveTheRetryLoop` (exists,
`gui/multisig_engrave_tail_walk_test.go:365` *[run]*) covers the loop's positive half on the
same inline code. The 119 s cost argument against the symmetric walk is consistent with the
measured suite budget. No finding.

### 3.2 — F-199 widening `verifyIncomplete`'s meaning globally: **CONFIRMED sound**

Every consumer of the widened verdict audited *[run: grep, then read each]*: (1) the two
retry loops — re-offer, which is the intended fix; (2) the re-offer lead
`multisigVerifyRetryLead = "Not every plate is verified. Try again?"`
(`gui/multisig_verify.go:61`) — true in the new case (nothing is verified); (3) the restore
document reads **no verdict at all** — `verifyStatusFor` derives from the recorded
`{pass, adverse}` only (`gui/verify_status.go:110-121`), and the readback-failure site sets
`rec.adverse = true` before returning, so a fail-then-clean-retry run reports
`statusVerifiedOnRetry` and a fail-then-walk-away run reports the DidNotPass line, whose
text — *"a plate could not be read or accounted for"* — literally names this case. The
doc-comment rewrite at `gui/multisig_verify.go:82-96` accurately re-describes the verdict.
No falsehood in any consumer. No finding.

### 3.3 — GATE 5.1b expected RED: **CONFIRMED RED, by execution**

*[run]* `go test ./gui/ -run TestGate51bMaxScrollAgreesWithVisibility` fails exactly as
required: **22 of 321** `bodysz.Y` values in **[239,260]** where `maxScroll > 0` disagrees
with GATE 5.1's panel predicate — matching the brief's settled numbers digit-for-digit. The
full-suite run reproduces it as the *only* failure among 858 tests. The test's own text
forbids loosening it and names the restoration condition (R-E's honest-geometry work). Red
here is data, recorded as such.

---

## 4. Cross-phase interactions checked and found SOUND

1. **P2 marking ↔ P3 plate ↔ P4 document on one run.** `COMB FP` on mk1/md1 (title/footer),
   `EXPECTED COMB FP` on the passphrase plate, `Fingerprint` inside the mk1 wire card, and
   the restore document's master fp all render the same `masterFP` variable from one
   derivation (`gui/singlesig.go:112`); the plate's `POLICY` id is value-equal to the mk1's
   own carried stub on both md1 forms (GATE 2.4b is a real value-equality test incl. a
   forms-differ guard against coincidence). Consistent by construction; C1/C2 are the two
   ways the *passphrase* can fall out of that consistency while every label still claims it.
2. **P3 offer ↔ P4 condition (cut, not offered).** Proven end-to-end:
   `TestRestoreDocReflectsARealCutPassphrasePlate` drives the real orchestrator through an
   actually-completed passphrase-plate engrave and reads the real document; decline, Back,
   and derivation-error paths all collapse to NotCut (unit + flow tests). The multisig
   callers hard-pass `false` with R-B comments. Sound.
3. **P1's F-199 ↔ P4's document** — via the record, not the verdict; audited under queued
   item 3.2. Sound.
4. **P5 arrows ↔ P6/P7 sweep boundary.** The sweep's op-tree check cannot see occlusion
   under the opaque chip; both files state this boundary and GATE 5.3 covers it with real
   pixel assertions (first/last readable rows clear the chip bands; chip-centre opacity).
   Body width pinned at 417 (`TestBodyClipWidthStaysAt417`), so R-I's decoupling holds and
   P6/P7's fit measurements remain valid. Sound (I1 is an input-state defect, not a geometry
   one).
5. **P1's R-M body ↔ P6.** The 251-char arm is pinned byte-for-byte, passes the real class
   check in both P1's own gate and P6's sweep, carries no em dash, and the struck/forbidden
   claims are negatively pinned. Sound.
6. **F-206's clause ↔ both verify flows.** "The ms1 you typed for each seed matched." is
   true on multisig (typed per seed) **and** on single-sig full verify — confirmed the ms1
   is hand-typed there too (`inputCodex32Flow(ctx, th, "Type ms1")`,
   `gui/singlesig_verify.go:157-166`), not read back. GATE 3.3 is genuinely flow-level with
   the 1-seed/2-legs middle case that kills the filed remedy. Sound.
7. **P2 optionality ↔ R-G goldens.** Empty title/footer provably byte-identical (frozen
   sixteen unmoved under the full suite); marked states got new goldens only. The budget
   gate is layout-based and pinned both ways (28 fits, 29 refused). Sound.
8. **P7's copy fold** (`buildFingerprintContradictsMessage` shortened): all load-bearing
   content retained (slot, both fingerprints, "Nothing was engraved", the not-bound
   explanation, the passphrase cause, the `me sysw pack` route); red→green under the real
   class check; the look-alike substring test it escaped is exactly the pattern the brief
   predicted a second instance of — P7 found it itself and said so.

Residual nits for a future sweep, no gate held: the variant-render test proves *some*
marking inked, not footer-specifically (a footer-only vanish would pass);
GATE 5.3's top-chip opacity sample is vacuous at scroll 0 (top chip absent there); no
behavioural test binds the `plateTitle, plateFooter` argument order at
`gui/singlesig.go:193` (a swap would render both, transposed, all green — the order is
correct today, verified by reading).

---

**Verdict: `RED 2C/2I`** — C1 and C2 put false derivation-grade claims on steel through the
cycle's own new flow and block the flash; I1 breaks the scroll-back affordance on every
overflowing safety modal (empirically demonstrated, two-line fix verified); I2 is a
specified gate that never ran. All four have small, localized fixes; nothing found
challenges the cycle's design decisions themselves.
