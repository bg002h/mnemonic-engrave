<!--
Persisted verbatim. opus-architect MANDATORY whole-diff adversarial execution review of the T2b
mk1-decode implementation (worktree seedhammer-wt-t2b-mk1, branch feat/mk1-decode-display, 6 signed+DCO
commits over 4d02021: f04c572 codex32.MKDataSymbols, c1f1fc8 mk header+fiveBitToBytes, cb8eff9 mk.Decode,
fa9ae81 mk1Gatherer, 9054f2e mk1DisplayFlow, 6cf77a8 mk1GatherFlow+mdmkFlow wire), BEFORE merge.
Reviewer agentId a6e1234521627dc08. Verdict: GREEN 0C/0I. Independently re-ran the full suite uncached,
FUZZED Decode/Bytecode/FiveBit/MKDataSymbols (~31M execs total, ZERO panics), re-proved V1/V3/V5/V7
parity vs corpus byte-for-byte, confirmed R0-C1 (chunk_index verbatim) + R0-M1 (raw hardened childNum)
in the real reassembly code (not just tests), verified the scanner goroutine/NFCReader lifecycle has no
leak/deadlock on any return path, display flow has no engrave/NFC/mutation and pages to the xpub tail
gap-free, mdmkFlow md1 path byte-identical with no engravings out-of-range, alloc gate intact, scope
clean (7 manifest files only), all 6 commits signed+DCO+authored Brian Goss with the Co-Authored-By
trailer. Disposition: GREEN — merged no-ff to fork main, pushed bg002h. The text below is the agent's
report verbatim (HTML entities un-escaped from the notification transport: &lt;→<, &gt;→>, &amp;→&).
-->

# ADVERSARIAL WHOLE-DIFF EXECUTION REVIEW — T2b mk1 decode→display

**Branch** `feat/mk1-decode-display` @ `6cf77a8` (base `4d02021`), 6 commits, +1139 LOC across the 7-file manifest.

## Verification Results (re-run, not trusted)

- **Full suite uncached** (`go test -count=1 ./codex32/ ./mk/ ./gui/ ./...`): **ALL PASS**. Every package in the repo green; the three target packages green on a final clean re-run (codex32 0.011s, mk 0.002s, gui 8.735s).
- **`go vet ./codex32/ ./mk/ ./gui/`**: clean (no output).
- **`gofmt -l codex32/ mk/ gui/`**: empty (formatted).
- **Alloc gate** (`go test -run TestAllocs ./gui/`): `--- PASS: TestAllocs (1.52s)` — invariant 2.6 intact.
- **Parity vs corpus** (`v0.1.json`, sha256 `ebd8f34d…78ff`, matches the pinned `tests/vectors.rs:41`; `family_token "mk-codec 0.2"`, schema 2): V1, V3 (testnet), V5 (explicit 4-comp), V7 (max 10-comp) all match the corpus `input` block byte-for-byte (network, path, fingerprint, stubs, xpub). I re-derived depth/childNum from the produced xpubs: V1/V5 depth=4, V7 depth=10, raw hardened childNum `0x80000002`/`0x80000007`/`0x80000009` — **R0-M1 (raw BIP-32 hardened-bit u32) confirmed in real code**.
- **R0-C1 (chunk_index verbatim) confirmed in real reassembly**, not just the header test: the long V1 chunk parses to index 0, short to index 1; `Decode` succeeds in BOTH input orders (slots[]-indexed determinism). A `+1` injection would mis-slot and fail the cross-chunk hash.
- **Adversarial fuzzing** (scratch tests in `/tmp/mkfuzz`, since deleted): `FuzzDecode` (~4.3M execs, 150 interesting), `FuzzBytecode` (~8.5M, 37), `FuzzFiveBit` (~8.4M), `FuzzMKDataSymbols` (~10M, 55) — **zero panics**. `decodeBytecode`/`decodePath`/`readLEB128`/`reconstructXpub`/`fiveBitToBytes` exercised directly with arbitrary bytes.
- **Uppercase round-trip**: an all-uppercase mk1 set decodes to the identical xpub (bech32 case-tolerance holds through `MKDataSymbols`→`feFromRune`; `hasMKPrefix` accepts "MK1").
- **Commits**: all 6 carry an SSH `gpgsig` header (the `%G?`=N was only this sandbox's missing `allowedSignersFile`, not unsigned), authored `Brian Goss <goss.brian@gmail.com>`, with both `Signed-off-by:` (DCO) and `Co-Authored-By: Claude Opus 4.8 (1M context)` trailers.
- **Scope**: `git diff --name-only` = exactly the 7 manifest files; no secret handling, no `wipeBytes`/`Unshared` (only a comment asserting "no secret handling" — correct for public mk1, invariant 2.7).

## Adversarial findings against the real code

- **Protocol fidelity** — `fiveBitToBytes` is a line-for-line transcription of `bch.rs:78-100` (symbol≥32 reject, `bits>=5` leftover reject, non-zero pad reject). Header decode matches `header.rs:124-163` (`total = wire+1`, `chunk_index` verbatim; `total==0`/`>32` impossible in both Rust and Go after the `&0x1F`+1, so the Go check is semantically equivalent, not a gap). Bytecode header masks match `header.rs` exactly (`FINGERPRINT_FLAG_MASK 0x04`, `RESERVED_MASK 0x0b`, `version = byte>>4`, `VERSION_SHIFT=4`). The 14-entry `standardPaths` table (incl. the v0.2.0 `0x16`→m/48'/1'/0'/1') matches `STANDARD_PATHS`. `readLEB128` matches `leb128_decode_u32` (shift≥35 bail, `>u32::MAX` reject); `count>10`→PathTooDeep; `count==0`→empty path→"m".
- **No panic paths** — `decodeBytecode`'s `read(n)` guards every slice (`cur+n > len(b)` → `errUnexpectedEnd`); `reconstructXpub` guards `len(compact) != 73`; `MKDataSymbols`' `data[:len(data)-checksum]` is safe because `ValidMK` gates `len(data)` ≥ the checksum length in every bracket; the `inputData` `panic("assert")` is unreachable (MKDataSymbols does its own symbol extraction). `stub_count`/`count` are read from single bytes (≤255/≤10) so no over-allocation; `total_chunks` capped at 32.
- **Cross-chunk hash genuinely reached** — `reassemble` computes `sha256(stream[:split])[:4]` vs `stream[split:]`; corpus N11 in the suite proves a mismatch is rejected. Map iteration in `collected()` is order-irrelevant because `reassemble` re-slots by verbatim `ChunkIndex`.
- **Goroutine/reader** — `mk1GatherFlow`'s scanner goroutine is a verbatim copy of the proven `StartScreen.Flow` idiom: deferred `close(closer); r.Close(); <-closed` runs on every return path (Back, complete, decode-error, ctx.Done); `r.Close()` unblocks the `s.Scan` read so the goroutine reaches `<-closer` and `close(closed)`. The `scans <- scan` cannot deadlock (it drains the 1-buffer first, then sends into the now-empty buffer — never waits on a consumer). No leak, no deadlock.
- **Display safety/paging (2.4/2.10)** — no engrave/NFC/plate/stepper call anywhere in `mk1_inspect.go` (only docstring comments). Paging is structurally identical to the merged, tested `ms1_decode.go`: the `i > start` guard renders the first line of every page unconditionally → `shown ≥ 1` → `start` strictly advances and cannot stick; wraps to 0 at the tail; no infinite loop. `TestMK1DisplayFlowPaging` confirms the 111-char xpub tail (`1hM7vFrc`) is reached.
- **`offer` priming** — single-string first chunk (total=1) primes and `complete()` is immediately true → defensive single-string path works. A foreign single-string offered mid-set hits `!h.Chunked` → `gatherForeign`, never mis-added.
- **`mdmkFlow` wiring** — `validateMdmk` appends labels/engravings in lockstep, so `len(labels)==len(engravings)`; mk1 `idx = choice-1` ranges `0..len-1` (choice 0 is the Inspect branch, which `continue`s and never indexes `engravings`) — no out-of-range. md1 path (`isMK=false`) is byte-identical: unchanged title/lead/choices, `idx := choice`. Returning from Inspect `continue`s the chooser loop with no state mutation. `TestMdmkFlowMD1NoInspect` confirms md1 shows no Inspect.

No Critical, Important, or Minor defects found.

## Verdict

**GREEN — 0 Critical / 0 Important**

The worktree and branch are left untouched (clean `git status`, scratch files deleted); nothing committed, pushed, or modified. Ready for disposition (merge no-ff → push `bg002h`).
