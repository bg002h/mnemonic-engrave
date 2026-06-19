<!--
Persisted verbatim. opus-architect MANDATORY whole-diff adversarial execution review of the T2c md1
single-string decode implementation (worktree seedhammer-wt-t2c-md1, branch feat/md1-decode-display,
5 signed+DCO commits over 2fed9b6: da7dd43 codex32.MDDataSymbols, 318ab2a md bit reader, b48ea6f decode
core/AST, 6d59693 validators+Decode/Template, 65eb700 GUI md1 flow), BEFORE merge. Reviewer agentId
a5cba8d0274edf2d1. Verdict: NOT GREEN — 0C/1I. The decode logic is a FAITHFUL, PANIC-FREE port: ~150M
fuzz execs across Decode/decodePayload/readNode/readTLV/readVarint/bitReader/symbolsToBytes with ZERO
panics; faithfulness confirmed vs md-codec 0.36.0 source + 9-vector parity re-proved against live
tests/vectors/*; OriginPath surfaces the decoded path (not canonical-implied); Multi/SortedMulti
distinguished; alloc gate + mk1 path intact; deps = codex32+stdlib only; 5 commits signed+DCO+authored.
The 1 IMPORTANT is a TEST-COVERAGE GAP (spec §6): the negative + Renderable=false-on-decode +
tr(NUMS,…)/explicit-origin-sh(multi) tests are absent (the plan's TestDecodeNegativeAndRenderable shipped
as nothing), so the reject/classify/is_nums-cursor paths have no regression guard. MINOR-1: bits.go read()
dropped Rust's bit_limit<=len*8 precondition (panics OOB on decodePayload([]byte{},64), proven UNREACHABLE
from Decode). MINOR-2: tr(@N,multi_a) classified Renderable=true — spec §4.2 wording ambiguity (impl matches
the plan; faithful-not-approximated). Worktree left as-is, clean, no merge/push. Disposition: fold
IMPORTANT-1 (add the tests) + MINOR-1 (harden read) + clarify MINOR-2 → re-review → merge. Text below
verbatim (entities un-escaped: &lt;→<, &gt;→>, &amp;→&).
-->

# ADVERSARIAL WHOLE-DIFF EXECUTION REVIEW — feat/md1-decode-display @ 65eb700

## Verification Results (re-run, not trusted)

**Test suite** (`go1.26.4`, `/home/bcg/.local/go/bin/go`):
- `go test -count=1 ./codex32/ ./md/ ./gui/` → all `ok` (codex32 0.002s, md 0.001s, gui 13.6s)
- `go test -count=1 ./...` → all packages `ok`, zero failures
- `go vet ./codex32/ ./md/ ./gui/` → clean (no output)
- `gofmt -l codex32/ md/ gui/` → empty (clean)
- `go test -count=1 -run TestAllocs ./gui/` → `ok` (invariant 2.8 intact)
- Targeted verbose: `TestDecodePayloadAST` (8 subtests), `TestDecodeParity` (9 subtests), `TestDecodeChunkedRefused`, all md1 GUI tests — all PASS.

**Parity re-proved against live `tests/vectors/*` (≥3 required, did 7):** `wpkh_basic`/`pkh_basic`/`wsh_sortedmulti`/`tr_keyonly`/`sh_wsh_multi`/`wsh_with_fingerprints`/`wsh_divergent_paths` — committed `phrase.txt` strings, `bytes.hex` (AST test), and `descriptor.json` (n/tree/tlv: fingerprints `@0=deadbeef,@1=cafebabe`; divergent override `@1=<2;3>/*`) all match the committed test expectations verbatim. `TestDecodeChunkedRefused` constant `md1fz4awqqpqsgqpsgvyyxqql8saf74dwdyqv` matches live `wsh_multi_chunked.phrase.txt` line 2 and returns `ErrChunkedUnsupported`.

**Faithfulness vs md-codec 0.36.0** (read all of `bitstream.rs`/`varint.rs`/`header.rs`/`tag.rs`/`tree.rs`/`tlv.rs`/`origin_path.rs`/`use_site_path.rs`/`decode.rs`/`validate.rs`/`canonical_origin.rs`): bit reader (MSB-first chunk/shift/mask, saturating `remaining`), varint LP4-ext (extension max 536870911 = 2^29-1, verified to fit u32 with no overflow), header (version==4 reject), tag (0x00–0x23 + 0x3F-ext-consume-4-reject + reserved reject), `readNode` (depth `>=128` pre-check, identical per-tag arms, k/count `+1` and `k>count` reject), the 5 validators, `kiw = 32-LeadingZeros32(n-1)`, `decodePayload` field+validator ORDER, and `canonicalOrigin` 5-shape table — **all faithful**. `OriginPath` surfaces the DECODED path ("m" for elided), never the canonical implied path. `Multi`/`SortedMulti`/`MultiA`/`SortedMultiA` distinguished. `errors.Is(err, ErrChunkedUnsupported)` matched in GUI.

**Fuzzing** (scratch `Fuzz*` in `md/`, run then deleted — `git status` clean, no `testdata/` or `zz*` leaked):
- `FuzzDecodeString` (public entry): 31.4M execs, **0 panics**
- `FuzzDecodePayload`/`Validated` constrained to production invariant `bitLen∈[0,len(b)·8]`: 38.4M execs, **0 panics**
- `FuzzReadNode`: 30.0M execs, **0 panics** (recursion bounded, no stack blowup)
- `FuzzReadTLV`: 30.9M execs, **0 panics**
- `FuzzReadVarint` 22.8M, `FuzzBitReader` 22.8M, `FuzzSymbolsToBytes` 17.2M — **0 panics**
- Probe: `read(count>64)` does not panic (returns garbage at 65, truncates beyond); production max read is 32, so out-of-contract but unreachable.

**One panic input found** (then proved unreachable from production): `decodePayload([]byte{}, 64)` → `index out of range [0]` at `bits.go:56` via `readHeader`. Cause: `newBitReader` accepts `bitLimit > len(bytes)·8`; `read()` checks only `remaining()` (against `bitLimit`), not the slice length — Rust guards this with `debug_assert!(bit_limit <= bytes.len()*8)` which the Go port dropped. I proved `5·len(syms) <= len(symbolsToBytes(syms))·8` for n∈[0,200] and confirmed `Decode` (line 1211) is the **only** production caller, always passing `bitLen=5·len(syms)`. So unreachable from any real input.

## Findings

**MINOR-1 — Dropped bit-reader precondition guard (defense-in-depth).** `md/bits.go:32,46-56`: `newBitReader`/`read` trust `bitLimit` without asserting `bitLimit <= len(bytes)*8` (Rust has `debug_assert!`). With a violated precondition (`decodePayload([]byte{},64)`) `read` panics OOB instead of returning `errTruncated`. NOT reachable via `Decode` (the sole caller's invariant holds, proven). Concrete fix: in `read`, replace `if r.remaining() < count` with a guard that also bounds against the slice, e.g. compute `availBits := min(r.bitLimit, len(r.bytes)*8)` and check against that — restoring the Rust invariant as a hard check rather than a debug-only assert.

**MINOR-2 — `tr(@N, multi_a/sortedmulti_a)` classified `Renderable=true` — spec §4.2 ambiguity.** `md/md.go:1248-1251`: a taproot with a single multi_a leaf returns `PolicyMultiA`/`PolicySortedMultiA` (renderable, GUI shows "N-of-M multisig (tapscript)"). Spec §4.2 line 81 restricts the renderable `Tr` to keyspend-only and line 82 lists "taptree branches"/"NUMS-internal-key taproot" under `Renderable=false`; but line 80's `<multi-family>` set explicitly includes `MultiA/SortedMultiA`, and the plan (line 424) + Template `PolicyKind` + GUI `policyLine` deliberately define these as renderable policies. The implementation matches the plan that passed R0 and the policy claim is faithful (not approximated), so I rate this a documentation/spec-wording ambiguity, not a behavioral defect. Note: the `b.tree==nil → PolicySingle` branch is safe for `is_nums=true` only because `validatePlaceholderUsage` rejects a NUMS-keypath-only `tr` (n≥1, no `@i` referenced) before `summarize` — a correct-but-non-local invariant; a defensive `if !b.isNums` in `classifyPolicy` would make it locally robust.

**IMPORTANT-1 — Negative + Renderable-classification + `tr(NUMS,…)` test coverage gap (spec §6 lines 104-105 mandate).** The committed `md` tests are entirely positive: `TestBitReader`, `TestDecodePayloadAST` (8 valid AST), `TestDecodeParity` (9 valid Templates), `TestDecodeChunkedRefused` (the *only* negative — chunked flag), `TestMDDataSymbols`. The plan's `TestDecodeNegativeAndRenderable` is **absent entirely** (not even a commented skeleton). **No committed test exercises any of these ship-critical reject/classify paths:**
  - The 5 validators' reject arms: `MissingExplicitOrigin`, `PlaceholderNotReferenced`, `PlaceholderFirstOccurrenceOutOfOrder`, `MultipathAltCountMismatch`, `ForbiddenTapTreeLeaf`, `NUMSSentinelConflict`, `InvalidXpubBytes` (last is a deliberate no-op in Go).
  - Structural rejects: wire-version≠4, reserved/`0x3F` tag, non-canonical root (`OperatorContextViolation`), K>N, placeholder-index-OOB, TLV length-overflow/ordering/empty, depth>128, truncation, >7-bit trailing padding.
  - A real **valid-but-complex md1 decoded to `Renderable=false`** (e.g. explicit-origin `wsh(and_v(...))`). `TestMD1DisplayFlowComplexRefuses` builds a `Template{Renderable:false}` *literal* — it does NOT decode a wire, so the decode-side `Renderable=false` classification path ships unverified.
  - The §2.13 / spec-line-105 mandated **`tr(NUMS,…)` parity vector** (the `is_nums` variable-width-cursor branch — the "highest-fragility" surface per the spec) and an **explicit-origin `sh(multi)`** renderable case.

  The decode logic is correct (faithful port confirmed by source read + 9-vector parity + ~150M fuzz execs with zero panics), but the cycle ships reject/`Renderable=false`/`tr-is_nums` paths with **no regression guard**. Recommend adding `TestDecodeNegative` (table of constructed-byte or md-codec-round-trip negative payloads asserting rejection per category, error-class not string-equality per spec line 104) + `TestDecodeRenderableFalse` (decode a real explicit-origin `wsh(and_v(...))` → `Renderable=false`, refusal copy, no policy claim) + a `tr(NUMS, multi_a(2,@0,@1,@2))` round-trip vector (sourced from md-codec encode, not the corpus) asserting the cursor decodes correctly.

## Scope / secrecy / provenance
- `go list -deps seedhammer.com/md` → only `seedhammer.com/codex32` + stdlib (no btcec/bip380/btcd). Confirmed.
- No `wipeBytes`/`Unshared`/secret handling (md1 public); `md1DisplayFlow` has zero engrave/NFC/NDEF/plate/mutation calls (grep confirms only the doc comment). Read-only (§2.6) holds.
- `mdmkFlow`: `idx--` applied for BOTH mk1 and md1 (`if inspect`), both prepend exactly one Inspect entry → `engravings[idx]` never out of range. mk1 path byte-identical (TestMdmkFlowMK1ShowsInspect passes). Both `errors.Is(ErrChunkedUnsupported)` and generic-error GUI messages reachable.
- `git diff --stat`: only the 11 manifest files (+2010/−22), nothing outside.
- 5 commits: SSH-signed (signature blob present; local `sig=N` is only the missing `allowedSignersFile` verifier), authored Brian Goss, DCO `Signed-off-by` + `Co-Authored-By` trailers on every commit.
- Worktree left as-is, clean, unmodified. No merge/push/commit performed.

## Verdict

**NOT GREEN — 0C/1I**

(IMPORTANT-1: spec §6-mandated negative + `Renderable=false`-on-decode + `tr(NUMS,…)`/explicit-origin-`sh(multi)` tests are absent; the reject/classify/`is_nums`-cursor paths ship without a regression guard. MINOR-1 and MINOR-2 are non-blocking. The decode logic itself is a faithful, panic-free port — the gap is test coverage, exactly the class this review exists to catch.)
