<!--
Persisted verbatim. Architect panel (firmware/TinyGo-resource lens), agentId a096ca174eb865a95. Verdict: feasible — footprint ~free, e=0/1 ~0.5-1.9s; software SHA-256 makes high-e hours-scale. Add high-e warn-confirm + progress + off-thread; clarify byte-oriented []byte unpack for 256-bit; file hardware-SHA followup.
The text below is the agent's report exactly as returned; do not edit.
-->

# Firmware-Resource Design Review — Cycle D (SLIP-39 on-device recovery, TinyGo/RP2350)

Lens: feasibility, performance, footprint. Read-only. Sources cited inline; the load-bearing perf/hardware facts were verified against the RP2350 datasheet, Go stdlib source, and TinyGo source by a dedicated research pass, and cross-checked against an independent first-principles cycle estimate.

## 1. PBKDF2 performance — the biggest feasibility risk

**Reasoning.** Recovery is 4 Feistel-decrypt rounds, each a PBKDF2-HMAC-SHA256 of `(10000<<e)/4 = 2500·2^e` iterations → `10000·2^e` HMAC-SHA256 invocations total. With short password/salt (`[i]||passphrase` and `salt_prefix||R`, both well under 55 bytes), each PBKDF2 iteration is **exactly 2 SHA-256 block compressions** (Go's `x/crypto/pbkdf2` + `crypto/hmac` precompute and cache the ipad/opad key states — verified in `crypto/internal/fips140/{pbkdf2,hmac}`), so cost ≈ `iters · 2 · cyc_per_block / 150e6`.

The decisive fact: **the RP2350 has a real hardware SHA-256 accelerator (datasheet §12.13, 121 cycles/block, 79.3 MB/s @150 MHz), but TinyGo does not use it.** TinyGo uses the unmodified Go stdlib `crypto/sha256`; on `GOARCH=arm`/thumb (Cortex-M33) there is no assembly path (`sha256block_arm.s` does not exist), so it falls to the pure-Go `blockGeneric`. There is no TinyGo `machine` SHA driver. Pure-Go `blockGeneric` on a register-starved single-issue M33 lands at **~4,000–7,000 cycles/block (~70–110 cyc/byte)** — anchored by measured Cortex-M33 numbers (CycloneCRYPTO 70.8 cyc/B, wolfCrypt 81.2 cyc/B on STM32U5) — i.e. **~33–58× slower than the idle on-chip accelerator.**

Wall-clock (Go software band, the realistic case):

| e | total iters | time |
|---|---|---|
| 0 | 10,000 | **~0.5–0.9 s** |
| 1 | 20,000 | **~1.1–1.9 s** |
| 2 | 40,000 | ~2–4 s |
| 4 | 160,000 | ~9–15 s |
| 8 | 2,560,000 | ~2.3–4 min |
| 15 | 327,680,000 | **~5–8.5 hours** |

(My independent bottom-up estimate gave e=0 ≈ 0.2–0.9 s / e=15 ≈ 1.8–4.9 h; the discrepancy is purely the cyc/block assumption — the agent's higher, silicon-measured `blockGeneric` figure is the one to quote.)

**Responsiveness verdict.** e=0 and e=1 are the realistic SLIP-39 cases (Trezor/`shamir-mnemonic` default e=0; `mnemonic_toolkit` defaults low). Both are sub-2-second — perfectly responsive on-device. It becomes user-hostile around **e≥4 (10s+)**, multi-minute at **e≥8**, and effectively a denial-of-service at **e=15 (hours)**. Note the operator does not choose e at recovery time — it is read from the share header, so a (hostile or merely high-cost) backup can pin the device for hours.

**Recommendation (concrete stance):**
1. **Ship software PBKDF2 (no hardware driver) for v1.** It is correct, dependency-free, and fine for the realistic e≤1 cases. Do not block Cycle D on a hardware SHA driver.
2. **Show progress + run off the UI thread.** The Feistel decrypt must run with a "Recovering… (round k/4)" indicator and the watchdog fed; do not freeze the GUI for even 1–2 s silently. The `-scheduler tasks` build (flake.nix) makes this straightforward.
3. **Warn-and-confirm on high e, do not silently cap.** Capping e changes the cryptosystem (a real backup with high e would become unrecoverable), so a hard cap is wrong. Instead: when the parsed `IterationExp` implies a long wait, show the estimated time and require explicit confirm (e.g. ">~10 s at e=4", "~hours at e=15"). This is a UX/safety mitigation, not a crypto change.
4. **File a follow-up to add an RP2350 hardware-SHA `machine` driver** for the Feistel round function. It would cut e=15 from ~5–8.5 h to **~9 min** (530 s @121 cyc/block) and e=0 to tens of ms. Worth it only if high-e backups are a real target; out of scope for v1. (Caveat: ACCESSCTRL defaults the SHA block to Secure-Privileged, single-instance with bootrom `LOCK_SHA_256` — a driver has real integration cost.)

## 2. Binary/RAM footprint

**Reasoning.** Two parts: the in-tree crypto port, and the crypto-library link closure.

- **Link closure: nothing new.** `go.mod` already pins `golang.org/x/crypto v0.52.0` as a direct dependency, and `bip39/bip39.go` already calls `pbkdf2.Key(..., 2048, 64, sha512.New)` for the 25th-word seed and `math/big` in `bip39.New`. So `crypto/sha256`, `crypto/hmac`, `golang.org/x/crypto/pbkdf2`, and `math/big` are **already linked and exercised on-device**. The SLIP-39 port adds `pbkdf2.Key(..., sha256.New)` — `crypto/sha256` is pulled by the existing seedqr/HD paths regardless, so this is **~0 new flash from dependencies**. The spec's §5 invariant "no `math/big` in the SLIP-39 crypto" is achievable as designed — GF(256) is pure byte-table arithmetic (confirmed against `gf256.rs`); `math/big` enters only the existing `bip39.New(entropy)` step, which the spec correctly carves out.
- **In-tree port:** I read all the oracle source. The Go port is ~600 LoC: `gf256.go` (~70, plus a **512 B** RAM cost for `expTbl[256]+logTbl[256]` built in `init()`), `lagrange.go` (~70), `feistel.go` (~80), `combine.go` (~200), `share.go` extension (~120). This is squarely in line with the in-tree GF(32) precedent: `codex32/gf32.go` is ~145 LoC with ~223 B of tables and the whole `codex32` package is the ~1,100 LoC the recon cited. Estimated added flash for the new logic: **low single-digit KB** (no new symbols, just more code over already-linked primitives). RAM: the 512 B tables + transient share/EMS buffers (≤32 B each) on a **16 KB stack** (`-stack-size 16kb`) — comfortable, but see the stack note in Q3.

This is dramatically better than the rejected `go-slip39` (~55 KB of Unicode tables that upstream SeedHammer explicitly disabled SLIP-39 over, plus `math/big`+gonum+golang-set+x/exp). The port adds roughly **two orders of magnitude less** footprint than vendoring.

**Recommendation.** Footprint is a non-issue. Confirm post-build with `tinygo build -size full` and diff against the pre-Cycle-D binary; budget expectation is low-single-digit KB flash + ~0.5 KB static RAM. No dependency review needed — everything is already in the closure.

## 3. TinyGo correctness gotchas

**Reasoning.**
- **`int` is 32-bit.** The spec keeps all multi-word bit assembly byte-oriented/`uint64`, and the existing `slip39/share.go` already does exactly this (`hdr := uint64(indices[0])<<30 | ...` with the explicit "uint64 is REQUIRED: on RP2350/TinyGo int is 32-bit" comment). For the new share-VALUE unpacking of up to 33 words (256-bit secret), a 256-bit value cannot live in a `uint64` — but the oracle (`share.rs::decode_value`) does **not** use a wide accumulator; it pulls one bit at a time with a `get_bit(i)` helper and packs MSB-first into a `[]byte`. That is fully 32-bit-safe and is the pattern to port. **Verify the Go port packs bit-at-a-time into `[]byte` (not into a `uint`/`uint64` accumulator)** — the spec §4.6 wording "shift into a running `uint` accumulator" is slightly risky phrasing; a per-byte (8-bit) accumulator is fine, a value-wide accumulator is not. This is the one place to watch.
- **`init()`-built tables:** fine on TinyGo. `init()` runs at startup; the codex32 package already uses package-level table vars. The gf256 `init()` loop is trivial.
- **`crypto/*` API on TinyGo:** `crypto/sha256`, `crypto/hmac`, `x/crypto/pbkdf2` all compile and run on TinyGo (proven — `bip39.MnemonicSeed` uses `pbkdf2.Key(...sha512.New)` today). No API in the port that compiles on host Go 1.25 but not TinyGo. The go1.26/`text.Style.Measure`/vet notes are GUI-only and do not touch this crypto.
- **Host-test vs TinyGo-build divergence (the Slice-1 lesson):** the crypto is pure computation with no `machine`/syscall/unsafe surface, so `go test ./slip39/` on the host exercises byte-identical logic to the TinyGo build. The divergence risk is low here — **except** the one real gap: **host SHA-256 uses the amd64 assembly path and is ~50× faster, so timing/responsiveness CANNOT be validated by `go test`.** The host green-bar tells you nothing about on-device wall-clock. Any e-cost UX threshold (Q1.3) must be validated on real hardware or against the cycle model, not inferred from host test speed.

**Recommendation.** Crypto is TinyGo-clean. Two test guards to add: (a) a vector that exercises the 33-word/256-bit value unpack path explicitly (33 words don't fit a `uint64` — pins the bit-at-a-time invariant); (b) since host timing is meaningless, add a comment/follow-up to bench the Feistel decrypt once on real RP2350 to confirm the e=0/1 sub-2s estimate.

## 4. Is the port the right engineering call (firmware-resource view)?

**Yes, decisively.** From a pure footprint/perf/maintainability angle:
- **Footprint:** ~0 new dependencies (all crypto primitives already linked), ~600 LoC + 512 B tables, vs. ~55 KB + `math/big`+gonum for the only Go library — which upstream already rejected on exactly this basis.
- **Perf:** identical to any alternative — they'd all hit the same pure-Go `blockGeneric` SHA. No library choice changes the PBKDF2 wall-clock.
- **Maintainability:** the Rust oracle (`mnemonic_toolkit::slip39`, ~1,900 LoC, already gate-reviewed) gives byte-for-byte TDD cross-checks against the official `vectors.json`; the firmware already has the GF(32)/RS1024 precedent so the idioms are established. The port is ~600 LoC of straight-line table/loop math with no allocation-heavy or platform-specific code.

The only alternative that would change the perf story is a hardware-SHA driver, which is orthogonal to "port vs vendor" and can be added later under the same port. Port is correct.

## 5. Top-3 feasibility/perf risks (ranked)

1. **High iteration-exponent DoS (e≥8 → minutes-to-hours).** A backup (or a malicious/typo'd share set passing the digest gate, or a high-e legitimate backup) can pin the device for hours on software SHA-256. **Mitigation:** read `IterationExp` from the header before starting, show an estimated-time warning + explicit confirm above a threshold, run the decrypt off the UI thread with progress and watchdog feeding. Do NOT hard-cap e (breaks recoverability of real high-e backups). File a follow-up for an optional RP2350 hardware-SHA driver (cuts e=15 from hours to ~9 min).
2. **256-bit value unpack overflowing a 32-bit/`uint64` accumulator.** Spec §4.6's "running `uint` accumulator" phrasing invites a value-wide accumulator that silently breaks on 33-word shares under TinyGo (`int` 32-bit). **Mitigation:** port the oracle's `get_bit`/per-byte packing verbatim into `[]byte`; add an explicit 33-word (256-bit) round-trip test vector. (The existing header decode already proves the team knows the `uint64` discipline — apply it here too.)
3. **Host-test green ≠ on-device validated (responsiveness + the panic-safety gate).** `go test` runs amd64-asm SHA ~50× faster, so it validates correctness but not timing; and the §4.4 panic-safety requirement (no malformed/duplicate-index share set may reach `gfDiv`/interpolation panics) is a real implementation-introduced regression risk TDD can miss. **Mitigation:** treat the §4.4 panic-safety negative tests as mandatory (matching the Cycle-B `ConsistentShares` precedent), and validate e=0/1 wall-clock against the cycle model or real hardware, not host test speed. This is squarely what the project's mandatory post-implementation adversarial whole-diff review should catch.

---

## Overall verdict

**Feasible as designed**, with one spec clarification and one UX addition (neither blocks the R0 gate; both are folds):
- Footprint is essentially free — all crypto primitives (`sha256`/`hmac`/`pbkdf2`/`math/big`) are already in the link closure (`go.mod` x/crypto v0.52.0; `bip39.go` already calls pbkdf2 and big.Int), the port is ~600 LoC + 512 B, two orders of magnitude under the rejected `go-slip39`. The "no math/big in SLIP-39 crypto" invariant holds.
- Performance is responsive for the realistic e=0/1 cases (**~0.5–1.9 s**), but software-only SHA-256 makes high-e backups **hours-scale** (RP2350 has a 121-cyc/block hardware SHA accelerator that TinyGo leaves idle — a ~33–58× gap). **Add: header-driven time estimate + confirm for high e, progress UI, off-thread decrypt. Clarify §4.6 to mandate byte-oriented `[]byte` packing (not a value-wide accumulator) for the 256-bit unpack.**

Key file references: spec `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_slip39_recovery.md`; oracle `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/slip39/{gf256,lagrange,feistel,share}.rs` and `mod.rs`; firmware `/scratch/code/shibboleth/seedhammer/{go.mod,flake.nix,slip39/share.go,bip39/bip39.go,codex32/gf32.go}`.
