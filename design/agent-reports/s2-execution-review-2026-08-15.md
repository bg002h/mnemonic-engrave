# S2 execution review — independent, adversarial, post-implementation (2026-08-15)

Reviewer: separate context from the implementer; did not see its conversation.
Scope: `seedhammer` commits `dcd90a5`, `101c8eb`, `f712a81`, `189b173`, `3ea3ede`
on `main`, against `design/IMPLEMENTATION_PLAN_multisig_build_repair.md` §0,
§0.1a, §0.1b and §3's S2 section (rewritten 2026-08-15, authoritative), plus
`design/agent-reports/fable-s2-inheritance-rulings-2026-08-15.md`.

Settled and NOT re-derived, per brief: S0 / S0b / S1; the `0..n`, F-175, §0.1a,
§0.1b, cc‖pk-basis and duplicate-check-first rulings; F-178's screens as
defect-screens; F-176 withdrawn.

---

## VERDICT

**1 Critical / 2 Important — BLOCKING.**

The funds-safety deliverable itself is sound: the duplicate check is correctly
placed, correctly based, unconditional on every route, and its refusal draws.
What fails is **coverage of the stage's own gates** — one gate half was never
executed at all, and the stage's headline discovery (a glyph blanks a whole
body, invisibly to every content assertion) was not applied to three operator
screens the stage itself edited.

---

## CRITICAL

### C1 — S2's gate half "by emulator walk" has never been run, and neither arm the plan names is driven

**Requirement, verbatim.** Plan line 952: *"**Gate.** Trace A completes end to
end: engrave, **by test and by emulator walk**."* Plan lines 916–918 (test 4,
authoritative): *"The collision it creates is the refusal's standing walk
fixture: **S2's walk drives BOTH arms** — default taps + payload seed → the
Duplicate key screen; SKIP, SKIP → B@0+C@0 → completed engrave."* The fable
ruling's own WHAT-I-COULD-NOT-CHECK (lines 257–260) explicitly hands this gate
the open question: *"Whether the engrave actually completes past the style
picker in the emulator … **S2's gate owns it.**"*

**Measured state.** `cmd/emu/walk_build_policy.js` was edited by S2 (`101c8eb`)
but only to add the `NEEDLE_GATHER` assertion. It still terminates at
`cmd/emu/walk_build_policy.js:285-288`:

    // S1 ENDS AT A SCREEN, NOT AN ENGRAVE (plan §3 preamble, F-175). …
    const screen = await waitFor("Input Seed");

and its success predicate (`:311`) is
`proven.length === 7 && presentedAtEnd === 0 && cardsGathered > 0 && selected` —
no engrave, no plate census, no refusal arm. `cmd/emu/walk_trace_a.js` was not
touched by S2 (last commit `88d43c7`, S0) and is a **bundle-engrave** walk, not
Trace A (plan lines 579–584, F-168). No new walk file was added: the whole
`cmd/emu` diff for S2 is `needle_test.go` + `walk_build_policy.js`.

**The stated blocker does not block this.** F-181 (`FOLLOWUPS.md:6280-6282`)
justifies stopping with *"S2's own gate is satisfied by the Go walk plus the
payload-leg emulator walk that RAN green."* That is false in both halves:

- the payload-leg walk stops at "Input Seed" **by design** (above), so it never
  reached an engrave and cannot satisfy a completed-engrave gate;
- the keyboard driver F-181 is about is **not needed** for either arm. The
  emulator payload carries a `ClassMnemonic` (master A) as record `@9`
  (`cmd/emu/sysw_cards_payload.go:28`), so the self seed arrives **from the
  payload** with confirm-taps only. The plan's own wording is *"default taps +
  **payload seed**"*.

**Reproduction (inputs → wrong behaviour).** Serve `cmd/emu` and run
`w.run()`: it returns `ok: true` after `waitFor("Input Seed")`. The two arms the
plan requires are one screen further on and are unscripted:

| arm | taps from "Input Seed" | expected |
| --- | --- | --- |
| refusal | seed **from payload** (master A); cards 1,2 taken by default (A@0, A@1) | the **Duplicate key** screen |
| clean | seed from payload; **SKIP, SKIP** on cards 1,2 → short-circuit takes B@0 + C@0 | engrave, 9 plates |

**Second, undriven consequence.** Every Go test on this path seeds the session
with `cosignerCardRecords(...)` — mk1 chunks only, **no `ClassMnemonic`** — so
`syswSeedPicker` is "a menu of one and is skipped"
(`gui/multisig_build_dupkey_test.go:254`). Verified by grep over all seven test
files that call `buildMultisigPolicyFlow`. Therefore **the self-seed-from-payload
route on the Build path has never been rendered or driven by any test or any
walk**, and it is one of §0.1b's two ruled primary data entries. The emulator
walk stops on precisely the screen where that choice is made.

**Why Critical rather than Important.** Project law (`CLAUDE.md`, "closure is
LENS-closure"): *"a plan may not close while any of its own gates has never been
run … A gate that has never executed is a hypothesis, not a gate."* This is that
case verbatim, on the stage whose gate the fable ruling deferred the open
question to.

**Remedy (small).** Extend `walk_build_policy.js` with an `engrave: true` /
`seedFrom: "payload"` leg reusing `walk_trace_a.js`'s existing hold + toolpath-
stall machinery, and a `duplicate: true` leg that asserts the Duplicate key
screen. Both are tap-only. Then run it and record the census.

---

## IMPORTANT

### I1 — D-4 gave three gather refusals the right title and left their bodies invisible; one is reachable from Build and I drove it

`gui/bundle_flow.go:184`, `:200`, `:202` are the three `bundleGatherFlow` "Done"
refusal bodies. All three carry an em-dash. Commit `101c8eb` edited the
`showError` calls that draw them (`:189`, `:205`) to thread the caller's title
through, and its own comment at `:195-198` states *"REACHABLE FROM BUILD (fold,
I2) … this message is what the operator reads."* The bodies were not fixed.

**Measured by me, through `runUITouchRaster` + `showError` at `sh2DisplaySize`
(scratch test, run then deleted):**

| body | em-dash | hyphen |
| --- | --- | --- |
| `bundle_flow.go:184` | **2652** | 6710 |
| `bundle_flow.go:200` | **2652** | 7184 |
| `bundle_flow.go:202` | **2652** | 9857 |
| `sysw_load.go:274` ("A SECRET … unencrypted in flash") | **2652** | 8441 |
| `sysw_source.go:114` ("NO integrity check at all") | **2652** | 8440 |

2652 is the title-only value for a two-button modal — **identical for every
body regardless of length.** The em-dash does not blank *its line*; it blanks
**the whole body**. That is stronger than the implementer's characterisation and
stronger than F-78's.

**Driven end to end through the production flow** (scratch test, run then
deleted): payload = B@0 (2 chunks) + C@0 (2 chunks) + **first chunk only** of
A@0; Build policy, all five picker defaults with n=3; tap Done at the gather:

    GATHER          ink=7897  "CosignerKeysmd1descriptors:0mk1keys:2Done…"
    PENDING REFUSAL reached from Build.  ink=2652   <- title only; body gone

The flow then returns the two surviving cards (`bundle_flow.go:206-208`) and
proceeds to build, so **the operator is never told a card was dropped** — they
see a screen titled "Cosigner Keys" with nothing on it and continue.

**Why S2's own tests cannot see this.** I measured `uiContains(content, …)`
returning **true** on the blank frame for every case above: the text ops still
report the string. Every content-based assertion in the repo — including
`gui/multisig_build_title_test.go:33-42`, the D-4 guard — is blind to this class.
The raster floor S2 built is the only instrument that sees it, and it was
applied only to the happy walk, which never visits these screens.

**Judgment on probe 3 ("is the fixed set the right set?").** No. The
implementer's own criterion was "sites on S2's own walk"; these three are on
S2's own flow, in the function S2 edited, in the same commit. F-179's deferral
to S3 is right for the other 24 sites and wrong for these three. My independent
enumeration of `gui/*.go` non-test string literals reproduces F-179's count
exactly: **27 live sites** (F-179's "31 minus 4 trailing `// F-78:` comments").
Two of the remaining 24 (`sysw_load.go:274-275,279-280`,
`sysw_source.go:114`) are secret-exposure warnings that also raster at 2652 —
they are on the Load Payload leg S2's own emulator walk drives, and they belong
to S3 as filed.

### I2 — the walk test's §0.1a check does not exist; a comment says it does

`gui/multisig_build_walk_test.go:209-214`:

    // THE POLICY REVIEW MUST HAVE SPOKEN. Re-read from the frame the walk
    // actually saw rather than from buildReviewLines: §0.1a's announcement is
    // only worth anything if it reaches the display.
    //
    // (Captured above by pumpUntil's return; re-asserted here against a fresh
    // read would race the flow, so the check is on the recorded content.)

**There is no check.** The `content` returned at `:196` is scoped to the `rest`
loop and discarded; nothing follows the comment block. Confirmed by
`grep -rn "Key origins"` over the whole repo: three hits, all in
`gui/multisig_build.go:872,878,885`. **No test asserts §0.1a's announcement on a
rendered surface** — `TestBuildReviewAnnouncesTheBip48Origin` inspects
`buildReviewLines`' strings only, which is exactly the assertion class this
stage proved cannot see a blank line.

**Behaviour is correct** — I proved it incidentally: under MUT-A the flow test's
last-screen dump reads
`"PolicyReviewSlots@1filledfromthepayload(card1of1,inpayloadorder).Keyorigins:m/48h/0h/0h/2h,theBIP-48pathfornativesegwit."`,
so the announcement does reach the frame. This is a **missing guard plus a
comment that lies about it**, on a ruled deliverable of this stage. Fix is ~3
lines: keep the `"Policy stub"` frame's content and assert
`multisigSharedOrigin().String()` and `"BIP-48"` on it.

---

## MINOR / NITS

- **M1 — S2's flagship refusal screens are never rastered.**
  `TestBuildFlowRefusesDuplicateBeforeReview` (`gui/multisig_build_dupkey_test.go:226`)
  and `TestBuildFlowRefusesForeignOriginCard` (`gui/multisig_build_origin_test.go:140`)
  use `runUI`, not `runUITouchRaster`, so no ink is measured on the two modals
  the stage exists to produce. I measured them: **Duplicate key 18139 px, Key
  origin mismatch 20325 px** — both healthy today, so no defect, but unguarded
  against the exact class I1 demonstrates (and against "a body too long to lay
  out", which the walk test's own error message names as a hazard for bodies
  this long: 335 and 362 chars).
- **M2 — `FuzzAssembleBuildPolicy`'s coverage collapsed silently.**
  `gui/multisig_build_test.go:474` `otherXpub := selfXpub`, so every generated
  card is the self key. Post-S2, every case with `n ≥ 2` returns at the
  duplicate check and `md.EncodeMultisig` is no longer reached through this
  target. The target asserts only "does not panic", so it stays green while
  fuzzing almost nothing. One-line fix (derive a second xpub). Self-reported by
  the implementer as item 5; confirmed.
- **M3 — the plan's S2 file table is decayed.** It claims *"two assertions at
  `multisig_build_flow_test.go:239,249` wait on the literal `"Engrave Bundle"`
  and break the moment D-4 lands"*. Measured: those lines now read
  `"me sysw pack"` (changed at S1). The one remaining `"Engrave Bundle"`
  assertion in that file is `:29`, on the **Supply** path, correctly unchanged.
  No code action; the plan row misleads the next reader.
- **M4 — F-182 is on S2's own completed walk.** `TestBuildWalkTypedSeed` drives
  the engrave tail straight past `bundleEngrave`'s hard-coded "Engrave Bundle"
  reminder without asserting it. Deferring to S5 (which owns the tail) is
  defensible; walking past a known D-4-class screen without a note in the test
  is not.
- **N1 — `titleOnlyInk`'s worst-case claim is unmeasured.**
  `gui/multisig_build_walk_test.go:72-75` asserts "three nav buttons, the most
  any screen on this walk draws" in prose. If a screen ever drew four, the blank
  baseline (5482) would be understated and the 518 px margin would shrink
  silently.
- **N2 — scope note, not a defect.** The duplicate check covers the **assembled**
  set only; `supplyMultisigPolicyFlow` engraves an operator-supplied md1 with
  repeated keys unchecked. Correct per §4.1's scoping (the device is not the
  author there), but "SOLE md1 producer" reads like "every md1 on the device is
  checked" and is not.

---

## RULING on the check-order judgment call

**SOUND. Keep duplicate-before-foreign-origin. No change.**

1. **§0.1 clause 2 is applied correctly.** With fp-presence Omit (the default,
   `multisigFpChoices` index 0) a repeated key renders every slot `(no fp)` and
   is invisible in every artifact the operator keeps; a foreign origin
   mis-states a path that is printed on the plate, in every mk1, and on the
   restore doc, so it is detectable by reading the output. Invisible outranks
   printed. That is the refuse-side rule verbatim.
2. **Lifetime settles it independently.** `errBuildForeignOrigin` is deleted at
   S5; §4.1's check is permanent. Duplicate-first makes S5's deletion a pure
   removal. Origin-first would mean that at S5, inputs that had reported "Key
   origin mismatch" silently start reporting "Duplicate key" — a behaviour
   change at a stage boundary that no test would pin.
3. **The distinguishing input exists and is asserted.** Self = master A, cards
   A@0 (byte-equal to the self key) + A@1 (declared `m/48h/0h/1h/2h`) fires both
   checks on one build; `gui/multisig_build_origin_test.go:102-112` pins the
   winner.
4. **Mutation-proved by me, not accepted.** I moved the origin loop above the
   duplicate check in `assembleBuildPolicy` (compiled clean, `gofmt` clean) and
   exactly one subtest went red:

       --- FAIL: TestBuildRefusesForeignOriginCardBeforeS5/a_duplicate_outranks_a_foreign_origin
           a build that is BOTH a duplicate and a foreign origin reported
           multisig build: slot @2 declares origin "m/48h/0h/1h/2h", not the
           shared origin; the duplicate must win

   The order is asserted, not incidental.
5. **The fable property holds at the assembler**, where it is testable: default
   taps + payload seed reach `errBuildDuplicateKey`. It is **not** preserved as a
   driven walk — that is C1, not an order problem.

---

## Probe-by-probe verdicts

| # | probe | verdict |
| --- | --- | --- |
| 1 | duplicate check reachable from every route | **PASS.** `md.EncodeMultisig` has exactly one non-test call site, `gui/multisig_build.go:799`, inside `assembleBuildPolicy`; `assembleBuildPolicy` has exactly one non-test caller, `:175`. The check at `:763` is unconditional — no early return, no branch, no flag precedes it. I found no path that assembles without it. A@0/A@1 sharing a master fingerprint is **asserted in the test** (`multisig_build_dupkey_test.go:150-155`), so the ruled cc‖pk basis is measured, not argued. |
| 2 | check order | **PASS + RULED SOUND** (above). |
| 3 | em-dash: right set fixed? | **FAIL → I1.** 2 fixed, 27 live; 3 of the 27 are inside the function S2 edited and one is reachable-and-driven from Build. The "blanks its line" characterisation understates it: it blanks the whole body, at a constant 2652 px. |
| 4 | can the raster floor FAIL? | **PASS.** Independently measured: worst blank 5482 < floor 6000 < thinnest real 6566. `TestBuildWalkRasterFloorIsCalibrated` fails on both sides. I re-introduced the em-dash into the EXPERIMENTAL body: `INK "EXPERIMENTAL" 4973`, floor red — the number the report cites, reproduced. |
| 5 | md1 byte-identity oracle | **PASS.** Resolves by **`binary-sha256`** to commit `5a0a4f41017d71d47f70684c145702d4ca0c3aa9` (reports `md 0.13.0`, matches pin `true`); 6 chunks byte-identical, stub `06215ac0`. Invoked at the **absolute** path `~/.cargo/bin/md` via `exec.Command` (`multisig_build_oracle_test.go:44,120`), never through a shell — the `md`→`mkdir -p` alias cannot reach it. Note the test `t.Skipf`s if the binary is absent; it did **not** skip here. |
| 6 | D-4 needles single-site | **PASS.** `TestBuildFlowNeedlesHaveExactlyOneProductionSite` asserts `len(sites) != 1` → fail **and** that the site is the expected file; 7 needles in `buildFlowNeedles`, 7 `NEEDLE_*` consts in the walk, `ok:` pins `proven.length === 7`. Ran green. Four callers pass `"Engrave Bundle"` byte-unchanged (diff-verified). |
| 7 | mutations | **PASS.** Three re-applied by me, all compiling — see WHAT I RAN. |
| 8 | F-181 stop | **Right to stop the typed leg; wrong to close the gate on it.** No half-built driver landed: `grep -rn "typeWord\|shKey\|keyRect\|shType" cmd/emu/` returns nothing, tree clean at `3ea3ede`. But the gate it was blocking does not need it — see C1. |

---

## WHAT I RAN

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"

    nix develop --command go test ./...
      GOTEST_EXIT=0    51 "ok" lines    0 FAIL/panic lines (grepped, not summed)
    nix develop --command go vet ./...
      VET_EXIT=1, 6 findings, all `testing.ArtifactDir requires go1.26` — the baseline
    nix develop --command gofmt -l ./
      FMT_EXIT=0, no output
    nix develop --command tinygo build -size short -o /dev/null -target pico-plus2 \
      -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
      TINYGO_EXIT=0    flash 1354568    ram 61908      (matches the report exactly)

    go test ./gui/ -run 'TestAssembledMd1MatchesThePrimaryByteForByte|TestBuildWalkTypedSeed|TestBuildWalkRasterFloorIsCalibrated' -v -count=1
      all PASS; every INK figure in the report reproduced (5482 / 6566 / 18563 / 9 plates)
    go test ./cmd/emu/ -run Needle -v -count=1        all PASS

Mutations, each applied to the tree, compiled, run, then `git checkout`-ed:

| # | mutation | result |
| --- | --- | --- |
| MUT-A | `duplicateSlotPair(all); dup && false` | compiles; **4 failures**, incl. flow tests whose last screen is the Policy Review carrying the real §0.1a announcement — the duplicate policy reaching review, verbatim |
| MUT-B | move the origin loop **above** the duplicate check | compiles, `gofmt` clean; **1 failure**, `a_duplicate_outranks_a_foreign_origin`, naming the foreign-origin error only reachable if the origin check ran first |
| MUT-C | restore the em-dash to the EXPERIMENTAL body | compiles; `INK "EXPERIMENTAL" 4973` < floor 6000 → `TestBuildWalkTypedSeed` red. The raster floor CAN fail. |

Scratch measurements (written to `gui/`, run, deleted; tree verified clean):

- em-dash vs hyphen ink for 5 live strings through `showError` (table in I1);
- ink of `buildDuplicateKeyMessage` / `buildForeignOriginMessage` /
  `buildSupplyRefusal` (M1);
- the pending-refusal drive from the Build path (I1's reproduction).

Working tree at `3ea3ede`, `git status --porcelain` empty at the end.

---

## WHAT I COULD NOT CHECK

Named explicitly, because an unchecked area named is worth more than a clean
bill implying coverage I did not have.

- **I did not run the emulator in a browser.** C1 rests on reading
  `walk_build_policy.js`'s terminal `waitFor("Input Seed")`, its `ok:`
  predicate, the absence of any engrave leg in the S2 diff, and the payload's
  record inventory. I did **not** confirm by driving that the engrave completes
  in the emulator — that is precisely the open question, and it stays open.
- **Per-commit flash deltas.** I measured only the final 1,354,568. I did not
  rebuild `dcd90a5`/`101c8eb`/`f712a81`/`189b173` to check 1352184 / 1352244 /
  1353816 / 1354568.
- **22 of the 27 live em-dash sites** were enumerated by script and not
  individually rastered; I measured 5 (all 2652). I did not check whether any
  of them sits in a multi-line body where only one line would be lost rather
  than the whole body.
- **Hardware.** Nothing ran on a physical SH2. D-1 remains S6's, untouched by
  this review.
- **`pins.json`'s own provenance.** I confirmed the oracle resolves by
  `binary-sha256` to a commit matching the pin; I did not verify that the pin
  itself corresponds to the intended upstream `md` release.
- **S0 / S0b / S1** were not re-audited, per the brief. S0b has its own queued
  review.
- **Non-`gui` em-dash sites** (19 string literals in `cmd/`, `seal/`, `oracle/`)
  were enumerated but not traced to a display surface; I believe none reaches
  the GUI body face, but I did not prove it.
- **`syswSeedPicker`'s payload arm on the Build path** — I established that no
  test or walk drives it; I did not myself drive it to check it works. That is
  the same gap C1 names, and it should be closed by the same walk.
