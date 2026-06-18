<!--
Persisted verbatim. opus-architect R0 gate of the Phase A implementation plan
(IMPLEMENTATION_PLAN_seedhammer_mstar_phaseA.md @ 289b0f6). Reviewer agentId a849079756b1d1228.
Verdict: GREEN 0C/0I (3 minor). The reviewer materialized the plan's gf1024.go/correct.go + tests
into a standalone codex32 copy and EXECUTED them against the real fork sources at 04a1e95, plus
differential/adversarial tests: 200k-iteration BM differential vs a faithful vec port (0 overflows,
max Λ len 6), exhaustive single-position orientation sweep (no double-flip; Edit.Pos==3+dataPos for
every position), genuine ≤4-error BM probe (deg=#errors, max len 5), dispatch-boundary equivalence,
orientation-direction pins. All 7 load-bearing areas verified sound (field substitution, §2.6
orientation, every guard, BM fixed-array equivalence + bound-9 safety, per-code constant reuse,
test-vector validity, TinyGo). Disposition: GREEN. Folded MINOR-1 (the (fe).rune()->[]rune compile
bug at two test sites, prescribed fix rune(...)), MINOR-2 (added a positive control to the negative
cross-constant test so it isn't vacuous), MINOR-3 (garbage check t.Log->hard ValidMD assert). These
are test-only edits over an already-GREEN verdict; the TDD + the mandatory post-implementation
whole-diff execution review backstop them. The text below is the agent's report exactly as returned.
-->

# R0 GATE REVIEW — m*1 BCH correction (Phase A plan)

**Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_mstar_phaseA.md` (`289b0f6`)
**Spec (GREEN R1):** `design/SPEC_seedhammer_mstar_correction.md`
**Method:** I did not trust the plan's self-claims. I read the Rust oracle (`mk-codec/.../bch_decode.rs`), the fork field/engine (`gf32.go`, `checksum.go`, `codex32.go`, `mdmk.go`), and the fork test vectors. I then **materialized the plan's exact code** (`gf1024.go`, `correct.go`) and its exact tests into a standalone copy of the `codex32` package and ran them, plus adversarial differential tests (exhaustive single-position location sweeps, a 200k-iteration BM differential against a faithful vec-based Rust port, a genuine ≤4-error BM-overflow probe, dispatch-boundary equivalence, and orientation-direction pins). All evidence below is from executed code against the real fork sources at base `04a1e95`.

---

## Verification Results

### 1. Field substitution (GF(1024) on the fork's log-table `fe.Mul`) — VERIFIED SOUND
- `TestForkGf32MatchesCarryless` **passes for all 32×32 inputs**: the fork's `fe.Mul` (`gf32.go:96-103`, log-table `invLogTbl[(logTbl[a]+logTbl[b])%31]`) equals the Rust carryless `gf32_mul` (`bch_decode.rs:102-119`, mask `0b0_1001 = α⁵+α³+1`). Same field, confirmed by execution.
- `TestAlphaPowersMatchInvLogTable` passes: powers of α=feZ(=2) reproduce `invLogTbl` exactly, and the fork's `invLogTbl` (`gf32.go:22-27`) is byte-identical to the Rust `expected[31]` table (`bch_decode.rs:635-638`).
- `gf1024.mul` identity `lo=ll^hh, hi=lh^hl^hh` (`gf1024.go:39-44`) matches `bch_decode.rs:163-172` exactly. `inv=pow(1022)` matches `bch_decode.rs:187-191`. `gf1024One={feP,feQ}={1,0}`, `gf1024Zero={0,0}` match `ONE`/`ZERO`.
- Constants match the Rust verbatim: `betaGf1024={feQ,feG}={0,8}` = `BETA{lo:0,hi:8}` (`bch_decode.rs:204`); `gammaGf1024={feE,feX}={25,6}` = `GAMMA{lo:25,hi:6}` (`bch_decode.rs:208`). Note `feE=25, feX=6` per the `fe` iota (`gf32.go:59,78`) — correct. `regularJStart=77`/`longJStart=1019` match `bch_decode.rs:212,217`.
- `TestBetaOrder93`, `TestGammaOrder1023`, `TestZetaCubeRoot`, `TestGeneratorConsecutiveRoots` all pass (β order 93, γ order 1023, ζ³=1, the 8 consecutive roots β^{77..84}/γ^{1019..1026} are genuine roots of the fork's MSB-first generators).

### 2. Orientation (THE landmine, §2.6) — VERIFIED SELF-CONSISTENT, NOT a double-flip
- `unpackSyms` (`mdmk.go:68-84`) is MSB-first: `out[0]` = top 5 bits = highest power. Confirmed by direct test (`unpackSyms(0, 0b00001_00010_00011, 3) = [1,2,3]`; the mk-regular hi=1 bit lands in the top symbol's bit-4).
- The engine `residue`/`target` are MSB-first (`checksum.go:11-18` doc + `inputFe` shift logic `checksum.go:156-170`; `feP`=1 sits at the last index = x⁰).
- The plan's reversal `coeffs[i] = (residue⊕target)[n-1-i]` (`correct.go:73` region) correctly converts MSB-first → LSB-first `coeffs[0]=x⁰`, matching the Rust `coeffs[i]=(res>>5i)&0x1F` (`bch_decode.rs:306`). Verified by `TestOrientationReversalDirection`.
- `k = L-1-d` (`decodeErrors`) matches `bch_decode.rs:587`. `splitHRP` (`codex32.go:453-459`) confirms the data part begins at `len(hrp)+1`; the plan's `offset=len(hrp)+1` is correct.
- **The double-reversal is self-consistent, proven by an EXHAUSTIVE asymmetric sweep**: corrupting *every* single data-part position (with an asymmetric mask `0b10110`) in md, mk-regular, mk-long, and ms-long, `Correct` recovered the original and reported `Edit.Pos == 3+dataPos` for **every** position. A double-flip would mislocate; it did not.

### 3. Every guard ported — VERIFIED (line-by-line, all present)
- All-zero-syndrome short-circuit (`decodeErrors`) → `bch_decode.rs:560`. ✓
- `deg==0 || deg>4` → `bch_decode.rs:566`. ✓
- Chien `len(degrees)!=deg` → `bch_decode.rs:415,572`. ✓
- Forney's three guards — `lampVal.isZero()→fail`, `mag.hi!=feQ→fail`, `mag.lo==feQ→fail` — match `bch_decode.rs:471,485,488`. ✓
- `d>=L` reject → `bch_decode.rs:583`. ✓
- Ascending sort → the plan's insertion sort matches the Rust `sort_by_key` semantics (verified deterministic by the round-trip Edit ordering). ✓
- Mandatory re-verify via `New`/`ValidMD`/`ValidMK` (`reverify`) → mirrors `bch_correct_*` re-verify; `New`/`ValidMD`/`ValidMK` are the same verifiers the device uses. ✓ Confirmed the 5-error case relies on it.

### 4. Berlekamp-Massey fixed-array port — VERIFIED BEHAVIORALLY IDENTICAL
- (a) Dual skip `i<=k && i<lamLen` matches `bch_decode.rs:343`. ✓
- (b) Increase-branch ordering: `t := lam` (value-copy of the whole array) is taken **before** the update loop; the loop runs over the OLD `prev`; `prev=t` is assigned **after**. This matches the Rust `let t = lam.clone(); … prev = t;` (`bch_decode.rs:354-362`). ✓
- (c) **Bound 9 is safe.** A 200,000-iteration differential test over random syndromes showed the fixed-array `berlekampMassey` is bit-identical to a faithful vec-based Rust port, with **0 overflow cases** and max Λ length 6. A separate 50,000-iteration test over **genuine ≤4-error** syndromes showed `deg(Λ)=#errors` exactly (max len 5), never approaching 9 — so a correctable word is **never** wrongly rejected by the overflow guard. The write index `lam[i+m]` (i<prevLen) reaches at most `newLen-1 ≤ 8 < 9`, and the `newLen > bmMaxLen → reject` guard runs before any write, so no OOB. ✓
- (d) Trailing-zero trim present (`for lamLen>1 && lam[lamLen-1].isZero()`) → `bch_decode.rs:377-379`. ✓

### 5. Per-code dispatch + constant reuse (§2.5) — VERIFIED EXACT
- `TestDispatchMatchesVerifiers` passes across all length boundaries: ms by **total** length (48..93 / 125..127, matching `New` `codex32.go:101-104`), md by **data-part** ≥13 with no upper bracket (matching `ValidMD`/`verifyMDMK` `mdmk.go:99,124`), mk by **data-part** (14..93 / 96..108, matching `ValidMK` `mdmk.go:138-145`), unknown HRP → false.
- The md/mk engines built in `paramsForHRP` (`generator: newShort/LongChecksum().generator`, `residue: unpackSyms(0, mdmkPolymodInitLo, n)`, `target: unpackSyms(targetHi, targetLo, n)`) are **byte-for-byte** the same construction as `verifyMDMK` (`mdmk.go:103-107`). No second copy of any constant; `mdmkPolymodInitLo`, the mk hi/lo target splits, and the brackets are reused from `mdmk.go`. ✓

### 6. Test vectors validity — VERIFIED VALID
- All five literals are valid codewords of their claimed classes (executed against the fork verifiers): `tvMS1Short`(total 48, =fork `secret` literal codex32_test.go:11)→`New` ok; `tvMS1Long`(total 127)→`New` ok; `tvMD1`(=`md1Regular` mdmk_test.go:13, data 21)→`ValidMD`; `tvMK1Reg`(=`mk1Regular`, data 77)→`ValidMK` regular; `tvMK1Long`(=`mk1Long`, data 108)→`ValidMK` long.
- Every round-trip case has **distinct, in-range, non-no-op** corruption positions within the data part, ≤4 per case (verified by `TestRoundTripPositionsValid`). The 5-error positions {0,5,10,15,20} are distinct with nonzero masks.
- The 5-error contract `!(ok && Corrected==original)` matches the Rust `five_errors_either_rejects_or_returns_bogus_recovery` (`bch_decode.rs:843`). ✓
- The negative cross-constant test's `"md"+string(r)[2:]` reconstruction is structurally correct (replaces "ms"→"md", keeps "1…"); see MINOR-1 on its strength.

### 7. TinyGo constraints (§2.7) — VERIFIED
- No `math/big`, no 128-bit anywhere in `gf1024.go`/`correct.go` (grepped). The BM hot path uses fixed `[bmMaxLen]gf1024`/`[synCount]gf1024` arrays — **no heap in BM**. The `make([]…)` calls live in `Correct`/`decodeErrors`/`chienSearch`/`forney` — the **on-demand** correction path (one "Fix?" press), not the per-frame path, which §2.7 scopes as acceptable.
- New code imports only `strings` (already used by the package); the package uses `strings`/`errors`/`fmt`/`slices`/`unicode`, all TinyGo-supported. Task 4 (TinyGo `pico-plus2` build covers `codex32` via the `gui` import) is sound; TinyGo is not on this host, which the plan explicitly defers to CI.

---

## Findings

### CRITICAL
None.

### IMPORTANT
None.

### MINOR

- **MINOR-1 — Test helper compile bug (`(fe).rune()` returns `byte`, assigned to a `rune` slot).** Plan Task 2 Step 1 (`correct_test.go` `corruptAt`, line ~329: `r[abs] = (orig.Add(mask)).rune()`) and Task 3 Step 3 (`TestNegativeCrossConstant`, line ~913: `r[abs] = (orig.Add(mags[i])).rune()`). Because `r := []rune(s)`, these assign a `byte` to a `rune` element and **do not compile** (`cannot use … (value of type byte) as rune value`). Fix: wrap in `rune(...)`, e.g. `r[abs] = rune((orig.Add(mask)).rune())`. This is test-only and trivially fixed (and `corruptUpper`/`feToByte` already wrap correctly), but as written the GREEN steps (Task 2 Step 2/4, Task 3) would fail to build until corrected. After the one-line fix, the entire battery passes (38/38 with the existing fork tests). Not crypto-load-bearing; flagged for accuracy of the "every code step shows complete code" self-claim.

- **MINOR-2 — `TestNegativeCrossConstant` is weaker than it reads.** Under the md constants the corrupted ms data yields `decodeErrors ok=false`, so the `if !ok { return }` path is taken and the reconstruction/`ValidMD` branch is never exercised. The test still correctly asserts the security property (no ms-under-md cross-validation), but it passes vacuously rather than by exercising an actual bogus decode→apply→reject. Optional hardening: also assert the *public* `Correct` on the corrupted ms string returns an `ms`-valid correction (it does — confirmed), and/or add a case where the wrong-constant decode does return something to exercise the reject. Non-blocking.

- **MINOR-3 — `TestCorrectSuppressesUncorrectable` garbage check uses `t.Log`, not `t.Error`.** The garbage sub-check is a soft observation by design (the re-verify gate is the real backstop, tested via the round-trips and 5-error case). Acceptable, but the assertion strength is lower than the surrounding tests; consider asserting `ValidMD(res.Corrected)` when `ok` to make any phantom fix a hard failure. Non-blocking.

*(For completeness: the md dispatch having no upper data-part bracket — mirroring `ValidMD` — means an md data-part >93 would still use the 13-symbol regular code beyond its design length L≤93. This exactly matches the verifier's own behavior and is backstopped by mandatory re-verify, so it is NOT a finding.)*

---

**GREEN — 0 Critical / 0 Important.**
