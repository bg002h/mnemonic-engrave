# SPEC — SeedHammer SLIP-39 secret recovery (Cycle D)

**Status:** R0 round 1 — folded against the opus R0 gate (`…-spec-review-R0.md`, 0C/2I+6m)
and the 3-lens architect panel (crypto-security, firmware-resource, design/decomposition;
persisted in `design/agent-reports/seedhammer-slip39-recovery-architect-panel-*`).
**Base:** fork `main` `20fa4c4` (Slice 3 merged). Fork-side only (no upstream PR).
**Recon:** `design/cycle-prep-recon-slip39-recovery.md` (`3c1e8ce`).
**Port source (oracle):** `mnemonic-toolkit/crates/mnemonic-toolkit/src/slip39/` — our own
audited Rust (`gf256.rs`, `lagrange.rs`, `feistel.rs`, `share.rs`, `mod.rs`).

---

## 0. Decomposition — TWO gated phases under one cycle (architect panel, unanimous)

Cycle D ships a fresh from-scratch cryptosystem **and** a novel two-level UX. Per the
design-lens recommendation it is split into two independently-gated, independently-mergeable
phases (Cycle B was one cycle only because it added **zero** crypto):

- **D1 — crypto port + share-value extraction (no GUI).** Files `slip39/{gf256,lagrange,
  feistel,combine}.go` + the `slip39/share.go` widening (all valid lengths, `Value []byte`,
  padding validation, the structural group-threshold check) + best-effort scrubbing + the
  full vector/negative/panic-safety/round-trip test suite diffed against the Rust oracle.
  **Mergeable on its own** (dormant until the GUI calls `Combine`; cannot regress any
  user-facing flow). Own plan → R0 gate → implement → whole-diff review → merge.
- **D2 — GUI recover flow + engrave bridge.** The `slip39ConfirmAction` enum + Recover
  button, `recoverSLIP39Flow`, the **widened share entry** (resolves R0 I1), the two-level
  **collection roster** UX, the passphrase choice, the **interpretation hold-to-confirm**,
  the **always-on fingerprint display**, the high-iteration-exponent warning, and the
  `backupWalletFlow` engrave dispatch. Own plan → R0 gate → implement → whole-diff review →
  merge. Written against D1's now-frozen `Combine` contract.

**Order is forced:** D1 before D2 (D2's sufficiency logic is written against `Combine`'s
preconditions). D1 merges to fork `main` between the two.

---

## 1. Goal & scope

On-device **SLIP-0039 secret recovery**: collect enough shares (group threshold + each
represented group's member threshold), reconstruct the master secret, engrave it as the
device-native BIP-39 seed plate. COMBINE direction only; no split/generation. The crypto is
a fresh in-tree Go port of `mnemonic_toolkit::slip39` (GF(256) Shamir + 4-round Feistel +
PBKDF2 — a different cryptosystem from codex32's GF(32)). GUI mirrors the Cycle-B codex32
recover *flow* (not its math).

### In scope
- D1 crypto: `slip39/gf256.go`, `slip39/lagrange.go`, `slip39/feistel.go`,
  `slip39/combine.go`; extend `slip39/share.go` to extract the share VALUE and accept **all
  valid SLIP-39 share lengths**.
- **All valid lengths, reachable on-device** (R0 I1 fold): master secret ∈ {16,20,24,28,32} B
  → share word counts {20,23,27,30,33}. D1 makes `ParseShare` accept all; **D2 widens the
  share-entry flow** (`inputSLIP39Flow`) so a non-20-word first share can actually be typed.
  This subsumes the bulk of FOLLOWUP `seedhammer-slip39-cycleC-all-lengths` (the single-share
  verbatim-engrave path inherits the same widened entry; mark that followup resolved-by-D2).
- **Full two-level recovery**: group threshold over groups; per-group member threshold.
- **Optional SLIP-39 passphrase** (EMS decryption), Slice-2 `PassphraseKeyboard`, default `""`.
- D2 GUI: Recover action + the two-level collection roster + the interpretation
  hold-to-confirm + fingerprint display + high-e warning + entropy→BIP-39 engrave via
  `backupWalletFlow`.

### Out of scope (explicit)
- SLIP-39 SPLIT/share generation in the firmware (no RNG, no Feistel-encrypt). The Rust split
  side is used ONLY as a test-fixture generator (§7), never ported into the firmware.
- SLIP-39 over NFC/RF — stays disabled (structurally: the scanner never yields a
  `slip39.Share`); shares are hand-typed, air-gapped.
- Trezor-native "master secret = direct BIP-32 seed" interpretation (see §3) — gated behind
  the §3 acknowledgement; a verbatim-hex engrave fallback for that case is a filed FOLLOWUP
  (`seedhammer-slip39-recovery-verbatim-hex`), not this cycle.
- An RP2350 hardware-SHA `machine` driver — filed FOLLOWUP
  (`seedhammer-slip39-hwsha`); v1 ships software PBKDF2 (§5.6).
- Constant-time GF(256)/PBKDF2 arithmetic — explicitly NOT done (theater for a single-user
  air-gapped device; the real hygiene item is secret scrubbing, §2/§4.8).

---

## 2. Security invariants (the R0 gate + crypto-security lens must verify each)

1. **Shares & the SLIP-39 passphrase are hand-typed, never over NFC/RF.** The SLIP-39 NFC
   path stays disabled (no `slip39.Share` is ever produced by the scanner).
2. **The recovered master secret & the SLIP-39 passphrase never leave the device, never go
   over NFC, are never persisted to flash/SD, and are never logged.** The only thing engraved
   is the seed plate the user explicitly confirms.
3. **SLIP-39 (EMS-decryption) passphrase ≠ BIP-39 25th-word passphrase.** Different stages,
   different algorithms, never conflated; prompts labeled by FUNCTION (§5.5).
4. **Two silent-wrong-seed channels exist and MUST be surfaced, not just avoided:**
   (a) the entropy-vs-direct-seed interpretation (§3) and (b) a wrong/skipped SLIP-39
   passphrase (SLIP-0039's deliberate plausible-deniability: a wrong passphrase yields a
   different *valid* secret with NO error). The UI MUST NOT claim to "verify" the passphrase;
   it MUST surface the recovered master fingerprint as a check-against-records (§5.4) and
   gate the engrave behind the §3 acknowledgement.
5. **Digest gate mandatory:** any threshold ≥ 2 layer must pass `HMAC-SHA256(R,S)[:4] ==
   digest` (compared with `crypto/subtle.ConstantTimeCompare`) or refuse. (T==1 layers have
   no digest — see §4.3 + §3.)
6. **No `math/big` in the SLIP-39 crypto.** GF(256) is byte-table-based. (`bip39.New` for the
   final entropy→words step uses the existing `math/big`-bearing BIP-39 package — carved out,
   not new.)
7. **Best-effort secret scrubbing (NEW — crypto-security lens; §4.8).** The firmware has
   *zero* zeroize discipline today, so invariants 2/4 are currently aspirational. D1 adds
   best-effort byte-wiping of the recovered secret, EMS, per-share value copies, and Feistel
   round-key buffers, with an honest comment that **TinyGo's GC may copy/retain** so this is
   defense-in-depth, not a guarantee. A test asserts the scrub hook fires (mirroring the
   Rust's pin-attempt-counter test pattern). `mlock` is NOT ported (no OS/swap on RP2350).
8. **TinyGo/RP2350 (`int` is 32-bit):** all multi-word bit assembly is byte-oriented (§4.6);
   no value-wide accumulator.

---

## 3. The "what to engrave" model (verified) + mandatory acknowledgement

**Verified (official SLIP-0039 vector #1):** `from_seed(MS)` (master secret used directly as
the BIP-32 seed) reproduces the vector's published `bip32_xprv` exactly. So in the *standard*
the master secret IS the BIP-32 seed; converting to BIP-39 words derives a *different* wallet
**unless** the backup was made under the "MS = BIP-39 entropy" convention. **The constellation
uses exactly that convention** (`mnemonic_toolkit`: `VALID_SECRET_LENGTHS=[16,20,24,28,32]`,
split from a BIP-39 phrase, `combine --to phrase`). This device is constellation tooling.

- **Decision:** interpret the recovered master secret as **BIP-39 entropy** and engrave the
  native BIP-39 seed plate (words + SeedQR) via `bip39.New(entropy)` → `backupWalletFlow`.
  Lengths map 16/20/24/28/32 B → 12/15/18/21/24 words.
- **Mandatory on-device acknowledgement (crypto-security lens, NOT just a doc note):** after a
  successful `Combine` and BEFORE `backupWalletFlow`, a **hold-to-confirm** screen (reuse the
  existing `ConfirmWarningScreen` hold pattern, `gui.go:2079`) states: *"Recovered as a BIP-39
  seed. Correct only for backups made from a BIP-39 phrase / this toolkit. If your shares came
  from a Trezor or other SLIP-39 wallet, this engraves the WRONG seed."* A doc note is
  invisible at the moment of irreversible engraving; this gate is required.

---

## 4. D1 — Go crypto port (combine direction)

Faithful port of the Rust (R0-verified faithful). Public functions are panic-free on
attacker-controlled input (§4.4).

### 4.1 `slip39/gf256.go` — GF(2⁸) Rijndael field
`reductionPoly=0x11b`, **generator 3**. Package-level `expTbl,logTbl [256]byte` built once in
`init()`: `x:=uint16(1); for i:=0;i<255;i++ { expTbl[i]=byte(x); logTbl[x]=byte(i);
x=(x<<1)^x; if x&0x100!=0 { x^=reductionPoly } }; expTbl[255]=1`. `gfAdd=^`; `gfMul` (0 if
either 0; else `expTbl[(logTbl[a]+logTbl[b]) ; -255 if ≥255]`); `gfInv` (prec a≠0,
`expTbl[(255-logTbl[a])%255]`); `gfDiv=gfMul(a,gfInv(b))` (prec b≠0). The `inv(0)`/`div(_,0)`
panics are unreachable in the combine path (denominators are `Π(xi⊕xj)` over **distinct**
x-coords, non-zero by §4.3 validation) — §4.4 guarantees no input reaches them.

### 4.2 `slip39/lagrange.go`
`interpolateAt(points []point, x byte) byte` (XOR-subtraction basis); `interpolateSecretAt(
points []bytePoint, x byte) []byte` (per-byte). `secretIndex=255`, `digestIndex=254`,
`digestLen=4`. Verified against `lagrange.rs`.

### 4.3 `slip39/combine.go`
```go
func Combine(shares []Share, passphrase []byte) ([]byte, error)
```
Port of `slip39_combine`/`recover_secret` (R0-verified step-for-step):
1. empty → `errEmptyShares`. 2. per-share value len ∈ {16,20,24,28,32} else
`errInvalidShareValueLength` (report input index). 3. cross-share consistency vs `shares[0]`
(identifier/extendable/iterationExp/groupThreshold/groupCount/value-len — distinct sentinels).
4. group by `GroupIndex` (sorted keys); per group: uniform memberThreshold, distinct member
indices, **exactly** memberThreshold shares; `recoverSecret(mt, memberPts)`. 5. **exactly**
groupThreshold groups. 6. `recoverSecret(groupThreshold, groupPts)` → EMS. 7. `feistelDecrypt(
ems, passphrase, iterationExp, identifier, extendable)` → master secret.
`recoverSecret`: T==1 → single value (NO digest); else `S=interp(255)`, `D=interp(254)`,
`digest=D[:4]`, `R=D[4:]`, refuse `errDigestVerificationFailed` unless
`subtle.ConstantTimeCompare(HMAC-SHA256(R,S)[:4], digest)==1`.

### 4.4 Panic-safety on attacker input (R0-critical, verified sufficient)
ALL preconditions violable by input are checked-and-returned BEFORE any interpolation/`gfDiv`
runs (steps 1–5), so x-coords are distinct and lengths equal by the time math executes. A test
MUST assert a malformed/duplicate-index set returns an error, never panics (the SLIP-39
analogue of Cycle-B's `ConsistentShares` precedent).

### 4.5 `slip39/feistel.go` — 4-round Feistel decrypt
```go
func feistelDecrypt(ems, passphrase []byte, iterationExp, identifier int, extendable bool) []byte
```
`L,R=ems[:half],ems[half:]`; `itersPerRound=(10000<<iterationExp)/4`; `saltPrefix`=empty if
extendable else `"shamir"||be16(identifier)`; rounds `i=3,2,1,0`: `F=pbkdf2.Key(password=
[]byte{byte(i)}||passphrase, salt=saltPrefix||R, itersPerRound, half, sha256.New)`; `L⊕=F`;
swap; return `R||L`. Verified verbatim against `feistel.rs`. The `"shamir_extendable"` string
is RS1024-ONLY (never the salt). Round-key buffer scrubbed after the pass (§4.8).

### 4.6 `slip39/share.go` — extend (all lengths + value + structural check)
- Accept share word counts **{20,23,27,30,33}** (drop the 20-only/33-reject gate +
  `errUnsupportedSize`). `valueWords=W-7`; `padBits=(10*valueWords)%16`, reject `>8`
  (`errBadPadding`); `valueBytes=(10*valueWords-padBits)/8` ∈ {16,20,24,28,32}. (R0-verified:
  all five counts satisfy padBits≤8; W=27 is the `==8` boundary.)
- **Value extraction (firmware-resource lens — TinyGo-critical):** unpack value words
  (indices 4..W-4) **bit-at-a-time MSB-first into a `[]byte`** (port the oracle's
  `get_bit`/per-byte packing). A 256-bit value does NOT fit a `uint64` — **no value-wide
  accumulator**; a per-byte (8-bit) accumulator only. Strip the `padBits` leading zero bits
  and verify they are 0 (`errBadPadding`).
- Add `Value []byte` to `Share` (the member share value). Existing fields/decode unchanged
  (the Cycle-C verbatim path ignores `Value`).
- **Structural group check (R0 I2 fold, Rust-faithful):** add `errGroupThresholdExceedsCount`
  — `ParseShare` rejects `GroupThreshold > GroupCount` (matches `share.rs:250`
  `GroupThresholdExceedsCount`). This is the sentinel for vector idx-9.
- `Describe` extended for `errBadPadding`, `errGroupThresholdExceedsCount`, and the §4.3
  combine sentinels. `errUnsupportedSize` removed (update `share_test.go:65-66,87` — minor M4).

### 4.7 `ConsistentShares` (incremental, two-level) for the GUI
```go
func ConsistentShares(shares []Share) error
```
Same identifier/extendable/iterationExp/groupThreshold/groupCount/value-len + no duplicate
`(GroupIndex, MemberIndex)`. Count-agnostic (validates partial sets). Returns the §4.3 step-3/4
sentinels + `errDuplicateMemberIndex`.

### 4.8 Best-effort secret scrubbing (NEW)
A small `wipe(b []byte)` helper (`for i := range b { b[i]=0 }`). `Combine`, `feistelDecrypt`,
and `recoverSecret` wipe their transient secret/EMS/round-key/share-value copies before
returning; `recoverSLIP39Flow` (D2) wipes the recovered-secret slice after `bip39.New`.
Comment honestly: **TinyGo GC may copy/retain — best-effort defense-in-depth, not a
guarantee.** A test asserts the wipe path executes (a test hook/counter). No `mlock`.

---

## 5. D2 — GUI recovery flow

All in `gui/slip39_polish.go` (the only GUI file changed besides `inputSLIP39Flow`'s widening;
**add `bip39` import for `bip39.Mnemonic`/`bip39.New` — minor M1**; `backupWalletFlow` needs no
import, same package). All `fmt`-using code stays here (no new `gui.go` imports).

### 5.1 Confirm action enum + Recover button
`type slip39ConfirmAction int; const ( slip39Back = iota; slip39Engrave; slip39Recover )`.
`confirmSLIP39Flow(ctx,th,s) slip39ConfirmAction`: Button1→Back; Button3(AltCenter)→Engrave;
**Button2→Recover, offered only when `s.MemberThreshold>1 || s.GroupThreshold>1`** (so the
Recover path always has a threshold≥2 layer → digest runs; the lone 1-of-1 share takes the
verbatim Engrave path). **Unconditional Button2 drain every frame** (the Cycle-B R0-C1
EventRouter footgun). Existing `TestConfirmSLIP39Render`/`…BackoutRecognized` stay green
(bool→enum: Back still maps to "recognized").

### 5.2 Widened share entry (R0 I1 fold)
`inputSLIP39Flow` gains a `title string` param AND accepts **variable share length**: the user
can enter 20/23/27/30/33-word shares (the word count is validated by `ParseShare`, not
hard-pinned to 20). Each collected share is sized to `len(first.Mnemonic)` (all shares in a
set are the same length). The menu `case 3:` first-share entry uses the widened flow.

### 5.3 `recoverSLIP39Flow` + the two-level collection ROSTER (design lens — redesigned)
```go
func recoverSLIP39Flow(ctx *Context, th *Colors, first slip39words.Share) (bip39.Mnemonic, bool)
```
The flat "share i of k" model is REPLACED by a **live group-satisfaction roster** (SLIP-39 has
no device-knowable single `k`):
1. From share 0: `Need <GT> of <groupCount> groups`. Maintain a roster: per group SEEN, a line
   `Group <GI>: <m>/<memberThreshold> [✓ when m==mt]`; a header `<satisfied>/<GT> groups`.
2. Per newly entered share: `ParseShare`; eager `ConsistentShares(append(shares,cand))` →
   `showError(…, Describe(err))` + re-prompt on mismatch/duplicate. Show a per-share readout
   ("Group GI, member I of t") so the user can sort a physical pile. Back removes the last
   share / aborts at the first prompt → `(nil, false)` (minor M2: `nil`, not `""`).
3. **Stop precisely at sufficiency** (`satisfied==GT`, each satisfied group at exactly its
   memberThreshold): stop prompting and offer **Continue** (do not require the user to guess
   they are done; do not over-collect — `Combine`'s exact-count rule would otherwise error).
   Provide a **start-over** affordance for the dead-end case (full member thresholds for only
   GT−1 groups, never completable with the held shares).
4. Optional **SLIP-39 passphrase** (§5.5), then `secret, err := slip39words.Combine(shares,
   passphrase)`. On error → `showError(…, Describe(err))` + retry/abort. On success:
   `m := bip39.New(secret)`; `wipe(secret)`; return `(m, true)`.

### 5.4 Always-on fingerprint display (crypto + design lenses)
After recovery, BEFORE engrave, ALWAYS show the recovered seed's BIP-39 master fingerprint
(`masterFingerprintFor(m, net, "")`, already computed downstream) labeled *"Fingerprint
XXXXXXXX — confirm this matches your wallet records before engraving."* This is the only
on-device handle to catch both silent-wrong-seed channels (§2.4); framed as
check-against-records, NOT a verification claim.

### 5.5 Passphrase prompts — safe default, function-labeled
The SLIP-39 passphrase choice defaults to **Skip** (index 0) with: *"SLIP-39 passphrase? Most
backups have none. A wrong passphrase silently recovers a different seed."* If
`backupWalletFlow` later offers a BIP-39 passphrase, it is labeled by function (*"BIP-39
wallet passphrase (25th word) — optional, separate from the SLIP-39 share passphrase"*). Fresh
`NewPassphraseKeyboard` per prompt (no state bleed). A test exercises both-passphrases-set and
asserts each string reaches only its own algorithm.

### 5.6 PBKDF2 responsiveness (firmware-resource lens)
The Feistel decrypt runs `10000·2^e` software SHA-256 blocks: e=0/1 ≈ 0.5–1.9 s (fine), but
high e is minutes-to-hours (e=15 ≈ 5–8.5 h; the operator does not choose e — it's read from
the share header). Therefore:
- Run the decrypt **off the UI thread** with a **"Recovering…"** progress indicator and the
  watchdog fed (`-scheduler tasks`).
- **Warn-and-confirm on high e** (NOT a hard cap — capping breaks recoverability of real
  high-e backups): when `IterationExp` implies a long wait, show the estimated time and
  require explicit confirm (e.g. ≥~10 s at e≥4, hours at e=15).
- v1 uses software SHA-256; the RP2350 hardware-SHA driver is a filed FOLLOWUP
  (`seedhammer-slip39-hwsha`).

### 5.7 Engrave dispatch
`engraveSLIP39` loops over `confirmSLIP39Flow`: `slip39Back`→return `true`;
`slip39Engrave`→verbatim single-share engrave (existing path); `slip39Recover`→
`m, ok := recoverSLIP39Flow(...)`; `!ok`→continue; ok→ the §3 hold-to-confirm + §5.4
fingerprint display, then `backupWalletFlow(ctx, th, m)` (reuses Slice-3 confirm → optional
BIP-39 passphrase → fingerprint choice → SeedQR+words engrave), then return `true`. Do NOT
make `backupWalletFlow` "recovery-aware" — keep it generic; disambiguate via labels +
the §3/§5.4 brackets (design lens).

---

## 6. Error taxonomy (`slip39.Describe`)
Existing: `bad checksum`, `unknown word`, `wrong length`. New: `bad padding`,
`group threshold exceeds count` (idx-9), `id mismatch`, `extendable mismatch`, `iteration
mismatch`, `group threshold mismatch`, `group count mismatch`, `value length mismatch`,
`member threshold mismatch`, `duplicate share`, `not enough shares`, `bad share set` (digest).
Removed: `256-bit not supported`. Unknown → `invalid`.

---

## 7. TDD (D1 crypto first; Rust oracle, incl. the SPLIT side as a fixture generator)

**Oracle strategy (design lens):** the static official corpus (45 entries) is thin on the
23/27/30-word intermediate lengths and combine-only Go cannot generate fresh sets. So a
checked-in, reproducible **Rust harness drives `slip39_split`** over each of the five
master-secret lengths × a couple of group topologies, emitting share sets + expected master
secrets as committed Go fixtures → **round-trip oracle**: Rust splits → Go `Combine` → must
equal the Rust input. Don't port split into the firmware (it's fixtures-only).

- **Combine units:** official idx 0 (1-of-1, T==1 no-digest), idx 3 (2-of-3/128) →
  `b43c…0864`, idx 17 (group-threshold/128) → `7c33…8d11`, idx 35 (256-bit/33-word
  multi-group), idx 42 (extendable/ext=1). Passphrase pair on idx 3: `"TREZOR"` vs `""` →
  distinct secrets. Plus generated fixtures covering **20/23/27/30/33-word** round-trips
  (the 33-word case explicitly pins the byte-oriented unpack invariant, §4.6).
- **share.go value extraction:** each positive vector + fixture → assert value byte length ∈
  {16,20,24,28,32} and round-trip.
- **Negatives (correct sentinel):** idx 1 (bad checksum @ParseShare), idx 4 (insufficient
  members), idx 13 (insufficient groups), idx 5 (id mismatch), **idx 9 (group threshold
  exceeds count → `errGroupThresholdExceedsCount` @ParseShare — R0 I2)**, **idx 12 (invalid
  digest — the critical gate)**, bad-padding. **Panic-safety: a malformed/duplicate-index set
  returns an error, never panics (§4.4 — mandatory, not covered by the oracle since the Rust
  asserts).**
- **Scrubbing:** a test asserts the wipe hook fires (§4.8).
- **D2 GUI:** `TestConfirmSLIP39MultiOffersRecover`/`…LoneNoRecover`; `TestRecoverSLIP39`
  (drive shares via `ctx.Router`, assert the recovered mnemonic + roster sufficiency);
  `TestRecoverSLIP39Mismatch`; `TestRecoverSLIP39BackoutRecognized`; the Button2-drain no-hang
  regression; the §5.5 both-passphrases test; a widened-length (33-word) entry test. **Note:
  host `go test` SHA-256 is amd64-asm (~50× faster) — it validates correctness, NOT on-device
  timing; the §5.6 e-cost UX must be validated against the cycle model, not host speed.**

Host: `/home/bcg/.local/go/bin/go test ./slip39/ ./gui/ ./bip39/`. Existing guards stay green
(codex32, BIP-39, backup goldens; `TestConfirmSLIP39Render`, `TestEngraveSLIP39BackoutRecognized`).

---

## 8. File manifest

| File | Phase | Change |
|---|---|---|
| `slip39/gf256.go` | D1 | new — GF(256) (port of `gf256.rs`). |
| `slip39/lagrange.go` | D1 | new — interpolation (port of `lagrange.rs`). |
| `slip39/feistel.go` | D1 | new — Feistel decrypt + round-key scrub (port of `feistel.rs`). |
| `slip39/combine.go` | D1 | new — `Combine`+`recoverSecret`+`ConsistentShares`+`wipe` (port of `slip39_combine`/`recover_secret`). |
| `slip39/share.go` | D1 | modify — accept {20,23,27,30,33} words; byte-oriented `Value []byte` extraction + padding validation; `errBadPadding`+`errGroupThresholdExceedsCount`; extend `Describe`; drop `errUnsupportedSize`. |
| `slip39/*_test.go` | D1 | new/modify — vectors + Rust-fixture round-trips + value + negatives + panic-safety + scrub; invert the old 33-word `errUnsupportedSize` assertions (M4). |
| `slip39_fixtures/` (Rust harness + committed fixtures) | D1 | new — reproducible split-side fixture generator (test-only). |
| `gui/slip39_polish.go` | D2 | modify — `slip39ConfirmAction`+Recover(Button2 drain), `recoverSLIP39Flow`+roster, §3 hold-to-confirm, §5.4 fingerprint, §5.5 passphrase, §5.6 progress/warn, `backupWalletFlow` dispatch; add `bip39` import. |
| `gui/slip39_polish_test.go` | D2 | modify — recover-flow + roster + passphrase + widened-length tests. |
| `gui/gui.go` (or wherever `inputSLIP39Flow` lives) | D2 | modify — `title` param + variable share length; no new imports. |

Unchanged (reused, must stay green): `codex32/`, `mdmk.go`, `backup/`, `bip32/`, `seedqr/`,
`gui/passphrase_keyboard.go`, the `gui.go` BIP-39/Slice-3 logic.

---

## 9. Process

Two phases, each the full gated pipeline. **D1:** plan → opus R0 loop to 0C/0I → single-
implementer TDD (port + Rust-fixture oracle) in worktree `seedhammer-wt-slip39-d1` (branch
`feat/slip39-recovery-crypto` off `20fa4c4`) → whole-diff adversarial execution review → fold
→ merge no-ff signed into fork `main` → push `bg002h`. **D2:** branch off the post-D1 `main`
→ plan → R0 loop → single-implementer TDD (GUI) → whole-diff review → fold → merge → push.
Reviews persist verbatim to `design/agent-reports/seedhammer-slip39-recovery-*`. Commits
SSH-signed + DCO, author Brian Goss. SemVer: firmware (version via ldflags). No upstream PR.
FOLLOWUPS to file: `seedhammer-slip39-recovery-verbatim-hex` (Trezor-native MS-as-seed engrave),
`seedhammer-slip39-hwsha` (RP2350 hardware-SHA driver); mark `seedhammer-slip39-cycleC-all-lengths`
resolved-by-D2.
