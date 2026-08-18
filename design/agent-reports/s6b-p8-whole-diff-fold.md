# S6b P8 — fold of the whole-diff adversarial review (C1, C2, I1, I2)

**Scope:** `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`, tip
`12a428d6fe250fa235cc0dbc7943cd74084a7ba1` (unchanged — this fold is
uncommitted working-tree state for the controller to commit). Reviewer input:
`design/agent-reports/s6b-whole-diff-review.md`, verbatim, RED `2C/2I`.

Every finding below was **reproduced independently** before any fix was
written — none of the four fixes were applied from the review's prose without
first seeing the real defect fail a real test against the real code. Two of
the review's four proposed fixes did not work as literally written; both are
called out below rather than silently patched around.

---

## C1 — preloaded passphrase editable, never re-checked

**Reproduced.** `gui/s6b_passphrase_plate_test.go`,
`TestPreloadedEntryEditIsRefusedNotEngraved`: drives
`engravePassphraseFlowPreloaded(body="hunter2", …)` through touch, accepts the
`srcDerived` acceptance screen, backspaces one character and retypes a
different one ("hunter2" → "hunter3" — the "fix a typo that wasn't there"
gesture the finding describes), taps OK, and — via `passphrasePlateHook`, the
seam that observes exactly what `ppBuildPlate` is handed — asserts the bytes
that would reach the plate equal `body`, the passphrase the wallet was
actually derived with.

**Failing output (before the fix, code at HEAD):**
```
s6b_passphrase_plate_test.go:569: never reached "changed"; last frame
"AQRisamachine-readablecopyofthepassphrase.NoQRAddQRQRCode"
```
The edited value proceeded straight to the QR screen with no refusal —
exactly the defect: an unchecked edit would have reached `ppBuildPlate` and
been stamped `DERIVED` under the original derivation's fingerprints.

**Fix** (`gui/passphrase_flow.go`, `engravePassphraseFlowPreloaded`,
`ppPLStepEntry` case): after `passphraseEntryFlow` returns, compare
`secret[:m]` against `body` with `bytes.Equal`. On mismatch: show
`"The passphrase was changed. A passphrase plate must record the exact
passphrase this wallet was derived with."`, reload the buffer from `body`
(`n = copy(secret, body)`), and stay on the entry step (`step--` before
`break`, netting to unchanged after the loop's `step++` — the same idiom the
function already uses for its other "go back" transitions). ~14 lines.

**Passing output (after):** `TestPreloadedEntryEditIsRefusedNotEngraved` —
PASS. Full sequence verified: edit → refusal shown → dismiss → buffer reads
back the TRUE passphrase (`7/100`, not the edit) → accept unedited → QR →
Confirm → Engrave → `passphrasePlateHook` fires with exactly `body`.

**What the operator sees:** a truthful, non-stranding modal —
*"The passphrase was changed. A passphrase plate must record the exact
passphrase this wallet was derived with."* — dismissed with the same OK
button as every other error screen. The entry screen then re-shows the
correct, original passphrase (not blank, not the edit); pressing OK again
proceeds normally. R-C is preserved: the operator never has to *retype* the
passphrase, they only cannot silently *diverge* from it.

**Proposed fix vs. what shipped:** the review's snippet was applied
essentially as proposed (byte-compare, reseed, stay on step); no rejection
here.

---

## C2 — silent truncation of an over-length wallet passphrase

**Reproduced.** `gui/s6b_passphrase_plate_test.go`,
`TestPassphrasePlateOfferRefusesOverLengthPassphrase`: derives a real
single-sig bundle with a 101-character wallet passphrase
(`strings.Repeat("a", passphrase.MaxLen+1)`), then drives
`singleSigPassphrasePlateOffer` directly and asserts a refusal reaches the
operator, the function returns `passphrasePlateNotCut`, and neither the
~31 s bare-seed KDF nor `engravePassphraseFlowPreloaded`'s own
(truncating) buffer ever runs.

**Failing output (before the fix):**
```
s6b_passphrase_plate_test.go:640: no refusal reached the operator; got
"Engraveapassphraseplate?SkipEngravePassphrasePlate"
```
The offer appeared unconditionally for a passphrase that cannot fit a
plate — exactly the defect: accepting it would have truncated the
passphrase while stamping the plate with the *full* passphrase's
fingerprints, `DERIVED`.

**Proposed fix REJECTED as literally written, and why.** The review's
one-liner was
`if err := passphrase.ValidatePassphrase(passphrase); err != nil { … }`
inside `singleSigPassphrasePlateOffer`. **That does not compile.** The
function's own parameter is `passphrase string` (`gui/singlesig.go:318`
pre-fix), which shadows the imported `passphrase` package for the function's
entire body — `passphrase.ValidatePassphrase` resolves to a method call on a
`string`, not a package call. This is the exact same class of defect C1/C2
exist to catch in the *plate*, now found in the *review's own proposed code*
by actually trying to build it, per the standing "machine-checkable claims
get machine-checked" rule.

**Fix that shipped** (`gui/singlesig.go`): renamed the parameter
`passphrase string` → `pass string` (2 identifier usages in the function
body + the 2 doc-comment lines that literally quoted the old condition in
Go-double-quote form; the doc line quoting GATE 2.6's own spec text in
single-quote form was left untouched, since it quotes the spec, not the
identifier). This has zero effect on the call site (`gui/singlesig.go:224`
passes positionally). Then, immediately after the existing
`if pass == "" { return passphrasePlateNotCut }` guard and *before* the
`ChoiceScreen` offer is even shown:
```go
if err := passphrase.ValidatePassphrase(pass); err != nil {
    showError(ctx, th, "Passphrase Plate", ppEntryError(err))
    return passphrasePlateNotCut
}
```
Reuses `ppEntryError` (existing constant-string mapping, no secret ever
touches an error message) rather than inventing new wording. ~30 lines incl.
comments; no new abstractions.

**Passing output (after):**
`TestPassphrasePlateOfferRefusesOverLengthPassphrase` — PASS. Confirmed:
`result == passphrasePlateNotCut`, `bareSeedCalls == 0`, `secretCalls == 0`
(the truncating buffer in `engravePassphraseFlowPreloaded` is never
allocated for this passphrase).

**What the operator sees:** *"Too long. At most 100 characters fit on one
plate."* under the title "Passphrase Plate" — the "Engrave a passphrase
plate?" offer never appears at all, since there is nothing a Skip/Engrave
choice could honestly mean for a passphrase that cannot fit a plate. After
dismissing, the flow proceeds exactly as a decline would: verify offer (if
not already done) and the restore document, which correctly states a
passphrase was used but no plate was cut.

---

## I1 — hidden arrow's stale press auto-repeats with no finger down

**Reproduced.** `gui/scroll_arrows_test.go`,
`TestI1StaleArrowPressGhostRepeatsWithNoFinger`: drives `Warning.Layout`
directly with **real `PointerEvent`s routed against the real per-frame
`op.Drawer`** (not `click(&ctx.Router, Down)` — a synthesized `ButtonEvent`
never touches `EventRouter.Events`' tag-bounds bookkeeping at all, which is
exactly why the existing button-driven `TestGate51ArrowActuallyScrolls`
cannot see this bug). Sequence: press-and-hold the down arrow while visible →
force `w.scroll` to the real max (`TestGate51DownArrowAbsentAtFullScroll`'s
own technique) so the arrow hides mid-press → the eventual release, routed
against the frame where the arrow's hit region is already gone, is silently
dropped by the router → `synctest`-advance 2 real seconds → scroll back to 0
with **no further pointer event** → assert `w.scroll` is still 0.

**Failing output (before the fix):**
```
scroll_arrows_test.go:395: the down arrow scrolled to 91 with NO finger on
the panel -- a stale Pressed state from the earlier, dropped release
auto-repeated (I1, s6b-whole-diff-review.md)
```
0→91 matches the review's own reported repro number digit-for-digit.

**Proposed fix EXTENDED — the review's two-line fix left a second, real
defect, found and closed here.** Applying exactly the proposed fix (`if
!showUp { w.arrowUp.Pressed = false }` / same for `arrowDown`) makes the
ghost-repeat test above pass. But `Clickable.Next`'s auto-repeat
(`gui/widget.go`) reads `c.repeat` — the next scheduled wakeup — the moment
`Pressed` next becomes true, **regardless of whether the current call is what
set Pressed**. Clearing only `Pressed` left `c.repeat` holding the *original*
hold's wakeup, long past by the time real time has elapsed for the arrow to
recover. A dedicated adversarial test,
`TestI1FreshTapAfterRecoveryScrollsExactlyOnce`, drives the identical
recovery sequence and then a single, ordinary, unhurried tap (press, 50 ms
dwell, release — not a hold) on the recovered arrow:

```
scroll_arrows_test.go:488: one tap on the down arrow scrolled to 270, want
exactly 135 (one step, maxScroll=9073 so this is not the clamp) -- a stale
Clickable.repeat from the earlier recovered hold made this fresh tap
register as an overdue auto-repeat as well as a click
```
(A short-body fixture cannot show this — a single step already exceeds a
short body's `maxScroll` and both a single- and double-fire clamp to the same
value; the test uses `modalFiller(20000)` and measures the real `maxScroll`
first, guarded by an INCONCLUSIVE check, so 270 vs 135 is a genuine
double-fire, not a clamp artifact.) **One ordinary tap scrolled the panel
twice.** The review's own text ("verified green against all 5.1 gates —
reproduce it before trusting it") did not claim to have checked this path,
and it does not hold up.

**Fix that shipped** (`gui/gui.go`, `Warning.Layout`): reset **both** fields
whenever a direction hides —
```go
if !showUp {
    w.arrowUp.Pressed = false
    w.arrowUp.repeat = time.Time{}
}
if !showDown {
    w.arrowDown.Pressed = false
    w.arrowDown.repeat = time.Time{}
}
```
This is exactly the invariant `Clickable.Next` itself restores after a real
release (`if !c.Pressed { c.repeat = time.Time{} }`); the two-line addition
just applies it at the one place `Next` cannot reach, because `Next` is never
called while the direction is hidden.

**Passing output (after, both tests):**
```
--- PASS: TestI1StaleArrowPressGhostRepeatsWithNoFinger (0.01s)
--- PASS: TestI1FreshTapAfterRecoveryScrollsExactlyOnce (0.31s)
```

**What the operator sees:** hold-scrolling to the bottom of any overflowing
safety modal, then releasing, then later scrolling back up — the arrows
behave exactly as a first-time reader would expect: no unexplained scroll
with no finger on the panel, and a plain tap on a recovered arrow moves the
content by exactly one step, never two.

---

## I2 — GATE 2.3e never became a test

**No code defect existed here** — the review's own audit already confirmed
the two surfaces agree today "only because each caller passes literals," and
nothing in this fold found that untrue. The finding is the *absence* of a
test pinning that agreement, so there is no RED-code / GREEN-code pair to
report; instead this section reports how the new gate was proven non-vacuous.

**Proposed fix REJECTED as literally written, and why.** The review specifies
"drive `ppConfirmBody` and `passphraseLayoutFor` from the same inputs."
`ppConfirmBody` lives in package `gui`; `passphraseLayoutFor` is
**unexported** in package `backup`. `gui` imports `backup` (not the reverse),
so no single test function can call both as named — the proposed test cannot
compile in either package. This is the same class of issue as C2's
parameter-shadowing: a fix specified without checking it against the real
package boundary.

**Fix that shipped:**
1. `backup/passphrase.go`: renamed `passphraseFooterFor` → `PassphraseFooterFor`
   (exported), updated its one call site in `passphraseLayoutFor`. **No
   behaviour change** — same body, same single caller, verified by diff.
2. `gui/s6b_passphrase_plate_test.go`,
   `TestGate23eConfirmProvenanceAgreesWithFooter`: for `derived ∈ {false,
   true}`, calls the REAL `ppConfirmBody` (gui) and the REAL
   `backup.PassphraseFooterFor` (backup) with the same `derived` flag and the
   same seedFP/combinedFP/policyID, extracts the confirm screen's rendered
   text via `op.Drawer.ExtractText`, and asserts both surfaces name the same
   provenance (`"derived by this device"` / `"typed, not verified"` vs.
   `"DERIVED"` / `"TYPED"`).

**Mutation-tested the gate itself** (the finding here has no natural RED, so
the gate's own sensitivity had to be checked directly, not inherited):
flipped `PassphraseFooterFor`'s `if !plate.Derived` to `if plate.Derived`
(inverting which form each provenance renders) and reran both the new test
and `backup`'s own `TestPassphraseFooterProvenance`:
```
--- FAIL: TestGate23eConfirmProvenanceAgreesWithFooter (0.02s)
    derived=false -- confirm screen says derived=false (...) but the footer
    says derived=true ("POLICY 1A2B 3C4D  DERIVED") -- they disagree
    derived=true -- confirm screen says derived=true (...) but the footer
    says derived=false ("FINGERPRINTS TYPED, NOT VERIFIED") -- they disagree
--- FAIL: TestPassphraseFooterProvenance (3 subtests)
```
Both failed as expected; the mutation was then reverted
(`git diff -- backup/passphrase.go` shows only the rename + call-site update,
confirmed clean). **Scope note, stated in the test's own doc comment:** this
closes the *wording* half of GATE 2.3e (a future edit to either side's string
that breaks agreement now goes RED); it does not additionally pin that every
future call site threads one `derived` value into both real call points —
today's two real callers (`engravePassphraseFlowFrom`,
`engravePassphraseFlowPreloaded`) do, by literal, and that residual is a
structural observation the review itself flagged as latent, not reachable.

---

## Build gate

```
export PATH="/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH"
go build ./...                                              # clean, no output
scripts/gui-shard-test.sh ./gui/ 6 20m
go test $(go list ./... | grep -v '/gui$') -count=1 -timeout 20m
```

**gui shard suite:** `863 top-level tests` enumerated, **`partition verified
exhaustive: 863 == 863`** (858 pre-existing + 5 new: the two C1/C2 tests, the
two I1 tests, the one I2 test). 6 shards, wall **129 s**. **Exactly one
failure, `TestGate51bMaxScrollAgreesWithVisibility`** — reproduced
digit-for-digit identical to the review's own numbers, `22 of 321` values in
`[239,260]`, unmodified (not touched by this fold; `git diff` over
`scroll_arrows_test.go` confirms no lines changed inside that function). No
other test failed in any shard.

**Non-gui packages:** every package `ok` (`address`, `backup`, `bc/*`,
`bezier`, `bip380`, `bip39`, `bip85`, `bspline`, `bundle`, `cmd/*`, `codex32`,
`driver/*`, `engrave`, `font/*`, `gui/assets`, `gui/op`, `gui/saver`,
`gui/text`, `gui/widget`, `image/*`, `internal/sh2`, `md`, `mk`, `nfc/*`,
`nonstandard`, `oracle`, `passphrase`, `picobin`, `seal`, `seedqr`, `seedxor`,
`slip39`, `stepper`, `sysw`, `uf2`); zero failures.

`go build ./...` clean. `gofmt -l` on every changed file is clean except a
**pre-existing** flag on `gui/singlesig.go` (a quote-normalization quirk in
this Go 1.26.3 toolchain's `gofmt` against a doc-comment line this fold never
touched — confirmed by running `gofmt -l` against the pre-fold `HEAD` copy of
the same file, which flags identically). `go vet ./gui/... ./backup/...`
clean apart from two pre-existing `go1.26`-vs-`go1.25` file-version warnings
in `gui/op/draw_test.go` and `gui/freetext_sizeproof_golden_test.go`,
unrelated to this fold.

**No golden moved.** `git status --short` over the worktree touches exactly
six files, none under `testdata/`: `backup/passphrase.go`, `gui/gui.go`,
`gui/passphrase_flow.go`, `gui/s6b_passphrase_plate_test.go`,
`gui/scroll_arrows_test.go`, `gui/singlesig.go`. `git diff --stat -- gui
backup testdata` shows no `testdata` hits at all.

---

## Files changed (uncommitted, `wt-s6b`, branch `s6b-pre-flash`)

- `gui/passphrase_flow.go` — C1 fix (`engravePassphraseFlowPreloaded`)
- `gui/singlesig.go` — C2 fix + parameter rename (`singleSigPassphrasePlateOffer`)
- `gui/gui.go` — I1 fix (`Warning.Layout`)
- `backup/passphrase.go` — I2: export `PassphraseFooterFor` (rename only)
- `gui/s6b_passphrase_plate_test.go` — C1, C2, I2 tests (+ imports)
- `gui/scroll_arrows_test.go` — I1 tests ×2 (+ imports)

Not committed here, per the dispatch brief; ready for the controller's fold
commit (build-gate output above belongs in that commit's message) and
re-review dispatch.
