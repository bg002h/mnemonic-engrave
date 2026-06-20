# Track B (scrub batch) — whole-diff execution review (round 0) — VERBATIM agent report

**Agent:** `ab882bad40f62a096` (adversarial opus architect; mandatory post-implementation gate; ran the M4 negative control + distinct-alloc/deferred-append probes in throwaway worktrees off the branch tip). **Branch:** `feat/fix-scrub-batch`. **Base:** `3a23dbb`. **Tip:** `a74c1bc`. **Date:** 2026-06-20.
**Verdict:** GREEN (0 Critical / 0 Important). MERGE recommended. 1 informational Minor.

---

# R0 EXEC REVIEW — Track B (scrub batch: M2/M3/M4/L1) — whole-diff post-implementation gate

**Reviewer:** Adversarial opus architect (post-implementation exec gate)
**Branch:** `feat/fix-scrub-batch` · **Base:** `3a23dbb` (fork `main`) · **Tip:** `a74c1bc`
**Commits:** M2 `ea8bf68`, M3 `dda0832`, M4 `8b8a390`, L1 `a74c1bc` (linear, single-parent chain back to base; no merge)
**VERDICT: GREEN — 0 Critical / 0 Important. MERGE.**

---

## Mandate 1 — Diff fidelity / scope

The diff touches exactly 8 files and nothing else:
```
gui/bip85.go            gui/bip85_test.go
gui/codex32_polish.go   gui/codex32_polish_test.go
seedxor/seedxor.go      seedxor/seedxor_test.go
slip39/combine.go       slip39/combine_test.go
```
- Forbidden-file scan EMPTY: no `gui/slip39_polish.go` (the `wipeBytes` helper), no `slip39/feistel.go` (`slip39.wipe`), no `seedxor.wipe` edit. `git diff -- gui/slip39_polish.go slip39/feistel.go` is empty → helpers unchanged.
- NO Track-A file touched (`bundle/verify.go`, `gui/multisig_verify.go`, `gui/multisig_supply.go`, `gui/singlesig_verify.go`, `gui/md1_gather.go`).
- The ONLY new production symbol is `+var bip85PkeyHook func(pkey *btcec.PrivateKey)` — test-only, declared nil, guarded `if bip85PkeyHook != nil`, assigned only inside the test (reset to nil via `defer`). A new btcec import was added to both `gui/bip85.go` and the test (required by the hook signature). The other `+var ems []byte` is a function-local hoist, not a package symbol. No new `func`/`type` in production.
- No scope creep, no unplanned hunk, no non-additive change.

## Mandate 2 — Each fix correct + complete on every leak path

- **M2** (`slip39/combine.go`): `var ems []byte` hoisted (`:81`); the `defer` closure ranging `groupShares`+`wipe(ems)` registered right after at `:87-92`, so it fires on the success return (`:131`) AND all three error returns (`:115`/`:120`/`:128`). The old success-path scrub loop is correctly removed (defer is the single site). `ems, err := recoverSecret(...)` stays `:=` at `:126` → compiles (Minor-1). `wipe(d)` added to the digest-fail branch only (`:148`); success-path `wipe(d)` at `:151` left intact (Q2).
- **M3** (`seedxor/seedxor.go`): `e0 := parts[0].Entropy(); out := append(...); wipe(e0)` before the `interopLen` check (`:38-40`, Q3) — bad-length return safe. In-loop `wipe(e)` on the mismatched-lengths return (`:49`) AND after the XOR on success (`:55`); explicit, not defer-in-loop. Empirically proven `out` is a distinct backing array from `e0` (probe: distinct pointers, `out` uncorrupted after wiping `e0`) → the early `wipe(e0)` cannot corrupt the result.
- **M4** (`gui/bip85.go`): `defer pkey.Zero()` at `:110`, immediately after `priv := pkey.Serialize()`, where `pkey` is guaranteed non-nil (the `err != nil` branch at `:103-105` already returned). Covers the success return and the `entLen` guard.
- **L1** (`gui/codex32_polish.go`): `_, _, ent, msErr := codex32.DecodeMS1(scan); wipeBytes(ent)` — wipes the entropy subslice only (Minor-3), nil-safe on the err path.

None changes a public output, signature, return value, or control flow. (Adversarial probe confirmed the M2 deferred closure observes `groupShares` entries appended after the defer was registered — so it genuinely scrubs leaked `gv`, not zero entries.)

## Mandate 3 — Negative controls (load-bearing proof)

**M4 negative control — OBSERVED:** in a throwaway worktree, stripping ONLY the live `defer pkey.Zero()` statement (line 110; the two remaining grep matches were the doc-comment and the test docstring, not statements):
```
=== RUN   TestDeriveBip85Child_ScrubsPkey
    bip85_test.go:526: pkey.Key not zeroed after deriveBip85Child returned (M4: missing defer pkey.Zero())
--- FAIL: TestDeriveBip85Child_ScrubsPkey (0.02s)
```
Restored → `--- PASS`. Genuine fail-before/pass-after.

- **M2 sentinels** assert the right sentinels: path(a) `pa[0].Value[0]^=0xff` → `errDigestVerificationFailed` (member-layer, `:115`); path(b) only `parsed[1]` → `errInsufficientShares` (`:120`); path(c) `pc[1].Value[0]^=0xff` → `errDigestVerificationFailed` (group-layer, `:128`). `TestCombineErrorPathSentinels` PASS.
- **M3 no-caller-mutation** guard `TestCombineNoCallerMutation` PASS — proves wiping `e0`/`e` cannot corrupt the caller's mnemonics.
- **L1 showSecret gate** unchanged: `TestConfirmCodex32Flow_ShowSecretGate` and the sibling `TestConfirmShowSecretGate` both PASS.
- M2/M3/L1 guards are honestly framed in their docstrings as regression+convention guards (green-before-and-after by design), NOT buffer-zeroed assertions — matches the spec-R0 Q1/Minor-2 ruling.

## Mandate 4 — Build / test / vet

- `go build ./...` — clean (no output).
- `go test ./...` — every package `ok`, 0 FAIL (38 packages; `slip39`/`seedxor`/`gui` and the four new tests all green).
- `go vet ./slip39/ ./seedxor/ ./gui/` — clean (0 findings) at the tip. Cold-vs-cold: the same vet at clean `3a23dbb` is also clean → ZERO new vet findings. I scoped to the touched packages, so the pre-existing `bspline`/`backup`/`engrave`/`gui/op` go1.25/1.26 notes (which `go vet ./...` surfaces and which the plan-R0 flagged) never enter the comparison — not fooled by them.

## Mandate 5 — Secret-hygiene + TinyGo

The scrubs zero exactly the targeted secret buffers (group-share `gv`/`ems`/`d`; per-part `e0`/`e`; the leaf scalar `pkey.Key`; the probe entropy subslice) and leave nothing MORE exposed — all additive `wipe`/`Zero`/`wipeBytes` on already-dead or discarded buffers. TinyGo-safe by inspection (`defer`/closures, `defer pkey.Zero()` method call, explicit in-loop `wipe`); device build correctly deferred to the controller's integration gate.

## Mandate 6 — Commit hygiene

Each of the 4 commits: raw `gpgsig -----BEGIN SSH SIGNATURE-----` block present; `Signed-off-by: Brian Goss <goss.brian@gmail.com>` (DCO); `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`; author Brian Goss. (`%G?` returns `N` only because `gpg.ssh.allowedSignersFile` is unconfigured for verification — the signature blocks are physically present, as the mandate requires.) Each commit is self-contained: builds clean and passes `slip39`/`seedxor`/`gui` at that commit. Explicit-path staging per plan; linear chain, no merge.

---

## Critical findings
None.

## Important findings
None.

## Minor findings
1. (Informational, not actionable) `%G?` is `N` for all four commits because the repo lacks `gpg.ssh.allowedSignersFile`; the raw SSH-signature blocks are present and config-correct. Same caveat the plan-R0 noted; not a defect.

## Verified-correct list
- Exactly 8 files; only new symbol `bip85PkeyHook`; helpers + all Track-A files untouched.
- M2 defer fires on success + all 3 error returns; `var ems` hoisted; `:=` preserved (compiles); redundant success scrub removed; `wipe(d)` digest-fail-only; deferred-closure-observes-appends semantic probe-confirmed.
- M3 `e0` wiped before `interopLen` (distinct-alloc safety probe-confirmed); in-loop `wipe(e)` on success + mismatch; no caller mutation.
- M4 genuine fail→pass negative control (FAIL `pkey.Key not zeroed…` → PASS); canonical child golden unchanged.
- L1 `wipeBytes(ent)` subslice-scoped, nil-safe; showSecret gate unchanged.
- build clean; `go test ./...` 0 FAIL; touched-package vet clean cold-vs-cold (zero new findings).
- 4 self-contained, signed, DCO+Co-Authored commits, author Brian Goss, no merge.

## Bottom line
The committed diff faithfully implements the GREEN plan and spec and folds all three spec-R0 Minors (Minor-1 `ems`/`:=`, Minor-2 guard posture per task, Minor-3 L1 subslice scope) plus Q1/Q2/Q3. Every fix is strictly additive and complete on each leak path; M4 is a true fail-before/pass-after via the sanctioned in-file-precedented hook; M2/M3/L1 are correctly-scoped, honestly-framed regression+convention guards. Build/test/vet are green with zero new vet findings.

**RECOMMENDATION: MERGE.**

**Fork state on exit:** clean on `main @ 3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`; branch `feat/fix-scrub-batch` intact at `a74c1bc`; throwaway worktrees (`/tmp/exec-B`, `/tmp/exec-B-base`, `/tmp/exec-B-neg`) removed and pruned; the implementer's `/tmp/scrub-batch-wt`, Track-A's `/tmp/seedhammer-verify-cluster`, and `seedhammer-wt-bip39` left untouched.
