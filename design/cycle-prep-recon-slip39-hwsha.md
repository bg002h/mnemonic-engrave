# cycle-prep recon — 2026-06-18 — seedhammer-slip39-hwsha

**Fork `main` SHA at recon time:** `9db3fd2` (Cycle D shipped).
**Design repo HEAD:** `323aad0`.
**Slug:** `seedhammer-slip39-hwsha` — use the RP2350 hardware SHA-256 accelerator for SLIP-39's
PBKDF2-HMAC-SHA256 Feistel round function (today: pure-Go software SHA under TinyGo), to make
high-iteration-exponent recovery fast.

Recon = four parallel agents (RP2350 SHA hardware facts; ACCESSCTRL/LOCK go-no-go; firmware
driver pattern + Go integration; — perf/threat already covered by the Cycle-D firmware-resource
panel). All hardware/protocol facts verified against the **RP2350 datasheet §12.13 / §10.6 /
§5.4** and the **pinned TinyGo `cmsis-svd` SVD** (commit `05a9562…`), not memory.

---

## Headline verdict: FEASIBILITY = **GO**, but the PERFORMANCE WIN is **UNPROVEN** — measure first

The chip can be used; the open question is whether using it actually helps for *this* workload.
**This is NOT a "write a driver" cycle — it is a "prototype + benchmark on real hardware to
confirm the win, THEN decide" cycle.** Two hard findings drive that framing:

1. **A naive drop-in `hash.Hash` likely does NOT win** (maybe inverts) — the hardware can't
   resume from a saved mid-message state, so it forfeits HMAC's marshal fast-path (≈doubles the
   block count), and per-block CPU-polled feeding is slower than the headline DMA number.
2. **The win only matters for high-e backups** (e≥4 ≈ >10 s today). e=0/1 (the realistic,
   Trezor-default cases) are already 0.5–1.9 s on software SHA. So the *value* is bounded to a
   rare configuration — weigh against the implementation/audit cost of a bespoke hardware
   crypto loop on the secret-recovery path.

---

## 1. Feasibility / access — GO (the architect note's `LOCK_SHA_256` worry was a misread)

(verified vs datasheet §10.6, §5.4, §3.x + the fork source)
- **Firmware runs in ARM Secure state** (ARM resets Secure; the fork has NO TrustZone/SAU/
  ACCESSCTRL setup anywhere; `driver/otp/otp_rp2350.go:32` looks up the **Arm-S** bootrom veneer
  `RT_FLAG_FUNC_ARM_SEC`). Secure-state code issues Secure bus accesses.
- **SHA-256 default ACCESSCTRL policy = Secure-Privileged allowed** (SP=1 at reset; §10.6.2.1 +
  Table 958, reg offset 0xb8). So Secure firmware reaches the block **with no grant write**.
- **`LOCK_SHA_256` is a cooperative bootrom-API mutex** (boot lock #0, §5.4.4), **off by
  default** — NOT a peripheral/TrustZone lock; it never walls off the SHA registers. The
  ACCESSCTRL `LOCK` register doesn't lock CORE0 out at reset either, and the bootrom doesn't set
  it under SeedHammer's boot config.
- **Secure-boot (which SeedHammer uses) ≠ Secure-state** — doesn't change any of the above.
- ⚠ One sub-agent mislabeled the security state "Non-secure / NOT POSSIBLE" — **that is wrong**
  (its own `RT_FLAG_FUNC_ARM_SEC` evidence proves Secure); do not carry it into the spec.
- **Belt-and-suspenders for the brainstorm:** a one-line confirm that TinyGo 0.41.1's stock
  `pico-plus2` reset/boot doesn't enable the SAU / drop to Non-secure (extremely unlikely for a
  bare-metal target; not a blocker).

## 2. Hardware facts (datasheet §12.13)

- Block at `0x400f8000`; regs `CSR`/`WDATA`/`SUM0..SUM7`. `CSR.START` (self-clearing) loads the
  **fixed FIPS IV only** (no arbitrary-IV load — see §4 risk); `BSWAP` (bit 12, default **on**)
  byte-swaps for little-endian buffers (correct for Go `[]byte`); poll `CSR.SUM_VLD` (bit 2) —
  **no NVIC IRQ**; DMA via `DREQ_SHA256` (channel 13/`rp.DREQ_SHA256=54`).
- **Per-block cost = 64 cyc feed + 57 cyc digest = 121 cyc/block**, → 79.3 MB/s @150 MHz **only
  with 32-bit DMA feeding**; CPU-polled WDATA is explicitly slower; the **57-cyc digest stall is
  non-overlappable** (must not write during it).
- **Software owns FIPS-180-4 padding** (the hardware digests whole 512-bit blocks only; under-
  feeding a partial block hangs `SUM_VLD`).
- **Single, stateful, shared block** — the **bootrom itself uses it** (boot-random + secure-boot
  signature verify), so it's genuinely contended.
- ⚠ **No datasheet tiny-message benchmark** — 121 cyc/block is the only anchor; the PBKDF2-over-
  millions-of-tiny-hashes figure of merit MUST be measured.

## 3. Driver / Go integration

- **TinyGo pre-declares `rp.SHA256`** (generated from the pinned `cmsis-svd` SVD: base
  `0x400f8000`, fields `CSR{BSWAP,DMA_SIZE,…,START}`, `WDATA`, `SUM0..7`) — a driver bit-bangs
  `rp.SHA256.CSR/WDATA/SUM*` directly; **no hand-declaration of register addresses needed**.
  `rp.DREQ_SHA256` is also declared (for the DMA path).
- **Driver pattern to mirror = `driver/otp/`**: tag-free portable logic + a function-pointer
  hook (`var otp_access …`), a `//go:build tinygo && rp2350` impl wired in `init()`, and a
  software impl installed in tests → the whole package stays host-testable. (The
  `driver/dma`/`driver/pio` register-driving style is the reference for the `volatile.Register32`
  / `rp.*` access.)
- **`crypto/sha256`/`hmac`/`pbkdf2` are NOT overridden by TinyGo** → on-device they're upstream
  Go stdlib. The integration point is `slip39/feistel.go:50` `pbkdf2.Key(pw, salt, iters, half,
  sha256.New)` (4×/recovery) — swapping the `h func() hash.Hash` constructor is a clean stdlib
  pass-through (`fips140hash.UnwrapNew` passes any non-SHA-3 hash through unchanged). The
  `hash.Hash` method set needed: `Write/Sum/Reset/Size(=32)/BlockSize(=64)`.

## 4. THE KEY RISK (driver agent) — drop-in may negate the win; a bespoke loop is the real path

- **Hardware can't resume from a saved mid-message state** (`START` loads only the FIPS IV; no
  IV-load register, no FIFO). So a hardware `hash.Hash` **cannot implement `UnmarshalBinary`** →
  it forfeits HMAC's marshal fast-path (stdlib `crypto/hmac` precomputes the ipad/opad
  compression once and restores it per `Reset`/`Sum`). Result: ~**4 compressions/iter instead of
  ~2** — roughly **doubling** the block count vs the software path it's trying to beat.
- Combined with per-call interface overhead + per-block APB-polled feeding (slower than the DMA
  headline) over **tens of thousands of single-block hashes** (e=0: ~10k PBKDF2 iters; ×2^e),
  a naive drop-in could **erase or invert** the hardware win.
- **The architecturally sound high-perf option:** a **bespoke hardware HMAC-PBKDF2 inner loop**
  — precompute the ipad/opad first-block digests in software once per round (hardware can't,
  lacking IV-load), then drive `rp.SHA256` directly for the iteration loop, ideally **32-bit
  DMA-fed** via `driver/dma`+`rp.DREQ_SHA256`, bypassing `hash.Hash`/`hmac`/`pbkdf2`. This
  **must be benchmarked on real RP2350 before committing** — "drop-in `hash.Hash`" is the
  correctness reference, not necessarily the shipped fast path.

## 5. Other must-handle items for the cycle

- **No active watchdog** (confirmed: `rp.WATCHDOG` only in `rebootIntoBOOTSEL`) → a hung
  `SUM_VLD`/`WDATA_RDY` poll would **hang the firmware**, not reset it. The driver MUST bound
  polling (timeout / `runtime.Gosched()`, cf. `pio.go:326`).
- **Concurrency:** single shared peripheral + `-scheduler tasks` goroutines + HMAC needs **two
  distinct `hash.Hash` instances** (stdlib panics if `h()` yields the same value) → either a
  reservation mutex (cf. `dma.mu`) or confine SHA use to the single secret-recovery path; the
  bespoke loop (which controls instance lifetime) sidesteps the two-distinct-hashes problem.
- **Secret hygiene:** the master secret flows through `WDATA`/`SUM*` → the driver must scrub
  those + re-pulse `START` after use (the software path's `wipe()` doesn't cover registers).
- **Host-test strategy:** keep `slip39` tag-free + software-backed as the host oracle (the CI
  gate); add a build-tag selector for the SHA constructor (`*_host.go` = `sha256.New`;
  `*_rp2350.go` = the hw path); golden hw==sw cross-check + the SLIP-39 fixtures run **on-device
  only**. **Slice-1 lesson:** CI never compiles the `tinygo && rp2350` files — the device build
  (`nix run .#build-firmware`) + the on-hardware cross-check are a separate manual gate; green CI
  ≠ device path builds.

---

## Recommended cycle scope (when picked up)

**Phase 0 (spike, REQUIRED before committing):** on real RP2350 hardware, prototype the bespoke
hw HMAC-PBKDF2 loop (or even the naive drop-in) and **benchmark e=0/1/4/8 wall-clock vs the
shipped software path.** Decision gate: if the bespoke loop doesn't materially beat software for
high-e (and at least not regress e=0/1), **don't build the cycle** — the value (rare high-e
backups) doesn't justify a bespoke crypto loop on the secret path. If it wins, proceed.
**Phase 1 (if Phase 0 passes):** the full gated pipeline — spec R0 → plan R0 → single-implementer
TDD (`driver/sha256/` + the bespoke loop + build-tag selector + bounded polling + register
scrub) → whole-diff execution review → merge. SemVer: firmware. No upstream PR.

**Honest priority:** LOW. It's an optional speedup for an uncommon configuration, gated behind a
hardware-benchmark spike that might itself say "not worth it." Park until there's a concrete need
(a user with a high-e backup) or appetite for the hardware spike.
