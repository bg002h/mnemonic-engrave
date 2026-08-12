# Implementation log — systemwide payloads, plan stages 12, 11, 13 and ruling D9

**I implemented these.** This continues `sysw-stages-7-8-impl-log.md` and
`sysw-stages-9-10-impl-log.md`. With this the plan's thirteen stages are all
built and logged except 1–6, which still have no log; that gap stays open.

- **Date:** 2026-08-12
- **Agent:** Claude Opus 5 (1M context), dispatched as the single implementer
- **Plan:** `design/IMPLEMENTATION_PLAN_systemwide_payloads.md`, stages 12, 11,
  13 in that order, plus §13 D9
- **Spec:** `SPEC_systemwide_payloads.md` — §7/§7.1.1/§7.2/§7.2.1/§7.4 (st 12);
  §12.2, §12.4, §3.2.1, §3.3.3 (st 11); §3.3.2/§3.3.2a, §13 D7 (st 13); §3.1
  (D9). §13 D3, D5, D8, D9, D10 read before deciding any refusal, screen or
  wording. D5's withdrawn reminder was NOT built; D8's visibility carve-out was
  not needed and not used.
- **Commits** (from `mnemonic-engrave` `8ad23f2` / `seedhammer` `b991739`):
  - `seedhammer` `dd02aba` — §13 D9, the source picker
  - `seedhammer` `3c7ef70` — stage 12, the word-plate verify
  - `mnemonic-engrave` `68e6781` — stage 12's coverage flip (2, 3, 17)
  - `seedhammer` `2f498b8` — stage 11, UNLOAD
  - `seedhammer` `85aa4cc` — stage 13, carrier-ready cells + the reconciliation
    tests
- Both trees end clean. **Nothing was flashed.** No stage had a hardware step —
  which since §13 D10 is true of stage 11 as well, and that is the ruling's
  whole point.

---

## What I changed

### D9 — the source picker only when a choice exists (`dd02aba`)

| file | change |
| --- | --- |
| `gui/derive_xpub.go` | `syswSeedPicker` builds its rows first and skips the screen when only one survives; the SCAN row is gated on the reader |
| `gui/gui.go` | `FeatureNFC` |
| `cmd/controller/platform_sh2.go` | sets it (the ST25R3916 is on every board) |
| `cmd/emu/platform.go` | reports it — `nfcSource` is a real source and §8.2 needs J-C walkable in a browser |
| 4 test files | the six clicks stage 10 added are REMOVED, not left inert |

### Stage 12 — §7's word-plate verify (`3c7ef70`, `68e6781`)

| file | change |
| --- | --- |
| `gui/plate_verify.go` | **new.** `verifyProvenance` + its four §7.1.1 strings; §7.2's menu; §7.2.1's selection incl. the CSPRNG draw; the typing, mismatch, assertion and outcome screens |
| `gui/gui.go` | `backupWalletFlow` offers it after a completed engrave |
| `gui/plate_verify_test.go` | **new.** 11 tests |
| `crates/me-cli/src/sysw/coverage.rs` | 2, 3, 17 `DeviceUnbuilt` → `Device`; 1's entry names its mechanism |

### Stage 11 — UNLOAD (`2f498b8`)

| file | change |
| --- | --- |
| `gui/sysw_unload.go` | **new.** `syswPayloadMenu`, `syswUnloadFlow`, `syswReloadCost` |
| `gui/sysw_session.go` | `digestShown`, and `load`'s signature |
| `gui/sysw_load.go` | sets it; F1's offer; `syswHasFlag` |
| `gui/gui.go` | the `loadPayload` carousel arm |
| `gui/sysw_source.go` | a stale comment corrected |
| `gui/sysw_unload_test.go` | **new.** 7 tests |

### Stage 13 — cells and reconciliation (`85aa4cc`)

| file | change |
| --- | --- |
| `gui/gui.go` | 13a: `newInputFlow` offers `ClassCodex32Secret` |
| `gui/sysw_source.go` | 13b: `syswPassphraseFlow` |
| `gui/derive_xpub.go`, `bip85.go`, `singlesig.go`, `multisig.go`, `multisig_build.go` | the five seam sites call it |
| `gui/multisig.go`, `gui/multisig_build.go` | 13c: the MDMK offer before the gather |
| `gui/singlesig_verify.go`, `gui/multisig_verify.go` | comments recording why they get NO offer |
| `gui/sysw_admit_oracle_test.go`, `gui/sysw_coverage_witness_test.go`, `gui/sysw_cells_test.go` | **new.** 13d plus the behavioural cell tests |

---

## Gates, run by me

Every command below was run with its exit code printed **unconditionally**
(`cmd; echo "exit $?"`), never `cmd && echo OK`. **A note on that:** the harness
shell is **fish**, where `${PIPESTATUS[0]}` expands to nothing — the previous
implementer was bitten by fish's word-splitting and this is the same trap one
door along. Every pipeline exit code below was captured inside an explicit
`bash -c` or inside `nix develop --command bash -c`, and I checked that the
number actually printed.

**Go** (`seedhammer`), at `85aa4cc`:

```
gofmt -l .                                     no files listed, exit 0
SYSW_REQUIRE_VECTORS=1 go test ./sysw/ ./gui/
  ok  seedhammer.com/gui   67.815s
  ok  seedhammer.com/sysw                      go test exit 0
go build -tags tinygo ./gui/                   exit 0
GOOS=js GOARCH=wasm go build ./cmd/emu/        exit 0
grep -rn "Erase\|erase" gui/ sysw/ | grep -v _test
                                               exit 0; 7 hits, ALL comments or
                                               the frozen seal program's
                                               pre-existing RAM-wipe wording.
                                               No flash-write path.
```

**A gate the previous two logs did not run, and I did:** the real firmware
build. `.github/workflows/test.yml` runs it, and `go build -tags tinygo ./gui/`
is only a type-check — both prior logs said so under "what I did not measure".

```
tinygo build -size short -target pico-plus2 -stack-size 16kb -gc precise \
  -opt 2 -scheduler tasks ./cmd/controller        exit 0

           flash      ram
b991739  1336140    61836     (baseline, before my work)
dd02aba  1336252    61836     D9        +112 / +0
3c7ef70  1339732    61892     stage 12  +3480 / +56
2f498b8  1340780    61892     stage 11  +1048 / +0
85aa4cc  1341040    61892     stage 13   +260 / +0
                              TOTAL     +4900 flash, +56 ram
```

It also covers `cmd/controller/platform_sh2.go`, which is `//go:build tinygo &&
rp` and which **no other gate in this repo compiles**. D9 edits that file.

**Rust** (`mnemonic-engrave`), at `68e6781`, whole run grepped for `FAILED` → 0:

```
lib 204 | main 1 | cli 30 | cross_lang 1 | golden 3 | preview_cross_lang 1
prop 6 | seal_cli 14 | sysw_cli 28 | doc-tests 0     — 0 failed anywhere
cargo test  exit 0
cargo clippy -p mnemonic-engrave --all-targets -- -D warnings   exit 0, clean
```

**The `DeviceUnbuilt` list is now EMPTY** — measured, not asserted:

```
cargo test -p mnemonic-engrave --lib sysw::coverage -- --nocapture
  spec §8.3 tests whose behaviour does NOT exist yet:
  withdrawn by ruling:
    13  operator ruling 2026-08-12: the reminder is dropped
```

`go vet ./gui/` still fails on the PRE-EXISTING go1.26 issue in
`freetext_sizeproof_golden_test.go`. Not mine, not touched. I did not touch the
spec, so `scripts/spec-check.py` was not required.

---

## Mutation testing

**38 mutants across the four commits, 37 killed, 1 surviving.** Every mutant
carried a marker on the mutated line and the firing count is recorded, so a
survivor can be told apart from a line that never ran.

| set | mutants | killed | notes |
| --- | --- | --- | --- |
| D9 | 4 | 4 | markers 1, 3, 3, 3 |
| stage 12 | 15 | 14 | one true survivor, below |
| stage 11 | 15 | 15 | two rounds; two survivors in round 1 became two new tests |
| stage 13 | 8 | 8 | markers 1, 1, 1, 1, 1, 1, 15 |

**My runner was wrong in a way that would have made every survivor look like an
unexecuted line.** `go test` DISCARDS the output of passing tests without `-v`,
so a surviving mutant's `println` marker is invisible — and my first stage-12
round reported three mutants as "KILLED/SURVIVED, marker never fired" when two
of them had executed thousands of times. I found it because P8 could not
possibly have failed to run. The runner now passes `-v` and every count above is
from a re-run. **This is exactly the failure the marker discipline exists to
catch, arriving through the harness rather than through the mutation.**

### The survivor, stated plainly

**P8: replacing `randIntn`'s rejection sampling with a bare `% n` SURVIVES**, with
the marker firing 60 050 times. The modulo bias over 24 positions is around
2⁻²⁷ relative and 20 000 draws cannot see it. The rejection sampling is correct
by construction and **not covered by any test here**; a test that could see it
would need more samples than a suite should run. Not an equivalent mutant — a
real behaviour difference no affordable test can observe.

### Four mutants that found missing tests

Each survived a first round and is now covered:

1. **Back at the §7.2 depth menu becoming a device comparison.** Nothing drove
   Back there. It is the highest-value outcome to get wrong: it silently
   certifies a plate nobody checked.
2. **A declined by-eye assertion recorded as an assertion anyway.** Every test
   took the accepting branch.
3. **`digestShown` hard-coded true at the load site.** The unload wording tests
   build the session by hand, so the WIRING was untested. Now driven end to end
   over the S-D region image — the only test in the tree that opens a sealed
   systemwide payload for real.
4. **`syswHasFlag` answering for any flag.** The F1 payload also raises F3, so an
   offer keyed on the wrong flag still appeared. S-E is the discriminator.

### One mutant kills without executing, correctly

**P9** (the `plateVerifyFlow` call removed from `backupWalletFlow`) is killed by
an AST test, so its marker reads 0. That is right for a structural guard — and
it also measures something real: **no test in this tree drives
`backupWalletFlow` through a COMPLETED engrave**, so the stage-12 integration is
pinned structurally and never behaviourally. Pre-existing gap, not one I made,
but it is why that number is 0.

---

## The defect I nearly shipped

**The `pass:` record body is HEX-ENCODED, and my first `syswPassphraseFlow`
returned it raw.** §5.3.1 reserves the prefix and encodes the body because EPD
§6.4 forbids the interior spaces a passphrase may contain. Returning the record
verbatim would have handed Account Xpub, BIP-85, Single-Sig and Multisig the
literal string `pass:6162…` as a BIP-39 passphrase — **deriving a different
wallet, silently, with no screen able to show the difference.**

I found it only because a test fixture refused to classify: I wrote
`sessionHolding("abandon about")` expecting `ClassPassphrase` and got
`ClassUnknown`. Both existing sites (`engravePassphraseFlowFrom`,
`engraveTextFlowFrom`) decode; mine was the third and the trap is identical at
all three. **A one-line `sysw.DecodeBody` away, and nothing structural would
have caught it** — the oracle test checks the CLASS a site names, not what it
does with the body.

---

## The plan is WRONG at stage 13b, not ambiguous

Stage 13b: *"`passphraseFlow` gains `syswOffer(sysw.ClassPassphrase, "Password
from where?")` at its head — the optional-passphrase step the spec's cell
reasons name, one edit serving all four callers."*

**Measured with grep before writing anything: `passphraseFlow` has TEN non-test
callers, not four.** Transcribed literally the edit breaks two NORMATIVE rules:

- **§3.3.2** — Backup Wallet REFUSES `ClassPassphrase`, and `backupWalletFlow`
  calls `passphraseFlow` (`gui.go`) for its fingerprint choice. The spec gives
  the reason: it engraves the mnemonic itself and the passphrase is deliberately
  never engraved and never in the QR, so a passphrase reaching it has nowhere to
  go.
- **§7.4** — `singleSigVerifyFlow` and `multisigVerifyFlow` call it too. A verify
  taking half its re-derivation input from the session is the cache answering a
  verification prompt on the operator's behalf. **Test 16 could not have caught
  it**: it scans `*_verify.go` for `seedEntryFlow`, and the payload access would
  have been inside `passphraseFlow`, in `gui.go`.

The tenth caller, `slip39_polish.go`, takes a SLIP-39 passphrase — a different
secret, and its own screen says so at length.

I could not implement it as written without breaking the spec, and I could not
skip it without leaving the stage unbuilt. **So I changed the MECHANISM and kept
the intent**: `syswPassphraseFlow` is a wrapper called by the five sites in the
four admitting programs, `passphraseFlow` is untouched, and
`TestTheSeamPassphraseOfferReachesOnlyProgramsThatAdmitIt` names the four
forbidden callers with the rule each would break. **This is a departure and the
reviewer should treat it as one.** The plan's sentence — including its count of
four — should be corrected rather than left to catch the next reader.

---

## Things I was unsure about — read this part

### 1. D9 does nothing on real hardware, and I think that is what it says

D9 offers the picker when "a loaded payload, or a tag source" exists. The
SeedHammer II has an ST25R3916 soldered to every board, so `FeatureNFC` is
always set and the picker always appears there with two REAL options. The click
D9 was ruled against is only removed on machines with no reader.

That reading is the letter of the ruling, and the ruling explicitly names a tag
source as one of the two things that make a choice exist — so I do not think it
is an accident. **But if the operator's actual complaint was the extra click on
the most-walked path, D9 as written does not fix it on the machine they use.**
The alternative reading — gate on the payload alone — is a one-word change here.
Flagging rather than resolving.

### 2. `FeatureNFC` is a new Platform-visible capability, and it is mine

The plan and spec name no mechanism. `NFCReader() != nil` is a trap the stage-10
log already recorded (the emulator's `reader()` consumes the pending tag), so a
free question needed a free answer. A `Features` bit was the smallest one —
`Features()` already exists and `FeatureSecureBoot` already drives UI. **The
emulator reporting it is a deliberate call**: `nfcSource` is a real source and
reporting no reader would have taken SCAN off every seed entry in the browser,
leaving §8.2's J-C unwalkable by the tool that exists to walk it.

### 3. §7.2's table has six rows and my menu has seven

`even words` and `odd words` share one table line and are two different
selections (§7.2.1 defines each separately). A single row meaning both is not
something an operator can choose, so I split them. The plan says "all six rows".
**I read that as a count of table lines, not of menu entries.**

### 4. Pre-filling the untested slots is my mechanism

`inputWordsFlow` steps to the next EMPTY slot and draws that slot's index, so
filling every non-drawn position makes the prompt read "Word 17 of 24" — the
PLATE's position. The alternative (a length-`k` scratch buffer) asks for "Word 3
of 3" and needs the mapping shown somewhere else. Neither plan nor spec says
which. The filler word is never compared; mutant P6 (fill with the plate's own
words) is killed.

### 5. The checksum gate is off even for `every word`

The plan says `checksumGate false — a subset has no checksum`, which does not
cover the full-word case. I keep it off there too, and for a stronger reason
than the plan's: with the gate ON the keyboard MASKS the last word to
checksum-valid candidates, which would **stop the operator typing the wrong last
word off a mis-cut plate** — hiding the exact defect §7 exists to find. My
reasoning, not the plan's.

### 6. Back at the depth menu is `not verified`

§7.1.1 has no fifth provenance for "the operator left", and §7.1 already calls
bypass a menu option. **My decision.** A mutant made it evidence rather than
taste: recording it as a device comparison certifies an unchecked plate.

### 7. The outcome screen wording beyond §7.1.1's four strings

`showNotice` titled "Verify Plate", the mismatch screen's `"Words 4, 9 did not
match the plate."`, and the assertion screen's `"Did the plate match, by eye?
The device did not check it."` are **mine**. The spec fixes only the four
provenance strings and I render those verbatim.

### 8. The carousel entry keeps LOAD AGAIN, which the plan does not mention

Stage 11 says the unload is "offered from the `loadPayload` carousel entry when
a payload is LOADED (UNLOAD / BACK)". Taken literally that removes the ability
to re-read the region while a payload is loaded — which is **journey J-E**, a
journey the plan's own map records as closing. So the entry offers
`{LOAD AGAIN, UNLOAD}`. **My decision**, and the one place I added a screen the
plan does not name.

### 9. `digestShown` is a session field the plan does not name

The confirm screen has to distinguish `pub_len == 0` from a sealed payload with
a digest, and the record list cannot answer that. `[digest-shown]` (§12.4) is
referenced by name; the field is a fact about the loaded payload, not a second
statement of the rule. It changed `load()`'s signature and seven test call
sites.

### 10. No KDF timing appears on any screen

The brief describes the sealed case as "a full passphrase entry plus its ~31 s
KDF". **I did not put a number on screen.** The plan gives the two sentences
verbatim and neither carries one, and the sysw container's iteration count comes
from the payload's own header — so a fixed number would be a claim about someone
else's file. The ~31 s figure is Sealed Payload's, measured for a different
container.

### 11. The F1 offer adds a screen to the load path

§3.3.3's F1 row says "offers erase". What it offers now is UNLOAD, as a
`{KEEP, UNLOAD}` choice after the warnings screen. The plan says "alongside the
F1 warning" and does not say what shape. `confirmReviewScreen` returns a bool
and cannot carry three outcomes, so it is a second screen. **A reviewer may
prefer it folded into the warning screen; that needs a different widget.**

### 12. Stage 13a offers `"Seed from where?"` twice

The plan gives 13a's lead verbatim, and it is the same string as the mnemonic
offer immediately above it. An operator whose payload holds **both** a seed and
an ms1, and who declines the first, sees the same sentence twice. I transcribed
it literally rather than inventing a second wording. Ugly; flagged.

### 13. The new `syswOffer` sites render no F3 acceptance screen

Consistent with the two existing `syswOffer` sites (`newInputFlow`,
`bundleFlow`) and inconsistent with the other two (`engraveTextFlowFrom`,
`engravePassphraseFlowFrom`) — which have one because they ALSO accept `srcNFC`
from `engraveObjectFlow`, where no offer screen exists. My reading: `syswOffer`'s
own screen names the source (the operator picks `FROM PAYLOAD`), so §3.2's
point-of-entry requirement is met. **If a reviewer disagrees, four sites need
the screen, not one.**

### 14. What the oracle test does NOT check

It reconciles **sites against `admits()`**, which §13 D7 makes the table's
transcription. It does **not** check the table against §3.3.2 — mutant T6
widened the table and the oracle passed (the cell tests in
`sysw_admit_test.go` killed it). Stated in the test's own comment so nobody
reads more into a green run than it earns.

### 15. Two spec tests were `Device` in `coverage.rs` with nothing discharging them

Writing 13d's witness test turned up **10** (compared once per payload) and
**21** (the passphrase buffer never regrows) claimed as device-built with no Go
test naming them — and 21's only relative in the tree belongs to the FROZEN seal
unlock, a different container's code path. I wrote both rather than demoting the
entries. **That is the witness test earning its keep on its first run**, and it
also means those two entries were the same kind of claim the `DeviceUnbuilt`
split was introduced to stop.

### 16. The plan's journey map is stale, for the third log running

J-B says "open at one step: st 9", J-C "open: st 10", J-I "open: st 7–8", J-D
"device open: st 11", J-G "open: st 12", J-H "partly open: st 13a–c". **All are
now built.** The previous implementer flagged this and did not edit the plan;
so have I, for the same reason. But a map whose entire purpose is to make an
absent step visible, wrong in six rows across three cycles, is the same failure
with the sign flipped. **One editing pass over the state column would close it.**

---

## Things I got wrong on the way

**1. My mutation runner hid every survivor's marker.** Covered above: `go test`
without `-v` discards passing tests' output. Three mutants were reported with
"marker never fired" when they had fired tens of thousands of times. Every
number in this log is from the re-run.

**2. I cut a mutant that did not compile and read the compile error as a kill.**
P8's first form removed the only use of `limit`, so the mutant "died" of
`declared and not used` rather than of any test noticing. Re-cut so it compiles;
it then survived, which is the honest answer.

**3. I invented a 24-word mnemonic fixture.** It has no valid checksum and
`ParseMnemonic` refused it, failing six tests at once. Replaced with
`bip39.New(sha256("\x00"*8))`, found by search — and the search itself was wrong
first time: I varied only the first 8 bytes of the entropy, so every candidate
was twenty-three `abandon`s and a checksum word, and the all-distinct condition
could never be met.

**4. My first `typePromptedWords` reported "0 words prompted" for a working
flow.** It broke at the first frame that was not a word prompt, and the frame
right after a queued click is the menu being redrawn. Two tests failed for a
reason that had nothing to do with the code under test.

**5. I nearly shipped the undecoded `pass:` body.** Covered above. Found by a
fixture, not by a test I designed to find it.

**6. I forgot two imports.** `seedhammer.com/sysw` in `multisig.go` and
`multisig_build.go`. Caught by the build in seconds — recorded only because
"the build caught it" is the reason the build runs before anything else does.

---

## What I did NOT do

- **`seal/`, `gui/unlock_kdf.go`, `unlockPayload`.** Untouched. §13 D8's
  visibility carve-out was not needed and not used.
- **Any flash write, `Eraser`, `SyswEraser()` or `erase_*.go`.** §13 D10.
  `TestNoErasePathExistsOnTheDevice` makes the plan's green grep a test that
  runs on every suite, over the AST so a comment explaining the absence does not
  trip it.
- **The post-engrave reminder (§13 D5).** Withdrawn; not built.
- **The two OPEN cells** the plan records: `Cdx32 → the seam` (§3.1's seam type
  cannot carry it) and `MDMK → Single-Sig` (no supplied-md1 carrier in that
  program). Both fail closed, as the plan says they must.
- **The spec and the plan.** Not edited. See uncertainties 16 and the 13b
  departure — both want an edit that is not mine to make mid-implementation.
- **Anything on hardware.** No stage had a hardware step. The firmware BUILDS
  (+4900 bytes flash, +56 ram over the whole cycle) and has not been flashed or
  run.
- **J-G in a browser.** The wasm build passes; I did not walk the verify menu in
  the emulator.
- **A behavioural test of stage 12's integration.** No test drives
  `backupWalletFlow` through a completed engrave, so the call is pinned by AST
  only. Recorded above as mutant P9's zero marker count.
