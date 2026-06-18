# Recon — SeedHammer SLIP-39 & CODEX32 on-device input flows

*Date: 2026-06-17 · Reviewer: ultracode recon workflow · Repo: `/scratch/code/shibboleth/seedhammer` (fork `main`, with #34 codex32-input + #35 md1/mk1 merged)*

> Scope note: this builds on the prior 4-agent input-UX recon (`design/RECON_seedhammer_input_ux.md`) which mapped `inputWordsFlow`/`inputCodex32Flow`/`inputSLIP39Flow` and first flagged that `codex32.Interpolate` is never called from the GUI. This document drills into the SLIP-39 and CODEX32 flows specifically, with spec verification, resource analysis, and fork-side effort/risk sizing. We are NOT pushing upstream (PR #36 closed); all paths below are fork-side only.

---

## 1. Executive summary

**SLIP-39.** The word-entry UI is ~90% complete and live in the binary, but it has **no entry point** (menu choice, call site, engrave branch, and NFC-scan branch are all commented out) and — critically — **no share-handling crypto exists at all**: the in-tree `slip39` package is wordlist-only, and the `slip39.Share`/`ParseShare`/`CombineMnemonics` symbols the commented code references have never existed in the repo. The headline gap is that enabling SLIP-39 is not a config flip; it requires sourcing or porting an entire RS1024 + GF(256) Shamir + Feistel/PBKDF2 crypto layer, and even then today's flow would engrave a single sub-threshold share verbatim — cryptographically useless.

**CODEX32.** Single-share entry is **already enabled on the fork** (PR #34 was effectively a one-token menu uncomment plus a test; the flow body is byte-identical to upstream). The crypto is fully present and correct (`codex32.New` validates on every keystroke; `Interpolate` does Lagrange over GF(32)). The headline gap is that **`codex32.Interpolate` is never reached from the GUI** — the flow collects exactly one share and engraves it verbatim, so a user trying to recover a wallet from k-of-n shares gets a plate of one share back, not their seed. Secondary gaps are UX polish (no error-class feedback, no field display, no char counter, no pre-engrave confirmation).

---

## 2. SLIP-39

### 2.1 Current state — reachable? commented? deps?

**Not reachable on either upstream/main or fork main.** The fork inherited this disabled state from upstream verbatim; the fork added nothing SLIP-39-related (its work was codex32 #34 + md1/mk1 #35).

**Live, compiled code (the UI):**
- `inputSLIP39Flow(ctx, th, mnemonic, selected) bool` — `gui/gui.go:684-767`. A complete word-entry loop, near-identical to `inputWordsFlow` (`gui.go:539`). Fills a `slip39words.Mnemonic` (`[]Word`) one word at a time using the shared `Keyboard` widget.
- Helpers, all live: `emptySLIP39Mnemonic` (`gui.go:503`), `completeSLIP39Word` (`gui.go:851`), `updateValidSLIP39Keys` (`gui.go:895`).
- The `slip39words` package = in-tree import `seedhammer.com/slip39` (alias `gui/gui.go:40`). Files: `slip39/slip39.go` (44 lines), `slip39/wordlist.go`, `slip39/wordlist.txt`.

**Commented out / missing (the wiring + ALL crypto):**
- Menu choice: `Choices: []string{"12 WORDS", "24 WORDS", "CODEX32" /* , "SLIP-39" */}` (`gui.go:1872`). **Verified.**
- Call site `case 3:` in `newInputFlow` — `gui.go:1892-1908`, commented. **Verified** (references `slip39.ParseShare`, `slip39words.LabelFor`).
- Engrave branch `case slip39.Share:` in `engraveObjectFlow` — `gui.go:1693-1708`, commented, prefaced `// TODO: re-enable SLIP39. See also nfcpoller.go.` **Verified** (references `scan.Words()`, `scan.Identifier`, `scan.MemberIndex`, `scan.MemberThreshold`).
- NFC-scan parse branch — `gui/scan.go:61-65`, commented: `// } else if m, err := slip39.ParseShare(sbuf)...`. **Verified.**
- **The entire SLIP-39 share crypto.** None of `slip39.ParseShare`, `slip39.Share`, `scan.Words()`, `scan.Identifier`, `scan.MemberIndex`, `scan.MemberThreshold` exist anywhere in the repo or its history. The in-tree `slip39` package is wordlist-only — it exports `Word`, `Mnemonic`, `NumWords`, `LabelFor`, `ClosestWord`, `ShortestWord(4)`, `LongestWord(8)` and nothing else. No `Share` type, no `ParseShare`, no RS1024, no Shamir/GF(256), no group/member threshold logic, no Feistel/PBKDF2.

The intended-but-never-integrated library is named by an in-tree comment (`gui/scan.go:61-63`, **verified**):
> `// TODO: re-enable SLIP39 support. Note that github.com/gavincarr/go-slip39 adds ~55kb of RAM use in the unicode package.`

This comment was introduced in commit `0aaf5e6 "all: implement v2 machine"` (Jan 2024) and is the documented origin of the maintainer's "excessive resource use by the go-slip39 module" objection.

### 2.2 Gap analysis — what's missing for usable on-device SLIP-39

The word-entry UI is ~90% done; the hard 90% (crypto + reconstruction) is 0% done.

1. **Share parsing + checksum (MISSING, must-have).** No `ParseShare`/RS1024 validation. The entry loop can collect 20 words but cannot tell the user whether the share's checksum is valid before engraving. (Contrast: codex32 calls `codex32.New` live at `gui.go:633`.)
2. **Multi-share reconstruction (MISSING, the core feature).** SLIP-39 exists for k-of-n recovery. The flow enters exactly one share (`emptySLIP39Mnemonic(20)`, hardcoded). There is no group/member collection loop, no `CombineMnemonics`. As with codex32 `Interpolate`, a single share would be **engraved verbatim, not reconstructed into a seed** — and a single SLIP-39 share below threshold is cryptographically useless. This is an active correctness gap, not polish.
3. **Field-confirmation screen (MISSING).** The commented engrave title `fmt.Sprintf("%d #%d 1/%d", scan.Identifier, scan.MemberIndex+1, scan.MemberThreshold)` (`gui.go:1701`) implies showing parsed fields, but there is no pre-engrave review of identifier/threshold/index.
4. **Hardcoded 128-bit only.** `emptySLIP39Mnemonic(20)` and the engrave guard `const maximumLength = 20 ... if len(w) > maximumLength { return false }` (`gui.go:1697-1699`, **verified**) silently reject 256-bit (33-word) shares — "No space for secrets > 128 bits." A 33-word share won't fit the plate.
5. **No tests.** Zero SLIP-39 GUI tests (`gui/codex32_input_test.go` is the only input test; no slip39 analog).
6. **Title bug.** The flow title is `"Input Words"` (`gui.go:756`), identical to BIP-39 — no SLIP-39 labeling.

### 2.3 Verified protocol facts (SLIP-0039) + impl/spec divergence

Verified against the authoritative spec: **[SLIP-0039](https://github.com/satoshilabs/slips/blob/master/slip-0039.md)**.

- **Wordlist:** exactly **1024 words**; min 4 / max 8 letters; unique 4-letter prefix; min Damerau-Levenshtein distance ≥ 2. → Firmware **MATCHES**: `slip39/wordlist.txt` has exactly 1024 lines (**verified: `wc -l` = 1024**); `ShortestWord=4`, `LongestWord=8`.
- **Share structure & bit-lengths (in order):** identifier 15 bits; extendable backup flag (ext) 1 bit; iteration exponent 4 bits; group index 4 bits; group threshold 4 bits; group count 4 bits; member index 4 bits; member threshold 4 bits; padded share value (padding + 8n bits); RS1024 checksum 30 bits (last 3 words).
- **Mnemonic length:** 128-bit secret → **20 words**; 256-bit secret → **33 words**.
- **Checksum:** RS1024, a Reed-Solomon code over GF(1024), 3 words / 30 bits; customization string `"shamir"` (ext=0) or `"shamir_extendable"` (ext=1).
- **Shamir field:** GF(256) / GF(2⁸), Rijndael polynomial x⁸+x⁴+x³+x+1 (AES).
- **Two-level scheme:** group threshold GT (1..G groups) and per-group member threshold Tᵢ (1..Nᵢ). **Reconstruction requires GT distinct groups, and within each of those groups Tᵢ member shares** — it is NOT simply "k of n flat shares."
- **Passphrase:** optional, printable ASCII 32–126; default is the empty string when none provided (`"TREZOR"` is the test-vector passphrase, not a default).
- **Encryption/digest:** master secret encrypted with a 4-round Feistel network using PBKDF2 as the round function; share index 254 carries a digest = first 4 bytes of HMAC-SHA256(R, S) for invalid-share detection (~2⁻³² false-accept).

**Divergences to flag:**
1. **Single-share, 128-bit-only entry** (`emptySLIP39Mnemonic(20)` + 20-word engrave cap). Cannot enter 256-bit/33-word shares; no multi-group reconstruction — a functional subset that does not satisfy SLIP-39's recovery model.
2. **Extendable backup flag (ext) is a mid-2024 SLIP-0039 amendment** (shamir_mnemonic added it May 2024; Trezor shipped June 2024). `go-slip39 v0.1.3` (Oct 2024) DOES model it (`Share.Extendable`), but the firmware's commented integration predates the amendment and never exercises it. Any revival must handle both customization strings (`"shamir"` / `"shamir_extendable"`).
3. **No checksum/Shamir validation reaches the user.** Today the firmware would engrave whatever 20 words were typed without RS1024 verification — unlike its codex32 path.

### 2.4 go-slip39 resource concern — substantiated

Source-verified by pulling `github.com/gavincarr/go-slip39@v0.1.3` from the Go module proxy (the GitHub repo is now 404, but the proxy retains v0.1.0–v0.1.3; latest Oct 2024). The maintainer's "excessive resource use" is **substantiated and specific**:

- **The "~55kb in the unicode package" is real and traceable.** `slip39.go` imports `regexp` (`reLabel = regexp.MustCompile(`^\d{3,6}$`)`). Go's `regexp` transitively pulls the `unicode` tables (~tens of KB of read-only data) — exactly the cited cost. For a single trivial `^\d{3,6}$` match this is gratuitous; replaceable by a hand-rolled digit check.
- **Heavy transitive dependency surface** for an embedded TinyGo target. Non-test code is ~2738 lines (`slip39.go` 1680 + `wordlist.go` 1058) and imports three third-party modules beyond stdlib: `github.com/deckarep/golang-set/v2`, `golang.org/x/exp/maps`, `gonum.org/v1/gonum/stat/combin`. It also uses `math/big`.
- **Dependency-tree note:** gonum IS already a direct dep (`gonum.org/v1/gonum v0.17.0`), so `stat/combin` is the cheapest of the three to absorb; `golang-set` and `x/exp` would be net-new to go.mod.
- **Public API matches the commented firmware code** (verified): `ParseShare(mnemonic string) (Share, error)`, `(Share).Words() ([]string, error)`, `CombineMnemonics`, `GenerateMnemonics`. `Share` exposes `Identifier`, `Extendable`, `IterationExponent`, `GroupThreshold`, `GroupCount`, `GroupIndex`, `MemberThreshold` (via embedded `ShareGroupParameters`/`ShareCommonParameters`), `MemberIndex`, `ShareValues []byte`. So `scan.Identifier`/`scan.MemberIndex`/`scan.MemberThreshold`/`scan.Words()` in the commented `gui.go:1695-1706` resolve against this exact module — confirming go-slip39 was the intended dependency.

**Bottom line:** the concern is **RAM/binary bloat on RP2350/TinyGo** (`regexp`→`unicode` plus three transitive modules), not algorithmic infeasibility. A fork-side path could vendor a trimmed subset (drop `regexp`, drop `golang-set`/`x/exp`, keep gonum/combin) — but that is real crypto work to port and audit, not a config flip.

### 2.5 Enablement / polish effort + risk

**Effort: LARGE — materially larger than the codex32 revival**, because SLIP-39 is missing its entire crypto layer (codex32 at least has `codex32.New`/`Interpolate` present; SLIP-39 has only a wordlist).

| # | Work | Size |
|---|------|------|
| a | **Source a share library:** integrate go-slip39 (absorb/trim its cost), OR write/port an in-tree impl (RS1024, GF(256) Shamir, Feistel+PBKDF2, digest verification, two-level combine). Dominant cost, security-critical. | **L** |
| b | **Multi-share collection UX:** loop `inputSLIP39Flow` across GT groups × Tᵢ members, track collected shares, surface thresholds. Net-new design. | **L** |
| c | **Field-confirmation + checksum-validation screens** (mirror codex32 must-fixes). | M |
| d | **Re-enable + wire:** uncomment `case 3:`, the menu choice, the engrave branch, the scan branch; fix the title; decide 128-bit-only vs 256-bit (plate space). | S |
| e | **Tests** from SLIP-0039 vectors: wordlist, parse, reconstruction. | M |

**Risks:**
- **Correctness/security:** a hand-rolled or partially-ported Shamir/GF(256)/Feistel is exactly the class of code where subtle bugs lose funds; demands the project's R0 architect gate + spec test vectors.
- **Resource budget:** the documented RAM/flash cost on RP2350/TinyGo; measure before committing.
- **Plate space:** 256-bit (33-word) shares don't fit the 20-word cap; enabling only 128-bit is a real functional limitation users must understand.
- **Maintainer alignment:** the maintainer cited both UX polish AND the go-slip39 resource cost; fork-side only.

### 2.6 Transferable Slice-1 (BIP-39 polish) patterns

Verified against branch `feat/bip39-entry-polish` (`gui/gui.go` diff vs main):

- **Button3-primary-accept: ALREADY DONE for SLIP-39.** The polish branch's `okBtn := &Clickable{Button: Button3}` was applied in all three flows; the hunk `@@ -685,7 +734,7 @@ func inputSLIP39Flow` confirms `inputSLIP39Flow`'s okBtn moved Button2→Button3. The consistency fix already covers SLIP-39 even while disabled.
- **Progress title "Word N of M": directly transferable.** Slice-1 added `layoutTitlef(... "Word %d of %d", selected+1, len(mnemonic))`. `inputSLIP39Flow` has the identical structure and currently shows the wrong static `"Input Words"` (`gui.go:756`) — a one-line swap.
- **Match count display: directly transferable.** `inputSLIP39Flow` already computes `nvalid` via `updateValidSLIP39Keys` (`gui.go:700`) and discards it — same as the pre-polish BIP-39 path.
- **Candidate-style key-restriction: PARTIALLY transferable, with a caveat.** The SLIP-39 wordlist is fixed/known and `updateValidSLIP39Keys` already does prefix-based key disabling (mirrors `updateValidBIP39Keys`), so progressive key-restriction works the same. BUT the Slice-1 *last-word checksum shortlist* (`bip39.LastWordCandidates`, `completeCandidateWord`, `updateValidCandidateKeys`) does NOT transfer: SLIP-39's checksum is **RS1024 over the last 3 words**, not a single BIP-39-style final-word checksum — there is no "candidate set for the final word." A SLIP-39 equivalent would need RS1024-aware logic over the last three words (and the share library that doesn't yet exist).
- **Shared keyboard widget:** SLIP-39 reuses the same `NewKeyboard`/`Keyboard` (`gui.go:790`) on `wordKeys` (`gui.go:685`), so any keyboard-level polish (touch-target sizing, key masking) benefits it automatically.

---

## 3. CODEX32

### 3.1 Current state — upstream vs fork / what #34 did / entry UX

**What PR #34 actually changed (commit `638dd14`, "gui: re-enable on-device CODEX32 seed entry"): essentially a one-token change plus a test.** The entire flow — `inputCodex32Flow`, the bech32 keypad, the call site, the `case codex32.String:` consumer — **all already exist in `upstream/main`, byte-identical**. Diffing `inputCodex32Flow` upstream vs fork: IDENTICAL. The only product change is uncommenting one menu entry:
- fork `gui/gui.go:1872`: `Choices: []string{"12 WORDS", "24 WORDS", "CODEX32" /* , "SLIP-39" */}` (**verified**)
- upstream `gui/gui.go:1806`: `Choices: []string{"12 WORDS", "24 WORDS" /* , "CODEX32", "SLIP-39" */}`

Plus new file `gui/codex32_input_test.go` (45 lines): drives the menu to choice index 2, types the BIP-93 vector-1 secret `ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw`, confirms with Button2, asserts `newInputFlow` returns `codex32.String`. (The test comment confirms it depends on the menu entry: without it, Down-selection caps at index 1.)

SLIP-39 remains commented out on both upstream and fork (§2.1).

**Current entry UX (`inputCodex32Flow`, `gui/gui.go:623-682`, head verified):**
- Keyboard alphabet (`gui.go:624`): `"1234567890\nqwertyup\nasdfghjk\nlzxcvnm"` — a 4-row QWERTY-derived layout with `b/i/o` dropped (matching bech32's exclusion of `b/i/o/1`, though `1` is present as separator and payload digit). NOT canonical bech32 order. The keyboard force-uppercases every typed rune (`Keyboard.rune()`, `gui.go:1036`: `unicode.ToUpper(r)`), so all entry is uppercase `MS1...`.
- On every keystroke it calls `codex32.New(kbd.Fragment)` (`gui.go:633`, **verified**) and sets `valid = (err == nil)`. The OK button (Button2) fires only when `valid` (`gui.go:639`). The whole string renders as a single wrapped label (`widget.Labelw`, `gui.go:651`) — no segmentation, no per-field parsing, no char counter.
- Title is the static string `"Input Codex32 Share"` (`gui.go:672`).
- On accept, `inputCodex32Flow` returns the validated `codex32.String` up through `newInputFlow` (`gui.go:1887-1889`).

**What happens to the accepted share (`case codex32.String:`, `gui/gui.go:1724-1731`, verified):**
```go
id, _, _ := scan.Split()
s := backup.SeedString{Title: id, Seed: scan.String(), Font: constant.Font}
backupSeedStringFlow(ctx, th, s)
```
The share string is engraved **verbatim** (TEXT + QR via `backup.EngraveSeedString`, `backup/backup.go:75`). `Split()` returns `(id, threshold, idx)` but only `id` is used (plate title, max 18 chars); threshold and share index are discarded. **This consumer is upstream's code, unchanged by #34** — the verbatim-engrave behavior predates the fork.

### 3.2 Gap analysis — the multi-share reconstruction gap (confirmed)

**Confirmed: `codex32.Interpolate` is never reachable from the GUI.** A repo-wide grep shows it is called only from `cmd/biptool/main.go:127` and `:334` (host-side CLI) and from `codex32/codex32_test.go`. **Zero calls in the `gui` package** (**verified: grep for `.Interpolate(` in non-test Go returns only biptool**). `String.Seed()` (byte extraction, `codex32.go:386`) is likewise never called from `gui` or `backup`.

**Trace — can a user enter k-of-n shares and recover the secret on-device today? No.**
- `inputCodex32Flow` handles exactly ONE share and returns a single `codex32.String`. No loop to collect k shares, no share-list state, no per-share id/threshold consistency checks, no `Interpolate`, no conversion of a recovered secret to a wallet seed.
- Whatever the user types — an unshared secret (`ms1` with threshold `0` / index `S`) or a single k-of-n share (e.g. `ms12namea...`) — is engraved literally. Entering one share of a 2-of-n scheme engraves that one share, NOT the reconstructed seed. As the prior recon flagged, this is actively wrong for a recovery workflow.
- The crypto is fully present and correct (`Interpolate` does Lagrange over GF(32) with id/threshold/length/hrp consistency checks, `codex32.go:188-276`; `biptool derive` proves the round-trip: `parseShares → Interpolate(shares,'S') → k.Seed() → hdkeychain.NewMaster`). The GUI simply never exercises it.

**Missing for on-device k-of-n recovery:**
1. A multi-share collection UI: enter share 1 → show parsed id/threshold/index → "add another share" until k reached (k is known from any share's threshold field via `Split`).
2. Cross-share validation surfacing (`Interpolate` already returns `errMismatchedID`/`errMismatchedHRP`/`errMismatchedThreshold`/`errMismatchedLength`/`errRepeatedIndex`/`errInsufficientShares`, `codex32.go:201-230` — none surfaced today).
3. Call `Interpolate(shares, 'S')` to recover the secret, then decide what to engrave: recovered seed bytes (`Seed()`) as a BIP-39/SeedQR plate, or the reconstructed `S` codex32 secret string. **A design decision, not just plumbing.**
4. A "single secret vs shares-to-recover" mode choice up front — entering an unshared secret (index `S`) and entering shares are different intents.

### 3.3 Verified protocol facts (BIP-93 / codex32) + impl/spec divergences

Source: **[BIP-93](https://raw.githubusercontent.com/bitcoin/bips/master/bip-0093.mediawiki)**; codex32 reference: [secretcodex32.com](https://secretcodex32.com) / [github.com/BlockstreamResearch/codex32](https://github.com/BlockstreamResearch/codex32). BIP-93 vectors 1-5 are mirrored in `codex32/codex32_test.go`.

**Verified against spec, matches firmware:**
- **bech32 charset** `qpzry9x8gf2tvdw0s3jn54khce6mua7l` (32 chars, 5-bit). Firmware `codex32.Alphabet` (`gf32.go:21`) is the same set, uppercased; `invCharsTbl` (`gf32.go:38`) decodes both cases. ✓
- **HRP + form:** HRP `ms` (or `MS`), separator `1` → `ms1...`. Firmware `splitHRP` cuts on the first `1` (`codex32.go:453`). ✓
- **Share structure** after `ms1`: threshold char (1 digit), 4-char identifier, 1-char share index, payload, BCH checksum. Firmware `partsInner` (`codex32.go:127-173`): `res[0]`=threshold, `res[1:5]`=id, `res[5]`=shareIdx, `res[6:len-checkLen]`=payload, trailing checksum. ✓
- **Threshold values** `0` or `2`–`9`; **`1` is explicitly invalid** per spec ("MUST be a single digit between '2' and '9', or the digit '0'"). Firmware `partsInner` switch (`codex32.go:134-155`) accepts `0,2-9`, returns `errInvalidThreshold` otherwise — including `1`. ✓ (`Split()` cosmetically reports a stored threshold of 0 back as `1`, `codex32.go:397-399` — display nicety for unshared secrets, not parse acceptance of `1`.)
- **Index `s`/`S` = unshared secret;** spec: if threshold is `0`, index MUST be `s`. Firmware enforces `threshold==0 && shareIdx!=feS → errInvalidShareIndex` (`codex32.go:169-171`). ✓
- **Checksum scheme** BCH over GF(32). Short/regular = 13 symbols (data part ≤93); long = 15 symbols (data part ≥96). Firmware `shortChecksumLen=13`, `longChecksumLen=15` (`codex32.go:45-46`, **verified**); engines `newShortChecksum`/`newLongChecksum` with distinct generator polynomials and target residues (`checksum.go:29-68`). Short target "secretshare32" (`checksum.go:41-46` = `feS feE feC feR feE feT feS feH feA feR feE fe3 fe2`) matches the spec residue. ✓
- **k-of-n recovery via Lagrange interpolation over GF(32)** (spec `ms32_recover`). Firmware `Interpolate` (`codex32.go:188`) implements Lagrange with the direct-output shortcut when the target index is itself an input share (`codex32.go:219-222`). ✓
- **Incomplete-group / padding rule** (spec: "any incomplete group at the end MUST be 4 bits or less"). Firmware `sanityCheck` rejects `(len(payload)*5)%8 > 4` (`codex32.go:54-56`). ✓
- **Mandatory single-case** (all-upper or all-lower, no mixed). Firmware `engine.setCase` (`checksum.go:132-153`) returns `errInvalidCase` on mixed case. ✓

**DIVERGENCE 1 — long-code length gate is narrower than spec.** Firmware gates on **total string length** (`codex32.go:40-47,98-107`, **verified**): short = 48–93, long = **125–127**, rejecting everything in the 94–124 gap. The spec defines validity on the **data part**: regular ≤93, long 96–108, with only data-part 94–95 illegal. With HRP `ms1` (3 chars), the spec's long code admits totals ~99–111+, but the firmware accepts only long totals 125–127. **Net effect: the firmware accepts only the BIP-93 long vectors (256-bit and 512-bit seeds, totals 125/127) and rejects shorter-but-spec-valid long-code strings.** In practice codex32 is used for 128/256/512-bit seeds, so the common cases (48 for 128-bit; 74 for 256-bit short; 127 for 512-bit long) all work. The test corpus only exercises 48-char and 125–127-char strings, so this gap is untested. **Flag as a known, low-severity restriction — it errs toward rejecting, not mis-accepting.**

**DIVERGENCE 2 (clarification, not a bug) — md1/mk1 are a DIFFERENT scheme.** Fork PR #35 added `codex32/mdmk.go` (absent upstream). md1/mk1 reuse codex32's BCH(93,80,8)/BCH(108,93,8) machinery but with a **different initial residue** `POLYMOD_INIT = 0x23181b3` (vs codex32's `1`, encoded as `feP` in `newShortChecksum.residue[12]`, `checksum.go:38-40`) and **NUMS-derived target residues** (md regular target `0x0815c07747a3392e7`, etc., `mdmk.go:55-62`). `ValidMD`/`ValidMK` are **pure verifiers (no error correction, no interpolation)** — md1/mk1 strings are engraved verbatim via `mdmkFlow` (`gui.go:1786`), exactly like a codex32 share. **These are NOT codex32 and must not be conflated with it; they share only the GF(32) engine struct. No interpolation/secret-sharing semantics apply.**

### 3.4 The "UI flow not sufficiently polished" assessment — concrete

The maintainer's objection is well-founded. Concrete deficiencies in `inputCodex32Flow`:

- **No error class surfaced.** `codex32.New` returns rich distinct errors — `errInvalidLength`, `errInvalidChecksum`, `errInvalidCase`, `errInvalidCharacter`, `errInvalidThreshold`, `errInvalidShareIndex`, `errIncompleteGroup` (`codex32.go:24-37`). The flow collapses all to `valid = err == nil` (`gui.go:634`); the only feedback is the OK button appearing/disappearing. A 1-char typo in a 127-char string gives no checksum-vs-length-vs-char distinction, no location.
- **No segmentation / no field display.** The string renders as one wrapped blob. No live parse showing "threshold k, id XXXX, share index N" as the user types past those positions (all derivable from `Split`/`partsInner`).
- **No char counter / progress.** Unlike BIP-39's "word N", no "char N of 48/93/127" hint, even though valid lengths are a tiny fixed set.
- **No pre-engrave confirmation of parsed fields.** Engraved straight from `backupSeedStringFlow` with no review screen showing id/threshold/index/fingerprint. (BIP-39 at least has `SeedScreen.Confirm`.)
- **Non-standard keyboard charset** (`gui.go:624`): custom QWERTY-minus-b/i/o rather than bech32 order or dimmed full-QWERTY. No per-position validity hinting.
- **Confirm on Button2 (middle)**, inconsistent with the Button3-primary convention (shared keyboard — §4).
- **The core feature (multi-share recovery) is simply absent (§3.2).** The heaviest "not polished" item: the flow looks like recovery but only single-share verbatim engraving works.

### 3.5 Enablement / polish effort + risk

**Already enabled on the fork (#34).** No further enablement needed to expose single-share entry; the gap is polish + the missing multi-share feature.

| Item | Size | Notes / risk |
|------|------|-------------|
| **Error-class surfacing** (map `codex32.New` errors to messages) | S | Errors are sentinel `errors.New` but **unexported** (`codex32.go:24-37`) — needs exporting or a small classifier helper. Low risk. |
| **Live field parse + char counter + segmented display** | M | `partsInner` is unexported and panics on malformed input via `parts()`; want an exported, error-returning partial-parse helper. Medium risk (touches codex32 API). |
| **Pre-engrave confirmation screen** (id / threshold / index / optionally fingerprint) | S–M | `Split` already exists. |
| **Multi-share k-of-n + `Interpolate`** | **L — and the only correctness fix** | New multi-share collection UI, cross-share validation surfacing, `Interpolate(shares,'S')`, and a design decision on what to engrave (recovered seed as BIP-39/SeedQR vs the `S` codex32 secret). Genuinely new flow design. Note `Interpolate` runs GF(32) math fine on-device (already used by `biptool`), so **no TinyGo/resource concern comparable to go-slip39.** |
| **Long-code length gate** (Divergence 1) | S | Optional; widen to BIP-93 data-part semantics, pair with vectors. Current behavior is safe (over-restrictive). |
| **Keyboard charset normalization** (bech32 order or dimmed QWERTY) | S | But it's the *shared* keyboard widget — §4. |

**General risk:** the keyboard is shared across BIP-39 / codex32 / (disabled) SLIP-39. Charset and confirm-button changes ripple. Per the project MEMORY standard, any implementation goes brainstorm → spec → plan → architect R0 gate (0C/0I) before code.

### 3.6 Transferable Slice-1 patterns

Slice 1 (`feat/bip39-entry-polish`): progress title, match count, Button3-primary-accept (keyboard commits on Center only), last-word candidate-restricted keyboard.

- **Button3-primary-accept: TRANSFERS directly.** `inputCodex32Flow` uses Button2 for OK (`gui.go:628,639`); moving to Button3 with Center-to-commit matches the new convention. A shared-keyboard win. Low effort.
- **Progress title / char counter: TRANSFERS as a *char* counter** ("N of 48/93/127", or "id XXXX · share N") rather than a word counter — the static title (`gui.go:672`) can go live. Data from `Split`/`partsInner`.
- **Match count: does NOT transfer literally** (no wordlist). The analog is **per-position validity / error-class feedback** (checksum vs length vs char) + live field parsing — different mechanism, same intent (surface already-computed state).
- **Candidate-restricted keyboard: does NOT directly transfer** — codex32 is free-form bech32, no fixed wordlist, no candidate set. A *per-position* restriction is partially possible: position 0 (threshold) is `0` or `2-9`; positions 1-4 (id) and the index are any bech32 char; the trailing 13/15 are checksum. So only the threshold position has a meaningful restriction; the bulk can't be candidate-constrained. The transferable idea is dimming non-bech32 keys generally, not position-by-position narrowing.
- **Fingerprint-on-confirm: TRANSFERS conceptually only AFTER multi-share recovery exists** (no seed = no fingerprint for a single share); for a single share, a parsed-fields confirmation (id/threshold/index) is the equivalent anchor.

---

## 4. Cross-cutting

- **Shared keyboard widget.** BIP-39, codex32, and (disabled) SLIP-39 all use the same `NewKeyboard`/`Keyboard` widget. `Keyboard.rune()` force-uppercases (`gui.go:1036`). Any keyboard-level change — touch-target sizing, key masking/dimming, the Button3-primary + Center-commit convention — ripples across all three. This is a feature for consistency and a hazard for regressions: changes must be regression-tested against BIP-39 (the only live, tested consumer).
- **What Slice 1 already gives both flows.** The Button3-primary-accept refactor on `feat/bip39-entry-polish` was applied to **all three** flows, including the disabled `inputSLIP39Flow` and `inputCodex32Flow`. So the button-convention consistency fix is already in place for both even though their flows are disabled (SLIP-39) or unpolished (codex32). The progress-title and match-count helpers (`layoutTitlef`, the `nvalid` surfacing pattern) are reusable scaffolding both flows can adopt — SLIP-39 as a word counter, codex32 as a char counter.
- **The deferred full-ASCII passphrase keyboard.** The deferred passphrase slices contemplate a full printable-ASCII keyboard (for SLIP-39/BIP-39 passphrases, printable ASCII 32–126 per SLIP-0039). **Does it help codex32? No, not directly.** codex32 entry is restricted to the bech32 charset (32 symbols, uppercased), which is *narrower* than ASCII, not wider — a full-ASCII keyboard would expose invalid keys and worsen entry, the opposite of what codex32 needs (which is bech32-restricted, ideally per-position-hinted keys). The full-ASCII keyboard is a passphrase concern; codex32 wants the existing restricted bech32 keypad, polished (charset order + dimming + error feedback), not broadened.

---

## 5. Recommended path (fork-side; no upstream PRs)

Given the fork is the maintained line and we are not pushing upstream, ordered by value-per-effort:

**Tier 1 — worth doing, in order (codex32 polish, low risk, no new crypto):**
1. **codex32 Button3-primary-accept + Center-commit** (S). Pure consistency with Slice 1's convention; the cheapest win; already de-risked by the shared-keyboard refactor. Do first.
2. **codex32 error-class surfacing** (S). Export the `codex32.New` sentinel errors (or add a classifier) and show "checksum invalid" / "wrong length" / "bad character" instead of a silently-disabled OK button. Highest UX value per effort; directly answers the maintainer's "not polished" objection.
3. **codex32 char counter + live field parse + pre-engrave confirmation** (M). Needs an exported, non-panicking partial-parse helper in the codex32 package. Turns blind 127-char entry into a guided, reviewable flow. This is the package of work that would make single-share codex32 genuinely "polished."

These three together address the maintainer's concrete polish objection without touching crypto or resource budget. They are a coherent fork-side slice.

**Tier 2 — worth doing only as a deliberate, R0-gated slice of its own (the real feature):**
4. **codex32 multi-share k-of-n recovery via `Interpolate`** (L). This is the *only correctness fix* and the actual reason the flow exists. The crypto is present and runs on-device (no resource concern). But it is genuinely new flow design (collection loop, cross-share validation surfacing, mode choice, and a design decision on what to engrave). Should be its own brainstorm→spec→plan→R0 cycle, separate from Tier 1. Worth doing — it is the difference between "engrave a share" and "recover a wallet."

**NOT worth doing (or: do not attempt without an explicit, separately-justified decision):**
- **SLIP-39 enablement (full).** This is the largest, highest-risk item and the lowest value-per-effort. It requires sourcing/porting an entire security-critical crypto layer (RS1024 + GF(256) Shamir + Feistel/PBKDF2 + digest + two-level combine), absorbing or trimming go-slip39's documented RAM/flash cost on TinyGo, designing a net-new multi-group collection UX, AND it still can't fit 256-bit shares on the plate. The maintainer's two objections (UX polish + go-slip39 resource cost) are both substantiated. **Recommendation: do not pursue SLIP-39 enablement** unless there is concrete user demand that justifies a multi-week, audit-grade crypto effort. If ever revisited, the only defensible route is a trimmed in-tree port (drop `regexp`/`golang-set`/`x/exp`, keep gonum/combin) behind the R0 gate with the full SLIP-0039 test-vector suite — not a go-slip39 drop-in.
- **codex32 long-code length-gate widening (Divergence 1).** Current behavior is *safe* (over-restrictive, never mis-accepting) and the common 128/256/512-bit cases all work. Only worth touching if paired with the field-parse work and spec test vectors; otherwise leave it.
- **A full-ASCII keyboard "for codex32."** Wrong direction (§4); codex32 wants the restricted bech32 keypad polished, not broadened.

---

## 6. Open questions / verify on hardware

1. **Plate-space reality for codex32 long codes.** A 127-char codex32 string is engraved verbatim as TEXT + QR via `backup.EngraveSeedString`. Confirm on hardware that a 127-char string actually fits the plate and the QR scans — the codex32 path has no plate-fit guard analogous to SLIP-39's 20-word cap.
2. **codex32 multi-share: what to engrave?** Design decision needing a product call: after `Interpolate(shares,'S')`, engrave the recovered seed bytes as a BIP-39/SeedQR plate, or the reconstructed `S` codex32 secret string? Each implies a different downstream restore workflow.
3. **codex32 single-share-vs-recover mode.** Confirm the intended UX: does the user pick "engrave this share as-is" vs "recover from k shares" up front, or is it inferred from the threshold/index of the first share entered?
4. **RP2350/TinyGo resource headroom** if SLIP-39 were ever revived: measure actual RAM/flash delta of go-slip39 (or a trimmed port) against the current free budget before any commitment — the "~55kb unicode" figure is the regexp-table cost, not the total module cost.
5. **Keyboard regression surface.** Any change to the shared keyboard (charset order, dimming, button convention) must be regression-tested against the live BIP-39 flow (the only consumer with tests) before touching codex32 — verify on hardware that touch targets and uppercasing behave unchanged.
6. **codex32 error-surfacing API.** Confirm whether the maintainer prefers exporting the `codex32.New` sentinel errors vs adding a classifier helper — this affects the codex32 package's public API and should be agreed before Tier-1 item 2.

---

### Source citations

- **SLIP-0039 spec:** https://github.com/satoshilabs/slips/blob/master/slip-0039.md
- **go-slip39:** https://pkg.go.dev/github.com/gavincarr/go-slip39 (v0.1.3, Oct 2024; GitHub repo now 404, module proxy retains v0.1.0–v0.1.3)
- **Trezor SLIP-39 docs:** https://docs.trezor.io/trezor-firmware/core/misc/slip0039.html
- **shamir-mnemonic (extendable-flag history):** https://pypi.org/project/shamir-mnemonic/
- **BIP-93:** https://raw.githubusercontent.com/bitcoin/bips/master/bip-0093.mediawiki
- **codex32 reference:** https://secretcodex32.com · https://github.com/BlockstreamResearch/codex32

### Key file:line anchors

- SLIP-39 flow `gui/gui.go:684-767`; helpers `gui.go:503,851,895`; commented call site `gui.go:1872,1892-1908`; commented engrave `gui.go:1693-1708`; commented scan `gui/scan.go:61-65`; wordlist-only package `slip39/slip39.go` + `slip39/wordlist.go`/`.txt`; go-slip39 origin comment introduced in commit `0aaf5e6`.
- codex32 flow `gui/gui.go:623-682` (`:624` keypad, `:1036` `Keyboard.rune()` uppercasing, `:672` title, `:1724-1731` verbatim-engrave consumer, `:1867-1912` `newInputFlow`/menu `:1872`, `:1833-1849` `backupSeedStringFlow`); crypto `codex32/codex32.go` (`New:98`, `partsInner:127`, `Interpolate:188`, `NewSeed:279`, `Seed:386`, `Split:394`, length consts `:40-47`), `codex32/checksum.go` (`newShortChecksum:29`, `newLongChecksum:50`, `setCase:132`), `codex32/gf32.go` (`Alphabet`, GF(32) tables), `codex32/mdmk.go` (fork-only PR #35). `Interpolate` callers: `cmd/biptool/main.go:127,:334` + tests only. #34 diff: commit `638dd14` + `gui/codex32_input_test.go`.
- Slice-1 reference: branch `feat/bip39-entry-polish`, `gui/gui.go` + `bip39/bip39.go` (`LastWordCandidates`).
