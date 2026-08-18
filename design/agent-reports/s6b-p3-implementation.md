# S6b P3 implementation — the passphrase plate (spec §2)

**Worktree:** `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`.
**Commits:** `381c95e` (backup-package mechanism), `fc2efa2` (gui-package
wiring), both on top of P2's `0166be4`.

Scope: `SPEC_s6b_pre_flash_cycle.md` §2 only. Gates 2.1, 2.2, 2.3, 2.3b,
2.3c, 2.3d, 2.4a, 2.4b, 2.4c, 2.5, 2.6.

---

## The most important thing in this report

**R-H's literal footer string does not fit the plate's own pre-existing
band-width test, and shipping it as specified would engrave 8mm into the
screw-hole zone.** Found by running `TestPassphraseBandBudget` against the
spec's own string before trusting the ruling — not by inspection. Full
account under "Findings" below; the corrected 25-character string is what
shipped, flagged as **not operator-approved wording** and needing
confirmation at this phase's gate.

---

## What changed and where

**`backup/passphrase.go`** (commit `381c95e`):

- `Passphrase` gains `PolicyID string` (spec 2.4's pre-formatted 8-hex
  wallet-policy-id/WDT-id stub, or `""`) and `Derived bool` (the
  PROVENANCE of `SeedFP`/`CombinedFP`: `true` iff this device derived
  them, `false` — the zero value — when the operator typed them).
- New `passphraseFooterFor(plate Passphrase) string` selects the footer
  TEXT by `plate.Derived`, never by `plate.PolicyID != ""` (spec 2.3d,
  R-D). `passphraseLayoutFor`'s existing gate — a footer line renders at
  all only when `SeedFP != "" || CombinedFP != ""` — is unchanged; what
  changed is which STRING fills that line.
- The typed-path string (`passphraseFooter`, "FINGERPRINTS TYPED, NOT
  VERIFIED") is byte-identical to before. The derived-path string is
  `"POLICY " + <8-hex grouped> + "  DERIVED"` — **25 characters, not
  R-H's specified 36** (`passphraseFooterDerivedSuffix`'s doc comment
  carries the full measured correction; see "Findings" below).

**`gui/sysw_admit.go` / `gui/sysw_source.go`** (commit `fc2efa2`):

- New `syswSource` value `srcDerived` — "carried from this session's own
  derivation" (R-C/R-J) — with an explicit `case` in `syswSourceName`
  returning `"this session's own derivation"`. The `default:` arm still
  resolves to `"the keyboard"`, so a value added without its own case
  would have been a printed falsehood with no compile error — exactly the
  trap spec 2.2 names.

**`gui/passphrase_flow.go`** (commit `fc2efa2`):

- New `ppPreloadedStep` type (`ppPLStepEntry, ppPLStepQR, ppPLStepConfirm,
  ppPLStepEngrave`) and `engravePassphraseFlowPreloaded(ctx, th, body,
  seedFP, combinedFP, policyID string)`. This is a **separate, shorter**
  step sequence from `engravePassphraseFlowFrom`'s existing `ppStep`, not
  the same sequence with the two fingerprint cases skipped inside it —
  spec 2.1.2 requires elision, and the reason is mechanical: `ppStep`'s
  Back transition is `step -= 2` ahead of an unconditional `step++`, so a
  present-but-instant fingerprint case would let Back from the QR step
  bounce silently back to QR through it. A type that cannot even name
  those two steps makes that impossible rather than merely untested (see
  GATE 2.1's mutation check below).
- `ppBuildPlate` and `ppConfirmFlow`/`ppConfirmBody` gain `policyID
  string, derived bool` parameters, threaded straight to
  `backup.Passphrase`. The typed path's one caller
  (`engravePassphraseFlowFrom`) passes `"", false` — byte-identical to
  pre-S6b behaviour.
- `ppConfirmWarningDerived`, a new confirm-screen string used when
  `derived == true` — see "A second truthfulness fix" below.

**`gui/singlesig.go`** (commit `fc2efa2`):

- New `singleSigPassphrasePlateOffer(ctx, th, mnemonic, passphrase,
  masterFP, path, b)` implements spec 2.6: offered only when `passphrase
  != ""`, inserted in `engraveSingleSigFlow` between the verify offer and
  `restoreDocFlow`. The bare-seed fingerprint derives LAZILY, behind the
  new `singleSigBareSeedFPHook` test seam, only when the operator accepts
  (R-K). The policy id is `md.FormAwareStubChunks(b.MD1)` — **not**
  `md.WalletPolicyIDStub`, the keyed-only branch — computed from the
  caller's FINAL `b`, i.e. post-`templateizeBundle` if that branch was
  taken (spec 2.4/2.4c).

**`gui/singlesig_derive.go`** (commit `fc2efa2`): the stale doc comment
naming `md.WalletPolicyIDStubChunks` (the wrong, keyed-only function)
where the code has always called `md.FormAwareStubChunks` is corrected,
per spec §8's instruction that it land with the §2.4 work.

**`gui/singlesig_truth_test.go`** (commit `fc2efa2`): pre-existing S6a
walk helper `s6aSingleSigWalk` gained one step (decline the new S6b
offer) — see "Regression found and fixed" below.

**New test file**: `gui/s6b_passphrase_plate_test.go` (gates 2.1, 2.2,
2.4a, 2.4b/2.4c, 2.6).

**Test call sites updated to compile** (new `ppBuildPlate`/`ppConfirmFlow`/
`ppConfirmBody` parameters): `gui/passphrase_flow_test.go`,
`gui/passphrase_passproof_test.go`, `gui/preview.go` (the CLI preview
sidecar, not a test — passes `"", false`).

---

## Per-gate TDD evidence

### GATE 2.1 — no fingerprint-entry step; Back from QR lands on a real prior step

New test `TestPreloadedFlowElidesFingerprintSteps`
(`gui/s6b_passphrase_plate_test.go`).

**RED** (`go vet ./gui/...` before `engravePassphraseFlowPreloaded`
existed): `undefined: engravePassphraseFlowPreloaded`.

**GREEN**:

```
=== RUN   TestPreloadedFlowElidesFingerprintSteps
--- PASS: TestPreloadedFlowElidesFingerprintSteps (0.02s)
```

**MUTATION-CHECKED**, because a false PASS here is the one that matters
most for this gate: changed the QR step's Back transition from `step -=
2` to `step -= 1` (reproducing, via the shortest possible edit, the class
of bug spec 2.1.2 forbids — a step landing one off from where it should).
Result:

```
--- FAIL: TestPreloadedFlowElidesFingerprintSteps (0.01s)
    s6b_passphrase_plate_test.go:126: Back from QR did not land on the
    real entry screen with the passphrase preserved; got
    "AQRisamachine-readablecopyofthepassphrase.NoQRAddQRQRCode"
```

Confirmed RED, then reverted; reran to confirm GREEN again.

### GATE 2.2 — `syswSourceName`'s rendered string for the new value

New tests `TestSrcDerivedRendersItsOwnName`,
`TestSrcDerivedIsDistinctFromEveryExistingSource`,
`TestSrcDerivedAcceptanceScreenRuns`.

**RED** (`go vet ./gui/...` before `srcDerived` existed): `undefined:
srcDerived` (3 sites).

**GREEN**, all 3:

```
=== RUN   TestSrcDerivedRendersItsOwnName
--- PASS: TestSrcDerivedRendersItsOwnName (0.00s)
=== RUN   TestSrcDerivedIsDistinctFromEveryExistingSource
--- PASS: TestSrcDerivedIsDistinctFromEveryExistingSource (0.00s)
=== RUN   TestSrcDerivedAcceptanceScreenRuns
--- PASS: TestSrcDerivedAcceptanceScreenRuns (0.00s)
```

`TestSrcDerivedAcceptanceScreenRuns` drives the real
`syswSourceAccept(ctx, th, "BIP-39 Password", sysw.ClassPassphrase,
srcDerived)` screen and asserts the rendered content contains "Source:
this session's own derivation" and does NOT contain "no integrity check"
(F4, which must not fire for this source — it is neither `srcNFC` nor
`srcPayload`).

### GATE 2.3 / 2.3b / 2.3c / 2.3d — the footer's selection and mutual exclusion

New test `TestPassphraseFooterProvenance` (`backup/passphrase_test.go`),
3 subtests, run in the backup-package commit.

**RED** (before `PolicyID`/`Derived` existed on `Passphrase`):

```
backup/passphrase_test.go:533:4: unknown field PolicyID in struct literal of type Passphrase
backup/passphrase_test.go:533:26: unknown field Derived in struct literal of type Passphrase
(and 6 more, one per call site touched across the file)
```

**GREEN**, all 3 (after the struct + `passphraseFooterFor` landed, before
the length correction below):

```
=== RUN   TestPassphraseFooterProvenance
=== RUN   TestPassphraseFooterProvenance/typed,_no_policy_id
=== RUN   TestPassphraseFooterProvenance/typed,_policy_id_present_(must_be_ignored,_2.3d)
=== RUN   TestPassphraseFooterProvenance/derived,_policy_id_present_(must_render,_2.3c)
--- PASS: TestPassphraseFooterProvenance (0.00s)
```

- **GATE 2.3** (never co-occur): each subtest asserts the OTHER footer
  form is absent, so the co-occurrence R-H's "50 against 42" arithmetic
  warns about is proven structurally unreachable through the public API,
  not merely untested.
- **GATE 2.3b** (no derivation claim on the typed path): "typed, no
  policy id" and "typed, policy id present" both pin the unchanged
  "FINGERPRINTS TYPED, NOT VERIFIED".
- **GATE 2.3c** (the policy id actually renders): "derived, policy id
  present" — this subtest is the one that caught the band-budget defect
  below on its first real run.
- **GATE 2.3d** (selection by provenance, not presence): "typed, policy
  id present" — `PolicyID` is set, `Derived` is `false`, footer stays the
  typed string.

The width defect GATE 2.3c caught, verbatim, the FIRST time
`TestPassphraseBandBudget` ran with R-H's literal 36-character string
added to it:

```
--- FAIL: TestPassphraseBandBudget
    passphrase_test.go:548: metadata line "POLICY 1A2B 3C4D  DERIVED, NOT TYPED" is 460800 wide, over the 409600 budget
```

This is the finding written up in full below. After the 25-character
correction, `TestPassphraseBandBudget` (extended with a derived+policy-id
case) is GREEN, and `TestPassphraseFooterProvenance`'s "derived, policy
id present" subtest asserts the corrected 25-character string.

### GATE 2.4a — the three device-side `mk.Card` sites set `Fingerprint`

New test `TestDeviceMkCardSitesSetFingerprint`.

**This is a REGRESSION PIN, GREEN THROUGHOUT — not red→green.** Spec §2.4
records this fact as already true today ("all three device-side `mk.Card`
sites set it"), and S6b's own mechanism (the "key-id" = the master
fingerprint, already on the plate) depends on it staying true. There was
never a moment this test was expected to fail:

```
=== RUN   TestDeviceMkCardSitesSetFingerprint
--- PASS: TestDeviceMkCardSitesSetFingerprint (0.00s)
```

Because a permanently-green pin is only as good as its scanner, the
scanner (`mkCardLiteralsSetFingerprint`, a brace-counting scan for
`mk.Card{...}` literals) is separately proven against a synthetic
fixture missing the field:

```
=== RUN   TestMkCardLiteralScanCatchesAMissingField
--- PASS: TestMkCardLiteralScanCatchesAMissingField (0.00s)
```

(That test's own body asserts the scanner returns `[false]` for a literal
missing `Fingerprint:` — its own PASS *is* the mutation check, so no
separate before/after capture was needed.)

### GATE 2.4b / 2.4c — the engraved policy id equals the mk1's own stub, on both forms, from the post-template md1

New test `TestPolicyIDMatchesTheMK1StubOnBothForms`.

This test doesn't have a meaningful RED phase distinct from GATE 2.1's
(it depends on the same `md`/`mk` package imports already present in the
repo; nothing new needed to exist before it could compile once written
against the real `deriveSingleSigBundle`/`templateizeBundle`). Result:

```
=== RUN   TestPolicyIDMatchesTheMK1StubOnBothForms
--- PASS: TestPolicyIDMatchesTheMK1StubOnBothForms (0.03s)
```

It derives a REAL bundle via `deriveSingleSigBundle`, templateizes it via
`templateizeBundle`, and for BOTH forms computes
`md.FormAwareStubChunks(mdChunks)` (the exact expression
`singleSigPassphrasePlateOffer` uses) and compares it — as hex, a value
equality, not a label check — against `mk.Decode(mk1).Stubs[0]`, the
stub the real mk1 actually carries. A same-test check that the two
forms' ids actually DIFFER guards against the specific hazard spec 2.4
names ("a value captured near the pre-template point would be the
pre-template one") passing by coincidence.

### GATE 2.5 — the QR carries the passphrase and nothing else

The existing `TestPassphraseQRIgnoresFingerprints` already covered
fingerprints; extended with a new `TestPassphraseQRIgnoresPolicyID` for
the two S6b fields:

```
=== RUN   TestPassphraseQRIgnoresPolicyID
--- PASS: TestPassphraseQRIgnoresPolicyID (0.00s)
```

Module-level comparison (every QR module identical with and without
`PolicyID`/`Derived` set), the same reasoning the existing test uses: a
decoder that ignored trailing data would pass while the modules
differed, so this is the strongest available statement that the fields
change nothing about what gets engraved into the code.

### GATE 2.6 — the offer appears only when `passphrase != ""`; the ~31s derivation does NOT run when no plate is engraved

New test `TestPassphrasePlateOfferGate`, 3 subtests, using a new test seam
`singleSigBareSeedFPHook` (fired immediately before the lazy
`deriveAccountXpub(mnemonic, "", ...)` call) as the observable proxy for
"did the KDF run" — **on this test-runner that derivation completes in
milliseconds; the ~31s figure is the real SeedHammer II hardware's cost,
not something this suite can time**, so the hook counts invocations.

**RED** (`go vet ./gui/...` before `singleSigPassphrasePlateOffer`
existed): `undefined: singleSigBareSeedFPHook`.

**GREEN**, all 3:

```
=== RUN   TestPassphrasePlateOfferGate
=== RUN   TestPassphrasePlateOfferGate/no_passphrase:_no_offer_shown,_no_derivation
=== RUN   TestPassphrasePlateOfferGate/passphrase_entered,_offer_declined:_no_derivation
=== RUN   TestPassphrasePlateOfferGate/passphrase_entered,_offer_accepted:_derivation_runs_exactly_once
--- PASS: TestPassphrasePlateOfferGate (0.04s)
```

**A SECOND, HEAVIER TEST closes a gap the above cannot**: the three
subtests call `singleSigPassphrasePlateOffer` directly, which proves the
extracted function's own logic but not that `engraveSingleSigFlow`
actually reaches it with the right arguments. New
`TestPassphrasePlateOfferReachableFromTheOrchestrator` drives the REAL
`engraveSingleSigFlow` end to end — word entry, wallet type, a typed
passphrase, watch-only mode (2 real plates, to bound the cost; GATE 2.6
does not condition the offer on `full`), through ACTUALLY COMPLETED
engraves via the shared `engraveOnePlate` + `runUITouchRaster` helpers —
and confirms "Passphrase Plate" is the very next screen after declining
the verify offer:

```
=== RUN   TestPassphrasePlateOfferReachableFromTheOrchestrator
--- PASS: TestPassphrasePlateOfferReachableFromTheOrchestrator (25.40s)
```

**MUTATION-CHECKED**: commented out the
`singleSigPassphrasePlateOffer(...)` call site in `singlesig.go` and
reran — the flow fell straight through to the restore document:

```
--- FAIL: TestPassphrasePlateOfferReachableFromTheOrchestrator (25.91s)
    s6b_passphrase_plate_test.go:271: the passphrase-plate offer was not
    reached after skipping verify; got "RestoreDocTheseplateswerenot
    fullychecked...Masterfp:ca2c62d2..."
```

Confirmed RED, then reverted; reran to confirm GREEN again (25.4s stable
across three runs — this is the single most expensive test P3 added, and
it earned its cost: it is the only one that exercises the actual
insertion point rather than the factored-out function).

---

## A second truthfulness fix, beyond the named gates

`ppConfirmWarning`'s own doc comment says its fingerprint clause
"deliberately echoes the plate's own footer, `backup.passphraseFooter`
("FINGERPRINTS TYPED, NOT VERIFIED"), so the screen and the steel say the
same thing." Once the plate footer became provenance-selected (spec 2.3),
leaving the pre-engrave CONFIRM SCREEN unconditionally saying "Fingerprints
are typed, not verified" would have broken that documented invariant and
repeated, one screen earlier, the exact class of falsehood §2.3 exists to
fix on the plate. This was not named by any P3 gate — the spec's §2 text
never mentions the confirm screen by name — but R-D ("all things said must
be true") is stated as a blanket rule covering "cases nobody has
enumerated yet," and this is one.

Added `ppConfirmWarningDerived` ("Fingerprints are derived by this device.
A wrong passphrase does not fail: it opens a DIFFERENT wallet."), shown
when `derived == true`. Covered by
`TestPreloadedConfirmScreenNamesDerivedProvenance`:

```
=== RUN   TestPreloadedConfirmScreenNamesDerivedProvenance
--- PASS: TestPreloadedConfirmScreenNamesDerivedProvenance (0.01s)
```

and by extending `TestConfirmFitsPanel` to also measure the derived
worst case — the derived string is 3 characters longer than the typed
one, and a wrapped, width-bound label can turn extra characters into an
extra line:

```
=== RUN   TestConfirmFitsPanel
--- PASS: TestConfirmFitsPanel (0.00s)
```

---

## Findings

### 1. R-H's literal footer string does not fit the plate's own pre-existing band-width test — THE finding of this phase

R-H specified the derived-path footer as `"POLICY 1A2B 3C4D  DERIVED, NOT
TYPED"` (36 characters), citing `SPIKE_s6b_q2_results.md` §3b's claim of
**"a band line holds 42 characters."** Running the actual gate before
trusting that number surfaced a real conflict, not a stylistic one:

- SPIKE §3b's 42-character figure measures the point printed text runs
  off the **85mm plate's physical edge** entirely (measured at 537600
  device units, `12800 units/char × 42`).
- `backup/passphrase_test.go`'s **pre-existing** `TestPassphraseBandBudget`
  (present before this phase, untouched in its core assertion) pins a
  **different, tighter** cap: `"spec 4.3: no metadata line may exceed
  64mm"` (409600 units), sized — per that test's own comment — to clear
  the 10mm corner screw-hole bands by 0.5mm on each side. This is a **2-D
  geometric constraint** (does the centred line run into a screw hole at
  either end of the band), not "does it fall off the plate at all."

SPIKE §3b's own measurement table already contained the disproof: it
lists the EXISTING 32-character `"FINGERPRINTS TYPED, NOT VERIFIED"`
line at **exactly 409600 units** — i.e. already sitting AT the 64mm
ceiling with zero characters of spare — but the spike's narrative framed
42 as "the budget" without ever cross-checking that figure against this
file's own, already-passing test. Five review lenses on the spec/spike
inherited "42" as the applicable number without asking "42 of what
bound?".

Measured directly, by running `TestPassphraseBandBudget` with R-H's
string added to it (see GATE 2.3c above for the exact RED output): R-H's
36-character string is **460800 units — 8mm OVER the 64mm cap, into the
screw-hole zone.**

**What shipped instead**: `"POLICY " + <grouped hex> + "  DERIVED"` — 25
characters, 320000 units, 7 characters/44800 units of spare against the
409600-unit cap. It keeps R-H's chosen prefix format (the same
`POLICY`-plus-grouped-hex shape, the same double-space separator SPIKE's
own table uses for its other merged-line candidates) and drops only ",
NOT TYPED": "DERIVED" is already a true, positive claim on this path, and
the immediately-preceding acceptance screen already states the source in
those exact words ("Source: this session's own derivation", GATE 2.2).

**This is NOT operator-approved wording.** R-M's replacement arm in P1
went through an explicit `ADOPTED WORDING — operator, 2026-08-17` step
with a verbatim quote and sign-off. This string did not. It is measured,
internally consistent, and true — but it is a wording choice made under
implementation pressure to close a physical-safety gap the spec's own
citation missed, and it should be confirmed (or replaced) at this
phase's review gate before it is treated as settled the way R-M's arm
is.

### 2. R-G's "(4) will legitimately move" prediction does not hold — a BETTER outcome, not a discrepancy to chase

Spec §7 / REQUIREMENTS R-G point 3 predicted `backup/testdata/passphrase-*.bin
(4)` would move under §2.3's footer change. Under the implemented (and
corrected) mechanism, they do not: `Derived` defaults to `false` (the Go
zero value) for every one of the four EXISTING `TestPassphraseGolden`
cases (`0-plain`, `1-qr`, `2-no-metadata`, `3-max-qr`), none of which I
touched, so they continue to exercise exactly the standalone/typed path —
which R-H's own ruling separately and explicitly calls "unchanged."
Confirmed mechanically, not asserted: `git diff --stat -- backup/testdata/`
against tracked files is **empty**, and the full-suite run's exit code
and lack of `git status` churn after it confirm this held under the whole
suite too, not just a scoped run.

Instead, ONE new golden was added — `passphrase-4-preloaded.bin`
(`TestPassphraseGolden`'s new `"4-preloaded"` case: both fingerprints,
`PolicyID` set, `Derived: true`, QR) — which is R-G's OTHER, more general
rule ("marked states get NEW golden files") applying to a genuinely new
device state, exactly as it did for P2's marked mk1/md1 goldens. All
sixteen previously-frozen files, including P2's `text-{0,1,2}-shards-1.bin`,
are byte-identical.

### 3. Everything else in spec §2 matched the fork's current source as measured

No other discrepancy was found between spec §2's cited line
numbers/function names and the checked-out source at the start of this
phase, beyond the stale `singlesig_derive.go` doc comment spec §8 already
flagged for correction in this commit (done, see "What changed and
where").

---

## Full-suite gate

`export PATH="/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH"`,
then `go test ./... -count=1`, stdout and stderr captured to **separate
files**, run in the background with a blocking wait for its own
completion signal (not polled) — did not return before it finished.

**Exit code: 0. stderr: empty (0 bytes).**

**stdout**: 71 lines, every one `ok` or `? ... [no test files]`. Zero
lines matching `FAIL` or `panic` (checked with `grep -iE "FAIL|panic"`,
no hits). Relevant lines:

```
ok  	seedhammer.com/backup	3.287s
ok  	seedhammer.com/gui	440.587s
```

440.587s is inside Go's 600s per-package default (~73%), consistent with
the plan's stated 429–507s range.

`git status --porcelain` after the run showed a clean tree (the report you
are reading was the only thing left to write) — no incidental golden
churn anywhere in the repo, not just under `backup/testdata/`.

**Pre-existing, not-mine `go vet` failures** (go1.26 `t.ArtifactDir()` on
go1.25-tagged files, documented in the runbook and reconfirmed by P2):
`gui/freetext_sizeproof_golden_test.go`, `gui/op/draw_test.go`,
`backup/backup_test.go`, `backup/freetext_test.go`. Not touched.

---

## What I could not do / spec discrepancies found

1. **R-H's literal derived-footer string does not fit** — see Finding 1.
   Shipped a measured, internally-consistent 25-character replacement
   that is NOT yet operator-approved wording. This is the item most in
   need of attention at this phase's gate.
2. **R-G's "(4) passphrase goldens will move" did not materialize** — see
   Finding 2. A better outcome than predicted (fewer goldens disturbed),
   not a defect, but worth recording since the spec's own text asserts
   the opposite.
3. **The confirm screen's fingerprint-provenance claim** (`ppConfirmWarning`)
   was not named by any P3 gate but was fixed under R-D for the reason
   given above — flagging it explicitly since it is scope the spec text
   did not enumerate, even though it is required by a rule the spec text
   does state.

Nothing else in spec §2 was ambiguous or contradicted by the checked-out
source. The `me` CLI does not exist in this repository (it lives in the
sibling `mnemonic-engrave` Rust repo) and was not touched by construction;
no `md1`/`mk1`/`ms1` wire-format encode/decode path was touched — confirmed
by `git diff --stat`, which shows only `backup/passphrase*.go` and
`gui/*.go`/`gui/*_test.go` files changed across both commits.

---

## Commits

```
fc2efa2 S6b P3 (2.1/2.2/2.4a/2.4b/2.4c/2.6): the preloaded passphrase-plate program
381c95e S6b P3 (2.1.1/2.3/2.3b/2.3c/2.3d/2.5): backup.Passphrase carries a policy id, selected by provenance not presence
```

Both carry their gate output in the commit message per this repo's
build-gate convention. Working tree is clean on `s6b-pre-flash`.
