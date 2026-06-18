# cycle-prep recon — 2026-06-18 — codex32-multishare-recovery (Cycle B)

**Source repo:** the SeedHammer fork `bg002h/seedhammer` (`/scratch/code/shibboleth/seedhammer`), branch `main` @ **`bf7f811`** (post-Cycle-A1).
**Protocol source:** BIP-93, fetched + quoted verbatim.
**Predecessor recons:** `design/cycle-prep-recon-codex32-slip39.md` + `design/RECON_seedhammer_slip39_codex32_input.md` (written vs `599ec9a`; their `codex32/` line numbers still ACCURATE, `gui/gui.go` numbers DRIFTED by A1 — refreshed below).
**Phase:** ultracode recon — external protocol facts (BIP-93 multi-share recovery) verified against authoritative spec text, not just the draft. Feeds the Cycle B brainstorm; the R0 spec/plan gates still follow.

---

# RECON — Cycle B: codex32 multi-share (k-of-n) recovery

**Source verified against:** fork `/scratch/code/shibboleth/seedhammer` @ `main` / `bf7f811` ("Merge feat/codex32-input-polish ... Cycle A1"). Every fork citation below was re-checked against THIS tree.
**Protocol source:** BIP-93 (`bitcoin/bips/master/bip-0093.mediawiki`), fetched and quoted verbatim.
**Drift baseline:** the prior recons (`cycle-prep-recon-codex32-slip39.md`, `RECON_seedhammer_slip39_codex32_input.md`) were written vs `599ec9a`. Their `codex32/` line numbers are still ACCURATE (the core package was untouched by A1); their `gui/gui.go` line numbers are uniformly DRIFTED (A1 reshaped that file and added `gui/codex32_polish.go` + `codex32/polish.go`). New line numbers given throughout.

---

## 1. codex32 recovery API (fork source)

### `Interpolate` — the GF(32) Lagrange recovery
`codex32/codex32.go:185-188` — **ACCURATE** (stale recon said `:188`; unchanged):
```go
// Interpolate a set of shares to derive a share at a specific index.
//
// Using the index 'S' will recover the master seed.
func Interpolate(shares []String, index rune) (String, error) {
```
Takes a **slice of shares** + a **target index rune** (`'S'` recovers the secret; any other valid bech32 char derives that share). Returns `(String, error)`. If a provided share already sits at the target index it is returned directly (`codex32.go:219-222`), avoiding a multiply-by-zero.

### gui-caller gap — CONFIRMED ZERO gui callers
Tree-wide `grep "Interpolate("` (verified directly):
- `codex32/codex32.go:188` — declaration
- `cmd/biptool/main.go:127` — `codex32.Interpolate(shares, 'S')` (recover secret in `derive`)
- `cmd/biptool/main.go:334` — `codex32.Interpolate(shares, shareIdx)` (derive arbitrary share)
- `codex32/codex32_test.go:49`, `cmd/biptool/main_test.go:137` — tests

`grep -rln "Interpolate(" gui/` → **no match (exit 1)**. **ACCURATE.** The device collects exactly one share and engraves it verbatim; it never reconstructs. This is the entire correctness gap.

### Cross-share validation sentinels (all `codex32/codex32.go`)
Definitions `:24-37`; trigger sites verified. **All ACCURATE** (stale recon cited the `:201-230` range — still correct):

| Sentinel | Def | Triggered at | Condition |
|---|---|---|---|
| `errInsufficientShares` | :31 | :191 / **:229** | (a) `len(shares)==0`; (b) `s0Parts.threshold > len(shares)` — fewer shares than k |
| `errInvalidShareIndex` | :27 | :195 | target `index` rune is not a valid bech32 char |
| `errMismatchedLength` | :32 | :202 | `len(shares[0].s) != len(share.s)` (differing total string length) |
| `errMismatchedHRP` | :34 | :205 | `s0Parts.hrp != parts.hrp` |
| `errMismatchedThreshold` | :35 | :208 | `s0Parts.threshold != parts.threshold` |
| `errMismatchedID` | :33 | :211 | `s0Parts.id != parts.id` (4-char identifier) |
| `errRepeatedIndex` | :37 | :245 | two input shares share an index (`idxi == idxj`, `i != j`) |
| `errInvalidIDLength` | :36 | :281 | (in `NewSeed`, not Interpolate) `len(id) != 4` |

These 6 (the first 6 mismatch/insufficient/repeated) are the cross-share errors the recovery UI must surface. Note the **order**: length → hrp → threshold → id are checked in the first pass (`:199-214`); the threshold-vs-count check is **after** (`:228`); repeated-index surfaces only mid-interpolation (`:245`).

### How "number of shares required" (k) is determined
The threshold digit is parsed from the **first data character** (position after the `1` separator) in `partsInner` (`codex32/codex32.go:127-173`) — **ACCURATE**:
```go
switch t := res[0]; t {
case '0': thres = 0
case '2': thres = 2
... case '9': thres = 9
default: return nil, errInvalidThreshold   // '1' is invalid
}
```
Exposed to callers via `String.Split() (id, threshold, idx)` at `:394` (note: `Split` remaps a stored threshold `0`→`1` for display, `:397-399`) and via the A1-added `codex32.ParsePrefix → Fields.Threshold/ThresholdKnown` (`polish.go:70`). So **k is known from the first valid share's header** before any others are entered. The recovery loop accepts `len(shares) >= k` — see §2.

### `Interpolate(shares,'S')` → secret String → bytes
`Interpolate(shares,'S')` returns a `codex32.String` that **IS the unshared secret** (HRP copied from inputs e.g. `ms`, threshold `0`, index `S`) — confirmed by `codex32/codex32_test.go:68-78` (recovers `MS12NAMES...` and its 16-byte payload `d1808e09...`). The header-build at `codex32.go:261-275` writes `hrp + '1' + result fes`.

**Path to seed bytes exists and is exercised:**
- `func (s String) Seed() []byte` — `codex32.go:386` → `s.parts().data()`
- `func (p *parts) data() []byte` — `codex32.go:417` (5-bit→8-bit, right-pads final byte)
- `cmd/biptool/main.go:127-132` proves the full round-trip: `Interpolate(shares,'S')` → `k.Seed()` → `hdkeychain.NewMaster(seed, …)`.

**No `Decode`/`Payload`/`Entropy` method exists** — `Seed()` (via internal `parts().data()`) is the *only* byte-extraction path; `String` has only `Seed()`, `Split()`, `String()` exported (type def `codex32.go:15-18`, single unexported field `s string`). **ACCURATE.**

---

## 2. BIP-93 recovery semantics (verbatim from fetched spec)

From **`===Recovering Secret===`**:
> "When the share index of a valid codex32 string (converted to lowercase) is not the letter "s", we call the string a codex32 share. The first character of the data part indicates the threshold of the share, and it is required to be a non-"0" digit.
> In order to recover a secret, one needs a set of valid shares such that:
> * All shares have the same threshold value, the same identifier, and the same length.
> * All of the share index values are distinct.
> * The number of shares is exactly equal to the (common) threshold value."

- **k shares needed = the threshold digit** common to the shares. Spec defines the threshold parameter: *"a single digit between "2" and "9", or the digit "0""*; *"If the threshold parameter is "0" then the share index ... MUST have a value of "s" (or "S")."*
- **index `s`/`S`** = the unshared secret: *"When the share index ... is the letter "s", we call the string a codex32 secret."* Recovering at index `s` produces that secret.
- **Must be consistent across shares:** threshold, identifier, length (HRP `ms` is implied identical). **Indices must be distinct.** → The fork's `Interpolate` checks length+hrp+threshold+id consistency and distinct indices, matching the spec exactly (§1 table). One nuance: BIP-93 says "the same length"; the fork enforces equal **total string length** (`:201`) which is equivalent given equal HRP.
- **Recovering MORE than k:** BIP-93's recovery uses Lagrange interpolation (`ms32_recover`/`ms32_interpolate`) — shares are points on a degree-(k-1) polynomial. The spec text literally requires *"exactly equal to the threshold"* for the canonical recover call, BUT the fork is **more permissive**: `Interpolate` errors only if `threshold > len(shares)` (`codex32.go:228`), i.e. it accepts `len(shares) >= k` and ignores extras beyond what Lagrange needs. **Design note:** providing extra shares is mathematically sound (over-determined but consistent), but the fork does NOT cross-check that extras lie on the same polynomial — they're folded into the sum. Surfacing "you have enough (k of k)" at exactly k is the clean UX; allowing >k is harmless only if every share is genuinely from the same set.
- **The recovered artifact** is the unshared secret as a codex32 `ms…s…` string (then optionally `Seed()` → master seed bytes). From **`===Unshared Secret===`**: the secret is decoded by *"converting the payload to bytes ... Translate the characters to 5 bits values ... Re-arrange those bits into groups of 8 bits. Any incomplete group at the end MUST be 4 bits or less, and is discarded."* — exactly what `parts.data()` does.

---

## 3. Existing firmware primitives for a multi-share flow

### `inputCodex32Flow` — single-share entry (post-A1)
`gui/gui.go:672` — **DRIFTED from stale `:623` (+49)**:
```go
func inputCodex32Flow(ctx *Context, th *Colors) (codex32.String, bool)
```
Body (`:672-753`): builds `newCodex32Keyboard`, OK now on **Button3** (`:675` — A1 changed this from Button2), parses every frame with `codex32.New` (gate) + `codex32.ParsePrefix` (advisory), shows status/field/feedback lines, returns `(share, true)` on accept or `(String{}, false)` on back. **Returns one share.** No loop, no list state.

**Consumed by `newInputFlow`** (`gui/gui.go:1990`, **DRIFTED from `:1887`**), `case 2:` at `:2009-2013`:
```go
case 2:
    s, ok := inputCodex32Flow(ctx, th)
    if ok { return s, true }
```
Then `uiFlow` passes the returned `any` to `engraveObjectFlow` (`gui.go:1437`).

### `engraveObjectFlow case codex32.String:` (post-A1, verbatim)
`gui/gui.go:1841-1854` — **DRIFTED from stale `:1724-1731`; now STRUCTURALLY CHANGED** (A1 added the confirm screen + return-true-on-cancel):
```go
case codex32.String:
    if !confirmCodex32Flow(ctx, th, scan) {
        // Recognized codex32 string, user declined to engrave — return true
        // (handled) like every other recognized case, NOT false (which the
        // caller maps to "Unknown format").
        return true
    }
    id, _, _ := scan.Split()
    s := backup.SeedString{
        Title: id,
        Seed:  scan.String(),
        Font:  constant.Font,
    }
    backupSeedStringFlow(ctx, th, s)
```
`engraveObjectFlow` signature: `func engraveObjectFlow(ctx *Context, th *Colors, obj any) bool` (`:1806`). Still engraves the share **verbatim** (`scan.String()`); only `id` from `Split()` used. `confirmCodex32Flow` (`gui/codex32_polish.go:72`) already shows, for a single share: *"Share C of a k-of-n set / engraves THIS share, not a recovered seed"* (`:79-80`) — i.e. A1 already added a textual warning that this is NOT recovery. That warning string is the natural hook for "to recover, enter more shares."

### Reusable building blocks
- **`ChoiceScreen`** (`gui/gui.go:1282` type; `Choose(ctx, th) (int, bool)` at `:1296`) — menu/branch; would back a "Scan another / Recover now" loop.
- **N-of-M sequential collection precedent: `inputWordsFlow`** (`gui/gui.go:539`): drives a counter `selected` / `len(mnemonic)`, advancing `selected++` until `selected == len(mnemonic)` (`:603-604`), with title `layoutTitlef(..., "Word %d of %d", selected+1, len(mnemonic))` (`:660`). This is the closest existing "item i of N" pattern to clone for "share i of k".
- **`SeedScreen.Confirm(ctx, th, mnemonic) bool`** (`gui/gui.go:2037` type, `:2042` method) — post-collection review screen with per-item re-edit (Button2) and confirm-all (Button3); model for a "review collected shares / remove one / recover" screen.
- **`ConfirmWarningScreen.Layout`** (`gui/gui.go:215`) — hold-to-confirm (1s) gated action; **`ErrorScreen.Layout`** (`gui/gui.go:198`) — dismissible modal for surfacing a mismatch sentinel.
- **`layoutTitlef`** (`gui/gui.go:1637`) — formatted titles ("Share %d of %d").
- **NFC scan is single-shot, NOT a loop:** `scanner.Scan(r) (any, error)` (`gui/scan.go:26`) decodes one tag; codex32 branch at `gui/scan.go:68` (`codex32.New` → returns one `codex32.String`). The continuous "scan again" behavior lives in `StartScreen.Flow`'s goroutine (`gui.go:~1468`) but it merges/replaces, it does **not** accumulate a list. **No existing "collect N items" loop for shares exists** — it must be built.
- A1's reusable codex32 helpers (`codex32/polish.go`): `ParsePrefix(frag) (Fields, error)` (`:70`), `Fields` struct with `Identifier/Threshold/ShareIndex/Unshared` + `*Known` flags (`:51-60`), `Describe(err) string` (`:26`) mapping every sentinel to a short label, and exported length consts (`:16-21`). These cover per-share header display AND error surfacing for the recovery collector with no new package work.

### mdmk scope
`codex32/mdmk.go` (PR #35 md1/mk1) and `gui/mdmk_gui_test.go` contain **zero** references to `Interpolate`/Shamir/recover/secret-share (grep verified). Pure BCH verifiers, engraved verbatim — **unrelated to recovery, out of scope.**

---

## 4. Design space + sizing (for the brainstorm)

**No new crypto needed.** `Interpolate` + `Seed()` are present, correct, BIP-93-conformant, and already run on-device-grade Go (proven by biptool). A1 added the partial-parse + error-describe helpers. This is a **GUI-flow + plumbing** cycle. — CONFIRMED.

### Collection UX
- Build a **share-collection loop** (new — no existing list collector). Cleanest: reuse `inputCodex32Flow` per share inside an outer loop; after the first valid share, read **k** from `ParsePrefix(share).Threshold` and drive a "Share i of k" counter (clone `inputWordsFlow`'s `selected`/`len` + `layoutTitlef` pattern, `gui.go:660`).
- After each share, show a `SeedScreen.Confirm`-style review list of collected shares with **remove/redo** (Button2 per item) and a "Recover" action enabled once `len(shares) >= k`.
- k-display: known after share 1 (`Fields.ThresholdKnown`). Show "id NAME · thr k · share X" via the existing `codex32FieldLine`/`ParsePrefix` (`polish.go:36/70`).
- Decide: allow exactly k (recommended — matches BIP-93 canonical recover and avoids the unchecked-extra-share subtlety in §2) vs `>=k`.

### Validation surfacing
- **Eagerly per added share** (cheap, best UX): on adding share *j*, re-run the §1 consistency checks against share 0 (or just call `Interpolate(collected, firstIdx)` and map the returned sentinel via `codex32.Describe`-style labels) and reject with an `ErrorScreen` showing "mismatched id" / "mismatched threshold" / "repeated index" etc. The six sentinels (`errMismatched{Length,HRP,Threshold,ID}`, `errRepeatedIndex`, `errInsufficientShares`) all need user-facing strings — `Describe` (`polish.go:26`) currently covers the single-share `New` errors but **NOT** the cross-share mismatch sentinels, so a small `Describe` extension (or a sibling classifier) is the only codex32-package change likely needed.
- A duplicate-index can only be detected mid-`Interpolate` (`:245`); surfacing it eagerly means comparing `Fields.ShareIndex` on add.

### What to engrave after recovery (the consequential decision)
- **Option A — engrave the recovered unshared-secret codex32 `S` string verbatim.** `Interpolate(shares,'S')` → feed the resulting `codex32.String` straight into the **existing** `case codex32.String:` engrave path (`gui.go:1848-1854`, `backupSeedStringFlow`). *Requires:* the collection loop + `Interpolate` call only; reuses all existing engrave/confirm code; artifact stays in codex32 form. **Lowest risk, smallest diff.** Downside: the user gets back a single codex32 secret plate (still requires a codex32-aware wallet to restore), not a BIP-39 mnemonic.
- **Option B — decode to BIP-39 / SeedQR and engrave that.** `Interpolate(shares,'S').Seed()` → entropy bytes → BIP-39 mnemonic / SeedQR. *Requires:* a codex32-payload-bytes → BIP-39-mnemonic conversion (entropy→words; the `bip39` package exists but a `EntropyToMnemonic` path must be located/added), choosing the engrave artifact type (`backup.Seed`/SeedQR vs SeedString), and plate-fit handling. **Bigger scope, changes artifact type, new conversion + its own test vectors.** Higher risk.
- **Recommendation for brainstorm:** Option A is the lower-risk default and reuses the just-polished single-share engrave path; Option B is a strictly larger follow-on. The two are separable — A can ship first.

### Menu placement
`newInputFlow`'s `ChoiceScreen` is `{"12 WORDS", "24 WORDS", "CODEX32"}` (`gui.go:1995`). Two options: (a) a new top-level choice "RECOVER CODEX32" (clear intent separation; entering one share vs k shares are different user goals); or (b) make the existing CODEX32 path branch — after the first share, if its index is a *share* (not `S`) offer "engrave this share / recover from k shares". Option (b) reuses A1's confirm-screen warning hook ("engraves THIS share, not a recovered seed", `polish.go:80`) and is the more discoverable place to convert that dead-end warning into an action.

### Sizing
- **Net-new:** share-collection loop + per-share consistency surfacing + recovery-result screen ≈ **~120-220 LoC** in `gui/` (mostly cloning `inputWordsFlow`/`SeedScreen` patterns) + a `codex32.Describe` extension for the mismatch sentinels (~15 LoC) + tests (clone `gui/codex32_input_test.go`, add multi-share BIP-93 vectors — already mirrored in `codex32/codex32_test.go`).
- **Option A** keeps it at the low end (reuses engrave path); **Option B** adds the codex32→BIP-39/SeedQR conversion + artifact-type wiring (another ~M).
- **No crypto, no resource concern** (contrast SLIP-39). R0 gate applies before any code per project standard.

### Open questions for the brainstorm
1. Engrave Option A vs B (the security/product call).
2. Menu: new "RECOVER" entry vs branch the existing CODEX32 path.
3. Accept exactly k vs `>=k` shares (BIP-93 canonical = exactly k; fork `Interpolate` permits `>=k` but does not cross-validate extras).
4. Eager per-share validation vs validate-at-recovery (eager recommended; needs cross-share labels in `Describe`).
5. Plate-fit for the recovered artifact (the existing codex32 path has no plate-fit guard — flagged in the prior recon's open questions).
