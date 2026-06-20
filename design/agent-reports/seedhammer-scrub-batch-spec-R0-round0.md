# Track B (scrub batch) SPEC — R0 review (round 0) — VERBATIM agent report

**Agent:** `a446aa6421d36faa7` (adversarial opus architect; applied all 4 fixes + probed defer/aliasing semantics in a throwaway worktree off `3a23dbb`). **Fork HEAD:** `3a23dbb`. **Spec commit:** `8cd9cd9`. **Date:** 2026-06-20.
**Verdict:** GREEN (0 Critical / 0 Important). 3 advisory Minors; Q1/Q2/Q3 ruled. Cleared for the plan phase (own plan-R0 follows). Part of the 8-finding fix; concurrent with Track A.

---

# R0 GATE REVIEW — SPEC_seedhammer_scrub_batch_fixes.md (Track B)

**Reviewer:** Adversarial opus architect (R0 gate)
**Fork HEAD:** `3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082` (`bg002h/seedhammer` `main`) — verified live
**Spec commit:** `8cd9cd9`
**Verdict:** **GREEN — 0 Critical / 0 Important.** (3 Minors, all advisory; Q1/Q2/Q3 ruled below.)

---

## Mandate 1 — Every cited fact verified at `3a23dbb`

All line numbers and code facts in the SPEC match the live source. Evidence:

**M2 — `slip39/combine.go`:** `recoverSecret(mt, pts)` at `:101`; error path (a) `return nil, err` at `:103`; `groupShares = append(...)` at `:105`; `len(groupShares) != first.GroupThreshold` → `return nil, errInsufficientShares` at `:107-108` (path b); `ems, err := recoverSecret(...)` at `:114`; path (c) `return nil, err` at `:116`; success-path scrub loop `for _, gs := range groupShares { wipe(gs.v) }` + `wipe(ems)` at `:119-122`. Confirmed: all three error returns skip the scrub — only `:119-122` (success) scrubs. `recoverSecret` digest-fail branch `:138-141` wipes `s` only (`wipe(s)` at `:139`); success-path `wipe(d)` at `:142`. `d` is NOT wiped on the digest-fail path — confirmed verbatim. `gv` is a fresh allocation on both `recoverSecret` branches (`append([]byte(nil), shares[0].y...)` at `:130`; `interpolateSecretAt`→`make([]byte,n)` at `lagrange.go:39-50`, no input aliasing) — leaking it leaks a real secret copy, not an input alias.

**M3 — `seedxor/seedxor.go`:** package-local `wipe` at `:25-29`; `out := append([]byte(nil), parts[0].Entropy()...)` at `:38`; bad-length `wipe(out); return nil, errBadLength` at `:40-41`; `e := p.Entropy()` at `:44`; `errMismatchedLengths` early return at `:45-47` (`wipe(out)` only — `e` left live); success `wipe(out)` at `:54`. `Entropy()` is a fresh allocation: `splitMnemonic`→`ent.Bytes()`→`append(padding, entBytes...)` at `bip39/bip39.go:193-195`. So both `e0` and each `e` are genuine un-wiped secret copies.

**M4 — `gui/bip85.go`:** `pkey, err := k.ECPrivKey()` at `:101`; `priv := pkey.Serialize()` at `:106`; `k.Zero()` at `:107`; `defer wipeBytes(priv)` at `:108`; `pkey` never zeroed. Authoritative type facts verified against the module cache (`decred/dcrd/dcrec/secp256k1/v4@v4.4.1/privkey.go`): `type PrivateKey struct { Key ModNScalar }` at `:16`; `func (p *PrivateKey) Zero()` pointer receiver at `:98`, documented "against memory scraping" (`:95-96`), body `p.Key.Zero()`; `func (p PrivateKey) Serialize() []byte` value receiver at `:107` (so `Serialize()` copies the scalar; `pkey.Key` stays live). `btcec/v2@v2.4.0/privkey.go:14` `type PrivateKey = secp.PrivateKey` (alias); `hdkeychain/extendedkey.go:546` `ECPrivKey() (*btcec.PrivateKey, error)`. `pkey.Key.IsZero()` exists (`modnscalar.go:195`, pointer receiver) — relevant to the Q1 ruling.

**L1 — `gui/codex32_polish.go`:** `_, _, _, msErr := codex32.DecodeMS1(scan)` at `:103` (entropy discarded with `_`); `showSecret := f.Unshared && msErr == nil` at `:104`. `DecodeMS1` signature `(prefix, language int, entropy []byte, err error)` at `codex32/mspayload.go:34`; the entropy return is a subslice (`data[1:]`/`data[2:]`) of a fresh `make`-backed buffer (`parts.data()` at `codex32/codex32.go:417`, no caching).

**`wipeBytes`** at `gui/slip39_polish.go:344` ranges-and-zeroes; `wipeBytes(nil)` is a safe no-op.

---

## Mandate 2 — Each fix is CORRECT and COMPLETE (every leak path)

I applied all four fixes in the throwaway worktree, built, vetted, and ran the existing suites — all green, no behavior regression (`go test ./slip39/ ./seedxor/ ./gui/` → ok).

- **M2 defer covers all paths:** probed defer-capture semantics — a `defer func(){ for _, gs := range groupShares { wipe(gs.v) }; wipe(ems) }()` registered right after `groupShares` is declared does observe entries appended before an early `return` (probe `TestR0DeferCapturesAppended` PASS). So it fires on success + all three error returns (a/b/c) AND any future early return. The `ems` hoist to a function-scope `var` is required so the defer sees it; `ems` is `nil` on paths a/b (defer wipes `nil` → no-op) and holds the secret only after `:114` succeeds — correct on every path. Multi-group vectors exist (testdata idx 17: GroupThreshold=2, GroupCount=4), so a real multi-group `Combine` test is constructible.
- **M2 `recoverSecret` `d`-wipe:** adding `wipe(d)` to the digest-fail branch covers the one missed path; `d` is always non-nil there (only reachable on the threshold>1 path where `d := interpolateSecretAt(...)` ran). Complete.
- **M3 complete on all 3 paths:** `wipe(e0)` immediately after the copy covers the bad-length return (`:40-41`); per-iteration `wipe(e)` at end-of-body covers success; explicit `wipe(e)` before the `errMismatchedLengths` return covers `:45-47`. Verified `out := append([]byte(nil), e0...)` allocates a distinct backing array (probe: wiping `e0` leaves `out` intact), so the early `wipe(e0)` is safe. The SPEC's "discouraged defer-in-loop" guidance is correct (loop defers accumulate to function end, delaying the wipe).
- **M4 complete:** `defer pkey.Zero()` placed after `:106` (where `pkey` is guaranteed non-nil — the `err != nil` branch at `:102-105` already returned) fires on the success return (`:118-119`) and the `entLen` guard (`:114-116`). The early `:102-105` path doesn't register it (pkey nil/unused) — correct. The value-receiver `Serialize()` copy is independently covered by `wipeBytes(priv)`.
- **L1 complete:** `wipeBytes(ent)` after the single use covers both decode-OK and decode-err (nil → no-op).

No public output / signature / return / control-flow change in any fix — confirmed: all are additive `wipe`/`Zero`/`defer` on dead or discarded buffers; existing tests pass unchanged.

---

## Mandate 3 — Rulings on Q1 / Q2 / Q3

**Q1 (test seam) — RULING, per finding.** Confirmed structurally that `Combine`'s `groupShares[].v`/`ems`, `recoverSecret`'s `d`, and `seedxor.Combine`'s per-part `e`/`e0` are NOT observable in-package without a production seam (each is a function-local; the functions return only their public result — probe `TestR0CannotObserveD` confirms no handle on `d`). The repo has no existing test-only seam in `slip39`/`seedxor` (unlike `gui`'s `bip85SeedHook`). Ruling:

- **M2 (`recoverSecret` `d`-wipe):** ACCEPTABLE WITHOUT A NEW SEAM. Extend `TestRecoverSecretWipesOnDigestFail` (`vectors_test.go:282`) with a white-box assertion (regression+convention guard). A genuine fail-before/pass-after on `d` is impossible seam-free; the SPEC's §1.4 fallback (white-box where reachable, else flag) is correct — flag accepted, no seam.
- **M2 (`Combine` `gv` leak, paths a/b/c):** SEAM-FREE regression+convention guard sufficient. Load-bearing test: construct multi-group sets hitting `:103`/`:108`/`:116`, assert the correct sentinel error on each + a `TestWipeZeroes`-style guard that the `defer` form is present. A direct `gv`-zeroed assertion would require a seam — not warranted for a MEDIUM best-effort scrub of a group-level Shamir share.
- **M3 (`e`/`e0`):** SEAM-FREE sufficient — strongest assertion is the existing `TestCombineMismatchedLengths` extended to confirm `out` is zeroed on the early-return path + a convention note. No seam.
- **M4 (`pkey`):** true fail-before/pass-after IS cheaply achievable via a sanctioned, precedented hook — ruled WARRANTED but OPTIONAL, implementer's choice. `pkey.Key.IsZero()` exists; a `bip85PkeyHook func(*btcec.PrivateKey)` mirroring the already-sanctioned `bip85SeedHook` (same file, same reviewed pattern) would let a test assert `pkey.Key.IsZero()==true` after `deriveBip85Child` returns (FAIL on `3a23dbb`, PASS after). Because the precedent is in this very file, adding it is NOT a unilateral design smell. A seam-free no-regression guard is also acceptable; lean toward the hook for M4 only (the lone genuine fail-before/pass-after in the batch).

  Net Q1: No NEW seam is required for any finding. One optional seam (M4's `pkey` hook) is sanctioned by direct in-file precedent. Not a blocker.

**Q2 (M2 `wipe(d)` placement) — RULING:** Add `wipe(d)` to the digest-fail branch only, leaving the success-path `:142` `wipe(d)` as-is. Lower diff, no behavior change. (SPEC recommendation accepted.)

**Q3 (M3 `e0` placement) — RULING:** Wipe `e0` immediately after the copy, before the `interopLen` check. Verified `out` is a distinct allocation, so this leaves `out` intact and ensures the bad-length return (`:40-41`) also leaves `e0` wiped. (SPEC recommendation accepted and verified.)

---

## Mandate 4 — Shared-helper + disjointness invariants

- No `wipeBytes` edit, no new helper: confirmed. M4/L1 only call `wipeBytes` (`gui/slip39_polish.go:344`); M2 calls `slip39.wipe`; M3 calls `seedxor.wipe`. No fix edits or adds a helper.
- Four files pairwise-disjoint: `slip39/combine.go`, `seedxor/seedxor.go`, `gui/bip85.go`, `gui/codex32_polish.go` — no two findings share a file; the two `gui` co-residents (M4 in `bip85.go`, L1 in `codex32_polish.go`) are different files/functions.
- Disjoint from Track A's files (`bundle/verify.go`, `gui/multisig_verify.go`, `gui/multisig_supply.go`, `gui/singlesig_verify.go`, `gui/md1_gather.go`): zero overlap. A∥B concurrency basis holds.
- L1's other two sites NOT in this spec: confirmed — `gui/singlesig_verify.go:116` and `gui/multisig_verify.go:93` are explicitly excluded (SPEC §0, §4.1); Track B owns only `codex32_polish.go:103`.

---

## Mandate 5 — Firmware-only / TinyGo-safe / no false claims

- `defer`/closures (M2), `defer pkey.Zero()` (M4), in-loop `wipe` (M3), and `wipeBytes` (L1) all compile on the TinyGo device target. SPEC correctly names the TinyGo device build CI as the final gate (not host `go build`/`go test`). I verified host build/vet/test only (clean); did NOT run the TinyGo build — SPEC defers that to the integration gate, which is correct.
- No false claims: `pkey.Zero()` exists (pointer receiver, scrubs `pkey.Key`), `Serialize()` is value-receiver, `wipeBytes(nil)` is a no-op, `Entropy()` returns fresh allocations — all verified at source.

---

## Critical findings
None.

## Important findings
None.

## Minor findings (advisory; do not block GREEN)

1. **M2 `ems` hoist / `err` scoping (implementation guidance).** `ems` must be hoisted to a function-scope `var ems []byte` for the defer to capture it, AND the existing `ems, err := recoverSecret(...)` at `:114` must remain `:=` (not `=`) — naively changing to `=` breaks compilation (`err` was only declared via that `:=`, the loop-body `gv, err :=` is iteration-scoped). With `ems, err := ...` (reusing pre-declared `ems` because `err` is new) it builds clean. Plan should lock the exact form. Pure mechanics; invariant unaffected.

2. **M2/M3/L1 tests are convention/regression guards, not true fail-before/pass-after on the leaked locals (per Q1).** The plan should state plainly that for M2-`gv`, M2-`d`, M3-`e`/`e0`, and L1-`ent`, the load-bearing test is a seam-free regression+convention guard (correct sentinel/behavior + helper-present), NOT a buffer-zeroed assertion — because those buffers are unobservable seam-free. Acceptable for best-effort scrubs but should be explicit so the exec reviewer doesn't expect a stronger assertion than the seam-free path can deliver.

3. **L1 entropy is a subslice, not the whole `data` buffer.** `wipeBytes(ent)` zeroes the entropy region (`data[1:]`/`data[2:]`) but not `data[0]`/`data[1]` (prefix/language — non-secret). Correct and matches `ms1_decode.go`; worth a one-line note so no one later "fixes" it to wipe the whole `data` slice (which the function doesn't hold a handle to anyway).

---

## Verified-correct list
- M2 defer-scrub fires on success + all three error returns (probe-confirmed) and `recoverSecret` `d`-wipe completes the digest-fail path.
- M3 wipes `e0` (bad-length path safe — `out` is a distinct allocation, probe-confirmed) and each per-part `e` on success + `errMismatchedLengths`.
- M4 `defer pkey.Zero()` scrubs the live leaf scalar; `Zero()` is a real pointer-receiver method documented for memory-scraping; `Serialize()` value-receiver copy independently covered.
- L1 `wipeBytes(ent)` zeroes the discarded probe entropy; nil-safe on the err path.
- All four: no public output/signature/return/control-flow change; existing `slip39`/`seedxor`/`gui` suites pass unchanged; host build + vet clean.
- No `wipeBytes` edit; no new helper; four files pairwise-disjoint and disjoint from Track A's five files; L1's two Track-A sites excluded.

---

## Bottom line

The SPEC is factually accurate against `3a23dbb` (every line number checked), the four fixes are correct, complete across all leak paths (probe-confirmed for the M2 defer and M3 `e0`/`out` aliasing), and strictly additive with no behavior change. The disjointness invariant that underpins A∥B concurrency holds. The one genuinely non-mechanical question — Q1 test observability — resolves cleanly: no new seam is required; seam-free regression+convention guards are the load-bearing tests for the unobservable locals, and M4 may optionally use a precedented `bip85SeedHook`-style hook for a true fail-before/pass-after. The three Minors are implementation-locking guidance the plan should fold but none blocks the gate.

**VERDICT: GREEN — 0 Critical / 0 Important.** Track B may proceed to the implementation-plan phase (which carries its own R0 plan-gate). Recommend folding the three Minors into the plan for precision.

**Fork state:** clean on `main @ 3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`; `/tmp/r0-trackB` worktree removed and pruned. The pre-existing `/tmp/r0-trackA` (Track A reviewer) and `seedhammer-wt-bip39` worktrees left untouched.
