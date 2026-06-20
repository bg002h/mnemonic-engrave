# SPEC — T7b-extension: BIP-85 custom (typed) hardened child index

**Status:** DRAFT — awaiting opus R0 gate (must reach 0C/0I before any code).
**Type:** brainstorm SPEC (design + load-bearing correctness lock). NOT a plan, NOT code.
**Fork base (source of truth):** `bg002h/seedhammer`, branch `main`, HEAD `8459654` (`84596549228116ac`). Go via `export PATH=$PATH:/home/bcg/.local/go/bin`. All file:line cites below verified LIVE against this HEAD (the recon's line numbers had drifted; the cites here are re-anchored).
**Feeds from:** `design/cycle-prep-recon-t7b-bip85-index-entry.md` (CLEAN; one structural correction folded — a reusable digit-typing widget DOES exist) + `design/SPEC_seedhammer_T7b_bip85_derive.md` (the shipped flow this extends; R0-GREEN, merged) + `design/agent-reports/seedhammer-T7b-*`.
**Slug:** `seedhammer-t7b-bip85-index-entry` ("option 1": replace the bounded `0..9` index `ChoiceScreen` with a typed arbitrary hardened index).

---

## 1. Why / context

The shipped T7b `bip85Derive` program lets the operator type a master BIP-39 seed, pick child params, and engrave a deterministic BIP-85 BIP-39 child (words + SeedQR). The child **index** is currently a **bounded `ChoiceScreen` (0..9, default 0)** — a deliberate "no reusable numeric widget" stopgap (T7b R0-I-B). BIP-85 places no sub-range on the index beyond the BIP-32 hardened max `2^31-1`, so users wanting child indices > 9 are stuck.

This cycle replaces that bounded picker with a **typed decimal index entry** (full `0 .. 2^31-1`), keeping the application canonical (`39'`/lang `0'`, word count ∈ {12,18,24}) so children stay reproducible by biptool and any other BIP-85 wallet. The single load-bearing correctness item is the **index width/overflow guard**: `deriveBip85Child`'s `index` parameter is a bare Go `int` guarded only by `index < 0`. That is safe on 32-bit TinyGo (RP2350) where `int` cannot hold `≥2^31` without going negative, but **UNSAFE on the 64-bit host test target**, where a 64-bit `int ∈ [2^31, 2^63-1]` passes `< 0` and is then **silently truncated/wrapped by `uint32(index)+h`** into a different, wrong hardened element with no error. The fix is a width-safe validator at the typed-string→index boundary AND a defense-in-depth upper-bound guard inside `deriveBip85Child`, both independent of `int` width.

No new crypto, no new program, no new widget. This extends one existing program's picker + hardens one existing guard.

---

## 2. Scope

### IN
- **Replace the index `ChoiceScreen`** in `bip85ParamPickFlow` (`gui/bip85.go:128-135`, the index stage only) with a **typed numeric index entry flow** — a new `bip85IndexEntryFlow(ctx, th) (index int, ok bool)` cloned from `typeAddressFlow` (`gui/verify_address.go:44-71`), reusing `NewAddressKeyboard` (`gui/passphrase_keyboard.go:133`, cleartext — the index is not secret), title e.g. "Child index". Keep the word-count `ChoiceScreen` (`gui/bip85.go:118-127`) unchanged. Back (Button1) → `(0, false)`; OK (Button3) → parse `kbd.Fragment` (`gui/passphrase_keyboard.go:48`) through the validator. App stays `39'`, lang `0'`, word count ∈ {12,18,24}; output unchanged (child seed words + SeedQR via `engraveBip85Child`/`engraveSeed`).
- **A width-safe index validator** `parseBip85Index(s string) (int, error)` (THE load-bearing item): parse the typed decimal into a **width-safe fixed type (`uint64` or `int64`, NEVER a bare `int`)**; reject empty, non-`[0-9]` (any letter/symbol/sign/whitespace — the keyboard can type all of these, see §3), leading-junk, and `> 2147483647` (= `2^31-1` = `hdkeychain.HardenedKeyStart - 1`) with a clear on-screen error and re-prompt. Only a valid `0 ≤ index ≤ 2^31-1` proceeds. (Optional UX: cap input length early to bound the parse — `2^31-1` is 10 digits.)
- **Harden `deriveBip85Child`'s guard** (`gui/bip85.go:32-38`): add the missing UPPER bound — reject `index > 2147483647` (and keep `index < 0`) BEFORE the `uint32(index)+h` cast (`gui/bip85.go:54`), and make the parameter / internal handling width-safe so a host-side caller cannot silently truncate. Defense-in-depth: this guard holds even if the picker is bypassed (tests, future callers). Express the bound symbolically against `hdkeychain.HardenedKeyStart` (`= 0x80000000`), not a bare literal, to keep it self-documenting.
- **Re-pin `TestBip85ParamBounds`** (`gui/bip85_test.go:138-163`) for the typed-entry contract: the index axis is no longer the enumerated `bip85IndexChoices` slice but "any value the validator accepts, `0 ≤ n ≤ 2^31-1`". Retire/repurpose `bip85IndexChoices` (`gui/bip85.go:112`) — it is no longer the index source (keep `bip85WordChoices`).
- **Re-point** the index step of `TestBip85DeriveFlow_ScrubsBothMnemonics` (`gui/bip85_test.go:239`, currently `chooseEntry(frame, &ctx.Router, 0)` for the index) to drive the new typed-entry flow (type digit(s), press OK).

### OUT (deferred → `seedhammer-t7b-bip85-followups`)
- Other BIP-85 applications (`32'` XPRV / WIF / hex / RSA); a fully-arbitrary path (any app/depth); a non-English language picker. These stay deferred — they need a different engraved artifact and would break the canonical BIP-39 app shape (interop footgun). Keep `39'`/`0'`/{12,18,24} fixed so children stay reproducible.
- The shipped index-0 derive goldens, the engrave/fingerprint path, the security spine, the program lockstep — all UNCHANGED (no new program → no enum/dispatch/t5-M1-guard/nav-test/lockstep edit; this extends an existing program).
- Firmware-only: no `md`/`mk`/`codex32` edit, no new `me` CLI flag, no `schema_mirror`, no `docs/manual` mirror, no SemVer bump.

---

## 3. Verified facts (cited LIVE against HEAD `8459654`)

### Keyboard reuse (the typed-entry path)
- **`typeAddressFlow` is the clone target** — `gui/verify_address.go:44-71`: `kbd := NewAddressKeyboard(ctx)`; `backBtn := &Clickable{Button: Button1}`, `okBtn := &Clickable{Button: Button3}`; loop `for kbd.Update(ctx) {}`; **Back (Button1) → `return "", false`** (`:51-53`); **OK (Button3) → `return kbd.Fragment, true`** (`:54-56`). A numeric-index flow is a third copy of this shape (alongside `passphraseFlow`), substituting the title and the parse-on-OK step.
- **`NewAddressKeyboard(ctx) *PassphraseKeyboard`** — `gui/passphrase_keyboard.go:133-137`: same widget as the passphrase keyboard with `k.revealed = true` (cleartext readout; the index is not a secret).
- **The digit charset** — `gui/passphrase_keyboard.go:21`: `ppPageSymbols = "1234567890\n-/:;()&$@\"\n.,?!'+=_#"`. All ten digits `0-9` are typeable (on the symbols page, reachable via the page-cycle key `ppPageCycleLabel = {"ABC","?123","abc"}`, `:27`).
- **`kbd.Fragment` returns the typed string** — exported field `gui/passphrase_keyboard.go:48` (`type PassphraseKeyboard struct { Fragment string … }`). `commit` appends each committed rune verbatim (NO ToUpper) — `:189-206` (`ppRune: k.Fragment += string(key.r)`; `ppBackspace` deletes the last rune). `Clear()` resets `Fragment=""` — `:171-178`.
- **CRITICAL UX FACT (drives validator design):** the keyboard does **NOT** restrict input to digits. `commit` appends ANY `ppRune` key (letters, symbols, space) and the cross-page rune handler (`Update`, `:247-258`) commits ANY rune that matches a key on ANY of the three pages. There is no digit-only keyboard variant (`NewKeyboard(ctx, "1234567890")` at `gui/gui.go` would be one, but its `Update` is coupled to BIP-39/SLIP-39 word completion via its consumers — out of scope to refactor). **Therefore digit-only enforcement is the VALIDATOR's job, not the keyboard's** — the validator MUST reject any non-`[0-9]` `Fragment`. This is folded from the recon and re-verified here.

### Derive guard + path walk (the correctness path)
- **`deriveBip85Child(m bip39.Mnemonic, passphrase string, words, index int) (bip39.Mnemonic, error)`** — `gui/bip85.go:32`. **Guards only `index < 0`** today: `gui/bip85.go:36-38` (`if index < 0 { return nil, fmt.Errorf("bip85: invalid index: %d", index) }`). **NO upper bound.**
- **The hardened path walk** — `gui/bip85.go:48-55`:
  ```go
  const h = hdkeychain.HardenedKeyStart
  path := []uint32{
      bip85.PathRoot,   // = 83696968 + 0x80000000 (bip85/bip85.go:11, verified)
      39 + h,
      0 + h,
      uint32(words) + h,
      uint32(index) + h,   // <-- silent truncation site for a 64-bit index ≥ 2^31
  }
  ```
  `uint32(index)+h` is the truncation point: a 64-bit `index ∈ [2^31, 2^63-1]` is masked to its low 32 bits before adding `h`, yielding a DIFFERENT hardened element with no error. The upper-bound guard MUST precede this cast.
- **`hdkeychain.HardenedKeyStart = 0x80000000` = 2147483648 = 2^31.** The hardened-max un-hardened index is `HardenedKeyStart - 1 = 2147483647 = 2^31-1`.
- **biptool's independent ceiling** (the interop reference) — `bip32.ParsePath`/`ParsePathElement` rejects index ≥ 2^31 (overflow check `iu32+offset < iu32`). PROBE-CONFIRMED below: biptool errors on `2147483648h` (`bip32: path element out of range`). The on-device validator's `2^31-1` ceiling matches biptool exactly → bit-for-bit reproducible.

### Engrave path (m*-free)
- **`engraveBip85Child(params, child) (Plate, uint32, error)`** — `gui/bip85.go:94-104`: computes the CHILD's own bare fp via `masterFingerprintFor(child, &chaincfg.MainNetParams, "")` (`:95`) then `engraveSeed(params, child, mfp)` (`:99`) — the Backup-Wallet seed-words + standard SeedQR primitive. **Unchanged by this cycle.**
- **m\*-free CONFIRMED:** `grep` for `seedhammer.com/codex32` / `/md` / `/mk` across `gui/bip85.go`, `gui/bip85_test.go`, `bip85/bip85.go` → zero matches (exit 1). `gui/bip85.go` imports only `errors`, `fmt`, btcd `hdkeychain`+`chaincfg`, and `seedhammer.com/{bip39,bip85,engrave,gui/assets,gui/op}`. No `md1`/`mk1`/`ms1` path anywhere on the BIP-85 surface.

### PROBE (R0-verified, computed at HEAD `8459654`)
Abandon master `abandon abandon abandon … about` → master xprv `xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu`. Computed two ways — (a) the in-tree primitive (`bip39.MnemonicSeed`→`hdkeychain`→`bip85.Entropy`→`bip39.New`) and (b) **independently** via `biptool derive -path … bip39 -words 12` (a SEPARATE code path through `bip32.ParsePath`):
- `m/83696968'/39'/0'/12'/1'` → `sing slogan bar group gauge sphere rescue fossil loyal vital model desert` — **matches the shipped `TestDeriveBip85Child_IndexVaries` golden** (harness fidelity proof).
- **`m/83696968'/39'/0'/12'/2147483647'` (= 2^31-1, the boundary) → `jewel solution patient quarter elite grace quarter dinosaur taste parent dial clump`** — byte-identical across both derivations. **This is the high-index golden to pin.**
- `m/83696968'/39'/0'/12'/2147483648'` (= 2^31) → biptool **rejects** (`bip32: path element out of range`). Confirms `2^31-1` is the canonical hardened max; the device must reject `≥2^31`.

---

## 4. Invariants (numbered — R0 confirms)

- **I-1 (derivation faithful):** the child is derived by the canonical fully-hardened path `m/83696968'/39'/0'/{words}'/{index}'` (index hardened, all elements ≥ `HardenedKeyStart`), `bip85.Entropy` over the leaf's 32-byte privkey, child = the LEADING `entLen=(n*11-n/3)/8` bytes → byte-identical to biptool/canonical BIP-85 at the TYPED index. The typed-index change widens only the index axis; the walk and truncation are unchanged. A wrong path/index → divergent child (silent-wrong-backup) — refuse via guard tests.
- **I-2 (Critical — width-safe parse, the load-bearing item):** the typed decimal is parsed into a **width-safe fixed type (NOT a bare `int`)** and **rejected if `> 2^31-1`**, with **NO silent `uint32()` truncation/wrap on ANY target** (32-bit TinyGo OR 64-bit host). This guard exists BOTH at the picker (`parseBip85Index`) AND inside `deriveBip85Child` (the upper-bound guard before `uint32(index)+h`). On the 64-bit host, `index ∈ [2^31, 2^63-1]` MUST error, never derive. Non-numeric / empty / leading-junk / signed input MUST error.
- **I-3 (m\*-free + firmware-only):** zero `md`/`mk`/`codex32` edit; no new program; no new widget (reuse `NewAddressKeyboard`); no enum / t5-M1 compile-guard / nav-test / 8-site lockstep edit; no `me` CLI flag / `schema_mirror` / `docs/manual` mirror / SemVer bump.
- **I-4 (security spine unchanged):** typed-only master seed (never scan→derive); per-leg scrub of master + child mnemonics + privkey/HMAC buffers on every exit (existing T7b discipline, untouched); child engraved onto owner steel only, never NFC; mainnet-only. The typed-index path adds no secret (the index is public) and touches none of the scrub buffers.

---

## 5. Acceptance gate (TDD — tests before impl; reviewer-loop to 0C/0I after every fold)

**Validator unit tests (`parseBip85Index`):**
1. `""` → error (empty).
2. `"0"` → `0`; `"9"` → `9`; `"1000000"` → `1000000` (representative interior values accepted).
3. `"2147483647"` → `2147483647` (= 2^31-1, the boundary — ACCEPTED).
4. `"2147483648"` → error (= 2^31, first out-of-range value — REJECTED).
5. `"4294967296"`, `"9223372036854775808"`, a value `> 2^63` → error (no overflow/wrap; width-safe).
6. `"12a"`, `"a12"`, `" 12"`, `"12 "`, `"+1"`, `"-1"`, `"1.0"`, `"0x10"`, `"١٢"` (non-ASCII digits) → error (non-`[0-9]`/leading-junk/sign/whitespace, all typeable on the keyboard).
7. `"00"`, `"007"` → DECIDE in R0: accept as `0`/`7` (leading zeros) OR reject. (Recommend ACCEPT — leading zeros are unambiguous decimal; flag as the one open UX question.)

**Derive guard tests (`deriveBip85Child` — defense-in-depth, independent of the picker):**
8. **Pinned high-index golden (PROBE-VERIFIED):** `deriveBip85Child(abandonAboutMnemonic(), "", 12, 2147483647)` → `jewel solution patient quarter elite grace quarter dinosaur taste parent dial clump` (the §3 probe value, cross-checked against biptool — byte-identical). Re-probe-verify at implementation time against the then-current HEAD.
9. `deriveBip85Child(…, 12, 2147483648)` → error (upper-bound guard; on the 64-bit host this value fits an `int` and would otherwise truncate). Also a 64-bit `int` `> 2^31` (e.g. `1<<40`) → error.
10. `deriveBip85Child(…, 12, -1)` → error (existing lower-bound, retained — keep `TestDeriveBip85Child_RejectsNegativeIndex`).
11. `deriveBip85Child(…, 12, 0)` → UNCHANGED vs the existing index-0 golden (`TestDeriveBip85Child_AbandonGoldens` / `_CanonicalVector` stay green; index 0 remains reachable).

**Flow / picker tests:**
12. **Re-pin `TestBip85ParamBounds`** (`gui/bip85_test.go:138`): drop the `bip85IndexChoices` len==10 / `[i]==i` enumeration; assert instead that the validator accepts `0` and `2^31-1` and rejects `2^31` / non-numeric (the typed-entry contract). Keep the `bip85WordChoices == [12 18 24]` assertion and the "every (words, accepted-index) pair derives a valid child" sweep over representative indices.
13. **Re-point `TestBip85DeriveFlow_ScrubsBothMnemonics`** (`gui/bip85_test.go:206`, index step at `:239`): drive `bip85IndexEntryFlow` — type a digit (e.g. `"0"` to keep the rest of the assertions stable, or a high index to exercise the typed path), press OK (Button3), and assert the flow proceeds. The two-secret scrub assertions stay green (I-4).
14. **`FuzzDeriveBip85Child`** (`gui/bip85_test.go:293`): grow the corpus with upper-bound seeds (`2^31`, `2^31-1`, a 64-bit `int ≥ 2^31`) and tighten the success-path assertion (`:307`) so accepting any `index > 2^31-1` is a test failure (currently it only checks `index < 0`).
15. **No-regression:** the rest of the T7b flow (master entry, passphrase, warning gate, engrave + child fp golden `0x02e8bff2`, two-secret scrub) byte-unchanged; `TestEngraveBip85Child_UsesChildFP`, `TestChildSeedWarningAbort`, `TestAllocs` green; Backup Wallet / T4 / T6 / codecs unaffected.

**Note:** the pinned high-index golden (test 8) MUST be computed AND R0-probe-verified at implementation time (re-run the §3 probe against the then-current HEAD; a derive-library bump could move it). The §3 value is the probe result at HEAD `8459654`.

---

## 6. Risks

1. **Int-width / overflow footgun (I-2, the #1 load-bearing risk).** `index int` is target-dependent: silent `uint32()` truncation on the 64-bit host for `index ≥ 2^31`. MITIGATION: parse into a width-safe fixed type, reject `> 2^31-1` at BOTH the picker and inside `deriveBip85Child` before the cast; tests 5/8/9 + the fuzz upper-bound case lock it. Do NOT rely on `int` width as the guard.
2. **Keyboard-reuse UX.** The address keyboard types letters/symbols/sign/space, not just digits — the validator MUST reject all non-`[0-9]` `Fragment`s (test 6) and re-prompt with a clear error rather than deriving a wrong/garbage index. Back/OK semantics (Button1/Button3) and clear/backspace come free from the `typeAddressFlow` clone. UX sub-decision for R0: empty-`Fragment` on OK → re-prompt (treat as the empty-input error), not a silent `0`.
3. **Re-pinning the bounds test.** `TestBip85ParamBounds` currently asserts the enumerated `0..9` slice; it must flip to the validator-contract form without weakening the "in-spec child only" guarantee. Risk: leaving a dead `bip85IndexChoices` referenced elsewhere — grep for all references when retiring it.
4. **Golden staleness.** The high-index golden (test 8) is pinned from a probe; it must be re-probe-verified at implementation time (note in §5).

---

## 7. Open question for R0
- **Leading-zero policy** (test 7): accept `"007"` as `7` (recommended — unambiguous decimal, matches `strconv.ParseUint` base-10 behavior) or reject as malformed? Lock this in R0 so the validator + tests agree.

---

## 8. Gate

This SPEC MUST pass an opus architect R0 review to **0 Critical / 0 Important** before ANY implementation. Fold findings → persist the review verbatim to `design/agent-reports/` → re-dispatch after every fold (folds can drift) until GREEN. Then: implementation plan → its own R0 → single-implementer TDD in a worktree → mandatory whole-diff adversarial execution review → merge no-ff (signed + DCO, authored Brian Goss) → push `bg002h`. No code before this gate is GREEN.
