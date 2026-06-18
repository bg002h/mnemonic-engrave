<!--
Persisted verbatim. opus-architect MANDATORY whole-diff execution review of Cycle D Phase
D2 (SLIP-39 GUI recover). Reviewer agentId aedf54bb12af20b6b. Diff feat/slip39-recovery-gui
f0092d5..f995961 (5 commits). Verdict: GREEN — 0C/0I, cleared to merge. Ran 50x + -race;
traced selectForCombine vs Combine preconditions (no bad input constructible); verified the
hold-to-confirm + fingerprint gates guard engrave, passphrase isolation, Button2 no-hang,
secret-wipe, scope. One informational minor (eager ConsistentShares defers member-threshold
to Combine — by D1 design). The text below is the agent's report exactly as returned; do not edit.
-->

# EXECUTION REVIEW — Cycle D D2 (SLIP-39 GUI recover) whole diff

**Reviewer:** opus architect (independent adversarial whole-diff execution review)
**Commit range:** `f0092d5..f995961` (5 commits) — base fork `main` `f0092d5` (D1 crypto, trusted)
**Branch / worktree:** `feat/slip39-recovery-gui` @ `/scratch/code/shibboleth/seedhammer-wt-slip39-d2`
**Date:** 2026-06-18
**Scope:** `gui/gui.go` (+14), `gui/slip39_polish.go` (+403), `gui/slip39_polish_test.go` (+431) = 827 insertions. Read the committed files; did NOT re-review `slip39/`.

## Reproduced build/test/vet/gofmt (tails)

```
go test ./gui/ ./slip39/ ./bip39/   → ok gui 3.626s | ok slip39 0.046s | ok bip39 0.030s
go vet ./gui/                        → (clean)
gofmt -l gui/gui.go slip39_polish.go slip39_polish_test.go → (empty / clean)
go build ./...                       → DONE (clean)
go test -run 'SLIP39|Recover' -v ./gui/ → 21 tests, all PASS
go test -count=50 -run 'TestRecoverSLIP39*|TestSelectForCombine' ./gui/ → ok 0.837s
go test -race -run 'SLIP39|Recover|SelectForCombine' ./gui/ → ok 1.638s
git diff --name-only f0092d5..HEAD   → gui/gui.go, gui/slip39_polish.go, gui/slip39_polish_test.go (only gui/)
```
`go vet ./...` emits warnings only in untouched packages (`bspline`, `gui/op`, `backup`, `engrave` — pre-existing unkeyed-field + go1.26 `ArtifactDir` noise); the in-scope `./gui/` package proper is vet-clean. Worktree clean (no stash / uncommitted).

## Per-focus findings

**1 — I1 fix / `selectForCombine` correctness (CLEAN).** Traced against `Combine`'s exact-count preconditions (`combine.go:94` every group `len==mt`; `:107` exactly GT groups; `:89` no duplicate member). `selectForCombine` (slip39_polish.go:168-191) sorts `gids`, includes a group only when `groupSatisfied` (`len==gs[0].MemberThreshold`), stops at `picked==groupThreshold`, and returns `ok=false` if `<GT`. Output is therefore exactly GT groups × their mt members — no partial/extra group, no over-fill. Over-fill is prevented at *collection* time too: line 240 rejects any candidate whose group is already `groupSatisfied`, so a group can never exceed mt. `ConsistentShares` (line 234) rejects duplicate `(group,member)` before append. The stray-partial-group unit test (`TestSelectForCombinePrunesStrayPartialGroup`) is real: it builds a roster with a lingering 1-member group 2 and asserts exactly the 4 members of groups 0+1 are returned and group 2 leaks nothing. I could not construct a roster that feeds `Combine` an over-filled or wrong-count input.

**2 — Two-level collection / roster control flow (CLEAN).** Stop condition `countSatisfied < GT` is monotone (each accepted share strictly grows one under-filled group; over-fill blocked). Already-satisfied-group rejection present (line 240). Dead-end/cancel: Back at the first prompt → `(nil,false)` (`TestRecoverSLIP39BackoutRecognized`). Subsequent shares sized to `L=len(first.Mnemonic)` (line 217/224). No off-by-one, no infinite loop, no path making a recoverable pile unrecoverable or accepting a wrong pile (final authority is `Combine`).

**3 — C1 passphrase wiring (CLEAN).** The SLIP-39 `pass` var is used at exactly ONE site — `Combine(sel, []byte(pass))` (line 287) — and is never passed to `backupWalletFlow`, which independently prompts "Add a BIP-39 passphrase?" (gui.go:1929 body, confirmed byte-identical to base). Defaults to `""` (Skip). `TestRecoverSLIP39` asserts Skip→`61cf…2664`, `TestRecoverSLIP39Passphrase` asserts TREZOR→`b43c…0864`; `TestSLIP39RecoveredSeedIsolatedFromBIP39Passphrase` pins isolation. No cross-contamination.

**4 — Button2 drain / no-hang (CLEAN).** `confirmSLIP39Flow` drains Button2 unconditionally every frame (line 107, with the R0-C1 comment) and acts only `if recover`; `confirmSLIP39Fingerprint` drains Button2 (line 425). Verified the mechanism: `EventRouter.Next` (event.go:266) only pops the head if a registered filter matches, and `Context.Frame` does *not* `Reset()` — so a direct-call loop must drain. The `ConfirmWarningScreen` ack/high-iter screens do not drain Button2, but they are a pre-existing shared widget always run inside the production `runUI` loop, which calls `ctx.Reset()` (→ `Router.Reset()`, event.go:281) every frame to discard unmatched head events; this matches existing usage (gui.go:2083) and the codex32 template. No new direct-call path reaches a ConfirmWarningScreen with a stuck Button2. `TestConfirmSLIP39LoneButton2NoHang` (direct-call, two queued Button2s then Button3) passes.

**5 — §3 hold + §5.4 fingerprint gates (CLEAN).** `engraveRecoveredSLIP39` (lines 384-408): hold-to-confirm ack (line 393, returns false on cancel) THEN fingerprint check (line 403, false on Back) THEN `backupWalletFlow` (line 406). `ConfirmWarningScreen.Layout` yields `ConfirmYes` only at `progress==1` (a real hold) and `ConfirmNo` on Button1. Either decline → `engraveSLIP39` `continue`s back to the original confirm, never to engrave. Fingerprint uses `%.8X` (line 416) and `masterFingerprintFor(m, &chaincfg.MainNetParams, "")` (line 398). The recovered seed cannot reach engrave without both gates. `TestSLIP39FingerprintBackRecognized` and `TestEngraveSLIP39RecoverToBackup` (full dispatch through ack `WRONG seed` → fingerprint `BDDBDA4F` → backup `GIFT`) confirm.

**6 — Go-specific hazards (CLEAN).** Determinism: `selectForCombine` sorts `gids` before iterating; `countSatisfied`/`allShares` are order-independent; `Combine` re-groups internally. 50× + `-race` runs stable. Slice aliasing: `append(out, gs...)` / `append(allShares,…)` copy elements; shares are value types holding slices, but `Combine` only reads `.Value`. `case 3:` picker: `n==0`→`break` (re-displays menu, matching base `break`-out-of-switch semantics); default path still reaches 20 via `TestSLIP39LengthPickDefault20`; cancel via `TestSLIP39LengthPickCancel`. `bip39.New` panics outside entropy ∈{16,20,24,28,32}, but `Combine`'s `validSecretLen` guarantees exactly those sizes, so no panic path. `inputSLIP39Flow` title-param change is mechanical — word-entry logic unchanged from base.

**7 — Security / scope (CLEAN).** No `log`/`fmt.Print`/`println`/`panic` of secret or passphrase in slip39_polish.go. Recovered `secret` wiped at line 293 (after `bip39.New`), exactly as plan L270/spec L255 specify. `gui.go` gained NO new import (diff touches only lines 793/865/2032). `backupWalletFlow` and `masterFingerprintFor` bodies byte-identical to base (verified by diff). Full `./gui/` suite green including codex32, BIP-39, backup goldens, `TestConfirmSLIP39Render`, `TestEngraveSLIP39BackoutRecognized`.

**8 — Test soundness (CLEAN).** `driveShare` genuinely types each word (full lowercase letters + Button3) through `inputSLIP39Flow`'s real word-completion path — not a shortcut. `TestRecoverSLIP39MultiGroup` is a non-tautological round-trip: GT=2-over-2-groups fixture, enters 4 distinct shares across 2 groups, asserts recovered entropy `101112…1f`. Assertions are load-bearing (hex of recovered entropy vs expected, distinct Skip/TREZOR secrets).

## Findings

- **CRITICAL:** none.
- **IMPORTANT:** none.
- **MINOR (informational, no fix required):** `ConsistentShares` does not check `MemberThreshold` equality within a group, so a mixed-threshold group could be eagerly accepted and only rejected later by `Combine` (`errMemberThresholdMismatch`, combine.go:87). This surfaces as a later error, never a wrong seed and never a panic — it matches D1's intentional "eager check is count/threshold-agnostic; `Combine` is the authority" design and the plan's delegation. Optional future UX polish only.

## Verdict

**GREEN — 0 Critical / 0 Important. Cleared to merge.**
