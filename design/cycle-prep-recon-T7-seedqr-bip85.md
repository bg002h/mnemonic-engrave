# cycle-prep recon — 2026-06-19 — T7 niceties (T7a SeedQR · T7b BIP-85)

**Fork HEAD at recon time:** `82d46b3` (Merge T6b: multisig/miniscript bundle via supplied md1)
**Fork branch:** `main`, sync: `## main...origin/main` (in sync, clean tree).
**Design repo:** `mnemonic-engrave`.
**Protocol facts re-verified against authoritative source this session** (per the external-fact-verification policy): BIP-85 spec (bitcoin/bips bip-0085.mediawiki) for the derivation path, application codes, entropy truncation, and HMAC; the in-tree `bip85_test.go` vector cross-checked against the spec's canonical test vector. **Recon ONLY — no spec, no implementation.**

---

## HEADLINE verdicts

- **T7a (SeedQR engrave) is effectively DONE and user-reachable.** A standard SeedQR is already engraved onto the seed backup plate by the shipped `backupWallet` (program 0) flow, via `engraveSeed` → `backup.EngraveSeed`. No new wiring required; the only "work" is documentation. (One secret-handling caveat: the *NFC-scanned-mnemonic* path also reaches it — pre-existing upstream footgun, not introduced here.)
- **T7b (BIP-85 on-device engrave) needs net-new wiring**, but the crypto primitive ships and is spec-correct. Recommended engrave target: **raw BIP-39 child words engraved as a full seed-backup plate** (reuse `engraveSeed`, the exact same path `backupWallet` uses). This is the most faithful BIP-85 output (BIP-39 application produces a mnemonic) and the least net-new code. Sizing **S–M (~350–500 LOC incl. tests)**.

---

# TASK A — SeedQR (T7a): is on-device SeedQR-engrave ALREADY user-reachable?

## A1 — What `seedqr/seedqr.go` provides (`seedqr/seedqr.go:1-88`)

It is an **encode/decode codec between a `bip39.Mnemonic` and the SeedQR *payload*** — NOT a QR-matrix renderer. Exported API:

- `Parse(qr []byte) (bip39.Mnemonic, bool)` — auto-detects standard vs compact, decodes to a mnemonic (`:15-20`).
- `QR(m bip39.Mnemonic) []byte` — **standard SeedQR**: emits the 4-digit-per-word decimal string, e.g. `"0115132511540127..."` (`:24-33`, `fmt.Fprintf(&qr,"%04d",w)`). Panics on invalid mnemonic.
- `CompactQR(m bip39.Mnemonic) []byte` — **CompactSeedQR**: emits the raw entropy bytes (`:37-42`, returns `m.Entropy()`). Panics on invalid mnemonic.
- unexported `parseSeedQR` / `parseCompactSeedQR` (`:44-88`).

The doc comment cites the SeedSigner SeedQR spec. The QR *matrix* is produced separately by `qr.Encode(string(seedqr.QR(m)), qr.M)` at the call site — `seedqr` only produces the textual payload that goes into the QR encoder.

## A2 — How `seedqr` is used in `gui/gui.go` (the ONLY non-test GUI use)

Single call site: **`gui/gui.go:462`** inside `engraveSeed`:
```go
func engraveSeed(params engrave.Params, m bip39.Mnemonic, mfp uint32) (Plate, error) {
    qrc, err := qr.Encode(string(seedqr.QR(m)), qr.M)   // standard SeedQR → QR matrix
    ...
    seedDesc := backup.Seed{ Mnemonic: words, QR: qrc, MasterFingerprint: mfp, ... }
    seedSide, err := backup.EngraveSeed(params, seedDesc)   // engraves words + QR matrix onto steel
```
`backup.EngraveSeed` (`backup/backup.go:62-66`) engraves the QR matrix when non-nil: `if plate.QR != nil { qrc, err = engrave.ConstantQR(plate.QR) }`. So the **standard SeedQR is engraved onto the seed plate alongside the words** — this is the genuine on-steel engrave, not a screen render.

## A3 — Reachability from a user-hit menu (USER-FACING flow EXISTS)

`engraveSeed` is called from **`backupWalletFlow` (`gui/gui.go:2050`)**, which is the **`backupWallet` program (enum 0, `gui.go:148`)** — the first, default-selected entry of the program carousel. Two reach-paths:

1. **TYPED (intended):** carousel `backupWallet` → `newInputFlow` (`gui.go:1509`, typed 12/24-word / M*1 / SLIP-39 / SEED XOR entry) → `engraveObjectFlow` (`gui.go:1516`) → `case bip39.Mnemonic: backupWalletFlow` (`gui.go:1896`) → `engraveSeed`. **This is the user-facing SeedQR engrave.** ✅
2. **NFC-SCAN (footgun, pre-existing):** `StartScreen.Flow` returns `startScreenAction{scan: cnt}` (`gui.go:1622`) when an NFC tag is read; `scanner.Scan` (`gui/scan.go:61-62`) returns a `bip39.Mnemonic` for a tag carrying a BIP-39 phrase → same `engraveObjectFlow` → `backupWalletFlow`.

**Verdict: T7a is DONE.** The SeedQR-of-seed engrave lives in `backupWalletFlow`/`engraveSeed` and is reached by selecting **Backup Wallet** (program 0) and typing a seed. Standard SeedQR (digit form) is used; CompactSeedQR is **not** wired into any GUI path (only `engrave_test.go:37` exercises `CompactQR`).

## A4 — Secret handling note (a SeedQR encodes the SEED → steel-only)

The seed/SeedQR is engraved onto **owner-held steel only** (`backup.EngraveSeed` → engrave plan → `NewEngraveScreen(...).Engrave`, `gui.go:2055`); it is never emitted over NFC. The typed path (A3-1) honors steel-only correctly.

**CAVEAT (pre-existing, not introduced by T7):** the NFC path (A3-2) accepts a *mnemonic read off an NFC tag* and engraves its SeedQR. This is the same `scan.go` bip39 footgun the T-series flagged for the secret-derivation flows; here it only re-engraves a secret the user already put on a tag (no derivation), but it is worth a one-line doc note. **No T7a code is needed**; if the team wants to harden it, that is a separate (optional) footgun-closure item, not T7a scope.

---

# TASK B — BIP-85 (T7b): scope wiring the lib into an on-device engrave flow

## B1 — `bip85/bip85.go` exported API — it is a THIN PRIMITIVE, not a derivation engine (`bip85/bip85.go:1-23`)

```go
const PathRoot = 83696968 + 0x80000000   // = m/83696968' (hardened)
const macKey = "bip-entropy-from-k"
func Entropy(privkey []byte) []byte       // HMAC-SHA512("bip-entropy-from-k", privkey), 64-byte output; panics if len(privkey)!=32
```

**Critical scoping fact:** the package implements ONLY the final HMAC-SHA512 entropy-extraction step. It does **NOT** do the BIP-32 path walk, the application-code path validation, the entropy truncation, or the word-count→mnemonic mapping. All of that lives in the **biptool driver**, and a GUI flow must re-create it (using the same in-tree primitives biptool uses). There are no BIP-85 application codes in the package — only `PathRoot`.

## B2 — How `cmd/biptool/main.go` drives BIP-85 (reference flow) (`cmd/biptool/main.go:99-245`)

`derive` reads an xprv/codex32/seed from stdin → walks the supplied `-path` hardened-only (`:137-167`):
```
xkey = NewMaster(seed)
for each path element p (must be ≥ HardenedKeyStart): xkey = xkey.Derive(p)
pkey  = xkey.ECPrivKey().Serialize()      // 32-byte privkey at the derived node
seed  = bip85.Entropy(pkey)               // 64-byte entropy   (:157)
```
Then per output format:
- **`bip39`** (`:174-190`): validates path == `m/83696968'/39'/0'/{words}'/{index}'` (`path[1]==39+h, path[2]==0+h, path[3]==words+h`, 5 elements, `:183`); computes `entLen := (n*11 - n/3)/8` (`:188`); `m := bip39.New(seed[:entLen])` (`:189`) — the **leading** entLen bytes of the 64-byte HMAC output.
- **`xprv`/`xpub`/`pubkey`/`privkey`** (`:206-240`): require path `m/83696968'/32'/{index}'` (`isXprvDeriv`, `:170`); rebuild an xprv from `chaincode=seed[:32], key=seed[32:64]` (`:163-166`).
- **`seed`** (`:191-205`): raw entropy bytes.

word-count guard: `n<12 || 24<n || n%3!=0` (`:179`). 12→16 B, 18→24 B, 24→32 B.

## B3 — PROTOCOL VERIFICATION (vs canonical BIP-85 spec — all ACCURATE)

Cross-checked bitcoin/bips `bip-0085.mediawiki`:

| Fact | Spec | In-tree | Match |
|---|---|---|---|
| Root | `m/83696968'` | `PathRoot = 83696968 + 0x80000000` (`bip85.go:11`) | ✅ |
| BIP-39 app path | `m/83696968'/39'/{lang}'/{words}'/{index}'` | biptool `:183` (lang=0=English) | ✅ |
| English code | `0'` | `path[2]==0+h` (`:183`) | ✅ |
| Entropy/word | 12→128b, 18→192b, 24→256b | `entLen=(n*11-n/3)/8` → 16/24/32 B (`:188`) | ✅ |
| Truncation | "truncate **trailing** (LSB) bytes" = keep leading | `seed[:entLen]` (`:189`) | ✅ |
| HMAC | key=`"bip-entropy-from-k"`, msg = 32-byte privkey k | `bip85.go:13,20-22` | ✅ |
| XPRV app | `m/83696968'/32'/{index}'`; chaincode=first 32B, key=next 32B | biptool `:163-166,170` | ✅ |
| Test vector | spec privkey `cca20ccb…` → entropy `efecfbcc…` | `bip85_test.go:14-16` | ✅ identical |

**No divergence from canonical BIP-85.** The one nuance a GUI flow MUST preserve: derivation is **fully hardened** (every element ≥ `hdkeychain.HardenedKeyStart`), and the entropy is the **leading** `entLen` bytes of the HMAC output (`seed[:entLen]`, not trailing).

## B4 — Engrave-target options (KEY QUESTION) and recommendation

What do we engrave the BIP-85 child *as*? Three options, against SHIPPED primitives:

- **(a) raw BIP-39 child words → seed-backup plate.** Path: child entropy → `bip39.New(entropy)` (`bip39/bip39.go:228`; already used at `gui/ms1_decode.go:33`, `slip39_polish.go:292`) → reuse **`engraveSeed`** (`gui.go:461`) exactly as `backupWallet` does → words + SeedQR engraved. **RECOMMENDED.** Most faithful (BIP-85's `39'` application IS a child BIP-39 mnemonic), least net-new (the words→engrave path is `engraveSeed`, already shipped and the very thing T7a exercises). Bonus: the child plate gets a SeedQR for free.
- **(b) child entropy as ms1** via `codex32.EncodeMS1(entropy)` (`codex32/msencode.go:17`, the T6a path). Feasible and minimal, but semantically lossy — it re-encodes the BIP-85 mnemonic into the m-format instead of presenting the canonical child mnemonic. Use only if the cycle explicitly wants an m-format child.
- **(c) full single-sig bundle** by running the child seed through `deriveSingleSigBundle`/`engraveSingleSigFlow` (`gui/singlesig.go`, `singlesig_derive.go`). Most code, conflates two features (BIP-85 + single-sig derivation); over-scoped for a "nicety."

**Recommendation: (a).** It mirrors the T4/T6a typed-seed→deterministic-derive→engrave spine and reuses `engraveSeed` verbatim.

## B5 — Secret-handling spine (deterministic; steel-only; defer-scrub master AND child)

- **Master seed:** typed-only via `seedEntryFlow` (`gui/derive_xpub.go:82`) — NEVER an NFC scan. `defer` zero the master `mnemonic []Word` on every exit (the exact `engraveSingleSigFlow` pattern, `singlesig.go:41-45`).
- **Path walk + privkey:** reuse the `deriveAccountXpub` scrub discipline (`gui/derive.go:19-58`): `wipeBytes(seed)` deferred, `.Zero()` each intermediate `ExtendedKey`, capture before zeroing. The 32-byte privkey serialization and the 64-byte HMAC output are SECRET — `wipeBytes` them after `bip39.New`.
- **Child mnemonic:** SECRET — engraved onto owner-held steel only (via `engraveSeed`), never NFC. The child `bip39.Mnemonic []Word` MUST also be `defer`-scrubbed (option (a) holds a second secret mnemonic — scrub master AND child).
- **No CSPRNG:** BIP-85 is a pure deterministic transform of master seed + (app,words,index) — fully on-device-feasible, no entropy source needed. ✅

## B6 — Lockstep sites for a new `deriveChild` (BIP-85) program (cited at HEAD `82d46b3`)

A new program (suggest enum name `bip85Derive`, inserted **after `engraveMultisig`, before `qaProgram`** so `qaProgram` stays the last non-navigable sentinel) touches the **8 lockstep sites**:

1. **enum const** — `gui/gui.go:148-154` (add between `engraveMultisig` (152) and `qaProgram` (153)).
2. **dispatch switch** — `gui/gui.go:1492-1514` (add `case bip85Derive: bip85DeriveFlow(ctx, th); continue`).
3. **carousel Right wrap bound** — `gui/gui.go:1648-1651` (`if m.prog > engraveMultisig { m.prog = 0 }` → bump to the new program).
4. **carousel Left wrap bound** — `gui/gui.go:1640-1643` (`if m.prog < 0 { m.prog = engraveMultisig }` → bump to the new program).
5. **title switch** — `gui/gui.go:1667-1678` (add `case bip85Derive: titleTxt = "..."`). MUST be non-blank (nav-test asserts).
6. **`npage`** — `gui/gui.go:1852` (`const npage = int(engraveMultisig) + 1` → new program).
7. **`layoutMainPlates` case** — `gui/gui.go:1860-1867` (add the new program to the `case` at `:1862`; the `panic("invalid page")` default at `:1867` is MANDATORY-or-panic).
8. **`npages`** — `gui/gui.go:1871` (`const npages = int(engraveMultisig) + 1` → new program).

(Sites 3/4/6/8 all reference `engraveMultisig` as the current upper bound — each must be repointed to the new program so the carousel includes it and the dot pager counts it.)

**Nav-tests:** the most recent precedent (T6b multisig) added **2** dedicated nav-tests in a new file `gui/<prog>_program_test.go` (`gui/multisig_program_test.go`): `Test…ProgramNavigable` (Right past the prior program → new upper bound → Right wraps to `backupWallet`; asserts non-blank title + no render-panic) and `Test…LeftWrap` (Left from `backupWallet` wraps to the new program). Earlier programs (bundle/xpub) shipped only 1. The task brief's "3 nav-tests" = these 2 + ensuring the existing carousel-count assertions in the prior programs' tests still pass (they hard-code the prior upper bound and Right-count, so they must be re-pointed). Plan for **2 new nav-tests + updating the prior program nav-tests' counts** (≥1 touched).

**Mainnet-only:** YES, consistent with T4/T6 — `engraveSingleSigFlow`/`deriveXpubFlow` derive against `&chaincfg.MainNetParams`; the BIP-85 path itself is network-agnostic, and the child seed-backup plate (option a) is just words+SeedQR (no network in the artifact). Mainnet-only is fine.

## B7 — Sizing (option a)

- `gui/bip85.go` orchestrator (typed seed → picker → derive → `engraveSeed`): ~120–160 LOC.
- Picker (`ChoiceScreen`-based app/words/index selection — app fixed to BIP-39, words ∈ {12,18,24}, index entry): ~60–100 LOC. **Keep the picker to what the lib/biptool support** (BIP-39 app, the 3 word counts, a hardened index). Index entry needs a small numeric-entry screen or a fixed small set; confirm an existing numeric-entry widget before assuming.
- Derive helper (path walk + `bip85.Entropy` + truncate + `bip39.New` + scrub): ~60–90 LOC (mirrors `deriveAccountXpub`).
- Tests (derive unit/known-vector + 2 nav-tests + scrub-hook test): ~100–150 LOC.
- **Total ~350–500 LOC → S–M.**

---

## Cross-cutting observations

- **`bip85.Entropy` is a primitive, not a flow.** The single biggest scoping risk is assuming the package "does BIP-85" — it does not derive paths or map word counts. The GUI flow re-implements the biptool `derive bip39` logic (B2) using `hdkeychain` + `bip39.New`. Budget for this; it is the meat of T7b.
- **Reuse `engraveSeed` for the child** (option a) → T7a and T7b share the same engrave primitive. This is also why doing T7a's doc note first clarifies the shared path.
- **Index/word-count picker bounds** must match biptool's guards exactly (`n∈{12,18,24}`, index hardened ≥ 0) to avoid a divergent on-device BIP-85 that disagrees with biptool/other wallets — a silent-wrong-backup class risk if the picker allows an out-of-spec word count.
- **Two secrets to scrub in T7b** (master mnemonic + derived child mnemonic) — more than the single-secret T4/T6a flows. The `defer`-scrub must cover both, plus the intermediate privkey/HMAC byte buffers (`wipeBytes`).
- The `seedqr` package's `CompactQR` is dead in the GUI (test-only) — if a future cycle wants compact SeedQR on steel, that is separate net-new wiring; out of T7 scope.

## Recommended build order

1. **T7a first — documentation only (no code).** Verify-and-document that Backup Wallet (program 0) already engraves a standard SeedQR via `engraveSeed`. Optionally note the NFC-mnemonic footgun. Effectively a same-day closeout. This also pins `engraveSeed` as the shared child-engrave primitive for T7b.
2. **T7b second — the real build (S–M).** No dependency on T7a code (only shares the `engraveSeed` primitive, already shipped). Follow the standard gate: RECON (this doc) → SPEC → R0 → IMPLEMENTATION_PLAN → R0 GREEN → TDD impl → adversarial review. Mirror the T6a/T6b lockstep + nav-test pattern. Engrave target = option (a) raw child BIP-39 words via `engraveSeed`.

No blocking dependency between them; T7a needs no engineering, so T7b is the cycle's real work.
