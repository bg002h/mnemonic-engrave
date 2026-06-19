<!--
Persisted verbatim. opus-architect MANDATORY whole-diff adversarial execution review of the T3
receive-address-verify implementation (worktree seedhammer-wt-t3-verify, branch feat/verify-address,
3 signed+DCO commits over d334861: 3ee8e0b address.Find, 032a0b1 verify result flow + DescriptorScreen
Show/Verify choice, 689f09e address input typed-keyboard + scan recognizer), BEFORE merge. Reviewer
agentId a6f46723b971ef34c. Verdict: GREEN 0C/0I, NO findings. Re-ran full suite; FUZZED address.Find
(4.70M execs) + scan recognizer/btcaddr.DecodeAddress (3.58M execs) = ~8.3M total, 0 panics; FALSE-
POSITIVE PROBE PASSED (60 foreign addresses → all found=false; own addr beyond gap-50 → not found;
positive control own recv[7] → matched). No false-positive possible (same btcd canonical encoder both
sides; IsForNet + base58 netID double-guard cross-network; derivation errors propagated, never ""-masked).
GUI hang (R0-C1) fixed + proven (one-shot Verifying ctx.Frame then Find then Back-able loop; read-only);
keyless guard before desc.Keys[0] (R0-I1); scan address-probe LAST so descriptors/seeds/md1/mk1 not
misrecognized (TestScan 8/8); engraveObjectFlow no addressText case; ChoiceScreen inside click branch →
TestAllocs 0-alloc; NewAddressKeyboard unmasked+case-preserving while passphraseFlow stays masked; no new
module deps; 3 commits signed+DCO+authored Brian Goss. Worktree left as-is; nothing merged/pushed.
Disposition: GREEN — merged no-ff to fork main, pushed bg002h. Text below verbatim (entities un-escaped:
&lt;→<, &gt;→>, &amp;→&).
-->

# Adversarial Whole-Diff Execution Review — T3 receive-address verify

## Verification Results (all run, not trusted)

| Check | Command | Result |
|---|---|---|
| Targeted suites | `go test -count=1 ./address/ ./gui/` | `ok address 0.090s`, `ok gui 7.146s` |
| Full suite | `go test -count=1 ./...` | all `ok` (no failures across 30+ pkgs) |
| Vet | `go vet ./address/ ./gui/` | clean (no output) |
| gofmt | `gofmt -l address/ gui/` | clean (empty) |
| Alloc invariant (§2.5) | `go test -run TestAllocs ./gui/` | PASS (1.09s) |
| Find-parity (live table) | `go test -run TestFind...` | TestFind 7/7 subtests + KeylessNoPanic + PropagatesDerivationError all PASS |
| Regression set | TestScan / TestScanRecognizesAddress / TestDescriptorConfirmAddressAffordance(+Unsupported) / TestRunVerifyResult / TestTypeAddressCasePreserved | all PASS |
| **Fuzz `address.Find`** | 35s, 24 workers | **4,703,516 execs, 0 panics, PASS** (seeds incl. keyless/zero descriptor, derivation-error desc, garbage/empty candidates) |
| **Fuzz scan recognizer + `btcaddr.DecodeAddress`** | 35s, 24 workers | **3,576,221 execs, 0 panics, PASS** |
| **False-positive probe** | custom test | **PASS** — 60 foreign addresses (30 recv+30 chg from a different xpub) → all `found=false`; own addr at index 100 beyond max-gap 50 → not found; positive control (own recv[7]) → matched (0,7) |
| Scope: deps | `go.mod`/`go.sum` diff | **UNCHANGED** — `btcaddr`/`chaincfg` already transitive via `address` pkg; no new module |
| Scope: secrets | grep diff | no `wipeBytes`/secret handling (correct — addresses public) |
| Worktree | `git status` | clean; all scratch fuzz files deleted |
| Provenance | `git cat-file` / log | 3 commits, all SSH-signed (`gpgsig` blocks present; verify can't run only because no allowed-signers file locally), authored Brian Goss, DCO `Signed-off-by` + Claude `Co-Authored-By` trailers present |

## Security-critical findings (false-positive / panic / hang)

- **False-positive: NOT possible.** `Find` compares `address.DecodeAddress(cand,net).String()` against `Receive`/`Change` output, and `addressAt` returns `addr.String()` from the *same* btcd `address/v2` package — identical canonical encoder, so a non-controlled address cannot canonicalize-collide. Empty-string masking is impossible: derivation errors are propagated (address.go:74/83), never compared as `""`.
- **Cross-network: double-guarded.** For bech32, `DecodeAddress` extracts the HRP from the address itself; `IsForNet(net)` (the load-bearing guard, address.go:68) rejects `tb1…` under mainnet (`AddressSegWit.IsForNet` compares `hrp==net.Bech32HRPSegwit`). For base58, `DecodeAddress` rejects wrong `netID` at decode time. Even absent `IsForNet`, the `.String()` comparison would still differ. Verified live by `TestFind/wrong_network`.
- **Panic: keyless guard is correct.** `len(desc.Keys)==0` returns `ErrUnsupported` (address.go:54) *before* any `desc.Keys[0]` access (R0-I1). `addrFindMaxGap=50` bounds the scan. 4.7M fuzz execs, zero panics.
- **GUI hang (R0-C1): fixed.** `runVerify`'s "Verifying…" is a single one-shot `ctx.Frame` (verify_address.go:175-178, no loop), then `Find` runs once, then a Back-able result loop. `TestRunVerifyResult` reaches all three result strings — not stuck on Verifying. No engrave/NFC-write/mutation anywhere in the verify path (read-only confirmed).
- **Scan ordering (§2.6): correct.** Address probe is the last branch before unknown-format fallthrough (after command/bip39/descriptor/codex32/md-mk), so descriptors/seeds/md1/mk1 are not misrecognized (proven by `TestScanRecognizesAddress`). `engraveObjectFlow` has no `addressText` case → top-level address scan is unknown-format (unchanged).
- **Alloc + affordance:** ChoiceScreen is inside the `addrBtn.Clicked && supported` branch (gui.go:2372-2380), not per-frame → TestAllocs 0-alloc holds. Affordance test drives Button2→Button3 through the ChoiceScreen to the address view (no T1 regression).
- **Typed keyboard:** `NewAddressKeyboard` sets `revealed=true` AFTER `NewPassphraseKeyboard`'s trailing `Clear()` (which resets it false), so the address readout is cleartext; case is preserved (NO ToUpper at keyboard.go:192/394; `TestTypeAddressCasePreserved` round-trips `bc1Q3`). The original `passphraseFlow` still uses `NewPassphraseKeyboard` → stays masked. Both behaviors correct.

## Findings
None. No CRITICAL, no IMPORTANT, no MINOR. The three R0-cited defects (C1 hang, I1 keyless panic, I2 swallowed derivation error) are all genuinely fixed in the committed code and exercised by real tests, not just named ones.

## Verdict
**GREEN — 0 Critical / 0 Important**

(Worktree left as-is; no merge/push/modify performed. Scratch fuzz files deleted; `git status` clean.)
