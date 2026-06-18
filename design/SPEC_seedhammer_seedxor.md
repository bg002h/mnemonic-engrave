# SPEC — SeedHammer Seed XOR combine (on-device recovery)

**Status:** draft for the opus-architect R0 gate.
**Base:** fork `main` `bc63caa`. Fork-side only (no upstream PR).
**Recon:** `design/cycle-prep-recon-seedxor.md` (`4b3b8db`).
**Architect consult (decisions locked here):** `design/agent-reports/seedhammer-seedxor-design-consult.md`.
**Port source (oracle):** `mnemonic_toolkit::seed_xor` (`seed_xor.rs` `seed_xor_combine` + the in-repo
G1 Coldcard byte-pin tests) + Coldcard `docs/seed-xor.md` vectors.

---

## 1. Goal & scope

Teach the fork to **combine N Coldcard Seed-XOR parts** into the original BIP-39 seed on-device
and engrave it as the native seed plate. Combine direction only. Seed XOR = bit-wise XOR of the
BIP-39 **entropy** of N parts (each part a valid BIP-39 mnemonic), strictly **N-of-N**, all parts
same length; the result is itself a BIP-39 mnemonic (`bip39.New` recomputes its checksum). Reuses
the codex32/SLIP-39 recovery machinery; the result rides the existing BIP-39 engrave path.

### In scope
- A tiny `seedxor` Go package: `Combine(parts []bip39.Mnemonic) (bip39.Mnemonic, error)` (pure
  XOR + validation; ~50 LoC). No `math/big`, no SHA, no field math.
- A `gui/seedxor_polish.go` combine flow: a "How many parts?" picker, a part-word-length pick,
  an N-part collection loop, the **mandatory** Seed-XOR fingerprint gate, then `backupWalletFlow`.
- A new `"SEED XOR"` entry on the input `ChoiceScreen` (Path A — returns a `bip39.Mnemonic`).

### Out of scope (explicit)
- **Seed XOR SPLIT** — needs a CSPRNG the firmware lacks (no `crypto/rand` in the UI path, no
  TRNG/ROSC driver; `driver/otp` is secure-boot, not entropy). Deferred behind a separate
  RNG-driver + "device mints secrets" threat-model cycle. An engraver reproduces seeds; it
  shouldn't generate them.
- **Non-Coldcard-interop lengths** — 15/21-word (20/28-byte) Seed XOR won't restore on a Coldcard
  (the originator); restricting avoids materializing a plate the user's wallet can't ingest. The
  toolkit CLI already serves that power-user niche behind an advisory.
- No two-way "Trezor/verbatim" fork and no interpretation hold-to-confirm (see §3 — there is no
  interpretation ambiguity).

---

## 2. Security invariants (the R0 gate must verify each)

1. **No authentication tag — the fingerprint gate is the ONLY safety net (architect: WORSE than
   SLIP-39).** Any N equal-length BIP-39 mnemonics XOR to *some* valid wallet (the decoy
   property), so there is no consistency relation to check — codex32/SLIP-39 catch a bad set via
   a checksum/digest; Seed XOR catches nothing. Therefore the recovered-master-fingerprint
   confirm is **MANDATORY and unskippable** on the only path to engrave, with **Seed-XOR-specific
   wording that names the absence of a check** (§4.3). An R0 reviewer should treat a
   skippable/soft gate as **Critical**.
2. **The user must already know their target fingerprint** — documented precondition; an operator
   without it gets zero protection (inherent to the primitive, not a fixable gap).
3. **Coldcard-interop lengths only** (16/24/32-byte = 12/18/24-word). `bip39.New` accepts
   16–32 B (mult of 4) and the entry flow is word-count-generic, so this is **NOT enforced for
   free** — `seedxor.Combine` MUST reject 20/28-byte (15/21-word), and the entry offers only
   12/18/24.
4. **Secrets:** part mnemonics & the recovered seed are hand-typed (never over NFC), never
   logged; intermediate entropy buffers wiped (`wipeBytes`). The BIP-39 25th-word passphrase is
   orthogonal (Seed XOR is on the mnemonic/entropy; passphrase handling is unchanged
   `backupWalletFlow`).
5. **Button2-drain idiom** (the recurring Cycle-B R0-C1 EventRouter footgun) replicated on any
   new confirm screen (the fingerprint gate) — `drainBtn.Clicked(ctx)` every frame.

---

## 3. No engrave-artifact ambiguity (confirmed) — drop the fork

The SLIP-39 two-way fork existed because a SLIP-39 master secret is ambiguous (BIP-39 entropy vs
direct BIP-32 seed). **Seed XOR has no such ambiguity by construction:** input type = output type
= `bip39.Mnemonic`; the operation is XOR of BIP-39 entropy; `bip39.New(result)` yields a BIP-39
mnemonic. There is exactly one correct engrave path — `backupWalletFlow` — and no interpretation
question for the user. So: **no Trezor/verbatim fork, no interpretation hold-to-confirm.** The
recovered seed is shown word-by-word + fingerprinted by `backupWalletFlow`'s own
`SeedScreen.Confirm`; the §4.3 gate is an *additional, Seed-XOR-specific* fingerprint screen
*before* that handoff (so its copy can name the no-check hazard).

---

## 4. Design

### 4.1 `seedxor` package — `Combine`

```go
package seedxor

// Combine reconstructs the original BIP-39 seed from N Coldcard Seed-XOR parts:
// bit-wise XOR of the parts' entropy (checksum excluded), then a fresh BIP-39
// checksum on the result. Strictly N-of-N; all parts must be the same,
// Coldcard-interop length (16/24/32-byte = 12/18/24-word). Order-independent.
// Pure: no RNG, no SHA, no math/big.
func Combine(parts []bip39.Mnemonic) (bip39.Mnemonic, error)
```
Algorithm (port of `seed_xor_combine`, `seed_xor.rs:161`): require `len(parts) >= 2`
(`errTooFewParts`); take `e0 := parts[0].Entropy()`; require `len(e0) ∈ {16,24,32}`
(`errBadLength` — the §2.3 interop guard, rejecting 20/28); for each later part require
`len(Entropy()) == len(e0)` (`errMismatchedLengths`); XOR all entropies byte-wise into `out`;
`m := bip39.New(out)` (safe — length validated); `wipe(out)`; return `m`. (`bip39.Mnemonic.Entropy()`
panics on an invalid mnemonic — callers pass only entry-validated/parsed mnemonics, so it's safe;
the unit tests parse from vectors.) `Describe(err)` for the GUI.

### 4.2 GUI combine flow — `gui/seedxor_polish.go`

- `seedXORPartCount(ctx, th) int` — a `ChoiceScreen` "Seed XOR" / "How many parts?" choices
  `{2,3,4,5}` (min 2 per `MIN_SHARES`); 0/Back → cancel.
- `seedXORPartLength(ctx, th) int` — a `ChoiceScreen` "Part length?" choices `{12, 18, 24}` words
  (Coldcard-interop). **Mechanically required:** `inputWordsFlow` fills a *pre-sized* slice
  (`emptyBIP39Mnemonic(nwords)`), so part-1's length must be known before entry — this is the
  one length pick (parts 2..N inherit it, NOT a per-part picker). *(This reconciles the architect's
  "first part fixes L" intent with the pre-sized entry reality; flagged for R0.)*
- `combineSeedXORFlow(ctx, th) (bip39.Mnemonic, bool)`:
  1. `n := seedXORPartCount(...)`; `0` → `(nil,false)`.
  2. `nwords := seedXORPartLength(...)`; `0` → `(nil,false)`. `L = nwords`.
  3. Collect `n` parts: for `i in 0..n`, `m := emptyBIP39Mnemonic(L)`;
     `inputWordsFlow(ctx, th, m, 0, title="Part i of n")` (so every part is exactly `L` words —
     mismatched length is structurally impossible). Back at any prompt → `(nil,false)`.
  4. `seed, err := seedxor.Combine(parts)`; on err → `showError(ctx, th, "Seed XOR",
     seedxor.Describe(err))` + `(nil,false)` (defensive — the pre-sized entry should make
     `errMismatchedLengths` unreachable; keep it for defense-in-depth).
  5. `mfp, _ := masterFingerprintFor(seed, &chaincfg.MainNetParams, "")`; **mandatory**
     `confirmSeedXORFingerprint(ctx, th, mfp)` (§4.3) — `false` → `(nil,false)`.
  6. return `(seed, true)`.

### 4.3 `confirmSeedXORFingerprint` — the mandatory, no-skip safety gate

A confirm screen (clone of `confirmSLIP39Fingerprint`, `slip39_polish.go:433`, with **Button2
drained every frame**) titled "Recovered Fingerprint", body naming the absence of a check:
> `Fingerprint %.8X` / "Seed XOR has no built-in check — any wrong part still produces a
> valid-looking wallet. Confirm this matches your wallet records before engraving."
Button1=Back → `false`; Button3/Center=Engrave → `true`. (Stronger than SLIP-39's wording, per
the architect — here the gate is load-bearing for the whole operation, not a residual belt.)

### 4.4 Menu hook (Path A)

Add `"SEED XOR"` to the input `ChoiceScreen` (`gui.go:~2012`, after `SLIP-39`). Its `case`
calls `m, ok := combineSeedXORFlow(...)`; on ok `return m, true` — a plain `bip39.Mnemonic` that
`engraveObjectFlow`'s existing `case bip39.Mnemonic:` (`gui.go:~1847`) already routes to
`backupWalletFlow`. **No new dispatch case.** Reject the "action on the BIP-39 confirm screen"
alternative (a bare mnemonic carries no "I'm a Seed XOR part" signal; combine consumes N inputs
*before* any single confirm).

---

## 5. Error taxonomy (`seedxor.Describe`)
`errTooFewParts` → "need at least 2 parts"; `errBadLength` → "unsupported length (use 12/18/24
words)"; `errMismatchedLengths` → "all parts must be the same length". Unknown → "invalid".

---

## 6. TDD (Coldcard vectors + toolkit G1 byte-pin as oracle)

- **`seedxor` unit (pure):** embed the captured Coldcard vectors (24-word 3-part →
  `silent toe … indoor`; 12-word 3-part → `cannon … trade`) as `testdata`; assert
  `Combine(parts) == expected` and **order-independence** (shuffle parts → same result). Add the
  toolkit **G1 byte-pin** relation as a cross-check. Negatives: <2 parts → `errTooFewParts`;
  mixed 12+24 → `errMismatchedLengths`; a 15/21-word (20/28-byte) part → `errBadLength` (the §2.3
  interop guard). (Cross-check option: the toolkit's `tests/lib_seed_xor.rs` / Coldcard
  `testing/test_seed_xor.py` vectors.)
- **GUI:** `TestCombineSeedXOR` (drive N parts via the `driveShare`-style helper → assert the
  recovered mnemonic's fingerprint screen + that `backupWalletFlow` is reached);
  `TestSeedXORFingerprintMandatory` (the gate is on the only success path; Back → no engrave);
  `TestSeedXORBackoutRecognized`; the Button2-drain no-hang regression; a length-mismatch /
  `errBadLength` error-path test.
- Host: `/home/bcg/.local/go/bin/go test ./seedxor/ ./gui/ ./bip39/` + `go vet` + `gofmt -l`.
  Existing guards (codex32/SLIP-39/BIP-39/backup goldens) stay green.

---

## 7. File manifest

| File | Change |
|---|---|
| `seedxor/seedxor.go` | **new** — `Combine` + `Describe` + sentinels (port of `seed_xor_combine`). |
| `seedxor/seedxor_test.go` + `seedxor/testdata/` | **new** — Coldcard/toolkit vectors + negatives. |
| `gui/seedxor_polish.go` | **new** — `seedXORPartCount`, `seedXORPartLength`, `combineSeedXORFlow`, `confirmSeedXORFingerprint`. |
| `gui/seedxor_polish_test.go` | **new** — combine flow + mandatory-gate + no-hang tests. |
| `gui/gui.go` | **modify** — add `"SEED XOR"` to the input `ChoiceScreen` + its `case` (returns `bip39.Mnemonic`); no new dispatch, no new import beyond what's present. |

**Unchanged (reused, must stay green):** `bip39/` (`Entropy`/`New`), `backupWalletFlow`/
`masterFingerprintFor`/`SeedScreen.Confirm`, `codex32/`, `slip39/`, `backup/`.

---

## 8. Process

Full gated pipeline (S cycle, seed-bearing): this spec → opus R0 loop to 0C/0I → plan R0 loop →
single-implementer TDD in worktree `seedhammer-wt-seedxor` (branch `feat/seedxor-combine` off
`bc63caa`), Coldcard/toolkit vectors as oracle → mandatory whole-diff execution review → fold →
merge no-ff signed into fork `main` → push `bg002h`. Reviews persist verbatim to
`design/agent-reports/seedhammer-seedxor-*`. Commits SSH-signed + DCO, author Brian Goss. No
upstream PR.
