# Adversarial verification — D6-6 (md1 chunked/single discriminator + md-codec-0.36 drift guard)

Verifier #0. Verdict: **REFUTED** (funds-safety severity not substantiated; residual is a low-severity test-hygiene nit, not an `important` lost-funds finding). Confidence: **high**.

## The claim

`bundle.rs:144-167`'s chunked/single discriminator (`symbols.first() & 0x01`, then `ChunkHeader::read`) has only two hand-built fixtures, leaves the `ChunkHeaderChunkedFlagMissing` and `WireVersionMismatch→Md1WireVersion` arms untested, and has no drift guard. Failure scenario: a masked-bit mutation (`& 0x03`) or an md-codec 0.36.x patch misclassifies **one chunk of a multi-chunk md1** as `Md1Single`; it is admitted as a standalone bch-only "complete" plate, the set-completeness check is bypassed, the remaining chunks are silently dropped, and the user engraves an incomplete backup that looks complete → lost funds.

## What is TRUE (the honest, weak residual)

- The `ChunkHeaderChunkedFlagMissing` arm (bundle.rs:160-162) is effectively dead / unreachable with the current fixtures: line 149's `if !chunked_flag { return Md1Single }` short-circuits every bit0==0 case *before* `ChunkHeader::read` is ever called, so the only way to reach line 160 is bit0==1 while `read` still reports the flag missing — contradictory. It is defensive.
- The `WireVersionMismatch→Md1WireVersion` arm (163-164) has no fixture (no bit0==1 / wrong-version string).
- No test *directly* asserts the documented 0.36 quirk ("single md1 → `ChunkHeader::read` returns `WireVersionMismatch{got:2}`").
- `Cargo.toml` uses `md-codec = "0.36"` (caret), so a `cargo update` could pull a 0.36.x patch; `Cargo.lock` pins `0.36.0` (checksum `75b1bfb…`) so it requires a deliberate update.

These are real but minor test-hardening items. They do **not** support the funds-loss severity, for the reasons below.

## Why the funds-loss failure scenario is NOT reachable

### 1. Authoritative bit layout (from md-codec 0.36.0 source)
`~/.cargo/registry/.../md-codec-0.36.0/src/chunk.rs`:
- Lines 3-4: first-symbol MSB-first is `[v3][v2][v1][v0][chunked]` → the chunked flag is **bit 0**.
- Line 52: chunk encoder writes `chunked = 1`.
- Line 606: md-codec's OWN discriminator is `symbols.first().map(|s| s & 0x01)`.
So `bundle.rs:147`'s `sym & 0x01` mirrors md-codec exactly. Any genuine chunk has bit0 = 1.

### 2. The cited escaping mutation (`& 0x03`) is behaviorally equivalent — it cannot misclassify a chunk as single
Empirical probe (standalone crate in `/var/tmp`, live md-codec 0.36.0, split of the same 6-key descriptor the repo test uses):
```
SINGLE   first5 = 00100 (0x04)  bit0=0  &0x03=0  &0x02=0
CHUNK[0] first5 = 01001 (0x09)  bit0=1  &0x03=1  &0x02=0
CHUNK[1..3] identical = 01001
ALL chunks bit0==1 (never misread as single by &0x01): true
ANY chunk &0x03==0 (would &0x03 misread it as single?):  false
```
Because every chunk has bit0=1, `sym & 0x03` (= bit0|bit1) is **always non-zero** → still classified as chunked. And the single fixture (0x04) gives `&0x03 == 0` → still single. So `& 0x03` produces *identical* funds-relevant classification to `& 0x01` on both real chunks and the single. The finder's own M4 note concedes `& 0x03` "escapes" the suite — but an escaping mutant that never misclassifies anything in the dangerous direction is not a funds risk. The `& 0x03` mask can only ever move a *single→chunked* (bit1==1 singles), which yields a false **rejection** (`Md1WireVersion`, exit 4), never a silently-accepted incomplete backup.

### 3. The mask that WOULD misclassify a chunk as single (`& 0x02`) is CAUGHT
The probe shows chunks have bit1 = 0 (`0x09 & 0x02 == 0`). So `& 0x02` → chunked_flag=false → chunk misread as single. But `md1_chunked_set_verifies_and_drop_fails` (bundle.rs:502) regenerates chunks from `md_codec::chunk::split` and asserts a `Kind::Md1` `SetVerified` set forms; under `& 0x02` all chunks become singles → no md1 group → that assertion fails. Confirmed the test is live and passing against md-codec 0.36.0 (`cargo test -p mnemonic-engrave --lib -- bundle::tests` → 16 passed).

### 4. This same live-regenerating test is a de-facto md-codec drift guard
`chunked_md1_vector()` does not hand-encode symbols — it calls the current `md_codec::chunk::split`. If a 0.36.x patch flipped the chunk wire format so real chunks read bit0=0 (the only way to drive chunk→single), `md1_chunked_set_verifies_and_drop_fails` would fail loudly (no verified set). So the *dangerous* direction is guarded; only the harmless quirk-pin (single→WireVersionMismatch) lacks a direct assertion.

### 5. Even a hypothetical single-chunk misclassification errors out, not silent
The scenario says "one chunk misclassified → remaining chunks silently dropped." Not so: if one member of an N≥2 set is misclassified as `Md1Single`, the remaining N-1 land in `md1_groups`, and `run_bundle` calls `md_codec::chunk::reassemble(&refs)` on them (bundle.rs:246). An incomplete group fails → `SetIncompleteMd` → exit 4, **no manifest** (`bundle_dropped_chunk_exit_4_no_stdout`). This is exactly what the drop-half of `md1_chunked_set_verifies_and_drop_fails` proves. A silently-complete-looking backup requires **all** chunks to misclassify simultaneously — which needs a wholesale wire flip, itself caught by §4.

## Severity assessment

The funds-loss outcome ("admits a set member as a complete plate → incomplete unrecoverable backup that looks complete") is **not concretely substantiated**: the one mutation that escapes the suite is behaviorally inert, the mutation that would bite is caught, a partial set errors out via `reassemble`, and md-codec drift on the dangerous axis breaks a live-regenerating test. What remains is a genuine but **low** test-hygiene gap (two untested defensive/dead error arms + no direct assertion pinning the 0.36 single-md1 quirk + a caret dep). That is worth a small P6-style hardening test, but it is not an `important` funds-safety finding.

## Evidence
- md-codec 0.36.0 `chunk.rs` lines 3-4, 52, 606 (bit0 = chunked flag; md-codec's own `& 0x01`).
- `cargo test -p mnemonic-engrave --lib -- bundle::tests` → 16 passed (incl. `md1_chunked_set_verifies_and_drop_fails`, `parses_unchunked_md1_as_bch_only`).
- Standalone probe (outside repo, `/var/tmp/d6verify/probe`) printing first-symbol bits: SINGLE 0x04, CHUNKS 0x09; `&0x03` never 0 for chunks.

Verdict: **refuted = true** (funds-safety claim unsubstantiated; residual is low-severity test hygiene).
