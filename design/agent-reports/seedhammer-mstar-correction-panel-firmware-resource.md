<!--
Persisted verbatim. Architect panel — firmware/TinyGo lens (agentId ac4daf3ec1a702a54). Verdict: feasible-with-changes — build Gf1024{lo,hi} on the fork's audited GF(32) fe.Mul (~0 new tables, no math/big, no 128-bit); run the decoder ON-DEMAND not per-keystroke; #1 landmine = LSB-vs-MSB symbol orientation at the polymod<->decoder boundary (pin with Rust vectors); add tinygo build -target=pico-plus2 to CI; fixed-size stack arrays in BM.
The text below is the agent's report exactly as returned; do not edit.
-->

# DESIGN INPUT — BCH error-correction decoder port to Go/TinyGo (RP2350)

*Scope: feasibility / footprint / perf of porting `mk-codec/.../bch_decode.rs` into the SeedHammer fork's `codex32` package. Read-only; no code.*

---

## 1. Footprint — GF(1024) representation

**Three options on the table:**

| Approach | Flash (code+data) | RAM/stack hit | Speed/mul | New deps |
|---|---|---|---|---|
| **A. Log/antilog tables for GF(1024)** | ~2KB antilog (1024×2B) + ~2KB log (1024×2B) ≈ **4KB data** + small mul code | tables in flash (`.rodata`), 0 stack | ~3 table lookups + 1 mod | none |
| **B. Computed carryless GF(1024) mul** | ~tens of bytes code, **0 data** | 0 | bit-loop, slowest | none |
| **C. `Gf1024{lo,hi}` over fork's GF(32) `fe.Mul`** (the Rust design) | **~0 new data** (reuses the existing 32+31-entry GF(32) tables, ~63 bytes already resident); ~150–250B mul/pow/inv code | 0 (8-byte value type if `struct{lo,hi fe}`; or pack into one `uint16`) | **4 GF(32) `fe.Mul` per GF(1024) mul** = 4 log-table lookups + adds | none |

**Recommendation: C — build `Gf1024` on the fork's existing `fe.Mul`/`fe.Add`.** This is exactly what the Rust source already does (`Gf1024{lo,hi}`, `mul` = 4 sub-field mults via the `ll/lh/hl/hh` identity at `bch_decode.rs:163-172`), so the port is near-mechanical: swap Rust's `gf32_mul` (carryless) for the fork's log-table `fe.Mul`. **Net-new data added ≈ 0** — the GF(32) log/antilog tables (`gf32.go:14-27`, ~63 bytes) are already linked into the firmware. **Estimated added flash for the whole decoder: ~6–12KB code, ~0 new tables** — negligible against the RP2350's 4MB flash. RAM: the value type is 2 bytes (pack `lo|hi<<5` into a `uint16`) or 2 bytes as `struct{lo,hi uint8}`; the largest working set is the syndrome array (8 elems) + Λ/Ω/Λ′ (≤16 elems each) + a stack-`[15]fe` coeff buffer — **well under 1KB, comfortably inside the 16KB stack.**

Reject **A** (4KB of tables to save microseconds the device doesn't need — wrong tradeoff; also a *second* field representation to keep in sync with the audited GF(32)). Reject **B** as the primary (no reuse of audited code; slower for no footprint win over C since C's data cost is also ~0). **Confirmed: no `math/big`, no new third-party deps** — the Rust is pure `u8`/`u128` and option C touches only `uint8`/`uint64`/`uint16`. The `third_party/seedhammer` submodule and `me-preview` Go sidecar are unaffected (this lands in the firmware fork's `codex32` package, not the CLI).

---

## 2. Perf + WHERE to run the decode

**Perf is a non-issue per decode, confirmed.** A full decode is: 8 syndromes × Horner over ≤15 coeffs (≤120 GF(1024) muls = ≤480 GF(32) lookups), BM O(t²)=O(16) iterations, Chien over ≤108 positions × Horner(≤5) (≤540 GF(1024) muls), Forney (a handful). Order **a few thousand GF(32) log-table lookups** ≈ tens of microseconds on a 150MHz M33 — **3–5 orders of magnitude below PBKDF2** (which is multi-second). Even run every keystroke it would be invisible.

**But do NOT run it every keystroke** — the reason is **UX correctness, not cost:**

- **Recommended trigger: on-demand, only when the *completed* string fails verification.** Concretely: the existing per-keystroke `codex32.New`/`Valid*` path stays the cheap live gate (detection). The decoder fires **only** when (a) the user signals completion (OK/confirm) on a string that the verifier *rejects*, OR (b) an explicit "Suggest fix" affordance. Never mid-typing.
- **Why not per-frame:** a mid-typed prefix is a *truncated* codeword — its length doesn't match a valid bracket and its residue is meaningless, so BM/Chien would churn out garbage "suggestions" that flicker and change every keystroke. Surfacing those is worse than useless (it trains users to ignore or, worse, accept them). The recon's §4 safety model ("suggest → confirm → re-verify, never auto-apply") *requires* a stable, deliberate trigger.
- **Length gate first.** Only attempt decode when `len(data)` is in a valid bracket (regular 14–93 / long 96–108 per `mdmk.go:48-50`); 94–95 and out-of-range are reserved-invalid → no decode attempt. This also means the decoder never runs on the still-growing prefix.
- **Mandatory re-verify after applying** (recon §4; Rust comment at `bch_decode.rs:34-36` and the `five_errors…` test): run the existing `verifyMDMK`/codex32 verify on the corrected string before *offering* it. Drop any suggestion that doesn't verify.

This keeps the live path exactly as fast as today and confines the heavier (still-cheap) decode to a single deliberate moment.

---

## 3. GF(32) impl reconciliation

**Recommendation: reuse the fork's audited log-table `fe.Mul`/`fe.Add` (`gf32.go:87-103`); do NOT port the Rust carryless `gf32_mul`.** Same field, same primitive polynomial `x⁵+x³+1`, same primitive element α=2="z". The Rust test `gf32_alpha_powers_match_bech32_log_inv_table` (`bch_decode.rs:631-645`) cross-checks Rust's carryless powers against *exactly the fork's `invLogTbl`* (`gf32.go:22-27`) — the two representations are proven identical, so swapping the multiply backend is sound. One field implementation, audited once.

**Endianness/encoding gotchas to pin in the spec (this is where it bites):**
- **Symbol packing direction.** The Rust unpacks the residue **LSB-first** — `coeffs[i] = (residue >> (5*i)) & 0x1F`, so `coeffs[0]` is x⁰ / the *lowest* 5 bits (`compute_syndromes`, `bch_decode.rs:305-306`). The fork's `unpackSyms` (`mdmk.go:68-83`) packs **MSB-first** — `out[0]` is the *highest* power. These are opposite conventions. The decoder's Horner walks `coeffs[i]` as the coefficient of `xⁱ`, so **the port must feed it LSB-first** (matching the Rust), or equivalently index `unpackSyms` output in reverse. **This must match the polymod engine the residue comes from.** The fork's `engine.residue` (`checksum.go:11-18`, `inputFe` at :156-170) stores coefficients with **index 0 = highest power x^{n-1}** (big-endian, per the struct comment). So the residue-as-delivered is MSB-first; the decoder wants LSB-first coefficient indexing. **Spec must state one canonical orientation and the conversion at the boundary, then pin it with a Rust-generated parity vector** (recon §6/§10 — never Go-self-generated).
- **The residue handed to the decoder is `residue ⊕ target`** (recon §6, §8: it operates on residue-minus-target ≡ E(x) mod g). The fork already computes both `residue` and `target` as `[]fe` (`checksum.go`, `mdmk.go`); the decoder consumes their XOR. Good — no new constant derivation, **but** the XOR must be done element-wise on the *same* orientation, then re-oriented to LSB-first for the decoder.
- **β/γ/j_start are field-encoding-specific.** `BETA={lo:0,hi:8}`, `GAMMA={lo:25,hi:6}`, `REGULAR_J_START=77`, `LONG_J_START=1019` (`bch_decode.rs:204-217`) are tied to the ζ²=ζ+1 basis and the GF(32) element numbering. Since the fork's `fe` numbering matches bech32 (verified in §3 above), these constants port verbatim — but they must be regression-pinned by the `beta_has_order_93` / `gamma_has_order_1023` / generator-root self-tests (`bch_decode.rs:657-708`), ported as Go tests.

---

## 4. TinyGo correctness gotchas

- **`int` is 32-bit on TinyGo/RP2350 — confirmed not a problem.** The Rust uses `u128` *only* as a bit-container for the packed residue (`residue_xor_const: u128`). The fork already proved you don't need 128-bit math here: `mdmk.go` carries the 65-/75-bit targets as **hi/lo `uint64` pairs** and unpacks 5-bit symbols (`unpackSyms`, :68-83). The decoder operates on **already-unpacked `[]fe` (5-bit) symbols**, so once unpacked it's pure `uint8`/`uint16` GF arithmetic — **no `u128`, no `uint64` arithmetic inside the decoder at all.** Keep the boundary as `(hi, lo uint64)` → `[]fe`, mirroring the existing `unpackSyms`. **Confirm in spec: decoder internals are `uint8`/`uint16` only; the only `uint64` is the residue/target container at the boundary.**
- **`init()`-built tables fine** — but per §1 you don't even need new ones; the GF(32) tables are package-level `var` literals (`gf32.go:14-47`), no `init()` required. If GF(1024) pow/inv ever wants memoization, a package `var` literal is preferable to `init()` for TinyGo determinism, but it's unnecessary at this scale.
- **The Slice-1 lesson — CI never compiles the TinyGo build (`flake.nix` `pico-plus2`).** This is the **single most likely way a regression ships.** Host `go test` exercises the algorithm but **not** the TinyGo codegen path. The decoder is `int`-width-sensitive (Chien indexes ≤108, BM lengths) and slice-allocation-sensitive. **Mitigation (spec requirement): add a TinyGo *device-target build* step (`tinygo build -target=pico-plus2`) to CI for the `codex32` package, even if it only compiles (doesn't run on-device).** Compiling under TinyGo catches the divergences host `go test` cannot.
- **Bounded loops / no-watchdog — confirm bounded, and tighten.** The recon notes a decode is fast so not a hang risk; that's correct, but make the bounds *explicit* so a corrupt input can't loop:
  - **Chien** (`bch_decode.rs:408-413`) is `for d in 0..data_with_checksum_len` — bounded by ≤108. Fine.
  - **BM** (`berlekamp_massey`, :337-375) is `for k in 0..8` — fixed 8 iterations. Fine.
  - **`pow`/`inv`** (`:174-191`) are `exp`-bounded (`inv` = `pow(1022)`, ≤10 squarings). Fine.
  - **Watch the Rust `Vec::resize`/`clone`/`pop` in BM** (`:354-379`): port to **fixed-size stack arrays with an explicit `deg` length field** (max Λ degree is 5, max Ω/Λ′ is 8). Avoid `append`/dynamic slices in the hot path — not for speed but to make the bounds statically obvious and avoid TinyGo heap escapes. Cap `deg(Λ) > 4 ⇒ reject` exactly as the Rust does (`:566-569`) — that's both the correctness gate and an implicit loop bound.

---

## 5. Top feasibility/perf risks (ranked) + mitigations

**1. Symbol-orientation / per-code-constant mismatch at the polymod↔decoder boundary (HIGHEST).** The recon's documented landmine class (§6): the constellation *already shipped* a bug from pairing the wrong init residue, and the Rust↔fork packing conventions are *opposite* (LSB-first vs MSB-first, §3). A silent orientation error produces a decoder that "works" on symmetric test cases but mis-locates errors on real ones — and for seed material a wrong-but-valid correction is catastrophic. **Mitigation:** spec states the exact orientation + per-code init/target table; TDD against **Rust-generated golden parity vectors only** (recon §10), including the corrupt-then-correct integration vectors (`bch_decode.rs:716-808`); mandatory re-verify after apply.

**2. TinyGo build divergence from host `go test` (the Slice-1 lesson).** CI green on host ≠ correct on device. **Mitigation:** add `tinygo build -target=pico-plus2` of `codex32` to CI; port the field self-tests (β order 93, γ order 1023, ζ³=1, generator roots) as Go tests so the field constants are regression-locked under both toolchains.

**3. Miscorrection beyond radius surfaced as a confident suggestion (UX/safety).** Not a perf risk — a perf *non-issue* creates the temptation to run it eagerly (§2). A 5+-error string can yield a valid-looking degree-≤4 locator → a *different valid codeword* (recon §4, `five_errors_either_rejects_or_returns_bogus_recovery`). **Mitigation:** on-demand trigger only; mandatory re-verify before offering; suggest→confirm with was/now shown (never auto-apply); only offer when the correction is unique within the guaranteed radius.

*(Honorable mention, not top-3: dynamic-slice churn in the BM port causing TinyGo heap escapes — fold into risk 2 via fixed-size arrays.)*

---

## Verdict

**FEASIBLE-WITH-CHANGES** — the port is near-mechanical onto the fork's audited GF(32) (`Gf1024{lo,hi}` over `fe.Mul`, ~0 new tables, ~6–12KB flash, no new deps, no `math/big`, no 128-bit math), perf is a non-issue; the required changes are (a) pin the LSB/MSB symbol-orientation + per-code constants with Rust-sourced parity vectors, (b) gate the decoder to an on-demand "suggest→confirm→re-verify" trigger (not per-keystroke), and (c) add a TinyGo device-target compile to CI.

**Sources:** `design/cycle-prep-recon-mstar-correction.md` (§1-10); `crates/mk-codec/src/string_layer/bch_decode.rs` (Gf1024 `:128-192`, syndromes `:292-318`, BM `:327-381`, Chien `:395-419`, Forney `:437-496`, field constants `:204-217`, GF(32)↔bech32 cross-check `:631-645`, integration tests `:716-848`); fork `codex32/gf32.go` (`fe.Mul` log-table `:96-103`, tables `:14-47`), `codex32/checksum.go` (engine/residue orientation `:11-18`, `:156-170`), `codex32/mdmk.go` (`unpackSyms` MSB-first `:68-84`, hi/lo uint64 targets `:54-63`, no-`math/big` note `:34`).
