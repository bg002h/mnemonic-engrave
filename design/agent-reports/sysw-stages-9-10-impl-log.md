# Implementation log — systemwide payloads, plan stages 9 and 10

**I implemented these two stages.** This continues
`sysw-stages-7-8-impl-log.md`; stages 1–6 and 11–13 still have no log, and that
gap is still open.

- **Date:** 2026-08-12
- **Agent:** Claude Opus 5 (1M context), dispatched as the single implementer
- **Plan:** `design/IMPLEMENTATION_PLAN_systemwide_payloads.md`, stages 9 and 10
- **Spec:** `SPEC_systemwide_payloads.md` — §8c and test 22 (stage 9); §3.1,
  §3.2, §3.3.2a, §3.3.3, §5.3.1 (stage 10). §13 D3 and D5 read before deciding
  any refusal; D5's withdrawn post-engrave reminder was NOT built.
- **Commits:**
  - `seedhammer` `5adc3d7` — stage 9 (from `0775e9e`, branch `sysw-port`)
  - `mnemonic-engrave` `7136039` — stage 9's coverage flip (from `a3d0617`,
    branch `sysw-container`)
  - `seedhammer` `b991739` — stage 10
- Both trees end clean. Nothing was flashed; neither stage has a hardware step.

---

## What I changed

### Stage 9 — §8c's confirmation, and Back ≠ `done`

| file | change |
| --- | --- |
| `gui/gui.go` | `inputWordsFlow(...) (n int, done bool)`; **the `done` nav button is now DRAWN**; the four exits return their own `done` |
| `gui/sysw_load.go` | the entry↔confirmation loop: `"N words — unlock?"` with `{BACK, UNLOCK}` before the KDF; Back out of entry aborts the load |
| `gui/sysw_load_test.go` | 4 new tests (3 flow-level, driven by TOUCH; 1 at `inputWordsFlow` itself) |
| `crates/me-cli/src/sysw/coverage.rs` | test 22 `DeviceUnbuilt` → `Device` |

### Stage 10 — the consuming half of NFC

| step | file | change |
| --- | --- | --- |
| 10a | `gui/scan.go` | `text:`/`pass:` matched before the sniffers, RESERVED; new `freeTextScan` / `passScan` |
| 10b | `gui/derive_xpub.go` | the picker becomes `{TYPE IT, SCAN [, FROM PAYLOAD]}`; new `scanSeedFlow`; `seedEntryFlow` loops over the sources |
| 10c | `gui/gui.go`, `gui/freetext_flow.go`, `gui/passphrase_flow.go` | `engraveObjectFlow` routes both types; `engraveTextFlowFrom` / `engravePassphraseFlowFrom`, with the old entries as `srcTyped` wrappers |
| 10d | `gui/sysw_source.go` (new) | `syswSourceAccept` renders F3 and F4 at the 10b/10c acceptance points |
| F-126 | `gui/nfc_scan.go` (new) + 5 call sites | one `startScanner`, with an idle backoff; **253 lines of duplicated loop deleted** |
| — | `cmd/emu/platform.go` | comment only: `NFCReader` had said "returns nil: this emulator has no tag source" since before stage 6 gave it one |

Tests: `gui/nfc_scan_test.go`, `gui/sysw_source_test.go` (new); six existing
tests gained one click; `TestEngraveTextPreFillsRatherThanBypassing` rewritten
over the AST.

---

## Gates, run by me

Every command below was run with its exit code printed unconditionally
(`cmd; echo "exit $?"`), never `cmd && echo OK`.

**Go** (`seedhammer`), at `b991739`:

```
gofmt -l .                                        no files listed, exit 0
SYSW_REQUIRE_VECTORS=1 go test ./sysw/ ./gui/
  ok  seedhammer.com/sysw  (cached)
  ok  seedhammer.com/gui   53.733s
                                                  go test exit 0
go build -tags tinygo ./gui/                      tinygo build exit 0
GOOS=js GOARCH=wasm go build -o /dev/null ./cmd/emu/   wasm build exit 0
```

**Rust** (`mnemonic-engrave`), at `7136039`, whole run grepped for `FAILED` → 0:

```
lib 204 | main 1 | cli 30 | cross_lang 1 | golden 3 | preview_cross_lang 1
prop 6 | seal_cli 14 | sysw_cli 28 | doc-tests 0     — 0 failed anywhere
cargo test  exit 0
cargo clippy -p mnemonic-engrave --all-targets -- -D warnings   exit 0, clean
```

`go vet ./gui/` still fails on the PRE-EXISTING go1.26 issue in
`freetext_sizeproof_golden_test.go`. Not mine, not touched.

---

## Mutation testing

18 mutants, 18 killed, every marker printed **on the mutated line** and every
firing count recorded, so a survivor could be told from an unexecuted line.

**Stage 9 (7):** `done` returns false; Back returns true; drop the nav draw;
ignore `done` at the caller; accept any confirmation choice; re-enter at slot 1;
name `len(m)` instead of `n`. All killed. **M4 killed by HANGING** — with `done`
ignored, teardown re-enters entry forever, which is a live demonstration of why
`ctx.Done` must return `done == false`.

**Stage 10 (11):**

```
N1  no idle backoff                      KILLED  markers=197812
N2  backoff keyed on the merged result   KILLED  markers=178700
N3  reserved prefixes fall through       KILLED  markers=4
N4  pass: routed as free text            KILLED  markers=1
N5  the picker drops SCAN                KILLED  markers=4
N6  the scan seam takes any object       KILLED  markers=3
N7  a declined acceptance is admitted    KILLED  markers=2
N8  F4 not rendered                      KILLED  markers=4
N9  acceptance screen on TYPED entry     KILLED  markers=1
N10 the free-text body is dropped        KILLED  markers=2
N11 the passphrase body is dropped       KILLED  markers=2
```

---

## The thing worth reading first: `done` was never pressable

Stage 9's brief says stage 5c "delivered the button and under-transcribed the
rest". **It did not deliver the button.** `doneBtn` was constructed and its
`Clicked` handled, but it was never passed to `layoutNavigation` — and a
leftover `_ = doneBtn` (redundant the moment `Clicked` was called) was the tell.

`layoutNavigation` is what installs the touch target (`op.Input`, `gui.go:2053`),
and this panel's only production input is the capacitive one — `start_screen_touch_test.go`
says so at length. So an undrawn nav button is not a button missing an icon; it
is a button that **cannot be pressed at all**. §8c's confirmation would have been
dead code on the machine, and a passphrase could only be terminated by filling
all 24 slots or backing out.

I found it because I wrote the test as a **touch tap** on the slot rather than a
synthesised `ButtonEvent`, and `tapNavSlot` reported `nav b2: no touch target at
(454,159)`. A button-event test would have passed against unreachable code, and
coverage.rs would have gone green on a claim that was half false at both ends —
its note read "the done button exists, its confirmation screen does not".

I drew it (`Button2`, middle slot, `IconRight`, `StyleSecondary`) and said so in
the commit and in the coverage entry. **Icon and style are my choice; the plan
and spec name neither.**

---

## Things I was unsure about — read this part

### 1. `done` for the two exits the plan did not name

The plan says only "Back returns `done == false`". There are four exits. I chose:
running off the end of the slice → `true` (entry finished by itself), and the
`ctx.Done` unwind → `false`. Mutant M4 turned the second into evidence rather
than taste: with `done` ignored, teardown spins the retry loop forever. **Still
my decision, not the plan's.**

### 2. `done` pressed on an EMPTY keyboard

The plan specifies the confirmation "on `done` with `n > 0`" and says nothing
about `n == 0`. I abort the load, matching what the code did before. An empty
passphrase opens nothing and §12.2 refuses to treat one as absence.

### 3. Where BACK re-enters entry

"BACK re-enters entry with the slots intact" does not say where the cursor
lands. Re-entering at slot 1 leaves the words in the buffer but overwrites the
first on the next keystroke, so "intact" would be true of the buffer and false
of the operator's experience. I resume at the first free slot, clamped to
`len(m)-1`. Mutant N/M6 pins it.

### 4. `"1 words — unlock?"`

The plan gives the Lead verbatim as `"N words — unlock?"`. I transcribed it
literally, so a one-word passphrase reads "1 words". Fixing it would have been
inventing a rule; it is ugly and I left it. **Flagging rather than resolving.**

### 5. SCAN is offered unconditionally — and the obvious gate is a trap

The plan does not say whether the SCAN row is conditional. Gating it on
`NFCReader() != nil` is what I would have written first, and it is wrong here:
`cmd/emu`'s `reader()` **hands out the pending record and marks it consumed**, so
probing for nil to decide whether to draw a row would eat the operator's tag
before they chose anything. So SCAN is always offered, and a platform without a
reader gets a scan screen it can only Back out of — the shape `mk1GatherFlow` and
`scanAddressFlow` have always had.

### 6. This adds a screen to the common path of four programs

The picker used to appear only when a payload was loaded. It is now the first
screen of **every** seed entry in BIP-85, Account Xpub, Single-Sig and Multisig.
Six existing tests needed one extra click. §3.1 is normative and this is what it
says, but it is a real UX cost on the most-walked path in the firmware and
nobody has weighed it since the spec was written. **If an operator review wants
the picker suppressed when there is only one source, that is a spec question, not
a code one.**

I also changed Back at the picker to leave seed entry rather than fall through to
the word-count picker. The fall-through was pre-existing and invisible while the
picker was rare; as the first screen it would mean Back moves the operator one
screen *deeper*.

### 7. What the acceptance screen renders

The plan's 10d row names F3 and F4. I render exactly those and deliberately do
**not** re-render F1/F2 there, because `syswLoadWarnings` states them once at
load over every class the payload holds, and F1's erase offer belongs to the
unbuilt §5.3.2 item. The rule itself is untouched — `syswFlags` still returns all
four; this screen just does not draw two of them. **A reviewer may reasonably
want F1 repeated at the point of use; §3.3.3's F1 row does say "offers erase".**

### 8. Declining an acceptance means two different things

At the seam (10b) a decline re-offers the sources. At a top-level scan (10c)
there is no picker to return to, so it leaves the program. The plan specifies
neither.

### 9. `passScan` is `[]byte`, `freeTextScan` is `string`

The plan names the types and not their representations. The existing scan types
(`mdmkText`, `addressText`) are strings; I made the SECRET one a `[]byte` so the
passphrase program can copy it into the buffer it can scrub, per that file's own
reasoning. §6.2.2a already accepts that the scanner's own copy is unwipeable.

### 10. `syswSeedPicker` is now a misleading name

It is no longer sysw-specific — it offers typing and scanning too. I kept the
name because the plan names the function. **A rename is safe** (test 16 matches
`seedEntryFlow`, not this) and probably right.

### 11. An empty `pass:` / `text:` body is ACCEPTED

I wrote a test asserting `"pass:"` is refused as a reserved prefix, and it was
wrong: an empty body is valid lowercase hex — of zero bytes — so both `DecodeBody`
implementations accept it and `Classify` returns the class. Refusing it in the
scanner alone would have put a rule in a second place. I dropped the case and
said why in the test.

### 12. I refactored five files the plan does not name

The brief made the F-126 spin non-optional for this stage. Fixing it five times
would have left the sixth copy — the one stage 10 adds — free to reintroduce it,
so `startScanner` is one function now. It touches `bundle_flow.go`,
`md1_gather.go`, `mk1_inspect.go`, `verify_address.go` and `StartScreen.Flow`,
which stage 10's table does not list. The five bodies were byte-identical apart
from comments, and 253 lines went with them.

### 13. The plan's own journey map is now stale

J-B says "open at one step: st 9" and J-C "open: st 10"; both are built.
J-I ("open: st 7–8") has been stale since the previous implementer. I did not
edit the plan — its state column is dated, and editing the artifact I am
implementing against is not mine to do mid-stage — but the map exists precisely
so an absent step is visible, and a stale one is the same failure with a
different sign.

---

## Things I got wrong on the way

**1. My mutation runner silently did nothing, and six mutants stacked up.** The
harness shell is **fish**, where `for f in $FILES` does not word-split — so both
the backup loop and the restore loop operated on one long filename and failed.
Mutants N1–N6 accumulated on top of each other, and the reported kills for N3–N6
were partly other mutants' failures. I caught it from `cp: cannot stat 'gui/... gui/...'`
in the output and from unrelated tests failing, reverted every mutant by
inverse-replacement, re-verified the tree built and passed, and rewrote the
runner in Python. **The whole stage-10 mutation table above is from the clean
re-run.**

**2. N4's marker could never fire.** I mutated `if bytes.HasPrefix(...)` to
`if false { println("MUTANT-N4 FIRED"); ... }` — the mutation removed its own
execution, which is exactly the failure mode the marker discipline exists to
catch. `markers=0` caught it. Re-cut as a mis-route on a line that runs.

**3. Two mutants SURVIVED, and both were my tests' fault, not the mutants'.**
N5 relabelled the row `"SCAN-UNDRAWN"` — and `uiContains(content, "SCAN")`
matches that, so a loose needle passed. N10 dropped the free-text body in
`engraveObjectFlow` — and my pre-fill tests called `engraveTextFlowFrom`
*directly*, leaving the ROUTE untested. I rewired both pre-fill tests to enter
through `engraveObjectFlow`, which is how a scan actually arrives, and re-cut N5
to drop the row outright.

**4. I over-claimed in a code comment, then corrected it before it landed.** I
wrote that the spin "freezes the emulator outright, which is why the NFC path has
never been walkable in a browser" — inherited from F-126, not measured by me. I
tried to reproduce it: built the wasm, served it, presented a tag with
`shNFC.present(...)`, measured ~60 fps; then rebuilt with the backoff deleted
and measured ~60 fps again. **The experiment never reached the loop**: a screen
fetches `Platform.NFCReader()` once at entry and `cmd/emu`'s source returns nil
until a record is pending, so a tag presented while a screen is already open is
invisible to it. Both comments now state only what I measured (4 reads/150 ms vs
~198,000 iterations) and mark the device and browser consequences as F-126's
claims. Amended into the stage-10 commit rather than stacked on top.

**5. Two test cases I wrote were simply wrong and the runs caught them.** A
`text:` case I annotated "odd nibble count" had 22 hex characters and decoded
fine; and my first delivery test read the *head* of the scans channel, which is
the in-progress report with a nil object.

---

## What I did NOT do

- **Stages 11, 12, 13.** Out of scope, untouched.
- **`seal/`, `gui/unlock_kdf.go`, `unlockPayload`.** Untouched; §13 D8's
  visibility-only exception was not needed and not used.
- **`gui/sysw_admit.go`.** Unchanged, as stage 10 says: the rules were already
  there, and this stage only made them fire.
- **The post-engrave reminder (§13 D5).** Withdrawn; not built.
- **Anything on hardware.** Neither stage was flashed.
- **J-C in a browser.** Not verified — see mistake 4. What I can say is that
  `shNFC.present(...)` must happen **before** the scan screen is entered, because
  of the reader-at-entry shape above. That is a limitation of stage 6's source,
  not of stage 10, and the plan's stage-10 green note ("then `shNFC.present(…)`
  walks J-C in the browser") reads as though order does not matter. It does.
- **Firmware size and load-time cost.** `go build -tags tinygo` is a type-check,
  not the firmware build; I did not compile with TinyGo or compare image sizes.
  `startScanner`'s backoff adds a 50 ms latency ceiling to first detection on
  every scanning screen, which I did not measure on hardware.
