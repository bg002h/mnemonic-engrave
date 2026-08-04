# BIP-39 Password — Phase D: Flow and Menu — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the feature to the machine — a six-step flow that a user can actually drive by touch, with warnings that tell the truth about what is and is not verified.

**Architecture:** A new `engravePassphraseFlow` following the existing flow idiom (`deriveXpubFlow`, `bip85DeriveFlow`), plus the seventh entry in the `program` enum. Consumes Phase B's validators and Phase C's plate type; adds no new engraving logic.

**Spec:** `design/SPEC_seedhammer_engrave_bip39_password.md` — R0 GREEN. Relevant: §5 (flow), §5.1 (warnings), §5.2 (touch), §5.3 (secret hygiene), §6 (menu), §7.

**Depends on:** Phase B (`passphrase.ValidatePassphrase`, `ValidateFingerprint`, `GroupFingerprint`) and Phase C (`backup.Passphrase`, `EngravePassphrase`).

**Repo:** paths relative to `/scratch/code/shibboleth/seedhammer`.

## Global Constraints

- Devshell only: `nix develop --command <cmd>` from the repo root.
- **`-update` forbidden.** Phase D adds no engraving; any golden movement is a bug.
- **Every interactive element must be reachable by TOUCH.** SeedHammer II has no directional buttons — the only production input is the `ft6x36` capacitive panel emitting `PointerEvent`s. A screen wired only to `ButtonFilter(...)` is dead on real hardware. This exact defect shipped once, in the StartScreen pager, and was fixed in `86e0da9`. Bind interactive elements to `Clickable` (which routes both `ButtonFilter(c.Button)` and `PointerFilter(c)`) and register an `op.Input` hit area for each.
- **Tests must drive the flow by `PointerEvent`**, using the `runUITouch` / `tap` harness in `gui/start_screen_touch_test.go`. Synthesised button events prove nothing — no production path emits them.
- Stage paths explicitly. `git commit -s`.

---

### Task 1: Menu wiring — the seventh program

Doing this first gives every later task a reachable entry point.

**Files:** `gui/gui.go`, `gui/gui_test.go`

- [ ] **Step 1: Write the failing test**

```go
func TestPassphraseProgramReachable(t *testing.T) {
	ctx := NewContext(newPlatform())
	m := new(StartScreen)
	frame, drawer, quit := runUITouch(ctx, func() { m.Flow(ctx, &descriptorTheme) })
	defer quit()
	if _, ok := frame(); !ok {
		t.Fatal("no frame")
	}
	_, right := arrowPoints(ctx)
	tap(&ctx.Router, drawer(), right) // one step from Backup Wallet
	content, ok := frame()
	if !ok {
		t.Fatal("no frame after tap")
	}
	if !uiContains(content, "BIP-39 Password") {
		t.Fatalf("second program is not the passphrase program; got %q", content)
	}
}
```

- [ ] **Step 2: Run it, confirm it fails**, then insert the program **second**:

```go
const (
	backupWallet program = iota
	engravePassphrase          // NEW — position 2 of 7
	engraveXpub
	engraveBundle
	engraveSingleSig
	engraveMultisig
	bip85Derive
	qaProgram
)
```

Inserting here (rather than appending) keeps `bip85Derive` last, so the compile-time guard at `gui.go:164`, the wrap bound `m.prog > bip85Derive`, and `npage = int(bip85Derive)+1` all stay correct automatically. Three sites need the new case: `layoutMainPlates`, `StartScreen.draw`'s title switch, and the flow dispatch (`gui.go:1502-1526`).

- [ ] **Step 3: Confirm the guard still holds and the pager still wraps**

Run the existing `TestStartScreenPagerTouch*` tests. They must pass with seven programs. **Verify `program` values are not persisted anywhere** (spec O2 — the R0 review already checked this and found `m.prog` runtime-only, but confirm before relying on it).

- [ ] **Step 4: Commit**

---

### Task 2: Passphrase entry (required step)

**Files:** `gui/passphrase_flow.go` (new), `gui/passphrase_flow_test.go` (new)

- [ ] **Step 1: Write the failing touch test**

```go
// The step cannot be completed empty, and the counter reflects what is typed.
func TestPassphraseEntryRequiresContent(t *testing.T) { /* tap OK with empty -> stays */ }
func TestPassphraseEntryCounter(t *testing.T)         { /* type 3 -> "3/100" */ }
```

- [ ] **Step 2: Implement**, reusing `PassphraseKeyboard` (masked, reveal toggle). Refuse to advance while empty. Show a live `n/100` counter. Reject invalid input via `passphrase.ValidatePassphrase`, surfacing the typed error — and **never echoing the passphrase into the message** (spec §5.3).

- [ ] **Step 3: Commit**

---

### Task 3: The two fingerprint steps (optional, skippable)

**Files:** `gui/passphrase_flow.go`, `gui/passphrase_flow_test.go`

- [ ] **Step 1: Write the tests**

```go
func TestFingerprintStepsSkippable(t *testing.T)   { /* skip both -> flow continues, fields empty */ }
func TestFingerprintDisplayGrouped(t *testing.T)   { /* typed A1B2C3D4 shows as "A1B2 C3D4" */ }
func TestFingerprintRejectsBadInput(t *testing.T)  { /* 7 digits, non-hex -> error, no advance */ }
```

- [ ] **Step 2: Implement.** Both fields optional. Display grouped 4-and-4 to match the plate (spec §4.3), storing the canonical form from `passphrase.ValidateFingerprint`. **The separator is a plain space, never the visible-space mark.**

Each step carries the warning that the value is **typed and unverified**. Step 3's is stronger: an incorrect passphrase does not fail — it silently opens a **different wallet**.

- [ ] **Step 3: Commit**

---

### Task 4: QR choice and the confirm screen

**Files:** `gui/passphrase_flow.go`, `gui/passphrase_flow_test.go`

The confirm screen is the last checkpoint before a permanent plate, and §5.1 makes a specific demand that is easy to under-build.

- [ ] **Step 1: Write the tests FIRST**

```go
// Revealing the text is NOT sufficient: a space is as invisible on screen as it
// is on metal, and a 100-char string wraps, hiding spaces adjacent to a break.
func TestConfirmRendersSpacesVisibly(t *testing.T) {
	// a passphrase with leading, trailing, interior and repeated spaces must
	// render the visible mark for each, not blanks
}

// A count is checkable against intent in a way a wall of characters is not.
func TestConfirmShowsDerivedCounts(t *testing.T) {
	// "100 chars · 3 spaces · 1 trailing" — leading and trailing called out
	// BY NAME, not merely included in the total
}
```

- [ ] **Step 2: Implement the QR choice** (default **off**), stating that the QR is a machine-readable copy of the secret.

- [ ] **Step 3: Implement the confirm screen** — passphrase **revealed** (a masked readout cannot be proof-read), spaces rendered with the mark, derived counts displayed, both fingerprints shown grouped with their unverified warning.

- [ ] **Step 4: Commit**

---

### Task 5: Secret hygiene

**Files:** `gui/passphrase_flow.go`, `gui/passphrase_flow_test.go`

- [ ] **Step 1: Implement `[]byte` accumulation** per §5.3, wiped on flow exit **and on abort**, using the existing `wipeBytes` idiom (`gui/derive.go`).

**Be honest about the limit.** `PassphraseKeyboard.Fragment` is a Go `string` grown by concatenation, and Go strings are immutable — every keystroke leaves an unwipeable heap copy of a prefix. A blanket "wiped on flow exit" claim would be false. Keep string conversions to the minimum needed for rendering and encoding, and **document in code where residual copies remain**. Mitigating context, not an excuse: RAM is volatile, the device is air-gapped, and it powers down between uses.

- [ ] **Step 2: Assert no logging path.** The passphrase must never reach `log.Printf` or any error string.

- [ ] **Step 3: Commit**

---

### Task 6: Touch coverage over the whole flow

**Files:** `gui/passphrase_flow_test.go`

- [ ] **Step 1: Drive the entire flow end-to-end by `PointerEvent` only** — entry, both fingerprints, QR choice, confirm, engrave — using `runUITouch` / `tap`.

If any step cannot be completed by touch, that is a **Critical** finding, not a test to adjust. The machine has no other input.

- [ ] **Step 2: Full suite, then commit**

---

## Phase D exit criteria

- [ ] Program reachable second in the menu; pager tests pass with seven programs; `gui.go:164` guard intact.
- [ ] Entry refuses empty, counts to 100, surfaces validation errors without echoing the secret.
- [ ] Both fingerprint steps skippable, grouped 4-and-4, warned as unverified.
- [ ] Confirm screen renders spaces visibly **and** shows derived counts including trailing.
- [ ] QR opt-in, default off, described as a machine-readable copy of the secret.
- [ ] Secret accumulated in `[]byte`, wiped on exit and abort, residual copies documented.
- [ ] **The entire flow driven by touch alone in tests.**
- [ ] Full suite green, no `-update`.
- [ ] Mandatory post-implementation adversarial review over the whole Phase D diff — **non-deferrable**.

## After Phase D

The feature is code-complete but **not done**. Two gates remain, and neither is closeable from a keyboard:

- **O1 — hardware legibility.** Lowercase has never been cut on this machine. Engrave a real plate and inspect every confusable pair from §3.2.1 by eye, especially the case-only class (`C/c O/o S/s U/u V/v W/w X/x Z/z K/k`) and `K`/`k`, which the font's own author flagged as weakest.
- **O4 — final legend and footer wording**, measured at 3 mm in the margin bands.
