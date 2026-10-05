# Recon F5: ms1 + Standard SeedQR plate (seedhammer fork @ be00ef8)

## Headline findings
- The ms1 plate today is `backup.EngraveSeedString`. It cuts the UPPERCASE ms1 string as 10-character groups, plus a QR of that same string at qr.M, refused if the QR is wider than 33 modules.
- Only two flows use it: the codex32 scan/typed flow and the sealed-unlock path.
- The **bundle ms1 cards** (single-sig, multisig, composer) do NOT use it. They go through `validateMdmkStrings` -> `backup.EngraveText`, which offers three variants: TEXT+QR (QR of the lowercase string at qr.L), TEXT ONLY, and QR ONLY.
- Standard SeedQR is numeric mode: **48 digits -> 25 modules (V2), 96 digits -> 29 modules (V3)**. Both are the same at qr.L and qr.M, so it fits under the 33 cap with 4 modules to spare.
- Recommendation: add a NEW layout. Do not replace `EngraveSeedString`, because `seal.MaxEngraveableCodex32Len` is pinned to it.

## 1. How backup/backup.go lays out the plates
- Types:
  - `Seed{Title, Mnemonic []string, ShortestWord, LongestWord, QR *qr.Code, MasterFingerprint, Font}` is at backup/backup.go:16-24. The caller builds the QR itself.
  - `SeedString{Title, Seed, MasterFingerprint, Font}` is at :26-31. It has no QR field: the QR is always made from `Seed`.
- **How the QR content is chosen** (`EngraveSeedString`, :158-173):
  - `seed := strings.ToUpper(plate.Seed)` then `qr.Encode(seed, seedQRLevel)`.
  - It refuses with `"seed too long to engrave QR"` when `qrc.Size > seedQRMaxSize`.
  - It then calls `engrave.ConstantQR`.
- Constants `seedQRLevel = qr.M` and `seedQRMaxSize = 33` are at :154-156. The doc comment at :138-153 says these two values alone decide `seal.MaxEngraveableCodex32Len`, which is 90.
- **Where the cap comes from:**
  - F-117/F-118 in mnemonic-engrave/design/FOLLOWUPS.md:2406-2435. The seed plate cuts modules at `qrScale = 3` (backup.go:194), so 33 modules is 33.3 mm on the 85 mm plate. Text plates use scale 2.
  - Raising the seed plate to 37 changes plate geometry and needs a hardware read.
  - `engrave.ConstantQR` itself now allows up to dim 53 (engrave/engrave.go:501-512), so 33 is a seed-plate policy limit, not an engraver limit.
- **Layout of the string plate** (`engraveSeedString`, :181-245):
  - The plate is 85x85 mm (:185-188). Font is `plateFontSize = 4.1` (:175). `groupLen = 10` (:179). Columns are `maxCol1 = 16` and `maxCol2 = 4` (:192-194).
  - `ngroups = ceil(len/10)` (:197). `col1Height = max(qrsz, pfs*endCol1)` (:199-200).
  - The master fingerprint is centered above the block, only when non-zero (:205-212).
  - Column 1 starts at the inner margin (`innerMargin = 10`, :74). The top of column 2 starts at x=44 mm.
  - The QR is centered at x=60 mm and centered vertically (:223-226). The bottom of column 2 sits below it.
  - **Title row** (:235-243): `strings.ToUpper(plate.Title)` at `plateSmallFontSize = 3`, centered just under the block (`(Y+col1Height)/2 + 4mm`). There is no length check here. `TitleString`/`MaxTitleLen = 18` (:71, :111) are available for that.
- **Word plate** (`EngraveSeed` :125-136 -> `frontSideSeed` :247-333):
  - The same frame: mfp row, word column 1, column 2 split around the QR.
  - The QR is at the same spot (`qrScale = 3`, centered x=60 mm, :308-314) and is drawn only when `qrc != nil`.
  - It has the same title row (:323-331). More than 24 words switches to a no-QR layout (:268-273, :293-298).
- **Upstream word plate** (gui/gui.go:851-873 `engraveSeed`): `qr.Encode(string(seedqr.QR(m)), qr.M)`, then `backup.Seed{...QR: qrc}`, then `EngraveSeed`.
- **Fit of the new plate:**
  - With StrokeWidth 0.3 mm (internal/sh2/params.go:25), a 29-module SeedQR at scale 3 is 26.1 mm, against 29.7 mm for today's 33-module ms1 QR.
  - A 75-character ms1 is 8 groups and a 50-character one is 5, so both stay entirely in column 1 (fewer than 16 rows).
  - The new plate is therefore strictly smaller than the shipped 24-word ms1 plate (inferred from the formulas above).

## 2. seedqr package and the Standard SeedQR format
- `QR(m)` (seedqr/seedqr.go:22-33): `fmt.Fprintf("%04d", w)` per word index, giving 4 zero-padded digits per word. It **panics** on an invalid mnemonic (:25-27).
- `CompactQR(m)` (:35-42) returns raw `m.Entropy()` bytes, which is CompactSeedQR (binary mode).
- `Parse` (:15-20) tries Standard first, then Compact.
- **Encoding mode:** the vendored kortschak-qr v0.3.2 `Encode` picks Num, then Alpha, then byte (qr.go:38-46). An all-digit string therefore gets numeric mode automatically, which is what SeedQR requires.
- **Measured** with a scratch module that imports the fork (repo not modified):

| SeedQR | qr.L | qr.M | qr.Q | qr.H |
|---|---|---|---|---|
| 12 words, 48 digits | 25 | 25 | 25 | 29 |
| 24 words, 96 digits | 29 | 29 | 33 | 37 |

- **Spec check:** I fetched the spec verbatim (raw.githubusercontent.com SeedSigner/seedsigner dev docs/seed_qr/README.md, saved at scratchpad/seedqr_spec.md). All of the following match the fork's encoder at qr.M, which is what upstream uses (gui.go:852):
  - "Each index must be exactly four digits, so shorter numbers must be zero-padded (`12` becomes `0012`)" (line 51).
  - "solely of numeric digits so that we can use the more efficient 'Numeric' format" (line 103).
  - "12-word mnemonic (48 digits) = 25x25 / 24-word mnemonic (96 digits) = 29x29" (lines 131-132).
  - The spec states sizes in terms of the "L" capacity chart (lines 109, 198, 320). It does not mandate an error-correction level.
  - The spec is English-only (line 25), consistent with entr being English-only.
- **Verdict: a 96-digit SeedQR is 29 modules at qr.M (or L), so it fits the existing 33 cap.** Even qr.Q (33) fits. Only qr.H (37) would not.
- For comparison, today's ms1 QR (uppercase, qr.M) measures 33 modules for a 24-word ms1 (75 characters) and 29 for a 12-word ms1 (50 characters). The SeedQR saves one version in both cases.

## 3. Callers that build an ms1 plate
**Calls to EngraveSeedString (the ms1 string + QR-of-string layout):**
- a. `engraveCodex32`, gui/codex32_polish.go:218-251. On `codex32Engrave` it builds `SeedString{Title: id, Seed: scan.String()}` (:248) and passes it to `backupSeedStringFlow`, gui/gui.go:2859-2873.
  - Inputs are any scanned or typed codex32: K-of-N shares, master-seed (non-entr) secrets, and recovered secrets. Most of these have **no BIP-39 words**.
  - **Keep the old layout as the default.** Offer the new one only when `codex32.DecodeMS1` gives prefix entr, or mnem with language 0, at 16 or 32 bytes (codex32/mspayload.go:35-51). Rebuild the words with `bip39.New(entropy)` (bip39/bip39.go:272), the same way gui/ms1_decode.go:22-33 does. This last part is (inferred).
- b. `unlockEngraveCodex32`, gui/unlock_session.go:189-246 (:206-210). This is the sealed record.
  - The seal admission limit `seal.MaxEngraveableCodex32Len = 90` is derived from `EngraveSeedString` (seal/record.go:94-120) and pinned by backup/engraveable_test.go:32-68 and :97.
  - **Keep it unchanged.** If F5 replaced this function, the pin's rationale would decouple, and non-entr records would lose any QR.

**ms1 bundle cards** (these go through validateMdmkStrings -> EngraveText, NOT SeedString):
- c. Single-sig full: gui/singlesig_derive.go:96-100 (`EncodeMS1(m.Entropy())`) -> `singleSigEngraveCards` gui/singlesig_engrave.go:20-28 (cardMS1).
- d. Multisig full: gui/multisig_derive.go:64-72 -> `multisigEngraveCardsMulti` gui/multisig_engrave.go:31-38.
- e. Composer: `composerSecretCards` gui/composer_flow.go:584-647 (`EncodeMS1` at :632, deduped by sha256(entropy)).
- How a bundle ms1 plate is cut today:
  - `bundleEngrave` (gui/bundle_flow.go:618-650) -> `validateMdmkStrings` (gui/gui.go:2667-2740).
  - That offers "TEXT + QR" (QR of the lowercase ms1 at qr.L, byte mode, measured 33 modules for 24 words), "TEXT ONLY" and "QR ONLY" (:2678-2690).
  - cardMS1 plates are never marked with a title or footer (`bundlePlateMark`, bundle_flow.go:575-580).
- These three flows hold the words `m` at derive time, so they are natural places for an "ms1 + SeedQR" variant.
  - The bundle plan packs strings per card as Text paragraphs (bundle_flow.go:467-555). A SeedString-style plate would need either a new card kind or plate type, or a new variant inside the per-plate picker.
  - The words must be carried alongside the ms1 string, or the entropy re-decoded from it. `bundleCard` today carries only `strings` (inferred).
- The composer explicitly deferred a words+SeedQR plate to **F-455** (gui/composer_engrave.go:5-11, :27-36; FOLLOWUPS.md:15827). F5 is the layout that unblocks it.
- **Refugium (F-702 F5/F3, mnemonic-engrave FOLLOWUPS.md:20359-20380):** ms1 plates must carry the SeedQR and a letter row (`SeedString.Title`).
- **Recommendation (inferred):**
  - Make it a NEW layout. For example, a `SeedQR *qr.Code` or `Mnemonic` field on a new `SeedStringSeedQR`/`EngraveSeedStringWithSeedQR`, or an optional QR on `SeedString` where nil keeps today's behavior.
  - Leave `EngraveSeedString` byte-identical (goldens codex32-0/1, the seal pin).
  - Under the Refugium build profile, select it for c/d/e and for entr ms1 in a, either as a new picker variant ("TEXT + SeedQR") or as the profile's only ms1 form.
- Adjacent non-callers:
  - The hashlock preimage ms1 (`EncodeMS1Preimage`, gui/composer_preimage_plate.go:287-295) is not BIP-39 and must never get a SeedQR.
  - `codex32.IsPreimage` guards are at codex32_polish.go:233 and unlock_session.go:198.
  - cmd/biptool/main.go:337-354 mirrors the EngraveSeedString QR recipe. Check whether it needs a sibling.

## 4. Golden tests and rendering
- **Goldens:**
  - They are in backup/testdata/*.bin, gzip-compressed spline encodings: `seed-0-words-24.bin`, `seed-1-words-12.bin`, `codex32-0.bin`, `codex32-1.bin`, slip39, text, passphrase, hashlock, freetext.
  - `compareGolden` (backup/backup_test.go:398-412) -> `golden.CompareBSpline(p, *update, t.ArtifactDir(), ...)` (internal/golden: writes the gzip when update is set, :25-35, and always dumps an `.svg` into the artifact dir, :20-24).
  - To update: `go test ./backup -run TestX -update` (flag at backup_test.go:29).
- **Existing tests:**
  - `TestSeed` (:197-216) uses `genSeed` (:362-395), which builds the SeedQR via `qr.Encode(string(seedqr.QR(m)), qr.M)`.
  - `TestCodex32` (:299-337) is the template for a new ms1+SeedQR golden. It derives `Title: id` and the mfp.
  - `TestEngraveSeedStringTooLong` / `Happy` are at :433-482.
  - The cap and level pin is in backup/engraveable_test.go:32-68 and 97.
  - Other gui golden tests are gui/transaction_golden_test.go and gui/freetext_sizeproof_golden_test.go.
- **Rendering:**
  - `cmd/plateview` renders a plate to SVG/PNG at sh2 params (cmd/plateview/main.go:1-40). The plates come from the `gui/preview.go` registry (`previewBuilders` :99-111, which includes `"seed"` -> `engraveSeed`, :203-212). Add an `"ms1seedqr"` entry to preview the new plate (inferred).
  - cmd/emu draws the plan as an overlay when the cut begins (cmd/emu/plate.go:1-17). Seeing the plate in emu means driving a flow to its engrave screen; walk scripts are cmd/emu/walk_*.js and shots_*.js.

## 5. How the ms1 string is derived (same-entropy assertion)
- `codex32.EncodeMS1(entropy)` (codex32/msencode.go:17-31):
  - Accepts 16/20/24/28/32 bytes. The payload is `[0x00 msPrefixEntr] ‖ entropy`, encoded as `NewSeed("ms", 0, "entr", 's', payload)`.
  - The id is the fixed literal `entr`, so the Title from `Split()` is "entr". It is English-only, with no language byte.
  - The round trip is documented: `DecodeMS1(New(EncodeMS1(e))) == e` (:16).
- Callers derive it as `m.Entropy()` -> `EncodeMS1` -> `wipeBytes` (singlesig_derive.go:96-98, multisig_derive.go:65-67, composer_flow.go:631-633).
- `seedqr.QR(m)` is built from the same `bip39.Mnemonic m`.
- **Assertion for the plan:** `seedqr.Parse(qrDigits) == m`, and `DecodeMS1(New(ms1)).entropy == m.Entropy()`. Equivalently, `bip39.New(DecodeMS1(...).entropy)` equals `m` word for word (bip39.New at bip39/bip39.go:272).
- **Secret-copy note:** `string(seedqr.QR(m))` and `qr.Code.Bitmap` are already listed as LIVE, non-wipeable copies in the F-88 inventory (gui/unlock_session.go:262). The new plate adds the same residue on the ms1 paths.
