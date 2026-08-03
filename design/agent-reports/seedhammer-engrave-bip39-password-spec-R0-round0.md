# Fable architect — R0 round 0 — SPEC_seedhammer_engrave_bip39_password.md

- **Reviewer role:** adversarial fable architect, mandatory pre-implementation R0 gate (must reach 0C/0I).
- **Spec under review:** `design/SPEC_seedhammer_engrave_bip39_password.md` @ `7b0280f`
- **Brief:** one question — does this spec, implemented as written, risk engraving a WRONG or UNRECOVERABLE passphrase onto a permanent plate? D1–D8 declared settled and out of scope. Highest-value task: verify the spec's eight empirical claims against source.
- **Date:** 2026-08-03

Persisted verbatim before folding, per the project standard.

---

## VERDICT
NOT GREEN (2C/6I)

## CLAIM VERIFICATION
1. **CONFIRMED** — probed the compiled face: 52 supported = space `# ' ( ) * , - . /` 0-9 `:` `@` A-Z `[ ] { }` (26 upper, 0 lower, 10 digits, 15 symbols); space decodes with advance=600 and 0 spline knots (blank advance). Missing 43 matches §3.1 exactly. Bonus: all 32 control slots 0x00–0x1F are free (`vector.Face` index covers 0–126, `found = Advance > 0`).
2. **CONFIRMED** — `panic(fmt.Errorf("unsupported rune: ..."))` at `/scratch/code/shibboleth/seedhammer/engrave/engrave.go:1365` in `StringCmd.engrave`, reached from both `Engrave` and `Measure`.
3. **CONFIRMED / cite IMPRECISE** — `TitleString` is at `backup/backup.go:49-61` (not 41-53); it keeps any face-decodable rune and space decodes, so titles can contain spaces. Stronger than stated: `EngraveText` paragraphs also engrave literal spaces, so remapping 0x20 would alter those plates too.
4. **CONFIRMED** — `bip39/bip39.go:217-226` feeds raw bytes into `pbkdf2.Key(sentence, []byte("mnemonic"+password), 2048, 64, sha512.New)`; `unicode/norm`/`golang.org/x/text` appear nowhere in the tree. NFKD is identity on ASCII; divergent for non-ASCII.
5. **CONFIRMED** — `bip32/bip32.go:37-41` (`Fingerprint` = first 4 bytes of `Hash160(pubkey)`); passphrase enters the PBKDF2 *salt* upstream of the master key. Both walls hold; no derivation path exists.
6. **CONFIRMED** — glyph geometry scales via `addScale` (`engrave.go:1375-1377`, divides by `Metrics.Height`); advances at `engrave.go:1370`. Changing the em box rescales every glyph and advance. (Spec's `:1370` cite points at the advance line, not the geometry line — substance correct.)
7. **CONFIRMED** — guard at `gui/gui.go:164` exactly as quoted; wrap at `gui.go:1654`; `npage`/`npages = int(bip85Derive)+1` at `gui.go:1859,1893` auto-bump to 7. The three §6 update sites (`gui.go:1503-1521`, `1670-1681`, `1883-1884`) are the complete set of switches over `program`. `m.prog` is never persisted.
8. **CONFIRMED arithmetic, two omissions found** — plate 85×85 (`backup.go:103-106`), innerMargin 10 (`backup.go:47`) → 65 mm usable; strokeWidth 0.3 mm (`cmd/controller/platform_sh2.go:188`); measured with the actual `kortschak-qr` v0.3.2: 100-char byte-mode L → **37 modules** (M → 41), so 33.3 mm at scale 3. But see C2/I1/I2: the constant-time QR primitive caps at 33 modules, and the height budget omits the metadata lines.

## FINDINGS

### C1 — The visible-space mark has no on-plate legend; a space-bearing passphrase is unrecoverable without out-of-band knowledge
**Where:** §3.3, §4.3, O3/O4
**Defect:** The mark is deliberately shaped to "not resemble any real glyph" (O3), yet nothing on the plate says what it means. The footer (§4.3) warns only about fingerprints. Every other element of the plate is directly human-readable; the mark is a private convention documented nowhere the reader will be.
**Failure:** Owner engraves `correct horse battery staple`, dies. Heir holds a plate with three unfamiliar symbols. Type the symbol? Skip it? A hyphen? Each wrong guess silently opens a different wallet. Spaces are among the most common passphrase characters; QR (the only self-describing copy) is opt-in and default-off. This is the brief's "unrecoverable passphrase" outcome, on a permanent medium, with a trivial fix and spare plate area (§4.1 leaves ≥5 mm).
**Fix:** Require a legend line (e.g. `<mark> = SPACE`, engraved with the real mark glyph) whenever the passphrase contains a space — or unconditionally. Fold into §4.3 and the §7 layout test; O4 (footer wording) should absorb it.

### C2 — §3.4's "load-bearing gate" does not gate the engrave path the spec itself specifies
**Where:** §3.4 + §4 ("reusing the existing column machinery")
**Defect:** The existing column machinery for string plates (`stringColumn`, `backup/backup.go:268-276`) is coupled to `engrave.ConstantStringer`, whose alphabet is hardcoded `constantAlphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"` (`engrave/engrave.go:750`) and which **panics independently of the face** on any other rune (`engrave.go:1286`). After the font work, lowercase/symbols/the mark all *decode* — so §3.4's "unreachable after the font work" row is satisfied — and the device still panics, because ConstantStringer's check is not `face.Decode`. The spec's claim that `ValidatePassphrase` is the only thing standing between input and a panic is false for the layout it mandates. Implemented exactly as written, essentially every real passphrase (any lowercase letter) crashes the machine at engrave time, possibly mid-plate. The alternative reading — swap in plain `engrave.String` — silently discards the timing-side-channel resistance that every other secret on this machine gets (ConstantStringer exists precisely so engraving timing doesn't leak secret content), an unflagged security decision.
**Failure:** A false load-bearing safety guarantee in the spec (the "false GREEN" class), plus either a guaranteed crash or an undecided security regression depending on which way the implementer jumps.
**Fix:** Specify the engraving primitive explicitly. Either (a) extend `constantAlphabet` to all 95 printable ASCII + the mark — noting the fixed-width requirement (`engrave.go:1218` panics on variable advance; satisfied automatically by `cmd/vectorfont`'s uniform `adv`, main.go:425-427) — or (b) mandate `engrave.String` with a recorded rationale for accepting the timing side channel. Update §3.4's guarantee to name every charset check on the path, and point the §7 no-panic test at the real plate-layout entry point, not just `engrave.String`.

### I1 — The specified 37-module QR cannot be engraved by the constant-time QR primitive, and short passphrases mask the failure
**Where:** §4.2
**Defect:** `engrave.ConstantQR` rejects `Size > 33` (`engrave/engrave.go:406-413`; `bitmapForQRStatic` supports versions 1–4 only). Measured boundary: ≤78-byte passphrases → 33 modules (works); ≥79 bytes → 37 modules (hard error). The non-constant `engrave.QR` works but is content-dependent-timing and is used today only for public data. The spec details a 37-module layout without naming a primitive that can produce it.
**Failure:** All development testing with normal-length passphrases passes; the first user with a 79+ char passphrase and QR opted in gets a failure after the confirm screen — or, if `engrave.QR` is chosen, the secret's QR is engraved with content-dependent timing, silently dropping the property seeds get.
**Fix:** Spec must either mandate extending `bitmapForQRStatic` to versions 5–6 (dims 37/41) or explicitly pick `engrave.QR` with rationale; add a §7 test that engraves the 100-char worst case with QR through the real command path.

### I2 — Plate-height arithmetic omits the metadata block; worst case overflows the usable area
**Where:** §4.1/§4.2 vs §4.3
**Defect:** §4.1 budgets 60 mm (text) against 65 mm usable; §4.3 then adds up to 3 lines at 3 mm (~9 mm plus inter-block gaps; existing code uses 4 mm `metaMargin`). No-QR worst case: 60 + 9 ≈ 69 mm > 65 mm. QR case: 55.8 + 9 ≈ 64.8 mm — over budget with any gap. The spec never says where the metadata goes (existing plates put mfp/title in the 10 mm screw-hole margin bands — `backup.go:123-130, 153-161` — but the spec doesn't invoke that), and the §7 fit test says "for a 100-character passphrase, with and without QR" — without fingerprints and footer.
**Failure:** Implementer places metadata in the usable area; a max-length plate clips, collides with screw holes, or overlaps blocks.
**Fix:** Specify metadata placement (e.g. fingerprints in the top margin band, footer in the bottom band, centered to clear corner holes, per existing practice) and extend the §7 layout test to the full worst case: 100 chars + QR + both fingerprints + footer.

### I3 — The confirm screen cannot surface spaces — the exact character class §3.3 declares wallet-changing
**Where:** §5/§5.1
**Defect:** Step 5 shows the passphrase "revealed... for proof-reading", but on screen a space is as invisible as on metal, and a 100-char string wraps (`widget.Labelw` MaxWidth), making trailing/double/line-break-adjacent spaces undetectable. The spec's own argument — "one space and two spaces look identical... `'hunter2 '` is a different wallet" — applies verbatim to the last checkpoint before permanent engraving. The masked entry readout (`*` per rune) plus counter gives weak detection at entry; confirm gives none.
**Failure:** User fat-fingers a trailing space, proof-reads the revealed string, sees nothing wrong, confirms; the plate faithfully engraves a passphrase (with a dutiful visible mark) that opens a different wallet than intended.
**Fix:** Specify that the confirm review renders spaces visibly (mirror the mark convention on screen) and/or displays derived facts: "100 chars, 3 spaces, 1 trailing". Add to the §7 touch-flow test.

### I4 — The specified input widget cannot type 13 of the 95 promised characters
**Where:** §5 step 1 vs D3
**Defect:** `PassphraseKeyboard` pages (`gui/passphrase_keyboard.go:18-22`) cover 26+26+10 + 19 symbols + space = 82 chars. Missing: `% * < > [ \ ] ^ ` { | } ~`. D3 promises all 95; the spec names the existing keyboard without requiring its extension, and §3.2 counts glyphs to author but not keys to add.
**Failure:** Not a wrong plate, but an unmet settled guarantee: a user whose real passphrase contains `~` or `[` cannot back it up — the feature's charset promise is hollow at the input stage.
**Fix:** Require extending the symbols page(s) to all 32 symbols, and add a §7 alignment test: every rune `ValidatePassphrase` accepts is both typeable on the keyboard and decodable by the face.

### I5 — §7 has no QR byte-exactness test; a raw/translated string swap is the one silent path straight into a wallet
**Where:** §4.2 + §7
**Defect:** Two variants of the secret are in flight: the raw string (QR, confirm) and the mark-translated string (engraver). §7 tests mark counts on the text side but never asserts `decode(QR) == passphrase as entered`. Swapping the variants engraves invisible real spaces (silent on metal) or QR-encodes the control-codepoint mark (scanner yields wrong bytes; pasted into a wallet it silently opens a different one). This is the highest-leverage single test the plan omits.
**Fix:** Add a §7 test: encode → decode the QR for passphrases with leading/trailing/interior/repeated spaces and all 95 chars; assert byte-identity with the input, and assert the engraved text stream contains zero `0x20` glyph indices.

### I6 — No confusable-glyph requirement for the 44 new glyphs
**Where:** §3.2/D5/O3
**Defect:** The plate becomes case-sensitive free text with no redundancy (unlike seed words, which are wordlist-redundant, or md1/mk1 strings, which are bech32 — a charset designed to exclude ambiguity). New single-stroke lowercase at ~2–2.7 mm x-height introduces `l/1/I`, `0/O/o`, `'`/`` ` ``, `;/:`, `,/.` collisions. O3 constrains only the space mark's shape; nothing requires the alphabet to be self-disambiguating.
**Failure:** Recoverer reads engraved `l` as `1` — no error, different wallet.
**Fix:** Add a font-authoring requirement: every confusable pair must be visually distinct at engraved size (slashed/dotted zero, serifed I, flagged 1, based l — conventions at the author's discretion), with the pairs enumerated and checked in the O1 hardware inspection.

### M1 — "Wiped on flow exit" is unachievable for a string-accumulated passphrase
**Where:** §5.3
`PassphraseKeyboard.Fragment` grows by string concatenation (`passphrase_keyboard.go:192`); Go strings are immutable, so every keystroke leaves unwipeable heap copies — the existing `wipeBytes`/`k.Zero` discipline (`gui/derive.go`) applies to `[]byte`/keys only. Either require `[]byte` accumulation + wipe, or record the volatile-RAM/air-gap rationale and soften the claim.

### M2 — "37 modules" is the byte-mode worst case, not a constant
**Where:** §4.2
Measured: a 100-char alphanumeric-subset passphrase (uppercase+digits+space etc.) encodes at 33 modules; byte-mode ≤78 chars likewise. Layout and tests must treat QR size as variable ≤37 (anchor/centering rules), not assert exactly 37.

### M3 — The vectorfont generator cannot currently address a control codepoint
**Where:** §3.3
`mapChar` (`cmd/vectorfont/main.go:704-771`) maps single-char and named SVG ids only; authoring the mark at 0x00–0x1F requires a generator extension (e.g. a named id → 0x1F). Unstated prerequisite; the binary format itself supports it.

### N1 — Cite drift
`TitleString` is `backup/backup.go:49-61` (spec: 41-53); glyph-geometry scaling is `engrave.go:1375-1377` (spec cites 1370, the advance line); `Fingerprint` func is `bip32/bip32.go:38` (comment at 37). All substantively correct.

## WHAT I CHECKED AND FOUND CLEAN
- Validation logic is airtight as specified: Go `range` decodes invalid UTF-8 to U+FFFD (>0x7E, rejected); DEL/`\n`/`\t`/controls rejected; the mark codepoint cannot survive validation, so no collision; translating before validating fails closed (mark is <0x20, rejected).
- Control slots 0x00–0x1F: all 32 free in the compiled face (probed); index covers 0–126.
- `program` values are runtime-only (`m.prog`); `gui/saver` persists nothing program-related — O2 should close trivially.
- §6's list of switch sites is complete (three sites: gui.go:1503-1521, 1670-1681, 1883-1884); insertion before `bip85Derive` keeps the `gui.go:164` guard at `[1]` and auto-bumps npage/npages to 7.
- UI bitmap fonts (poppins/comfortaa) are generated with the full printable-ASCII default alphabet (`cmd/bitmapfont/main.go:32`) — the revealed readout and confirm screen can render all 95 on screen.
- `PassphraseKeyboard` exists, is case-preserving (no `ToUpper` on commit), masked with reveal toggle; `passphraseFlow` pattern exists (`gui/gui.go:507-536`).
- The vectorfont pipeline enforces a single uniform advance for every glyph (`main.go:425-427`) — monospace/"position implies index"/"no intra-row gaps" hold automatically for the new glyphs.
- Font metrics as claimed: Ascent=800/Height=900 (em 9, baseline 8), `A` spans y2→y8 (probed), advance 6 units; descender at y9 of row *n* cannot collide with caps (y2) of row *n*+1 at pitch=em; `engrave.String` LineHeight defaults to 1.
- D6's premise: `EngraveSeedString` uppercases at `backup.go:77` and `:114` — the separate-type argument is sound; §4.2's "exactly as entered" QR wording correctly avoids that trap.
- Footer/labels (`SEED FP:`, `EXPECTED COMB FP:`, `FINGERPRINTS TYPED, NOT VERIFIED`) and uppercased hex fingerprints are fully engraveable with today's font.
- QR arithmetic verified against the real library (kortschak-qr v0.3.2): 100-char byte L=37/M=41, alnum L=33, 78/79-char L boundary 33→37; 65 mm usable, 0.3 mm stroke, scale 3 → 33.3 mm all confirmed.
- Scrubbing helpers (`wipeBytes`, `k.Zero`) exist and are used for seeds/keys (`gui/derive.go`), so §5.3's cited discipline is real (see M1 for the string caveat).

The two Criticals are cheap to fix in the spec (a legend requirement; naming the real engraving primitive and correcting the §3.4 guarantee), but both sit exactly on the wrong-or-unrecoverable-plate axis, so the gate stays closed until they are folded.
