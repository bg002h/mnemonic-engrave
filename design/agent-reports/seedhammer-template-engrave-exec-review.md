# Exec Review — SeedHammer fork: on-device wallet-policy TEMPLATE engraving

- **Type:** MANDATORY independent whole-diff adversarial EXECUTION review (post-implementation; R0 validated plan correctness, this catches implementation-introduced regressions/deviations TDD missed).
- **Reviewer:** opus architect (independent).
- **Date:** 2026-06-20.
- **Diff under review:** `main..HEAD` on `feat/template-engrave` in worktree `/tmp/seedhammer-wt-template`. Base fork `main` `39cb5cf`; branch HEAD `3d328d6`, 8 commits, 24 files (+1417/-20).
- **SPEC:** `design/SPEC_seedhammer_template_engrave.md` (R0 GREEN). **Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_template_engrave.md` (R0 GREEN).
- **Test run:** `go build ./...` exit 0; `go test -count=1 ./md/... ./bundle/... ./gui/...` → **ALL PASS** (`md` 0.018s, `bundle` 0.006s, `gui` 10.0s, `gui/op|saver|text|widget` PASS). Spot-checked key tests are non-vacuous (foreign-mk1 negative, id-space distinction, own-readback, consent strings, depth-2 experimental gate all genuinely exercised).

---

## VERDICT

**NOT GREEN — 1 Critical / 0 Important.** Blocking ID: **C1** (template-engrave guard over-refuses legacy `multi`-in-combinator, wrongly rejecting the §5 degrade2 general-miniscript wallet that the authoritative toolkit ADMITS, contradicting SPEC DD3/DD7 and the Task-4 strip golden).

Everything else verified clean: the WDT-Id port is byte-correct against authoritative Rust (`b02b44037119e6b6fd1d82f61aa17e21` confirmed by the `md inspect` CLI at the pinned `54dd765`); all four mint sites route through the form-aware selector and the keyed/full-policy path is byte-identical; the strip goldens are genuinely independent (the degrade2 template golden is byte-for-byte the toolkit's `bundle --md1-form=template` output, re-generated and matched during this review); D1 supply flow untouched; no GUI regression to the default full path; TinyGo static check clean.

---

## Critical

### C1 — Template-engrave guard OVER-REFUSES legacy `multi`-in-combinator → wrongly rejects the §5 degrade2 general wallet (the toolkit ADMITS it)
- **Location:** `md/template_guard.go:54-59` (the `case tagSortedMulti, tagMulti:` arm) + the file-header comment `md/template_guard.go:7-9` and the inline comment `:56`, which encode the same wrong premise ("sortedmulti/multi nested under a combinator").
- **Problem:** The guard refuses BOTH `tagSortedMulti` AND `tagMulti` when `inCombinator==true`. But the authoritative off-device gate (`mnemonic-toolkit synthesize.rs:1113 template_admissible`) refuses ONLY what fails to render through rust-miniscript: `tr(sortedmulti_a)` and `sortedmulti`-in-combinator. Legacy `multi(...)` is a real, renderable miniscript fragment; nesting it under `or_i`/`and_v`/`thresh` renders fine. The §5 degrade2 11-key degrading wallet is `wsh(or_i(and_v(...multi(3,...)), or_i(and_v(...multi(2,...)), or_i(and_v(...multi(2,...)), and_v(...multi(1,...))))))` — `multi` under combinators — and the guard refuses it.
- **Why blocking:** This directly contradicts (a) the verified authoritative toolkit semantics, (b) SPEC DD3 ("Scope = any ADMISSIBLE md1 ... general miniscript like the §5 degrading wallet") and DD7 (the §5 wallet "stays in scope"), and (c) the Task-4 strip golden `md/testdata/template/degrade2_11key.*` which ships the §5 wallet as a first-class supported template fixture. `templateEngraveShapeGuardChunks` is described in-code as "the single template-engrave gate"; refusing a SPEC-in-scope, toolkit-admissible shape is an incorrect refusal that engraves nothing where it should engrave a valid (recoverable) template.
- **Evidence (verified against authoritative source this review):**
  - Toolkit `template_admissible` (`mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs:1113-1122`): for `n>1` returns `to_miniscript_descriptor(d,0).is_ok()` (+ no hardened use-site) — a pure renderability test. Its own unit test (`synthesize.rs` `template_admissible_gate`) asserts `wsh(or_d(...))` "general policy admitted".
  - Toolkit ADMITS degrade2 as a template: `mnemonic bundle --network mainnet --md1-form=template --descriptor-file .examples-build/degrade2.desc --group-size 0` exits 0 and its `md1` lines are **byte-identical** to the committed golden `md/testdata/template/degrade2_11key.tmpl.md1.txt`.
  - Toolkit REFUSES `sortedmulti`-in-combinator: the same command on a `wsh(or_i(sortedmulti(...),...))` descriptor errors "sortedmulti inside a combinator ... does not render". (So the guard's `tagSortedMulti`-in-combinator arm IS correct; only the `tagMulti` half is wrong.)
  - Fork guard REFUSES degrade2: a temporary probe (`Reassemble(degrade2_11key.policy.md1.txt)` → `templateEngraveShapeGuard`, and `TemplateEngraveShapeGuardChunks` on both policy and template) returned `errTemplateUnsupportedShape` for all three. (Probe removed; tree was left clean.)
- **Reachability / mitigant (does NOT downgrade severity, but informs the fix):** the two CURRENT GUI callers (`gui/singlesig.go:97` single-sig build; `gui/multisig_build.go:114` multisig BUILD, whose template picker is locked to the three sortedmulti wrappers) never construct a §5-style general policy, so the over-refusal is not reachable from today's GUI. But the defect is in an exported `md` API that the SPEC's own scope (and any future supplied-template path) requires to admit §5, and the test suite never probes the guard with the degrade2 fixture — so the bug is latent and untested. Fix it now; it is a one-line correction plus a test.
- **Fix:**
  1. Split the arm so only `sortedmulti` is refused in a combinator:
     ```go
     case tagSortedMulti:
         if inCombinator {
             return errTemplateUnsupportedShape // sortedmulti has no miniscript node → no renderer
         }
         return nil
     case tagMulti:
         return nil // legacy multi renders inside combinators (valid miniscript fragment)
     ```
     (Also fix the file-header comment `:7-9` and inline `:56` to say "sortedmulti-in-combinator", not "sortedmulti/multi".)
  2. Add a regression test: `TemplateEngraveShapeGuardChunks(degrade2_11key.policy.md1.txt)` and `(...tmpl.md1.txt)` must both return nil (ADMIT), pinned to the toolkit admission proven above. Keep the existing `wsh(or_i(sortedmulti,...))` REFUSE case.

---

## Important
None.

---

## Minor

### M1 — `StripToTemplate` doc says "on a decoded clone" but mutates the decode result in place
- **Location:** `md/template_strip.go:6-7` (comment) vs `:25-50` (impl mutates the `*descriptor` returned by `Reassemble` directly: `d.tlv.pubkeys = nil`, etc.).
- **Problem:** No clone is taken. This is SAFE — `Reassemble(md1Chunks)` allocates a fresh `*descriptor` per call, so there is no caller-visible aliasing and the input `md1Chunks []string` is untouched — but the comment is inaccurate and could mislead a future maintainer into assuming an input-`*descriptor` overload would be non-mutating.
- **Fix:** Drop "on a decoded clone" (or say "on the freshly-decoded descriptor"). No behavior change.

### M2 — WDT-Id carries defensive guards the Rust source does not (intentional, but un-pinned by a test)
- **Location:** `md/template_id.go:62-71` — `errEmptyTLVEncode` when `useSitePresent && len(useSiteOverrides)==0`, and an ascending-`idx` `errOverrideOrder` check inside the override loop. Rust `compute_wallet_descriptor_template_id` (`identity.rs:79-98`) iterates the overrides as-stored with no such guards.
- **Problem:** These are reasonable defense-in-depth (a decoded descriptor's overrides are already order-validated at decode, `md.go:669`, so they should never fire on the public path), and they cannot diverge the golden (no override present in the pinned vectors). But there is no test that a present-with-overrides template still produces the byte-correct id, so the override-TLV branch of the port is only exercised structurally, not against a golden. Not blocking (the override path is rare and the byte order matches Rust on inspection).
- **Fix (optional):** add a small golden for a template carrying a `UseSitePathOverrides` entry, or note the gap in FOLLOWUPS.

---

## Nit

### N1 — `complexScriptFamily` could mislabel a depth-0 general (non-taproot) policy
- **Location:** `gui/template_engrave.go:117-122` — returns `"general miniscript"` when `tapDepth < 1`. Correct for §5-style `wsh(or_i(...))`. Fine as-is; just noting the honest-minimal label is coarse (no per-branch breakdown), which is exactly the DD7-deferred `seedhammer-template-engrave-policy-summary-display` FOLLOWUP. No action.

---

## Adversarial-focus findings (per the 8 directed items)

1. **WDT-Id (`md/template_id.go`) — PASS.** Preimage built from forward serializers `writeUseSitePath`/`writeNode`/use-site-override branch (NOT cloned from `WalletPolicyId`); NO `canonicalize`; `width = kiw(d.n)`; the `pathDecl.n != d.n` guard is INSIDE the function. Byte-for-byte faithful to Rust `compute_wallet_descriptor_template_id` (`md-codec-0.37.0/src/identity.rs:71-104`). Golden `b02b44037119e6b6fd1d82f61aa17e21` confirmed THREE ways: the Go test, AND the authoritative `md inspect md1yzpqqxppcgsc9kdmw6d5dp08f` CLI (`descriptor-mnemonic@54dd765`) printing `wallet-descriptor-template-id: b02b44037119e6b6fd1d82f61aa17e21`. Origin-invariance (`TestWalletDescriptorTemplateId_OriginInvariant`, 3 origins) and distinct-per-shape (`_Distinct`: multi≠sortedmulti, k=1≠2, N=2≠3) are real, not vacuous. `isWalletPolicy` = `pubPresent && len(pubkeys)>0` matches Rust `encode.rs:50-52` Some-AND-non-empty.

2. **Four-site form-aware mint + verify (C2) — PASS.** All four confirmed routing through the selector: `bundle/verify.go:118` (`FormAwareStubChunks`), `gui/singlesig_derive.go:68`, `gui/multisig_derive.go:43`, `md/encode_multisig.go:159` (`FormAwareStub`). The keyed/full-policy path is BYTE-IDENTICAL: `TestFormAwareStub`/`TestFormAwareStubChunks` assert `FormAwareStub(keyed) == WalletPolicyIDStub(keyed)` exactly; `EncodeMultisig`/derive always pass a keyed `d` (strip happens later on the GUI side). Foreign mk1 fails (`TestVerifyTemplateBundleForeignMk1Fails` — template md1 + full-policy mk1 → "stub mismatch"); template vs full id-spaces never cross (`TestTemplateizeBundle` asserts fullStub ≠ tmplStub; the bundle own-readback verifies).

3. **Strip golden NON-CIRCULARITY (`md/template_strip.go`) — PASS.** The goldens are INDEPENDENT toolkit output, verified by re-running `mnemonic bundle --md1-form=template` (toolkit built from `/scratch/code/shibboleth/mnemonic-toolkit`, v0.60.0) on `.examples-build/degrade2.desc` — its `md1` lines are byte-for-byte the committed `degrade2_11key.tmpl.md1.txt`. The test asserts `StripToTemplate(policy) == toolkit_template` (not a self-re-encode). §5 `degrade2_11key` KEEPS its source origins (`originElided:false`; `validateExplicitOriginRequired` passes on the strip output → origins survived). Both `pubPresent`+`pubkeys` AND `fpPresent`+`fingerprints` cleared (`template_strip.go:32-37`, asserted in `template_strip_test.go:64-69`).

4. **Task-8 guard (`md/template_guard.go`) — FAIL (see C1).** Correctly refuses `tr(sortedmulti_a)` and `sortedmulti`-in-combinator; correctly admits `tr(NUMS,multi_a)`, canonical `wsh(sortedmulti)`, and `wsh(multi)` directly-under-wsh. BUT incorrectly refuses legacy `multi`-in-combinator → over-refuses the §5 degrade2 wallet that the toolkit admits. The existing test set never probes the guard with the degrade2 fixture, so TDD missed it. This is exactly the over-refusal the focus item warned about (the focus note assumed §5 used `multi_a`-in-combinator; the actual §5 wallet uses legacy `multi`-in-combinator, which is likewise renderable and admitted — same conclusion: must NOT be refused).

5. **D1 — supply flow untouched — PASS.** `git diff main..HEAD -- gui/multisig_supply.go gui/multisig_match.go` is EMPTY. The multisig-BUILD template path binds via the device's own form-aware readback: `deriveMultisigLeg` mints the self mk1 stub with `FormAwareStubChunks(engraveMd1)` (WDT-Id for the stripped template), and `bundle.Verify`'s `checkStubBinding` is form-aware — so a built template bundle is genuinely bound/verified, not silently unverified. The BUILD path correctly SKIPS the xpub-cross-match verify OFFER for templates (a keyless template has no xpub for `findUserSlot`), but that is the cross-match leg only; the stub-binding leg still holds at engrave time. No verify gap.

6. **Wire-dialect deviation — PASS (no golden weakened).** Confirmed: the fork `split`/`Reassemble` dialect (`md1f…`, version byte 9) differs from `md encode`/`md inspect` (`md1y…`, version byte 4) — `md inspect` on a fork-dialect chunk errors "wire-format version mismatch: got 9, expected 4". This does NOT weaken any golden: the WDT-Id golden is AST-derived (the test builds the descriptor in-Go and the `md inspect` confirmation uses the y-dialect string of the SAME AST, both yielding `b02b4403…`), and the strip/string-input goldens use toolkit `split`/bundle-dialect strings (the only dialect the device ever sees — bundle/NFC, never `md encode` output). In-distribution.

7. **GUI regressions — PASS.** The default full-policy path is behaviorally unchanged: the new "Engrave wallet policy" ChoiceScreen defaults to "Full policy md1" (choice 0); `TestEngraveSingleSigFlowFull`/`WatchOnly` were updated to click the default and still reach the identical `Card 1 of 3` / `Card 1 of 2` engrave. The four stub-mint swaps are one-line `WalletPolicyID* → FormAware*` replacements that are byte-identical for keyed input. `singlesig_verify.go` only adds a `template`-gated `templateizeBundle` of the re-derived comparator baseline (correct — the readback plates are keyless for a template engrave). No change to derivation, scrub, or verify semantics for the full flows.

8. **TinyGo — PASS (static; authoritative gate is fork CI `tinygo-device-build`).** The device toolchain is unavailable in this env (as expected). Static scan of the new files (`md/template_id.go`, `md/template_strip.go`, `md/template_guard.go`, `gui/template_engrave.go`): no new generics, no reflection, no `encoding/json`/`regexp`/`text|html/template`, no load-bearing nondeterministic map iteration; imports are only `errors`, `fmt`, and existing `seedhammer.com/{md,bundle,mk}` packages (the WDT-Id sha256 goes through the pre-existing `sha256First16` helper). Clean for TinyGo on inspection; the push-time CI gate remains authoritative.

---

## What was run
- `go build ./...` → exit 0.
- `go test -count=1 ./md/... ./bundle/... ./gui/...` → ALL PASS.
- Authoritative cross-checks: `md inspect` (descriptor-mnemonic `54dd765`, `md 0.7.1`) confirmed the WDT-Id golden; `mnemonic bundle --md1-form=template` (mnemonic-toolkit v0.60.0) re-generated the degrade2 template golden byte-identically and confirmed toolkit admission of degrade2 + refusal of sortedmulti-in-combinator; read Rust `identity.rs:71-104` / `encode.rs:50-52` / `synthesize.rs:1113-1212` as the source of truth.
- A temporary Go probe was used to demonstrate C1 (`templateEngraveShapeGuard(degrade2)` → refused) and was REMOVED; `git status --porcelain` is clean (no source modified by this review).

---

**VERDICT: NOT GREEN — 1C / 0I. Blocking: C1 (`md/template_guard.go:54` — refuse `tagSortedMulti`-in-combinator ONLY, admit `tagMulti`; add the degrade2 admission regression test).**
