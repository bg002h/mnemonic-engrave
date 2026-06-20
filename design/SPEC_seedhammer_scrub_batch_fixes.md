# SPEC — Track B: secret-scrub batch fixes (M2, M3, M4, L1-codex32_polish)

**Status:** R0-ready brainstorm spec (single author). **NOT a plan; no code.**
**Author:** Track B spec author (read-only on the fork).
**Base:** fork `bg002h/seedhammer` `main` @ **`3a23dbb`** (verified live: `3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`).
**Findings source:** `design/agent-reports/seedhammer-fork-own-code-bughunt.md` (M2, M3, M4, L1).
**Orchestration:** `design/seedhammer-own-code-fix-orchestration-plan.md` — Track B = branch `fix/scrub-batch`, light, one batched spec/plan/R0.
**Toolchain confirmed:** host `go version go1.26.4`; `go build ./slip39/... ./seedxor/... ./gui/...` is clean at `3a23dbb`.
**Date:** 2026-06-20.

---

## 0. Scope, theme, and non-goals

Track B is a **pure additive secret-scrub batch**: four mechanical, local, pairwise-disjoint fixes that add zeroing of already-discarded secret-bearing buffers. Each fix follows the package's **existing** scrub helper. No public output, no function signature, no return value, and no control-flow behavior changes (the scrubs are additive `wipe`/`Zero` calls on dead values and `defer`s that fire after the existing return decisions). This is the basis for safe A∥B concurrency and for in-track sequential single-implementer TDD in one worktree.

**The four findings and the helper each uses (all helpers already exist — Track B only CALLS them):**

| # | File:line @ `3a23dbb` | Package | Existing helper to call |
|---|---|---|---|
| **M2** | `slip39/combine.go:101-123` (+ `recoverSecret` `:128-144`) | `slip39` | package-local `wipe` (`combine.go:139,142` style) |
| **M3** | `seedxor/seedxor.go:38,44` | `seedxor` | package-local `wipe` (`seedxor.go:25-29`) |
| **M4** | `gui/bip85.go:101-108` | `gui` | `pkey.Zero()` (method on `*btcec.PrivateKey`); the existing `wipeBytes` covers the byte slices already |
| **L1** | `gui/codex32_polish.go:103` | `gui` | `wipeBytes` (`gui/slip39_polish.go:344`) |

**Non-goals / hard constraints (carried from the orchestration plan):**
- **Do NOT edit the shared helper `wipeBytes` (`gui/slip39_polish.go:344`).** Track B only *calls* it. If any fix appears to need a new helper or a new test seam, that is a **design smell to flag at R0, not implement** (see §6 open questions).
- **Do NOT touch the Track-A L1 sites** `gui/singlesig_verify.go:116` or `gui/multisig_verify.go:93`. Track B owns **only** the disjoint `gui/codex32_polish.go:103` site. This split is what keeps the tracks file-disjoint.
- **Firmware-only, TinyGo-safe.** `defer`/closures compile on the device target, BUT host `go build`/`go test` is **not** a sufficient gate — the final integration gate is the **TinyGo device build CI**. The spec assumes nothing about host build alone.

**Pairwise-disjoint files** (verified): `slip39/combine.go`, `seedxor/seedxor.go`, `gui/bip85.go`, `gui/codex32_polish.go` (+ their `_test.go`). No two findings share a file. → in-track sequential implementation in one worktree is conflict-free; the only `gui`-package co-residents (M4, L1) live in different files and touch different functions.

**Best-effort caveat (applies to all four):** TinyGo's GC may copy/retain regardless; under TinyGo's non-moving GC the un-scrubbed objects simply *persist* (which strengthens, not weakens, the case). These fixes are defense-in-depth for consistency with the codebase's own stated `wipeBytes`/`wipe` contract and to minimize the in-RAM residency window — not a guarantee. Tests therefore assert the buffer **the code itself controls** (the named local slice / the `PrivateKey` object), not GC-copied ghosts (see §5 test-design note).

---

## 1. M2 — SLIP-39 `Combine`: scrub recovered group-share secrets on ALL error/abort paths; wipe `d` on digest-fail

### 1.1 Verified facts (live `slip39/combine.go` @ `3a23dbb`)

`Combine` (`:39-124`):
- The recovered group-share secrets are accumulated as `groupShares []gshare` where `gshare.v []byte` is a **fresh** secret-bearing allocation per group:
  - `gv, err := recoverSecret(mt, pts)` at **`:101`**, appended at `:105` (`groupShares = append(groupShares, gshare{byte(g), gv})`).
  - `recoverSecret` returns a fresh slice on both branches: threshold==1 → `append([]byte(nil), shares[0].y...)` (`:130`); threshold>1 → `interpolateSecretAt(shares, secretIndex)` (`:132`, a `make`-backed buffer). So `gv` is **not** an alias of an input share — leaking it leaks a real secret copy.
- The EMS (`ems`) is itself a fresh secret slice from `recoverSecret(first.GroupThreshold, gpts)` at **`:114`**.
- The scrub of `groupShares[].v` and `ems` happens **only on the success path** at **`:119-122`**:
  ```
  for _, gs := range groupShares { wipe(gs.v) }
  wipe(ems)
  ```
- **Three error returns skip that scrub** (the bug):
  - **(a) `:103`** `return nil, err` — a later group's `recoverSecret` fails its digest; **prior groups' `gv` (already appended to `groupShares`) stay live.**
  - **(b) `:108`** `return nil, errInsufficientShares` — recovered-group count `len(groupShares) != first.GroupThreshold`; **all accumulated `gv` leak.**
  - **(c) `:116`** `return nil, err` — the group-layer `recoverSecret(... gpts ...)` digest fails; **every `groupShares[].v` leaks** (and `ems` is `nil` here, so no `ems` leak on this path, but the group `gv`s do leak).

`recoverSecret` (`:128-144`):
- Success branch wipes `d` at **`:142`** (`wipe(d)`), then returns `s`.
- Digest-fail branch (`:138-141`) wipes **only `s`** (`wipe(s)`) and returns the sentinel `errDigestVerificationFailed` — **`d` (`digest‖random`, interpolated from secret-bearing points) is NOT wiped on this path.** `d` is `interpolateSecretAt(shares, digestIndex)` at `:133`, a fresh secret-derived buffer.

**Trigger (verified reachable):** on-device multi-group SLIP-39 Recover with `GroupThreshold ≥ 2`. A member-threshold-1 group carries the group share directly (no member-level digest), so a transcription error surviving the detection-only RS1024 checksum "recovers" at the member layer then fails the group-layer digest at `:114-116` → path (c). A 2-group set where group A verifies and group B's member digest fails → path (a), leaking A's `gv`. A short set → path (b).

**Existing test coverage gap:** `slip39/vectors_test.go:282-298` (`TestRecoverSecretWipesOnDigestFail`) asserts only that `recoverSecret` returns a **nil secret** on digest failure — it does **not** assert the `d` buffer is zeroed, and there is no `Combine`-level test for the three error-path `gv` leaks. The tests are in-package (`package slip39`, `combine_test.go:1`), so they can call `Combine`/`recoverSecret` and the `wipe` helper directly and inspect internal buffers.

### 1.2 The fix

- In `Combine`, move the `groupShares[].v` and `ems` scrub into a `defer` that fires on **all** return paths (success + the three error returns + any future early return). Style: a deferred closure that ranges `groupShares` calling `wipe(gs.v)` and wipes `ems` if non-nil — mirroring the existing `:119-122` loop, just hoisted into `defer`. The `defer` must be registered **after** `groupShares`/`ems` are in scope and positioned so it observes the final contents of the slice (a `defer func(){ for _, gs := range groupShares { wipe(gs.v) }; wipe(ems) }()` reads `groupShares`/`ems` at defer-execution time, capturing whatever was appended before the return — correct for all three paths). Remove the now-redundant explicit success-path scrub (or leave the `defer` as the single scrub site).
- In `recoverSecret`, on the digest-fail branch (`:138-141`) also `wipe(d)` alongside the existing `wipe(s)` before returning the sentinel. (Equivalently, hoist `wipe(d)` so it runs on both branches; either is acceptable — the implementer picks the lower-drift form. `d` is always non-nil on the threshold>1 path where this branch is reachable.)
- **Note (TinyGo):** `defer` with a closure compiles on the device target; the final gate is the TinyGo build (§0).

### 1.3 Invariant

After `Combine` returns by **any** path (success, `:103`, `:108`, `:116`), every `groupShares[i].v` backing array the function allocated is zeroed, and `ems` (when non-nil) is zeroed. After `recoverSecret` returns on the **digest-fail** path, both `s` and `d` are zeroed. No control-flow, return-value, or error-classification change.

### 1.4 Test (M2)

In-package tests in `slip39` (extend `vectors_test.go` / `combine_test.go`):
- **Path (c) — group-layer digest fail:** construct a multi-group set (`GroupThreshold ≥ 2`) where each group recovers at the member layer but the assembled group-layer digest fails (e.g. a member-threshold-1 group with a perturbed value surviving RS1024). The test must observe the `gv` backing arrays. **Observation strategy (must avoid a new production seam — see §6):** the cleanest in-package observation is to capture the slice headers the function controls. Because `groupShares` is a function-local, the test cannot read it directly without a seam; therefore the M2 test asserts via the **publicly observable proxy that the same `wipe` helper is invoked on the controlled buffers** — design the test around what is reachable in-package without editing production code:
  - **(c-primary) `recoverSecret` `d`-wipe:** drive `recoverSecret` directly (as `TestRecoverSecretWipesOnDigestFail` already does at `vectors_test.go:287`) with two `idx-12` shares whose digest fails; extend that test to also assert the interpolated `d` buffer is zeroed. Since `d` is a function-local not returned, capture it via the **same in-package technique the existing test uses** — and if `d` is not reachable without a seam, this is flagged at R0 (§6 Q1) rather than adding a seam. *Fallback that needs no seam:* assert the post-condition behaviorally — re-run with the success vector and confirm `wipe(d)` is exercised on success (`:142`) and add the digest-fail `d`-wipe as a **white-box** assertion only if reachable in-package.
  - **(a)/(b) `Combine` `gv`-leak:** assert that for a set hitting each of `:103` (later-group digest fail with a prior good group) and `:108` (`errInsufficientShares`), `Combine` returns the correct sentinel error AND that no secret residue is left in the buffers the function controls. As with (c), if observing the function-local `groupShares` requires a production seam, **flag at R0** (§6 Q1) — do not add a hook.
- **fail-before / pass-after:** each new assertion must FAIL on `3a23dbb` (the `defer`/`wipe(d)` absent) and PASS after the fix, to the extent the buffer is observable in-package.
- Follow the existing pattern: `TestWipeZeroes` (`vectors_test.go:302-310`) and `TestRecoverSecretWipesOnDigestFail` show the in-package, buffer-inspecting style this repo uses.

> **R0 dependency:** the M2 test's *observability of the function-local secret buffers without adding a production seam* is the one non-mechanical aspect of this finding. See §6 Q1 — R0 must rule on the observation strategy before the plan locks the test shape.

---

## 2. M3 — `seedxor.Combine`: wipe per-part `Entropy()` intermediates on every path

### 2.1 Verified facts (live `seedxor/seedxor.go` @ `3a23dbb`)

- `seedxor` has a package-local `wipe` (`:25-29`) — the helper to use.
- `Combine` (`:34-56`) already scrubs the accumulator `out` on every exit (`:40` bad-length, `:46` mismatched-lengths, `:54` success) and documents itself as a port of the "Zeroizing-everywhere" Rust reference.
- **The per-part entropy copies are never wiped (the bug):**
  - **`:38`** `out := append([]byte(nil), parts[0].Entropy()...)` — `parts[0].Entropy()` returns a **fresh** secret heap slice (`bip39.splitMnemonic` → `big.Int.Bytes()` then `append(padding, entBytes...)`, a new allocation per call — verified at `bip39/bip39.go:177-197`). Its bytes are copied into `out` and the intermediate is then abandoned, never wiped.
  - **`:44`** `e := p.Entropy()` inside `for _, p := range parts[1:]` — a **fresh** secret slice per part. Wiped on **no** path:
    - not after the XOR loop on success;
    - not on the `errMismatchedLengths` early return at **`:45-47`** (`wipe(out)` runs but the mismatched part's `e` is left live).
  - For an N-part combine, N un-scrubbed raw share-entropy copies are left on the heap.
- **Convention precedent (our own code already does the missing pattern):** `gui/singlesig_derive.go:85-87` (`entropy := m.Entropy(); … ; wipeBytes(entropy)`) and `gui/ms1_decode.go:29` (`defer wipeBytes(entropy)`). `Combine` is the outlier.

**Trigger (verified reachable):** Main menu → input flow → "SEED XOR" → `combineSeedXORFlow` → `seedxor.Combine`. Every successful N-part combine leaks N raw-entropy copies; the mismatched-lengths path leaks the offending part's copy.

### 2.2 The fix

- **`:38`:** bind the first part's entropy to a named var, copy into `out`, then `wipe` it: e.g. `e0 := parts[0].Entropy(); out := append([]byte(nil), e0...); wipe(e0)` — placed so `e0` is wiped before any return after it (the `interopLen` bad-length return at `:39-42` must also leave `e0` wiped; simplest is to `wipe(e0)` immediately after the copy, before the `interopLen` check).
- **`:44`:** `e := p.Entropy()` — wipe `e` on **every** path out of the loop body: after the XOR completes for that part, and alongside `wipe(out)` on the `errMismatchedLengths` early return at `:46`. A `wipe(e)` placed at the end of each iteration plus an explicit `wipe(e)` before the `errMismatchedLengths` return covers both; or `defer`-per-iteration is **discouraged** (defers in a loop accumulate until function return and would delay the wipe — prefer explicit in-loop `wipe(e)`). The implementer mirrors `singlesig_derive.go:85-87` (bind → use → `wipe`).
- No signature/return/error change; `out` scrubbing stays exactly as is.

### 2.3 Invariant

After `Combine` returns by **any** path (success, `errBadLength` at `:39-42`, `errMismatchedLengths` at `:45-47`), every per-part `Entropy()` intermediate the function allocated (`parts[0]`'s and each `parts[1:]`'s) is zeroed, in addition to the already-zeroed `out`. The returned `bip39.Mnemonic` and all error semantics are unchanged.

### 2.4 Test (M3)

In-package `seedxor` test (extend `seedxor/seedxor_test.go`; tests are `package seedxor`, can call `wipe` and inspect):
- **Per-part entropy zeroed on success:** because `Entropy()` returns a fresh allocation each call (proven by `TestCombineNoCallerMutation` at `:114`), the test cannot read the *internal* `e`/`e0` headers directly. **Observation strategy without a new seam:** assert the **post-condition the function controls** — i.e. that the caller's input mnemonics are NOT mutated (already covered) AND, for the controllable buffer, drive `Combine` and then re-derive each part's entropy and confirm `Combine` did not retain a live alias. Since the leaked `e`/`e0` are function-locals, the strongest seam-free assertion is the **mismatched-lengths path**, where the test can still only observe `out`; the per-part `e` is internal.
  - **R0 dependency (Q1, shared with M2):** observing a function-local intermediate without a production seam is the crux. The recommended seam-free design: a white-box in-package test that **re-implements the wipe expectation as a guard** — e.g. confirm the fix compiles and that `out` is zeroed on the `errMismatchedLengths` path (regression guard), and document that the per-part `e` zeroing is asserted at the **convention level** (the fix mirrors `singlesig_derive.go`). If a stronger direct assertion is wanted, it requires either a test-only build-tag hook or refactoring `Combine` to take an injectable wipe — **both are seams to flag at R0, not add unilaterally.**
- **Mismatched-lengths path (`:45-47`):** reuse `TestCombineMismatchedLengths` (`:143`, 12-word XOR 24-word → `errMismatchedLengths`) and add the regression assertion that `out` is zeroed on that path (and, if reachable in-package, that the offending `e` is zeroed).
- **fail-before/pass-after** where the buffer is observable.

---

## 3. M4 — BIP-85: `defer pkey.Zero()` so the leaf EC private-key scalar is scrubbed

### 3.1 Verified facts (live `gui/bip85.go` @ `3a23dbb`)

`deriveBip85Child` (`:60-120`):
- `:101` `pkey, err := k.ECPrivKey()` — `pkey` is a `*btcec.PrivateKey`.
- `:106` `priv := pkey.Serialize()` — 32-byte secret copy.
- `:107` `k.Zero()` then `:108` `defer wipeBytes(priv)`.
- The function scrubs `priv` (the serialized bytes) and `k` (the ExtendedKey), but **never scrubs `pkey`** — the live `*PrivateKey` object holding the raw leaf scalar survives the return.

**Authoritative type/method facts (verified against the module cache):** `github.com/decred/dcrd/dcrec/secp256k1/v4@v4.4.1/privkey.go`:
- `:16` `type PrivateKey struct { Key ModNScalar }`.
- `:97-98` `// against memory scraping.` / `func (p *PrivateKey) Zero()` — **pointer receiver**, documented precisely for memory-scraping defense. **Method name is `Zero` (no args).**
- `:107` `func (p PrivateKey) Serialize() []byte` — **value receiver**, so `Serialize()` copies the whole scalar; the copy is what `priv` reads while `pkey.Key` stays untouched. After return, the live `PrivateKey` retains the secret.
- `btcec/v2` aliases the decred type; `hdkeychain`'s `ECPrivKey()` materializes the scalar. The sibling `deriveAccountXpub` uses `ECPubKey()/Neuter()` and never materializes a `*PrivateKey`, so the gap is bip85-specific.

This contradicts the function's own SECURITY docstring (`:56-59`), which enumerates the scrubbed buffers and omits the privkey object.

### 3.2 The fix

- Immediately after `:106` `priv := pkey.Serialize()` (i.e. once `pkey` is materialized and the serialization captured), add **`defer pkey.Zero()`**. `pkey` is guaranteed non-nil at that point (the `err != nil` branch at `:102-105` already returned). Place it adjacent to the existing `defer wipeBytes(priv)` at `:108`. (Defer-order is immaterial here — both zero independent secret holders.)
- Optionally update the SECURITY docstring (`:56-59`) to list the privkey object among the scrubbed buffers — **non-load-bearing comment hygiene; the implementer MAY include it, but it is not required for the invariant.**
- **TinyGo note:** `defer pkey.Zero()` is a plain deferred method call — compiles on the device target. The pointer-receiver `Zero()` mutates `pkey.Key` in place; the value-receiver `Serialize()` copy is already covered by `wipeBytes(priv)`. The TinyGo build is the final gate.

### 3.3 Invariant

After `deriveBip85Child` returns by any path **reached after `pkey` is materialized** (success at `:118-119`, the `entLen` guard at `:114-116`), the `pkey.Key` scalar is zeroed via `pkey.Zero()`, in addition to the already-scrubbed `priv`, `seed`, `hmacOut`, and each intermediate `k`. The returned child mnemonic and all error semantics are unchanged. (On the early `:102-105` error path `pkey` is nil/unused, so no scrub is needed — the `defer` is registered only after the non-nil materialization.)

### 3.4 Test (M4)

`gui` test (extend `gui/bip85_test.go`):
- **Direct unit test of `deriveBip85Child` scrub:** the cleanest seam-free assertion is to hold a reference to the `*PrivateKey` and check `.Key.IsZero()` after the function returns — but `pkey` is a function-local, not exposed. **Observation strategy without a new production seam:** the existing tests (`TestDeriveBip85Child_CanonicalVector` `:54`, `TestBip85DeriveFlow_ScrubsBothMnemonics` `:211` via `bip85SeedHook`) show the repo's scrub-observation style is a **test-only hook seam already present in production-for-test** (`bip85SeedHook`). M4 should follow that established pattern's *spirit*: assert what the function controls. Since `pkey.Zero()` zeroes a function-local object, the directly-assertable post-condition is **the `priv` serialization the existing `defer wipeBytes(priv)` already covers** — so the *new* `pkey.Zero()` is best asserted via a focused unit test that:
  - confirms `deriveBip85Child` still returns the correct canonical child (no behavior regression — reuse `TestDeriveBip85Child_CanonicalVector`'s vector), and
  - asserts the scrub at the level the code controls. **If asserting `pkey.Key.IsZero()` requires exposing `pkey` (a new seam), flag at R0 (§6 Q1)** rather than adding one; the convention precedent (`bip85SeedHook` is itself a sanctioned test-only seam in this very file) is the relevant prior art the R0 reviewer should weigh.
- **fail-before/pass-after:** if a seam-free or sanctioned-seam assertion of `pkey` zeroing is achievable, it must FAIL on `3a23dbb` (no `pkey.Zero()`) and PASS after. Otherwise the test reduces to a no-behavior-regression guard + the §6 Q1 flag.

---

## 4. L1 (codex32_polish site ONLY) — capture and `wipeBytes()` the `DecodeMS1` probe entropy

### 4.1 Verified facts (live `gui/codex32_polish.go` @ `3a23dbb`)

- `:103` `_, _, _, msErr := codex32.DecodeMS1(scan)` — `DecodeMS1` is used purely as a validity probe; the returned **entropy is discarded with `_`, unscrubbed**.
- `codex32.DecodeMS1` signature (verified `codex32/mspayload.go:34`): `func DecodeMS1(s String) (prefix, language int, entropy []byte, err error)`. The third return is the secret BIP-39 entropy. The probe allocates a fresh `[]byte` ([prefix][full seed entropy], 16–32 bytes) on every call (per the report: `data()` does `make+append`, no caching) and leaves it for the GC.
- **Convention precedent:** `gui/ms1_decode.go:22-29` captures the entropy and `defer wipeBytes(entropy)`; `bundle/verify.go:ms1Entropy` copies-then-`wipe`. The codex32_polish probe is the outlier.
- **This is the disjoint site** — the other two L1 sites (`singlesig_verify.go:116`, `multisig_verify.go:93`) are **Track A's**; Track B must NOT touch them.
- **Severity context (from the report, do not re-litigate):** downgraded to LOW because at this site the same secret is already resident, longer-lived, in the immutable `scan codex32.String` (engraved verbatim) — zeroing the short-lived probe buffer does not shrink the dominant exposure window. The fix is a low-cost **consistency** fix.

### 4.2 The fix

- At `:103`, capture the entropy return into a named var and `wipeBytes` it after the probe's only use (the `msErr == nil` test at `:104`): e.g.
  ```
  _, _, ent, msErr := codex32.DecodeMS1(scan)
  wipeBytes(ent) // ent is nil on the err path; wipeBytes(nil) is a no-op
  showSecret := f.Unshared && msErr == nil
  ```
  `wipeBytes(nil)` ranges an empty slice — a safe no-op — so it is correct on the error path too. Mirrors `ms1_decode.go:29`. (A `defer` is unnecessary here since the probe value is used exactly once on the same line region; an immediate `wipeBytes` after the last read is the lower-footprint choice. The implementer MAY use `defer wipeBytes(ent)` to match `ms1_decode.go` style — either satisfies the invariant.)
- **Uses the existing `wipeBytes` (`gui/slip39_polish.go:344`) — NOT edited.**

### 4.3 Invariant

After the probe at `:103`, the entropy slice returned by `DecodeMS1` is zeroed before `confirmCodex32Flow` proceeds (and on both the decode-OK and decode-err sub-cases — `nil` on error is a no-op). The `showSecret` decision, all subsequent flow logic, and the engraved-verbatim `scan` string are unchanged.

### 4.4 Test (L1)

`gui` test (new focused test or extend an existing codex32_polish test):
- Assert the probe entropy buffer is zeroed after use. Because the probe entropy is a function-local, the seam-free assertion follows the same constraint as M2/M3/M4 (§6 Q1). The recommended low-cost form: a unit test that drives `confirmCodex32Flow` (or directly calls `codex32.DecodeMS1` to confirm the entropy return is non-nil for an ms1 secret, establishing the buffer exists to scrub) and asserts the **convention is followed** (the fix matches `ms1_decode.go:29`). A behavioral regression guard: confirm `showSecret` is still computed correctly (unshared ms1 → true; non-ms1 unshared → false) so the additive `wipeBytes` did not perturb the probe semantics.
- fail-before/pass-after only insofar as the buffer is observable in-package without a new seam.

---

## 5. Cross-cutting: test-design note on best-effort scrubs and observability

Per the bug-hunt's severity calibration and the orchestration plan, all four scrubs are **best-effort under TinyGo's GC**, which may copy/retain. Therefore:
- Tests assert the buffer **the code itself controls** (a named local slice the test can reach in-package, or the `*PrivateKey` object), NOT GC-copied ghost copies — asserting on copies the GC may have made is both impossible and not the contract.
- The repo's established in-package scrub-test idioms are the templates: `slip39/vectors_test.go:302-310` (`TestWipeZeroes`), `slip39/vectors_test.go:282-298` (`TestRecoverSecretWipesOnDigestFail`), `seedxor/seedxor_test.go:114` (`TestCombineNoCallerMutation` — proves `Entropy()` returns fresh copies), and the sanctioned **test-only hook seam** `bip85SeedHook` (`gui/bip85.go:241`, used at `gui/bip85_test.go:211`).
- **The single recurring design question across all four (Q1):** three of the four leaked buffers are **function-locals not exposed by any production symbol**. Asserting their zeroing directly may require either (a) reaching them in-package without a seam (possible for `recoverSecret`'s direct-call shape; harder for `Combine`/`seedxor.Combine`/`deriveBip85Child` locals), or (b) adding a test-only seam. Per the hard constraints, **adding a new production helper or seam is a design smell to flag at R0, not implement.** The R0 architect must rule, per finding, whether the seam-free assertion is strong enough or whether a *sanctioned, precedented* test-only hook (à la `bip85SeedHook`) is warranted — and if so, that becomes an explicit, reviewed plan item, not an implementer's unilateral addition.

---

## 6. Confirmations required by R0 (per the orchestration plan)

- **No public output / signature / return-value / control-flow change.** Every fix is an additive `wipe`/`Zero`/`defer` on an already-dead or already-discarded secret buffer; the existing return decisions, error classifications, and outputs are untouched. (M2: `defer` fires after the existing returns; M3: in-loop/explicit `wipe` on dead intermediates; M4: `defer pkey.Zero()` on a function-local; L1: `wipeBytes` on a `_`-discarded value.)
- **No shared-helper edit.** `wipeBytes` (`gui/slip39_polish.go:344`) is only **called** (by M4's byte-slice scrub is already present; by L1). `slip39.wipe` and `seedxor.wipe` are package-local and only called. **No fix edits any helper.**
- **No new helper.** No fix introduces a new scrub helper. Any urge to do so is flagged here (Q1) for R0, not implemented.
- **Firmware-only; TinyGo device build is the real gate.** `defer`/closures/`defer`-method-call all compile on the device target, but the final integration gate is the **TinyGo device build CI**, not host `go build`/`go test` (which is clean at `3a23dbb`).
- **Four files pairwise-disjoint** (`slip39/combine.go`, `seedxor/seedxor.go`, `gui/bip85.go`, `gui/codex32_polish.go`) → in-track **sequential single-implementer TDD in one worktree** (`fix/scrub-batch`), conflict-free; the two `gui`-package fixes (M4, L1) touch different files and different functions.
- **Track-A L1 sites untouched** (`singlesig_verify.go`, `multisig_verify.go`) — Track B owns only `codex32_polish.go:103`.

### Open questions for R0

- **Q1 (the only material one) — test observability vs. the no-new-seam constraint.** Three of four leaked secrets are function-locals (`Combine`'s `groupShares[].v`/`ems`, `seedxor.Combine`'s per-part `e`/`e0`, `deriveBip85Child`'s `pkey`). A direct "buffer is zeroed" assertion may need either an in-package white-box reach (feasible for `recoverSecret`'s `d` via the existing direct-call test shape; less so for the others' deep locals) or a **sanctioned test-only hook** (the repo already does this with `bip85SeedHook`). **R0 must rule, per finding:** is the seam-free assertion (behavioral regression guard + convention-match) sufficient for these LOW/MEDIUM best-effort scrubs, OR is a precedented test-only hook (reviewed, not unilateral) warranted to get a true fail-before/pass-after assertion? This determines the test shape the plan locks.
- **Q2 (minor) — M2 `recoverSecret` `wipe(d)` placement.** Hoist `wipe(d)` to run on both branches vs. add it only to the digest-fail branch. Both satisfy the invariant; R0 picks the lower-drift form (recommendation: add to the digest-fail branch only, leaving the success-path `:142` as-is, for minimal diff).
- **Q3 (minor) — M3 first-part scrub placement.** Whether `wipe(e0)` for `parts[0].Entropy()` goes immediately after the copy (before the `interopLen` check, so the bad-length return also leaves it wiped) — recommended — vs. later. R0 confirms the bad-length path (`:39-42`) must also scrub `e0`.

---

## 7. Next gate (mandatory)

This spec must pass an **opus architect R0 review to 0 Critical / 0 Important** before any implementation plan is authored. Fold findings → persist the review verbatim to `design/agent-reports/` → re-dispatch after every fold until GREEN. No code before GREEN. (Track B runs this gate independently of, and concurrently with, Track A per the orchestration plan.)
