# T6a-1 (headless) Implementation Plan — `md.EncodeSingleSig` + `codex32.EncodeMS1` + verify comparator

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development or executing-plans. `- [ ]` checkboxes; strict TDD (fail → run-fail → impl → run-pass → commit per task).

**Goal:** The HEADLESS wire-format core of T6a — a byte-faithful single-sig **wallet-policy** md1 encoder, the net-new ms1 encoder, and the deterministic verify-bundle comparator — so the byte-lock-risky work is gated + proven before any GUI (T6a-2).

**Architecture:** `md.EncodeSingleSig` builds a wallet-policy `*descriptor` (n=1, pubkeys+fp TLV, explicit origin) for 4 script shapes inside package `md` and calls the shipped `split` (CHUNKED — a single-sig wallet-policy payload is 81 bytes > the 320-bit single-string cap, so md1 is always chunked; `encodeMD1String` is NOT used). `codex32.EncodeMS1` wraps `codex32.NewSeed` with the correct ms1 entr payload. The comparator deterministically re-derives + diffs. All three are headless + golden/round-trip-gated.

**Tech stack:** Go (host tests via `/home/bcg/.local/go/bin/go`; TinyGo-safe). Module `seedhammer.com`.

**Spec:** `design/SPEC_seedhammer_T6a_singlesig_flagship.md` (GREEN @ R1, `42f7edc`) — Phase A. **Spec R0+R1:** `design/agent-reports/seedhammer-T6a-singlesig-spec-review-R{0,1}.md`. **Rust reference (pinned):** `md-codec` v0.36.0 @ `c85cd49`, `ms-codec` v0.4.4, the toolkit `mnemonic-toolkit` (the wallet-policy md1 shape `synthesize.rs:140-155` / `cell_7_wpkh_full` `md-codec/tests/wallet_policy.rs:190-204`).

## Locked (from spec, R0-gated)
Wallet-policy md1 (NOT template-only); 4 distinct AST bodies; explicit origin EMITTED for all 4 (R0-P2: the validator only REQUIRES it for sh-wpkh, but we EMIT it for all to match the toolkit + determinism); ms1 = `NewSeed("ms",0,"entr",'s',[0x00‖entropy])`, English/entr-only; mainnet-only; canonicalize runs (no-op at n=1).

---

## Task 0: Worktree + baseline + generate wallet-policy reference goldens (R0-C1/A1, R0-I1)
- [ ] **Step 1:** `git worktree add ../seedhammer-wt-t6a1 -b feat/t6a1-headless e4013a8` (sibling-dir; sandbox-fallback `git checkout -b` in place + say so). Work there.
- [ ] **Step 2:** Baseline `/home/bcg/.local/go/bin/go test ./md/... ./codex32/... ./bip39/... ./bip32/...` → all pass; else BLOCKED.
- [ ] **Step 3 (the differential goldens):** Generate NEW **key-bearing wallet-policy** md1 reference strings (the template-only `wpkh_basic`/etc. CANNOT serve — R0-C1; every vendored `*.descriptor.json` has `pubkeys:null`). Run the Rust toolkit `mnemonic-toolkit` — confirmed-feasible form: `mnemonic bundle --template bip84 --slot @0.phrase=<abandon seed> --json` emits the key-bearing single-sig md1 + xpub + fp (`73c5da0a` for the abandon seed) — for each of the 4 paths: BIP-44 `m/44'/0'/0'` (pkh), BIP-49 `m/49'/0'/0'` (sh-wpkh), BIP-84 `m/84'/0'/0'` (wpkh), BIP-86 `m/86'/0'/0'` (tr). **Each md1 is CHUNKED** (multiple strings, R0-I1) — capture ALL chunk strings + the account xpub (base58) + the master fingerprint + the origin path. Vendor into `md/testdata/vectors/singlesig_*.{md1.txt (one chunk per line),xpub.txt,meta.json}` with a `README` noting provenance (**R0-M3:** `md-codec` registry v0.36.0 == git `c85cd49`; toolkit tree SHA; the test seed; pin fp `73c5da0a`). **Fallback if the toolkit won't run:** a throwaway Rust harness building the `cell_7_wpkh_full`-style AST per script via `md-codec` + `chunk::split`; vendor its output. Confirm each vendored md1 passes the shipped **`md.DecodeChunks`** (NOT `md.Decode` — it refuses chunked, R0-I2) + `md.ExpandWalletPolicyChunks` and recovers the expected xpub/fp/origin/script.
- [ ] **Step 4: Commit** (signed+DCO, Brian Goss, Co-Authored-By; explicit paths) — `md: vendor key-bearing wallet-policy md1 reference goldens (T6a-1)`.

---

## Task 1: `codex32.EncodeMS1` (NET-NEW, R0-C4)

**Files:** Create `codex32/msencode.go`; Test `codex32/msencode_test.go`.

`EncodeMS1(entropy []byte) (string, error)` = `NewSeed("ms", 0, "entr", 's', payload)`, `payload = append([]byte{0x00}, entropy...)` (the `0x00` entr prefix, `codex32/mspayload.go:5-12`; id FIXED `"entr"`; share lowercase `'s'`). English/entr-only (no language byte). Validate entropy length (BIP-39: 16/20/24/28/32 bytes).

- [ ] **Step 1: Failing tests** (`codex32/msencode_test.go`): `DecodeMS1(EncodeMS1(entropy)) == entropy` for 16/32-byte entropy; **the wire begins `ms10entrs` (R0-M5 — assert this exact prefix: hrp `ms`, threshold `0`, id `entr`, share `s`)** and the recovered `Seed()[0]==0x00` / `id=="entr"`; an invalid entropy length → error; **(R0-M4) pin the known vector:** `EncodeMS1([16 zero bytes]) == "ms10entrsqqqq…cj9sxraq34v7f"` (the fork's own verified vector + live toolkit output); and the abandon-seed's 16-byte entropy → its expected `ms10entrs…` string (capture once from the toolkit/`DecodeMS1` round-trip).
- [ ] **Step 2: Run → FAIL** (`EncodeMS1` undefined — fork has only `DecodeMS1`).
- [ ] **Step 3: Implement** `codex32/msencode.go` per above.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `codex32: EncodeMS1 (entr payload, fixed id, round-trips DecodeMS1) (T6a-1, C4)`.

---

## Task 2: `md.EncodeSingleSig` (wallet-policy, 4 shapes; R0-C1/C2/C3, P1)

**Files:** Create `md/encode_singlesig.go` (in package `md`); Test `md/encode_singlesig_test.go`.

**Signature (R0-I2):** `func EncodeSingleSig(chainCode [32]byte, compressedPubkey [33]byte, fp [4]byte, origin []PathComponent, script ScriptKind) ([]string, error)` — returns the **CHUNKED** md1 strings (≥2; ~3 for wpkh — see R0-I1). **R0-P1:** `PathComponent{Hardened bool, Value uint32}` is a NET-NEW EXPORTED type (the internal `pathComponent` `md/md.go:173` is unexported) — declare it + map to the internal form. **R0-M2:** the existing `ScriptKind` enum has NO `ScriptShWpkh` value → **APPEND `ScriptShWpkh` after `ScriptTr`** (do NOT insert/renumber — that breaks `rootScriptKind`/#10b consumers); reuse the other exported values. (R0-M5: `PathComponent` is the encoder's RAW component — `{Hardened, Value}` — NOT the in-band `+HardenedKeyStart` `bip32.Path` form; do not conflate.)

Build the wallet-policy `*descriptor`, then **`split` it (CHUNKED — R0-I1):** a single-sig wallet-policy payload is **644 bits / 81 bytes**, which EXCEEDS the single-string cap, so it is ALWAYS chunked (the toolkit uses `chunk::split`, ~3 strings for wpkh, NEVER `encode_md1_string`). `EncodeSingleSig` calls the shipped `split` (`md/chunk.go:121`) — DROP `encodeMD1String` (single-string) from this path.
- `n=1`; `pathDecl{n:1, paths: Shared(originPath from the 4 components)}`; `useSite = <0;1>/*` (hasMultipath, alts {0},{1}, wildcard unhardened);
- `tlv{ pubPresent:true, pubkeys:[{idx:0, xpub: chainCode‖compressedPubkey}], fpPresent:true, fingerprints:[{idx:0, fp}] }`;
- **tree per script (R0-C2):** `ScriptPkh → node{tagPkh, keyArgBody{0}}`; `ScriptWpkh → node{tagWpkh, keyArgBody{0}}`; `ScriptTr → node{tagTr, trBody{isNums:false, keyIndex:0, tree:nil}}`; `ScriptShWpkh → node{tagSh, childrenBody{[node{tagWpkh, keyArgBody{0}}]}}`;
- route through `encodePayload`→`canonicalize` (no-op at n=1 but DO NOT bypass); reject if `len(origin)==0` (explicit origin mandatory — emit always).

- [ ] **Step 1: Failing tests.** (a) **PRIMARY parity (R0-A1, R0-M1 — form-independent):** for each of the 4 vendored goldens (Task 0), parse the vendored xpub → (chainCode, compressedPubkey), `EncodeSingleSig(...)` → reassemble Go's chunked strings back to PAYLOAD BYTES and assert byte-equal to the toolkit's reassembled payload (form-independent — robust to any chunk-framing/csid variance); AND assert the chunked STRINGS equal the vendored toolkit strings (exact wire — deterministic); each chunk `ValidMD`. (b) **round-trip safety net (R0-A2/I2):** `md.DecodeChunks`→`ExpandWalletPolicyChunks` of the output recovers the embedded xpub/fp/origin/script. **Use `DecodeChunks`/`ExpandWalletPolicyChunks` ONLY — `md.Decode` REFUSES chunked input** (and `DecodeChunks` refuses single — mutually exclusive). (c) sh-wpkh with empty origin → error; tr emits a `trBody` (decode confirms `is_nums:false, tree:nil`).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `md/encode_singlesig.go` (+ exported `PathComponent`, the appended `ScriptShWpkh`) per above; build the unexported `*descriptor` in-package; call the shipped `split` (NOT `encodeMD1String`).
- [ ] **Step 4: Run → PASS** (all 4 shapes byte-equal + round-trip).
- [ ] **Step 5: Commit** — `md: EncodeSingleSig — wallet-policy md1, 4 shapes, byte-equal vs toolkit goldens (T6a-1, C1-C3,I1,I2)`.

---

## Task 2W: PORT `md.WalletPolicyId` (`compute_wallet_policy_id`) — the mk1 stub source (NET-NEW; bundle-composition recon)

**Files:** Create `md/walletpolicyid.go` (package `md`); Test `md/walletpolicyid_test.go`.

**Why (recon `seedhammer-T6-recon-bundle-composition-stub.md`):** in a bundle the mk1 KEY card binds to the md1 POLICY card via `policy_id_stub = WalletPolicyId(descriptor)[0:4]` (SPEC_mk v0.1 §3.3, audit-I1 — NOT the bytecode/`Md1EncodingId` chunk-id; the `mk-codec key_card.rs:27` doc comment is STALE). The fork has ONLY `computeEncodingID` (= `Md1EncodingId`, the chunk-set-id source, `md/identity.go`) — the WRONG primitive for the stub. `WalletPolicyId` does NOT exist in Go (grep: zero hits) → port it. (T6a-2/GUI consumes this to set the bound stub; T4's stub-0 + "Unbound Key Card" warning does NOT apply to the bundle's policy-bound mk1.)

`md.WalletPolicyId(d *descriptor) ([16]byte, error)` = the canonical-expanded policy hash, byte-exact vs Rust `md-codec/src/identity.rs:172-240`: SHA-256 over (placeholder-form tree bytes ‖ per-@N records, where each record = `presence_byte (fp_present | xpub_present<<1)` + fp[4] + xpub[65] when present), truncated to 16 bytes. Distinct from `WalletDescriptorTemplateId` (`identity.rs:71-104`, template-only) and `Md1EncodingId` (`identity.rs:39-45`, the chunk-id). Add an exported `func WalletPolicyIDStub(d) [4]byte` = `WalletPolicyId(d)[0:4]` for the caller.

- [ ] **Step 1: Failing tests.** **Differential vs Rust:** for each of the 4 vendored wallet-policy goldens (Task 0), `WalletPolicyId(decoded)` byte-equals the toolkit's `compute_wallet_policy_id` for the same wallet (capture from the toolkit, e.g. the engraved mk1 stub = `WalletPolicyId[0:4]` — pin it). **Key-presence-significance:** nulling the pubkeys+fp (template form) yields a DIFFERENT id (mirror `walletpolicyid_template_only_differs_from_full_cell_7`, `identity.rs:610-617`). **Encoding-stability:** origin-elided vs explicit-origin forms of the same wallet yield the SAME id (`identity.rs:572-605`). Confirm `WalletPolicyId != computeEncodingID` for a key-bearing descriptor.
- [ ] **Step 2: Run → FAIL** (`WalletPolicyId` undefined).
- [ ] **Step 3: Implement** `md/walletpolicyid.go` — the canonical-expanded preimage + SHA-256[:16], byte-exact vs `identity.rs:172-240`; `WalletPolicyIDStub`.
- [ ] **Step 4: Run → PASS** (differential + presence-significance + stability).
- [ ] **Step 5: Commit** — `md: port WalletPolicyId (compute_wallet_policy_id) byte-exact vs Rust — the mk1 stub source (T6a-1)`.

---

## Task 3: verify-bundle deterministic comparator (R0-I6)

**Files:** Create `md/verify_bundle.go` or a `gui`-adjacent headless helper (decide home: it composes `md` + `mk` + `codex32` — a small `seedhammer.com/bundle`-like helper or a `gui` headless func); Test alongside.

A pure function comparing a freshly-derived single-sig set against a read-back set: master fingerprint, account xpub, origin path, **md1 string exact-match** (deterministic), **ms1 recovered-ENTROPY bytes** (compare entropy, not string), and **the mk1↔md1 stub binding** — assert the read-back mk1's `policy_id_stub == md.WalletPolicyIDStub(decoded md1)` (the cards belong together). Returns PASS or the first diverging field. (Wordlist-language dropped per the entr lock.)

- [ ] **Step 1: Failing tests.** A correct derived-vs-readback set → PASS; a mutated xpub / descriptor string / entropy → FAIL naming the field; **a mk1 whose stub ≠ `WalletPolicyIDStub(md1)` → FAIL "stub mismatch"** (the key card doesn't belong to this policy); ms1 compared on recovered entropy (so a re-typed ms1 with the same entropy but any incidental string difference still matches).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** the comparator (deterministic; no secret retained beyond the compare; scrub any entropy copy it makes).
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `md/bundle: deterministic verify-bundle comparator (fp/xpub/path/md1/ms1-entropy) (T6a-1, I6)`.

---

## Task 4: No-regression + fuzz

**Files:** Test only.
- [ ] **Step 1:** `/home/bcg/.local/go/bin/go test -count=1 ./...` green; `go vet ./md/... ./codex32/...` clean (vs baseline); `gofmt -l` empty. Existing md/mk/codex32/ms1 decode + #10a/#10b + T5 unchanged (their tests pass verbatim).
- [ ] **Step 2: Fuzz** `FuzzEncodeSingleSig` (random valid (xpub,fp,origin,script) → encode → `DecodeChunks`→`ExpandWalletPolicyChunks` recovers the inputs; no panic), `FuzzEncodeMS1` (random entropy → `DecodeMS1` round-trips; no panic), `FuzzWalletPolicyId` (random decoded descriptors → `WalletPolicyId` no panic + deterministic + ≠ `computeEncodingID`), and the comparator (no panic). ≥1M execs each.
- [ ] **Step 3: Run → 0 panics.**
- [ ] **Step 4: Commit** — `md/codex32: no-regression + fuzz for the T6a-1 encoders + comparator (T6a-1)`.

---

## Acceptance (GREEN bar for the exec review)
- `EncodeSingleSig` byte-equal to the vendored key-bearing wallet-policy goldens for ALL 4 shapes (wpkh/pkh/tr/sh-wpkh) + round-trip recovers xpub/fp/origin/script (Task 2). `EncodeMS1`↔`DecodeMS1` round-trips entropy (Task 1). **`WalletPolicyId` byte-exact vs the toolkit + key-presence-significant + ≠ `computeEncodingID` (Task 2W)** — the mk1 stub source. The comparator is deterministic + field-precise incl. the stub-binding check (Task 3). Full suite green; fuzz 0 panics; no regression to shipped codecs/flows (Task 4). NO GUI in this cycle (T6a-2).

## Self-review (author, post-bundle-composition-fold)
- Spec Phase-A coverage: `EncodeSingleSig`→T2; `EncodeMS1`→T1; **`WalletPolicyId` port→T2W**; comparator (incl. stub-binding)→T3; differential goldens→T0. Spec-R0 folds: C1/C2/C3 wire→T2; C4 ms1→T1; spec-I2 signature→T2; restore/scrub→T6a-2 (GUI).
- **Bundle-composition fold (this round, user-confirmed full-policy-only):** the mk1↔md1 stub = `WalletPolicyId(md1)[0:4]` (NOT `computeEncodingID`/chunk-id, NOT the stale bytecode-hash doc; SPEC_mk §3.3 audit-I1) → NET-NEW `md.WalletPolicyId` port (T2W, byte-exact vs Rust `identity.rs:172-240`) + the comparator stub-binding check (T3). T6's mk1 stub is policy-bound/NON-ZERO (vs T4 stub-0 + "Unbound Key Card" warning) — the stub-SETTING + warning-drop is a T6a-2/GUI concern (the bundle passes `WalletPolicyIDStub(md1)` to `mk.Encode`). Template-only md1 is OUT (full-policy only; constellation-level template engraving → `mnemonic-engrave`/`mnemonic-toolkit` FOLLOWUPS).
- **Plan-R0 folds (this round):** **I1** md1 is CHUNKED (644b/81B) → `split` not `encodeMD1String`, output ≥2 strings → T2 + T0 (goldens are multi-chunk); **I2** round-trip + golden-verify via `DecodeChunks`/`ExpandWalletPolicyChunks` ONLY (`Decode` refuses chunked) → T0/T2/T4; **M1** payload-byte parity as the form-independent primary gate → T2.1; **M2** APPEND `ScriptShWpkh` (no renumber) → T2; **M3** provenance v0.36.0==c85cd49 → T0; **M4** pin fp `73c5da0a` + the `ms10entrsqqqq…` vector → T0/T1; **M5** assert `ms10entrs` prefix → T1. P1 (PathComponent exported)→T2; P2 (emit-origin-for-all)→T2; P3 (nav/consts)→T6a-2 (GUI, NOT here). ✓
- No placeholders; each step cites the Rust shape + the Go reuse. The differential gate (payload-byte parity + chunked-string equality) is the make-or-break (Task 2.1). ✓

## Gate
This plan MUST pass opus R0 (the FOCUSED wire-format gate) to 0C/0I before code; fold → persist → re-dispatch until GREEN. Then single-implementer TDD in the worktree → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h. Then the T6a-2 (GUI) plan → its own R0 → impl → exec review. Then T6b.
