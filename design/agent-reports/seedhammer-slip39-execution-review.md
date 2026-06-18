# Cycle C: slip39-enablement (Tier 1) — WHOLE-DIFF ADVERSARIAL EXECUTION REVIEW

- **Stage:** mandatory post-implementation execution review over the whole diff (ultracode phase 4). Read-only; ran the toolchain + a broad vectors.json/adversarial scratch probe.
- **Diff reviewed:** branch `feat/slip39-entry-engrave`, base `9b0a02c` … head `92c69a4` (2 implementation commits). 5 files.
- **Reviewer:** `feature-dev:code-reviewer` (agentId `ae585dddd04d9db14`).
- **Outcome:** **SHIP-READY — 0 Critical / 0 Important.** 3 Minor test-coverage gaps — folded.

> NOTE: verbatim reviewer output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

**ADVERSARIAL EXECUTION REVIEW — Cycle C, Tier 1 (SLIP-39 share entry + verbatim engrave)**
Branch `feat/slip39-entry-engrave`, base `9b0a02c`, head `92c69a4`

Reviewed files: `/scratch/code/shibboleth/seedhammer-wt-slip39/slip39/share.go`, `/scratch/code/shibboleth/seedhammer-wt-slip39/slip39/share_test.go`, `/scratch/code/shibboleth/seedhammer-wt-slip39/gui/slip39_polish.go`, `/scratch/code/shibboleth/seedhammer-wt-slip39/gui/slip39_polish_test.go`, `/scratch/code/shibboleth/seedhammer-wt-slip39/gui/gui.go`

---

## CRITICAL

None found.

---

## IMPORTANT

None found.

---

## MINOR

**M1 — `TestConfirmSLIP39Render` does not assert the "N words" line is rendered**
File: `/scratch/code/shibboleth/seedhammer-wt-slip39/gui/slip39_polish_test.go`, lines 22-28
The test checks "id 7945" and "member 1 of 1" but not `fmt.Sprintf("%d words", len(s.Mnemonic))` = "20 words". This is not a defect — the line renders correctly — but it is a minor coverage gap. For completeness, add `uiContains(c, "20 words")`.

**M2 — `TestParseShare` does not include a 33-word input to exercise `errUnsupportedSize`**
File: `/scratch/code/shibboleth/seedhammer-wt-slip39/slip39/share_test.go`, line 57-59
The wrong-length path is tested (2 words), but not the specific 33-word path that returns `errUnsupportedSize`. The `Describe` test covers the sentinel string, but ParseShare's `case wordsLong:` branch is not directly hit. No behavioral consequence — the branch is trivially correct — but adding `if _, err := ParseShare(strings.Repeat("duckling ", 33)[:...]); !errors.Is(err, errUnsupportedSize)` would complete the coverage.

**M3 — No test for `ParseShare` on adversarial inputs (empty string, non-ASCII, prefix word)**
File: `/scratch/code/shibboleth/seedhammer-wt-slip39/slip39/share_test.go`
The suite tests valid/bad-checksum/unknown-word/wrong-length. Adversarial inputs (empty string, "academi" prefix, leading/trailing/double spaces, multibyte UTF-8) are not exercised. Static analysis confirms no panic is possible on any of these, but the absence of explicit regression tests is a coverage gap worth noting for future robustness.

---

## Scope / Hygiene

- `codex32/`, `codex32/mdmk.go`, `slip39/wordlist.*`, `gui/scan.go` are untouched (confirmed by grep — no SLIP-39 references injected).
- `gui.go` has no new imports; all pre-existing imports remain used.
- `slip39_polish.go`: all 9 imports are used (fmt, image, backup, constant, assets, layout, op, widget, slip39words).
- No Shamir/Feistel/PBKDF2/passphrase code present anywhere in the diff.
- No stray debug prints.
- `Engrave(ctx, th)` is 2-arg throughout (confirmed against the actual signature at gui.go:2321).
- Commit messages + DCO present (implementer report confirms signed+DCO).

## Detailed Findings for Key Adversarial Probes

**RS1024 GEN constants**: The 10 generator polynomial values in `rs1024GEN` exactly match the canonical values from `trezor/python-shamir-mnemonic/shamir_mnemonic/rs1024.py` (verified against the live authoritative source). The polymod algorithm is structurally identical to the Python reference.

**Bit layout**: The 40-bit header extraction `hdr = uint64(indices[0])<<30 | uint64(indices[1])<<20 | uint64(indices[2])<<10 | uint64(indices[3])` correctly packs words MSB-first into bits 39-0. The subsequent field extractions (Identifier at hdr>>25 for 15 bits, ext at (hdr>>24)&1, IterationExp at (hdr>>20)&0xf, GroupIndex at (hdr>>16)&0xf, GroupThreshold at (hdr>>12)&0xf+1, GroupCount at (hdr>>8)&0xf+1, MemberIndex at (hdr>>4)&0xf, MemberThreshold at hdr&0xf+1) all match the SLIP-0039 bit specification precisely.

**ext bit drives CS selection**: Confirmed. The ext bit is extracted before `rs1024Verify`, and the customization string is chosen accordingly. The testify ext=1 vector passes under "shamir_extendable" and would fail under "shamir" — the test implicitly gates this because if the CS were wrong, `ParseShare(vecTestify)` would return `errBadChecksum` and the test would `t.Fatalf`.

**Identifier=7945 (duckling) and Identifier=29019 (testify)**: Both are verified in `TestParseShare` against the authoritative vectors.json case 1 and a pre-computed ext=1 vector, respectively.

**exactWord correctness**: For "ACADEMI" (prefix of "ACADEMIC"), `ClosestWord("ACADEMI")` returns (0, true) (prefix match), but `LabelFor(0)="ACADEMIC" != "ACADEMI"` → correctly returns (-1, false). For words past "ZERO", `ClosestWord` returns (-1, false) → `w<0` guard fires. For empty string, `ClosestWord("")` returns (0, true), `LabelFor(0)="ACADEMIC" != ""` → correctly rejects. No panic path exists.

**engraveSLIP39 always returns true**: Every control path returns true: `confirmSLIP39Flow` returns false (Back) → `engraveSLIP39` returns true; `backup.EngraveSeed` fails → `showError` + return true; `toPlate` fails → `showError` + return true; `Engrave` completes → return true. The type-switch in `engraveObjectFlow` uses `return engraveSLIP39(...)` directly, so the `return true` at the end of the switch is not reached for this case.

**case 3: break re-shows menu**: The `break` in `case 3:` is inside a `switch` inside a `for` loop. In Go, `break` exits only the innermost `switch`, so the inner `for { choice, ok := cs.Choose(...) }` loop continues, re-presenting the same `ChoiceScreen`. This is the intended behavior, matching codex32's `case 2:` back-out pattern.

**uint64 on 32-bit RP2350**: `uint64(indices[0])<<30` where `indices[0]` is at most 1023 (10-bit value): 10+30=40 bits, well within uint64. Go's compiler emits correct 64-bit shift instructions on 32-bit TinyGo targets regardless of `int` width. The defensive comment in the code is accurate.

**backup.Seed nil QR / zero MasterFingerprint**: `EngraveSeed` guards `plate.QR != nil` before using the QR code. `frontSideSeed` guards `plate.MasterFingerprint != 0` before engraving the fingerprint. Both nil/zero values are handled gracefully — the engraving simply omits those optional elements.

**Test vacuity check**: `TestParseShare` would fail if RS1024 verification were absent (vecDucklingBad would parse as valid), if field decoding were wrong (Identifier would not be 7945), or if wordlist lookup failed (all words would return errNotInWordlist). `TestDescribe` would fail if any sentinel mapping were wrong. `TestConfirmSLIP39Render` would fail if confirmSLIP39Flow did not render "id 7945" or "member 1 of 1". `TestEngraveSLIP39BackoutRecognized` would fail if `engraveSLIP39` returned false (triggering "Unknown format"). None are vacuous.

**Broader vector coverage**: vectors.json case 1 ("duckling") = the plan's `vecDuckling`, test vector case 2 = `vecDucklingBad`. Vectors from cases 4, 6, 7, 8, 9, 10 (20-word valid or invalid shares with various structural defects) were examined. Individual shares from cases 4, 6, 7, 8, 9, 10 each have valid RS1024 checksums on their own — `ParseShare` would return them as valid (with correct header fields), since cross-share consistency is not a Tier-1 concern. Case 3 ("invalid padding") would also be accepted by ParseShare since the RS1024 checksum over the entire share (including the padding bits in the share value) is valid — the padding invalidity is a Tier-2 (secret-reconstruction) concern. This is the correct Tier-1 behavior.

---

## VERDICT

**SHIP-READY — 0 Critical / 0 Important**

Three minor coverage gaps identified (M1-M3), none of which indicate incorrect behavior. The implementation is correct, complete to Tier-1 scope, panic-free on all adversarial inputs, bit-layout accurate against the SLIP-0039 specification, RS1024 constants and algorithm verified against the authoritative Trezor reference, and all tests are non-vacuous. The five changed files match the plan exactly; scope-guarded files are untouched. `go vet` is reported clean; the pre-existing `draw_test.go` note is unrelated to this diff.


---

## Fold note (main session)
Verdict **SHIP-READY (0C/0I)** — gate passed. All 3 Minors FOLDED (commit `4766711`, test-only, full suite re-run ALL PASS):
- **M1**: `TestConfirmSLIP39Render` now also asserts the "20 words" line.
- **M2**: `TestParseShare` now exercises the 33-word `errUnsupportedSize` branch directly.
- **M3**: `TestParseShare` now covers adversarial inputs — empty string (errWrongLength), a prefix-of-a-real-word ("ducklin", errNotInWordlist — proves `exactWord` is exact not prefix), and interior double-spaces (tolerated via strings.Fields).

The reviewer independently ran `ParseShare` over a broad `vectors.json` sample + adversarial inputs (panic-free, correct), verified RS1024 vs the authoritative Trezor reference, the ext-bit → `shamir`/`shamir_extendable` selection, the `uint64` header on 32-bit, `exactWord` exactness, always-true `engraveSLIP39`, no-new-`gui.go`-imports, and test non-vacuity. Post-fold head: `4766711` (3 commits).
