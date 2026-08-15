# S2 fold re-review — tight, scoped to "did the fold fix each finding, and did it introduce a new defect" (2026-08-15)

Reviewer: same independent context that produced
`s2-execution-review-2026-08-15.md` (1C / 2I, persisted `d23e587`).
Under review: `seedhammer` `4b8488e` + `mnemonic-engrave` `6c41045`, on top of
the five S2 commits. Implementer's report: `s2-fold-2026-08-15.md` (`54859b4`).

**Not a fresh audit.** Out of scope and untouched: the duplicate-key deliverable,
the check-order ruling (sound, duplicate-first), S0b, S1, every §0 ruling.

---

## VERDICT

**0 Critical / 0 Important — the loop CLOSES.**

All three findings are fixed in code, and each fix is proved able to fail by a
mutation I applied myself. One **Minor** defect was introduced by the fold, in a
record rather than in code: F-179's site list was not propagated and now carries
three mutually inconsistent counts. It gates nothing and should be corrected in
the closing commit rather than in another round.

---

## Per-finding

### C1 (Critical) — the emulator-walk gate — **FIXED**

The walk is now **asserting**, not merely reaching. Checked line by line:

- **Both outcomes are first-class.** `raceFor(["Duplicate key", "Policy stub"])`
  (`walk_build_policy.js:99-118`, used at `:475`) forks on whichever appears and
  **throws** if it is not the one the run declared, so a hang can no longer be
  read as "the other arm happened". This was the specific failure mode that made
  the old walk's silence look like success.
- **`ok` is outcome-bound** (`:522-531`): a duplicate run is green only with
  `refusal !== null`; an engrave run only with
  `census.strings.length === plates && census.unattributed === 0`.
  `unattributed` is the emulator's own attribution of cut plates against
  announced strings (`cmd/emu/engraved.go:131,140`) — **not** driver-supplied.
- **Does a green run depend on anything the driver supplies?** `plates = 9` is a
  driver default, documented as DERIVED (1 ms1 + 2 mk1 + 6 md1) per F-170. It
  cannot manufacture a pass: a short run never reaches the count, hits an
  unrecognised screen, and `ENGRAVE_HANDLERS` **stops** rather than tapping past
  (`:145-149`) — `acts` records `STALLED` and `ok` is false. The `decisions`
  array is bookkeeping only. A missed SKIP tap does not pass silently either: it
  takes A@0, the assembler refuses as a duplicate, `raceFor` returns
  "Duplicate key", and `expect === "engrave"` **throws**.
- **The picker loop is screen-driven** (`:406-412`), breaking when the
  remaining-equals-needed short-circuit takes over — the tap-counting bug the
  review would otherwise have inherited.

**Stub identity — my answer to the probe.** It is a **real agreement between two
independently built surfaces, but it is recorded, not asserted.** The chain is
honest: the wasm build's on-screen stub `06215ac0` == the host build's stub ==
bytes byte-identical to the Rust primary (`TestAssembledMd1MatchesThePrimaryByteForByte`,
which I re-ran at review time and which logs `06215ac0`). The *cross-implementation*
leg is the Rust comparison and it is machine-asserted; the emulator↔oracle leg is
the **same Go algorithm compiled twice**, so it proves "the same policy was
built", not "the bytes are right". That is exactly what the fold claims, so the
claim is accurate. The residual: **nothing in the walk pins the value** — a
future divergence would not turn it red. The fold did close half of this
independently: `06215ac0` is now pinned in Go on the payload-sourced review
(`gui/multisig_build_payloadseed_test.go:131`). A one-line `expectStub` param
would finish it. **Nit, not blocking.**

**Cross-check I could run in place of the browser.** Every screen the two new
legs depend on exists in production `gui` source (squashed match, script over
non-test files): `Where from?`, `FROM PAYLOAD`, `systemwide payload`,
`Add a BIP-39 passphrase?`, `Duplicate key`, `Policy stub`, `Which md1?`,
`EXPERIMENTAL`, `What to engrave?`, `Chooseengraving`,
`Holdbuttontostarttheengravingprocess`, `Engravingcompletedsuccessfully`,
`Input Seed` — **13/13 present**. `rowY` is defined (`:54`) and used
consistently. The file parses under `node`. A walk citing a screen that does not
exist could not have run as reported; nothing here contradicts the record.

**F-181 — withdrawn correctly.** Struck through in the heading, premise named
false, and the false sentence *"S2's own gate is satisfied by the Go walk plus
the payload-leg emulator walk that RAN green"* is **quoted in place and then
struck**, not deleted — the F-176 pattern. The keyboard driver is re-scoped as
optional with its measurements preserved.

### The knock-on (self-seed-from-payload) — **FIXED**

`gui/multisig_build_payloadseed_test.go`. Both probes pass:

- **Does it fail if the route breaks?** Yes. `pumpUntil(frame, "Where from?")`
  is a `t.Fatalf` — *"the seed SOURCE picker never appeared, so the payload arm
  is still unreachable"*. If the payload arm regressed to a one-row picker the
  test dies there.
- **Is the picker genuinely exercised rather than defaulted through?** Yes, three
  ways: the screen must be drawn, it must contain `"FROM PAYLOAD"`, and it must
  raster above the floor (`:88-99`); then the **payload-only** acceptance surface
  `"systemwide payload"` must appear (`:104`) — a typed fall-through cannot reach
  it and would hang at the word-count picker. Neither test types a word.
- The review frame additionally pins stub `06215ac0` and §0.1a's origin.

### I1 (Important) — invisible refusals — **FIXED, and wider than I found**

Six production strings, not three: my `:184/:200/:202` plus the three
`feedback()` bodies at `:62/:65/:67` that are appended to the **gather's own
body** — one of which blanks the card tally rather than a modal. I re-enumerated
`gui/*.go` non-test literals with the same script that reproduced 27 pre-fold:
**all six are clean**, and `bundle_flow.go` now has only `:430,:438` left.

Reproduced the fix independently: `TestGatherPendingRefusalIsReadableFromBuild`
logs **9855 px** from the real Build flow (was 2652 = title-only). Matches the
claim.

**Guard mutation — I applied both halves in one run:**

    G1  em-dash into a PRODUCTION literal  ("Card added — nice." in feedback())
    G2  em-dash into a COMMENT inside bundleGatherFlow

    --- FAIL: TestGatherScreenTextCarriesNoBlankingGlyph
        func (s *bundleGatherScreen) feedback( draws "Card added — nice.",
        which carries "—" …
    --- PASS: TestStringLiteralScannerCanSee

**Exactly one failure, naming the literal; the comment was ignored.** The guard
catches a new production em-dash and stays quiet on prose — both probes pass.
`stringLiterals`' own non-vacuity (`len(lits) == 0` → `t.Fatalf`) and the
scanner-can-see mutation proof are present.

**The two secret-exposure warnings are honestly recorded as S3's,** not counted
as done: `sysw_load.go:274,275,279,280` and `sysw_source.go:114` are still in
F-179's remaining list, still named in its prose as *"where a blank body is the
worst possible outcome"*, and the fold report's own "what I did NOT do" names
them as *"real and they are not S2's"*. Correct.

### I2 (Important) — the phantom §0.1a check — **FIXED**

`gui/multisig_build_walk_test.go:216-219` now captures the `"Policy stub"` frame
into `reviewFrame`, and `:238-247` asserts `multisigSharedOrigin().String()` and
`"BIP-48"` on it. The old comment is replaced by one that states there was no
check and why a string-level test cannot substitute.

**M9 re-applied by me** (deleted `buildOriginAnnouncement` from
`buildReviewLines`; compiles):

    --- FAIL: TestBuildWalkTypedSeed
      the Policy Review reached the display without §0.1a's origin announcement:
      no "m/48h/0h/0h/2h" in "PolicyReviewSlots@1and@2filledfromthepayload(cards2
      and3of4,inpayloadorder).Policystub:06215ac0Slots:@0(nofp)@1(nofp)@2(nofp)
      FingerprintsOMITTEDoneveryslot."

The quoted frame is the genuine rendered review with the origin line removed and
everything else intact — the check reads the display, not a string. Confirmed.

### Minors from the first review

| # | verdict |
| --- | --- |
| M1 | **FIXED** — both flagship refusals rastered; figures match mine (18139 / ~20.3k). |
| M2 | **FIXED, and proved.** Pool of 8 distinct accounts of a second master + a non-vacuity assertion. I ran the fuzz: **2,929,137 execs / 20s / 0 failures** (claim was 2,906,990 — same order, different run). **M10 re-applied by me**: restoring `Xpub: selfXpub` turns the **seed corpus** red — *"n=2 with 1 distinct pool card(s) was refused as a duplicate … this target is fuzzing nothing"*. The assertion can fire. |
| M3 | **FIXED** — plan row corrected in place, old text struck not deleted, with the measurement. |
| M4 | **FIXED** — F-182 named in the walk that crosses it (`:313-317`). |
| N1 | **FIXED, and the claim is structural.** `titleOnlyInk` now searches 1..3 and returns the max. Verified the ceiling myself: `gui/gui.go:2118` `ys := [3]int{` indexed by `clk.Button - Button1` — a fourth is not expressible. |
| N2 | **FIXED** — `assembleBuildPolicy`'s scope comment now bounds "SOLE md1 producer" to md1 the device *mints*, and names `supplyMultisigPolicyFlow` as out of scope by §4.1. |

---

## NEW DEFECT introduced by the fold

### Minor — F-179's site list was not propagated; three counts now disagree

`design/FOLLOWUPS.md`, F-179. The heading and prose were rewritten ("6 of the
sites FIXED at S2"), but **the enumerated list underneath — the one the entry
tells the reader to work from — is byte-unchanged and still names all six fixed
sites as outstanding**, still totalling "27 live strings":

    gui/bundle_flow.go:62,65,67,184,200,202,430,438     <- 62..202 are FIXED

Three numbers are in circulation and none is right:

| source | says | |
| --- | --- | --- |
| F-179's list | **27** remaining | stale — pre-fold |
| fold report (`:104`, `:185`) | **24** remaining | = 27 − 3; forgot the three `feedback()` sites it fixed |
| my re-run of the same enumeration | **21** remaining | 27 − 6 |

The prose is also self-inconsistent: *"all four gather strings in
`gui/bundle_flow.go` — the two 'Done' refusals plus the three `feedback()`
messages"* enumerates 3 + 3 = **6** strings, calls them "four", and totals the
whole fix at "6 sites" when it is **8** (the 6 above plus S2's EXPERIMENTAL body
and the fp line).

**Why Minor and not blocking:** the code is correct and machine-guarded; no S2
gate depends on this; and what will actually drive S3 is
`TestGatherScreenTextCarriesNoBlankingGlyph` widened package-wide, not the list.
**Why it must still be fixed in the closing commit:** S3 is the owning phase and
would spend its first hour re-fixing six already-fixed strings. This is the
"folds fail by incomplete propagation" pattern exactly — the facts were updated
and the duplicates were left.

**Remedy, one paste:** re-run the enumeration, replace the list and both totals
with **21**, correct "6 sites" → "8 sites" and "four gather strings" → "six",
and commit the enumerator as a script so the number is a command rather than a
hand-count.

---

## WHAT I RAN

    nix develop --command go test ./...      51 ok, 0 FAIL/panic (grepped), GOTEST_EXIT=0
    nix develop --command go vet ./...       VET_EXIT=1, 6 findings, all ArtifactDir/go1.26 baseline
    nix develop --command gofmt -l ./        no output, FMT_EXIT=0
    nix develop --command tinygo build -size short … ./cmd/controller
                                             flash 1354552  ram 61908  TINYGO_EXIT=0

**Flash 1,354,552 — the claimed −16, reproduced exactly.**

Mutations, each applied to the tree, compiled, run, then `git checkout`-ed
(tree verified clean at `4b8488e` afterwards):

| # | mutation | result |
| --- | --- | --- |
| G1 | em-dash into a `feedback()` production literal | guard **red**, naming the literal |
| G2 | em-dash into a comment inside `bundleGatherFlow` | guard **silent** — no false positive |
| M9 | drop `buildOriginAnnouncement` from `buildReviewLines` | `TestBuildWalkTypedSeed` **red**, quoting the real rendered frame |
| M10 | restore `Xpub: selfXpub` in the fuzz pool | **seed corpus red** — the non-vacuity assertion fires |

Plus: `FuzzAssembleBuildPolicy -fuzztime 20s` (2,929,137 execs, 0 failures);
re-enumeration of `gui/*.go` em-dash literals (21 remaining, all six fixes
confirmed); needle-existence cross-check of the walk's 13 new screen strings
against production source (13/13); `node` parse of `walk_build_policy.js`;
`gui/gui.go:2118` read for N1's structural ceiling.

**Process note, worth recording:** my first read of the gate output was against
**stale files** left in the scratch directory by an earlier run — they showed
25 `ok` lines and flash 1,349,428 and would have produced a wrong report. Caught
by checking file mtimes rather than trusting content. "Empty output is not
absence" has a sibling: *content is not freshness*.

---

## WHAT I COULD NOT CHECK

- **The browser walk itself.** I did not rebuild `emu.wasm`, serve it, or drive
  either arm. C1's fix is judged on the driver's code (which asserts, and whose
  `ok` is outcome-bound), on 13/13 needle existence in production source, and on
  the two Go tests that mirror both arms and which I ran. The reported run
  figures — `elapsed 303s`, `digests 9`, `acts 27`, `decisions [...]` — are the
  implementer's record and I did not reproduce them.
- **The walk does not assert the stub value**, so the emulator↔oracle policy
  identity remains recorded rather than machine-checked (nit above).
- **The other 21 em-dash sites** were enumerated, not individually rastered.
  Owned by S3.
- **Hardware.** Nothing ran on a physical SH2; D-1 remains S6's.
- **Non-`gui` em-dash literals** (19 in `cmd/`, `seal/`, `oracle/`) remain
  untraced to a display surface, as both prior reports flagged.
- I did not re-audit S0b, S1, the duplicate-key deliverable or the check-order
  ruling, per the brief.
