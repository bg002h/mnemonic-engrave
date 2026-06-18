<!--
Persisted verbatim. opus-architect MANDATORY whole-diff execution review of Seed XOR combine.
Reviewer agentId ac7dbb4fca87b21dd. Diff feat/seedxor-combine bc63caa..3572dab (4 commits).
Verdict: GREEN — 0C/0I, cleared to merge. Re-ran tests/vet/gofmt/build; authenticated the
vectors vs TWO Coldcard upstream sources (docs/seed-xor.md + testing/test_seed_xor.py) +
independently recomputed; verified Combine vs the oracle + load-bearing {16,24,32} guard, the I1
per-part panic guard, the mandatory Button2-drained gate, the byte-identical title refactor (all
10 callers), no aliasing/secret-log, scope. Zero findings. Do not edit.
-->

# EXECUTION REVIEW — Seed XOR combine whole diff

**Reviewer:** opus architect (mandatory non-deferrable adversarial whole-diff execution review)
**Commit range:** `bc63caa..HEAD` (4 commits: `df271dc`, `f451000`, `4830732`, `3572dab`), branch `feat/seedxor-combine`
**Base:** fork `main` `bc63caa`
**Date:** 2026-06-18
**Worktree:** `/scratch/code/shibboleth/seedhammer-wt-seedxor`

## Reproduced verification (tails)

```
$ go test -count=1 ./seedxor/ ./gui/ ./bip39/
ok  seedhammer.com/seedxor 0.002s
ok  seedhammer.com/gui     6.906s
ok  seedhammer.com/bip39   0.036s

$ go vet ./seedxor/ ./gui/        → EXIT:0 (clean)
$ gofmt -l seedxor/ gui/          → (no files listed; clean)
$ go build ./...                  → (clean)
$ go test -count=1 ./...          → (no failures; all packages ok)
$ go test ./codex32/ ./slip39/ ./backup/ ./bip39/
ok  codex32 / slip39 / backup / bip39   (all green — reused machinery intact)

$ go test -run 'SeedXOR|Combine|WordFlow' -v ./seedxor/ ./gui/
  TestCombineVectors{24word,12word} PASS, TestCombineOrderIndependent PASS,
  TestCombineNoCallerMutation PASS, TestCombineTooFewParts/MismatchedLengths/BadLength PASS,
  TestWordFlowProgressTitle/MatchCount/LastWord{24,12}/NoFlash PASS,
  TestCombineSeedXOR / FingerprintMandatory / BackoutRecognized / PartCountBackout /
  PartLengthBackout / Confirm…Button2NoHang / NewInputFlowSeedXOREntry / …NamesNoCheck PASS

$ git diff --name-only bc63caa..HEAD → exactly 8 files:
  gui/gui.go, gui/gui_test.go, gui/seedxor_polish.go, gui/seedxor_polish_test.go,
  seedxor/seedxor.go, seedxor/seedxor_test.go, seedxor/testdata/{SOURCE.md,vectors.json}
```

## Per-focus findings

**1. `Combine` correctness vs the oracle — PASS.** Pure byte-XOR fold (`out[i] ^= e[i]`, `seedxor.go:50`) matches the toolkit oracle `seed_xor_combine` (`seed_xor.rs:172-177`, `out[i] ^= share[i]`). `len(parts)<2 → errTooFewParts` mirrors `validate_share_count`/`MIN_SHARES=2`. The `{16,24,32}` interop guard (`interopLen`, `seedxor.go:23,39`) is genuinely load-bearing: independently confirmed `bip39.New` (`bip39.go:228-234`) accepts any 16–32-byte multiple-of-4, i.e. 20/28-byte (15/21-word) too — so `Combine` is the only thing rejecting them; `TestCombineBadLength` constructs a real 15-word/20-byte valid mnemonic and asserts `errBadLength`. Equal-length enforced (`errMismatchedLengths`, `seedxor.go:45`). `bip39.New(out)` recomputes a fresh result checksum. `append([]byte(nil), parts[0].Entropy()...)` (`seedxor.go:38`) copies — `TestCombineNoCallerMutation` proves `parts[0].Entropy()` is unchanged after combine; additionally `Entropy()→splitMnemonic` returns a freshly-built `entBytes` slice (`bip39.go:193-196`), no aliasing. `wipe` zeroes `out` on every return path including error paths. No byte-order/fold error. Note: the toolkit additionally accepts 20/28-byte (toolkit-only extension); the fork intentionally narrows to Coldcard-interop `{16,24,32}` per SPEC §2.3 — a deliberate, documented divergence, not a defect.

**2. Vectors REAL & non-tautological — PASS.** I authenticated the committed `vectors.json` against TWO Coldcard upstream sources fetched live: `docs/seed-xor.md` (the cited source — its worked examples show the 24-word parts and result `…primary [555] / indoor [398]` and the 12-word `cannon [10B]…trade [735]`) AND `testing/test_seed_xor.py` (the 24-word part phrases + result `…primary indoor` appear verbatim, character-identical). I independently recomputed both vectors via raw `bip39.Entropy()` XOR (NOT through `seedxor.Combine`) → 24-word entropy `c87c…aaab` → `…primary indoor`; 12-word `2173…cbf3` → `…real trade`. `TestCombineVectors` asserts `Combine(parts).String()==parseM(v.Result).String()` AND a bytewise entropy pin against the *independently-declared* JSON `result`, not a same-path recompute. `TestCombineOrderIndependent` is real: reverses and rotates (`[1,2,0,…]`) the 3-part vectors → asserts identical result.

**3. I1 per-part panic guard (safety crux) — PASS.** `combineSeedXORFlow` runs `if !isMnemonicComplete(m) || !m.Valid()` (`seedxor_polish.go:56`) before `append`-collecting EACH part. `inputWordsFlow` Back does a bare `return` (`gui.go:631-633`) leaving a partial slice with `-1` slots; `isMnemonicComplete` (`gui.go:2201`, `slices.Contains(m,-1)`) rejects it → flow aborts before any `Entropy()`. The "full" return path (`gui.go:645-646`) only fires after the last word, which is constrained to checksum-valid `LastWordCandidates` — and the explicit `m.Valid()` re-check is belt-and-suspenders against any invalid-checksum full entry. No path reaches `Entropy()` (which panics on `!Valid()`, `bip39.go:159-161`) with a bad mnemonic. `TestSeedXORBackoutRecognized` genuinely drives partial-then-Back (enters "silent"+Button3 = 1 of 24 words, then Button1) → `(nil,false)`, no panic.

**4. Mandatory fingerprint gate — PASS.** `confirmSeedXORFingerprint` is on the ONLY success path (`seedxor_polish.go:71`); `false`→`(nil,false)`, so a recovered seed cannot reach `backupWalletFlow` without passing it. The loop (`for !ctx.Done`) returns `true` only on explicit Button3/Center=Engrave; Button1=Back→`false`; `drainBtn.Clicked(ctx)` runs every frame (`seedxor_polish.go:94`). Seed-XOR-specific wording "Seed XOR has no built-in check — any wrong part still makes a valid wallet…" (`seedxor_polish.go:85`). Faithful clone of `confirmSLIP39Fingerprint`. `TestSeedXORFingerprintMandatory` (Back at gate→no engrave), `…Button2NoHang`, and `…NamesNoCheck` all pass.

**5. `inputWordsFlow` title param — PASS (additive).** `title==""` calls `layoutTitlef(ctx, dims.X, th.Text, "Word %d of %d", selected+1, len(mnemonic))` (`gui.go:706`) — identical args to the pre-refactor line; the only change is renaming the inline result var to `titleOp` and wrapping in the `if title==""` branch. `TestWordFlowProgressTitle` asserts `"Word 1 of 24"` and is green, proving byte-identical empty-title output. `title!=""` renders free-form `layoutTitle(…, title)`. ALL call sites compile: 2 real calls in gui.go (`:2033`, `:2118`, both `""`) + 8 in gui_test.go (`:285,491,507,604,625,642,662,681`, all `""`). No regression to wallet-backup/SeedScreen/EngraveScreen flows (full `./...` green).

**6. Menu/dispatch — PASS.** `"SEED XOR"` appended at index 4 of `newInputFlow`'s `ChoiceScreen` (`gui.go:2023`); `case 4:` calls `combineSeedXORFlow` and `return m, true` (`gui.go:2064-2069`). The `bip39.Mnemonic` rides the existing `engraveObjectFlow case bip39.Mnemonic:` (`gui.go:1857`, OUTSIDE the diff) — no new dispatch case, no interpretation fork. `TestNewInputFlowSeedXOREntry` drives the full menu path and asserts the recovered mnemonic. No new import added to gui.go (`fmt`/`chaincfg`/`bip39` already present in `seedxor_polish.go`'s own import block).

**7. Go-specifics + scope — PASS.** No slice aliasing in the XOR fold or the `parts[0]` copy (verified above). On Back/error, `combineSeedXORFlow` returns nil `bip39.Mnemonic`; `engraveObjectFlow`/menu only proceed on `ok==true`. No secret logged (`grep` of new files: zero `log`/`Print`/`println`). No `math/big` in `seedxor/` (the only "math/big" hit is the doc-comment word "no math/big"). `backupWalletFlow` (`gui.go:1937`), `masterFingerprintFor` (`gui.go:479`), `engraveObjectFlow` (`gui.go:1855`), and `case bip39.Mnemonic:` (`gui.go:1857`) all sit outside every diff hunk (hunks: 577, 698, 2012, 2022, 2053, 2099) — bodies unchanged. Only the 8 stated files touched. codex32/SLIP-39/BIP-39/backup goldens green.

## Findings

- CRITICAL: none
- IMPORTANT: none
- MINOR: none

## Verdict

**GREEN — 0 Critical / 0 Important. Cleared to merge.**
