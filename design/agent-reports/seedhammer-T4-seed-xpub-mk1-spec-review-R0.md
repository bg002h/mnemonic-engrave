# R0 spec review — SeedHammer T4 (hand-typed seed → account xpub → engrave as mk1)

**Doc reviewed:** `design/SPEC_seedhammer_T4_seed_xpub_mk1.md` @ commit `49a43d6`
**Supporting:** `design/cycle-prep-recon-T4-seed-xpub-mk1.md`
**Authoritative sources verified:** fork `/scratch/code/shibboleth/seedhammer` @ `a4d669d` (binding decoder oracle); reference wire spec `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec` (working tree `0.4.0`, fork ported from family-token `mk-codec 0.2`); btcsuite `hdkeychain@v2.0.0`.
**Method:** ultracode recon phase — 5 parallel source-verification subagents (round-trip invertibility `acedf9e…`, compact-73/path/stub `ab8aee0…`, security spine `a66a4e3…`, GUI surface `a4dc387…`, derivation/determinism `acadf72…`), plus reviewer's own direct reads of `mk/mk.go`, `bytecode/encode.rs`, `xpub_compact.rs`, `chunk.rs`, `pipeline.rs`, `header.rs`, `consts.rs`, `codex32/checksum.go`, `codex32/mdmk.go`, `codex32/codex32.go`, `bip32/bip32.go`, `gui/gui.go`.

**USER DECISION honored (binding):** emit mk1; when no policy id can be derived (always in T4), warn on screen + set the policy-id stub to `0x00000000` (`stub_count=1`). This review checks structural validity + that the warning is mandated; it does not relitigate the product call.

---

## Verification Results (per numbered review item)

### 1. Encoder round-trip feasibility (§2.1/§4.1) — CONFIRMED feasible
Every decode step in the binding oracle (`mk/mk.go`) has a well-defined inverse:
- `fiveBitToBytes` (mk.go:98-120) — invertible; inverse = a `bytesToFive` (MSB-first pack, mirror of Rust `bytes_to_5bit` bch.rs:56-72). **Encoder obligation: zero pad bits** — the decoder rejects non-zero trailing pad (mk.go:116-118; tested mk_test.go:44-46).
- `reassemble` (mk.go:175-224) — invertible; inverse = `split_into_chunks` (chunk.rs:50-94). **Encoder obligation: append `SHA-256(bytecode)[0..4]` before splitting** — decoder validates it (mk.go:219-222; errCrossChunkHash).
- `decodeBytecode` (mk.go:226-287) — bijective TLV walk with trailing-bytes reject (mk.go:279-280). Inverse = `encode_bytecode` (encode.rs:55-67).
- `decodePath` (mk.go:308-339) — invertible **given canonical table-first encoding** (a path in the 14-entry table has two valid wire forms: std indicator AND explicit 0xFE; the Rust encoder always prefers the table — path.rs lookup-first; the Go encoder MUST match to reproduce golden vectors).
- `reconstructXpub` (mk.go:378-404) — the genuinely lossy field: compact-73 **drops depth + child_number** (xpub_compact.rs:5-6), reconstructed from the path on decode (mk.go:397-401). Invertible **iff** the encode-side guard (depth==pathlen AND child==terminal) is enforced.

The round-trip oracle `mk.Decode(mk.Encode(card)) == card` is sound, with the note that csid is not a Card field (so equality is on Card fields only; determinism pins csid — see item 3). `codex32.NewSeed` (codex32.go:279-383) provides a complete 5-bit-pack + BCH-generate + string-assembly template; the generate engine is target-agnostic (see item 8 / Finding I-1).

### 2. compact-73 encode invariant (§2.2) — CONFIRMED
encode.rs:38-48: `xpub.depth == component_count(origin_path) AND xpub.child_number == terminal_component` (empty path → child = Normal{0}). `bip32.Derive` (bip32.go:43-53) walks each element then `.Neuter()`s, producing an account key with `Depth()==len(path)` and childNum==terminal index — so m/84'/0'/0' (depth 3, child 0') and m/48'/0'/0'/2' (depth 4, child 2') both satisfy it. Decode mirror confirmed mk.go:397-401. The spec's "reject if depth/child disagree" is correct and the Go encoder must replicate it.

### 3. Deterministic chunk_set_id / no-CSPRNG (§2.3) — CONFIRMED
Decoder checks csid CONSISTENCY across chunks only, never its VALUE (mk.go:192; Rust chunk.rs:149). csid is a plain opaque 20-bit field (header.rs:27); **no reserved value** — csid=0 and csid=all-ones both round-trip (header.rs tests `chunked_round_trip_zero_csid`, `..._max_csid`). `top20(SHA-256(bytecode))` is wire-safe for any value. Entire derive/encode chain is rand-free (PBKDF2-SHA512, HMAC-SHA512 CKD, Hash160 FP); the only fork RNG is `bip39.RandomWord` (mnemonic GEN, off the typed path) + a CLI tool + the screensaver — none on-path. Device has no app CSPRNG (Platform exposes no entropy method). (Precision gap → Finding M-2.)

### 4. Stub-0 structural validity (§2.4, USER DECISION) — CONFIRMED
encode requires `stub_count >= 1` (encode.rs:24); decode rejects `stub_count==0` (mk.go:252-254; decode.rs:26). A card with `stub_count=1, stubs=[[0,0,0,0]]` passes encode (non-empty vec, bytes copied verbatim, no value inspection) and decodes to `Stubs==[[0,0,0,0]]` with NO error (mk.go:255-262 reads N stubs, zero value-checks). **No reserved/sentinel stub value** anywhere (key_card.rs:25-30, consts.rs:56, error.rs): the only stub-related reject is count==0. `0x00000000` does not collide with any codec-layer sentinel. Spec mandates the unskippable warning (§2.4) — confirmed present in §4.2 step 5 and §6 GUI tests. (See "Also assess" for the recovery-model safety note.)

### 5. SECURITY SPINE (§2.5) — CONFIRMED with two spec inaccuracies (Findings I-1-sec, I-2-sec) + one structural hazard the plan must enforce
- **No NFC writer:** Platform interface (gui.go:2620-2635) exposes only `NFCReader() io.ReadCloser` (read-only) — no write/emit method. Seed cannot be NFC-emitted. CONFIRMED.
- **Typed-only entry:** `newInputFlow`/`inputWordsFlow` (gui.go:2068/582) is a typed word keyboard; no NFC/QR ingest into it. CONFIRMED.
- **`.Neuter()` → no xprv:** `bip32.Derive` returns a neutered (public-only) key (bip32.go:51); its `.String()` serializes xpub, never xprv (extendedkey.go:486/592). CONFIRMED.
- **Seed-buffer scrub:** `bip39.MnemonicSeed` returns a 64-byte `[]byte` (bip39.go:217-226); `wipeBytes` zeroes a `[]byte` (slip39_polish.go:330); `defer wipeBytes(seed)` is the correct scrub. Existing `deriveMasterKey` does NOT scrub (gui.go:191-198) — spec's claim accurate; new helper adds it. CONFIRMED.
- **No engraveSeed path:** `engraveSeed` (gui.go:457) / `backup.EngraveSeed` engrave the SECRET seed; only call site is `backupWalletFlow` (gui.go:2023). T4's separate `program` keeps them apart. CONFIRMED structurally — but see the hazard below.

### 6. Multi-plate engrave (§2.6/§4.3) — CONFIRMED net-new; per-plate reset clean; set-level abort UNDEFINED (Finding I-3)
Today only single-plate engrave exists: `validateMdmk`/`mdmkFlow` (gui.go:1891/1933) lay ONE string into TEXT/QR variants of the SAME string; every engrave call site engraves exactly one `Plate`; multi-chunk is read-side only (`mk1GatherFlow`/`mk1DisplayFlow`). A loop with a fresh `NewEngraveScreen` per chunk is feasible — per-plate state is fresh (`newEngraverJob`; hardware acquired+Closed per job). `EngraveScreen.Engrave` returns `bool` (true=done, false=back/abort). The set-level semantics on mid-sequence abort are unpinned (Finding I-3).

### 7. Path picker (§2.7/§4.2) — CONFIRMED overflow; "copy mk1DisplayFlow" is not implementable as written (Finding I-4)
`ChoiceScreen.Draw` (gui.go:1411-1474) renders ALL choices unpaginated/unclipped (stacks `h += c.Size.Y`, centers the block); 14 entries overflow (largest existing list = 5). `mk1DisplayFlow`'s measure-and-advance (mk1_inspect.go:113-148) is a **pure display scroller** — no cursor, no `s.choice`, no per-line click target, no return. The 14 paths + depth-3/depth-4 (BIP-48) variety confirmed (path.rs:38-55; mk.go:291-306). (Finding I-4.)

### 8. Determinism/test strategy + scope — CONFIRMED feasible; one Critical-class precision gap (Finding C-1) + scope notes
- Round-trip through `mk.Decode` is a sound primary oracle (no separate Go encoder to diff). CONFIRMED.
- Golden-vector derivation feasible: the standard BIP-39 `abandon…about` vector exists (bip39_test.go:114); chain serializes **xpub** (not zpub) — version bytes are network-derived, not purpose-derived (chaincfg mainnet HDPublicKeyID = 0488b21e). CONFIRMED.
- `encode_with_chunk_set_id` is byte-deterministic given a fixed csid (no map iteration on the encode path; stubs are an ordered slice). CONFIRMED pinnable.
- BCH-generate: the fork's `codex32` engine (`inputTarget` checksum.go:124-128 + `inputFe` 156-170) operates purely on struct `generator`/`residue`/`target` fields — target/init-agnostic — and `verifyMDMK` already constructs the engine with mk1 generator + `POLYMOD_INIT` residue + mk targets (mdmk.go:103-107). A Go mk1 GENERATE wrapper is therefore ~5-10 net-new lines reusing that engine (the `NewSeed` generate pattern at codex32.go:369-372). NOT a blocker, but carries the POLYMOD_INIT trap (Finding C-1).

---

## Findings

### CRITICAL — must fix before GREEN

**C-1. The BCH-generate POLYMOD_INIT trap is not flagged; the spec implies the codex32 generate machinery is reusable as-is.**
*Where:* §3 ("the fork ALREADY has the generators + mk1 targets … `codex32.NewSeed` … is a complete working encoder template → reuse the pattern"); §4.1 ("generate the per-chunk BCH checksum … via the `codex32` engine + mk1 targets"); §6 ("BCH gen (reusing `codex32` machinery)").
*Why:* `codex32.NewSeed` (codex32.go:279-383) GENERATES against codex32's residue init **POLYMOD_INIT=1** and the codex32 SECRETSHARE target via `newShortChecksum()`/`newLongChecksum()` (codex32.go:354-359; checksum.go:36-46/57-66). mk1 requires a DIFFERENT initial residue **POLYMOD_INIT=0x23181b3** (mdmk.go:39) and the mk targets. The fork's own `mdmk.go` header (lines 7-13) warns verbatim that "copying newShortChecksum's residue field and only swapping target would compute every checksum against the wrong starting state and silently mis-validate" — and the GENERATE side has the identical trap. An encoder that follows the spec's "reuse the NewSeed pattern + mk targets" literally, without also replacing the residue init, will emit `mk1…` strings that pass nothing and that `ValidMK` rejects — but only at runtime, not at compile time. Two source agents independently flagged this; the spec text as written would lead a single implementer straight into it. (This is the spec's analog of the "1 valid last word"-class false-consensus hazard the project policy warns about.)
*Fix:* §4.1 must state explicitly that the mk1 GENERATE wrapper builds the `codex32` engine with `residue = unpackSyms(0, mdmkPolymodInitLo, n)` (POLYMOD_INIT 0x23181b3, **NOT** codex32's 1) + the mk regular/long generator + the mk regular/long target (mirroring `verifyMDMK` mdmk.go:103-107, the only existing mk1-correct engine constructor), then `inputHRP("mk")` + `inputData(payload)` + `inputTarget()` + read `residue` as the checksum symbols. Add a TDD assertion in §6 that each emitted chunk passes `codex32.ValidMK` (already listed) AND a unit assertion that the generate routine uses the mk POLYMOD_INIT (e.g. a known-answer checksum vector), so the wrong-init bug fails a test rather than shipping. Also pin the regular-vs-long selection by data-part length (14..=93 regular / 96..=108 long; 94-95 reserved-invalid — mdmk.go:47-50).

### IMPORTANT — must fix before GREEN

**I-1. Master + intermediate private keys are unscrubbed, and the spec's mitigation is factually wrong (a scrub API exists).**
*Where:* §2.5 / §4.1 (seed scrub only); recon line 33 ("Master `*hdkeychain.ExtendedKey` private material has no easy scrub hook (defense-in-depth, document)").
*Why:* `(*ExtendedKey).Zero()` **does exist** (hdkeychain extendedkey.go:634 — zeroes key/pubKey/chainCode/parentFP); the "no scrub hook" claim is false. Worse, `bip32.Derive` (bip32.go:44-51) walks through **intermediate PRIVATE child keys** (master is private → every `key.Derive(p)` yields a private child) and only `.Neuter()`s the final one — so the master and every depth-1..n-1 intermediate are live secret buffers that `Derive` returns without zeroing. The spec scrubs only the 64-byte seed, leaving the higher-value private keys resident. This is the cycle's highest-risk surface and the spec under-specifies it.
*Fix:* §4.1's derivation helper must (a) `defer master.Zero()`, and (b) zero each intermediate private child (either inside a T4-local derive that walks+zeroes, or by documenting that `bip32.Derive` is unsuitable and providing a scrubbing variant). Correct the recon/spec text: a scrub hook exists; the mitigation is to CALL it, not merely document. (Not Critical: none of these buffers is serialized/engraved/emitted — no exfil path — but for a SECRET-handling cycle this must be specified.)

**I-2. The mnemonic itself (the root secret) is never scrubbed and `wipeBytes` cannot scrub it.**
*Where:* §2.5 / §4 (no mention of mnemonic lifetime).
*Why:* `bip39.Mnemonic` is `[]Word` where `Word int` (bip39.go) — the root secret, more sensitive than the derived seed, held from seed entry through derive+engrave. `wipeBytes([]byte)` does not apply to `[]Word`; scrubbing needs an index loop. No existing flow scrubs it (consistent codebase gap), but a SECRET-handling cycle should address it.
*Fix:* §4.2 must specify the mnemonic's lifetime and a best-effort scrub (e.g. `for i := range m { m[i] = 0 }` or a typed helper) once derivation completes, and §6 should assert it best-effort.

**I-3. Multi-plate set-level abort/resume semantics are undefined; "Back/abort handled cleanly" is true only at the primitive level.**
*Where:* §2.6, §4.3 ("Back/abort handled cleanly").
*Why:* The per-plate primitive resets cleanly, but aborting after plate 1 of 2 leaves a HALF-ENGRAVED set (plate 1 physically engraved, plate 2 blank) and nothing in the codebase tracks or surfaces partial-set state. There is no precedent. Leaving this to the implementer risks an operator stranded with a useless single steel plate and no guidance.
*Fix:* §2.6/§4.3 must define the set-level contract: on mid-sequence abort, what does the UI say (e.g. "Plate 1 of 2 already engraved — discard and restart, or resume at plate 2?"), is resume supported, and is the partial state communicated. Add a §6 multi-plate test asserting the abort-after-plate-1 path reaches a defined screen (not a silent return).

**I-4. The path-picker resolution names an unimplementable option as a co-equal choice.**
*Where:* §1, §4.2 step 2 ("a paged `ChoiceScreen` (copy `mk1DisplayFlow`'s measure-and-advance) OR a two-stage picker").
*Why:* `mk1DisplayFlow`'s measure-and-advance (mk1_inspect.go:113-148) is a pure read-only scroller — no cursor, no per-line hit region, no selected index, no return value. "Copy it" yields paged DISPLAY with zero selection mechanism; making it selectable is net-new widget work (cursor model + window-crossing Up/Down + clipped per-line hit regions + chosen-index return), none of which the spec enumerates. Offering it as an equal alternative under-specifies the plan.
*Fix:* §4.2 should commit to the **two-stage picker** (script-type → network) — each stage fits the proven unpaginated `ChoiceScreen` within the ~5-entry limit and reuses `ChoiceScreen.Choose` verbatim — OR explicitly scope a paged-selectable `ChoiceScreen` as net-new widget work with the four additions enumerated. Recommend the two-stage picker.

**I-5. Adding a user-navigable `program` is not the "additive, low-risk" change the spec implies; several sites hard-clamp to a single navigable program.**
*Where:* §2.8 / §4.2 ("New top-level `program` (parallel to `backupWallet`)"); §4.4 ("new `program` enum value + StartScreen/uiFlow dispatch").
*Why:* The enum already has two values (`backupWallet`, `qaProgram` gui.go:148-149), but `qaProgram` is reachable ONLY via a hidden NFC debug string — it is NOT navigable. `StartScreen.Flow` clamps paging to `backupWallet` on both edges (gui.go:1626, 1633-1634); `layoutMainPlates` does `panic("invalid page")` for any page != `backupWallet` (gui.go:1836-1843); the pager constants are `const npage/npages = int(backupWallet)+1` (=1, gui.go:1828/1847), which also gate whether the L/R arrows render; the draw title switch (gui.go:1651-1654) has only a `backupWallet` case. A user-navigable T4 program requires updating ALL of these together (new max, new plate image / panic-safe default, new pager count, new title case) — and any new per-frame draw on the StartScreen path must stay 0-alloc (TestAllocs gates `StartScreen.Flow`).
*Fix:* §4.4 must enumerate these touch points (navigation clamps, `layoutMainPlates` default/panic, pager constants `npage`/`npages`, draw title case) as part of the additive work, and the plan must verify `TestAllocs` still passes if the StartScreen now renders a multi-page pager.

### MINOR — fix opportunistically; do not block

**M-1. Fingerprint conflation hazard (master FP vs parent FP) — implementation note, not a wire defect.**
The compact-73 carries the account xpub's **parent** fingerprint (xpub_compact.rs:36,49; mk.go:402 builds the xpub with `parentFP=compact[4:8]`), while the bytecode `[fp]` field is the **master** fingerprint (key_card.rs:33-34; mk.go:263-270). For depth-3/4 accounts these always differ. The round-trip oracle is SAFE provided the encoder sources the parent FP **from the account xpub base58** (which `bip32.Derive` populates correctly) and the bytecode `[fp]` from the master FP — but the encode-side invariant validates only depth+child, **not** parent FP (encode.rs:41), so a hand-built card with a wrong/zero parent FP would encode silently. §4.1 should state: bytecode `[fp]` ← master FP; account `Xpub` (carrying its true parent FP) MUST come from a real CKD `Derive`, never a hand-assembled key. (Minor because the GUI path always uses `bip32.Derive`, which is correct by construction; the hazard is only for hand-built test cards.)

**M-2. Pin the exact "top20(SHA-256(bytecode))" derivation.**
csid affects the output bytes (it is in the chunked header + folded into each chunk's BCH), so golden-vector stability requires a byte-exact definition. §2.3/§4.1 say "top-20-bits of SHA-256(bytecode)" but not the slice/shift/endianness. Pin it precisely, e.g. `csid = (be_u32(SHA256(canonical_bytecode)[0..4]) >> 12) & 0xFFFFF`, and state it hashes the **canonical bytecode** (not the bytecode‖hash stream). (Minor: any choice round-trips; this is for cross-impl reproducibility.)

**M-3. "always ≥2 chunks" is correct for T4 card shapes but must not be hardcoded.**
The minimum mk1 bytecode (80 bytes, no-fp 1-stub) exceeds the single-string ceiling `SINGLE_STRING_LONG_BYTES=56` (pipeline.rs:73; pipeline.rs:356 comment), so every real mk1 card is chunked, and 80+4 / 84+4 → 2 chunks. So "always chunked, ≥2 chunks" holds. But a 3-stub or long explicit-path card is 3 chunks (golden vectors V5/V6/V7), so the encoder must compute `ceil(stream_len/53)` (chunk.rs:73), not hardcode 2. §4.3's "plate 1 of N" framing is correct; ensure §6's "assert ≥2 chunks" stays `>= 2`, not `== 2`.

**M-4. Stale family token.**
The fork self-identifies as `mk-codec 0.2` (consts.rs:50 `GENERATOR_FAMILY`; mk.go:5) while its runtime decode behavior matches 0.4 (it accepts the depth-0/empty-path case and the 0x16 indicator). No wire drift affects round-trip (the decoder is a superset; the spec's standard-path picker emits only depth-3/4 cards anyway). If the spec/plan asserts a "0.4" family token or scopes depth-0 emission, reconcile the token text; otherwise informational.

---

## Also assess

- **Is an all-zero-stub mk1 card actively harmful beyond "unbound"?** No codec-layer collision: `0x00000000` is not a reserved/wildcard/"any-policy" sentinel anywhere in the codec (key_card.rs, consts.rs, error.rs, mk.go all treat the stub as an opaque truncated hash with no special values). Recovery-model safety: a stub is defined as `SHA-256(policy_canonical_bytecode)[0..4]`, so `0x00000000` is an extraordinarily improbable real hash prefix — it cannot be mistaken for a genuine policy binding, and a downstream verifier that re-derives the stub from a real policy will simply mismatch (correctly indicating "not this policy"). The only residual risk is purely semantic — a user could mistake an unbound card for a bound one — which the mandatory §2.4 warning addresses. No safety consequence requiring a spec change beyond keeping the warning unskippable (already mandated). The user's stub-0 decision is structurally and recovery-model sound.
- **No false protocol assumption** survived verification except the two flagged (C-1 generate-init, M-1 fingerprint). The 14-path table, stub semantics, chunk layout, cross-chunk hash, header bit layout, and 73-byte compact form all match the binding decoder exactly.

---

## Verdict

**NOT GREEN — 1C / 5I**

Re-dispatch after folding C-1 + I-1..I-5 (M-1..M-4 may be folded opportunistically). The encoder/derivation/security architecture is sound and the round-trip oracle strategy is correct; the blocking items are (C-1) a silent BCH-init correctness trap the spec text would lead an implementer into, and (I-1..I-5) under-specified secret-scrub coverage, multi-plate abort semantics, picker implementability, and the StartScreen wiring footprint of a new navigable program. None require an architecture change — all are spec-precision folds.
