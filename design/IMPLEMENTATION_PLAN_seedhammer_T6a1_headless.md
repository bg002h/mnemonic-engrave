# T6a-1 (headless) Implementation Plan — `md.EncodeSingleSig` + `codex32.EncodeMS1` + verify comparator

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development or executing-plans. `- [ ]` checkboxes; strict TDD (fail → run-fail → impl → run-pass → commit per task).

**Goal:** The HEADLESS wire-format core of T6a — a byte-faithful single-sig **wallet-policy** md1 encoder, the net-new ms1 encoder, and the deterministic verify-bundle comparator — so the byte-lock-risky work is gated + proven before any GUI (T6a-2).

**Architecture:** `md.EncodeSingleSig` builds a wallet-policy `*descriptor` (n=1, pubkeys+fp TLV, explicit origin) for 4 script shapes inside package `md` and calls the shipped `split`/`encodeMD1String`. `codex32.EncodeMS1` wraps `codex32.NewSeed` with the correct ms1 entr payload. The comparator deterministically re-derives + diffs. All three are headless + golden/round-trip-gated.

**Tech stack:** Go (host tests via `/home/bcg/.local/go/bin/go`; TinyGo-safe). Module `seedhammer.com`.

**Spec:** `design/SPEC_seedhammer_T6a_singlesig_flagship.md` (GREEN @ R1, `42f7edc`) — Phase A. **Spec R0+R1:** `design/agent-reports/seedhammer-T6a-singlesig-spec-review-R{0,1}.md`. **Rust reference (pinned):** `md-codec` v0.36.0 @ `c85cd49`, `ms-codec` v0.4.4, the toolkit `mnemonic-toolkit` (the wallet-policy md1 shape `synthesize.rs:140-155` / `cell_7_wpkh_full` `md-codec/tests/wallet_policy.rs:190-204`).

## Locked (from spec, R0-gated)
Wallet-policy md1 (NOT template-only); 4 distinct AST bodies; explicit origin EMITTED for all 4 (R0-P2: the validator only REQUIRES it for sh-wpkh, but we EMIT it for all to match the toolkit + determinism); ms1 = `NewSeed("ms",0,"entr",'s',[0x00‖entropy])`, English/entr-only; mainnet-only; canonicalize runs (no-op at n=1).

---

## Task 0: Worktree + baseline + generate wallet-policy reference goldens (R0-C1/A1, R0-I1)
- [ ] **Step 1:** `git worktree add ../seedhammer-wt-t6a1 -b feat/t6a1-headless e4013a8` (sibling-dir; sandbox-fallback `git checkout -b` in place + say so). Work there.
- [ ] **Step 2:** Baseline `/home/bcg/.local/go/bin/go test ./md/... ./codex32/... ./bip39/... ./bip32/...` → all pass; else BLOCKED.
- [ ] **Step 3 (the differential goldens):** Generate NEW **key-bearing wallet-policy** md1 reference strings (the template-only `wpkh_basic`/etc. CANNOT serve — R0-C1). Run the Rust toolkit `mnemonic-toolkit` (at its pinned tree) to produce a single-sig md1 for a KNOWN test seed (the BIP-39 "abandon … about" vector) at each of the 4 paths — BIP-44 `m/44'/0'/0'` (pkh), BIP-49 `m/49'/0'/0'` (sh-wpkh), BIP-84 `m/84'/0'/0'` (wpkh), BIP-86 `m/86'/0'/0'` (tr) — capturing for each: the md1 string(s), the derived account xpub (base58), the master fingerprint, the origin path. Vendor into `md/testdata/vectors/singlesig_*.{md1.txt,xpub.txt,meta.json}` with a `README` noting provenance (toolkit @ its SHA + the test seed). **Fallback if the toolkit won't run:** a throwaway Rust harness constructing the `cell_7_wpkh_full`-style AST per script via `md-codec` directly + `encode_md1_string`; vendor its output. Confirm each vendored md1 passes the shipped `md.DecodeChunks`/`md.Decode` + `md.ExpandWalletPolicy` and recovers the expected xpub/fp/origin/script.
- [ ] **Step 4: Commit** (signed+DCO, Brian Goss, Co-Authored-By; explicit paths) — `md: vendor key-bearing wallet-policy md1 reference goldens (T6a-1)`.

---

## Task 1: `codex32.EncodeMS1` (NET-NEW, R0-C4)

**Files:** Create `codex32/msencode.go`; Test `codex32/msencode_test.go`.

`EncodeMS1(entropy []byte) (string, error)` = `NewSeed("ms", 0, "entr", 's', payload)`, `payload = append([]byte{0x00}, entropy...)` (the `0x00` entr prefix, `codex32/mspayload.go:5-12`; id FIXED `"entr"`; share lowercase `'s'`). English/entr-only (no language byte). Validate entropy length (BIP-39: 16/20/24/28/32 bytes).

- [ ] **Step 1: Failing tests** (`codex32/msencode_test.go`): `DecodeMS1(EncodeMS1(entropy)) == entropy` for 16/32-byte entropy; the wire begins `ms1...` and the recovered `Seed()[0]==0x00` / `id=="entr"` / share `'s'`; an invalid entropy length → error; a known vector (the abandon-seed's 16-byte entropy) → the expected `ms1entrs...` string (capture from the toolkit/`DecodeMS1` round-trip).
- [ ] **Step 2: Run → FAIL** (`EncodeMS1` undefined — fork has only `DecodeMS1`).
- [ ] **Step 3: Implement** `codex32/msencode.go` per above.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `codex32: EncodeMS1 (entr payload, fixed id, round-trips DecodeMS1) (T6a-1, C4)`.

---

## Task 2: `md.EncodeSingleSig` (wallet-policy, 4 shapes; R0-C1/C2/C3, P1)

**Files:** Create `md/encode_singlesig.go` (in package `md`); Test `md/encode_singlesig_test.go`.

**Signature (R0-I2):** `func EncodeSingleSig(chainCode [32]byte, compressedPubkey [33]byte, fp [4]byte, origin []PathComponent, script ScriptKind) ([]string, error)`. **R0-P1:** `PathComponent{Hardened bool, Value uint32}` is a NET-NEW EXPORTED type (the internal `pathComponent` `md/md.go:173` is unexported) — declare it + map to the internal form; reuse the already-exported `ScriptKind` (the type of `md.Template.Root`) — confirm its exported values cover wpkh/pkh/tr/sh; if a needed value is missing, add it. (R0-M5: `PathComponent` is the encoder's RAW component — `{Hardened, Value}` — NOT the in-band `+HardenedKeyStart` `bip32.Path` form used by expand/display; do not conflate.)

Build the wallet-policy `*descriptor` then call `encodeMD1String`/`split`:
- `n=1`; `pathDecl{n:1, paths: Shared(originPath from the 4 components)}`; `useSite = <0;1>/*` (hasMultipath, alts {0},{1}, wildcard unhardened);
- `tlv{ pubPresent:true, pubkeys:[{idx:0, xpub: chainCode‖compressedPubkey}], fpPresent:true, fingerprints:[{idx:0, fp}] }`;
- **tree per script (R0-C2):** `ScriptPkh → node{tagPkh, keyArgBody{0}}`; `ScriptWpkh → node{tagWpkh, keyArgBody{0}}`; `ScriptTr → node{tagTr, trBody{isNums:false, keyIndex:0, tree:nil}}`; `sh-wpkh → node{tagSh, childrenBody{[node{tagWpkh, keyArgBody{0}}]}}` (the script enum must distinguish sh-wpkh from bare wpkh — add a `ScriptShWpkh` value or a wrap flag);
- route through `encodePayload`→`canonicalize` (no-op at n=1 but DO NOT bypass); reject if `len(origin)==0` (explicit origin mandatory — emit always).

- [ ] **Step 1: Failing tests.** (a) **PRIMARY differential (R0-A1):** for each of the 4 vendored goldens (Task 0), parse the vendored xpub → (chainCode, compressedPubkey), `EncodeSingleSig(...)` → assert byte-equal to the vendored md1 string(s); each chunk `ValidMD`. (b) **round-trip safety net (R0-A2):** `md.DecodeChunks`/`Decode`→`ExpandWalletPolicy` of the output recovers the embedded xpub/fp/origin/script. (c) sh-wpkh with empty origin → error; tr emits a `trBody` (decode confirms `is_nums:false, tree:nil`).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `md/encode_singlesig.go` (+ `PathComponent`, the script-enum addition) per above; build the unexported `*descriptor` in-package; call the shipped `encodeMD1String`/`split`.
- [ ] **Step 4: Run → PASS** (all 4 shapes byte-equal + round-trip).
- [ ] **Step 5: Commit** — `md: EncodeSingleSig — wallet-policy md1, 4 shapes, byte-equal vs toolkit goldens (T6a-1, C1-C3,I1,I2)`.

---

## Task 3: verify-bundle deterministic comparator (R0-I6)

**Files:** Create `md/verify_bundle.go` or a `gui`-adjacent headless helper (decide home: it composes `md` + `mk` + `codex32` — a small `seedhammer.com/bundle`-like helper or a `gui` headless func); Test alongside.

A pure function comparing a freshly-derived single-sig set against a read-back set: master fingerprint, account xpub, origin path, **md1 string exact-match** (deterministic), **ms1 recovered-ENTROPY bytes** (compare entropy, not string). Returns PASS or the first diverging field. (Wordlist-language dropped per the entr lock.)

- [ ] **Step 1: Failing tests.** A correct derived-vs-readback set → PASS; a mutated xpub / descriptor string / entropy → FAIL naming the field; ms1 compared on recovered entropy (so a re-typed ms1 with the same entropy but any incidental string difference still matches).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** the comparator (deterministic; no secret retained beyond the compare; scrub any entropy copy it makes).
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `md/bundle: deterministic verify-bundle comparator (fp/xpub/path/md1/ms1-entropy) (T6a-1, I6)`.

---

## Task 4: No-regression + fuzz

**Files:** Test only.
- [ ] **Step 1:** `/home/bcg/.local/go/bin/go test -count=1 ./...` green; `go vet ./md/... ./codex32/...` clean (vs baseline); `gofmt -l` empty. Existing md/mk/codex32/ms1 decode + #10a/#10b + T5 unchanged (their tests pass verbatim).
- [ ] **Step 2: Fuzz** `FuzzEncodeSingleSig` (random valid (xpub,fp,origin,script) → encode → `DecodeChunks`→`ExpandWalletPolicy` recovers the inputs; no panic), `FuzzEncodeMS1` (random entropy → `DecodeMS1` round-trips; no panic), and the comparator (no panic). ≥1M execs each.
- [ ] **Step 3: Run → 0 panics.**
- [ ] **Step 4: Commit** — `md/codex32: no-regression + fuzz for the T6a-1 encoders + comparator (T6a-1)`.

---

## Acceptance (GREEN bar for the exec review)
- `EncodeSingleSig` byte-equal to the vendored key-bearing wallet-policy goldens for ALL 4 shapes (wpkh/pkh/tr/sh-wpkh) + round-trip recovers xpub/fp/origin/script (Task 2). `EncodeMS1`↔`DecodeMS1` round-trips entropy (Task 1). The comparator is deterministic + field-precise (Task 3). Full suite green; fuzz 0 panics; no regression to shipped codecs/flows (Task 4). NO GUI in this cycle (T6a-2).

## Self-review (author, pre-R0)
- Spec Phase-A coverage: `EncodeSingleSig`→T2; `EncodeMS1`→T1; comparator→T3; differential goldens→T0. R0 folds: C1/C2/C3→T2; C4→T1; I1→T0/T2; I2→T2 sig; I6→T3. Precision notes: P1 (PathComponent net-new exported)→T2; P2 (emit-origin-for-all)→T2 (locked-emit); P3 (nav/consts)→T6a-2 (GUI, NOT here). ✓
- No placeholders; each step cites the Rust shape + the Go reuse. The differential gate is the make-or-break (Task 2.1). ✓

## Gate
This plan MUST pass opus R0 (the FOCUSED wire-format gate) to 0C/0I before code; fold → persist → re-dispatch until GREEN. Then single-implementer TDD in the worktree → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h. Then the T6a-2 (GUI) plan → its own R0 → impl → exec review. Then T6b.
