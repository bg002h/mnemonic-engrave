# Review — lens: does this plan walk the user's journeys in the emulator, using payloads?

Artifact: `design/IMPLEMENTATION_PLAN_multisig_build_repair.md`
Fork source: `/scratch/code/shibboleth/seedhammer` @ `a10d007` (clean)
Date: 2026-08-13

**Verdict: 3 Critical, 3 Important, 2 Minor. The §4.5 emulator walk is invoked
in five stage gates and is executable in none of them.**

---

## The plain answer: would these walks have caught the blank screen?

**No.** Not one of them would have run far enough to see it.

The blank screen (F-150 item 1 / D-1) lives *past* the cosigner gather. To reach
it in `cmd/emu` you must get cosigner cards into the build flow. Phase 1 deletes
the NFC route deliberately, so the payload is the only route — and **the
emulator's embedded payload contains no cosigner cards.** Machine-checked, not
read off a comment:

```
$ python3 -c "…" cmd/emu/sysw_test_payload.bin      # 265 bytes, magic MNEMSYSW
  52  b'text:5345454448414d4d45522049492044454d4f205041594c4f4144'
 110  b'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'
 204  b'pass:636f727265637420686f727365206261747465727920737461706c65'
```

Three records: `ClassFreeText`, `ClassMnemonic`, `ClassPassphrase`. **Zero
`ClassMDMK`.** `cmd/emu/sysw_test_payload_host_test.go`'s
`TestSyswTestPayloadCarriesThreeClasses` pins exactly those three and no more.

So a Trace A walk today runs: boot → LOAD payload → digest compare → Engrave
Multisig → Build policy → five pickers → `syswOffer(ctx, th, sysw.ClassMDMK, …)`
finds nothing, the offer never appears → `bundleGatherFlow` → F-158's dead
scanner → tally stays `mk1 keys: 0`. The walk halts **one screen before** the
defect it exists to catch, having produced no artifact, and nothing in the plan
says that is a failure.

That is the F-150 shape exactly: a gate that is green because the second half
never executed.

Two things keep this from being fatal to the whole plan, and both are worth
stating. First, the plan's **host tests** are sound on this point — `testPlatform`
exposes a settable `sysw` field (`gui/gui_test.go:453`: `func (p *testPlatform)
SyswReader() sysw.Reader { return p.sysw }`), so S1's tests 1–7 can drive an
arbitrary MDMK payload, and S2 test 5 explicitly demands a raster assertion
calibrated against the real D-1. The tests would probably find the blank screen.
Second, the plan's honesty elsewhere is real — §5 names blind spots, §3 adds the
artifact-census rule.

But §4.5 exists *because* a green suite already missed this once. The walk is the
backstop for what tests miss, and the backstop cannot be built from what this
plan delivers. As specified, "emulator walk" is a phrase that discharges a gate
without defining work.

---

## Per-stage walk coverage

| stage | walk the gate demands | drivable in `cmd/emu` today? | blocker |
| --- | --- | --- | --- |
| **S0** | none stated (harness only) | n/a | S0 builds the oracle harness but **not the walk harness** — nobody owns it |
| **S1** | "Trace A reaches the gather with both cards" | **NO** | payload has no `ClassMDMK`; also no screen-assertion primitive, and zero artifacts so §3's census cannot gate it |
| **S2** | "Trace A completes end to end: engrave" | **NO** | same payload blocker; md1 byte-comparison has no string-extraction mechanism (C3) |
| **S3** | "shows `P2SH-P2WSH` on the restore doc for an `sh(wsh)` build" | **NO** | restore doc is **display-only**, never reaches the toolpath; and **no trace is `sh(wsh)`** (I1) |
| **S4** | "walk of the `both` happy path and of one loud failure" | **NO** | **no trace has a `both` slot** (I2); payload carries no card matching its mnemonic |
| **S5** | "Trace B completes", + byte comparison of every mk1 and every ms1 | **partially** | `shToolpath` digest-equality genuinely works for S5 test 7 (re-run determinism) — the one place the plan names a real mechanism. Everything else blocked by C1/C3 |
| **S6** | hardware, not a walk | n/a | correctly scoped |

---

## Verified mechanism inventory (what the emulator actually offers a walk)

| capability | status | evidence |
| --- | --- | --- |
| payload injection | **fixed blob only** | `platform.go:257` returns `embeddedSyswReader{syswTestPayload}`; no flag, no query param, no JS global |
| payload with cosigner cards | **absent** | byte dump above |
| input driving (taps) | **absent** | the only two globals are `shNFC` (`nfc_js.go:27`) and `shToolpath` (`toolpath_js.go:67`). Input is browser `pointerdown`/`pointerup` on the canvas (`platform.go:107-108`) |
| screen / text assertion | **absent** | `Summary` is `{Words, Steps, CutSteps, Vertices, Truncated, EndX, EndY, Bounds, Digest, CutsThroughOrigin, LongCuts}` — geometry only. No API returns any engraved or displayed string |
| toolpath geometry digest | **present, works** | `shToolpath.summary().digest`, order-sensitive over the vertex list |
| plate layout SVG | **present** | `shToolpath.plan()`; matches `internal/golden.Vectorize` byte-for-byte (`TestPlanPathMatchesVectorize`) |
| frame capture | **present, receiver only** | `design/journeys/shot_server.py` writes POSTed frames; it does not drive. `build_pdf_payload.py` shells headless Chrome only to print HTML→PDF |
| NFC card injection | **present but dead** | `NFCReader()` returns `p.nfc.reader()` (`platform.go:224`) — *not* nil — yet F-158 verified empirically that a card presented via `shNFC` never reaches any gather, cause not isolated |
| host-side plate render | **present** | `cmd/plateview`, same `FitBlocks`/`EngraveFitted`/`PlanEngraving` calls as the firmware |

---

## Findings

### C1 — The emulator payload carries no cosigner cards, so S1–S5's walks cannot execute, and no stage owns fixing it

**Owning stage: S0.**

`grep -i "sysw_test_payload\|test payload\|cmd/emu"` over the plan returns
**zero hits**. The plan's traces both require cosigner mk1 cards on a payload
(Trace A: two; Trace B: one), the emulator's payload has none, and no deliverable
in any stage changes that. Every gate from S1 to S5 depends on a fixture the plan
does not create.

**Bounded fix.** S0 adds a *second* js-only blob — do **not** mutate the existing
one. `syswTestDigest` (`sysw_test_payload.go:64`) is quoted on the host side of
the air gap in the published Load Payload journey PDF and photographed on the
device side; `TestSyswTestPayloadCarriesThreeClasses` pins its exact three
classes. Regenerating it silently invalidates a published document in the one way
a reader cannot detect — which is what that test's comment says.

So: `cmd/emu/sysw_multisig_test_payload.{bin,go}`, `//go:build js`, carrying a
mnemonic plus the mk1 cards Traces A/B/C need (see I2 for the `both`-slot card).
Add a selector — a `?payload=multisig` query param read via `syscall/js`, which
`platform.go:236-242` already names as the supported mechanism for exactly this —
so `SyswReader()` picks a blob. Extend `confinement_test.go`'s `guarded` list and
`TestSyswTestPayloadIsConfinedToJSOnlyFiles`'s `allowed` map to cover the new
identifiers, and add a class-pin test for it. Record its digest the same way.

### C2 — "Emulator walk" is undefined: no harness exists, and the emulator has no input-driving API

**Owning stage: S0.**

§4.5 requires the walk be "automated (a script, not a remembered click
sequence)". Nothing in the repo drives the emulator. `shot_server.py` receives
frames. The emulator exposes no tap injection, so the only route is a browser
driver dispatching pointer events at hardcoded canvas coordinates on a 480×320
framebuffer with no accessibility tree — feasible, unbuilt, and unowned. The plan
cites "emulator walk" at lines 169, 321, 358, 438, 536 and defines it at none of
them: not what it drives, not what it asserts, not what "passed" means.

**Bounded fix.** S0 delivers two things. (1) `cmd/emu/drive_js.go` exposing
`window.shTap(x, y)` and `window.shScreen()` — the latter returning the current
screen's title string, which is the minimum needed to assert *where a walk got
to*. (2) `design/journeys/walk_multisig.py`, a Playwright script taking an
ordered step list of `(tap | screen-checkpoint)` pairs, and writing a gate record
containing the input tuple, the oracle commits (§1a already requires these), and
the checkpoint list **reached vs. required**. Then each stage gate names its step
list instead of the phrase "emulator walk".

### C3 — §4.5's byte comparison has no mechanism: the emulator cannot emit the engraved strings

**Owning stage: S0 (rule the plane), first bites at S2.**

§4.5 and §1a require the walk's artifacts be compared byte-for-byte against the
primary's output. **No API returns a string from the emulator.** `shToolpath`
returns geometry; the strings exist only as stroked b-spline paths. S2's gate —
"the current primary BUILDS an md1 from the same inputs and the strings are
equal" — is executable in a host test and not in a walk. S5's extension to
"every mk1 and EVERY ms1, byte for byte" inherits the same hole.

**Bounded fix — pick one and write it into §1a's table.**

(a) *Smaller, matches the current wording.* Add `shToolpath.strings()` returning
the source strings of the loaded plate, plumbed through the existing
`gui.PlateAware` hook (`gui/plate_hook.go:49`) that already hands `cmd/emu` the
plan. Compare against the primary per §1a.

(b) *Larger, also proves layout.* Rule the walk's plane geometric:
`shToolpath.plan()` SVG versus `cmd/plateview` rendering the primary's string.
`plan()` already matches `golden.Vectorize` byte-for-byte, so this is a string
comparison of two SVGs. Requires extending `plateview`, which today offers only
fixed named plates plus `-plate freetext` — no constellation-string plate.

(a) discharges §1a as written; (b) additionally catches a correct string laid out
wrongly. (a) plus S6 is defensible; silence is not.

### I1 — S3's gate demands an `sh(wsh)` walk that neither trace contains

**Owning stage: S3.**

§2 says the two traces "are this plan's acceptance criteria rather than
illustrations" and "a stage that closes green while its trace still breaks has
not closed." Trace A is `template(wsh)`; Trace B is a 3-of-4 wsh. S3's gate needs
an `sh(wsh)` build. There is no trace for it, so S3's walk is unmapped.

Compounding: the restore doc is **display-only** —
`gui/multisig_restore.go:56-58`, "Display-only — no secret, no engrave" — so it
never enters the toolpath and C3's fix (a) does not reach it either. Asserting
`P2SH-P2WSH` on that screen needs the `shScreen()` checkpoint from C2, or a
framebuffer hash.

**Bounded fix.** Add **Trace A′** to §2 — Trace A with `template(sh(wsh))` — and
state that its assertion is a screen checkpoint, not an artifact comparison.

### I2 — S4's gate demands a `both`-slot walk; neither trace has a `both` slot

**Owning stage: S4.**

S4's whole deliverable is the seed↔key cross-check, and every one of its gate
tests 1–2 fires only on a `both` slot (payload seed *and* payload card for the
same slot). Trace A: operator types the seed, cosigners are cards — no `both`.
Trace B: `@0..@2` derived from seeds, `@3` is a foreign cosigner card — no
`both`. So S4's gate ("Emulator walk of the `both` happy path and of one loud
failure") has no journey behind it, and the live exposure §2.2 D-5 names is
precisely the `both` case.

**Bounded fix.** Add **Trace C** to §2: a payload carrying a mnemonic *and* the
mk1 derived from it, one slot marked `both`. The loud-failure arm is the same
walk against a twin blob whose card key does not derive from the seed. This also
fixes C1's fixture requirement — the multisig blob must carry a seed-matched card
— so build them together.

### I3 — The artifact census cannot gate a walk that produces no artifacts

**Owning stage: S0 (with the harness).**

§3's rule is good and correctly aimed: the census derives from the recorded input
tuple, never from what the walk produced, so a walk that fell over after plate
one cannot satisfy a total gate. But it only fires on **artifacts**. S1's gate
("reaches the gather with both cards") and S3's gate (a display-only screen)
produce zero engraved artifacts, so the plan's only stated anti-false-pass rule
for walks is **inert for exactly the two stages whose walks are furthest from an
engrave** — including S1, the stage that owns the defect class this plan exists
for.

**Bounded fix.** The census covers the tail; add checkpoints for the head. A walk
fails unless it reached every required `shScreen()` checkpoint in order, and the
gate record prints reached-vs-required. One sentence in §3, next to the census
rule, plus the `shScreen()` API from C2.

### M1 — §5's blind-spot list omits the two blind spots that bind hardest

§5 names hardware and `plan-cite-gate.sh`. It does not name (i) the walk has no
text or screen assertion, so anything not engraved is human-eyeball-only, or
(ii) which traces are walkable at all is determined by the embedded payload blob,
a fixture outside the plan. Given the plan's own standard — "a gate that hides
its blind spot is worse than none" — both belong there once C1–C3 are folded.

### M2 — F-158's mechanism is misdescribed in a way that hides the absence of a workaround

Not the plan's text, but it affects how a reader judges the walk. `cmd/emu`'s
`NFCReader()` does **not** return nil — it returns `p.nfc.reader()`
(`platform.go:224`), a page-fed source, and `platform.go:209-214` records that
the older "returns nil" comment "stopped being true when nfc.go was added." The
nil-returning reader is `gui`'s *test* platform (`gui_test.go:445`). F-158 item 1
is still empirically correct — cards presented via `shNFC` never reach a gather —
but the cause is unisolated, so NFC is not an available fallback for getting a
cosigner card into an emulator walk. Worth one line wherever the plan leans on
"NFC is out of phase 1" as a scoping decision rather than a live limitation.

---

## What a green walk would and would not establish, once buildable

Stated plainly, since the plan's §5 gestures at this but does not enumerate it.

**Would establish:** the real `gui` package, compiled from the shipped firmware
source, routes an operator from boot to a completed engrave without a dead end;
the plate geometry the driver receives is what the layout intended; re-running
identical inputs mints byte-identical plates (S5 test 7, the one walk assertion
the plan grounds in a real mechanism).

**Would not establish:** anything about stepper motion, plate registration, cut
depth, or NFC; that the *strings* match the primary, unless C3 is fixed; that any
screen is legible, correct or non-blank, unless C2's checkpoints or a raster
assertion cover it — and a blank body under a correct title is precisely F-151's
defect class, which a title checkpoint would pass.

That last sentence is the one to carry into the fold. The walk is necessary and
this plan is right to gate on it; it is not sufficient, and the gap between
"reached the screen" and "the screen drew" is where the original defect lived.
