# cycle-prep recon — 2026-06-18 — slip39-secret-recovery (Cycle D)

**Fork `main` SHA at recon time:** `20fa4c4` (Slice 3 merged)
**Design repo (mnemonic-engrave) HEAD:** `1f80dd7`
**Scope:** SLIP-0039 secret RECOVERY on the SeedHammer fork — collect enough shares
(group threshold + per-group member thresholds), reconstruct the master secret,
engrave it. Fork-side only (no upstream PR per the standing strategy).

Recon = four parallel agents (in-tree state, SLIP-0039 spec vs authoritative source,
go-slip39 survey, official test-vectors pre-verification) + a constellation-codebase
investigation (ms-codec / mnemonic-toolkit) + a hand-verification of the SLIP-39 ↔
BIP-32 relationship against the official vector. All protocol facts verified against
authoritative source text (SLIP-0039 spec + Trezor `python-shamir-mnemonic` + official
`vectors.json`), per the ultracode external-protocol-fact rule.

---

## 1. Decisive finding — crypto sourcing: PORT OUR OWN RUST (in-tree Go)

- **`ms-codec`/`ms-cli` (the constellation `ms1` format) is BIP-93 codex32 — GF(32), NOT
  SLIP-39.** It is the *same* family the firmware already supports natively and that we
  shipped in **Cycles A (input polish) + B (multi-share recovery)**. It does NOT contain
  SLIP-39 crypto and does not port to Cycle D. (`crates/ms-codec/src/{bch,bch_decode,
  shares}.rs` — `Gf32`, `GF32_REDUCE=0b0_1001`, codex32 alphabet, `codex32::interpolate_at`.)
- **The constellation's SLIP-39 reference IS `mnemonic_toolkit::slip39`** — a complete,
  from-scratch, dependency-free Rust implementation that already passed the constellation's
  gated review (`SPEC_slip39_v0_13_0.md`). This is the Go-port source:
  - `slip39/gf256.rs` (88 LoC) — GF(2⁸) Rijndael field, `REDUCTION_POLY=0x11b`, exp/log
    tables, **generator 3** (not 2). No `math/big`.
  - `slip39/lagrange.rs` (91) — interpolation, `SECRET_INDEX=255`, `DIGEST_INDEX=254`.
  - `slip39/feistel.rs` (212) — 4-round Feistel + PBKDF2-HMAC-SHA256 round function.
  - `slip39/rs1024.rs` (104) — RS1024 (firmware already has an equivalent from Cycle C).
  - `slip39/share.rs` (394) — share parse/encode + bit-packing + padding.
  - `slip39/mod.rs` (696) — two-level group/member split+combine orchestration.
  - `slip39/{wordlist,error}.rs`. CLI driver `cmd/slip39.rs` (919).
- **Independent go-slip39 survey CONCURS: do not vendor, implement in-tree.** The only real
  Go lib (`gavincarr/go-slip39`) pulls `math/big` (gratuitous — used only for word-index
  packing, TinyGo-hostile, issue #890), gonum, golang-set, x/exp; its GitHub repo now 404s;
  and **upstream SeedHammer already disabled SLIP-39 specifically over its footprint**
  (`gui/scan.go`: "go-slip39 adds ~55kb of RAM use in the unicode"). In-tree estimate
  ~600–900 LoC, anchored by the fork's own codex32 GF(32) precedent (1122 LoC).
- **Therefore: port `mnemonic_toolkit::slip39` → Go in-tree.** Highest provenance, byte-
  compatible with the toolkit, no new deps (`crypto/sha256`+`crypto/hmac`+`x/crypto/pbkdf2`,
  all TinyGo-importable; firmware already ships SHA-256-class crypto on TinyGo). The Rust is
  the **TDD oracle** (run it on the official vectors, diff the Go).

---

## 2. Verified SLIP-0039 recovery algorithm (authoritative)

Spec: `satoshilabs/slips/slip-0039.md`; reference `trezor/python-shamir-mnemonic` (path
`shamir_mnemonic/`, not `src/…`). All six areas verified, spec ≡ reference (one cosmetic
diff: `2500<<e` ≡ `(10000<<e)//4`).

- **Header (40 bits):** id(15) | ext(1) | e(4) | GI(4) | Gt=GT−1(4) | g=G−1(4) | I(4) |
  t=T−1(4), then padded share value (8n + left-zero-pad to a 10-bit multiple) + RS1024
  C(30). Thresholds/counts stored value−1. **Iteration exponent is 4 bits**, with `ext` the
  separate 1-bit flag (historically the 5th exponent bit). The firmware's Cycle-C
  `slip39/share.go` already decodes ALL header fields into a `uint64` (RP2350 int is 32-bit)
  — but does NOT extract the share-value payload (the gap to fill).
- **Two-level combine:** per group reaching its member threshold T_i, `s_i =
  RecoverSecret(T_i, [(member_index, value)…])`; then `EMS = RecoverSecret(GT,
  [(group_index, s_i)…])`; then `MS = Decrypt(EMS, P, e, id, ext)`. x = member index
  (inner) / group index (outer). Preconditions: same id/ext/e/GT/G; distinct group indices
  count == GT; per group, same T_i, distinct member indices, count == T_i.
- **RecoverSecret over GF(256)** (Rijndael `0x11B`, **generator 3**): if T==1 the single
  value IS the secret (no digest). Else `S = Interpolate(255, pts)`, `D = Interpolate(254,
  pts)`; `digest=D[:4]`, `R=D[4:]`; **abort unless `HMAC-SHA256(key=R, msg=S)[:4]==digest`**.
- **Feistel DECRYPT (4 rounds, reversed index):** `L,R = EMS[:n/2], EMS[n/2:]`; `for i in
  [3,2,1,0]: L,R = R, L XOR F(i,R)`; **`MS = R || L`**. `F(i,R) = PBKDF2(HMAC-SHA256,
  password = byte(i)||passphrase, salt = salt_prefix||R, iters = 2500·2^e, dkLen = n/2)`.
- **The two "shamir" strings (the silent-wrong-secret trap):** RS1024 customization string
  = `"shamir"` (ext=0) / `"shamir_extendable"` (ext=1) — for *checksum only*. PBKDF2 salt
  prefix = `"shamir"||id_be16` (ext=0) / **empty** (ext=1). Do NOT salt with
  `"shamir_extendable"`. Parse `ext` first (it's in the first 2 words) before picking the cs.
- **Padding:** reject if pad bits > 8 or any pad bit is 1.
- **Passphrase:** enters ONLY the Feistel round function; default empty `b""`; a wrong
  passphrase yields a different-but-valid MS with no error (by design).

---

## 3. Official test vectors (pre-verified from scratch)

`trezor/python-shamir-mnemonic/vectors.json` — 45 entries, tuple `[desc, [mnemonics],
master_secret_hex (""=invalid), bip32_xprv]`. **All VALID vectors use passphrase
`b"TREZOR"`**, not empty (empty → a different secret). A from-scratch recon impl reproduced
**15/15 VALID** and rejected the negatives for the correct distinct reasons; it caught the
two classic bugs (generator must be **3**; Feistel decrypt returns **`r||l`** with reversed
rounds) — both of which our Rust port already gets right.

Recommended TDD set: positives idx 3 (basic 2-of-3, 128) → `b43c…0864`, idx 17 (group-thr
2-of-4, 128) → `7c33…8d11`, idx 42 (extendable, ext=1), idx 35 (256-bit multi-group);
passphrase pair on idx 3 (`TREZOR` vs `""`); negatives idx 1 (bad checksum), 4 (insufficient
members), 13 (insufficient groups), 5 (mismatched id), 9 (group-thr>count), **12 (invalid
digest — the critical one)**.

---

## 4. The "what to engrave" decision — VERIFIED + DECIDED

**Verified against official vector #1:** `from_seed(MS)` (master secret used **directly as
the BIP-32 seed**) reproduces the vector's published `bip32_xprv` exactly. So in the SLIP-39
standard the recovered MS is the **BIP-32 seed**, not BIP-39 entropy, and is never run
through BIP-39's PBKDF2. Converting MS → BIP-39 words and loading into a BIP-39 wallet would
derive a *different* wallet — silent corruption — **unless** the backup was made under the
"MS = BIP-39 entropy" convention.

**Constellation convention (governing for this device):** `mnemonic_toolkit` aligns the
SLIP-39 master secret to BIP-39 entropy sizes (`VALID_SECRET_LENGTHS=[16,20,24,28,32]`),
splits from a BIP-39 phrase (`slip39 split --from phrase=…`), and round-trips via `combine
--to phrase`. So a constellation-origin SLIP-39 backup recovers to BIP-39 entropy.

**Decision (user-confirmed 2026-06-18):** the device interprets the recovered MS as BIP-39
entropy (constellation model) and engraves the **native BIP-39 seed plate (words + SeedQR)**
via the device's existing seed confirm/engrave path (reusing the Slice-3 work). The spec
MUST document the entropy-convention assumption prominently (Trezor-native MS-as-direct-seed
is out of the supported model). The **SLIP-39 (EMS-decryption) passphrase** used during
recovery is kept cleanly separate from any **BIP-39 25th-word passphrase** offered by the
engrave flow — two distinct, independently-optional secrets at two distinct stages.

---

## 5. GUI integration point + the codex32-recovery shape to mirror

- **Entry:** menu `case 3:` → `inputSLIP39Flow` → `slip39words.ParseShare` → returns
  `slip39words.Share` (`gui/gui.go:2034-2051`); engrave dispatch `case slip39words.Share:`
  → `engraveSLIP39` (`gui/gui.go:1851`); flow in `gui/slip39_polish.go` (`confirmSLIP39Flow`
  Back/Engrave-only today, `engraveSLIP39`, `showError`).
- **Mirror the Cycle-B codex32 recover shape** (`gui/codex32_polish.go`): a confirm-action
  enum {Back/Engrave/Recover} with the **mandatory unconditional Button2-drain** (the R0-C1
  queue-head-blocking footgun); a `recover…Flow(first)` that reads k, loops `inputCodex32Flow
  (…, "Share i of k")`, eagerly `ConsistentShares`-validates + ErrorScreen on mismatch; an
  engrave loop that re-confirms the recovered secret. **SLIP-39 is harder: two-level**
  collection (group + member), so the "i of k" prompt and a `ConsistentShares` analogue must
  account for group/member structure (same id/ext/e, per-group T_i, distinct (GI,I) pairs,
  GT groups). NFC stays disabled (sensitive shares hand-typed, air-gapped).

---

## 6. Scope (user-confirmed) + lengths + TinyGo watch-items

- **All valid lengths:** MS ∈ {16,20,24,28,32} B → 20-/33-word (and intermediate) shares;
  full two-level group/member recovery (free from the port).
- **Cycle C followup FILED** (see `FOLLOWUPS.md`): widen the single-share entry+engrave
  (Cycle C, currently 20-word/128-bit only, rejects 33-word) to all valid lengths.
- **Optional SLIP-39 passphrase** via the Slice-2 `PassphraseKeyboard`; default `""`.
- **TinyGo / RP2350:** keep all bit-packing byte-oriented / `uint64` (int is 32-bit, the
  Cycle-C header already does this). GF(256) is byte-table-based (no `math/big`). PBKDF2-
  HMAC-SHA256 via stdlib + `x/crypto/pbkdf2` (TinyGo-OK). The recovered-entropy → BIP-39
  words step uses `bip39.New` (`bip39/bip39.go`), which uses `math/big` — already in-firmware.

---

## 7. Recommended cycle plan

Single XL cycle, full gated pipeline: spec (this recon → single author) → opus R0 loop to
0C/0I → implementation plan → plan R0 loop to 0C/0I → single-implementer TDD in a worktree
(port `mnemonic_toolkit::slip39` to Go: `slip39/{gf256,lagrange,feistel,combine}.go` +
extend `share.go` payload extraction + the two-level recover + GUI recover flow + entropy→
BIP-39 engrave), with the Rust as oracle → mandatory whole-diff adversarial execution review
→ fold → merge no-ff signed into fork `main` → push to `bg002h`. Reviews persisted verbatim
to `design/agent-reports/seedhammer-slip39-recovery-*`. SemVer: firmware (no committed
version constant; injected via ldflags). No upstream PR.
