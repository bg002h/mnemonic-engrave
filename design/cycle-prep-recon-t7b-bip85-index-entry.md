# cycle-prep recon — 2026-06-19 — seedhammer-t7b-bip85-index-entry

**Origin/master SHA at recon time:** `8459654` (fork `bg002h/seedhammer`, ref `origin/main` — this repo uses `main`, no `master`)
**Local branch:** `main`
**Sync state:** up-to-date (0 ahead / 0 behind `origin/main`)
**Untracked:** none (clean working tree)

Slug verified: `seedhammer-t7b-bip85-index-entry` (registered as **deferred scope** under `### seedhammer-t7b-bip85-followups` in `seedhammer/design/FOLLOWUPS.md`, not a standalone `###` entry). This is "option 1": replace the bounded `0..9` index `ChoiceScreen` with a **typed arbitrary hardened index** in the on-device BIP-85 derive-child flow, app fixed `39'`/lang `0'`/words∈{12,18,24}, output = seed words + SeedQR. Hard requirement: **ZERO m\* codec changes**. Expectation going in: the FOLLOWUP's "NO reusable free-form numeric-entry widget exists" claim needed re-checking — it is **STRUCTURALLY-WRONG** (see below). All other facts ACCURATE.

---

## Per-slug verification

### seedhammer-t7b-bip85-index-entry

- **WHAT (from FOLLOWUPS.md):** "the on-device BIP-85 child index is a BOUNDED `ChoiceScreen` (0–9, default 0) because NO reusable free-form numeric-entry widget exists in the tree … A free/large index … would need a net-new numeric-entry widget — out of scope … build it if a user wants high child indices."

- **Citations / claims:**

  - **`gui/bip85.go:112` — `bip85IndexChoices = []int{0,1,2,3,4,5,6,7,8,9}`** — **ACCURATE.** Verified verbatim: `var bip85IndexChoices = []int{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}`. Also `bip85WordChoices = []int{12, 18, 24}` at line 111.

  - **`gui/bip85.go:117-139` — `bip85ParamPickFlow(ctx, th)` picks word count then bounded index via two `ChoiceScreen`s** — **ACCURATE.** Signature `func bip85ParamPickFlow(ctx *Context, th *Colors) (words, index int, ok bool)`. The index `ChoiceScreen` hard-codes `Choices: []string{"0".."9"}` (line 131) and returns `bip85WordChoices[wsel], bip85IndexChoices[isel]`. (Note: the FOLLOWUP body does not name `bip85ParamPickFlow`; the task prompt guessed the name and it is correct.)

  - **`gui/bip85.go:32` — `deriveBip85Child(m bip39.Mnemonic, passphrase string, words, index int)`** — **ACCURATE.** `index` is Go `int`. The path-walk (lines 49-55) is exactly:
    ```go
    const h = hdkeychain.HardenedKeyStart
    path := []uint32{
        bip85.PathRoot,   // 83696968 + 0x80000000
        39 + h,
        0 + h,
        uint32(words) + h,
        uint32(index) + h,
    }
    ```
    Hardening is `uint32(index) + h` (h = `hdkeychain.HardenedKeyStart` = `0x80000000`). `bip85.PathRoot = 83696968 + 0x80000000` (`bip85/bip85.go:11`) — ACCURATE.

  - **Index guard — lower bound present, UPPER bound ABSENT** — **ACCURATE (as the FOLLOWUP implies; flagged for option 1).** `deriveBip85Child` guards only `index < 0` (lines 36-38: `return nil, fmt.Errorf("bip85: invalid index: %d", index)`). There is **NO** `index < 2^31` / max-range guard — it is currently only ever fed `0..9` from the bounded picker, so the gap is latent. For option 1 (arbitrary index) this guard MUST be added.

  - **int 32-bit-overflow flag** — **CONFIRMED REAL — FLAG.** `index` is Go `int`. The fork's firmware target is TinyGo on RP2350 (ARM Cortex-M33, 32-bit), where Go `int` is **32-bit**. With a 32-bit `int`, `index` cannot even *hold* a value ≥ 2^31 without already being negative (sign bit), so the existing `index < 0` check coincidentally rejects the sign-flipped case **on 32-bit**. BUT: (a) on the 64-bit host test target, `int` is 64-bit, so a 64-bit `index` ∈ `[2^31, 2^63-1]` passes the `< 0` check, then `uint32(index)+h` silently truncates/wraps → a DIFFERENT, wrong hardened element with no error. (b) The correct, portable fix is an explicit upper-bound validator `0 ≤ index ≤ 2^31-1` BEFORE the cast, parsing into a wide type (e.g. `int64`/`uint64`) and rejecting out-of-range — never relying on `int` width. The validator must live at the typed-string→index parse boundary AND/OR inside `deriveBip85Child` (defense in depth; the fuzz test `FuzzDeriveBip85Child` should grow an upper-bound case).

  - **m\*-free claim (engrave-as-words app, no md/mk/codex32)** — **ACCURATE / CONFIRMED.** `grep` for `seedhammer.com/{md,mk,codex32}` across `gui/bip85.go`, `gui/bip85_test.go`, `gui/bip85_program_test.go`, `bip85/bip85.go`, `bip85/bip85_test.go` → **zero matches** (exit 1). `gui/bip85.go` imports only `errors`, `fmt`, btcd `hdkeychain`+`chaincfg`, and `seedhammer.com/{bip39,bip85,engrave,gui/assets,gui/op}`. The engrave path is `engraveBip85Child` (line 94) → `masterFingerprintFor(child,…,"")` + `engraveSeed(params, child, mfp)` (the exact Backup-Wallet seed-words + standard SeedQR primitive). **NOT** an md1/mk1/ms1 path. **VERDICT: option 1 is buildable with ZERO m\* edits.**

  - **"NO reusable free-form numeric-entry widget exists"** — **STRUCTURALLY-WRONG (the load-bearing finding).** Two reusable widgets already accept digit entry:
    1. **`PassphraseKeyboard` (`gui/passphrase_keyboard.go`)** — its symbols page is `ppPageSymbols = "1234567890\n-/:;()&$@\"\n.,?!'+=_#"` (line 21): **all ten digits 0-9 are typeable.** It exposes the typed string as the exported field `Fragment` (line 48). `NewAddressKeyboard` (line 133) is the same widget rendering in cleartext (digits are not secret).
    2. **`NewKeyboard(ctx, alphabet string)` (`gui/gui.go:938`)** builds a grid from an **arbitrary alphabet string** — `NewKeyboard(ctx, "1234567890")` would produce a digits-only keypad directly. However the `Keyboard` type's `Update`/completion logic is currently coupled to BIP-39/SLIP-39 word completion (`completeBIP39Word`/`completeSLIP39Word`, gui.go:1008/999) via its *consumers* (seed-word entry), not the constructor — the constructor itself is generic.
    The existing flow wrappers `passphraseFlow` (`gui/gui.go:509-536`) and `typeAddressFlow` (`gui/verify_address.go:44-67`) are ~30-line near-twins: build keyboard → `for kbd.Update(ctx) {}` loop → Back (Button1) returns `("",false)`, OK (Button3) returns `(kbd.Fragment, true)`. A numeric-index flow is a third copy of this exact shape. **Reuse is trivial; a net-new widget is NOT required.** This contradicts the FOLLOWUP and is the single most important correction for the brainstorm.

- **Action for brainstorm spec:** Build option 1 by adding a `bip85IndexEntryFlow(ctx, th) (index int, ok bool)` modeled on `typeAddressFlow`/`passphraseFlow` (reuse `NewAddressKeyboard` — cleartext, digits not secret), parse `kbd.Fragment` through a new validator, and **replace only the index `ChoiceScreen`** inside `bip85ParamPickFlow` (keep the word-count `ChoiceScreen`). Add an upper-bound guard in `deriveBip85Child` (`0 ≤ index ≤ 2^31-1`). Correct the FOLLOWUP's "no numeric widget" claim. Cite source SHA `8459654`.

---

## Protocol fact — BIP-85 index range (PRIMARY-SOURCE verified)

- **Spec (bitcoin/bips `bip-0085.mediawiki`, fetched 2026-06-19):** BIP39 application path is **`m/83696968'/39'/{language}'/{words}'/{index}'`**; the `{index}` element **is hardened** (apostrophe); BIP-85 imposes **no sub-range** on `{index}` beyond the standard BIP-32 hardened maximum **`2^31 - 1`**. Example sequence `…/12'/0'` → `…/12'/1'` with no stated upper bound. — **ACCURATE.**

- **Fork cross-check (`cmd/biptool/main.go` → `bip32.ParsePath`/`ParsePathElement`, `bip32/bip32.go:69-84`):** the index path element is parsed by `ParsePathElement`:
  ```go
  idx, err := strconv.ParseInt(p, 10, 0)   // base-10
  iu32 := uint32(idx)
  if int64(iu32) != idx || iu32+offset < iu32 {   // offset = HardenedKeyStart for 'h'/'\''
      return 0, "out of range"
  }
  return iu32 + offset, nil
  ```
  With `offset = 0x80000000`, the overflow check `iu32+offset < iu32` forces the **un-hardened index `iu32 ∈ [0, 2^31-1]`** (`2^31` would overflow uint32 after adding `0x80000000`). biptool's `derive bip39` additionally requires `path[4]` be exactly the hardened index element of a 5-element path (it does NOT cap the index any lower than `2^31-1`). **CONFIRMED: biptool caps at `2^31-1`, matching the BIP-85/BIP-32 hardened max.** This is the exact bound the on-device validator must enforce so the device stays bit-for-bit reproducible with biptool and any other BIP-85 wallet.

- **Validator spec (what option 1 needs):** parse decimal string → reject empty / non-`[0-9]` / leading-`+-`/whitespace; parse into a width-safe type (`uint64`/`int64`, NOT `int`); enforce `0 ≤ n ≤ 2147483647` (= `2^31 - 1` = `hdkeychain.HardenedKeyStart - 1`); then feed `int(n)` to `deriveBip85Child`, which itself re-guards the upper bound before `uint32(index)+h`. Optional UX: cap input length and reject overlong strings early.

---

## Test approach

- **Driving a typed-index flow under `synctest`:** mirror the passphrase keyboard tests. `gui/passphrase_keyboard_test.go`:
  - `TestPassphraseRuneEntryCrossPage` types via RuneEvents and asserts `k.Fragment` (digits are part of the cross-page rune set) — the closest existing pattern for "type characters, read the string."
  - `TestPassphrasePageCycleRender` (`runUI` + `uiContains(got, "1")`) confirms the symbols page renders digit `1` — proof the keyboard already shows digits.
  - `passphraseFrame`/`runUI` helpers + `press(&ctx.Router, <Button>)` are the frame-pump/input primitives.
  Existing BIP-85 flow test `TestBip85DeriveFlow_ScrubsBothMnemonics` (`gui/bip85_test.go:206`) already drives the full flow with `driveWords(...)`, `chooseEntry(frame, &ctx.Router, 0)` (word count) and `chooseEntry(... ,0)` (index) — the *index* `chooseEntry` call must be re-pointed at the new typed-index flow (type the digits, press OK) once the picker swaps.
- **Golden re-pin scope (smaller than the task prompt feared):** the existing derive goldens are all at **index 0**, which stays reachable under option 1, so they do **NOT** change:
  - `TestDeriveBip85Child_AbandonGoldens` (idx 0), `TestDeriveBip85Child_CanonicalVector` (idx 0), `TestEngraveBip85Child_UsesChildFP` (pinned child fp, abandon/12/idx 0) — **all unchanged.**
  - `TestDeriveBip85Child_IndexVaries` (idx 0 vs 1) — **unchanged.**
  - **`TestBip85ParamBounds` (`gui/bip85_test.go:138`)** asserts the picker bounds are word∈{12,18,24} × index∈{0..9} — this is the ONE test that must change (the index axis becomes "any parsed value 0..2^31-1," validated by the new parser rather than enumerated).
  - `FuzzDeriveBip85Child` (line 293) — grow the corpus with an **upper-bound** seed (e.g. `2^31`, `2^31-1`, a 64-bit `int` ≥ 2^31) to lock the new guard.
  - **New tests to add:** index-string validator unit tests (empty, non-numeric, leading sign, `0`, `2147483647`, `2147483648`→reject, huge), and a derive vector at a high index (e.g. `2147483647`) cross-checked against `biptool derive bip39 --path m/83696968h/39h/0h/12h/2147483647h`.

---

## Cross-cutting observations

1. **One structural error, one latent gap.** The FOLLOWUP's "no reusable numeric-entry widget" is **STRUCTURALLY-WRONG** — `PassphraseKeyboard`/`NewAddressKeyboard` already type digits 0-9; `NewKeyboard` takes an arbitrary alphabet. The "out of scope, needs a net-new widget" justification dissolves: option 1 is a ~small reuse, not a new widget. Separately, the absent upper-bound guard in `deriveBip85Child` is latent (masked today by the `0..9` picker) and becomes load-bearing the moment the index is typed.

2. **`int`-width portability is a genuine cross-target footgun.** `index int` is fine on 32-bit TinyGo (where ≥2^31 can't be non-negative) but UNSAFE on the 64-bit host test target (a 64-bit `int` ≥ 2^31 passes `< 0`, then `uint32()` truncates silently). Spec the validator to parse into a fixed-width type and reject `> 2^31-1` explicitly, independent of `int` width. Do NOT lean on `int` overflow as the guard.

3. **m\*-free is unconditional.** No `md`/`mk`/`codex32` import anywhere in the BIP-85 surface; the engrave path is the seed-words+SeedQR `engraveSeed` primitive. The hard requirement is satisfiable with zero codec edits — confirmed by grep + import audit, not just the doc.

4. **No new program → no lockstep.** This EXTENDS the existing `bip85Derive` program's picker. No new enum variant, so the **t5-M1 compile-time guard** (`var _ [1]struct{} = [qaProgram - bip85Derive]struct{}{}`, `gui/gui.go:164`) and the `npage/npages = int(bip85Derive)+1` consts (lines 1867/1886) are untouched. **No `schema_mirror`** (no clap flag — this is firmware GUI, not the `me` CLI), **no `docs/manual` mirror**, **no SemVer bump on the `me` crate** (firmware-only). Those mirrors are N/A here.

5. **Experimental posture preserved.** Option 1 keeps the canonical BIP-39 app (`39'`/`0'`), so children stay reproducible by any BIP-85 wallet (NOT the interop footgun of a fully-arbitrary path). Keep the existing unskippable `childSeedWarning` (`gui/bip85.go:145`) and any "experimental" framing; arbitrary index does not weaken reproducibility, only widens the (already-hardened, in-spec) index axis.

6. **FOLLOWUP registry note (claim-counting):** the slug is a sub-bullet of `### seedhammer-t7b-bip85-followups`, alongside two sibling deferred items (other BIP-85 applications `32'`/WIF/hex/`128169'`; and the inert M-1 switch-case ordering note). This recon scopes ONLY the index-entry item — do not conflate the sibling "other applications" deferral (which WOULD need new artifacts / md changes) with option 1 (which does not).

---

## Recommended brainstorm-session scope

**One small firmware-GUI cycle, single slug.** Build order:

1. **Validator (`parseBip85Index(s string) (int, error)`)** — decimal-only, reject empty/non-numeric/sign/whitespace, enforce `0 ≤ n ≤ 2^31-1` parsed via a width-safe type. + upper-bound guard inside `deriveBip85Child` before the `uint32(index)+h` cast (defense in depth). **~25-40 LOC + tests.**
2. **Input flow (`bip85IndexEntryFlow`)** — copy `typeAddressFlow` (`gui/verify_address.go`), reuse `NewAddressKeyboard` (cleartext, digits not secret), title "Child index", Back→`(0,false)`, OK→parse `Fragment`; on parse error re-prompt (showError / stay on screen). **~30-45 LOC.** (Net-new minimal numeric keypad NOT needed; if a digits-only keyboard is preferred for UX, `NewKeyboard(ctx, "1234567890")` is a one-liner alternative, but the address keyboard already works — recommend reuse.)
3. **Picker swap** — in `bip85ParamPickFlow` (`gui/bip85.go:117`), replace the index `ChoiceScreen` (lines 128-135) with a call to `bip85IndexEntryFlow`; keep the word-count `ChoiceScreen`. Retire `bip85IndexChoices` (or keep as default-0 seed). **~10-15 LOC net.**
4. **Tests** — re-point the index step of `TestBip85DeriveFlow_ScrubsBothMnemonics`; rewrite `TestBip85ParamBounds`'s index axis; add validator unit tests; add a high-index derive vector cross-checked against `biptool`; grow `FuzzDeriveBip85Child` with an upper-bound case. Index-0 goldens (`AbandonGoldens`, `UsesChildFP`, `CanonicalVector`, `IndexVaries`) **unchanged**.

**Total estimate:** ~80-120 LOC production + ~80-120 LOC tests. Net-new widget LOC if chosen instead of reuse: a minimal numeric keypad would be ~120-180 LOC (constructor + Update + Layout) — **NOT recommended**; reuse is far cheaper and battle-tested.

**SemVer / lockstep:** firmware-only; **no `me` crate version bump**, **no clap/`schema_mirror`**, **no `docs/manual` mirror**, **no enum/t5-M1-guard edit** (extends an existing program). Mainnet-only, experimental posture and `childSeedWarning` retained.

**MANDATORY next gate:** the resulting brainstorm SPEC + IMPLEMENTATION_PLAN must pass an **opus architect R0 review to 0C/0I** (persist verbatim to `design/agent-reports/`) BEFORE any code. This recon FEEDS that gate; it does not replace it.
