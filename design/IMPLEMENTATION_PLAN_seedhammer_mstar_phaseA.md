# SeedHammer m*1 Correction — Phase A (pure BCH decoder) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a pure, GUI-less BCH error-correction decoder to the fork's `codex32` package that, for a mistyped `ms1`/`md1`/`mk1` string, returns the unique within-radius (≤4 substitutions) correction that **re-verifies** as a valid codeword — or nothing. Ships **dormant** (no caller).

**Architecture:** Build GF(1024)=GF(32²) on top of the fork's existing log-table GF(32) (`fe.Mul`), porting the constellation Rust decoder (`mk-codec/src/string_layer/bch_decode.rs`) verbatim: syndromes → Berlekamp-Massey → Chien → Forney → apply → **mandatory re-verify** via the fork's existing verifier (`New`/`ValidMD`/`ValidMK`). The decoder consumes the fork engine's `residue ⊕ target` directly (no 128-bit packing), reversing the fork's MSB-first orientation to the Rust's LSB-first at that one boundary.

**Tech Stack:** Go (host `go test`) + TinyGo (`pico-plus2`, 32-bit int, no `math/big`/128-bit, fixed-size stack arrays in the hot path). Spec: `design/SPEC_seedhammer_mstar_correction.md` (GREEN R1). Base: fork `04a1e95`.

**Gate status:** **GREEN (R0, 0C/0I)** — the opus architect materialized this plan's code + tests and executed them against the real fork sources (200k-iteration BM differential, exhaustive orientation sweep, dispatch-boundary equivalence). Review verbatim: `design/agent-reports/seedhammer-mstar-correction-phaseA-plan-review-R0.md`. The 3 minors are folded (MINOR-1 the `rune(...)` wrap, MINOR-2 positive control, MINOR-3 hard ValidMD assert).

**SHIPPED:** Phase A merged dormant to fork `main` `3342165` (no-ff, signed+DCO), pushed `bg002h`. Whole-diff execution review GREEN (0C/0I; 1 doc minor folded — `Edit.Pos` is a rune index): `design/agent-reports/seedhammer-mstar-correction-phaseA-execution-review.md`. 38 tests pass / 0 fail; TinyGo `pico-plus2` build wired into `test.yml` (covers `codex32` via `cmd/controller`). **Next: Phase B** (HRP-dispatched typed entry + suggest→confirm "Fix?" gate) — its own spec→R0→plan→R0→TDD→exec-review cycle.

---

## Source-of-truth facts (verified against current source by the recon fan-out)

**Fork GF(32)** (`codex32/gf32.go`): `type fe uint8`; bech32 numeric order `feQ=0,feP=1,feZ=2,…,feE=25,feX=6,feG=8,…`; `(fe).Add`=XOR, `(fe).Mul`=`invLogTbl[(logTbl[a]+logTbl[b])%31]` (log-table, **not** carryless), `(fe).Div`; tables `logTbl [32]uint8`, `invLogTbl [31]fe`, `charsLowerTbl [32]byte = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"`; helpers `feFromRune(rune)(fe,bool)`, `feFromInt(int)(fe,bool)`, `(fe).rune() rune`.

**Fork engine** (`codex32/checksum.go`): `type engine struct{ _case charCase; generator, residue, target []fe }`; `inputHRP`, `inputData`, `inputFe` (the polymod step), `isValid()=slices.Equal(residue,target)`; `newShortChecksum()` (generator 13 syms, residue `{feQ×12,feP}`=codex32 init 1, target `SECRETSHARE32`), `newLongChecksum()` (generator 15 syms, residue `{feQ×14,feP}`, target `SECRETSHARE32EX`). The XOR with `target` is **not** pre-applied — `isValid` only compares; the decoder must compute `residue⊕target` itself. `residue`/`target`/`generator` are **MSB-first** (index 0 = highest power; `feP`=1 sits at the last index = x⁰).

**Per-code constants** (`codex32/mdmk.go`): `mdmkPolymodInitLo uint64 = 0x23181b3`; `mdmkShortSyms=13`, `mdmkLongSyms=15`; `mkRegularMinLen=14`,`mkRegularMaxLen=93`,`mkLongMinLen=96`,`mkLongMaxLen=108`; targets `mdRegularTargetHi/Lo=0x0 / 0x0815c07747a3392e7`, `mkRegularTargetHi/Lo=0x1 / 0x62435f91072fa5c`, `mkLongTargetHi/Lo=0x418 / 0x90d7e441cbe97273`; `unpackSyms(hi,lo uint64,n int) []fe` (MSB-first, `out[0]`=top 5 bits). `verifyMDMK` builds `&engine{generator: newShort/LongChecksum().generator, residue: unpackSyms(0,mdmkPolymodInitLo,n), target: unpackSyms(targetHi,targetLo,n)}`. md uses the regular code only; mk dispatches regular/long by **data-part** length; ms (`New`) dispatches short/long by **total** length (48..93 / 125..127).

**Rust decoder** (`mk-codec/.../bch_decode.rs`): `Gf1024{lo,hi}` = `lo + hi·ζ`, `ζ²=ζ+1`; `mul` → `lo=ll^hh, hi=lh^hl^hh` (with `ll=lo·lo'`, `lh=lo·hi'`, `hl=hi·lo'`, `hh=hi·hi'` via GF(32) mul); `add`=XOR; `ONE={1,0}`, `ZERO={0,0}`, `ZETA={0,1}`; `pow`=square-and-multiply; `inv`=`pow(1022)`. `BETA={lo:0,hi:8}` (order 93, regular), `GAMMA={lo:25,hi:6}` (order 1023, long); `REGULAR_J_START=77`, `LONG_J_START=1019`. 8 syndromes, `t=4`. Internally **LSB-first**: `coeffs[i]=(residue>>(5*i))&0x1F` = coeff of xⁱ; Horner iterates high-index→low. `decode_errors`: syndromes → all-zero short-circuit → BM → `deg==0||deg>4`→None → Chien (`roots==deg` guard) → Forney (3 guards: `Λ'(X⁻¹)≠0`, `mag.hi==0`, `mag.lo≠0`) → translate `k=L-1-d` → sort ascending. `bch_correct_*` apply `corrected[p]^=m` then **re-verify**.

**Orientation boundary (the §2.6 landmine):** fork `residue⊕target` vector `v[j]` (j=0..n-1) = coeff of x^{n-1-j} (MSB-first). The Rust wants `coeffs[i]` = coeff of xⁱ (LSB-first). ⇒ `coeffs[i] = v[n-1-i]` (reverse). Chien returns polynomial degrees `d`; data-part index `k = L-1-d`; full-string index `= len(hrp)+1+k`.

**BM buffer sizing (M-3):** over 8 syndromes the classic invariant gives `deg(Λ) ≤ L ≤ 8`, so `lam`/`prev` size `[9]` with an explicit overflow guard (reject if a resize would exceed 9 — correctable words yield `deg≤4` and never approach it). `Ω` is `[8]` by definition (`mod x⁸`); `Λ'` is `[8]`.

**Parity vectors:** the cleanest deterministic oracle is round-trip on the fork's own valid literals (`codex32_test.go`, `mdmk_test.go`) corrupted at known `(pos,mask)` via the codex32-alphabet XOR helper, asserting `Correct` recovers the original byte-for-byte and reports the right `Edits`. The 5-error contract: `Correct` must **not** silently return the original (mirrors Rust `five_errors_either_rejects_or_returns_bogus_recovery`).

---

## File manifest

| File | Change |
|---|---|
| `codex32/gf1024.go` | **new** — GF(1024) on the fork's GF(32) (`fe.Mul`); `betaGf1024`/`gammaGf1024`/j-starts. |
| `codex32/gf1024_test.go` | **new** — field self-tests: carryless cross-check, α-powers=`invLogTbl`, ζ³=1, β order 93, γ order 1023, generator roots. |
| `codex32/correct.go` | **new** — `Edit`, `CorrectionResult`, `Correct`, per-HRP dispatch, the internal decoder (`horner`/`hornerExt`/`computeSyndromes`/`berlekampMassey`/`chienSearch`/`forney`/`decodeErrors`), apply + re-verify. |
| `codex32/correct_test.go` | **new** — string-level parity (1/2/4-error per code, ms-long, mk-long), 5-error non-silent, negative cross-constant, orientation pin, case preservation, suppress-when-uncorrectable. |
| `.github/workflows/*` | **modify** — ensure a `tinygo build -target=pico-plus2` exercises the `codex32` package. |

Unchanged/reused: `gf32.go`, `checksum.go`, `mdmk.go` (`unpackSyms`, params, `verifyMDMK`/`ValidMD`/`ValidMK`), `codex32.go` (`splitHRP`, `New`, length consts).

---

## Task 0: Worktree + clean baseline

**Files:** none (setup only).

- [ ] **Step 1: Create the isolated worktree off the base commit**

Run from the fork repo:
```bash
cd /scratch/code/shibboleth/seedhammer
git worktree add -b feat/mstar-correct-decoder ../seedhammer-wt-mstar-a 04a1e95
cd ../seedhammer-wt-mstar-a
```
Expected: a new worktree on branch `feat/mstar-correct-decoder` at HEAD `04a1e95`.

- [ ] **Step 2: Verify the clean baseline compiles and tests pass**

Run: `go test ./codex32/...`
Expected: PASS (all existing codex32 tests green).

If TinyGo is installed, also run the device build you intend to wire into CI (Task 4) to confirm the baseline compiles:
Run: `tinygo build -o /dev/null -target=pico-plus2 . 2>&1 | tail -5` (from repo root of the worktree)
Expected: builds, or a known pre-existing condition you record. If TinyGo is unavailable on the host, note it and rely on CI (Task 4).

---

## Task 1: GF(1024) field on the fork's GF(32)

**Files:**
- Create: `codex32/gf1024.go`
- Test: `codex32/gf1024_test.go`

- [ ] **Step 1: Write the failing field self-tests**

Create `codex32/gf1024_test.go`:
```go
package codex32

import "testing"

// carrylessGf32Mul is the Rust reference GF(32) multiply (mk-codec
// bch_decode.rs gf32_mul): carryless multiply mod α⁵+α³+1 (mask 0b0_1001).
// Used ONLY to cross-check that the fork's log-table fe.Mul is the SAME
// field, which is what licenses building GF(1024) on fe.Mul (SPEC §3.1).
func carrylessGf32Mul(a, b uint8) uint8 {
	const reduce = 0b0_1001
	var result uint8
	for i := 0; i < 5; i++ {
		if (b>>uint(i))&1 != 0 {
			result ^= a
		}
		carry := (a >> 4) & 1
		a = (a << 1) & 0x1f
		if carry != 0 {
			a ^= reduce
		}
	}
	return result
}

func TestForkGf32MatchesCarryless(t *testing.T) {
	for a := 0; a < 32; a++ {
		for b := 0; b < 32; b++ {
			got := fe(a).Mul(fe(b))
			want := carrylessGf32Mul(uint8(a), uint8(b))
			if uint8(got) != want {
				t.Fatalf("fe(%d).Mul(fe(%d)) = %d, carryless = %d", a, b, got, want)
			}
		}
	}
}

func TestAlphaPowersMatchInvLogTable(t *testing.T) {
	// Powers of α=feZ(=2) via fe.Mul must reproduce invLogTbl, pinning the
	// fork field to the standard codex32 GF(32) the Rust cross-checks.
	a := fe(1)
	for i := 0; i < 31; i++ {
		if a != invLogTbl[i] {
			t.Fatalf("α^%d = %d, want %d", i, a, invLogTbl[i])
		}
		a = a.Mul(feZ)
	}
	if a != fe(1) {
		t.Fatalf("α^31 = %d, want 1", a)
	}
}

func TestZetaCubeRoot(t *testing.T) {
	zeta := gf1024{lo: feQ, hi: feP} // {0,1}
	if got := zeta.mul(zeta); got != zeta.add(gf1024One) {
		t.Fatalf("ζ² = %+v, want ζ+1 = %+v", got, zeta.add(gf1024One))
	}
	if got := zeta.mul(zeta).mul(zeta); got != gf1024One {
		t.Fatalf("ζ³ = %+v, want 1", got)
	}
}

func TestBetaOrder93(t *testing.T) {
	p := gf1024One
	for j := 1; j <= 93; j++ {
		p = p.mul(betaGf1024)
		if p == gf1024One && j != 93 {
			t.Fatalf("β returned to 1 prematurely at exponent %d", j)
		}
	}
	if p != gf1024One {
		t.Fatalf("β^93 = %+v, want 1", p)
	}
}

func TestGammaOrder1023(t *testing.T) {
	for _, q := range []uint32{341, 93, 33} { // 1023/{3,11,31}
		if gammaGf1024.pow(q) == gf1024One {
			t.Fatalf("γ^%d = 1 (γ not order 1023)", q)
		}
	}
	if gammaGf1024.pow(1023) != gf1024One {
		t.Fatalf("γ^1023 != 1")
	}
}

func TestGeneratorConsecutiveRoots(t *testing.T) {
	// g(x) = x^n + Σ generator[j]·x^{n-1-j} (generator MSB-first, monic
	// leading implied). Its 8 consecutive defining roots are β^{77..84}
	// (regular) and γ^{1019..1026} (long); verify g evaluates to zero there.
	evalGen := func(gen []fe, x gf1024) gf1024 {
		n := len(gen)
		acc := x.pow(uint32(n)) // x^n (leading monic term)
		for j := 0; j < n; j++ {
			acc = acc.add(gf1024FromFe(gen[j]).mul(x.pow(uint32(n - 1 - j))))
		}
		return acc
	}
	gReg := newShortChecksum().generator
	for j := uint32(77); j <= 84; j++ {
		if !evalGen(gReg, betaGf1024.pow(j)).isZero() {
			t.Fatalf("g_regular(β^%d) != 0", j)
		}
	}
	gLong := newLongChecksum().generator
	for j := uint32(1019); j <= 1026; j++ {
		if !evalGen(gLong, gammaGf1024.pow(j)).isZero() {
			t.Fatalf("g_long(γ^%d) != 0", j)
		}
	}
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `go test ./codex32/ -run 'Gf32|AlphaPowers|Zeta|Beta|Gamma|Generator' -v`
Expected: FAIL — `gf1024`, `gf1024One`, `betaGf1024`, `gammaGf1024`, `gf1024FromFe` undefined.

- [ ] **Step 3: Implement the field**

Create `codex32/gf1024.go`:
```go
package codex32

// gf1024 is an element of GF(1024)=GF(32²), represented as lo + hi·ζ with
// the field relation ζ²=ζ+1. Built on the fork's GF(32) (fe.Mul); this is a
// faithful port of the constellation Gf1024 (mk-codec bch_decode.rs), whose
// carryless GF(32) multiply is identical to the fork's log-table fe.Mul
// (pinned by TestForkGf32MatchesCarryless). Pure; no allocation.
type gf1024 struct {
	lo, hi fe
}

var (
	gf1024Zero = gf1024{lo: feQ, hi: feQ} // {0,0}
	gf1024One  = gf1024{lo: feP, hi: feQ} // {1,0}

	// betaGf1024 = β = 8·ζ (feQ + feG·ζ), order 93, the regular code's
	// BCH-defining primitive element. gammaGf1024 = γ = 25 + 6·ζ
	// (feE + feX·ζ), order 1023, the long code's. (bch_decode.rs:204-211.)
	betaGf1024  = gf1024{lo: feQ, hi: feG} // {0,8}
	gammaGf1024 = gf1024{lo: feE, hi: feX} // {25,6}
)

const (
	regularJStart uint32 = 77   // β^{77..84} are the regular generator's roots
	longJStart    uint32 = 1019 // γ^{1019..1026} the long generator's roots
)

func gf1024FromFe(v fe) gf1024 { return gf1024{lo: v, hi: feQ} }

func (a gf1024) add(b gf1024) gf1024 {
	return gf1024{lo: a.lo ^ b.lo, hi: a.hi ^ b.hi}
}

func (a gf1024) isZero() bool { return a.lo == feQ && a.hi == feQ }

// mul multiplies in GF(1024) via the 4-subfield identity (ζ²=ζ+1):
//   (lo+hi·ζ)(lo'+hi'·ζ) = (ll+hh) + (lh+hl+hh)·ζ.
func (a gf1024) mul(b gf1024) gf1024 {
	ll := a.lo.Mul(b.lo)
	lh := a.lo.Mul(b.hi)
	hl := a.hi.Mul(b.lo)
	hh := a.hi.Mul(b.hi)
	return gf1024{lo: ll ^ hh, hi: lh ^ hl ^ hh}
}

// pow is square-and-multiply. exp is a small fixed exponent (j_start, 1022,
// position degrees), never attacker-controlled timing-sensitive material.
func (a gf1024) pow(exp uint32) gf1024 {
	base := a
	res := gf1024One
	for exp > 0 {
		if exp&1 == 1 {
			res = res.mul(base)
		}
		base = base.mul(base)
		exp >>= 1
	}
	return res
}

// inv is the Fermat inverse a^(2^10-2)=a^1022. Callers guard against inverting
// zero (zero syndromes / Λ'(X⁻¹)≠0) before reaching here.
func (a gf1024) inv() gf1024 { return a.pow(1022) }
```

- [ ] **Step 4: Run the field tests to verify they pass**

Run: `go test ./codex32/ -run 'Gf32|AlphaPowers|Zeta|Beta|Gamma|Generator' -v`
Expected: PASS (all six).

- [ ] **Step 5: Commit**

```bash
git add codex32/gf1024.go codex32/gf1024_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "codex32: GF(1024) field on the fork's GF(32) (Phase A)

Port of the constellation Gf1024 (lo+hi·ζ, ζ²=ζ+1) built on the fork's
log-table fe.Mul. Field self-tests pin: fork GF(32) == Rust carryless
GF(32), α-powers == invLogTbl, ζ³=1, β order 93, γ order 1023, the 8
consecutive generator roots β^{77..84}/γ^{1019..1026}.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: The decoder + public `Correct`

**Files:**
- Create: `codex32/correct.go`
- Test: `codex32/correct_test.go` (one driving test now; the full battery in Task 3)

- [ ] **Step 1: Write the first failing end-to-end test (+ the shared corruption helper)**

Create `codex32/correct_test.go`:
```go
package codex32

import (
	"strings"
	"testing"
)

// Valid fork literals (codex32_test.go / mdmk_test.go) used as correction seeds.
const (
	tvMS1Short = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw"
	tvMD1      = "md1yqpqqxqq8xtwhw4xwn4qh"
	tvMK1Reg   = "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x"
	tvMK1Long  = "mk1qp0zgpzp3xqgpqqgqjyty8ssyqcq0tdd4kk6mtdd4kk6mtdd4kk6mtdd4kk6mtdd4kk6mtdd4kk6mtddq2vfczmkedtrj2rjl6la2h9ek48q"
	tvMS1Long  = "ms100c8vsm32zxfguhpchtlupzry9x8gf2tvdw0s3jn54khce6mua7lqpzygsfjd6an074rxvcemlh8wu3tk925acdefghjklmnpqrstuvwxy06fhpv80undvarhrak"
)

// corruptAt substitutes the data-part symbol at dataPos by XORing mask (a
// nonzero GF(32) value) into it — the codex32-alphabet substitution the Rust
// corrupt_at helper performs. hrpLen is 3 ("ms1"/"md1"/"mk1"). Seeds here are
// lowercase, so the corrected char is emitted lowercase.
func corruptAt(t *testing.T, s string, dataPos int, mask fe) string {
	t.Helper()
	if mask == 0 {
		t.Fatal("mask must be nonzero")
	}
	r := []rune(s)
	abs := 3 + dataPos
	orig, ok := feFromRune(r[abs])
	if !ok {
		t.Fatalf("bad seed char %q at %d", r[abs], abs)
	}
	r[abs] = rune((orig.Add(mask)).rune()) // (fe).rune() returns byte
	return string(r)
}

func TestCorrectMD1OneError_OrientationPin(t *testing.T) {
	// Asymmetric single error at data position 5: if the MSB/LSB orientation
	// boundary is flipped, the decoder locates L-1-5 (or garbage) and re-verify
	// fails. So this is also the orientation pin (SPEC §2.6).
	corrupted := corruptAt(t, tvMD1, 5, 0b10101)
	if corrupted == tvMD1 {
		t.Fatal("corruption was a no-op")
	}
	res, ok := Correct(corrupted)
	if !ok {
		t.Fatal("expected a correction")
	}
	if res.Corrected != tvMD1 {
		t.Fatalf("Corrected = %q, want %q", res.Corrected, tvMD1)
	}
	if len(res.Edits) != 1 {
		t.Fatalf("len(Edits) = %d, want 1", len(res.Edits))
	}
	e := res.Edits[0]
	if e.Pos != 3+5 {
		t.Errorf("Edit.Pos = %d, want %d", e.Pos, 3+5)
	}
	if e.Now != tvMD1[3+5] {
		t.Errorf("Edit.Now = %q, want %q (original char)", e.Now, tvMD1[3+5])
	}
	if e.Was != corrupted[3+5] {
		t.Errorf("Edit.Was = %q, want %q (corrupted char)", e.Was, corrupted[3+5])
	}
	_ = strings.TrimSpace
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `go test ./codex32/ -run TestCorrectMD1OneError -v`
Expected: FAIL — `Correct`, `CorrectionResult`, `Edit` undefined.

- [ ] **Step 3: Implement the decoder and `Correct`**

Create `codex32/correct.go`:
```go
package codex32

// Edit is one substitution the decoder applied: the full-string byte index and
// the before/after bech32 characters (for the GUI's per-position confirm diff).
type Edit struct {
	Pos      int
	Was, Now byte
}

// CorrectionResult is a unique within-radius (≤4 substitutions) correction that
// RE-VERIFIES as a valid codeword.
type CorrectionResult struct {
	Corrected string
	Edits     []Edit
}

// synCount is the number of BCH syndromes (the codes are BCH(•,•,8), t=4).
const synCount = 8

// bmMaxLen bounds the Berlekamp-Massey connection-polynomial array. Over 8
// syndromes deg(Λ)≤L≤8 (classic BM invariant), so 9 coefficients suffice; a
// resize that would exceed this is treated as uncorrectable (correctable words
// yield deg≤4 and never approach it). See SPEC §2.7 / plan M-3.
const bmMaxLen = synCount + 1

// bchParams selects the per-code BCH machinery. eng is a fresh verifier engine
// (generator + POLYMOD_INIT residue + target) — the SAME constants the
// verifier uses (no second copy; SPEC §2.5). alpha is β (regular) or γ (long).
type bchParams struct {
	eng    *engine
	alpha  gf1024
	jStart uint32
	nSyms  int
}

// paramsForHRP returns the decoder parameters for a fragment, mirroring the
// verifiers' dispatch exactly: ms by total length (New: 48..93 short / 125..127
// long), md regular-only (data ≥13), mk by data-part length (14..93 regular /
// 96..108 long). ok=false for any out-of-bracket / unknown HRP.
func paramsForHRP(hrp string, total, dataLen int) (bchParams, bool) {
	switch strings.ToLower(hrp) {
	case "ms":
		switch {
		case total >= shortCodeMinLength && total <= shortCodeMaxLength:
			return bchParams{newShortChecksum(), betaGf1024, regularJStart, shortChecksumLen}, true
		case total >= longCodeMinLength && total <= longCodeMaxLength:
			return bchParams{newLongChecksum(), gammaGf1024, longJStart, longChecksumLen}, true
		}
	case "md":
		if dataLen >= mdmkShortSyms {
			eng := &engine{
				generator: newShortChecksum().generator,
				residue:   unpackSyms(0, mdmkPolymodInitLo, mdmkShortSyms),
				target:    unpackSyms(mdRegularTargetHi, mdRegularTargetLo, mdmkShortSyms),
			}
			return bchParams{eng, betaGf1024, regularJStart, mdmkShortSyms}, true
		}
	case "mk":
		switch {
		case dataLen >= mkRegularMinLen && dataLen <= mkRegularMaxLen:
			eng := &engine{
				generator: newShortChecksum().generator,
				residue:   unpackSyms(0, mdmkPolymodInitLo, mdmkShortSyms),
				target:    unpackSyms(mkRegularTargetHi, mkRegularTargetLo, mdmkShortSyms),
			}
			return bchParams{eng, betaGf1024, regularJStart, mdmkShortSyms}, true
		case dataLen >= mkLongMinLen && dataLen <= mkLongMaxLen:
			eng := &engine{
				generator: newLongChecksum().generator,
				residue:   unpackSyms(0, mdmkPolymodInitLo, mdmkLongSyms),
				target:    unpackSyms(mkLongTargetHi, mkLongTargetLo, mdmkLongSyms),
			}
			return bchParams{eng, gammaGf1024, longJStart, mdmkLongSyms}, true
		}
	}
	return bchParams{}, false
}

// Correct attempts to error-correct an invalid codex32-family string of the
// given parsed code. Returns (result, true) ONLY for a unique within-radius
// (≤4 substitutions) correction that RE-VERIFIES as a valid codeword; (_, false)
// otherwise (uncorrectable / >radius / re-verify fail). It NEVER guesses and
// NEVER auto-applies — the caller confirms result.Edits against the source card.
func Correct(frag string) (CorrectionResult, bool) {
	hrp, data := splitHRP(frag)
	p, ok := paramsForHRP(hrp, len(frag), len(data))
	if !ok {
		return CorrectionResult{}, false
	}
	if err := p.eng.inputHRP(hrp); err != nil {
		return CorrectionResult{}, false
	}
	if err := p.eng.inputData(data); err != nil {
		return CorrectionResult{}, false
	}
	// Syndrome polynomial = residue ⊕ target, reversed from the engine's
	// MSB-first layout to LSB-first coeffs[i]=coeff of xⁱ (SPEC §2.6).
	n := p.nSyms
	coeffs := make([]fe, n)
	for i := 0; i < n; i++ {
		coeffs[i] = p.eng.residue[n-1-i] ^ p.eng.target[n-1-i]
	}
	positions, mags, ok := decodeErrors(coeffs, len(data), p.alpha, p.jStart)
	if !ok || len(positions) == 0 {
		return CorrectionResult{}, false
	}
	// Apply: the data part begins at full-string index len(hrp)+1 (HRP + the
	// '1' separator). Preserve the fragment's case so the result re-verifies.
	useUpper := fragUsesUpper(frag)
	offset := len(hrp) + 1
	r := []rune(frag)
	edits := make([]Edit, 0, len(positions))
	for i, k := range positions {
		abs := offset + k
		if abs < 0 || abs >= len(r) {
			return CorrectionResult{}, false
		}
		was := r[abs]
		orig, ok := feFromRune(was)
		if !ok {
			return CorrectionResult{}, false
		}
		now := feToByte(orig.Add(mags[i]), useUpper)
		r[abs] = rune(now)
		edits = append(edits, Edit{Pos: abs, Was: byte(was), Now: now})
	}
	corrected := string(r)
	if !reverify(hrp, corrected) {
		return CorrectionResult{}, false // mandatory re-verify (SPEC §2.2)
	}
	return CorrectionResult{Corrected: corrected, Edits: edits}, true
}

// reverify runs the SAME verifier the device uses, dispatched by HRP.
func reverify(hrp, s string) bool {
	switch strings.ToLower(hrp) {
	case "ms":
		_, err := New(s)
		return err == nil
	case "md":
		return ValidMD(s)
	case "mk":
		return ValidMK(s)
	}
	return false
}

// fragUsesUpper reports whether the fragment is upper-cased (no lowercase
// letter). Valid codex32 strings are single-cased; the corrected char matches.
func fragUsesUpper(s string) bool {
	for _, c := range s {
		if c >= 'a' && c <= 'z' {
			return false
		}
	}
	return true
}

// feToByte renders a GF(32) symbol as a bech32 char in the requested case.
func feToByte(v fe, upper bool) byte {
	b := byte(v.rune()) // lowercase
	if upper && b >= 'a' && b <= 'z' {
		b -= 'a' - 'A'
	}
	return b
}

// decodeErrors runs the BCH pipeline over LSB-first GF(32) coeffs (the residue⊕
// target, length nSyms) for a data part of length L. Returns ascending-sorted
// data-part positions and matching GF(32) magnitudes, or ok=false. Port of the
// Rust decode_errors (bch_decode.rs:550).
func decodeErrors(coeffs []fe, L int, alpha gf1024, jStart uint32) ([]int, []fe, bool) {
	syn := computeSyndromes(coeffs, alpha, jStart)
	allZero := true
	for _, s := range syn {
		if !s.isZero() {
			allZero = false
			break
		}
	}
	if allZero {
		return nil, nil, false // already a codeword; nothing to correct
	}
	lam, lamLen, ok := berlekampMassey(syn)
	if !ok {
		return nil, nil, false
	}
	deg := lamLen - 1
	if deg == 0 || deg > 4 {
		return nil, nil, false // >4 errors exceeds t=4 capacity
	}
	degrees, ok := chienSearch(lam, lamLen, L, alpha)
	if !ok || len(degrees) != deg {
		return nil, nil, false
	}
	mags, ok := forney(syn, lam, lamLen, degrees, alpha, jStart)
	if !ok {
		return nil, nil, false
	}
	// Translate polynomial degree d -> data index k = L-1-d, then sort
	// ascending by position (insertion sort: ≤4 elements, TinyGo-friendly).
	pos := make([]int, len(degrees))
	mg := make([]fe, len(degrees))
	for i, d := range degrees {
		if d >= L {
			return nil, nil, false
		}
		k := L - 1 - d
		j := i
		for j > 0 && pos[j-1] > k {
			pos[j] = pos[j-1]
			mg[j] = mg[j-1]
			j--
		}
		pos[j] = k
		mg[j] = mags[i]
	}
	return pos, mg, true
}

// computeSyndromes: S_m = E(α^{jStart+m}) for m=0..7 (bch_decode.rs:286).
func computeSyndromes(coeffs []fe, alpha gf1024, jStart uint32) [synCount]gf1024 {
	var syn [synCount]gf1024
	aj := alpha.pow(jStart)
	for m := 0; m < synCount; m++ {
		syn[m] = horner(coeffs, aj)
		aj = aj.mul(alpha)
	}
	return syn
}

// horner evaluates a GF(32)-coefficient polynomial (coeffs[i]=coeff of xⁱ) at a
// GF(1024) point, high index first.
func horner(coeffs []fe, x gf1024) gf1024 {
	acc := gf1024Zero
	for i := len(coeffs) - 1; i >= 0; i-- {
		acc = acc.mul(x).add(gf1024FromFe(coeffs[i]))
	}
	return acc
}

// hornerExt is horner for GF(1024)-coefficient polynomials.
func hornerExt(coeffs []gf1024, x gf1024) gf1024 {
	acc := gf1024Zero
	for i := len(coeffs) - 1; i >= 0; i-- {
		acc = acc.mul(x).add(coeffs[i])
	}
	return acc
}

// berlekampMassey returns the error-locator Λ (Λ(0)=1) and its length, or
// ok=false on buffer overflow. Fixed-size arrays; no heap (SPEC §2.7). Port of
// bch_decode.rs:324.
func berlekampMassey(syn [synCount]gf1024) ([bmMaxLen]gf1024, int, bool) {
	var lam, prev [bmMaxLen]gf1024
	lam[0] = gf1024One
	prev[0] = gf1024One
	lamLen, prevLen := 1, 1
	l := 0
	m := 1
	b := gf1024One

	for k := 0; k < synCount; k++ {
		d := syn[k]
		for i := 1; i <= l; i++ {
			if i <= k && i < lamLen {
				d = d.add(lam[i].mul(syn[k-i]))
			}
		}
		if d.isZero() {
			m++
			continue
		}
		scale := d.mul(b.inv())
		newLen := lamLen
		if prevLen+m > newLen {
			newLen = prevLen + m
		}
		if newLen > bmMaxLen {
			return lam, 0, false
		}
		if 2*l <= k {
			t := lam // value copy of the whole array
			tLen := lamLen
			lamLen = newLen
			for i := 0; i < prevLen; i++ {
				lam[i+m] = lam[i+m].add(scale.mul(prev[i]))
			}
			l = k + 1 - l
			prev = t
			prevLen = tLen
			b = d
			m = 1
		} else {
			lamLen = newLen
			for i := 0; i < prevLen; i++ {
				lam[i+m] = lam[i+m].add(scale.mul(prev[i]))
			}
			m++
		}
	}
	for lamLen > 1 && lam[lamLen-1].isZero() {
		lamLen--
	}
	return lam, lamLen, true
}

// chienSearch returns the polynomial degrees d∈[0,L) with Λ(α^{-d})=0, or
// ok=false if the root count != deg(Λ) (bch_decode.rs:387).
func chienSearch(lam [bmMaxLen]gf1024, lamLen, L int, alpha gf1024) ([]int, bool) {
	deg := lamLen - 1
	if deg == 0 {
		return nil, true
	}
	degrees := make([]int, 0, deg)
	aInv := alpha.inv()
	cur := gf1024One
	for d := 0; d < L; d++ {
		if hornerExt(lam[:lamLen], cur).isZero() {
			degrees = append(degrees, d)
		}
		cur = cur.mul(aInv)
	}
	if len(degrees) != deg {
		return nil, false
	}
	return degrees, true
}

// forney returns the GF(32) error magnitudes for the located degrees, or
// ok=false on any guard (Λ'(X⁻¹)=0, mag∉GF(32), mag=0). Port of
// bch_decode.rs:421.
func forney(syn [synCount]gf1024, lam [bmMaxLen]gf1024, lamLen int, degrees []int, alpha gf1024, jStart uint32) ([]fe, bool) {
	// Ω(x) = S(x)·Λ(x) mod x⁸.
	var omega [synCount]gf1024
	for i := 0; i < synCount; i++ {
		for j := 0; j < lamLen; j++ {
			if i+j < synCount {
				omega[i+j] = omega[i+j].add(syn[i].mul(lam[j]))
			}
		}
	}
	// Λ'(x): char-2 formal derivative keeps only odd-power terms.
	var lamPrime [synCount]gf1024
	lpLen := lamLen - 1
	if lpLen < 0 {
		lpLen = 0
	}
	for i := 1; i < lamLen; i++ {
		if i%2 == 1 {
			lamPrime[i-1] = lam[i]
		}
	}
	shift := uint32(0)
	if jStart > 0 {
		shift = jStart - 1
	}
	mags := make([]fe, 0, len(degrees))
	for _, d := range degrees {
		xk := alpha.pow(uint32(d))
		xkInv := xk.inv()
		omegaVal := hornerExt(omega[:], xkInv)
		lampVal := hornerExt(lamPrime[:lpLen], xkInv)
		if lampVal.isZero() {
			return nil, false
		}
		xkShift := xkInv.pow(shift) // X_k^{1-jStart}
		mag := xkShift.mul(omegaVal.mul(lampVal.inv()))
		if mag.hi != feQ {
			return nil, false // magnitude must lie in GF(32)
		}
		if mag.lo == feQ {
			return nil, false // zero magnitude ⇒ not a real error
		}
		mags = append(mags, mag.lo)
	}
	return mags, true
}
```

Add the `strings` import to `correct.go` (used by `paramsForHRP`/`reverify`). The file's import block is:
```go
import "strings"
```

- [ ] **Step 4: Run the driving test to verify it passes**

Run: `go test ./codex32/ -run TestCorrectMD1OneError -v`
Expected: PASS.

- [ ] **Step 5: Run the whole package to confirm no regressions**

Run: `go test ./codex32/`
Expected: PASS (existing tests + the new ones).

- [ ] **Step 6: Commit**

```bash
git add codex32/correct.go codex32/correct_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "codex32: BCH suggest-correction decoder, dormant (Phase A)

Port of the constellation decode_errors: syndromes -> Berlekamp-Massey ->
Chien -> Forney -> apply -> mandatory re-verify, over GF(1024). Consumes
the fork engine's residue⊕target directly (reversed MSB->LSB at the §2.6
boundary; no 128-bit). Per-HRP dispatch reuses the verifier constants
(New/ValidMD/ValidMK) — no second copy. Subs-only, unique-within-radius
or nothing; never auto-applies. No GUI caller yet.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Full parity / adversarial test battery

**Files:**
- Modify: `codex32/correct_test.go` (append)

- [ ] **Step 1: Write the multi-error, multi-code parity tests**

Append to `codex32/correct_test.go`:
```go
// applyKnown corrupts s at the given (dataPos,mask) pairs and asserts Correct
// recovers the original byte-for-byte with the expected edit count.
func assertRoundTrip(t *testing.T, valid string, pos []int, mask []fe) {
	t.Helper()
	c := valid
	for i := range pos {
		c = corruptAt(t, c, pos[i], mask[i])
	}
	if c == valid {
		t.Fatal("corruption was a no-op")
	}
	res, ok := Correct(c)
	if !ok {
		t.Fatalf("expected a correction for %d errors", len(pos))
	}
	if res.Corrected != valid {
		t.Fatalf("Corrected = %q, want %q", res.Corrected, valid)
	}
	if len(res.Edits) != len(pos) {
		t.Fatalf("len(Edits) = %d, want %d", len(res.Edits), len(pos))
	}
	for _, e := range res.Edits {
		if e.Now != valid[e.Pos] {
			t.Errorf("Edit at %d: Now=%q, want original %q", e.Pos, e.Now, valid[e.Pos])
		}
	}
}

func TestCorrectRoundTrips(t *testing.T) {
	cases := []struct {
		name  string
		valid string
		pos   []int
		mask  []fe
	}{
		{"md1/2err", tvMD1, []int{2, 14}, []fe{0b11001, 0b00111}},
		{"md1/4err", tvMD1, []int{0, 5, 11, 20}, []fe{0b00001, 0b10000, 0b11111, 0b01010}},
		{"mk1reg/1err", tvMK1Reg, []int{40}, []fe{0b10101}},
		{"mk1reg/4err", tvMK1Reg, []int{3, 17, 50, 76}, []fe{1, 16, 31, 10}},
		{"mk1long/1err", tvMK1Long, []int{60}, []fe{0b01110}},
		{"mk1long/4err", tvMK1Long, []int{0, 5, 18, 28}, []fe{0b00001, 0b10000, 0b11111, 0b01010}},
		{"ms1short/1err", tvMS1Short, []int{7}, []fe{0b01011}},
		{"ms1short/4err", tvMS1Short, []int{0, 11, 23, 44}, []fe{1, 16, 31, 10}},
		{"ms1long/4err", tvMS1Long, []int{0, 30, 60, 120}, []fe{0b00001, 0b10000, 0b11111, 0b01010}},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) { assertRoundTrip(t, tc.valid, tc.pos, tc.mask) })
	}
}
```

- [ ] **Step 2: Run them**

Run: `go test ./codex32/ -run TestCorrectRoundTrips -v`
Expected: PASS for every subcase. (If any single subcase fails on a position landing on an already-corrupted index — adjust that case's positions so all are distinct; positions must be distinct and within the data part.)

- [ ] **Step 3: Write the adversarial tests (5-error, suppress, negative cross-constant, case)**

Append:
```go
func TestCorrectFiveErrorsNotSilentOriginal(t *testing.T) {
	// >t errors: Correct must NEVER silently return the original. It may fail
	// (false) or return a different (bogus) string — the human diff-gate is the
	// backstop (SPEC §2.3). Mirrors the Rust 5-error contract.
	c := tvMK1Long
	for i, p := range []int{0, 5, 10, 15, 20} {
		c = corruptAt(t, c, p, fe(i+1))
	}
	res, ok := Correct(c)
	if ok && res.Corrected == tvMK1Long {
		t.Fatal("5-error corruption must not silently recover the original")
	}
}

func TestCorrectSuppressesUncorrectable(t *testing.T) {
	// A valid string has zero syndromes -> nothing to correct -> (_,false).
	if _, ok := Correct(tvMD1); ok {
		t.Error("a valid string must not yield a correction")
	}
	// Random garbage of a valid length may fail (expected) — but if Correct
	// ever claims a fix, the mandatory re-verify means it MUST be md-valid
	// (MINOR-3: a phantom, non-verifying "fix" is a hard failure).
	garbage := "md1" + strings.Repeat("q", len(tvMD1)-3-1) + "p"
	if res, ok := Correct(garbage); ok && !ValidMD(res.Corrected) {
		t.Errorf("Correct returned a non-re-verifying fix: %q", res.Corrected)
	}
}

func TestNegativeCrossConstant(t *testing.T) {
	// A one-error-corrupted VALID ms1 string, decoded under the md constants
	// (different POLYMOD_INIT + target), must NOT yield an md-valid string.
	// Guards against a single shared constant table cross-validating (SPEC §2.5).
	corrupted := corruptAt(t, tvMS1Short, 7, 0b01011)
	_, data := splitHRP(corrupted)
	eng := &engine{
		generator: newShortChecksum().generator,
		residue:   unpackSyms(0, mdmkPolymodInitLo, mdmkShortSyms),
		target:    unpackSyms(mdRegularTargetHi, mdRegularTargetLo, mdmkShortSyms),
	}
	if err := eng.inputHRP("ms"); err != nil {
		t.Fatal(err)
	}
	if err := eng.inputData(data); err != nil {
		t.Fatal(err)
	}
	n := mdmkShortSyms
	coeffs := make([]fe, n)
	for i := 0; i < n; i++ {
		coeffs[i] = eng.residue[n-1-i] ^ eng.target[n-1-i]
	}
	pos, mags, ok := decodeErrors(coeffs, len(data), betaGf1024, regularJStart)
	if ok {
		// If it did "decode", applying it must NOT yield an md-valid string.
		r := []rune(corrupted)
		for i, k := range pos {
			abs := 3 + k
			orig, _ := feFromRune(r[abs])
			r[abs] = rune((orig.Add(mags[i])).rune()) // (fe).rune() returns byte
		}
		if ValidMD("md" + string(r)[2:]) {
			t.Fatal("ms data cross-validated under md constants")
		}
	}
	// Positive control (MINOR-2): the SAME corrupted ms1 string MUST correct
	// under its own (ms) constants — so this test isn't merely vacuous.
	res, ok := Correct(corrupted)
	if !ok || res.Corrected != tvMS1Short {
		t.Fatalf("ms1 should self-correct under ms constants: ok=%v got=%q", ok, res.Corrected)
	}
}

func TestCorrectCasePreserved(t *testing.T) {
	// Uppercase input must yield an uppercase, re-verifying correction.
	upper := strings.ToUpper(tvMD1)
	corrupted := corruptUpper(t, upper, 5, 0b10101)
	res, ok := Correct(corrupted)
	if !ok {
		t.Fatal("expected a correction")
	}
	if res.Corrected != upper {
		t.Fatalf("Corrected = %q, want %q", res.Corrected, upper)
	}
}

// corruptUpper is corruptAt for an uppercase string (emits an uppercase char).
func corruptUpper(t *testing.T, s string, dataPos int, mask fe) string {
	t.Helper()
	r := []rune(s)
	abs := 3 + dataPos
	orig, ok := feFromRune(r[abs])
	if !ok {
		t.Fatalf("bad seed char %q", r[abs])
	}
	r[abs] = rune(feToByte(orig.Add(mask), true))
	return string(r)
}
```

- [ ] **Step 4: Run the full package**

Run: `go test ./codex32/ -v`
Expected: PASS across the field self-tests, all round-trips, and the adversarial battery. Investigate any failure (a likely cause: a chosen corruption position colliding with another or falling outside the data part — fix the test's positions, never weaken `Correct`).

- [ ] **Step 5: go vet**

Run: `go vet ./codex32/`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add codex32/correct_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "codex32: BCH decoder parity + adversarial tests (Phase A)

Round-trip parity for ms/md/mk (1/2/4-error, regular + long), the
5-error non-silent contract, suppress-when-uncorrectable, the negative
cross-constant test (ms data under md constants must not validate), the
orientation pin (asymmetric single error), and case preservation.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: TinyGo device-build coverage in CI

**Files:**
- Modify: `.github/workflows/*` (the existing CI workflow)

- [ ] **Step 1: Locate the CI workflow and any existing TinyGo step**

Run: `ls .github/workflows/ && grep -rn "tinygo" .github/workflows/ || echo "no tinygo step yet"`
Expected: lists the workflow file(s); shows whether a TinyGo build already runs.

- [ ] **Step 2: Ensure a TinyGo build exercises `codex32`**

The `codex32` package is imported by `gui`, so a TinyGo build of the firmware main compiles all of `codex32` (including `gf1024.go`/`correct.go`, even dormant). If a firmware TinyGo build already exists, confirm it targets `pico-plus2` and add a comment noting it covers `codex32`. If none exists, add a step (in the existing build job, after Go setup + TinyGo install):

```yaml
      - name: TinyGo device build (covers codex32)
        run: tinygo build -o /dev/null -target=pico-plus2 .
```

(Use the repository's established TinyGo install action/version; match the existing job's runner and Go/TinyGo setup steps. Do not introduce a new workflow file if one exists — extend it.)

- [ ] **Step 3: Validate the workflow locally if possible**

Run (if TinyGo is on the host): `tinygo build -o /dev/null -target=pico-plus2 . 2>&1 | tail -5`
Expected: builds cleanly (the new `codex32` code compiles under TinyGo — confirms no `math/big`, no 64-bit-only assumptions, fixed arrays OK). If TinyGo is not on the host, state that CI will enforce it.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "ci: TinyGo pico-plus2 build covers codex32 (Phase A)

Ensures the new GF(1024)/BCH decoder compiles for the device target
(the Slice-1 lesson — host go test never compiles the TinyGo build).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Done criteria (Phase A)

- `go test ./codex32/` and `go vet ./codex32/` green; the field self-tests, all round-trip parity, and the adversarial battery pass.
- TinyGo `pico-plus2` build compiles `codex32`.
- `Correct` ships **dormant** (no GUI caller) — Phase B wires it in a separate gated cycle.
- After all tasks: a mandatory whole-diff adversarial execution review (R0 = plan correctness; this catches implementation-introduced regressions), persisted verbatim to `design/agent-reports/`, then merge no-ff signed+DCO into fork `main` and push `bg002h`.

---

## Self-review (against the spec)

- **§3.1 GF(1024) on `fe.Mul`** → Task 1 `gf1024.go` + the carryless cross-check pinning the field substitution. ✔
- **§3.2 pipeline + every guard** (`deg==0||>4`, Chien root-count, Forney 3 guards, re-verify) → Task 2 `decodeErrors`/`forney`/`reverify`. ✔
- **§2.5 per-code-constant integrity** (reuse verifier constants, no second copy; Rust-vector-only; negative cross-constant) → `paramsForHRP` reuses `newShort/LongChecksum`/`unpackSyms`/mdmk targets; `TestNegativeCrossConstant`. ✔
- **§2.6 orientation** (one canonical orientation + boundary conversion, Rust-pinned) → `coeffs[i]=v[n-1-i]` reverse + `k=L-1-d`; `TestCorrectMD1OneError_OrientationPin`. ✔
- **§2.7 TinyGo** (uint8/uint16 internals via `fe`/`gf1024`; no `math/big`/128-bit; fixed BM arrays; `deg>4` bound; CI tinygo build; M-3 `[9]`/`[8]` sizing) → `gf1024`/`berlekampMassey` fixed arrays + Task 4. ✔
- **§2.1/§2.2 no-auto-apply + mandatory re-verify** → `Correct` returns data only; `reverify` gate; dormant (no engrave path). ✔
- **§3.3 ships dormant** → no caller added; Task list ends before any GUI wiring. ✔
- **§7 TDD (Rust parity vectors, all three codes, 1/2/4-sub + 5-error reject, negative cross-constant, field self-tests, orientation pin)** → Tasks 1/3. ✔

No placeholders; every code step shows complete code; types (`gf1024`, `Edit`, `CorrectionResult`, `bchParams`) are consistent across tasks.
