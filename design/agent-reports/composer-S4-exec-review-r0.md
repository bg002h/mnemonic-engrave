# composer S4 — independent adversarial EXECUTION review, round 0

**Verdict: 0 Critical / 1 Important / 4 Minor / 4 Nit.**

Scope: fork `composer-s4-emu` `86cec95b05f9fd5c444c5c23983a79ca1387a2e6`
(base `main` `60bee002` + the merged W-2 `3cc71d9b`) and mnemonic-engrave
`composer-s4-emu` `c6adac22b71c6ba95dcf14be6d61fb9e571461af` (base `master`
`a262e7d`), against `design/IMPLEMENTATION_PLAN_composer_S4_acceptance.md` at
`a5f92c8`, Tasks 1–3.

**The one question — answered.** The comparison is a comparison and it CAN fail:
the plan's named mutation (the unchunked keyless string) is caught byte for byte,
a one-hex Policy-ID corruption is caught, a dropped md1 chunk is caught, and a
one-character address corruption is caught. `shTargets()` is a reader, and on a
pre-W-2 build the driver refuses with "no tappable rows" rather than walking a
screen the machine cannot operate — reproduced here, not taken from the report.
The one Important is that the **negative control cannot attribute its own
failure**: it reports PASSED when the walk breaks for any reason at all.

Read-only throughout. Every mutation ran in `cp -r` copies under
`/scratch/code/shibboleth/.tmp/s4rev/` (`fork-a`, `fork-b`, `eng-a`, each with
its `.git` link removed so no `git` command could reach a shared gitdir).
Nothing was committed. Both worktrees end clean at the tips above:

```
$ git -C /scratch/code/shibboleth/wt-composer-s4-emu status --porcelain   # empty
$ git -C /scratch/code/shibboleth/wt-engrave-s4-emu  status --porcelain   # empty
86cec95b05f9fd5c444c5c23983a79ca1387a2e6
c6adac22b71c6ba95dcf14be6d61fb9e571461af
```

Ports 8811–8826 were used throughout; 8793/8734 were never bound.

---

## Lens 1 — the comparison can fail. Proved three ways.

### (a) The plan's named mutation — the UNCHUNKED keyless string. CAUGHT.

```
$ printf 'md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc\n' > out/composer/keyless-tr.md1.txt
$ EMU=…/fork-a/cmd/emu python3 capture_composer.py --arm keyless --no-build --port 8811 --shot-port 8812
EXIT=1
DRIVER FAILED on leg keyless: the key-less template plate: string 1 of 1 does not match the host's BYTE FOR BYTE.
  device: "md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3" (56 chars)
  host:   "md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc" (47 chars)
    at compareEngraved (shots_composer.js:371)
```

The driver does not verify, it compares. Baseline for contrast: the same command
on the untouched artifact exits 0 in 31 s with `all legs matched the host.`

### (b) One character of one address, and one hex of the Policy-ID. BOTH CAUGHT.

Address (this is `--prove-it-can-fail` itself, run on a healthy tree):

```
$ EMU=… python3 capture_composer.py --arm keyed --prove-it-can-fail --no-build …
NEGATIVE CONTROL: expecting bc1q8cf5g5f…n3mqk0tp4q (corrupted)
DRIVER FAILED on leg keyed-A: Error: the device's proof does not match the host's:
NEGATIVE CONTROL PASSED: the walk refused the corrupted address.     EXIT=0   (39 s)
```

Policy-ID, `4dd749a8…` → `4dd749a9…` in `out/composer/keyed.id.txt`:

```
EXIT=1
DRIVER FAILED on leg keyed-A: the Policy-ID: the screen does not carry
  "Policy-ID: 4dd749a9372af515a61d7104faf944ef".
    at must (shots_composer.js:140) / run (shots_composer.js:722)
```

It fails at the seated stub screen (row 16), one screen EARLIER than the consent
— so the id is checked twice, which is stronger than the plan requires.

### (c) A dropped chunk, and the entry-count assertion made inert.

`head -6` of the 7-chunk `keyed.md1.txt`:

```
EXIT=1
DRIVER FAILED on leg keyed-A: form A: the plates split into 7 string(s), the host wrote 6.
    at compareEngraved (shots_composer.js:366)
```

This run is itself the "tautological entry count" experiment: the census entry
count was **2 == 2 and passed**, `censusClaim (2) === entries.length (2)` passed,
and the **flat byte comparison** is what caught it. So the two halves are
genuinely disjoint, exactly as `compareEngraved`'s own header claims — the flat
list catches a wrong string, the entry count catches a repacking (7 chunks onto
3 plates would trip `entries.length !== expect.entries` while the byte list
stayed identical). Neither is load-bearing for the other.

### `--prove-it-can-fail` itself — what it corrupts

`capture_composer.py:224-231` flips the LAST character of `forms["A"]["addresses"][0]`
(`bad[:-1] + ("q" if bad[-1] != "q" else "p")`). It is computed from the host
file, not a fixed string, so a refactor cannot leave it stale — but see **I-1**.

---

## Lens 2 — counterexamples through the driver

**Stale `emu.wasm`.** Covered twice: `capture_composer.py:170`
(`wait_for_function("window.shTargets !== undefined", timeout=60_000)`) and
`shots_composer.js:443-446` at the top of `run()`. A third copy of the guard
inside `chooseRow` is unreachable — see **N-1**.

**A wrapped paged screen — stop on recurrence, or over-count?** Stops. Measured
shot/page counts on a clean `--arm both` (exit 0, 50 shots):

| leg | stub | mapping | consent | census claim | shots |
| --- | --- | --- | --- | --- | --- |
| keyed-A | 2 then 3 | 2 | 4 | 2 | 21 |
| keyed-B | 2 then 3 | 2 | 4 | 4 | 21 |
| keyless | 2 | – | 2 | 1 | 8 |

`readAllPages` breaks on `text === pages[0]` (the wrap) and throws if `maxPages`
is reached without one, so the shot count is bounded by the page count and is
never a loop bound. Page counts are asserted where the plan pins them (stub1 = 2,
lock echo = 1, key-less stub = 2); the unpinned ones (stub2, mapping, consent)
are covered downstream — a consent that lost its address pages fails the address
comparison.

**`shToolpath.strings()` newline-joined?** Yes, and it is the source that says
so, not the plan: `gui/engraved_hook.go:124` — `ea.PlateText(ids, strings.Join(strs, "\n"))`.
Measured too: form A's 2 entries split into exactly the host's 7 chunks (lens 1c
output above). `census.unattributed` (`cmd/emu/engraved.go:140`) exists and the
driver asserts it is 0.

**The `SCAN` row at `Where from?` — by name or by index?** By **index 0**, with a
post-condition that names the destination (`waitFor("Source: the systemwide payload")`).
That is safe by construction: `gui/derive_xpub.go:283-288` appends `FROM PAYLOAD`
first whenever the payload carries a `ClassMnemonic`, `TYPE IT` second, and
`SCAN` last under `FeatureNFC` — so row 0 is `FROM PAYLOAD` on the emulator and
on the device alike. Measured on the emulator: `Wherefrom?FROMPAYLOADTYPEITSCAN`,
3 targets. **No finding.**

**Every tap-by-index, measured against the frame the emulator actually drew.** I
instrumented `chooseRow` in a copy to record `shTargets().length` and the screen
for all 26 calls per keyed leg. Every index the driver uses matches the plan's
itinerary row and the emulator's own row set:

```
i=1 rows=2  door                Keysloaded:2,plus1seed.ScancardsBuildanewpolicy
i=1 rows=4  Which script?       Taproot(tr)Segwit(wsh)Nested(sh-wsh)Legacy(sh)
i=0 rows=7  Start from?         Buildmyownpaths + the six presets      (W-1)
i=0 rows=3  Spend paths         slots:0/keysavailable:2 …
i=1 rows=5  n=2 picker          12345
i=1 rows=5  open Path 2         Path1:2-of-2 Path2:1key Add… Change… Done
i=1 rows=5  Time lock           Keys Timelock Hashlock Removepath Moveup
i=0 rows=3  hash 1              hash1abababab..abababab Type64hex Nohashlock
i=4 rows=5  Done                (computed as listRows-1, not hard-coded)
i=0 rows=4  Seat @0             two 73c5da0a rows, Typeaseed, Leaveunseated
i=0 rows=3  Where from?         FROMPAYLOAD TYPEIT SCAN
i=1 rows=2  Watch-only (keys)   Full(seed+keys) Watch-only(keys)
```

`Done` is computed from `shTargets().length - 1` in both arms, so it cannot go
stale when a path is added. **No index diverges.** One post-condition is inert —
see **M-1**.

**The F1 `Payload Warnings` page.** One CONFIRM, then `waitFor("Keep this payload
loaded?")`; a second page would time out rather than pass. The F1 copy itself is
asserted verbatim.

**The `Bundle engraved` modal and the door after it.** Handled as the fourth
`ENGRAVE_HANDLERS` entry; the loop terminates on `DOOR_ROW` and both arms then
assert the door's own lead (`Keys loaded: 2, plus 1 seed.` /
`No keys loaded. This builds a key-less template.`). An unrecognised screen
pushes `STALLED` and breaks, and `if (!reachedDoor) throw` makes that loud.

---

## Lens 2b — `shTargets()` is a READER. Confirmed by reading and by running.

**By reading.** `cmd/emu/screen.go:59-118` — `frameTargets` calls
`op.Drawer.Hit`, which is `gui/op/op.go:639-646`: a linear scan of `d.inputs`
returning the first bounds containing the point. It mutates nothing, allocates
one `[]image.Rectangle`, and stores plain rectangles rather than tags (the
comment gives the lifetime reason and it is correct — a tag is a live pointer
into GUI state). `gui/event.go:319` performs the *same* `d.Hit(pe.Pos)` for a
real fingertip, so the equivalence claim is exact. The recorder builds its own
`op.Drawer` per frame (`screen.go:143`), so `skipInputOps` is never set and the
inputs are always populated. `screen_js.go:62-77` only reads and converts.
**It injects no event, reaches no flow, and synthesises no target.**

**By running — the pre-W-2 build.** In a copy of the Task-3 tip I reverted
`gui/composer_paged.go` to its `60bee002` content (the only W-2 file; the four
call sites of `composerPageLines` are all inside it or in `_test.go`), rebuilt
`emu.wasm`, and ran the **unmodified** driver:

```
$ EMU=…/fork-a/cmd/emu python3 capture_composer.py --arm keyless --no-build --port 8813 --shot-port 8814
EXIT=1
DRIVER FAILED on leg keyless: choosing Add a spend path: the frame offers 0 tappable row(s),
so row 0 cannot be reached BY TOUCH. On a device whose only input is the panel that is a
defect in the screen, not in this walk.
Screen: "Spendpathsslots:0AddaspendpathChangethescriptDone"
```

Zero rows on a composer pick screen, and the driver names it as a defect in the
screen. The report's central claim reproduces.

**Does it include the navigation column, or miss a drawn row?** Neither.
Measured over 26 screens (table above): the target count equals the visible row
count on every one, and the lead/header — drawn by the same
`composerPageLines` but wired with no `op.Input` (`composer_paged.go:163-172`,
`j < shown` from `rowBase`) — correctly has no target. The pick screen's bands
are `x=8 w=419` on a 480 px panel, stopping short of the nav column at 427
(`bandRight = min(bandLeft+lineWidth, dims.X - NavBtnPrimary.width)`), and the
centre-line probe at `x=240` is inside every band and outside every nav button.

---

## Lens 3 — the payload

**`cmd/buildpayloadcomposer` refuses on a wrong xpub.** Both pins, both refuse:

```
$ sed -i 's/…MBXd6Vk/…MBXd6VZ/' cmd/buildpayloadcomposer/main.go && go run ./cmd/buildpayloadcomposer
buildpayloadcomposer: A@1 — composer slot @1, same master, second account:
  the device's own derivation at m/48'/0'/1'/2' gives …Vk but this file pins ms derive's …VZ
  One of the two implementations is wrong. Do NOT re-pin without finding out which.
EXIT=1
$ (same for seedWantXpub)                                                     EXIT=1
```

It emits the earlier records before refusing; the transcript's
`diff <(go run ./cmd/buildpayloadcomposer) records.txt` gate catches the partial
output regardless of the swallowed exit code, so the gate holds.

**The digest test recomputes rather than compares strings.** `sysw.Open` +
`sysw.PublicDataHash` (`sysw_composer_payload_host_test.go:32-36`), with the pin
read out of the `//go:build js` source. Both mutations caught:

```
# one byte of the .bin flipped
digest drift: the blob hashes to "b986 885f 4b74 9ed8 …" but syswComposerDigest pins "dbe9 e774 …"
  … and TestSyswComposerPayloadCarriesTheComposerClasses also fired:
  record 1 classifies as 0, want 10; inventory: 1 ClassKey, 1 ClassHash, 1 ClassNow, 1 ClassMnemonic

# the pinned constant changed dbe9 -> dbe0
digest drift: the blob hashes to "dbe9 e774 …" but syswComposerDigest pins "dbe0 e774 …"
```

The cross-implementation arm runs and its absence is fatal, as advertised:

```
$ ME=…/target/debug/me go test -tags oraclelive -run TestSyswComposerPayloadDigestAgreesWithMe ./cmd/emu/
    auditing against …/target/debug/me (me 0.8.0)                 --- PASS
$ ME=/nonexistent-me  (same command)                              --- FAIL (not a skip)
```

**The confinement test discovers the new embed.** Removing the `//go:build js`
line from `cmd/emu/sysw_composer_payload.go`:

```
--- FAIL: TestEveryEmbeddedPayloadIsStructurallyConfined
  cmd/emu/sysw_composer_payload.go carries a //go:embed but is not //go:build js …
  names "syswComposerPayload" / "syswComposerDigest" / "syswComposerReader" …
  confined 16 embed token(s) across 696 scanned files
```

**`shSysw("composer")` before the door.** Yes — `bootAndChoosePayload` calls it
immediately after declining the boot offer, before `goTo("Load Payload")`, and
refuses if the returned string is not `"composer"` (which is what catches a wasm
that predates `platform.go`'s new case).

---

## Lens 4 — mutating the host half

**Without `--force-chunked` on the keyless mint** — caught twice over, at the
host and then at the device:

```
$ ./transcript_composer.sh                       TRANSCRIPT EXIT=1, 3 GATE FAILURES
GATE FAIL  keyless md1 string   got md15zfdsssj6…  want md1fkzyyqq9q…
GATE FAIL  keyless md1 length   got 47             want 56
GATE FAIL  the unchunked form is a DIFFERENT string   got same  want different
```

and the artifact it leaves behind is exactly the string lens 1(a) proves the
capture rejects.

**`@2` at `48'/0'/2'/2'` instead of B's account `0'`** — 5 gate failures, exit 1:

```
GATE FAIL  keyed md1 chunk 1        md1ftggrrq9q…  want md1flv5xrq9q…
GATE FAIL  Policy-ID                a35aa5031b1cfcd43573c890c02d6431  want 4dd749a8372af515a61d7104faf944ef
GATE FAIL  md1-encoding-id          5a1033dff98c14db53f325fd27bf76a0  want fb28698ee8bdbc18c6ee36598f2124fe
GATE FAIL  form-B template chunk 1 / chunk 2
```

Worth recording because it is counter-intuitive and correct: **Template-ID and
the four addresses do NOT move.** The Template-ID is origin-independent (which
is exactly why the plan can say the unseated chunk set "shares the Template-ID"),
and `md address` derives from the `--key` xpubs, so the declared origin is
metadata. Lens 1(b) already shows the Policy-ID change alone is caught on the
device, so the mutation is closed.

**The plan's own gate.** `diff <(go run ./cmd/buildpayloadcomposer) out/composer/records.txt`
is empty; `cmp out/composer/payload.bin cmd/emu/sysw_composer_payload.bin` is
identical (782 bytes); digest `dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b`,
`pub_len: 730`, `sealed: false`. Full script re-run from my copy: **exit 0, 27
GATE PASS, zero failures.**

---

## Lens 5 — what the diff made false elsewhere

**The three shipped drivers, run unmodified against this tip** (their `EMU` is
hard-coded to `../../../seedhammer/cmd/emu`, so I staged a tree whose
`seedhammer` symlinks to the fork copy — neither main checkout was written):

| driver | result |
| --- | --- |
| `capture_walletpolicy.py --port 8821 --shot-port 8822` | **exit 0** |
| `capture_seating.py --port 8823 --shot-port 8824` | **exit 0** |
| `capture_tr_pathological.py --port 8825 --shot-port 8826` | **exit 0** |

**`cmd/emu/needle_test.go`** pins no new needle for this walk; `shots_composer.js`
declares no `NEEDLE_*` constant. I measured the production-site count of the
walk's load-bearing anchors rather than trusting that (`grep --include=*.go`,
`_test.go` excluded, comment hits discounted): `Build a new policy` **1** rendered
site (`gui/composer_door.go:116`; the other five hits are comments),
`Which script?` 1, `Start from?` 1, `Which form?` 1, `Bundle engraved` 1,
`Where from?` 1. Nothing is ambiguous today. See **N-4**.

**`design/journeys/README.md`** has no composer row and **`design/FOLLOWUPS.md`**
is untouched by the diff. Both are correct: plan Task 6 (controller, last) owns
the README row, the PDF builder, and F-460…F-463. F-462 and F-463 are filed with
owning phase *post-S4 polish*, which is what the driver's comments cite.

**Rust:** the engrave diff touches only `design/journeys/{capture_composer.py,
transcript_composer.sh,transcript_composer.txt}` — **no Rust file changed**, so
`cargo fmt --check`, clippy and nextest are unaffected and were not run.

**`transcript_composer.txt` regenerates.** Re-running the script produces a file
differing from the committed one only in absolute paths, the `ls -la` mtimes, and
the fork rev line (**N-3**).

---

## Lens 6 — whole-repo gates, as CI runs them

Go 1.26.7 by path, `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`,
`TMPDIR=/scratch/code/shibboleth/.tmp`.

| gate | result |
| --- | --- |
| `go test -timeout 30m $(go list ./... \| grep -v /gui$)` | **all ok, EXIT=0** (53 packages) |
| `gui-shard-test.sh ./gui/ 24` | **ok — all 1188 tests ran across 24 shards**, 60 s |
| `go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/ ./cmd/emu/` (the CI step this diff edits) | **ok** ×4 |
| `scripts/test-32bit.sh` | `GOARCH=386 test: exit 0`, `GOARCH=arm build: exit 0` |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | **exit 0** |
| `go vet ./gui/` | only the two pre-existing `testing.ArtifactDir` findings |
| `gofmt -l` on the diff's own `.go` files | **clean** |
| `gofmt -l cmd/` | clean |
| `gofmt -l cmd/ gui/` | **3 files** — see **N-2** |
| firmware size recipe | **1,548,784 code / 31,796 data / 31,004 bss → 1,580,580 B flash / 62,800 B RAM** |

The firmware number **matches plan §4's pin at `a5f92c8` exactly** (W-2's +640 B
over `60bee002`; Tasks 1–3 add nothing). The review brief's lens-6 figure of
1,579,940 B is the pre-W-2 number — see **N-2**.

Both new gui tests were confirmed to RUN, not merely to be filtered away:

```
=== RUN   TestWalkDigitPadCoordinatesTypeTheIntendedNumber   --- PASS
=== RUN   TestComposerPickScreenRowsAreTouchable             --- PASS
```

and the digit-pad test has real discriminating power — four mutations of the JS
constants, four distinct failures:

```
pitch 34→51        digit 2 (2): the walk taps (257,152) and there is no touch target there at all
row0 x0 206→240    digit 3 (9): tapped (274,244) and the field does not read "129"; frame "…239blocks(about1.7days)…"
row3 x0 240→206    digit 5 (0): the walk taps (206,290) and there is no touch target there at all
row1 y 198→244     digit 4 (6): tapped (274,244) and the field does not read "1296"
```

(`pitch 34→35` passes, correctly: the property asserted is "these coordinates
type the intended number", and a 1 px shift still lands on the same key.)

---

# Findings

## I-1 — `--prove-it-can-fail` cannot attribute its own failure: ANY driver error reports PASSED

**`design/journeys/capture_composer.py:182-188, 271-276`**

`drive()` returns `None` on any `page.evaluate` exception (`:182-188`), and
`main()` reads `res is None` as proof that the address comparison caught the
corruption (`:271-274`). It is the same value for "the walk broke at step 2".

**Reproduction** (mutated only the host artifact, no source edit):

```
$ sed -i 's/dbe9/dbe0/' out/composer/payload.digest.txt
$ EMU=… python3 capture_composer.py --arm keyed --prove-it-can-fail --no-build --port 8811 --shot-port 8812
NEGATIVE CONTROL: expecting bc1q8cf5g5f…tp4q (corrupted)
DRIVER FAILED on leg keyed-A: the device's payload digest does not equal the host's `me sysw show`:
  the screen does not carry "dbe0 e774 e9a4 9231 0b62 626c 2b41 cf4b".
NEGATIVE CONTROL PASSED: the walk refused the corrupted address.
real 0m8.4s          EXIT=0
```

Eight seconds. The walk died at itinerary row 2 and never reached the consent
screen at row 17, where the address is compared — and the control still declared
the address comparison proven. The plan's words are *"exits 0 only if the walk
caught it"*; the code exits 0 if the walk stopped. A control that passes for the
wrong reason is the one thing a control must not do, and this class is precisely
what `--prove-it-can-fail` exists to rule out for everything else.

Not Critical: the comparison it is meant to certify **does** work — on a healthy
tree the same command fails with `the device's proof does not match the host's:
address bc1q8cf5g5f…tp4q`, 39 s, having walked the whole itinerary. What is
missing is the control's own attribution arm.

**Hypothesis.** Have `drive()` return the exception text (or `("FAILED", msg)`)
instead of `None`, and in the `prove_it_can_fail` branch require that text to
contain `the device's proof does not match the host's` — or the corrupted
address itself — before printing PASSED; otherwise `sys.exit("NEGATIVE CONTROL
INCONCLUSIVE: the walk failed before the comparison — …")`. Roughly six lines,
and it makes the recorded `--prove-it-can-fail exit 0` mean what the plan says it
means.

## M-1 — one `chooseRow` post-condition is already true before the tap

**`cmd/emu/shots_composer.js:695`** — `await chooseRow(0, "seed 1", "Skip the passphrase");`

`chooseRow`'s own header says *"`expect` is not optional politeness … the
post-condition is the only thing that makes a coordinate safe"*. For this one
call the post-condition is satisfied by the pre-tap frame, so it certifies
nothing. Measured by instrumenting `chooseRow` in a copy to test the current
screen against `expect` before tapping — **one hit out of 26 calls per keyed leg,
zero in the keyless arm**:

```
PRESAT Skip the passphrase expects "seed 1" which is ALREADY on screen:
  "AddaBIP-39passphrase?SkipAddpassphrasePassphraseseed1"
```

Consequence is bounded: taking `Add passphrase` instead would land on a keyboard,
and the next line's `must(seat2b, "(any slots)")` would catch it. So this is a
dead assertion, not a blind step.

**Hypothesis.** Use `"(any slots)"` (or `"seed 1  (any slots)"`) as the
post-condition — measured absent from the passphrase screen and present on the
re-drawn pick list.

## M-2 — `Choose engraving` is asserted by inclusion where the plan claims exclusivity

**`cmd/emu/shots_composer.js:779`** (`variantRows: "TEXT ONLY"`), **`:524`**
(`"TEXT + QR"`), consumed at **`:319`** (`must(v, variantRows, …)`).

Plan row 20a: *"on every packed plate `Choose engraving` offers `TEXT ONLY`
alone"*; keyless row 6: *"`TEXT + QR` is row 0; `QR ONLY` would give the operator
a plate the SH2 can never read back"*. Both are checked with a substring test.
Measured screens:

```
keyed-A   "ChooseengravingTEXTONLYCard1of1|Plate1of2"
keyless   "ChooseengravingTEXT+QRTEXTONLYQRONLYCard1of1|Plate1of1"
```

So `must(v,"TEXT ONLY")` would still pass if a packed plate started offering all
three — the "alone" half is unasserted. And the handler takes the variant with a
bare `shTap(CONFIRM)` (`:317`), i.e. whatever row is selected by default, so
`must(v,"TEXT + QR")` does not establish that TEXT + QR is the row taken. The
engraved strings are unaffected either way, so the exposure is the journey
RECORD, not the comparison.

**Hypothesis.** Add `mustNot(v, "QR ONLY")` on the keyed arm; on the keyless arm
select row 0 through `shTargets()` (as `chooseRow` does) rather than relying on
the default selection, and assert the taken row's label.

## M-3 — the address comparison degrades silently to zero comparisons

**`design/journeys/capture_composer.py:94`** (`"addresses": recv[:2] + chg[:2]`)
and **`cmd/emu/shots_composer.js:743`** (`for (const a of (expect.addresses || []))`).

`need()` fails only on a MISSING file, not an empty one, and the driver's loop
over an empty list compares nothing while the leg still prints `all legs matched
the host.` Demonstrated by driving `read_keyed()` directly:

```
addresses with EMPTY receive/change files: []
policyId still present: 4dd749a8372af515a61d7104faf944ef
```

Mitigated in practice, which is why this is Minor rather than Important:
`transcript_composer.sh`'s `runcap` **deletes** a capture file that matched
nothing (`:68-74`), so the empty-file state is not reachable through the intended
path — `need()` would fatal on the absence instead. It is a defence-in-depth gap
in the half of THE COMPARISON that the plan calls its point.

**Hypothesis.** In `read_keyed()`, `if len(addresses) != 4: sys.exit(…)` — one
line, and it also pins the count the consent screen must carry.

## M-4 — the new driver's default ports collide with `capture_seating.py`'s, and its docstring names a third pair

**`design/journeys/capture_composer.py:24, 205-206`**

Every shipped driver has a unique port pair; this one duplicates
`capture_seating.py`'s exactly:

```
capture_operator.py        8791 / 8732        capture_pathological.py    8791 / 8732
capture_walletpolicy.py    8793 / 8734        capture_tr_pathological.py 8795 / 8736
capture_seating.py         8797 / 8738  <──   capture_composer.py        8797 / 8738
capture_csid_warning.py    8798 / 8739        capture_hashvault.py       8799 / 8740
capture_rcw.py             8801 / 8742
```

and the usage line at `:24` advertises `[--port 8793] [--shot-port 8734]`, which
are `capture_walletpolicy.py`'s. `serve()` exits with a clear message on a
collision, so the failure is loud, not silent.

**Hypothesis.** Take the free pair `8803 / 8744` and correct the docstring.

## N-1 — the stale-`emu.wasm` guard inside `chooseRow` is unreachable

**`cmd/emu/shots_composer.js:168-172`** — `const targets = window.shTargets();`
is on the line **before** `if (typeof window.shTargets !== "function") throw …`.
On a stale wasm the call throws `TypeError: window.shTargets is not a function`
first, so the friendly message can never print. Harmless: the identical guard at
the top of `run()` (`:443-446`) and `capture_composer.py:170` both cover the case
before a walk starts. Delete the inner copy or hoist it above the call.

## N-2 — two gate results in the review brief do not reproduce

Recorded so the controller's own record is right, not as a defect in the diff.

1. **`gofmt -l cmd/ gui/` is not clean.** It lists `gui/transaction.go`,
   `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go` (two
   blank lines where gofmt wants one). All three are **outside this diff**
   (`git diff --name-only 60bee002..HEAD | grep transaction` → nothing) and
   unformatted at `60bee002`, `3cc71d9b` and `05d903b` alike; `.github/workflows`
   runs no gofmt check. The plan's actual gate is `gofmt -l cmd/`, which IS
   clean, and so are the diff's own `.go` files. Pre-existing, unowned by S4.
2. **The firmware pin.** The brief's lens 6 requires `1,579,940 B flash`; that is
   the `60bee002` number. The tree measures `1,580,580 B flash / 62,800 B RAM`,
   which is what plan §4 pins after the `a5f92c8` fold. The plan is right and the
   brief is stale.

## N-3 — `transcript_composer.txt` records fork rev `05d903b`, not the branch tip

The committed transcript was generated at the Task-1/2 tip. Regenerating it at
`86cec95` changes only the `git rev-parse --short HEAD` line (plus absolute paths
and `ls -la` mtimes) — the payload bytes, ids, addresses and md1/mk1 strings are
byte-identical, and all 27 gates pass either way. Worth one regeneration at the
merge commit so the record names the tree it was produced from.

## N-4 — no composer needle is pinned in `cmd/emu/needle_test.go`

`needle_test.go` guarantees a Build-policy walk anchors on strings with exactly
one production site; nothing equivalent covers the composer walk, whose engrave
loop terminates on the bare `DOOR_ROW = "Buildanewpolicy"`. Measured today that
is a single rendered site (`gui/composer_door.go:116`), so there is no live
ambiguity — but a future screen reusing that copy would end the engrave tail
early and `compareEngraved` would then run against a partial census. Adding
`{"Build a new policy", "gui/composer_door.go"}` and `{"Which script?",
"gui/composer_shape.go"}` to `buildFlowNeedles`' pattern would make the anchor a
machine-checked fact rather than a measurement in this report.

---

# Closing counts

**0 Critical / 1 Important / 4 Minor / 4 Nit.**

Lenses that found nothing: **lens 1** (all three mutations caught; both halves of
the census assertion are live and disjoint), **lens 2b** (`shTargets()` is a
pure reader; zero rows on the pre-W-2 build, reproduced; no nav column, no missed
row across 26 measured screens), **lens 4** (both host mutations caught at the
host AND at the device), and **lens 6** (every gate green; the firmware size is
the plan's pinned number to the byte). Lens 2 produced M-1 and M-2, lens 3 found
nothing (all four payload mutations caught), lens 5 produced N-3 and N-4.

Only **I-1** blocks. It is confined to `capture_composer.py`'s
`--prove-it-can-fail` branch, needs no change to the driver or to any oracle, and
does not put a re-run of `--arm both` in question.
