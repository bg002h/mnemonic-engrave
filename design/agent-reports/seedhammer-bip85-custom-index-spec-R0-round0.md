# BIP-85 custom-index SPEC — R0 review (round 0) — VERBATIM agent report

**Agent:** `a2b0a5c919d751a43` (adversarial opus architect; re-derived the golden 2 independent ways on a 64-bit host + empirically reproduced the silent-truncation bug). **Fork HEAD:** `8459654`. **Spec commit:** `0e33239`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I). 3 non-blocking Minors → plan. Persisted per the gate discipline.

---

# BIP-85 custom-index SPEC — R0 review (round 0)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 8459654  **Spec commit:** 0e33239  **Verdict:** GREEN (0C/0I)

## High-index golden re-verification (MANDATE #1) — RAN
Re-derived independently in an isolated throwaway worktree (removed+pruned after), on a **64-bit host** (`strconv.IntSize == 64` — where the truncation concern is live). Two independent code paths:
**(a) Independent path-walk** (own test, `bip39`+`hdkeychain`+`bip85` only, NOT calling `gui.deriveBip85Child`):
```
HardenedKeyStart = 2147483648 (0x80000000)
index 0       => "prosper short ramp prepare exchange stove life snack client enough purpose fold"
index 1       => "sing slogan bar group gauge sphere rescue fossil loyal vital model desert"
index 2^31-1  => "jewel solution patient quarter elite grace quarter dinosaur taste parent dial clump" (valid=true, words=12)
--- PASS
```
**(b) biptool** (`cmd/biptool derive`, SEPARATE path through `bip32.ParsePath`/`ParsePathElement`), master XPRV from abandon:
```
Master XPRV = xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu
m/83696968h/39h/0h/12h/1h          => sing slogan bar group gauge sphere rescue fossil loyal vital model desert
m/83696968h/39h/0h/12h/2147483647h => jewel solution patient quarter elite grace quarter dinosaur taste parent dial clump
m/83696968h/39h/0h/12h/2147483648h => biptool: bip32: path element out of range: "2147483648"  (exit 1)
m/83696968h/39h/0h/12h/0h          => prosper short ramp prepare exchange stove life snack client enough purpose fold
```
Pinned high-index golden **byte-identical across both** → CORRECT. Master XPRV matches §3 verbatim. index 0/1 match the SHIPPED goldens (`TestDeriveBip85Child_AbandonGoldens[12]:31`, `TestDeriveBip85Child_IndexVaries:79`) → typed path doesn't change existing children. 2^31 rejected with the cited error. Baseline T7b tests PASS.

## Width-safe validator + overflow guard (MANDATE #2)
- Current `deriveBip85Child` (`gui/bip85.go:32`) takes `index int`, guards ONLY `index < 0` (`:36-38`), then `uint32(index)+h` (`:54`) — no upper bound today.
- **Empirically demonstrated the silent truncation on this 64-bit host** (throwaway test on the real `deriveBip85Child`): `index=2^31` → NO error, derived `"success fuel awesome..."`; `index=2^31+1` → NO error, `"blossom december breeze..."`. Worse than "a different hardened child": `uint32(index)+h` overflows uint32 → `2^31→element 0`, `2^31+1→element 1` — an **UNHARDENED** element, off-spec, no error. Exactly the silent-wrong-backup I-2 must prevent. Spec's truncation-site call (`uint32(index)+h`, guard must precede) is right.
- Spec mandates BOTH (a) `parseBip85Index` into a width-safe type (uint64/int64, never bare `int`) rejecting `>2147483647`, AND (b) an upper-bound guard INSIDE `deriveBip85Child` before the cast. Defense-in-depth sound.
- **Residual-path check:** only production caller is `bip85DeriveFlow` (`:220`); tests/future callers can pass a raw `int` — the in-`deriveBip85Child` guard closes this; no residual un-rejected `uint32()` site.
- **BIP-85 index max = 2^31-1**, cross-checked 3 ways: authoritative `bip-0085.mediawiki` (index hardened, no sub-range beyond BIP-32 max); `bip32.ParsePathElement` (`bip32.go:69-84`) overflow check; `hdkeychain.HardenedKeyStart=0x80000000`. 2^31 correctly rejected.

## Keyboard reuse + scope (MANDATE #3)
- `typeAddressFlow` (`gui/verify_address.go:44-71`): Back(Button1)→`("",false)`, OK(Button3)→`kbd.Fragment,true`. Confirmed.
- `NewAddressKeyboard(ctx)` (`gui/passphrase_keyboard.go:133-137`) = `NewPassphraseKeyboard` revealed=true; `Fragment` exported (`:48`). Confirmed.
- **Keyboard NOT digit-restricted:** `ppPageSymbols` has digits (`:21`); `commit` appends any rune (`:192`); cross-page handler commits any page rune (`:247-258`) → letters/symbols/space typeable → **digit-only is the validator's job** (spec §3 CRITICAL UX FACT). A digit-only `NewKeyboard(ctx,"1234567890")` exists but its `Update` couples to word-completion; reusing the address keyboard + validator is sound.
- Clone `bip85IndexEntryFlow` faithful (third copy alongside `passphraseFlow`), title + parse-on-OK substituted.
- **m\*-free + firmware-only:** `gui/bip85.go` imports only `errors`,`fmt`, btcd `hdkeychain`/`chaincfg`, `seedhammer.com/{bip39,bip85,engrave,gui/assets,gui/op}`; no `md`/`mk`/`codex32`. No new program → no enum/t5-M1-guard/lockstep edit. No `me` flag/schema/docs-mirror/SemVer.
- **Security spine unchanged:** typed-only master, per-leg scrub, child steel-only never NFC, mainnet-only; index public (no new secret). Output unchanged (words+SeedQR; child bare fp at `:95`).

## Ambiguity adjudication
1. **Leading zeros (`"007"`→`7`): ACCEPT.** Matches `strconv.ParseUint(s,10,…)`; bounded by `≤2^31-1` regardless; rejecting surprises users. Forbid leading `+`/`-`/whitespace/`0x` (covered by `[0-9]`-only). LOCKED.
2. **Empty `Fragment` on OK → re-prompt (not silent 0): CONFIRMED mandated** (§2/§5 test 1/risk 2). A silent 0 is a footgun. LOCKED.

## Critical / Important
None / None.
## Minor
- **M-1 (polish).** `FuzzDeriveBip85Child` (`gui/bip85_test.go:300-313`) success assertion `:307` (`!validBip85Words || index<0`) silently accepts a truncated child for `index≥2^31`. Tighten to fail on `index>2^31-1`; seed the corpus with `1<<31` AND `1<<31+1` (both wrap cases). Non-blocking.
- **M-2 (polish).** An optional early input-length cap must be ≥10 digits (`2147483647`=10) AND must NOT masquerade as the range guard (a 10-digit `"9999999999"` still exceeds 2^31-1 → the validator stays the authority). Non-blocking.
- **M-3 (nit).** Keep the existing lower-bound error message format / add a distinct upper-bound message so test diagnostics distinguish the two. Non-blocking.

## Verified-correct
High-index golden (2 paths), index 0/1 shipped-unchanged, master XPRV, 2^31 rejection (biptool + truncation demo), the silent-truncation bug reproduced (unhardened element), all cited file:lines @8459654, BIP-85 protocol facts vs the BIP, the only-production-caller fact. Worktree cleaned.

## Bottom line
**GREEN (0C/0I).** High-index golden independently confirmed byte-identical via two code paths; the silent-truncation bug is real and reproduced on the 64-bit host (worse than feared — unhardened element, no error); the spec's I-2 design (width-safe `parseBip85Index` + an upper-bound guard inside `deriveBip85Child` before `uint32(index)+h`) closes every path incl. direct callers. BIP-85 index range correct vs the BIP/biptool/HardenedKeyStart. Scope m\*-free, firmware-only, no lockstep. Both ambiguities adjudicated/defaulted. The 3 Minors are implementation-phase polish. Proceed to the implementation-plan phase (its own R0). Re-run the §3 probe at impl time (test-8 staleness guard) — held at HEAD 8459654.
