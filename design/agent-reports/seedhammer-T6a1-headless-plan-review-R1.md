# R1 GATE RE-REVIEW (FOCUSED, wire-format) — IMPLEMENTATION_PLAN_seedhammer_T6a1_headless.md (opus architect)

**Date:** 2026-06-19
**Round:** R1 (focused plan-R0 wire-format gate, RE-REVIEW after the R0 fold; BEFORE any code).
**Reviewer role:** byte-lock gate. T6a-1 introduces NEW public API on a Rust-golden-byte-locked package (`md.EncodeSingleSig`) + a net-new ms1 encoder (`codex32.EncodeMS1`) + a deterministic verify-bundle comparator. The byte-lock is the whole point of this gate.
**PLAN under review (folded):** `design/IMPLEMENTATION_PLAN_seedhammer_T6a1_headless.md`
**Prior R0 (NOT GREEN, 0C/2I/5m):** `design/agent-reports/seedhammer-T6a1-headless-plan-review-R0.md`
**SPEC (GREEN @ R1):** `design/SPEC_seedhammer_T6a_singlesig_flagship.md` (Phase A = T6a-1; Phase B GUI = T6a-2, NOT in this plan).

**Sources verified first-hand @ pinned trees THIS ROUND (NOT the plan's prose):**
- Fork — `git rev-parse HEAD = e4013a88011284c71f6da1b5629555bdc52c7e88` (the plan's claimed base `e4013a8`), tree `/scratch/code/shibboleth/seedhammer`. ✓
- Rust md-codec v0.36.0 — local source-of-truth tree `/scratch/code/shibboleth/descriptor-mnemonic` `git rev-parse HEAD = c85cd498c690d9f91c7884234cf25d0c39264608` (== `c85cd49`). ✓
- mnemonic-toolkit `/scratch/code/shibboleth/mnemonic-toolkit` (`f7e6fca1…`) — built + run LIVE this round (`mnemonic bundle` for the abandon seed, all 4 templates).

**Method:** every folded claim re-verified against source text AND, where load-bearing, against a LIVE throwaway build/run (Go host probes of `encodePayload`/`split`/`NewSeed`/`DecodeMS1`; toolkit `mnemonic bundle --json` for bip44/49/84/86). All throwaway probes removed; working tree confirmed clean (`git status --porcelain` empty) at end.

---

## VERDICT: GREEN

**0 Critical, 0 Important, 0 (residual) Minor.**

Both Important findings from R0 are CLOSED, all five Minors are folded, and the fold introduced NO drift. The wire spine the R0 verified correct still holds verbatim. The plan is **cleared for single-implementer TDD** in the worktree.

The decisive new confirmation this round: I re-probed the TOOLKIT-FAITHFUL shape (explicit BIP-84 origin `m/84'/0'/0'` + fingerprint TLV + pubkey TLV — the actual `EncodeSingleSig` output, NOT the fp-less empty-origin `singlesigWithPubkey` test fixture) and `encodePayload` returns **81 bytes / 644 bits**, and `split` returns **3 chunks** — matching the plan's I1 numbers ("644 bits / 81 bytes", "~3 strings for wpkh") EXACTLY. The live toolkit independently emitted **3 chunked md1 strings** for every one of the 4 templates. The single-string cap is 320 bits (`md/chunk.go:39`), so 644 > 320 ⇒ ALWAYS chunked. I1 is not just folded in prose — it is numerically correct against source.

---

## CLOSED/STILL-OPEN dispositions

### I1 — lock output to CHUNKED (`split`); drop `encodeMD1String`. — **CLOSED.**

The plan now unambiguously commits the operative paths to `split` (chunked) and drops `encodeMD1String`:
- Task 2 line 46: "**`split` it (CHUNKED — R0-I1):** a single-sig wallet-policy payload is **644 bits / 81 bytes**, which EXCEEDS the single-string cap, so it is ALWAYS chunked … `EncodeSingleSig` calls the shipped `split` (`md/chunk.go:121`) — **DROP `encodeMD1String`** (single-string) from this path." ✓
- Task 2 line 54: "call the shipped `split` (**NOT `encodeMD1String`**)." ✓
- Signature returns `([]string, error)` (line 44) — the chunked form. ✓
- Task 0 line 21 captures goldens as MULTI-CHUNK: "**Each md1 is CHUNKED** (multiple strings, R0-I1) — capture ALL chunk strings". ✓

**Source re-verification (this round):**
- `split(d)` (`md/chunk.go:121`) ALWAYS sets `hdr.Chunked = true` (`md/chunk.go:159`) for every chunk, even count==1. ✓
- `singleStringPayloadBitLimit = 64 * 5 = 320` (`md/chunk.go:39`). ✓
- LIVE: the fp-bearing wpkh wallet-policy with explicit `m/84'/0'/0'` origin = **81 bytes / 644 bits → `split` = 3 chunks** (Go host probe). The earlier R0-cited 72B/575b figure was the fp-LESS, empty-origin `singlesigWithPubkey` fixture (`md/validate_test.go:29-40`: `shared:&originPath{}`, no fingerprints) — NOT the toolkit shape. The plan's 644b/81B is the correct toolkit-faithful number. ✓
- LIVE toolkit (`mnemonic bundle --json`, abandon seed, mainnet): bip84/bip44/bip49/bip86 each emitted **3 chunked md1 strings** (`md1fgdxlpq…` shared-csid prefix). ✓
- The toolkit emits md1 EXCLUSIVELY via `md_codec::chunk::split` (`synthesize.rs:183,219,276,481,653`); `Bundle.md1: Vec<String>` (`synthesize.rs:29`); `encode_md1_string` appears NOWHERE in toolkit `src/`. ✓

**Residual prose nit (NOT a finding, NO fold required):** the one-line Architecture summary (line 7) still reads "calls the shipped `split`/`encodeMD1String`" with the slash. This is the high-level abstract; the two OPERATIVE task lines (46, 54) and Step 3 both lock `split` and explicitly DROP `encodeMD1String`. A single-implementer following the checkboxed Task 2 cannot emit the wrong form. I flag it for tidiness but it is below Minor and does not gate — the operative instructions are unambiguous and correct.

### I2 — round-trip leg uses the CHUNKED decoders ONLY (`DecodeChunks`/`ExpandWalletPolicyChunks`), not `Decode`. — **CLOSED.**

The plan now names the chunked decoders exclusively, at all three sites:
- Task 0 Step 3 line 21: "Confirm each vendored md1 passes the shipped **`md.DecodeChunks`** (**NOT `md.Decode` — it refuses chunked, R0-I2**) + `md.ExpandWalletPolicyChunks`". ✓
- Task 2 Step 1(b) line 52: "**Use `DecodeChunks`/`ExpandWalletPolicyChunks` ONLY — `md.Decode` REFUSES chunked input** (and `DecodeChunks` refuses single — mutually exclusive)." ✓
- Task 4 Step 2 line 78: `FuzzEncodeSingleSig` "encode → `DecodeChunks`→`ExpandWalletPolicyChunks` recovers the inputs". ✓

The ambiguous "`DecodeChunks`/`Decode`→`ExpandWalletPolicy`" slash phrasing the R0 flagged is GONE from every site; no bare `md.Decode` survives on any round-trip leg (grep of the plan: the only `Decode` hit at line 21/52 is the explicit "NOT `md.Decode`" negation).

**Source re-verification (this round):**
- `Decode(s string)` (`md/md.go:1216`) refuses chunked: `if len(syms)==0 || syms[0]&1 == 1 { return Template{}, ErrChunkedUnsupported }` (`md/md.go:1221-1222`). ✓
- `DecodeChunks`/`Reassemble` refuse single: `readChunkHeader` returns `errChunkFlagMissing` when the chunked bit is clear (`md/chunk.go:95`). Mutually exclusive by wire form. ✓
- `ExpandWalletPolicyChunks([]string) (Template, []ExpandedKey, error)` (`md/expand.go:102-112`) = `Reassemble` → `ExpandWalletPolicy` → `summarize` — the correct per-@N xpub/fp/origin/script recovery from a chunk set (it returns the Template AND the keys, so the round-trip recovers script+xpub+fp+origin in one call). ✓

### M1 — payload-byte parity as the form-independent PRIMARY gate. — **CLOSED.**

Task 2 Step 1(a) line 52: "**PRIMARY parity (R0-A1, R0-M1 — form-independent):** … reassemble Go's chunked strings back to PAYLOAD BYTES and assert byte-equal to the toolkit's reassembled payload (form-independent …); AND assert the chunked STRINGS equal the vendored toolkit strings (exact wire — deterministic); each chunk `ValidMD`." Payload-byte parity is PRIMARY, chunked-string equality SECONDARY — exactly as M1 recommended. ✓ (Consistent with the fork's existing dual discipline: `encodePayload` bytes vs `.bytes.hex` and wrapped string vs `.phrase.txt`, `md/encode_test.go`.)

### M2 — APPEND `ScriptShWpkh` after `ScriptTr` (no insert/renumber). — **CLOSED.**

Task 2 line 44: "the existing `ScriptKind` enum has NO `ScriptShWpkh` value → **APPEND `ScriptShWpkh` after `ScriptTr`** (do NOT insert/renumber — that breaks `rootScriptKind`/#10b consumers)." ✓

**Source re-verification (this round):**
- `ScriptKind int` (`md/md.go:1163`) with `ScriptWpkh = iota` (0), `ScriptPkh` (1), `ScriptSh` (2), `ScriptWsh` (3), `ScriptTr` (4) (`md/md.go:1166-1170`). NO `ScriptShWpkh`. Appending → `ScriptShWpkh = 5`, leaving 0..4 byte-stable. ✓
- `rootScriptKind(t tag)` (`md/md.go:1234`) switches on the wire `tag`, NOT on the enum integer value, and is used by `summarize` (`md/md.go:1326`) — appending a never-decoded-to value cannot perturb it. `Template.Root` consumers (#10b) read the existing values, unchanged. Appending is byte-safe; inserting would renumber and break them. The plan's "do NOT insert/renumber" is the correct lock. ✓

### M3 — provenance v0.36.0 == `c85cd49`. — **CLOSED.**

Task 0 line 21 README note: "**R0-M3:** `md-codec` registry v0.36.0 == git `c85cd49`; toolkit tree SHA; the test seed; pin fp `73c5da0a`." ✓ Confirmed first-hand: `descriptor-mnemonic` HEAD == `c85cd498…` and the fork's vendored vectors derive from it. ✓

### M4 — pin fp `73c5da0a` + the `ms10entrsqqqq…` vector. — **CLOSED.**

- fp `73c5da0a`: Task 0 line 21 ("the master fingerprint (`73c5da0a` for the abandon seed)") + line 21 README pin + Task 1 line 32. LIVE toolkit `--json` confirmed `master_fingerprint: "73c5da0a"` for the abandon seed at m/84'/0'/0' (and identical across bip44/49/86 — fp is seed-derived, account-independent). ✓
- ms1 vector: Task 1 line 32 pins `EncodeMS1([16 zero bytes]) == "ms10entrsqqqq…cj9sxraq34v7f"`. LIVE Go probe: `NewSeed("ms",0,"entr",'s',[0x00‖zeros16]).String() == "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"` — byte-identical to BOTH the fork's own `entr16-zero` vector (`codex32/mspayload_test.go:25`) AND the live toolkit `ms1` output for the abandon seed (whose entropy is all-zero). The pinned vector is exact. ✓

### M5 — assert the `ms10entrs` prefix. — **CLOSED.**

Task 1 line 32: "the wire begins **`ms10entrs`** (R0-M5 — assert this exact prefix: hrp `ms`, threshold `0`, id `entr`, share `s`)". The vague "begins `ms1...`" from R0 is replaced with the exact `ms10entrs`. LIVE-confirmed prefix on both the Go probe output and the toolkit output. ✓

---

## Fold-drift check (folds can introduce drift) — NONE FOUND

I re-verified the entire wire spine the R0 PASSed, to confirm the fold did not perturb it:

- **4 AST shapes — still CORRECT.** pkh/wpkh → `keyArgBody{0}`; tr → `trBody{isNums:false, keyIndex:0, tree:nil}` (NOT keyArgBody); sh-wpkh → `node{tagSh, childrenBody{[node{tagWpkh, keyArgBody{0}}]}}`. The `cell_7_wpkh_full` fixture (`descriptor-mnemonic` `crates/md-codec/tests/wallet_policy.rs:190-204`) re-read this round: `Shared(bip84_path())`, `tree=wpkh_at_0()`, `fingerprints Some([(0,[0xDE,0xAD,0xBE,0xEF])])`, `pubkeys Some([(0, make_xpub(0x11))])` — wallet-policy with both TLVs, matching the plan's Task 2 build verbatim. Plan lines 49-50 unchanged from the R0-verified text. ✓
- **wallet-policy TLV (pubkeys+fp) + explicit Shared origin for all 4 — still CORRECT.** `is_wallet_policy` = `pubkeys Some(non-empty)` (`md-codec/encode.rs:50-52`). Plan line 48 (`pubPresent:true … fpPresent:true`) + Locked line 14 (emit explicit origin for all 4) intact; matches the toolkit's unconditional `PathDeclPaths::Shared` + always-emitted fingerprint TLV. ✓
- **`EncodeMS1` recipe — still byte-exact.** `NewSeed("ms",0,"entr",'s',[0x00‖entropy])`; `0x00` entr prefix (`codex32/mspayload.go:9`), id FIXED `"entr"`, share `'s'`. LIVE-proven this round. ✓
- **`EncodeSingleSig` signature + exported `PathComponent` — unchanged + sound.** `(chainCode [32]byte, compressedPubkey [33]byte, fp [4]byte, origin []PathComponent, script ScriptKind) ([]string, error)`; net-new exported `PathComponent{Hardened bool, Value uint32}` mapping to the unexported `pathComponent{hardened bool; value uint32}` (`md/md.go:173-176`); the R0-M5 raw-vs-in-band-hardening caveat retained (plan line 44). ✓
- **canonicalize runs at n=1 (no-op, not bypassed)** — plan line 50 "route through `encodePayload`→`canonicalize` (no-op at n=1 but DO NOT bypass)". ✓
- **comparator (fp/xpub/path/md1/ms1-entropy)** — Task 3; ms1 compared on RECOVERED ENTROPY not string; md1 comparison consistent with the chunked form. ✓
- **HEADLESS-ONLY** — Tasks add `codex32/msencode.go`, `md/encode_singlesig.go`, the comparator, tests, goldens; no GUI/program/picker. Confirmed `EncodeMS1`/`EncodeSingleSig`/`PathComponent`/`ScriptShWpkh` do NOT yet exist in the fork (grep: zero hits) — all genuinely net-new. ✓
- **picker-default refinement (BIP-84 default) correctly NOT in this plan** — Self-review P3 → T6a-2 (line 89). ✓
- **no shipped-behavior touch** — Task 4 line 77 asserts existing md/mk/codex32/ms1 + #10a/#10b + T5 tests pass verbatim; the only API surface change is the additive `PathComponent` + appended `ScriptShWpkh` (byte-stable per M2). ✓
- **TDD order (goldens before encoder)** — Task 0 (differential goldens) precedes Task 2 (`EncodeSingleSig`); each task test-first (fail→run-fail→impl→run-pass→commit). ✓

No drift. Every fold is additive precision; none weakened or contradicted an R0-verified fact.

---

## What GREEN means here

All R0 Importants (I1, I2) are CLOSED against source; all five Minors (M1–M5) are folded and source-confirmed; the fold introduced no drift; the wire spine, the ms1 recipe, the signature/`PathComponent`, the appended `ScriptShWpkh`, and the comparator are all source-verified correct. No open Critical or Important. The plan is **cleared for single-implementer TDD** in the worktree (`feat/t6a1-headless` off `e4013a8`), to be followed by the mandatory whole-diff adversarial exec review before merge. No further plan-R0 round is required.

**One sub-Minor courtesy note (non-gating, optional tidy):** Architecture line 7's one-line summary still says "calls the shipped `split`/`encodeMD1String`" — the operative Task 2 lines (46, 54) correctly lock `split` and DROP `encodeMD1String`, so this is harmless, but the implementer may delete `/encodeMD1String` from line 7 in passing for consistency. This does NOT hold the gate.
