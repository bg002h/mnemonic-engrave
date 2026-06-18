<!--
Persisted verbatim. opus-architect MANDATORY whole-diff execution review of the Phase A
implementation (worktree seedhammer-wt-mstar-a, branch feat/mstar-correct-decoder, 4 commits over
04a1e95), BEFORE merge. Reviewer agentId ae611563684297cd3. Verdict: GREEN 0C/0I (1 minor, doc).
The reviewer independently re-ran go test (38 pass/0 fail/0 skip), go vet, go build (all clean);
audited the committed crypto against the Rust oracle (gf1024.mul, inv=pow(1022), constants, §2.6
orientation reversal, k=L-1-d, BM fixed-array snapshot ordering + zero-fill invariant + bound-9
guard, Chien/Forney guards, paramsForHRP constant reuse) — all faithful; ran its own panic-probe
inputs (empty/"1"/embedded-1/uppercase/non-alphabet/multibyte/reserved-length) with no panics and
re-verify always load-bearing; validated the new TinyGo CI job with actionlint + the
cmd/controller->gui->codex32 import chain (genuinely compiles the new code on pull_request, not a
false-green); confirmed §2.1/§2.2/§2.5 invariants and DORMANT (no non-test caller). The single
MINOR: Edit.Pos is documented "full-string byte index" but is a rune index (== byte for ASCII
bech32; Phase B should treat it as a rune index). Disposition: folded the doc fix; re-ran the
suite green; merged. The text below is the agent's report exactly as returned.
-->

# WHOLE-DIFF EXECUTION REVIEW — m*1 BCH correction (Phase A)

Reviewer: opus architect. Worktree: `/scratch/code/shibboleth/seedhammer-wt-mstar-a`, branch `feat/mstar-correct-decoder`, 4 commits over base `04a1e95`. Diff: `gf1024.go`(+63), `gf1024_test.go`(+110), `correct.go`(+380), `correct_test.go`(+212), `.github/workflows/test.yml`(+16). I did not edit, merge, or push. A temporary probe test was created and removed; the worktree is clean.

## Verification Results

**Test suite (re-run by me, not the implementer's report).** Go is at `/home/bcg/.local/go/bin/go` (go1.26.4). `go test ./codex32/ -v -count=1`:
- **PASS: 38, FAIL: 0, SKIP: 0** (12 new top-level test funcs + 9 `TestCorrectRoundTrips` subtests + pre-existing codex32 tests). `grep t.Skip codex32/` → none. Package result: `ok seedhammer.com/codex32`.
- All the required adversarial tests are present and actually run (confirmed in verbose output, none skipped): `TestCorrectMD1OneError_OrientationPin`, `TestCorrectRoundTrips` (md/mk-reg/mk-long/ms-short/ms-long, 1/2/4-error), `TestCorrectFiveErrorsNotSilentOriginal`, `TestCorrectSuppressesUncorrectable`, `TestNegativeCrossConstant`, `TestCorrectCasePreserved`. Field self-tests: `TestForkGf32MatchesCarryless`, `TestAlphaPowersMatchInvLogTable`, `TestZetaCubeRoot`, `TestBetaOrder93`, `TestGammaOrder1023`, `TestGeneratorConsecutiveRoots`.
- `go vet ./codex32/` → exit 0 (clean). `go build ./...` → exit 0.

**Folded minors present as committed:** MINOR-1 `rune((...).rune())` wrap (`correct_test.go:32,174`); MINOR-2 positive control (`correct_test.go:180-184`); MINOR-3 hard `ValidMD` assert (`correct_test.go:140-141`).

**Crypto transcription audit (committed code vs Rust oracle `bch_decode.rs`):**
- `gf1024.mul` identity `lo=ll^hh, hi=lh^hl^hh` (`gf1024.go:46-52`) == Rust `bch_decode.rs:163-172`. ✓ `inv=pow(1022)` (`gf1024.go:65`) == Rust:187-191. ✓
- Constants pinned against fork field values (feQ=0,feP=1,feZ=2,feX=6,feG=8,feE=25 in `gf32.go:52-85`): `betaGf1024={feQ,feG}={0,8}`, `gammaGf1024={feE,feX}={25,6}`, `gf1024One={feP,feQ}={1,0}`, `gf1024Zero={0,0}`, jstarts 77/1019 (`gf1024.go:15-27`) == Rust:204-217. ✓
- §2.6 orientation: `coeffs[i]=residue[n-1-i]^target[n-1-i]` (`correct.go:102-103`). Fork engine residue/target are MSB-first (index 0 = highest power, per `checksum.go:156-170` + `unpackSyms` `mdmk.go:65-84`); reversing yields LSB-first `coeffs[i]=x^i` = Rust's `(residue>>5i)&0x1F`. ✓ Fork `residue` is the raw polymod (isValid only *compares* to target, never XORs — `checksum.go:72-74`), so `residue⊕target` == Rust `polymod⊕const`. ✓ `k=L-1-d` (`correct.go:210`), `offset=len(hrp)+1` (`correct.go:112`) == Rust:579-587 + `splitHRP` first-`1` cut (`codex32.go:453`). ✓
- BM fixed-array port (`correct.go:256-308`) is a faithful transcription of the Rust vec port (`bch_decode.rs:327-381`): discrepancy loop guard `i<=k && i<lamLen`, `d.isZero()⇒m++/continue`, `scale=d*b.inv()`, `newLen=max(lamLen,prevLen+m)`, increase-branch `t:=lam` snapshot taken BEFORE in-place update then `prev=t` (matches Rust `let t=lam.clone()` placement), `l=k+1-l`, trailing-zero trim. The fixed-array zero-fill invariant (newly-activated slots `[oldLamLen,newLen)` are guaranteed never previously written, because `lamLen` is monotonic in-loop and all writes land `<newLen`) holds — equivalent to Rust's `resize(_,ZERO)`. The bound-9 overflow guard (`correct.go:281-283`) rejects only inputs that can't be correctable (deg≤4⇒lamLen≤5); for >4 errors both implementations return None/false. ✓
- Chien (`correct.go:312-330`) and Forney (`correct.go:335-380`) match Rust:395-496: root-count==deg guard, Ω=S·Λ mod x⁸ over `[8]`, char-2 derivative odd-terms with `lamPrime[:lpLen]` length == `lambda.len()-1`, `shift=jStart-1`, `xkShift=xkInv.pow(shift)`, all three Forney guards (`Λ'(X⁻¹)=0`, `mag.hi≠0`, `mag.lo=0`). ✓
- `paramsForHRP` (`correct.go:42-79`) reuses the verifier's exact constants — `newShort/LongChecksum().generator`, `unpackSyms(0,mdmkPolymodInitLo,n)`, mdmk targets — no second copy; ms dispatch (`48..93`/`125..127` total) and mk dispatch (`14..93`/`96..108` data) mirror `New` (`codex32.go:101-104`) and `ValidMK` (`mdmk.go:138-148`). ✓

**Independent bug hunt (probe tests, then removed):** empty/`"1"`/`md`/`md1`/embedded-`1`/uppercase/non-alphabet-char/multibyte-rune/reserved-length(94,95) inputs — **no panics**; `Correct` returns `(_,false)` or a result that re-verifies. `feFromRune` is bounds-checked (`gf32.go:126-135`); `(fe).rune()` indexes with `e∈[0,32)` invariant preserved through XOR. `offset+k` is bounds-guarded (`correct.go:117`). When `ok=true`, the result always re-verifies (re-verify is load-bearing). No aliasing: each `paramsForHRP` allocates a fresh engine with fresh `unpackSyms`/`generator` slices, and `coeffs` is a fresh `make`.

**CI deviation (the unvalidated piece):** `actionlint .github/workflows/test.yml` → exit 0 (well-formed, both jobs valid). Triggers `pull_request` + `push` (lines 3-4) ⇒ pre-merge enforcement. Flags exactly match `flake.nix:80` `tinygo-flags` (`-target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks`); entrypoint `./cmd/controller`. `nix develop` enters the `default` devShell which provides `tinygo` (`flake.nix:139-151`). Import chain verified: `cmd/controller/main.go` (`//go:build tinygo && rp`) imports `seedhammer.com/gui`; `gui/gui.go`/`codex32_polish.go`/`scan.go` (package gui, no build tag) import `seedhammer.com/codex32` unconditionally (`go list -deps ./gui` shows codex32). TinyGo compiles whole imported packages, so `gf1024.go`/`correct.go` ARE type-checked and compiled under the device target — the job is not a no-op or false-green. The only difference from `build-firmware` is the omitted `-ldflags=-X main.Version` (harmless; version injection only).

**Spec-invariant spot check:** §2.1 no auto-apply — `Correct` returns data only, no engrave path, dormant. §2.2 mandatory re-verify present and load-bearing (`correct.go:130-132`, dispatches the same `New`/`ValidMD`/`ValidMK`). §2.5 no second copy of per-code constants (verified above). §3.3 dormant — `grep` confirms **no** non-test caller of `Correct`/`CorrectionResult`/`Edit` anywhere in the tree.

**Process:** all 4 commits authored+committed Brian Goss, with `Signed-off-by` (DCO) and `Co-Authored-By: Claude` trailers, base `04a1e95`. (`sig: N` is a host verification-config limitation — no `allowedSignersFile` locally — not evidence of unsigned commits.)

## Findings

### CRITICAL
None.

### IMPORTANT
None.

### MINOR
- **MINOR-1 (doc imprecision, non-blocking).** `correct.go:5-9`: `Edit.Pos` is documented as "the full-string byte index" but is actually a **rune index** into `[]rune(frag)` (set from `abs` indexing `r := []rune(frag)`, `correct.go:113-127`). For valid codex32/bech32 strings (pure ASCII) rune index == byte index, so values coincide and behavior is correct. The Phase B GUI consumer should treat `Pos` as a rune index (or the comment should say so). No functional defect in Phase A.

## Verdict

**GREEN — 0 Critical / 0 Important.**

The committed code is a faithful transcription of the R0-reviewed plan and the Rust oracle: the GF(1024) algebra, the BM fixed-array port (including the snapshot ordering and zero-fill invariant), Chien/Forney guards, orientation reversal, `k=L-1-d`/offset translation, and constant-reuse all check out. The suite is real (38 pass / 0 fail / 0 skip, vet+build clean), the adversarial tests run, the mandatory re-verify is load-bearing, the feature ships dormant, and the new TinyGo CI job genuinely compiles the new code on `pull_request` (not a false-green). The lone MINOR is a documentation nit that does not affect Phase A behavior. Cleared to merge.
