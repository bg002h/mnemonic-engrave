# SPEC — SeedHammer SLIP-39 secret recovery (Cycle D)

**Status:** draft for the opus-architect R0 gate.
**Base:** fork `main` `20fa4c4` (Slice 3 merged). Fork-side only (no upstream PR).
**Recon:** `design/cycle-prep-recon-slip39-recovery.md` (`3c1e8ce`).
**Port source (oracle):** `mnemonic-toolkit/crates/mnemonic-toolkit/src/slip39/` — our own
audited Rust (`gf256.rs`, `lagrange.rs`, `feistel.rs`, `share.rs`, `mod.rs::slip39_combine`).

---

## 1. Goal & scope

Add on-device **SLIP-0039 secret recovery** to the fork: collect enough SLIP-39 shares
(satisfying the group threshold and each represented group's member threshold), reconstruct
the master secret, and engrave it as the device-native BIP-39 seed plate. Recovery mirrors
the Cycle-B codex32 multi-share recover *flow*, but the crypto is a fresh in-tree Go port of
`mnemonic_toolkit::slip39` (GF(256) Shamir + 4-round Feistel + PBKDF2 — a different
cryptosystem from codex32's GF(32)). The COMBINE direction only; no split/generation.

### In scope
- New Go crypto: `slip39/gf256.go`, `slip39/lagrange.go`, `slip39/feistel.go`,
  `slip39/combine.go`; extend `slip39/share.go` to extract the share-VALUE payload.
- **All valid share lengths**: master-secret ∈ {16,20,24,28,32} B → share word counts
  {20,23,27,30,33}. (Today `ParseShare` accepts only 20 and hard-rejects 33.)
- **Full two-level recovery**: group threshold over groups; per-group member threshold.
- **Optional SLIP-39 passphrase** (EMS decryption), entered on the Slice-2
  `PassphraseKeyboard`; default empty `""`.
- GUI: a Recover action on the SLIP-39 confirm screen + a two-level share-collection flow +
  an entropy→BIP-39 engrave path reusing `backupWalletFlow`.

### Out of scope (explicit)
- SLIP-39 SPLIT/share generation (no RNG, no Feistel-encrypt). Recovery only.
- SLIP-39 over NFC/RF — stays disabled; shares are hand-typed (air-gapped), like Cycle C.
- Trezor-native "master secret = direct BIP-32 seed" interpretation (see §3).
- The Cycle-C single-share entry length widening — tracked separately as FOLLOWUP
  `seedhammer-slip39-cycleC-all-lengths` (this cycle's `ParseShare` length work makes it a
  trivial follow-on).

---

## 2. Security invariants (highest priority; the R0 gate must verify each)

1. **Shares & SLIP-39 passphrase are hand-typed, never over NFC/RF.** The SLIP-39 NFC path
   stays disabled.
2. **The recovered master secret and the SLIP-39 passphrase never leave the device**, never
   go over NFC, and are never logged. The only thing engraved is the seed plate the user
   explicitly confirms.
3. **The SLIP-39 (EMS-decryption) passphrase is distinct from the BIP-39 25th-word
   passphrase.** They are entered at different stages, feed different algorithms, and must
   never be conflated. A wrong SLIP-39 passphrase yields a different-but-valid master secret
   with no error (SLIP-0039 plausible-deniability property) — the UI must not claim to
   "verify" it.
4. **Digest gate:** for any threshold ≥ 2 recovery layer, the HMAC-SHA256 digest check
   (`HMAC-SHA256(R, S)[:4] == digest`) MUST gate success; a forged/mismatched share set must
   be refused, not silently engraved.
5. **No `math/big` in the SLIP-39 crypto.** GF(256) is byte-table-based. (`bip39.New` for the
   final entropy→words step uses `math/big`, but that is the existing, already-in-firmware
   BIP-39 package, not the new SLIP-39 code.)
6. **TinyGo/RP2350 (`int` is 32-bit):** all multi-word bit assembly stays byte-oriented or
   `uint64` (the existing header decode already does this).

---

## 3. The "what to engrave" model (verified, decided)

**Verified fact (against the official SLIP-0039 vector #1):** using the recovered master
secret directly as the BIP-32 seed (`from_seed(MS)`) reproduces the vector's published
`bip32_xprv` exactly. So in the *standard*, the master secret IS the BIP-32 seed, not BIP-39
entropy, and converting it to BIP-39 words would derive a different wallet **unless** the
backup was created under the "master secret = BIP-39 entropy" convention.

**The constellation uses exactly that convention** (`mnemonic_toolkit`:
`VALID_SECRET_LENGTHS=[16,20,24,28,32]`, split from a BIP-39 phrase, `combine --to phrase`).
This device is constellation tooling, so:

- **Decision:** the device interprets the recovered master secret as **BIP-39 entropy** and
  engraves the native BIP-39 seed plate (words + SeedQR) via `bip39.New(entropy)` →
  `backupWalletFlow`. The recovered-bytes length (16/20/24/28/32) maps to 12/15/18/21/24
  BIP-39 words.
- **Documented assumption (must appear in the spec + an on-device or doc note):** this is
  correct for constellation-origin SLIP-39 backups; a Trezor-native backup (MS used directly
  as the seed) is out of the supported model and would round-trip to a different wallet.

---

## 4. Go crypto port (combine direction)

Faithful port of the Rust. All functions panic-free at the public boundary; internal helpers
may rely on validated preconditions (matching the Rust's `assert!`-as-precondition style, but
**the Go port must convert would-be panics on attacker-controlled input into returned
errors** — see §4.4).

### 4.1 `slip39/gf256.go` — GF(2⁸) Rijndael field

- `reductionPoly = 0x11b`; **generator = 3**.
- Package-level `expTbl [256]byte`, `logTbl [256]byte`, built once in `init()` (TinyGo-safe):
  `x:=uint16(1); for i:=0;i<255;i++ { expTbl[i]=byte(x); logTbl[x]=byte(i); x=(x<<1)^x; if x&0x100!=0 { x ^= reductionPoly } }; expTbl[255]=1`.
- `gfAdd(a,b byte) byte { return a^b }`.
- `gfMul(a,b byte) byte` — 0 if either is 0; else `expTbl[(logTbl[a]+logTbl[b]) mod 255]`
  (use the `>=255 ? -255` form, not `%`, matching the Rust; both are correct).
- `gfInv(a byte) byte` — precondition `a!=0`; `expTbl[(255-logTbl[a]) mod 255]`.
- `gfDiv(a,b byte) byte` — precondition `b!=0`; `gfMul(a, gfInv(b))`.

`gfInv(0)`/`gfDiv(_,0)` are unreachable in the combine path (interpolation denominators are
products of `xi^xj` over **distinct** x-coordinates, guaranteed non-zero by the
distinct-index validation in §4.3). They may panic as an internal invariant; §4.4 ensures no
attacker input reaches them.

### 4.2 `slip39/lagrange.go` — interpolation over GF(256)

- `interpolateAt(points []point, x byte) byte` — `point{x, y byte}`; Lagrange basis with XOR
  subtraction: `L_i = Π_{j≠i}(x⊕xj)/(xi⊕xj)`, `result = Σ y_i·L_i`. Precondition: distinct
  `xi` (enforced upstream).
- `interpolateSecretAt(points []bytePoint, x byte) []byte` — `bytePoint{x byte; y []byte}`;
  runs `interpolateAt` per byte position; all `y` equal length (enforced upstream).
- Constants `secretIndex = 255`, `digestIndex = 254`, `digestLen = 4`.

### 4.3 `slip39/combine.go` — two-level recovery

Port of `slip39_combine` + `recover_secret`. Public surface:

```go
// Combine reconstructs the SLIP-39 master secret from a set of shares.
// All shares must share identifier/extendable/iterationExp/groupThreshold/
// groupCount and value length; exactly groupThreshold distinct groups must be
// present, each with exactly its memberThreshold shares at distinct member
// indices. passphrase is the SLIP-39 EMS-decryption passphrase ("" = none).
// Returns the master-secret bytes (BIP-39 entropy sizes) or a classifiable error.
func Combine(shares []Share, passphrase []byte) ([]byte, error)
```

Algorithm (mirrors `mod.rs:206-331`):
1. Empty → `errEmptyShares`.
2. Per-share value length ∈ {16,20,24,28,32} else `errInvalidShareValueLength` (report input
   index).
3. Cross-share consistency vs `shares[0]`: identifier, extendable, iterationExp,
   groupThreshold, groupCount, value length — each its own sentinel
   (`errIdentifierMismatch`, `errExtendableMismatch`, `errIterationExponentMismatch`,
   `errGroupThresholdMismatch`, `errGroupCountMismatch`, `errShareValueLengthMismatch`).
4. Group by `GroupIndex` (deterministic order — sort the keys). Per group: uniform
   `MemberThreshold` (`errMemberThresholdMismatch`); distinct member indices
   (`errDuplicateMemberIndex`); **exactly** `memberThreshold` shares (`errInsufficientShares`
   — both too-few and too-many); then `recoverSecret(mt, memberPoints)` → group value.
5. **Exactly** `groupThreshold` groups present (`errInsufficientShares`, group-level).
6. `recoverSecret(groupThreshold, groupPoints)` → EMS.
7. `feistelDecrypt(ems, passphrase, iterationExp, identifier, extendable)` → master secret.

`recoverSecret(threshold, pts)`:
- `threshold == 1` → the single value (no digest).
- else `S = interpolateSecretAt(pts, 255)`, `D = interpolateSecretAt(pts, 254)`;
  `digest=D[:4]`, `R=D[4:]`; recompute `HMAC-SHA256(key=R, msg=S)[:4]`; mismatch →
  `errDigestVerificationFailed`; else `S`.

### 4.4 Panic-safety on attacker input (R0-critical)

The Rust uses `assert!` for "caller validated" preconditions. In the firmware the share
SET is attacker/typo-controlled, so **all preconditions that could be violated by input must
be checked and returned as errors BEFORE any interpolation runs** (steps 2–5 above), so that
by the time `interpolateAt`/`gfDiv` execute, x-coordinates are guaranteed distinct and value
lengths equal. The implementer MUST confirm (and a test MUST assert) that no malformed input
set reaches a panic. (This is the SLIP-39 analogue of the Cycle-B `ConsistentShares`
panic-precondition finding.)

### 4.5 `slip39/feistel.go` — 4-round Feistel decrypt

Port of `feistel::decrypt` (`feistel.rs`):
```go
// feistelDecrypt runs the SLIP-0039 4-round Feistel in reverse to turn the
// encrypted master secret (EMS) into the master secret.
func feistelDecrypt(ems, passphrase []byte, iterationExp int, identifier int, extendable bool) []byte
```
- `n=len(ems)` (even, 16..32). `half=n/2`. `L,R = ems[:half], ems[half:]`.
- `itersPerRound = (10000 << iterationExp) / 4` (= `2500·2^e`).
- `saltPrefix`: `extendable` → empty; else `"shamir" || be16(identifier)`.
- rounds `i = 3,2,1,0`: `F = pbkdf2.Key(hmacSHA256, password=[]byte{byte(i)}||passphrase,
  salt=saltPrefix||R, iters=itersPerRound, dkLen=half)`; `L = L ⊕ F`; swap(L,R).
- return `R || L`.

Uses `crypto/sha256`, `crypto/hmac`, `golang.org/x/crypto/pbkdf2` (firmware already depends on
`x/crypto`; all TinyGo-importable). **Verified facts the port must honor** (cycle-prep §2):
reduction `0x11b`/generator 3; decrypt round order `[3,2,1,0]` with output `R||L`; salt is
`"shamir"||id` (ext=0) / empty (ext=1) — NOT `"shamir_extendable"` (that string is RS1024-only);
iteration exponent is 4-bit; passphrase enters only here.

### 4.6 `slip39/share.go` — extend to extract the share value

Today `ParseShare` decodes the header + verifies RS1024 but discards the value. Add:
- Accept share word counts {20,23,27,30,33} (replace the 20-only / 33-reject gate). Compute
  `valueWords = W-7` (4 header + 3 checksum); `padBits = (10*valueWords) % 16` — reject if
  `padBits > 8` (`errBadPadding`); `valueBytes = (10*valueWords - padBits) / 8` ∈
  {16,20,24,28,32}.
- Unpack the value words (indices 4 .. W-4) as a big-endian bit stream into a big integer-free
  byte assembly (shift into a running `uint` accumulator, byte-oriented), strip the `padBits`
  leading zero bits, and **verify all stripped pad bits are 0** (`errBadPadding`). Mirror
  `share.rs`'s `value_int.to_bytes(valueBytes)`-overflow check.
- Add a `Value []byte` field to `Share` (the extracted member share value), populated by
  `ParseShare`. Existing fields unchanged; the Cycle-C single-share confirm/engrave path
  ignores `Value`.
- New sentinels: `errBadPadding`. `Describe` extended for it + the §4.3 combine sentinels (so
  the GUI can show a class). Drop `errUnsupportedSize` (256-bit is now supported); map any
  lingering reference accordingly.

### 4.7 Two-level `ConsistentShares` analogue (incremental validation for the GUI)

```go
// ConsistentShares reports whether a partial share set is mutually consistent
// (same identifier/extendable/iterationExp/groupThreshold/groupCount/value len,
// and no duplicate (groupIndex, memberIndex) pair). Used to validate each newly
// entered share eagerly during collection (mirrors codex32.ConsistentShares,
// but two-level). It does NOT check counts (validates partial sets).
func ConsistentShares(shares []Share) error
```
Returns the same sentinels as §4.3 steps 3–4 where applicable, plus
`errDuplicateMemberIndex` for a repeated `(GroupIndex, MemberIndex)`.

---

## 5. GUI recovery flow (mirror Cycle-B codex32 recover)

In `gui/slip39_polish.go` (the only GUI file changed; like Cycle C, **all `fmt`-using code
lives here so `gui.go` gets no new imports**).

### 5.1 Confirm action enum + Recover button
Replace `confirmSLIP39Flow`'s `bool` return with:
```go
type slip39ConfirmAction int
const ( slip39Back slip39ConfirmAction = iota; slip39Engrave; slip39Recover )
func confirmSLIP39Flow(ctx, th, s slip39words.Share) slip39ConfirmAction
```
- Button1 → `slip39Back`; Button3 (AltButton Center) → `slip39Engrave`; **Button2 →
  `slip39Recover`, offered only when the share is part of a multi-share set**
  (`s.MemberThreshold > 1 || s.GroupThreshold > 1`).
- **Unconditional Button2 drain** (the Cycle-B R0-C1 footgun): the `recoverBtn.Clicked(ctx)`
  must be called every frame even when Recover isn't offered, so a queued Button2 cannot
  block the EventRouter and stall Button3 in the direct-call tests.
- Title/labels branch: a lone share (`slip39Engrave` only) engraves verbatim (existing
  behavior); a multi-share offers Recover.

### 5.2 `recoverSLIP39Flow`
```go
// recoverSLIP39Flow collects the remaining shares needed to recover the master
// secret, decrypts with an optional SLIP-39 passphrase, and returns the
// recovered seed as a BIP-39 mnemonic. ("", false) on Back/abort.
func recoverSLIP39Flow(ctx *Context, th *Colors, first slip39words.Share) (bip39.Mnemonic, bool)
```
1. Seed `shares := []Share{first}`.
2. **Collect until the set is sufficient.** Sufficiency = exactly `groupThreshold` distinct
   groups each holding exactly its `memberThreshold` shares (the Combine precondition). Drive
   a per-share prompt with a progress title: group/member context, e.g. `"Group g · share
   m of t"` (and the count of groups still needed). Reuse `inputSLIP39Flow(ctx, th, mnemonic,
   0, title)` — add a `title` parameter (mirrors Cycle-B's `inputCodex32Flow(…, title)`).
3. Per newly entered share: `ParseShare`; reject a duplicate `(group,member)` or a metadata
   mismatch eagerly via `ConsistentShares(append(shares, cand))` → `showError(…,
   slip39words.Describe(err))` + re-prompt. Back at any prompt aborts recovery → `("", false)`.
4. When sufficient, **optionally collect the SLIP-39 passphrase** via the Slice-2
   `PassphraseKeyboard` (a choice "SLIP-39 passphrase?" → Skip = empty, or enter). Label it
   explicitly "SLIP-39 passphrase (not a BIP-39 passphrase)".
5. `secret, err := slip39words.Combine(shares, passphrase)`. On error (e.g. digest mismatch
   — wrong/forged set) → `showError(…, Describe(err))` and abort (or allow retry). On success:
   `m := bip39.New(secret)`; return `(m, true)`.
6. Scrub: keep share/secret/passphrase byte slices local; do not retain after return.

### 5.3 Engrave dispatch
In `engraveSLIP39` (the `case slip39words.Share:` handler), loop over `confirmSLIP39Flow`:
- `slip39Back` → return `true` (recognized; never "Unknown format" — the Cycle-A/B rule).
- `slip39Engrave` → engrave the single share verbatim (existing path).
- `slip39Recover` → `m, ok := recoverSLIP39Flow(ctx, th, scan)`; if `!ok`, continue the loop
  (back to confirm); if ok, **engrave the recovered seed** via `backupWalletFlow(ctx, th, m)`
  (reuses the Slice-3 confirm → optional BIP-39 passphrase → fingerprint choice → SeedQR+words
  engrave), then return `true`.

**Two-passphrase UX note (documented, R0-relevant):** during recovery the user may enter a
SLIP-39 passphrase (decrypts shares → entropy); then `backupWalletFlow` may *separately* offer
a BIP-39 25th-word passphrase (for the engraved fingerprint). These are independent and both
optional; the labels must make the distinction explicit.

---

## 6. Error taxonomy

`slip39.Describe(err) string` returns short GUI labels for every sentinel: existing
(`bad checksum`, `unknown word`, `wrong length`) + new: `bad padding`, `id mismatch`,
`extendable mismatch`, `iteration mismatch`, `group threshold mismatch`, `group count
mismatch`, `value length mismatch`, `member threshold mismatch`, `duplicate share`, `not
enough shares`, `bad share set` (digest). Unknown → `invalid`. (`256-bit not supported` is
removed.)

---

## 7. TDD vector set (from official `vectors.json`, pre-verified — cycle-prep §3)

Embed the chosen vectors as test fixtures (the Rust toolkit is the cross-check oracle; run it
on the same vectors and diff). Passphrase for all valid vectors is `"TREZOR"`.

- **Crypto unit (combine):** idx 3 (basic 2-of-3, 128-bit) → `b43c…0864`; idx 17 (group-thr
  2-of-4, 128-bit) → `7c33…8d11`; idx 35 (256-bit multi-group); idx 42 (extendable, ext=1);
  idx 0 (1-of-1, T==1 no-digest path). Passphrase pair on idx 3: `"TREZOR"` vs `""` → distinct
  secrets.
- **share.go value extraction:** decode each positive vector's shares → assert the value byte
  length ∈ {16,20,24,28,32} and the round-trip Combine. Add a 256-bit (33-word) and a 23/27/
  30-word length case if present in the corpus; otherwise synthesize via the Rust oracle.
- **Negatives (Combine refuses, correct sentinel):** idx 1 (bad checksum — at ParseShare),
  idx 4 (insufficient members), idx 13 (insufficient groups), idx 5 (id mismatch), idx 9
  (group-thr>count), **idx 12 (invalid digest — the critical gate)**, idx 2/3-class (bad
  padding). Plus a panic-safety test: a malformed/duplicate-index set returns an error, never
  panics (§4.4).
- **GUI:** `TestConfirmSLIP39MultiOffersRecover` / `…LoneNoRecover`; `TestRecoverSLIP39`
  (drive shares via `ctx.Router`, assert the recovered mnemonic); `TestRecoverSLIP39Mismatch`
  (eager ConsistentShares error); `TestRecoverSLIP39BackoutRecognized`; a passphrase-path
  test. Mirror the Cycle-B `gui/codex32_polish_test.go` patterns + the Button2-drain
  no-hang regression.

Host test command: `/home/bcg/.local/go/bin/go test ./slip39/ ./gui/ ./bip39/`. Existing
guards (`TestConfirmSLIP39Render`, `TestEngraveSLIP39BackoutRecognized`, codex32, BIP-39,
backup goldens) stay green.

---

## 8. File manifest

| File | Change |
|---|---|
| `slip39/gf256.go` | **new** — GF(256) field (port of `gf256.rs`). |
| `slip39/lagrange.go` | **new** — interpolation (port of `lagrange.rs`). |
| `slip39/feistel.go` | **new** — 4-round Feistel decrypt (port of `feistel.rs`). |
| `slip39/combine.go` | **new** — `Combine` + `recoverSecret` + `ConsistentShares` (port of `slip39_combine`/`recover_secret`). |
| `slip39/share.go` | **modify** — accept {20,23,27,30,33} words; extract `Value []byte` with padding validation; new `errBadPadding`; extend `Describe`; drop `errUnsupportedSize`. |
| `slip39/*_test.go` | **new/modify** — crypto vectors + share-value + negative + panic-safety. |
| `gui/slip39_polish.go` | **modify** — `slip39ConfirmAction` enum + Recover button (Button2 drain), `recoverSLIP39Flow`, `inputSLIP39Flow` title param, engrave dispatch → `backupWalletFlow`. |
| `gui/slip39_polish_test.go` | **modify** — recover-flow tests. |
| `gui/gui.go` | **modify only if** `inputSLIP39Flow` lives there (add the `title` param); no new imports. |

`codex32/`, `mdmk.go`, `backup/`, `bip32/`, `seedqr/`, `gui/passphrase_keyboard.go`,
`gui/gui.go` BIP-39/Slice-3 logic — **unchanged** (reused, must stay green).

---

## 9. Process

Gated pipeline: this spec → opus R0 loop to 0C/0I → implementation plan → plan R0 loop to
0C/0I → single-implementer TDD in a worktree (`seedhammer-wt-slip39rec`, branch
`feat/slip39-recovery` off `20fa4c4`), Rust as oracle → mandatory whole-diff adversarial
execution review → fold → merge no-ff signed into fork `main` → push `bg002h`. Reviews persist
verbatim to `design/agent-reports/seedhammer-slip39-recovery-*`. Commits SSH-signed + DCO,
author Brian Goss. SemVer: firmware (version injected via ldflags; no committed constant). No
upstream PR (standing strategy).
