# B2a-ii lens 1 — fold re-review of I1 + M1 (`3c477b9`, context `c3e73a3`)

Scope: the ONE question — did the fold fix I1 and M1, and did it break anything or
claim anything untrue? Not a fresh audit. Suite state assumed green per brief.

```
FOLD VERDICT: FIXED
NEW DEFECTS: 0 Critical / 1 Important / 1 Minor / 0 Nit
```

Both findings are fixed, and all four scrubs are **safe** — no aliasing, no
control-flow change, no caller broken. The two new defects are both in the new
**prose**, specifically in the rewritten HONEST CAVEAT, which is the artefact the
brief singled out. The code needs no change.

---

## I1 / M1 — the fixes themselves

### The priority check: does `hdkeychain.NewMaster` retain `seed`? **No. The wipe is safe.**

`~/go/pkg/mod/github.com/btcsuite/btcd/btcutil/v2@v2.0.0/hdkeychain/extendedkey.go:654-681`.
`NewMaster` writes `seed` into an HMAC-SHA512 and never stores it:

```go
hmac512 := hmac.New(sha512.New, masterKey)
_, _ = hmac512.Write(seed)
lr := hmac512.Sum(nil)          // FRESH buffer, not backed by seed
secretKey := lr[:len(lr)/2]     // slices of lr
chainCode := lr[len(lr)/2:]
parentFP := []byte{0x00,0x00,0x00,0x00}
return NewExtendedKey(net.HDPrivateKeyID[:], secretKey, chainCode, parentFP, 0, 0, true), nil
```

`NewExtendedKey` (`:124-138`) stores exactly those slices. Every field of the
returned `*ExtendedKey` is backed by `lr`, by the `chaincfg` global, or by a fresh
literal. **Nothing points into `seed`.** `defer wipeBytes(seed)` in
`gui/gui.go:227-241` cannot corrupt the returned key. The silently-wrong-key
failure mode does not exist here.

Corollary checked: `bip39.MnemonicSeed` (`bip39/bip39.go:217-226`) returns
`pbkdf2.Key(...)`, a freshly allocated 64-byte slice owned by nobody else — so
the wipe also cannot reach back into `m` or a shared buffer.

### `defer mk.Zero()` in `masterFingerprintFor` (`gui/gui.go:545-562`) — sound

- `mk` is non-nil wherever the defer is reachable: `deriveMasterKey` returns
  `mk, err == nil`, and the defer is placed **after** the `if !ok { return }`.
- `return bip32.Fingerprint(pkey), nil` — Go evaluates result expressions, assigns
  to the (unnamed, therefore un-modifiable) result params, *then* runs defers. The
  `uint32` is fully computed before `Zero()`.
- `bip32.Fingerprint` (`bip32/bip32.go:38-41`) is `Hash160(pkey.SerializeCompressed())[:4]`
  → `binary.BigEndian.Uint32`. Value semantics all the way out; `pkey` is dead after.
- **The claim that derive.go's R0-C1 `Neuter`-aliasing warning does not bite here is
  TRUE.** That warning is about `Neuter()` (`extendedkey.go:486-504`) passing
  `k.chainCode` and `k.parentFP` **by reference** into the neutered child. This
  function never calls `Neuter`, never serialises, and `mk` is a `NewMaster` key
  whose `parentFP` is a fresh literal. Separately verified that upstream `Zero()`
  (`:634-644`) sets `k.version = nil` rather than zeroing it — important, since
  `version` aliases `net.HDPrivateKeyID[:]` in the **global** `chaincfg.MainNetParams`.
  Upstream handles it; the fold does not disturb it.

### The validity probe (`gui/gui.go:2386-2401`) — control flow identical, nil-safe

Before: `if _, ok := deriveMasterKey(...); !ok { showErr; continue }`.
After: `mk, ok := ...; if ok { mk.Zero() }; if !ok { showErr; continue }`.
Same branch, same `continue`, same `return true`. `mk.Zero()` runs only when
`ok`, and `ok == (err == nil)` is exactly when `mk` is non-nil. No shadowing
problem with the later `e, ok := inp.Next(...)` (separate block, `:=`).

### `clear(m)` in `seal.Classify` (`seal/record.go:140-147`) — sound

`bip39.Parse` returns a freshly `append`-built `Mnemonic` (`[]Word`, `Word = int`),
independent of the input `b`. Nothing reads `m` after the `clear`, and the branch
is gated on `err == nil` only — **no input's classification changes**. `m` is
non-nil whenever `err == nil`, and `clear(nil)` is a no-op regardless.

### The five other callers — none depends on survival

`masterFingerprintFor` has six non-test call sites in **five** distinct functions,
each consuming only the `uint32`: `combineSeedXORFlow` (`gui/seedxor_polish.go:66`),
`engraveBip85Child` (`gui/bip85.go:137`), `backupWalletFlow` (`gui/gui.go:2166` and
`:2175`), `engraveRecoveredSLIP39` (`gui/slip39_polish.go:432`),
`unlockEngraveMnemonic` (`gui/unlock_session.go:214`). `deriveMasterKey` has exactly
two non-test callers — `masterFingerprintFor` and the probe — and both now Zero.
The commit's "five calling functions" count is correct; "byte-identical at
`421dca8`" verified against `git show 421dca8:gui/gui.go`.

---

## New defects

### D1 — Important

**WHERE** `gui/unlock_session.go:175-187` (the HONEST CAVEAT table).

**DEFECT** The header claims "**the full inventory** of seed-equivalent copies this
path makes". It is not full: at least three wipeable seed-equivalent buffers on
this exact path are absent, and the `DROPPED` row's stated *reason* is wrong for
both items it lists.

**EVIDENCE** Traced `unlockEngraveMnemonic` end to end.

1. **`sentence []byte` — the plaintext mnemonic itself.**
   `bip39/bip39.go:217-226`, reached via `unlockEngraveMnemonic` →
   `masterFingerprintFor` → `deriveMasterKey` → `MnemonicSeed`:
   ```go
   var sentence []byte
   for i, w := range m { sentence = append(sentence, bytes.ToLower([]byte(LabelFor(w)))...) ... }
   return pbkdf2.Key(sentence, ...)
   ```
   Never wiped. Built by `append` from nil, so it also leaves several orphaned
   reallocation-generation copies of a *prefix* of the sentence on the heap. This
   is arguably more sensitive than the master key the fold *did* fix, and it is
   the one row a reader would most expect to see.
2. **The `[]byte` behind `string(seedqr.QR(m))`.** `seedqr/seedqr.go:24-33` returns
   `qr.Bytes()` from a `bytes.Buffer` — the 96-digit SeedQR encoding, a wipeable
   buffer, dropped unwiped. The table lists only the derived *string*.
3. **`qrc.Bitmap []byte`** in `engraveSeed` (`gui/gui.go:522`,
   `qr.Encode(string(seedqr.QR(m)), qr.M)`). `kortschak-qr@v0.3.2/qr.go:84-89`:
   `type Code struct { Bitmap []byte; ... }` — the QR of the seed as a wipeable
   byte buffer, dropped.

   And the reason attached to the `DROPPED` row — "immutable Go strings, unwipeable
   by construction" — is **false for `words []string`**. `bip39.LabelFor`
   (`bip39/bip39.go:79-89`) returns `words[start:end]`, a substring of the
   **public** package wordlist; the string bytes are not secret at all. The secret
   is *which* words and in what order, i.e. the slice header array — and
   `clear(words)` destroys exactly that, for free. It is wipeable in the only sense
   that matters. (The claim *is* correct for `string(seedqr.QR(m))`, which is a
   fresh allocation of the secret digits.)

**CONSEQUENCE** This is the precise failure mode the brief names: a caveat that
reads as exhaustive and is not. A maintainer auditing §10.2.2 against this table
closes the topic believing the plaintext mnemonic sentence is accounted for. It is
strictly worse than the old comment, which at least did not claim to be a list.

**FIX** Prose only — no code change is required for this to be correct. Either
(a) add the three rows under a `DROPPED (wipeable, follow-up)` heading and correct
the `words []string` reason, or (b) drop the word "full" and say "the copies this
function and its callees can reach", naming `bip39.MnemonicSeed`'s `sentence` as
known-unwiped and out of this fold's scope (it is shared upstream machinery — same
argument the commit already makes for `deriveMasterKey`). The actual scrubs belong
in a follow-up, not here; `sentence` is unreachable from `gui` and fixing it is a
`bip39` change that also touches `deriveAccountXpub` and `bip85.go`.

### D2 — Minor

**WHERE** `gui/unlock_session.go:189-191`.

**DEFECT** "three of them are UNPINNED: **no test can observe them without unsafe**."
The count of three is right; the stated reason is false for `rec`, and it
suppresses a cheap honest test.

**EVIDENCE** `gui/unlock_session_test.go:364` `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp`
reads `p.Secret[first].Record` directly while the engrave screen is up — no
`unsafe`. It does not cover `unlockEngraveMnemonic` only because it runs on vector
F, whose three secrets are all `ms1` codex32 strings
(`seal/testdata/vectors.json`, vector `F`), so `firstSecretIdx`
(`gui/unlock_session_test.go:495`) lands on the codex32 arm. `rec` is
`p.Secret[i].Record` itself (`gui/unlock_session.go:112`), so it is directly
observable. (Row `m` *is* genuinely pinned, by
`TestMnemonicWordsAreZeroWhenThePlateReachesEngrave`, `:639-669` — the count of
three unpinned is therefore correct.)

**CONSEQUENCE** `clear(rec)` on the **mnemonic** arm is an unpinned §10.2.2
guarantee: deleting that line today leaves the suite green. The comment tells the
next maintainer not to bother trying.

**FIX** The cheap honest test the brief invited: run the existing body against
vector A, whose single secret is a bare 24-word mnemonic
(`gui/unlock_session_test.go:646`), asserting `allZeroBytes(p.Secret[0].Record)`
after `h.mustReach("Insert a blank plate")`. Roughly a ten-line copy of the
codex32 test, no `unsafe`. Then reword the comment to "two of them are UNPINNED —
the seed and the master key are internal to functions that return neither."

---

## What I checked and found sound

- `hdkeychain.NewMaster` does not alias `seed` (read from module-cache source, not
  inferred from green tests). `defer wipeBytes(seed)` is safe.
- `defer mk.Zero()` cannot race the `uint32` return; `Neuter`-aliasing warning
  correctly ruled inapplicable; upstream `Zero()` nils rather than zeroes
  `version`, so the `chaincfg` global is not corrupted.
- Probe control flow byte-for-byte equivalent; `mk` non-nil exactly when `ok`.
- `clear(m)` in `Classify` changes no classification for any input; nothing reads
  `m` after.
- All five `masterFingerprintFor` callers and both `deriveMasterKey` callers
  consume value types only; none depends on the key or seed surviving.
- Commit-message citations verified: `derive.go:21` is `defer wipeBytes(seed)`;
  `derive.go:31` is `masterFP = bip32.Fingerprint(pk) // capture BEFORE zeroing
  master`; `gui/bip85.go:76-77` does the same; `seal/record.go:436-445` does
  document the not-regression-tested property attributed to it; the three
  functions were byte-identical at `421dca8`; five calling functions.
- `c3e73a3`'s claim verified: `Payload.Wipe` loops `p.Secret` **and** `p.Public`
  (`seal/open.go`, the two loops at :58-63), `SecretsResident` scans `p.Secret`
  only (`seal/session.go:29-41`).
