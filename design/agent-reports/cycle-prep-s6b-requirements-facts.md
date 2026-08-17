# Fact-verification: `design/REQUIREMENTS_s6b_pre_flash_cycle.md` §2 (MEASURED FACTS)

**Read-only recon. No fixes proposed.** Verifies every citation and number in
§2.1–§2.6 and the §1-DECIDED block against the fork's actual working tree.

- Doc repo: `/scratch/code/shibboleth/mnemonic-engrave`, `master` = `5fd0b74` (confirmed clean, HEAD matches).
- Source repo: `/scratch/code/shibboleth/seedhammer`, `main` = `b1479a1b38f6b045d27443764c858906e4e6e122` (confirmed clean, HEAD matches).
- Doc's own stated source SHA: `main` = `b8a23bf` (`b8a23bf3dcf45f0b996bedf8b17f7141f092d282`, "merge s5-multislot").

---

## §2.1 — keys/descriptor bind the passphrase, ms1 does not

**Citation `deriveAccountXpub` at `gui/singlesig_derive.go:10` — STRUCTURALLY-WRONG.**
Line 10 of `gui/singlesig_derive.go` is an import (`"seedhammer.com/bip32"`), not
a function. `deriveAccountXpub` is not defined in `singlesig_derive.go` at all —
it is called from there (line 46: `xpub, masterFP, err = deriveAccountXpub(m,
passphrase, net, path)`) but **defined in a different file**:

```
gui/derive.go:19:func deriveAccountXpub(m bip39.Mnemonic, passphrase string, net *chaincfg.Params, path bip32.Path) (xpub string, masterFP uint32, err error) {
```

**This is not drift from the S6a merge — it was already wrong at the doc's own
cited SHA.** Checked directly:
```
$ git show b8a23bf:gui/singlesig_derive.go | sed -n '10p'
	"seedhammer.com/bip32"
$ git show b8a23bf:gui/derive.go | grep -n '^func deriveAccountXpub'
19:func deriveAccountXpub(...)
```
The function has lived at `gui/derive.go:19` since it was added (`git log
--follow`: commit `3164b93 gui: add deriveAccountXpub — scrub-complete
account-xpub derivation (T4)`), predating the doc's own read-out SHA.
Correct citation: **`gui/derive.go:19`**.

**The CODE-PATH claim itself is ACCURATE**, independent of the bad citation.
Traced `gui/derive.go:19-52`:
```go
func deriveAccountXpub(m bip39.Mnemonic, passphrase string, net *chaincfg.Params, path bip32.Path) (...) {
	seed := bip39.MnemonicSeed(m, passphrase)   // line 20 — passphrase feeds the seed
	...
	masterFP = bip32.Fingerprint(pk)            // line 31
	...
	xpub = acct.String()                        // line 50
```
`masterFP`/`xpub` (which feed both `md1` via `md.EncodeSingleSig` and `mk1` via
`mk.Encode`, in `gui/singlesig_derive.go` steps 3 and 5) are derived from a seed
that includes `passphrase`. So **mk1/md1 genuinely are passphrase-bound**.

`codex32.EncodeMS1(entropy)` — **ACCURATE**, exact line:
```
gui/singlesig_derive.go:87:	ms1, err := codex32.EncodeMS1(entropy)
```
`entropy := m.Entropy()` (line 86) comes from the mnemonic only. Confirmed the
function signature takes no passphrase:
```
codex32/msencode.go:17:func EncodeMS1(entropy []byte) (string, error) {
```
So **`ms1` genuinely does not depend on the passphrase** — code-path confirmed,
independent of the byte-value table (not reproduced, per instructions).

---

## §2.3 — four plate-text mechanisms

| citation | verdict | detail |
| --- | --- | --- |
| `Fitted.Title`/`.Footer` @ `backup/fit.go:117-121` | **ACCURATE** | Comment "The screw-hole rows..." starts line 117; `Title, Footer string` at 120; `TitleFace, FooterFace *vector.Face` at 121. |
| `MaxTitleLen = 18` / `TitleString` @ `backup/backup.go:98` | **ACCURATE** | `const MaxTitleLen = 18` is actually at line 58 (not cited at that number — the doc's `:98` citation is specifically for `TitleString`, which is correct: `func TitleString(face *vector.Face, s string) string {` is exactly at line 98). |
| `Seed.Title`/`SeedString.Title` @ `backup/backup.go:17,27` | **ACCURATE** | `type Seed struct` at 16, `Title string` at 17; `type SeedString struct` at 26, `Title string` at 27. |
| `strings.ToUpper` renders @ `:223`, `:311` | **ACCURATE** | Both lines are `title := strings.ToUpper(plate.Title)`, inside the title-engraving blocks of `EngraveSeed`/`EngraveSeedString`-shaped functions. |
| passphrase plate `topLines`/`bottomLines`, footer 32 chars, no 18-cap @ `backup/passphrase.go` | **ACCURATE** | Fields at lines 133-134; footer const at line 156: `const passphraseFooter = "FINGERPRINTS TYPED, NOT VERIFIED"`. Measured: `len("FINGERPRINTS TYPED, NOT VERIFIED") == 32` (python3, confirmed). This path never calls `TitleString`/`MaxTitleLen` — confirmed by reading the whole file's rendering path (bands, not title/footer fields). |
| `Text.Paragraphs` has no title/no footer, used by mk1/md1 via `validateMdmk` in `gui/gui.go` | **ACCURATE** | `type Text struct { Paragraphs []Paragraph; Font *vector.Face; FontSize float32 }` (`backup/backup.go:33-41`) — genuinely no title/footer fields. `validateMdmk` (`gui/gui.go:2288`) builds only `backup.Text{Paragraphs: ..., Font: sh.Font}` for all three engraving variants (TEXT+QR/TEXT ONLY/QR ONLY) of an md1/mk1 string; confirmed both md1 and mk1 route through it via `mdmkFlow` (`gui/gui.go:2342-2344`) and the `mdmkText` type used consistently for both card kinds across the repo (`bundle_fuzz_test.go`, `bundle_test.go`, etc.). No sibling function bypasses this for mk1/md1 — call sites of `validateMdmk` (`gui/unlock_platelist.go:222`, `gui/derive_xpub.go:494`, `gui/gui.go:2344`, `gui/bundle_flow.go:407`) are all `mdmkText`-shaped. |

---

## §2.4 — 18-char cap and silent truncation

`TitleString` body read in full (`backup/backup.go:98-110`, confirmed the
function's own braces span exactly that range):
```go
func TitleString(face *vector.Face, s string) string {
	s = strings.ToUpper(s)
	res := ""
	for _, r := range s {
		if _, _, valid := face.Decode(r); valid {
			res += string(r)
		}
		if len(res) == MaxTitleLen {
			break
		}
	}
	return res
}
```
**ACCURATE, and genuinely silent**: stops at exactly `MaxTitleLen` (18, line
105's comparison); when `face.Decode(r)` is invalid the rune is skipped with no
error return (function signature returns only `string`), no log call, no panic
on that branch. Confirmed by reading the whole function — there is no other
exit path.

**Character counts — measured with a tool, not by eye:**
```
$ python3 -c "..."
 19  'PASSPHRASE REQUIRED'
 17  'PASSWORD REQUIRED'
 18  'SEED FP: 73C5 DA0A'
 18  'COMB FP: FC60 C6DF'
 17  'PASSPHRASE NEEDED'
 16  'NEEDS PASSPHRASE'
 27  'EXPECTED COMB FP: FC60 C6DF'
```
**All ACCURATE** — every count in the doc's list matches exactly, including the
DECIDED string `PASSWORD REQUIRED` = 17 ("fits with 1 to spare" against the
18-cap is correct: 18 − 17 = 1).

`passphrase.GroupFingerprint` — **ACCURATE**:
```
passphrase/passphrase.go:90:func GroupFingerprint(canonical string) string {
```
```go
func GroupFingerprint(canonical string) string {
	if len(canonical) != FingerprintLen {
		return canonical
	}
	return canonical[:4] + " " + canonical[4:]
}
```
Splits at 4, matches the doc's `"73C5DA0A" → "73C5 DA0A"` example structurally.

---

## §2.5 — passphrase plate: R5/R6/R7

- `engravePassphraseFlow` @ `gui/passphrase_flow.go:605` — **ACCURATE**:
  `func engravePassphraseFlow(ctx *Context, th *Colors) {` is exactly line 605.
  It is a distinct top-level program (called from `gui/gui.go:1865`), separate
  from `engravePassphraseFlowFrom` (line 617) which it wraps.
- `qr.Encode(plate.Passphrase, qr.L)` @ `backup/passphrase.go:86` — **ACCURATE**,
  and genuinely passphrase-only. Read the whole `passphraseQRCode` function and
  its doc comment (lines 78-87): only `plate.Passphrase` is passed to
  `qr.Encode`; the comment explicitly states "encodes the passphrase EXACTLY as
  entered: the same bytes, the same case... never SpaceMark." No metadata field
  is concatenated in. The `Passphrase` struct itself (lines 23-33: `Passphrase`,
  `SeedFP`, `CombinedFP`, `QR bool`, `Font`) carries no key-id/wallet-policy-id
  field for the QR path to leak in the first place.
- `SeedFP`/`CombinedFP` as `topLines` @ `:176-180` — **ACCURATE**, exact block:
  ```go
  176: if plate.SeedFP != "" {
  177:     l.topLines = append(l.topLines, "SEED FP: "+passphrase.GroupFingerprint(plate.SeedFP))
  178: }
  179: if plate.CombinedFP != "" {
  180:     l.topLines = append(l.topLines, "EXPECTED COMB FP: "+passphrase.GroupFingerprint(plate.CombinedFP))
  181: }
  ```
- Footer @ `:156` — **ACCURATE** (see §2.3 above): `const passphraseFooter =
  "FINGERPRINTS TYPED, NOT VERIFIED"` at exactly line 156.
- "No key-id and no wallet-policy id" — **ACCURATE**. The `Passphrase` struct
  (`backup/passphrase.go:23-33`) has exactly `Passphrase`, `SeedFP`,
  `CombinedFP`, `QR`, `Font` — no identifier field exists to carry either.

---

## §2.6 — identifier widths

`md`/`mk` packages **do exist in the fork repo** at `md/` and `mk/` (top level,
not a citation to another repo — flagging this only because the recon brief
asked me to check).

- `md.WalletPolicyId` @ `md/walletpolicyid.go:30` — **ACCURATE**:
  ```
  md/walletpolicyid.go:30:func WalletPolicyId(d *descriptor) ([16]byte, error) {
  ```
  Return type `[16]byte` confirms 16 bytes → 32 hex.
- `md.WalletPolicyIDStub` @ `md/walletpolicyid.go:106` — **ACCURATE**:
  ```
  md/walletpolicyid.go:106:func WalletPolicyIDStub(d *descriptor) ([4]byte, error) {
  ```
  Return type `[4]byte` confirms 4 bytes → 8 hex.
- `mk.Header.ChunkSetID` @ `mk/mk.go:50` — **ACCURATE**:
  ```
  mk/mk.go:48:type Header struct {
  mk/mk.go:50:	ChunkSetID  uint32
  ```
  Field is Go-typed `uint32`, but the wire encoding is genuinely 20 bits — traced
  both directions: `mk/encode.go:237` computes `csid := top20(bytecode)` and
  packs it as four 5-bit groups (`mk/encode.go:266-269`, each `& 0x1f`); the
  decoder (`mk/mk.go:81`) reassembles the same four 5-bit groups shifted by
  15/10/5/0. 4×5 = 20 bits. **ACCURATE.**

---

## §1-DECIDED — "BIP-39 Password" already shown to the operator

- `gui/gui.go:1997` — **ACCURATE**:
  ```
  1997:		titleTxt = "BIP-39 Password"
  ```
  (inside `StartScreen.draw`, `case engravePassphrase:`)
- `gui/passphrase_flow.go:645` — **ACCURATE**:
  ```
  645:	if !syswSourceAccept(ctx, th, "BIP-39 Password", sysw.ClassPassphrase, src) {
  ```

Both exact-line matches, exact string matches.

---

## Summary table

| tag | count |
| --- | --- |
| ACCURATE | 19 |
| DRIFTED-by-N | 0 |
| STRUCTURALLY-WRONG | 1 |

(Counted as one row per distinct citation/number verified above: §2.1 has 2
citations — 1 wrong, 1 accurate — plus the 2 code-path claims (ms1, mk1/md1)
which are accurate; §2.3 has 6 citation rows, all accurate; §2.4 has 7 counts +
1 function citation + 1 GroupFingerprint citation, all accurate; §2.5 has 5
citations, all accurate; §2.6 has 3 citations, all accurate; §1-DECIDED has 2
citations, all accurate.)

## The one finding that matters

**STRUCTURALLY-WRONG: §2.1's citation `deriveAccountXpub` (`gui/singlesig_derive.go:10`) points at the wrong file and the wrong line.** The function is defined at `gui/derive.go:19`. This citation was wrong even at the doc's own claimed read-out SHA (`b8a23bf`) — it is not new drift from the S6a merge, it is a pre-existing citation error.

**This is NOT load-bearing for any design decision.** The underlying fact the
citation supports — that `mk1`/`md1` are passphrase-bound via
`deriveAccountXpub` while `ms1` is not — is independently verified TRUE by
tracing the actual code (see §2.1 above). A spec consuming §2 would inherit a
correct *fact* with a broken *pointer*. Recommend the spec fix the citation to
`gui/derive.go:19` when it re-cites this fact, but no fact in §2 needs
re-deriving, and no decision in §1/§3 rests on the citation being right rather
than the fact being right.

Every other citation and every measured number in §2.1–§2.6 and the
§1-DECIDED block reproduces exactly against the fork at `main` = `b1479a1`.
Nothing has drifted since the S6a merge (21 files, +3209/−73) despite the
doc's own SHA being 8+ months of history behind.
