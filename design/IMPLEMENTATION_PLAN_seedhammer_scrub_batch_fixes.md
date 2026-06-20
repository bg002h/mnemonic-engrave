# Track B — secret-scrub batch fixes (M2, M3, M4, L1-codex32_polish) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure-additive zeroing of already-discarded secret-bearing buffers at four mechanical, pairwise-disjoint sites in the SeedHammer fork (`slip39/combine.go`, `seedxor/seedxor.go`, `gui/bip85.go`, `gui/codex32_polish.go`), each calling the package's existing `wipe`/`wipeBytes` helper — no public output, signature, return-value, or control-flow change.

**Architecture:** Four sequential fixes in ONE worktree on branch `feat/fix-scrub-batch` off `3a23dbb`. Each fix is test-first (TDD): write the test → run it → apply the minimal `wipe`/`Zero`/`defer` → run it → commit. **One fix (M4) is a genuine fail-before/pass-after assertion** via a precedented, in-file-sanctioned test-only hook (`bip85PkeyHook`, mirroring the existing `bip85SeedHook`); the other three buffers are function-locals unobservable seam-free, so their load-bearing tests are **seam-free regression + convention guards** (correct sentinel/behaviour + helper-present), NOT buffer-zeroed assertions — this is explicit per the spec R0 ruling (Q1, Minor-2), and the exec reviewer must expect exactly that.

**Tech Stack:** Go 1.26.4 (host: `export PATH=$PATH:/home/bcg/.local/go/bin`); the fork's package-local `slip39.wipe`/`seedxor.wipe` and `gui.wipeBytes` helpers; `github.com/btcsuite/btcd/btcec/v2` (`(*PrivateKey).Zero()` / `.Key.IsZero()`). **The final integration gate is the TinyGo device build (the controller's integration pass), NOT host `go build`** — every fix is TinyGo-safe (`defer`/closures/`defer`-method-call all compile on the device target), but this plan verifies only host build/vet/test.

---

## Pre-flight facts (verified live at `3a23dbb`, do not re-derive)

- **Fork HEAD:** `3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`, branch `main`, clean working tree. Path `/scratch/code/shibboleth/seedhammer`.
- **Do NOT touch the `seedhammer-wt-bip39` worktree** (`feat/bip39-entry-polish` @ `0e610dd`) — it is a sibling cycle's workspace.
- **Go:** `go version go1.26.4`; baseline `go test ./slip39/... ./seedxor/... ./gui/...` is GREEN at `3a23dbb`.
- **Commit signing:** the repo is configured for SSH-format signing (`gpg.format=ssh`, `user.signingkey=/home/bcg/.ssh/id_ed25519.pub`). All commits use `-S -s`, author `Brian Goss <goss.brian@gmail.com>`, and a `Co-Authored-By` trailer.
- **`Share.Value` is a public, mutable `[]byte`** (`slip39/share.go:23`) — M2 error-path tests perturb a *copy* of it to force digest failures.
- **`slip39` vector idx 17** (`testdata/slip39_vectors.json`, key `"17"`) has `GroupThreshold=2, GroupCount=4` and exactly 3 shares: group 1 = one share `MemberThreshold=1` (a threshold-1 group, carries its share directly, NO member digest); group 3 = two shares `MemberThreshold=2`. This is the structure that makes all three M2 error paths constructible (probe-confirmed below).
- **Helpers (CALL only, never edit):** `slip39.wipe` (`slip39/feistel.go:17`), `seedxor.wipe` (`seedxor/seedxor.go:25`), `gui.wipeBytes` (`gui/slip39_polish.go:344`). `wipeBytes(nil)` is a safe no-op.
- **M4 type facts:** `k.ECPrivKey()` returns `(*btcec.PrivateKey, error)`; `func (p *PrivateKey) Zero()` is a pointer receiver that scrubs `p.Key` (documented "against memory scraping"); `func (p PrivateKey) Serialize() []byte` is a value receiver (copies the scalar — already covered by `defer wipeBytes(priv)`). `pkey.Key.IsZero()` returns `false` before `Zero()`, `true` after (probe-confirmed). The btcec import path in this repo is `github.com/btcsuite/btcd/btcec/v2`.

## R0 rulings folded into this plan (from `design/agent-reports/seedhammer-scrub-batch-spec-R0-round0.md`)

- **Q1 (test seam):** No NEW production seam is required for any finding. M2-`gv`/`d`, M3-`e`/`e0`, L1-`ent` are unobservable seam-free → seam-free **regression + convention guards** (NOT buffer-zeroed assertions). M4 MAY use a precedented test-only hook — and this plan DOES (the lone genuine fail-before/pass-after).
- **Q2 (M2 `wipe(d)` placement):** Add `wipe(d)` to the digest-fail branch ONLY; leave the success-path `:142 wipe(d)` as-is (lower diff).
- **Q3 (M3 `e0` placement):** Wipe `e0` immediately after the copy, before the `interopLen` check, so the bad-length return also leaves `e0` wiped (`out` is a distinct allocation — probe-confirmed safe).
- **Minor-1 (M2 mechanics):** `ems` MUST be hoisted to a function-scope `var ems []byte`; the assignment at `combine.go:114` MUST stay `:=` (`ems` reuses the pre-declared var, `err` is the new name in that scope) — changing to `=` breaks compilation.
- **Minor-2 (test posture):** State plainly that M2/M3/L1 tests are regression+convention guards, not buffer-zeroed assertions.
- **Minor-3 (L1 scope):** Wipe ONLY the entropy subslice `DecodeMS1` returns (`data[1:]`/`data[2:]`); do NOT attempt to wipe the whole `data` buffer (the function does not hold a handle to it).

## Hard constraints (carried from spec §6 + orchestration plan)

- **Sequential single implementer in ONE worktree.** Do NOT spawn parallel re-implementations. The four files are pairwise-disjoint, so the fixes compose cleanly in sequence.
- **Do NOT edit any helper** (`wipeBytes`, `slip39.wipe`, `seedxor.wipe`) — only CALL them. The only new symbol is M4's `bip85PkeyHook` (test-only, nil in production, sanctioned by the in-file `bip85SeedHook` precedent).
- **Do NOT touch the Track-A L1 sites** `gui/singlesig_verify.go:116` or `gui/multisig_verify.go:93`. Track B owns ONLY `gui/codex32_polish.go:103`.
- **Pure additive scrubs** — no public output/signature/return/control-flow change (except the M4 test hook, which is test-only and a no-op in production).
- **Leave the fork clean on `main @ 3a23dbb`** at the end; remove the probe worktree; do NOT merge. Pushing/merging is a later, separately-gated step.

---

## Task 0: Worktree setup + baseline green

**Files:**
- No source changes. Creates a worktree at a `/tmp` path on a new branch.

- [ ] **Step 1: Confirm the fork is clean at `3a23dbb`**

```bash
export PATH=$PATH:/home/bcg/.local/go/bin
cd /scratch/code/shibboleth/seedhammer
git rev-parse HEAD          # expect 3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082
git status --short          # expect EMPTY
git worktree list           # expect main @ 3a23dbb + seedhammer-wt-bip39 (DO NOT TOUCH)
```

Expected: HEAD is `3a23dbb…`, working tree empty, the `seedhammer-wt-bip39` worktree present.

- [ ] **Step 2: Create the implementation worktree on a new branch off `3a23dbb`**

```bash
cd /scratch/code/shibboleth/seedhammer
git worktree add -b feat/fix-scrub-batch /tmp/scrub-batch-wt 3a23dbb
cd /tmp/scrub-batch-wt
git rev-parse HEAD          # expect 3a23dbb…
git branch --show-current   # expect feat/fix-scrub-batch
```

Expected: worktree at `/tmp/scrub-batch-wt`, branch `feat/fix-scrub-batch`, HEAD `3a23dbb`.

- [ ] **Step 3: Configure the commit identity for this worktree**

```bash
cd /tmp/scrub-batch-wt
git config user.name "Brian Goss"
git config user.email "goss.brian@gmail.com"
```

Expected: no output (config set locally for this worktree).

- [ ] **Step 4: Baseline test run — confirm GREEN before any change**

Run:
```bash
cd /tmp/scrub-batch-wt
export PATH=$PATH:/home/bcg/.local/go/bin
go test ./slip39/... ./seedxor/... ./gui/...
```

Expected: PASS — `ok seedhammer.com/slip39`, `ok seedhammer.com/seedxor`, `ok seedhammer.com/gui`, and the `gui/op`,`gui/saver`,`gui/text`,`gui/widget` subpackages all `ok`. No `FAIL`.

(No commit for Task 0 — it only sets up the workspace.)

---

## Task 1: M2 — `slip39/combine.go` group-share scrub on all paths + `recoverSecret` digest-fail `d`-wipe

**Files:**
- Modify: `slip39/combine.go` (`Combine` `:80-124`, `recoverSecret` `:138-141`)
- Test: `slip39/combine_test.go` (new regression-guard test, in-package `package slip39`)

**What this fixes:** `Combine` scrubs `groupShares[].v` and `ems` only on the success path (`:119-122`); three error returns (`:103`, `:108`, `:116`) skip the scrub, leaking recovered group-share secrets. Separately, `recoverSecret`'s digest-fail branch (`:138-141`) wipes `s` but not `d` (the interpolated digest‖random buffer). **Test posture (Q1/Minor-2):** the leaked `gv`/`ems`/`d` are function-locals, unobservable seam-free → the load-bearing test is a **regression+convention guard**: assert the correct sentinel error on each of the three error paths (proves the `defer` did not perturb control flow / error classification), plus the existing `TestWipeZeroes` already pins that `wipe` zeroes. This is NOT a buffer-zeroed assertion.

- [ ] **Step 1: Write the failing/guard test for the three M2 error paths**

Append to `slip39/combine_test.go` (the file is `package slip39`, already imports `encoding/hex` and `testing`; add `errors`):

First, update the import block at the top of `slip39/combine_test.go` from:

```go
import (
	"encoding/hex"
	"testing"
)
```

to:

```go
import (
	"encoding/hex"
	"errors"
	"testing"
)
```

Then append this test to the end of `slip39/combine_test.go`:

```go
// copyShare / copyShares deep-copy a share (including its mutable Value backing
// array) so a test can perturb a value byte without disturbing the source.
func copyShare(s Share) Share { s.Value = append([]byte(nil), s.Value...); return s }
func copyShares(ss []Share) []Share {
	out := make([]Share, len(ss))
	for i, s := range ss {
		out[i] = copyShare(s)
	}
	return out
}

// TestCombineErrorPathSentinels is the M2 regression+convention guard. It does
// NOT assert the leaked group-share buffers are zeroed (they are function-local
// to Combine and unobservable seam-free — spec R0 Q1/Minor-2). Instead it proves
// the additive scrub `defer` did NOT change control flow or error classification:
// each of the three error returns that previously skipped the scrub
// (combine.go:103 / :108 / :116) still returns its correct sentinel. The
// success-path equivalence is covered by the existing official-vector tests; the
// helper-zeroes-its-buffer invariant by TestWipeZeroes.
//
// Vector idx 17 has GroupThreshold=2, GroupCount=4, with group 1 = one
// MemberThreshold=1 share (parsed[1], carries its share directly, no member
// digest) and group 3 = two MemberThreshold=2 shares (parsed[0], parsed[2]).
func TestCombineErrorPathSentinels(t *testing.T) {
	parsed := parseAll(t, vectorShares(t, 17))

	// Sanity: the clean set recovers (so the perturbations below are the only
	// reason any path errors).
	if _, err := Combine(copyShares(parsed), []byte("TREZOR")); err != nil {
		t.Fatalf("clean idx17 Combine: %v", err)
	}

	// Path (a) combine.go:103 — a member of the 2-member group (group 3) is
	// perturbed so its member-layer digest fails AFTER group 1 has already
	// recovered and appended its gv (groups iterate sorted: 1 then 3).
	pa := copyShares(parsed)
	pa[0].Value[0] ^= 0xff
	if _, err := Combine(pa, []byte("TREZOR")); !errors.Is(err, errDigestVerificationFailed) {
		t.Fatalf("path(a): err = %v, want errDigestVerificationFailed", err)
	}

	// Path (b) combine.go:108 — supply only group 1's single share, so the
	// recovered-group count (1) != GroupThreshold (2).
	pb := []Share{copyShare(parsed[1])}
	if _, err := Combine(pb, []byte("TREZOR")); !errors.Is(err, errInsufficientShares) {
		t.Fatalf("path(b): err = %v, want errInsufficientShares", err)
	}

	// Path (c) combine.go:116 — perturb group 1's threshold-1 share so it
	// "recovers" a corrupt gv (no member digest to catch it), group 3 recovers
	// cleanly, then the GROUP-layer recoverSecret digest fails.
	pc := copyShares(parsed)
	pc[1].Value[0] ^= 0xff
	if _, err := Combine(pc, []byte("TREZOR")); !errors.Is(err, errDigestVerificationFailed) {
		t.Fatalf("path(c): err = %v, want errDigestVerificationFailed", err)
	}
}
```

- [ ] **Step 2: Run the test to confirm it passes on `3a23dbb` (regression guard — green BEFORE the fix)**

Run:
```bash
cd /tmp/scrub-batch-wt
export PATH=$PATH:/home/bcg/.local/go/bin
go test ./slip39/ -run TestCombineErrorPathSentinels -v
```

Expected: PASS. **This is a regression guard, not a fail-before test** — the sentinel errors are already correct at `3a23dbb`; the test exists to prove the M2 `defer` (Step 3) does not perturb them. (Probe-confirmed at `3a23dbb`: path(a)/(c) → `slip39: bad share set`, path(b) → `slip39: not enough shares`.)

- [ ] **Step 3: Apply the M2 fix (Old → New) in `slip39/combine.go`**

**Edit 3a — hoist the scrub into a `defer` registered right after `groupShares` is declared, and hoist `ems` to a function-scope var.**

Old (`combine.go:80-81`):
```go
	groupShares := make([]gshare, 0, len(gids))
	for _, g := range gids {
```

New:
```go
	groupShares := make([]gshare, 0, len(gids))
	var ems []byte
	// M2: scrub recovered group-share secrets + ems on EVERY return path
	// (success + all three error returns at :103/:108/:116). The closure reads
	// groupShares/ems at defer-execution time, so it observes whatever was
	// appended before the return; ems is nil on paths (a)/(b) (wipe(nil) is a
	// no-op) and holds the EMS only after the group-layer recoverSecret succeeds.
	defer func() {
		for _, gs := range groupShares {
			wipe(gs.v)
		}
		wipe(ems)
	}()
	for _, g := range gids {
```

**Edit 3b — keep the `ems` assignment as `:=` (Minor-1: `ems` reuses the pre-declared var, `err` is new in that scope — do NOT change to `=`).**

The line at `combine.go:114` is already correct and MUST stay verbatim:
```go
	ems, err := recoverSecret(first.GroupThreshold, gpts)
```
No edit to this line. (Documented here because a naive implementer might "tidy" it to `=` after seeing `var ems []byte` above — that breaks compilation, since `err` is only introduced by this `:=`.)

**Edit 3c — remove the now-redundant success-path scrub loop + `wipe(ems)` (the `defer` is now the single scrub site).**

Old (`combine.go:118-123`):
```go
	master := feistelDecrypt(ems, passphrase, first.IterationExp, first.Identifier, first.Extendable)
	for _, gs := range groupShares {
		wipe(gs.v)
	}
	wipe(ems)
	return master, nil
```

New:
```go
	master := feistelDecrypt(ems, passphrase, first.IterationExp, first.Identifier, first.Extendable)
	return master, nil
```

**Edit 3d — add `wipe(d)` to the `recoverSecret` digest-fail branch ONLY (Q2).**

Old (`combine.go:138-141`):
```go
	if subtle.ConstantTimeCompare(digest, sum[:digestLen]) != 1 {
		wipe(s)
		return nil, errDigestVerificationFailed
	}
```

New:
```go
	if subtle.ConstantTimeCompare(digest, sum[:digestLen]) != 1 {
		wipe(s)
		wipe(d) // M2: also scrub the interpolated digest‖random buffer on this path
		return nil, errDigestVerificationFailed
	}
```

(Leave the success-path `wipe(d)` at `combine.go:142` exactly as-is.)

- [ ] **Step 4: Run the M2 test + the existing slip39 suite to confirm PASS, no regression**

Run:
```bash
cd /tmp/scrub-batch-wt
export PATH=$PATH:/home/bcg/.local/go/bin
go vet ./slip39/
go test ./slip39/ -v -run 'TestCombineErrorPathSentinels|TestCombineOfficialVectors|TestRecoverSecretWipesOnDigestFail|TestWipeZeroes|TestCombineBasic2of3'
```

Expected: `go vet` clean; all listed tests PASS. The full `slip39` suite (`go test ./slip39/`) is also green (run it if in doubt).

- [ ] **Step 5: Commit M2**

```bash
cd /tmp/scrub-batch-wt
git add slip39/combine.go slip39/combine_test.go
git commit -S -s -m "$(cat <<'EOF'
slip39: scrub recovered group-share secrets on all Combine paths (M2)

Combine scrubbed groupShares[].v + ems only on the success path; the three
error returns (combine.go:103/:108/:116) skipped the scrub, leaking recovered
group-share secrets on multi-group (GroupThreshold>=2) error/abort paths. Hoist
the scrub into a defer registered right after groupShares is declared so it
fires on every return path, with ems hoisted to a function-scope var. Also wipe
the interpolated digest buffer d on recoverSecret's digest-fail branch (it was
wiped only on success). Pure-additive; no control-flow or error change. Adds a
seam-free regression guard asserting the correct sentinel on each error path
(the leaked locals are unobservable seam-free per spec R0 Q1).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

Expected: one signed, DCO-signed commit on `feat/fix-scrub-batch`.

---

## Task 2: M3 — `seedxor/seedxor.go` wipe per-part `Entropy()` intermediates on every path

**Files:**
- Modify: `seedxor/seedxor.go` (`Combine` `:38`, `:44`)
- Test: `seedxor/seedxor_test.go` (extend the existing mismatched-lengths regression test; in-package `package seedxor`)

**What this fixes:** `Combine` scrubs the accumulator `out` on every exit but never wipes the per-part `Entropy()` copies: `parts[0].Entropy()` (consumed at `:38`) and each `e := p.Entropy()` (`:44`) — fresh secret allocations left live on success AND on the `errMismatchedLengths` early return (`:45-47`). **Test posture (Q1/Minor-2):** `e0`/`e` are function-locals, unobservable seam-free → the load-bearing test is a **regression+convention guard**: extend the existing `TestCombineMismatchedLengths` to also confirm a successful combine still returns the correct result (proves the added in-loop `wipe(e)` did not corrupt the XOR by wiping `e` before it was read). NOT a buffer-zeroed assertion.

- [ ] **Step 1: Write the M3 regression-guard test extension**

The existing `TestCombineMismatchedLengths` (`seedxor/seedxor_test.go:143-151`) already covers the `errMismatchedLengths` path. Add a focused success-path guard. Append this to the end of `seedxor/seedxor_test.go` (the file is `package seedxor`, already imports `errors` and `testing` and `seedhammer.com/bip39`):

```go
// TestCombineScrubNoCorruption is the M3 regression guard. The per-part entropy
// copies wiped by the fix (e0 at seedxor.go:38, e at :44) are function-local and
// unobservable seam-free (spec R0 Q1/Minor-2), so this is NOT a buffer-zeroed
// assertion. It proves the additive in-loop `wipe(e)` does not corrupt the XOR:
// each part's entropy is wiped only AFTER it has been XORed into out, so the
// combined result must still match the vector. Also re-confirms the
// errMismatchedLengths path (where wipe(e) now runs alongside wipe(out)) still
// returns its sentinel.
func TestCombineScrubNoCorruption(t *testing.T) {
	// Success path: the result must be byte-identical to the vector even though
	// each per-part entropy copy is wiped immediately after use.
	for _, v := range loadVectors(t) {
		t.Run(v.Name, func(t *testing.T) {
			parts := make([]bip39.Mnemonic, len(v.Parts))
			for i, p := range v.Parts {
				parts[i] = parseM(t, p)
			}
			want := parseM(t, v.Result)
			got, err := Combine(parts)
			if err != nil {
				t.Fatalf("Combine: %v", err)
			}
			if string(got.Entropy()) != string(want.Entropy()) {
				t.Fatalf("Combine entropy = %x, want %x (wipe(e) corrupted the XOR?)",
					got.Entropy(), want.Entropy())
			}
		})
	}

	// Mismatched-lengths path still returns its sentinel (now wipes e too).
	twelve := parseM(t, "romance wink lottery autumn shop bring dawn tongue range crater truth ability")
	twentyfour := parseM(t, "silent toe meat possible chair blossom wait occur this worth option bag nurse find fish scene bench asthma bike wage world quit primary indoor")
	if _, err := Combine([]bip39.Mnemonic{twelve, twentyfour}); !errors.Is(err, errMismatchedLengths) {
		t.Fatalf("mismatched: err = %v, want errMismatchedLengths", err)
	}
}
```

- [ ] **Step 2: Run the test to confirm it passes on `3a23dbb` (regression guard — green BEFORE the fix)**

Run:
```bash
cd /tmp/scrub-batch-wt
export PATH=$PATH:/home/bcg/.local/go/bin
go test ./seedxor/ -run TestCombineScrubNoCorruption -v
```

Expected: PASS (regression guard — the result is already correct at `3a23dbb`; the test exists to catch a wipe-before-read corruption introduced by Step 3).

- [ ] **Step 3: Apply the M3 fix (Old → New) in `seedxor/seedxor.go`**

**Edit 3a — bind `parts[0].Entropy()` to `e0`, copy into `out`, then `wipe(e0)` immediately (Q3: before the `interopLen` check, so the bad-length return at `:39-41` also leaves `e0` wiped; `out` is a distinct allocation — probe-confirmed safe).**

Old (`seedxor.go:38-39`):
```go
	out := append([]byte(nil), parts[0].Entropy()...)
	if !interopLen(len(out)) {
```

New:
```go
	e0 := parts[0].Entropy()
	out := append([]byte(nil), e0...)
	wipe(e0) // M3: scrub the first part's entropy copy (out is a distinct alloc)
	if !interopLen(len(out)) {
```

**Edit 3b — wipe each per-part `e` on BOTH the `errMismatchedLengths` early return and the success path (explicit in-loop `wipe`, NOT defer-in-loop — defers accumulate to function return and would delay the wipe).**

Old (`seedxor.go:43-52`):
```go
	for _, p := range parts[1:] {
		e := p.Entropy()
		if len(e) != len(out) {
			wipe(out)
			return nil, errMismatchedLengths
		}
		for i := range out {
			out[i] ^= e[i]
		}
	}
```

New:
```go
	for _, p := range parts[1:] {
		e := p.Entropy()
		if len(e) != len(out) {
			wipe(out)
			wipe(e) // M3: scrub the mismatched part's entropy before the early return
			return nil, errMismatchedLengths
		}
		for i := range out {
			out[i] ^= e[i]
		}
		wipe(e) // M3: scrub this part's entropy copy after it is XORed into out
	}
```

- [ ] **Step 4: Run the M3 test + the existing seedxor suite to confirm PASS, no regression**

Run:
```bash
cd /tmp/scrub-batch-wt
export PATH=$PATH:/home/bcg/.local/go/bin
go vet ./seedxor/
go test ./seedxor/ -v
```

Expected: `go vet` clean; the whole `seedxor` suite PASSES (`TestCombineScrubNoCorruption`, `TestCombineVectors`, `TestCombineOrderIndependent`, `TestCombineNoCallerMutation`, `TestCombineMismatchedLengths`, `TestCombineBadLength`, `TestCombineTooFewParts`, `TestDescribe`). No `FAIL`.

- [ ] **Step 5: Commit M3**

```bash
cd /tmp/scrub-batch-wt
git add seedxor/seedxor.go seedxor/seedxor_test.go
git commit -S -s -m "$(cat <<'EOF'
seedxor: wipe per-part entropy intermediates on every Combine path (M3)

Combine scrubbed the accumulator out everywhere but left each per-part
Entropy() copy live: parts[0]'s (seedxor.go:38) and each parts[1:]'s e
(seedxor.go:44), on success and on the errMismatchedLengths early return. Bind
parts[0].Entropy() to e0 and wipe it immediately after the copy (before the
interopLen check, so the bad-length return also leaves it wiped; out is a
distinct allocation), and wipe each per-part e after it is XORed in and before
the mismatched-lengths return. Explicit in-loop wipe (not defer-in-loop, which
would delay the wipe to function return). Mirrors the package's own
singlesig_derive.go bind->use->wipe convention. Pure-additive; no behaviour
change. Adds a seam-free regression guard that the in-loop wipe does not corrupt
the XOR (the locals are unobservable seam-free per spec R0 Q1).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

Expected: one signed, DCO-signed commit on `feat/fix-scrub-batch`.

---

## Task 3: M4 — `gui/bip85.go` `defer pkey.Zero()` + precedented test-only `bip85PkeyHook`

**Files:**
- Modify: `gui/bip85.go` (`deriveBip85Child` after `:106`, the `import` block, and a new test-only hook var near `bip85SeedHook` at `:238-241`)
- Test: `gui/bip85_test.go` (new `TestDeriveBip85Child_ScrubsPkey`, in-package `package gui`)

**What this fixes:** `deriveBip85Child` scrubs `priv` (the serialized 32 bytes) and `k` (the ExtendedKey) but never scrubs `pkey` — the live `*btcec.PrivateKey` holding the raw leaf scalar survives the return. **Test posture (Q1):** unlike the other three, M4 gets a TRUE fail-before/pass-after assertion via a precedented, in-file-sanctioned test-only hook (`bip85PkeyHook`, mirroring the existing `bip85SeedHook` at `:238-241`): a test holds the `*btcec.PrivateKey` and asserts `pkey.Key.IsZero()==true` after the function returns — this FAILS on `3a23dbb` (no `pkey.Zero()`) and PASSES after the fix. **Probe-confirmed:** with the hook + `defer pkey.Zero()`, the test passes; removing only the `defer pkey.Zero()` line makes it fail with "pkey.Key not zeroed".

- [ ] **Step 1: Add the test-only hook + the production fix to `gui/bip85.go` FIRST, then write a failing test (TDD note)**

**Why fix-then-test here, uniquely:** the test cannot even compile without the new `bip85PkeyHook` symbol and the btcec import. To honour fail-before/pass-after, after both the hook and the test are in place we will *temporarily revert only the `defer pkey.Zero()` line* in Step 2b to observe the FAIL, then restore it. (The hook and import are inert test infrastructure; the load-bearing production change under test is the single `defer pkey.Zero()` line.)

**Edit 1a — add the `github.com/btcsuite/btcd/btcec/v2` import.**

Old (`gui/bip85.go:8`):
```go
	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
```

New:
```go
	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
```

**Edit 1b — add `defer pkey.Zero()` immediately after `priv := pkey.Serialize()` (`pkey` is guaranteed non-nil there — the `err != nil` branch at `:102-105` already returned), and call the test-only hook synchronously while `pkey` is live.**

Old (`gui/bip85.go:106-108`):
```go
	priv := pkey.Serialize() // 32-byte secret
	k.Zero()
	defer wipeBytes(priv)
```

New:
```go
	priv := pkey.Serialize() // 32-byte secret
	k.Zero()
	defer wipeBytes(priv)
	defer pkey.Zero() // M4: scrub the live leaf EC private-key scalar (pkey.Key)
	// Test-only seam: observe the live *PrivateKey so a test can assert
	// pkey.Zero() scrubbed pkey.Key on return. nil in production. Mirrors
	// bip85SeedHook (the sanctioned in-file test-only seam).
	if bip85PkeyHook != nil {
		bip85PkeyHook(pkey)
	}
```

**Edit 1c — declare the test-only hook var next to `bip85SeedHook`.**

Old (`gui/bip85.go:238-241`):
```go
// bip85SeedHook is a test-only seam to observe the master + child mnemonics (to
// assert both are scrubbed on exit, I-3). nil in production. Mirrors
// singleSigSeedHook.
var bip85SeedHook func(master, child bip39.Mnemonic)
```

New:
```go
// bip85SeedHook is a test-only seam to observe the master + child mnemonics (to
// assert both are scrubbed on exit, I-3). nil in production. Mirrors
// singleSigSeedHook.
var bip85SeedHook func(master, child bip39.Mnemonic)

// bip85PkeyHook is a test-only seam to observe the leaf EC private key so a test
// can assert deriveBip85Child's `defer pkey.Zero()` scrubbed pkey.Key on return
// (M4). nil in production. Mirrors bip85SeedHook (the sanctioned in-file seam).
var bip85PkeyHook func(pkey *btcec.PrivateKey)
```

- [ ] **Step 2: Write the M4 fail-before/pass-after test**

Append this to the end of `gui/bip85_test.go` (the file is `package gui`; add the `github.com/btcsuite/btcd/btcec/v2` import to its import block):

First, update the import block at the top of `gui/bip85_test.go` from:

```go
import (
	"testing"
	"testing/synctest"
	"time"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
)
```

to:

```go
import (
	"testing"
	"testing/synctest"
	"time"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
)
```

Then append this test:

```go
// TestDeriveBip85Child_ScrubsPkey is the M4 GENUINE fail-before/pass-after
// assertion (the lone true one in this batch). Via the sanctioned, in-file
// bip85PkeyHook (mirroring bip85SeedHook), it captures the live *btcec.PrivateKey
// and asserts the leaf scalar (pkey.Key) is zeroed after deriveBip85Child
// returns. FAILS on 3a23dbb (no `defer pkey.Zero()`); PASSES after the fix.
func TestDeriveBip85Child_ScrubsPkey(t *testing.T) {
	var captured *btcec.PrivateKey
	bip85PkeyHook = func(p *btcec.PrivateKey) { captured = p }
	defer func() { bip85PkeyHook = nil }()

	child, err := deriveBip85Child(abandonAboutMnemonic(), "", 12, 0)
	if err != nil {
		t.Fatalf("deriveBip85Child: %v", err)
	}
	// No-behaviour-regression: the canonical abandon/12/idx0 child is unchanged.
	const wantChild = "prosper short ramp prepare exchange stove life snack client enough purpose fold"
	if got := child.String(); got != wantChild {
		t.Fatalf("child mismatch:\n got %q\nwant %q", got, wantChild)
	}
	if captured == nil {
		t.Fatal("bip85PkeyHook never fired — the hook was not invoked")
	}
	// The load-bearing assertion: the leaf scalar is scrubbed on return.
	if !captured.Key.IsZero() {
		t.Fatal("pkey.Key not zeroed after deriveBip85Child returned (M4: missing defer pkey.Zero())")
	}
}
```

- [ ] **Step 2b: Verify FAIL-BEFORE by temporarily reverting only the `defer pkey.Zero()` line**

Run (temporarily remove the one production line under test, run, then restore it):
```bash
cd /tmp/scrub-batch-wt
export PATH=$PATH:/home/bcg/.local/go/bin
# Temporarily strip the defer pkey.Zero() line:
cp gui/bip85.go /tmp/bip85.go.bak
grep -v 'defer pkey.Zero() // M4: scrub the live leaf EC private-key scalar' gui/bip85.go > /tmp/bip85.go.nofix && mv /tmp/bip85.go.nofix gui/bip85.go
go test ./gui/ -run TestDeriveBip85Child_ScrubsPkey -v
```

Expected: **FAIL** with `pkey.Key not zeroed after deriveBip85Child returned (M4: missing defer pkey.Zero())`. (Probe-confirmed.)

Then restore the fix:
```bash
cd /tmp/scrub-batch-wt
mv /tmp/bip85.go.bak gui/bip85.go
```

Expected: `gui/bip85.go` is back to the fixed version (the `defer pkey.Zero()` line present again).

- [ ] **Step 3: Run the M4 test to confirm PASS-AFTER, plus the existing bip85 tests for no regression**

Run:
```bash
cd /tmp/scrub-batch-wt
export PATH=$PATH:/home/bcg/.local/go/bin
go vet ./gui/ 2>&1 | grep -v 'gui/op/draw_test.go' || true
go test ./gui/ -run 'TestDeriveBip85Child_ScrubsPkey|TestDeriveBip85Child_CanonicalVector|TestDeriveBip85Child_AbandonGoldens|TestBip85DeriveFlow_ScrubsBothMnemonics' -v
```

Expected: `TestDeriveBip85Child_ScrubsPkey` PASS, and the existing bip85 tests PASS (no behaviour regression). Note: `go vet ./gui/` may emit a PRE-EXISTING, unrelated note about `gui/op/draw_test.go:176` (`testing.ArtifactDir requires go1.26 …` — a go1.25/1.26 file-tag artifact in `gui/op`, not a Track-B file); ignore it. The vet of the Track-B-touched `gui` package itself is clean.

- [ ] **Step 4: Commit M4**

```bash
cd /tmp/scrub-batch-wt
git add gui/bip85.go gui/bip85_test.go
git commit -S -s -m "$(cat <<'EOF'
gui/bip85: scrub the leaf EC private-key scalar with defer pkey.Zero() (M4)

deriveBip85Child scrubbed priv (the serialized 32 bytes) and k (the
ExtendedKey) but never scrubbed pkey — the live *btcec.PrivateKey holding the
raw leaf scalar survived the return (Serialize() is a value receiver, so it
copies; pkey.Key stayed live). Add `defer pkey.Zero()` right after the
serialization (pkey is guaranteed non-nil there). Adds bip85PkeyHook, a
test-only seam mirroring the sanctioned bip85SeedHook, so a test asserts
pkey.Key.IsZero() after return — fails before this change, passes after. The
hook is nil in production. Pure-additive; no behaviour change.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

Expected: one signed, DCO-signed commit on `feat/fix-scrub-batch`.

---

## Task 4: L1 — `gui/codex32_polish.go` capture + `wipeBytes()` the `DecodeMS1` probe entropy

**Files:**
- Modify: `gui/codex32_polish.go` (`confirmCodex32Flow` `:103-104`)
- Test: `gui/codex32_polish_test.go` (new `TestConfirmCodex32Flow_ShowSecretGate`, in-package `package gui`)

**What this fixes:** `confirmCodex32Flow` calls `codex32.DecodeMS1(scan)` purely as a validity probe and discards the returned entropy with `_` (`:103`), leaving the secret BIP-39 entropy subslice un-scrubbed. **Minor-3:** `wipeBytes(ent)` wipes ONLY the entropy subslice `DecodeMS1` returns (`data[1:]`/`data[2:]`); the function does not hold a handle to the whole `data` buffer — do NOT try to wipe more. **Test posture (Q1/Minor-2):** `ent` is consumed-and-wiped within the function and unobservable seam-free → the load-bearing test is a **regression+convention guard**: assert the `showSecret` decision is unchanged (unshared ms1 `entr` → the "Show secret" affordance opens the decode view), proving the additive `wipeBytes(ent)` did not perturb the probe semantics. NOT a buffer-zeroed assertion. **Do NOT touch the Track-A sites** `singlesig_verify.go:116` / `multisig_verify.go:93`.

- [ ] **Step 1: Write the L1 regression-guard test**

Append this to the end of `gui/codex32_polish_test.go` (the file is `package gui`, already imports `strings`, `testing`, `seedhammer.com/codex32`). The `mustCodex32T` helper, `confirmCodex32Flow`, `NewContext`, `newPlatform`, `runUI`, `click`, `Button2`, `descriptorTheme`, `uiContains`, and `bip39` are all already present in the `gui` test package (the sibling `gui/ms1_decode_test.go:76` `TestConfirmShowSecretGate` uses exactly this shape). Add the `seedhammer.com/bip39` import:

First, update the import block at the top of `gui/codex32_polish_test.go` from:

```go
import (
	"strings"
	"testing"

	"seedhammer.com/codex32"
)
```

to:

```go
import (
	"strings"
	"testing"

	"seedhammer.com/bip39"
	"seedhammer.com/codex32"
)
```

Then append this test:

```go
// TestConfirmCodex32Flow_ShowSecretGate is the L1 regression+convention guard.
// The probe entropy that the fix wipes (codex32_polish.go:103) is consumed and
// scrubbed inside confirmCodex32Flow and unobservable seam-free (spec R0
// Q1/Minor-2), so this is NOT a buffer-zeroed assertion. It proves the additive
// wipeBytes(ent) does NOT perturb the showSecret decision: an unshared ms1
// `entr` secret (msErr == nil && f.Unshared) must still offer "Show secret"
// (Button2 opens the decode view). Mirrors TestConfirmShowSecretGate
// (gui/ms1_decode_test.go:76).
func TestConfirmCodex32Flow_ShowSecretGate(t *testing.T) {
	const ms1 = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f" // unshared entr secret (entropy 0*16)
	s := mustCodex32T(t, ms1)
	want := bip39.LabelFor(bip39.New(make([]byte, 16))[0]) // first decoded word label
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Button2) // Show secret -> opens ms1DecodeFlow (only for unshared)
	frame, quit := runUI(ctx, func() { confirmCodex32Flow(ctx, &descriptorTheme, s) })
	defer quit()
	seen := false
	for i := 0; i < 10; i++ {
		c, ok := frame()
		if !ok {
			break
		}
		if uiContains(c, want) {
			seen = true
			break
		}
	}
	if !seen {
		t.Fatal("Show secret did not open the decode view on the unshared secret (showSecret gate perturbed?)")
	}
}
```

- [ ] **Step 2: Run the test to confirm it passes on `3a23dbb` (regression guard — green BEFORE the fix)**

Run:
```bash
cd /tmp/scrub-batch-wt
export PATH=$PATH:/home/bcg/.local/go/bin
go test ./gui/ -run TestConfirmCodex32Flow_ShowSecretGate -v
```

Expected: PASS (regression guard — the `showSecret` gate is already correct at `3a23dbb`; the test exists to prove the `wipeBytes(ent)` of Step 3 does not break it).

- [ ] **Step 3: Apply the L1 fix (Old → New) in `gui/codex32_polish.go`**

Old (`gui/codex32_polish.go:103-104`):
```go
	_, _, _, msErr := codex32.DecodeMS1(scan)
	showSecret := f.Unshared && msErr == nil
```

New:
```go
	_, _, ent, msErr := codex32.DecodeMS1(scan)
	wipeBytes(ent) // L1: scrub the discarded probe entropy (nil on err -> no-op)
	showSecret := f.Unshared && msErr == nil
```

- [ ] **Step 4: Run the L1 test + the existing codex32_polish / ms1_decode tests to confirm PASS, no regression**

Run:
```bash
cd /tmp/scrub-batch-wt
export PATH=$PATH:/home/bcg/.local/go/bin
go vet ./gui/ 2>&1 | grep -v 'gui/op/draw_test.go' || true
go test ./gui/ -run 'TestConfirmCodex32Flow_ShowSecretGate|TestConfirmShowSecretGate|TestCodex32StatusLine|TestCodex32FieldLine|TestMS1DecodeFlow' -v
```

Expected: all listed tests PASS. (Ignore the pre-existing `gui/op/draw_test.go` vet note as in Task 3.)

- [ ] **Step 5: Commit L1**

```bash
cd /tmp/scrub-batch-wt
git add gui/codex32_polish.go gui/codex32_polish_test.go
git commit -S -s -m "$(cat <<'EOF'
gui/codex32: scrub the DecodeMS1 probe entropy in confirmCodex32Flow (L1)

confirmCodex32Flow used codex32.DecodeMS1 purely as a validity probe and
discarded the returned BIP-39 entropy subslice with _ (codex32_polish.go:103),
leaving it un-scrubbed against the codebase's own ms1_decode.go convention.
Capture the entropy and wipeBytes it (nil on the err path -> no-op). Wipes only
the entropy subslice the function holds, not the whole codex32 data buffer
(which it has no handle to). This is the disjoint codex32_polish L1 site only;
the two verify-flow L1 sites belong to Track A and are untouched. Pure-additive;
no behaviour change. Adds a seam-free regression guard that the showSecret gate
is unchanged (the probe entropy is unobservable seam-free per spec R0 Q1).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

Expected: one signed, DCO-signed commit on `feat/fix-scrub-batch`.

---

## Task 5: Final verification + leave the fork clean

**Files:**
- No source changes. Full-suite verification + worktree teardown.

- [ ] **Step 1: Full build, vet, and test across the whole module**

Run:
```bash
cd /tmp/scrub-batch-wt
export PATH=$PATH:/home/bcg/.local/go/bin
go build ./...
go vet ./... 2>&1 | grep -v 'gui/op/draw_test.go' || true
go test ./...
```

Expected:
- `go build ./...` — no output (success).
- `go vet ./...` — clean except the PRE-EXISTING, unrelated `gui/op/draw_test.go:176` `testing.ArtifactDir requires go1.26 …` note (a go1.25-tagged file artifact, present at `3a23dbb`, not introduced by Track B; the `grep -v` filters it). If ANY other vet finding appears, STOP and investigate.
- `go test ./...` — all packages `ok` (or `[no test files]`); no `FAIL`. The three target packages and their new tests are green: `TestCombineErrorPathSentinels`, `TestCombineScrubNoCorruption`, `TestDeriveBip85Child_ScrubsPkey`, `TestConfirmCodex32Flow_ShowSecretGate`.

- [ ] **Step 2: Confirm the commit log — four commits, all signed + DCO, correct author**

Run:
```bash
cd /tmp/scrub-batch-wt
git log --oneline --format='%h %G? %an <%ae> %s' 3a23dbb..HEAD
git log --format='%h %s%n%b' 3a23dbb..HEAD | grep -c 'Co-Authored-By:'
```

Expected: exactly 4 commits (M2, M3, M4, L1) above `3a23dbb`, each with a signature indicator (`G`/`U` per the SSH-signing setup; NOT `N`), author `Brian Goss <goss.brian@gmail.com>`, and the `Co-Authored-By:` count == 4.

- [ ] **Step 3: Confirm the diff scope — exactly four source files + four test files, no helper edits, no Track-A files**

Run:
```bash
cd /tmp/scrub-batch-wt
git diff --stat 3a23dbb..HEAD
```

Expected: ONLY these eight files changed:
- `slip39/combine.go`, `slip39/combine_test.go`
- `seedxor/seedxor.go`, `seedxor/seedxor_test.go`
- `gui/bip85.go`, `gui/bip85_test.go`
- `gui/codex32_polish.go`, `gui/codex32_polish_test.go`

Verify NONE of these appear: `gui/slip39_polish.go` (the `wipeBytes` helper — must NOT be edited), `slip39/feistel.go` (`slip39.wipe` — must NOT be edited), `gui/singlesig_verify.go`, `gui/multisig_verify.go`, `gui/multisig_supply.go`, `bundle/verify.go`, `gui/md1_gather.go` (Track-A files — must NOT be touched). If any forbidden file appears, STOP.

- [ ] **Step 4: (Do NOT merge.) Leave the branch in place for the exec-review gate**

The branch `feat/fix-scrub-batch` stays on its worktree for the mandatory post-implementation adversarial exec review. **Do NOT merge to `main`, do NOT push.** Merge is a later, separately-gated step (orchestration plan: serial merge B → A after each track's exec review).

- [ ] **Step 5: Confirm the fork's `main` checkout is clean and untouched**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer
git rev-parse HEAD          # expect 3a23dbb…
git branch --show-current   # expect main
git status --short          # expect EMPTY
git worktree list           # expect main@3a23dbb + feat/fix-scrub-batch@/tmp/scrub-batch-wt + seedhammer-wt-bip39 (untouched)
```

Expected: the primary `main` checkout is still on `3a23dbb` with an empty working tree; `seedhammer-wt-bip39` is untouched; the new `feat/fix-scrub-batch` worktree at `/tmp/scrub-batch-wt` carries the four commits.

---

## Self-Review (run after the plan, before handoff)

**Spec coverage:** every spec finding has a task — M2 → Task 1, M3 → Task 2, M4 → Task 3, L1 → Task 4; worktree/baseline → Task 0; final gate → Task 5. All three folded R0 Minors are addressed (Minor-1: Task 1 Edit 3a/3b; Minor-2: stated in every task's "Test posture"; Minor-3: Task 4 preamble + commit). Q1 (M4 hook, seam-free guards), Q2 (digest-fail-only `wipe(d)`), Q3 (`e0` placement before `interopLen`) all folded.

**Placeholder scan:** no TBD/TODO; every code step shows the actual Old→New diff or full test code; every run step has an exact command and expected output.

**Type consistency:** `bip85PkeyHook func(*btcec.PrivateKey)` is declared in Task 3 Edit 1c and consumed in Task 3 Edit 1b and the Task 3 test, all using `github.com/btcsuite/btcd/btcec/v2`. `copyShare`/`copyShares` defined and used in Task 1. `e0`/`e` consistent across Task 2 edits and test. `ent` consistent in Task 4.

## Execution notes for the implementer

- **COMPILE-CHECK every snippet in the throwaway worktree.** This plan's diffs and tests were all probe-applied at `3a23dbb` and confirmed: all four fixes build + vet clean, the existing `slip39`/`seedxor`/`gui` suites pass unchanged, the M4 hook test fails-before/passes-after, and the M2 error-path sentinels hold. If any step diverges from its expected output, STOP and use `superpowers:systematic-debugging` — do not "fix forward" past a red gate.
- **TinyGo is the real final gate** (the controller's integration pass), not host `go build`. Every fix is TinyGo-safe (`defer`/closures/`defer`-method-call compile on the device target), but this plan verifies only host build/vet/test; do NOT claim the TinyGo gate from a host run.
- **Residual for the plan-R0 reviewer:** the plan adds ONE new (test-only, nil-in-production, in-file-precedented) symbol — `bip85PkeyHook`. Confirm the reviewer is comfortable that this is the sanctioned exception (per the spec R0 Q1 ruling), not a new production seam, and that the M2/M3/L1 guards being regression+convention (not buffer-zeroed) matches the ruling.
