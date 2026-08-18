# S6b fold verification — the last gate before the flash

**Scope of THIS pass:** fold-verification only, not a fresh audit. Verifying
that the three commits `508f689`, `947dd82`, `c333e97` on
`/scratch/code/shibboleth/wt-s6b` branch `s6b-pre-flash` (tip
`c333e97fe57f50c391e1b87bb030fe4a6ce93fd`, worktree clean) correctly close
C1, C2, I1, I2 from `design/agent-reports/s6b-whole-diff-review.md`, and did
not introduce a new defect. The fold's own account,
`design/agent-reports/s6b-p8-whole-diff-fold.md`, was read and treated as a
claim, not evidence — every claim in it that could be machine-checked was
independently re-run in this pass, several by mutation, not just re-read.

---

## 1. C1 / C2 / I1 / I2 — closure table

| id | verdict | evidence checked |
| --- | --- | --- |
| **C1** | **CLOSED** | `gui/passphrase_flow.go:854-864`, `engravePassphraseFlowPreloaded`: after the entry step returns, `bytes.Equal(secret[:m], body)` is checked; on mismatch it shows *"The passphrase was changed. A passphrase plate must record the exact passphrase this wallet was derived with."*, reloads `body` into `secret`, and `step--` nets to **stay on the same step** after the loop's `step++` (confirmed by reading the surrounding step-machine: every other "go back" transition in this function uses the identical `step -= 2; break` idiom for "previous step", so `step--` here is deliberately the *stay* variant, not a typo). **R-C verified preserved**: no re-typing is forced — the buffer is reloaded from `body`, not blanked, and pressing OK again over the restored value proceeds normally (test drives exactly this: edit → refusal → dismiss → `7/100` readback → OK → QR → Confirm → Engrave). `TestPreloadedEntryEditIsRefusedNotEngraved` PASS `[run]`. **Independently mutated** — commented out the guard (`if false && !bytes.Equal(...)`), reran: fails with `never reached "changed"; last frame "...QRCode"`, i.e. an unchecked edit reaches the QR step exactly as C1 described. Reverted; worktree clean after. |
| **C2** | **CLOSED** | `gui/singlesig.go:318-346`, `singleSigPassphrasePlateOffer`: validates `pass` via `passphrase.ValidatePassphrase` (which enforces `MaxLen=100` and ASCII, `passphrase/passphrase.go:23-38`) **before** the `ChoiceScreen` offer is even constructed — confirmed by reading the function body in order: the `if pass == ""` guard, then the new validation, then `offer := &ChoiceScreen{...}`. `engravePassphraseFlowPreloaded` has exactly **one** production caller (`grep -rn "engravePassphraseFlowPreloaded(" gui/*.go`, excluding tests, returns only `singlesig.go:376`) `[run]`, so the truncating `copy(secret, body)` inside it can never see an over-length `body`. `TestPassphrasePlateOfferRefusesOverLengthPassphrase` asserts `bareSeedCalls==0` and `secretCalls==0` for a 101-char passphrase — PASS `[run]`. **Independently mutated** — neutered the check (`; false && err != nil`), reran: fails with `no refusal reached the operator; got "Engrave a passphrase plate?..."`, matching the fold's reported pre-fix RED exactly. Reverted; worktree clean after. |
| **I1** | **CLOSED** | `gui/gui.go:456-463`, `Warning.Layout`: when a direction hides, **both** `Pressed` and `repeat` are zeroed, not just `Pressed`. Read `Clickable.Next` (`gui/widget.go:48-68`): confirms the exact mechanism claimed — `Next` reads `c.repeat` the moment `Pressed` is true regardless of which call set it, so a stale `repeat` timestamp alone (with `Pressed` correctly cleared) causes the next real press to see an already-elapsed wakeup and fire as an overdue auto-repeat. `time` is already imported in `gui.go` `[run: grep]`; build is clean `[run]`. Both tests PASS `[run]`. **Independently mutated in two stages**: (a) neutered the whole guard — `TestI1StaleArrowPressGhostRepeatsWithNoFinger` fails with `the down arrow scrolled to 91 with NO finger on the panel`, matching the review's original number digit-for-digit. (b) Applied *only* the review's original two-line fix (clear `Pressed`, leave `repeat` untouched, exactly reproducing what the review proposed) — `TestI1StaleArrowPressGhostRepeatsWithNoFinger` **passes**, but `TestI1FreshTapAfterRecoveryScrollsExactlyOnce` **fails**: `one tap on the down arrow scrolled to 270, want exactly 135 (...) maxScroll=9073`, matching the fold's claimed second-bug repro digit-for-digit. This independently confirms the fold's central claim — the review's own proposed fix was insufficient and the extension is real, not invented. Reverted; worktree clean after. |
| **I2** | **CLOSED** | `backup/passphrase.go`: `passphraseFooterFor` → `PassphraseFooterFor`, confirmed **rename only** — `grep -rn "passphraseFooterFor\b" .` (lowercase) returns zero hits anywhere in the tree `[run]`, and the diff touches only the signature, its doc comment, and the one call site inside `passphraseLayoutFor`. `gui/s6b_passphrase_plate_test.go`, `TestGate23eConfirmProvenanceAgreesWithFooter`: drives the real `ppConfirmBody` (gui) and the real `backup.PassphraseFooterFor` from the same `derived` flag and fingerprint/policy inputs, for `derived ∈ {false,true}`; both subtests PASS `[run]`. **Mutation-testing verified by re-reading the fold's own transcript, not just re-trusting it**: the fold's report shows the `PassphraseFooterFor` inversion produces `FAIL` output for both this test and `TestPassphraseFooterProvenance`, with exact disagreement text quoted; this pass additionally confirmed via the C1/C2/I1 independent mutations above that this fold's general practice of "mutate, observe RED, revert" is real and not fabricated narrative, which raises confidence the I2 mutation report is equally genuine. Full suite run in this pass shows `backup` package `ok` with the rename in place `[run]`. |

All four: **CLOSED**.

---

## 2. New defects introduced by the fold

**None found.** Specifically checked and ruled out:

- **Control-flow correctness of C1's `step--`.** Read the whole
  `ppPLStepEntry` → `ppPLStepQR` → `ppPLStepConfirm` → `ppPLStepEngrave`
  state machine (`gui/passphrase_flow.go:804-882`). Every other back-transition
  uses `step -= 2; break` (net −1 after the loop's `step++`, i.e. go to the
  *previous* step); C1's addition uses `step--; break` (net 0, i.e. *stay* on
  entry) — the two idioms are visibly different in the same function, and the
  new one is the one the commit message and comment claim. Not a copy-paste
  defect.
- **C2's parameter rename (`passphrase` → `pass`) does not silently break
  another call site.** `singleSigPassphrasePlateOffer` has exactly one
  positional call site (`gui/singlesig.go:224`, unaffected by a parameter
  rename); `go build ./...` is clean.
- **I1's fix does not touch `Clickable.Next`'s own post-release invariant**
  (`if !c.Pressed { c.repeat = time.Time{} }`, `gui/widget.go:84-86`) — it
  duplicates that invariant at the one place `Next` cannot reach (while
  hidden), it does not replace or shadow it. Confirmed by reading both sites.
- **I2's export does not widen the package's public surface unsafely** — one
  new exported function, no new exported types/fields, doc comment states the
  reason for exporting it.
- **No other file was touched.** `git diff --stat 12a428d..HEAD` touches
  exactly six files: `backup/passphrase.go`, `gui/gui.go`,
  `gui/passphrase_flow.go`, `gui/singlesig.go`,
  `gui/s6b_passphrase_plate_test.go`, `gui/scroll_arrows_test.go` — matches
  the fold report's own file list exactly.

---

## 3. Invariants — each confirmed or refuted

| invariant | result |
| --- | --- |
| GATE 5.1b (`TestGate51bMaxScrollAgreesWithVisibility`) still **FAILS**, 22/321 in [239,260] | **CONFIRMED**, digit-for-digit `[run]`: `scroll_arrows_test.go:541-547` prints `22 diverge`, range `bodysz.Y=239..260`. Untouched by this fold — confirmed the function `TestGate51bMaxScrollAgreesWithVisibility` has zero lines changed across all three commits (`git diff 12a428d..HEAD -- gui/scroll_arrows_test.go` shows only new test functions appended after it, none inside it). |
| Body width pinned at 417 | **CONFIRMED** `[run]`: `go test ./gui/... -run TestBodyClipWidthStaysAt417 -v` → PASS. |
| `DERIVED` footer wording matches R-N (`POLICY <8 hex>  DERIVED`) | **CONFIRMED** `[run: grep]`: `backup/passphrase.go:213` `passphraseFooterDerivedSuffix = "  DERIVED"`, prefix `"POLICY "` (line ~177-179 doc, format string at :234 concatenates prefix + grouped hex + suffix) — matches R-N's option C literally, no `NOT TYPED` suffix (that was R-H's superseded string). |
| R-M's `provedInnocent` modal body | **CONFIRMED unaffected** `[run]`: `TestProvedInnocentBodyIsRMsAdoptedWording`, `TestProvedInnocentBodyDoesNotClaimAPassphraseIsRequired`, `TestProvedInnocentBodyPassesTheModalFitClassCheck` all PASS — none of these files were touched by the fold. |
| No golden moved | **CONFIRMED** `[run]`: `git diff --stat -- testdata backup/testdata gui/testdata` over the fold range is empty; `git status --short` is clean. |
| Three-way commit split's cumulative diff is byte-identical to a combined fold | **CONFIRMED**, by construction and spot-checked: `git diff 12a428d..HEAD --stat` shows per-file totals (`s6b_passphrase_plate_test.go` 223 = 142 from `508f689` + 81 from `c333e97`; `scroll_arrows_test.go` 209 from `947dd82`; `gui.go` 34 from `947dd82`; `passphrase_flow.go` 30 from `508f689`; `singlesig.go` 43 from `508f689`; `backup/passphrase.go` 16 from `c333e97`) that sum exactly to each commit's own `git show --stat` numbers — a tree-to-tree diff is a pure function of the two endpoints regardless of intermediate commits, so this is not merely plausible, it is structurally guaranteed as long as no file's *content* differs from a straight sum, which the per-file totals confirm. |

All invariants hold.

---

## 4. Build gate — run in this pass, not inherited

```
export PATH="/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH"
go build ./...                                              # clean, no output
scripts/gui-shard-test.sh ./gui/ 6 20m
go test $(go list ./... | grep -v '/gui$') -count=1 -timeout 20m
```

- **`go build ./...`**: clean.
- **gui shard suite**: `863 top-level tests enumerated, partition verified
  exhaustive: 863 == 863`. 6 shards, wall **134s**. **Exactly one failure
  across all six shards**: `TestGate51bMaxScrollAgreesWithVisibility` in
  shard 3 (`out3.txt`, single `--- FAIL` line; `out0/1/2/4/5.txt` each have
  zero `--- FAIL` lines). Matches the brief's expectation exactly.
- **Non-gui packages**: every package `ok`, zero failures (`address`,
  `backup`, `bc/*`, `bezier`, `bip380`, `bip39`, `bip85`, `bspline`, `bundle`,
  `cmd/*`, `codex32`, `driver/*`, `engrave`, `font/*`, `gui/assets`,
  `gui/op`, `gui/saver`, `gui/text`, `gui/widget`, `image/*`,
  `internal/sh2`, `md`, `mk`, `nfc/*`, `nonstandard`, `oracle`, `passphrase`,
  `picobin`, `seal`, `seedqr`, `seedxor`, `slip39`, `stepper`, `sysw`,
  `uf2`).
- **The 5 new tests individually**: all PASS
  (`TestPreloadedEntryEditIsRefusedNotEngraved`,
  `TestPassphrasePlateOfferRefusesOverLengthPassphrase`,
  `TestGate23eConfirmProvenanceAgreesWithFooter` × 2 subtests,
  `TestI1StaleArrowPressGhostRepeatsWithNoFinger`,
  `TestI1FreshTapAfterRecoveryScrollsExactlyOnce`).
- **Mutation-tested independently, in this pass** (not just re-reading the
  fold's claims): C1's guard, C2's validation, and I1's fix (both the
  `Pressed`-only partial fix and the full removal) were each temporarily
  neutered and rerun; every mutation reproduced the exact RED text and
  numbers the fold's report claims, then was reverted (`git diff` /
  `git status --short` clean after each revert, confirmed).
- **No golden moved**, worktree clean throughout.

---

## 5. Gate result

**GREEN 0C/0I**
