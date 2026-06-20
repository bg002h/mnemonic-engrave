# BIP-85 custom-index — whole-diff exec review (round 0) — VERBATIM agent report

**Agent:** `a039ad6527dee4166` (adversarial opus architect; re-ran the overflow guard + validator + negative-control fuzz on a 64-bit host). **Branch:** `feat/bip85-custom-index`. **Base:** `8459654`. **Final commit:** `5ee82da`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I). No new Minors. Mandatory post-implementation gate per CLAUDE.md phase (4). Cleared for merge.

---

# BIP-85 custom-index — whole-diff exec review (round 0)
**Reviewer:** opus architect (adversarial)  **Branch:** feat/bip85-custom-index  **Base:** 8459654  **Verdict:** GREEN (0C/0I)

Host: go1.26.4 linux/amd64, strconv.IntSize=64 (the unsafe 64-bit target). Diff: 5 commits 2065091..5ee82da; exactly 2 files (gui/bip85.go, gui/bip85_test.go). All probes RAN against the COMMITTED code (HEAD 5ee82da); throwaway probes added then removed — worktree clean.

## Overflow guard re-run (MANDATE #1) — RAN
Placement CONFIRMED: `gui/bip85.go:64` `if index < 0` (distinct msg `"bip85: invalid index: -1"`) then `:67` `if index > bip85MaxIndex` (`"bip85: index %d exceeds the maximum %d"`), BOTH strictly BEFORE the only `uint32(index)+h` cast at `:88`. `bip85MaxIndex = hdkeychain.HardenedKeyStart-1` (`:27`) = 2147483647. Ran committed `deriveBip85Child(abandon,"",12,X)` on the 64-bit host:
- X=1<<31 / 1<<31+1 / 1<<40 / 2147483648 → ALL ERROR; NONE derive a child. No silent uint32 truncation.
- X=-1 → ERROR with the DISTINCT lower-bound message.
- X=2147483647 → `jewel solution patient quarter elite grace quarter dinosaur taste parent dial clump` (GOLDEN MATCH).
- X=0 → `prosper short ramp…fold`; X=1 → `sing slogan bar…desert` (byte-unchanged).

## Validator (MANDATE #2) — RAN
`parseBip85Index` uses `strconv.ParseUint(s,10,64)` (`:40`) — not bare int. Matrix on committed code: ACCEPT "0"/"7"/"007"/"2147483647" (all ≤2^31-1); REJECT ""/"12a"/"-1"/"+1"/" 1"/"0x10"/"2147483648"/"9999999999"/"9223372036854775808" (+ max-uint64, 2^64). "9999999999" rejected by RANGE (not length — proven by accepting same-length "2000000000"). R0-M2 satisfied; validator is the range authority. No input returns >2^31-1; no panic; `int(v)` (`:47`) provably safe.

## Re-prompt deviation + flow (MANDATE #3)
R0-m1 deviation CORRECT. On parse error (`:186-192`) the impl calls `showError` then `kbd = NewAddressKeyboard(ctx)` (revealed=true) instead of `kbd.Clear()` (revealed=false). Probed: fresh `NewAddressKeyboard`→revealed=true; re-prompt stays CLEARTEXT (index public); no masked-secret regression. On error the flow LOOPS (no silent 0, no abort). Back(Button1)→(0,false), OK(Button3)→`parseBip85Index(kbd.Fragment)`. `bip85IndexEntryFlow` byte-faithful clone of `typeAddressFlow` (`verify_address.go:44-71`); picker (`bip85ParamPickFlow:160`) calls it; the 0..9 ChoiceScreen retired; `bip85IndexChoices` GONE (grep rc=1). `TestBip85IndexEntryFlow` (valid_high_index/back_exits/nonnumeric_reprompts_then_valid) PASS.

## No-regression + security + scope
- `go test -count=1 ./gui/... ./bip85/...` all green. Named tests PASS: TestParseBip85Index, _RejectsHighIndex, _HighIndexGolden, _AbandonGoldens, _CanonicalVector, _IndexVaries, _RejectsNegativeIndex, TestBip85ParamBounds, TestEngraveBip85Child_UsesChildFP, TestChildSeedWarningAbort, TestAllocs.
- Fuzz: seed corpus (incl. `1<<31`,`1<<31+1`,`2147483647`) PASS; 8s = 900K execs, 0 crashes. **NEGATIVE CONTROL:** temporarily removing the upper guard → FuzzDeriveBip85Child seed#6/#7 FAIL (`accepted out-of-spec index=2147483648/2147483649`) and TestDeriveBip85Child_RejectsHighIndex FAILs `got nil`. Guard restored; green. The success path asserts `index > bip85MaxIndex` (`bip85_test.go:319`) — does NOT silently accept a truncated child.
- Security spine: TestBip85DeriveFlow_ScrubsBothMnemonics PASS driving the NEW typed-index step (runes("0")+Button3) and asserting both master+child []Word zeroed (I-4). Typed-only master; child engraved steel-only never NFC; mainnet-only; output (words+SeedQR, child bare fp) unchanged. Index adds no secret.
- Two-layer defense: single production caller `deriveBip85Child:291 ← bip85DeriveFlow ← bip85ParamPickFlow ← bip85IndexEntryFlow ← parseBip85Index`; even a non-picker caller hits the in-function guard; the only `uint32(index)` site (`:88`) is strictly guarded.
- Scope: diff = EXACTLY gui/bip85.go + gui/bip85_test.go. m*-free (grep codex32/md1/mk1/ms1/schema_mirror → rc=1). New imports only `strconv` + `gui/layout` (already used by the clone source). No new program/enum/lockstep/CLI/schema/docs.
- `go vet ./gui/` rc=0; `go build ./...` rc=0.

## Critical / Important
None / None.
## Minor
None new. (R0-m1 folded as the cleartext-re-prompt fix + verified; R0-m2 was a verbose note, resolved.)

## Verified-correct
Guard placement before the cast + distinct messages; high-index golden byte-match; index 0/1 unchanged; ParseUint width-safety incl. >2^63/2^64; range-not-length rejection; cleartext re-prompt; typeAddressFlow clone fidelity; bip85IndexChoices fully retired (0 refs); single guarded uint32 site; two-layer guard; fuzz assertion proven load-bearing via negative control; two-secret scrub intact through the typed step; scope 2 files m*-free no-lockstep; vet/build clean.

## Bottom line
**GREEN (0C/0I).** Ran every mandate on the committed code on the 64-bit (unsafe) host. The silent uint32-truncation bug is closed: all X≥2^31 ERROR rather than deriving an unhardened element; the high-index golden (2^31-1) is byte-identical and index 0/1 unchanged. The validator is width-safe (ParseUint(…,64), range authority, ≤2^31-1 return). The R0-m1 deviation correctly keeps the public index cleartext (no silent-0/abort), and the entry flow is a faithful typeAddressFlow clone wired into the picker (0..9 ChoiceScreen + bip85IndexChoices retired). The fuzz/guard tests are proven to FAIL on a truncated child (negative control). Security spine, output, scope intact and m*-free; vet+build clean. Cleared for merge.
