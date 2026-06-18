# SLIP-39 recovery — D2 (GUI recover flow) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development.
> Steps use `- [ ]` checkboxes. **Phase D2** of Cycle D — the GUI recover flow + engrave
> bridge, built against D1's now-merged, trusted `slip39.Combine` contract (fork `main`
> `f0092d5`). See `SPEC_seedhammer_slip39_recovery.md` (R1 GREEN) §0/§5; D1 crypto is shipped.

**Goal:** Wire on-device SLIP-39 secret recovery into the GUI: a Recover action on the
SLIP-39 confirm screen, a two-level (group/member) share-collection flow with a live
satisfaction roster, an optional SLIP-39 passphrase, a mandatory entropy-interpretation
acknowledgement, an always-on recovered-fingerprint display, and engrave of the recovered
seed as the native BIP-39 plate via `backupWalletFlow`. Also widen SLIP-39 share entry to all
valid lengths (resolves the Cycle-C followup).

**Architecture:** All changes in `gui/slip39_polish.go` (+ its test) plus a small widening of
`inputSLIP39Flow` and the menu `case 3:` in `gui/gui.go`. No crypto (D1 owns it). `fmt`-using
code stays in `slip39_polish.go` so `gui.go` gets no new `fmt` dependency; `gui.go` gains a
`bip39` use only if not already present (it is — `backupWalletFlow` lives there).

**Tech stack:** Go/TinyGo. Reuses: `slip39.Combine`/`ConsistentShares`/`Describe` (D1),
`PassphraseKeyboard` (Slice 2), `backupWalletFlow`/`masterFingerprintFor`/`bip39.New`
(Slice 3 / existing), `ConfirmWarningScreen` hold pattern, `ChoiceScreen`, the codex32
recover shape (`gui/codex32_polish.go`) as the structural template.

**Test command (host):** `/home/bcg/.local/go/bin/go test ./gui/ ./slip39/ ./bip39/`
+ `go vet ./gui/` + `gofmt -l gui/`.

**Commit hygiene:** explicit paths; SSH-signed + DCO (`git commit -S -s`, author Brian Goss);
`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

---

## Open design points this plan PINS (the R0 gate must scrutinize)

1. **Variable-length first-share entry = a word-count picker.** `inputSLIP39Flow` fills a
   pre-sized `slip39words.Mnemonic` slice and returns when full, so the length must be known
   at allocation. The user can *count the words on their physical share* (20 vs 33 is obvious),
   so before first-share entry show a `ChoiceScreen`: **"How many words? 20 / 33 / 20·160-bit
   variants"** → allocate `emptySLIP39Mnemonic(n)`. The five valid counts are {20,23,27,30,33};
   present 20 and 33 prominently (the only counts mainstream wallets emit) and the three
   intermediate as additional options. Subsequent recovery shares inherit `len(first.Mnemonic)`
   (no re-pick). This is simpler/robust vs an incremental-parse auto-detect; auto-detect is a
   noted future refinement, out of scope.
2. **PBKDF2 responsiveness (SPEC §5.6).** Recovery runs `10000·2^e` software SHA-256 blocks:
   e=0/1 ≈ 0.5–1.9 s (fine), high e is minutes-to-hours. The plan: (a) read `IterationExp`
   from the first share BEFORE collecting/decrypting; (b) if it implies a long wait
   (threshold: e ≥ 4 ≈ >10 s), show a `ConfirmWarningScreen` with the estimate and require
   hold-to-confirm (NOT a hard cap — that breaks recoverability); (c) show a "Recovering…"
   frame before the blocking `Combine` call. **The implementer MUST verify the firmware
   watchdog timeout** (`driver/`/`cmd/controller`) — if a blocking `Combine` at e=0/1 could
   exceed it, feed the watchdog or run `Combine` on a goroutine under `-scheduler tasks` and
   poll. If the watchdog comfortably exceeds ~2 s, the pre-shown frame + blocking call is
   acceptable for e≤1 and the high-e gate covers the rest. Pin the actual mechanism at impl
   time and record it.

---

## File structure

| File | Change |
|---|---|
| `gui/slip39_polish.go` | `slip39ConfirmAction` enum + Recover button (Button2 drain); `recoverSLIP39Flow` (two-level roster + passphrase); `slip39LengthPick`; the §3 hold-to-confirm + §5.4 fingerprint; engrave dispatch → `backupWalletFlow`. Add `bip39` + `slip39words.Combine`/`ConsistentShares` use. |
| `gui/slip39_polish_test.go` | recover-flow + roster + passphrase + widened-length + Button2-drain-no-hang tests. |
| `gui/gui.go` | `inputSLIP39Flow` gains a `title string` param; menu `case 3:` adds the word-count picker + passes the title; no new imports (`bip39`/`fmt`/`slip39words` already imported). |

**Unchanged (reused, must stay green):** `slip39/` (D1 crypto), `codex32/`, `backup/`,
`bip39/`, `gui/passphrase_keyboard.go`, `gui.go`'s `backupWalletFlow`/`masterFingerprintFor`
bodies (the §5.5 BIP-39-passphrase prompt text stays as-is per spec R1 note).

---

## Task 0: Worktree

- [ ] **Step 1:** `git -C /scratch/code/shibboleth/seedhammer worktree add /scratch/code/shibboleth/seedhammer-wt-slip39-d2 -b feat/slip39-recovery-gui f0092d5`
- [ ] **Step 2:** Baseline green: `cd …-d2 && /home/bcg/.local/go/bin/go test ./gui/ ./slip39/ ./bip39/`.

---

## Task 1: `confirmSLIP39Flow` → action enum + Recover button

**Files:** `gui/slip39_polish.go`, `gui/slip39_polish_test.go`. Mirror `codex32_polish.go:69-141`.

- [ ] **Step 1: Update the existing tests + add the offer/no-offer tests.** `TestConfirmSLIP39Render`
  stays (renders id/member/words). Change `TestEngraveSLIP39BackoutRecognized` if it asserts the
  bool return. Add:

```go
func TestConfirmSLIP39MultiOffersRecover(t *testing.T) {
	// A share from a 2-of-3 set (memberThreshold>1) must offer Recover (Button2).
	s := parseFixtureShare(t, /* a 2-of-3 share, mt>1 */)
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Down, Button2) // Button2 = Recover
	got := confirmSLIP39Flow(ctx, &descriptorTheme, s)
	if got != slip39Recover {
		t.Errorf("multi-share confirm: got %v want slip39Recover", got)
	}
}

func TestConfirmSLIP39LoneNoRecover(t *testing.T) {
	// A 1-of-1 share (memberThreshold==1, groupThreshold==1): Button2 is a no-op (drained);
	// Button3 still engraves. Pin no-hang.
	s := parseFixtureShare(t, slip39Duckling) // 1-of-1
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Button2, Button3) // Button2 must be drained, Button3 acts
	got := confirmSLIP39Flow(ctx, &descriptorTheme, s)
	if got != slip39Engrave {
		t.Errorf("lone share: Button2 must be drained, Button3 engrave; got %v", got)
	}
}
```

- [ ] **Step 2:** Run → FAIL (`slip39Recover` undefined).

- [ ] **Step 3: Implement** in `slip39_polish.go` — replace `confirmSLIP39Flow`'s `bool` with:

```go
type slip39ConfirmAction int

const (
	slip39Back    slip39ConfirmAction = iota // Button1
	slip39Engrave                            // Button3 / Center
	slip39Recover                            // Button2 — only when part of a multi-share set
)
```
  Body mirrors `confirmCodex32Flow`: Button1→`slip39Back`; **always** `recoverBtn.Clicked(ctx)`
  (Button2 drain); offer Recover only when `s.MemberThreshold > 1 || s.GroupThreshold > 1` →
  `slip39Recover`; Button3/Center→`slip39Engrave`. The nav row includes the Recover button
  (`assets.IconRight`, `StyleSecondary`) only when offered. Title/lines: keep the existing
  id/member/[group]/words lines; when multi-share, add a line "Engrave this share, or Recover
  the seed".

- [ ] **Step 4:** Run → PASS; vet/gofmt clean.
- [ ] **Step 5: Commit** → `feat: slip39 confirm action enum + Recover button (Button2 drain)`.

> Test-helper note: add `parseFixtureShare(t, mnemonic string) slip39words.Share` =
> `ParseShare` + `t.Fatal` on error. For a real 2-of-3 share, load a mnemonic from D1's
> `slip39/testdata/slip39_vectors.json` idx 3 (copy the string into the gui test, or add a
> tiny loader); confirm `MemberThreshold==2`.

---

## Task 2: Widen share entry (all lengths) — word-count picker + `inputSLIP39Flow` title param

**Files:** `gui/gui.go`. Resolves R0 I1 + the Cycle-C all-lengths followup.

- [ ] **Step 1: Failing test** (in `slip39_polish_test.go`): drive the menu/entry for a 33-word
  share via the length pick and assert it parses (or, if menu-driving is heavy, unit-test the
  picker→`emptySLIP39Mnemonic(n)` sizing). Keep it focused:

```go
func TestSLIP39LengthPick33(t *testing.T) {
	// slip39LengthPick returns the chosen word count; selecting "33 words" → 33.
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Down, Button3) // move to the 33-word option, select
	if n := slip39LengthPick(ctx, &descriptorTheme); n != 33 {
		t.Errorf("length pick = %d want 33", n)
	}
}
```

- [ ] **Step 2:** Run → FAIL.

- [ ] **Step 3: Implement.**
  1. Add `title string` as the LAST param of `inputSLIP39Flow` (in `gui.go:796`); render it via
     the existing title path (replace the current fixed SLIP-39 title with `title`). Update the
     sole existing call site signature.
  2. Add `slip39LengthPick(ctx, th) int` — a `ChoiceScreen` titled "Words on your share?" with
     choices `["20 (128-bit)", "33 (256-bit)", "23 (160-bit)", "27 (192-bit)", "30 (224-bit)"]`
     returning the chosen count; default index 0 (20). (Cancel → return 0 / sentinel; menu
     treats it as back.)
  3. Menu `case 3:` (`gui.go:2034`): call `n := slip39LengthPick(...)`; if cancelled, `break`;
     `mnemonic := emptySLIP39Mnemonic(n)`; `inputSLIP39Flow(ctx, th, mnemonic, 0, "Input SLIP-39 Share")`;
     build + `ParseShare` as today (now any of the 5 lengths parses).

- [ ] **Step 4:** Run → PASS; the existing single-share entry still works (20-word path
  unchanged behaviorally except the one extra pick tap). vet/gofmt clean.
- [ ] **Step 5: Commit** → `feat: slip39 all-length share entry via word-count picker`.

---

## Task 3: `recoverSLIP39Flow` — two-level roster + optional passphrase

**Files:** `gui/slip39_polish.go`, `gui/slip39_polish_test.go`. Mirror `recoverCodex32Flow` but
two-level (SPEC §5.3).

- [ ] **Step 1: Failing test:**

```go
func TestRecoverSLIP39(t *testing.T) {
	// idx 3 = 2-of-3 single-group. Enter 2 shares; expect the recovered bip39.Mnemonic.
	first := parseFixtureShare(t, vec3Share(t, 0))
	ctx := NewContext(newPlatform())
	// queue: type share-1's words + accept, then the passphrase "Skip", etc. (drive via runUI)
	// assert recoverSLIP39Flow returns ok and a mnemonic whose entropy == idx-3 master.
	m, ok := driveRecover(t, ctx, first /* + queued events */)
	if !ok { t.Fatal("recover failed") }
	if hexOfEntropy(m) != "b43ceb7e57a0ea8766221624d01b0864" {
		t.Errorf("recovered entropy mismatch: %s", hexOfEntropy(m))
	}
}

func TestRecoverSLIP39Mismatch(t *testing.T) {
	// Entering a share with a different identifier → eager ConsistentShares error + re-prompt.
}
func TestRecoverSLIP39BackoutRecognized(t *testing.T) {
	// Back at the first collection prompt → (nil, false); engrave dispatch returns true.
}
```

- [ ] **Step 2:** Run → FAIL.

- [ ] **Step 3: Implement** `recoverSLIP39Flow(ctx, th, first slip39words.Share) (bip39.Mnemonic, bool)`:
  1. `shares := []slip39words.Share{first}`. `GT := first.GroupThreshold`, `gc := first.GroupCount`,
     word-len `L := len(first.Mnemonic)`.
  2. **Sufficiency** = exactly `GT` distinct group indices each holding exactly its
     `MemberThreshold` shares. Maintain a roster: `byGroup map[int][]slip39words.Share`; a group
     is *satisfied* when `len==mt`. Loop while `satisfied < GT`:
     - Build a title showing progress: `fmt.Sprintf("Group %d · share %d/%d · %d/%d groups",
       …)` and the count still needed. Allocate `emptySLIP39Mnemonic(L)`; `inputSLIP39Flow(ctx,
       th, m, 0, title)`. Back → `return nil, false`.
     - `ParseShare`; eager `slip39words.ConsistentShares(append(shares, cand))` →
       `showError(ctx, th, "SLIP-39", slip39words.Describe(err))` + continue (re-prompt). Reject
       a duplicate `(GroupIndex, MemberIndex)` (ConsistentShares covers it). Append on success;
       update roster.
     - **Stop exactly at sufficiency** (don't over-collect — `Combine`'s exact-count rule would
       error): once `satisfied == GT`, break and offer Continue.
     - **Dead-end affordance:** if the user has entered shares but cannot reach `GT` satisfied
       groups with what they hold, offer a "Start over / Cancel" path (e.g. Back unwinds).
  3. **High-e gate (§5.6):** if `first.IterationExp >= 4`, show a `ConfirmWarningScreen` with the
     estimated wait; abort on cancel.
  4. **Optional SLIP-39 passphrase (§5.5):** a `ChoiceScreen` "SLIP-39 passphrase? (NOT a BIP-39
     passphrase)" default **Skip** (index 0) with the warning "A wrong passphrase silently
     recovers a different seed." If entered, use a fresh `NewPassphraseKeyboard` (Slice 2) →
     `pass`. Skip → `pass = ""`.
  5. Show a "Recovering…" frame; `secret, err := slip39words.Combine(shares, []byte(pass))`. On
     error → `showError(ctx, th, "SLIP-39", slip39words.Describe(err))`; return `nil, false`. On
     success: `m := bip39.New(secret)`; `slip39words`-side already wiped its internals; wipe the
     local `secret` slice; return `m, true`.

- [ ] **Step 4:** Run → PASS; vet/gofmt clean. (Drive via `runUI` + queued `ctx.Router` events,
  mirroring `gui/codex32_polish_test.go`'s recover tests.)
- [ ] **Step 5: Commit** → `feat: slip39 two-level recover flow (roster + optional passphrase)`.

---

## Task 4: Engrave dispatch — acknowledgement + fingerprint + `backupWalletFlow`

**Files:** `gui/slip39_polish.go`, `gui/slip39_polish_test.go`.

- [ ] **Step 1: Failing test:**

```go
func TestEngraveSLIP39RecoverToBackup(t *testing.T) {
	// confirm(Recover) → recoverSLIP39Flow ok → hold-to-confirm ack → fingerprint shown →
	// backupWalletFlow reached. Assert the ack screen text + that a recovered seed reaches engrave.
}
```

- [ ] **Step 2:** Run → FAIL.

- [ ] **Step 3: Implement** the engrave dispatch in `engraveSLIP39` (loop over `confirmSLIP39Flow`):
  - `slip39Back` → `return true` (recognized; never "Unknown format").
  - `slip39Engrave` → the existing verbatim single-share engrave (unchanged).
  - `slip39Recover` → `m, ok := recoverSLIP39Flow(ctx, th, scan)`; `!ok` → `continue`; ok →
    1. **§3 hold-to-confirm:** a `ConfirmWarningScreen` (hold pattern, `gui.go:312`) — *"Recovered
       as a BIP-39 seed. Correct only for backups made from a BIP-39 phrase / this toolkit. A
       Trezor/other SLIP-39 wallet backup would engrave the WRONG seed."* Cancel → `continue`.
    2. **§5.4 fingerprint display:** compute `mfp, _ := masterFingerprintFor(m, &chaincfg.MainNetParams, "")`;
       show a screen *"Fingerprint %08X — confirm against your records before engraving"*
       (`fmt.Sprintf("%08X", mfp)`); a Back here → `continue`, Engrave/OK → proceed.
    3. `backupWalletFlow(ctx, th, m)` (reuses Slice-3 confirm → optional BIP-39 passphrase →
       fingerprint choice → SeedQR+words engrave). Then `return true`.

- [ ] **Step 4:** Run → PASS; vet/gofmt clean.
- [ ] **Step 5: Commit** → `feat: slip39 recovered-seed engrave via acknowledgement + fingerprint + backupWalletFlow`.

---

## Task 5: Full guard + the no-hang / passphrase-isolation regressions

**Files:** `gui/slip39_polish_test.go`.

- [ ] **Step 1:** Button2-drain no-hang regression (the Cycle-B class): a direct-call test that a
  queued Button2 on a lone share does not stall Button3 (covered by Task 1's `…LoneNoRecover`;
  confirm it runs without `runUI` if applicable).
- [ ] **Step 2:** Passphrase-isolation: a test driving BOTH a SLIP-39 passphrase (recovery) and a
  BIP-39 passphrase (in `backupWalletFlow`) asserting the recovered seed is unchanged by the
  BIP-39 passphrase choice (they're independent; the BIP-39 one only changes the engraved
  fingerprint, not the words). Keep it light if full drive is heavy — at minimum assert the two
  prompts are distinct screens with distinct labels.
- [ ] **Step 3: Run the FULL guard:** `…/go test ./gui/ ./slip39/ ./bip39/`, `go vet ./gui/`,
  `gofmt -l gui/`. All green/clean. Existing guards (`TestConfirmSLIP39Render`,
  `TestEngraveSLIP39BackoutRecognized`, codex32, BIP-39, backup goldens) stay green.
- [ ] **Step 4: Commit** → `test: slip39 recover no-hang + passphrase-isolation guards`.

---

## Self-review checklist

- The two pinned design points (word-count picker; high-e warn + watchdog mechanism) are
  implemented and the watchdog behavior is verified against the actual firmware (recorded in
  the commit/PR notes) — NOT assumed.
- Recover offered only for multi-share (`MemberThreshold>1 || GroupThreshold>1`); Button2 always
  drained (no-hang); Back always returns `true` (recognized) in the engrave dispatch.
- Two-level roster stops EXACTLY at `GT` satisfied groups (no over-collection); dead-end has a
  start-over/cancel path; subsequent shares sized to `len(first.Mnemonic)`.
- The §3 acknowledgement is a hold-to-confirm BEFORE engrave; the §5.4 fingerprint is shown
  before engrave; the SLIP-39 passphrase prompt defaults to Skip with the silent-wrong-seed
  warning and is labeled distinctly from the BIP-39 25th-word passphrase.
- The recovered seed engraves via `backupWalletFlow` (no parallel confirm/engrave path);
  `backupWalletFlow`/`masterFingerprintFor` bodies UNCHANGED.
- `gui.go` gains no new import; `fmt`-using code is in `slip39_polish.go`.
- Signed + DCO + Brian Goss on every commit; the D1 crypto + GUI guards stay green.
