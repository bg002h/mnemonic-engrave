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
| `gui/slip39_polish.go` | `slip39ConfirmAction` enum + Recover button (Button2 drain); `recoverSLIP39Flow` (two-level roster + `selectForCombine` + passphrase); `slip39LengthPick`; the §3 hold-to-confirm + §5.4 fingerprint; engrave dispatch → `backupWalletFlow`. **New imports: `bip39` AND `github.com/btcsuite/btcd/chaincfg/v2`** (the §5.4 fingerprint calls `masterFingerprintFor(m, &chaincfg.MainNetParams, "")`); + `slip39words.Combine`/`ConsistentShares` use. |
| `gui/slip39_polish_test.go` | recover-flow + roster + `selectForCombine` unit + multi-group + passphrase + widened-length + Button2-drain-no-hang tests. |
| `gui/gui.go` | `inputSLIP39Flow` gains a `title string` param (param-izes the hard-coded `"Input Words"` title literal at `gui.go:868`); menu `case 3:` calls `slip39LengthPick` then passes the title; **no new imports** (`bip39`/`fmt`/`strings`/`chaincfg`/`slip39words` already imported there). |

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
	click(&ctx.Router, Button2) // Button2 = Recover (no list to navigate; M2: no spurious Down)
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

**Files:** `gui/gui.go` (the `inputSLIP39Flow` title param + the menu `case 3:` picker call)
and `gui/slip39_polish.go` (define `slip39LengthPick` there — same package, keeps the
`fmt`/`ChoiceScreen` helper beside the other SLIP-39 flow funcs; M4). Resolves R0 I1 of the
spec + the Cycle-C all-lengths followup.

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
  1. Add `title string` as the LAST param of `inputSLIP39Flow` (in `gui.go:796`); param-ize the
     hard-coded title literal `"Input Words"` (`gui.go:868`) to render `title` instead (M5).
     Update the sole existing call site signature (menu `case 3:`).
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
	// idx 3 = 2-of-3 single-group. Enter the 2nd share, SKIP the passphrase (empty).
	// CRITICAL (plan-R0 C1): with an EMPTY passphrase the recovered secret is the
	// empty-passphrase value, NOT the "TREZOR" corpus value. Verified empirically
	// against D1 Combine: pp="" -> 61cf4d6c0d8a07d8c2fd3cff22432664;
	// pp="TREZOR" -> b43ceb7e57a0ea8766221624d01b0864. This test drives Skip, so:
	first := parseFixtureShare(t, vec3Share(t, 0))
	ctx := NewContext(newPlatform())
	// queue: type share-1's words + accept, then choose "Skip" at the passphrase prompt.
	m, ok := driveRecover(t, ctx, first /* + queued events, passphrase=Skip */)
	if !ok { t.Fatal("recover failed") }
	if hexOfEntropy(m) != "61cf4d6c0d8a07d8c2fd3cff22432664" {
		t.Errorf("recovered entropy (empty passphrase) mismatch: %s", hexOfEntropy(m))
	}
}

func TestRecoverSLIP39Passphrase(t *testing.T) {
	// Same 2 shares but TYPE "TREZOR" at the passphrase prompt (drive the Slice-2
	// PassphraseKeyboard) → the canonical corpus secret. Proves the SLIP-39
	// passphrase feeds the Feistel decrypt and changes the result.
	first := parseFixtureShare(t, vec3Share(t, 0))
	ctx := NewContext(newPlatform())
	m, ok := driveRecover(t, ctx, first /* + queued events, passphrase="TREZOR" */)
	if !ok { t.Fatal("recover failed") }
	if hexOfEntropy(m) != "b43ceb7e57a0ea8766221624d01b0864" {
		t.Errorf("recovered entropy (TREZOR) mismatch: %s", hexOfEntropy(m))
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
  1. `GT := first.GroupThreshold`; word-len `L := len(first.Mnemonic)`. Maintain a roster
     `byGroup map[int][]slip39words.Share` seeded with `first`. A group is *satisfied* when it
     holds exactly its `MemberThreshold` distinct members; `satisfied()` counts satisfied groups.
  2. **Collection loop** while `satisfied() < GT`:
     - Title shows live progress: `fmt.Sprintf("Group %d · %d/%d · %d/%d groups done", …)`.
       Allocate `emptySLIP39Mnemonic(L)`; `inputSLIP39Flow(ctx, th, m, 0, title)`. Back →
       `return nil, false`.
     - `ParseShare(cand)`; eager `slip39words.ConsistentShares(append(allShares(byGroup), cand))`
       → `showError(…, slip39words.Describe(err))` + continue. ConsistentShares covers id/ext/
       iterExp/groupThr/groupCount/value-len + duplicate `(GroupIndex, MemberIndex)`.
     - **Reject a share whose group is ALREADY satisfied** (would over-fill it; `Combine` needs
       exactly memberThreshold): `showError(…, "that group is already complete")` + continue. So
       no group ever exceeds its `MemberThreshold`.
     - Append `cand` to `byGroup[cand.GroupIndex]`.
  3. **(I1 — the assembly rule.)** When `satisfied() == GT`, build the `Combine` input from
     **exactly the GT satisfied groups' members** — prune any partially-filled / extra group that
     lingers in the roster (e.g. a lone share from a wrong pile). Factor this into a pure,
     unit-testable helper:
     ```go
     // selectForCombine returns the flattened members of exactly groupThreshold
     // satisfied groups (a group is satisfied when it holds exactly its
     // MemberThreshold members), dropping partial/extra groups. ok=false if fewer
     // than groupThreshold groups are satisfied.
     func selectForCombine(byGroup map[int][]slip39words.Share, groupThreshold int) (shares []slip39words.Share, ok bool)
     ```
     Pass `selectForCombine(byGroup, GT)`'s result to `Combine`, NOT the raw accumulation. (A
     flat slice with a stray partial group makes `Combine` return `errInsufficientShares` on a
     genuinely-sufficient pile — the bug plan-R0 I1 caught; the single-group idx-3 fixture cannot
     surface it, hence the unit test + the multi-group GUI test in Step 1bis below.)
     - **Dead-end affordance:** Back at any collection prompt unwinds/cancels recovery
       (`return nil, false`) — the user re-enters from the share they hold.
  4. **High-e gate (§5.6):** if `first.IterationExp >= 4`, `ConfirmWarningScreen` with the
     estimated wait; abort on cancel.
  5. **Optional SLIP-39 passphrase (§5.5):** a `ChoiceScreen` "SLIP-39 passphrase? (NOT a BIP-39
     passphrase)" default **Skip** (index 0), warning "A wrong passphrase silently recovers a
     different seed." If entered, a fresh `NewPassphraseKeyboard` (Slice 2) → `pass`; Skip → `""`.
  6. "Recovering…" frame; `secret, err := slip39words.Combine(sel, []byte(pass))`. Error →
     `showError(…, slip39words.Describe(err))`; `return nil, false`. Success: `m := bip39.New(secret)`;
     wipe the local `secret` slice; `return m, true`.

- [ ] **Step 1bis (I1 coverage): unit-test `selectForCombine`** directly (no GUI drive needed):
  (a) a single satisfied group → its members; (b) a stray partial group present alongside GT
  satisfied groups → pruned, returns only the GT groups' members and `ok=true`; (c) fewer than
  GT satisfied → `ok=false`. Plus a **multi-group GUI happy-path** test using a D1 fixture with
  a real group threshold (the `group(2-of-3 over 2-of-3 groups, GT=2)` topology in
  `slip39/testdata/slip39_fixtures.json`) round-tripping to its `secret_hex`.

- [ ] **Step 4:** Run → PASS; vet/gofmt clean. **Test-driving note (M1):** `inputSLIP39Flow`
  accepts each word only on `Button3` after the typed prefix is unambiguous (`completeSLIP39Word`
  → complete when `nvalid==1` or an exact match; `gui.go:821-839,963`). So `driveRecover` must,
  per word, `runes(&ctx.Router, <disambiguating prefix>)` then `click(&ctx.Router, Button3)` —
  ~20 (or 33) words × (runes+Button3) per share. Build a `driveShare(t, ctx, mnemonic string)`
  helper that emits the right per-word prefix+accept sequence (derive the shortest unambiguous
  prefix per word from the wordlist); reuse it across the recover tests. Budget TDD time for
  this — it is the heaviest part of D2's tests. `parseFixtureShare`/`vec3Share`/`hexOfEntropy`
  are small helpers (ParseShare+Fatal; load a mnemonic from `slip39/testdata/slip39_vectors.json`
  / `slip39_fixtures.json`; `hex.EncodeToString(m.Entropy())`).
- [ ] **Step 5: Commit** → `feat: slip39 two-level recover flow (roster + selectForCombine + passphrase)`.

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
       show a screen *"Fingerprint %.8X — confirm against your records before engraving"*
       (`fmt.Sprintf("%.8X", mfp)` — match `backupWalletFlow`'s `%.8X` format, M3); a Back here →
       `continue`, Engrave/OK → proceed.
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
- Two-level roster stops EXACTLY at `GT` satisfied groups; a share for an already-satisfied
  group is rejected (no over-fill); the `Combine` input is built by `selectForCombine` =
  exactly the GT satisfied groups' members (partial/extra groups pruned — plan-R0 I1), proven
  by the `selectForCombine` unit test (incl. the stray-partial-group case) + a multi-group GUI
  round-trip; dead-end has a cancel path; subsequent shares sized to `len(first.Mnemonic)`.
- The `TestRecoverSLIP39` Skip-path assertion is the EMPTY-passphrase secret
  (`61cf…2664`), and the TREZOR-path test asserts `b43c…0864` (plan-R0 C1 — empirically
  verified the two differ).
- `slip39_polish.go` imports both `bip39` and `chaincfg/v2` (the §5.4 fingerprint); `gui.go`
  gains no new import.
- The §3 acknowledgement is a hold-to-confirm BEFORE engrave; the §5.4 fingerprint is shown
  before engrave; the SLIP-39 passphrase prompt defaults to Skip with the silent-wrong-seed
  warning and is labeled distinctly from the BIP-39 25th-word passphrase.
- The recovered seed engraves via `backupWalletFlow` (no parallel confirm/engrave path);
  `backupWalletFlow`/`masterFingerprintFor` bodies UNCHANGED.
- `gui.go` gains no new import; `fmt`-using code is in `slip39_polish.go`.
- Signed + DCO + Brian Goss on every commit; the D1 crypto + GUI guards stay green.
