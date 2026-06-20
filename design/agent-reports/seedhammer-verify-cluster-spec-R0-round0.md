# Track A (verify-cluster) SPEC — R0 review (round 0) — VERBATIM agent report

**Agent:** `a886ceabadd2c1c03` (adversarial opus architect; proved all 3 flow-level tests fail-before/pass-after by direct probe in a throwaway worktree off `3a23dbb`). **Fork HEAD:** `3a23dbb`. **Spec commit:** `803296a`. **Date:** 2026-06-20.
**Verdict:** GREEN (0 Critical / 0 Important). 2 Minors + 2 source-citation corrections to fold into the plan (none change a fix design or invariant; the corrections reinforce the chosen designs). Cleared for the plan phase (own plan-R0 follows). Part of the 8-finding fix; concurrent with Track B.

---

# R0 GATE REVIEW — SPEC_seedhammer_verify_cluster_fixes (Track A)

**Reviewer:** adversarial opus architect (R0 gate)
**Fork HEAD:** `3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082` (`main`, verified live)
**Spec commit:** `803296a`
**Verdict:** **GREEN — 0 Critical / 0 Important.** Two Minor items + a small set of source-citation corrections to fold (none blocking; none change a fix design or invariant).

---

## Per-mandate findings (with probe evidence)

### Mandate 1 — Every cited fact verified at `3a23dbb`

**H1 — VERIFIED.** `gui/multisig_verify.go:100` passes `reDerived.MK1` as the 3rd (mk1) arg — the re-derived value, not a readback. `gui/multisig_verify.go:60` calls only `extractSuppliedMd1(cards)`. `gui/multisig_supply.go:26` is `case cardMK1, cardMS1: return nil, false` — structurally refuses any mk1, so the operator plate is never read back. The `derived` param IS the comparator baseline (carries `reDerived.MK1/.MD1/.MS1` into `bundle.Verify`), used exactly as `verifySingleSig` (`gui/singlesig_verify.go:49-57`) uses it — not dead; the bug is the *argument*, confirming the spec's §3.H1.4 correction of the bug-hunt's "drop the dead param" suggestion. The correct sibling `singleSigReadbackCards` (`gui/singlesig_verify.go:23-42`) requires both cards — H1 is a genuine divergence/regression.

**H2 — VERIFIED.** `md1Gatherer.collected()` (`gui/md1_gather.go:57-63`) ranges the Go map `g.set`; `md/chunk.go:145` `split()` emits index order; `bundle/verify.go:64,138-148` is positional `equalStrings`. Probe (5 runs × 4 permutations): `collected()` produced non-index order on all 20 trials — fail-before is real and deterministic-in-aggregate.

**M1 — VERIFIED.** `bundle/verify.go:127` discards prefix+language (`_, _, entropy, err`); `:92` compares `bytes.Equal` only. `codex32/mspayload.go:34-60` admits `mnem` (0x02) with language 1..9. `gui/ms1_decode.go` treats non-English `mnem` as a different-wordlist wallet. BIP-39 language byte is load-bearing (different wordlist → different words → different seed). Confirmed against source.

**L2 — VERIFIED.** `gui/multisig_derive.go:60` `md1 := append([]string(nil), suppliedMd1...)` (clone tautology). `findUserSlot` (`gui/multisig_match.go:34-70`) genuinely re-derives + `bytes.Equal`-matches the operator xpub — operator slot IS cross-checked; foreign cosigner xpubs have no source of truth (inherent limit). Success copy at `:104` over-claims. Copy-only fix is the correct, non-over-engineered call.

**L1 — VERIFIED.** `gui/singlesig_verify.go:116` and `gui/multisig_verify.go:93` both `_, _, _, err := codex32.DecodeMS1(...)` discard entropy unscrubbed. `gui/codex32_polish.go:103` (Track B, out of scope) confirmed disjoint. `wipeBytes` (`gui/slip39_polish.go:344`) is a plain zeroing loop; convention `ms1_decode.go:29 defer wipeBytes(entropy)`.

### Mandate 2 — The three flow-level tests are LOAD-BEARING (core of the gate)

All three proven fail-before / pass-after, routing the production functions — by direct probe in the throwaway worktree:

- **T-M1:** `codex32.NewSeed("ms",0,"entr",'s',[]byte{0x02,0x01,<zero16>})` → on `3a23dbb`, `Verify` PASSED (the bug); after applying the spec's option-(a) language-compare it FAILED with `"verify: ms1 wordlist/language mismatch"`. All 10 existing `bundle` tests still pass. Confirmed the fixture is a genuine entropy-identical/language-differ (0 vs 1) case. Routes through production `Verify`/`ms1Entropy`.
- **T-H2:** shuffled gather → `md1Gatherer.collected()` returned non-index order on every probe run (fail-before); after the index-walk fix it returned index order deterministically across 10×4 trials, and the full gui suite passed. Routes through production `collected()`.
- **T-H1:** built a readback `[]bundleCard` with a mutated operator mk1, routed through a production `extractSuppliedMd1AndMk1` + `verifyMultisig`. Correct mk1 → PASS; undecodable mk1 → FAIL (decode leg); decodable-but-wrong mk1 → FAIL via the stub-binding leg (`"verify: readback mk1/md1 stub mismatch"`) — the real discrimination. Reproduced the bug: passing `reDerived.MK1` (today's self-compare) PASSES the mutated plate. Routes the production extraction, not a re-derive-both-sides stub.

**`TestVerifyBundleMd1Reordered` handling — CORRECT.** Confirmed it currently asserts reordered md1 FAILS (encodes the H2 behavior). Under the H2 fix at `collected()`, `bundle.Verify` stays positional-by-contract; the test still passes unchanged (verified). The spec's plan to relabel/keep as a comparator-contract test cross-referencing T-H2 (not delete, not invert) is correct.

### Mandate 3 — Fix designs + open questions Q1–Q6

- **Q1 (H1, option b + keep param): APPROVED.** New `extractSuppliedMd1AndMk1` modeled on `singleSigReadbackCards`; keep the `derived` param (comparator baseline). Probes show it works and the mk1 leg is now real. Additionally strengthened by a fan-out correction (below).
- **Q2 (M1, option a, compare LANGUAGE not raw prefix): APPROVED.** Probe confirmed an English-`mnem` (prefix 0x02, language 0) readback against `entr` derived PASSES under language-compare (no over-rejection), whereas raw-prefix compare would have falsely rejected it. Treat language-0 `mnem`≡`entr`. Defensible against codex32 source.
- **Q3 (H2 index-walk; relabel `TestVerifyBundleMd1Reordered`): APPROVED.** All `md1Gatherer.collected()` call sites guard with `complete()` (`md1_gather.go:76`, `:140`, and `bundle.go:231-234`), so the index-walk has no zero-value gaps. Index-walk preferred over sort-present-keys.
- **Q4 (L2 copy): APPROVED in principle.** Honest scoping ("operator key + secret verified; other cosigners' keys taken as supplied") is correct given `findUserSlot`'s real operator-xpub check. Optional UI-copy test: recommend requiring it (cheap `uiContains` assertion) so the over-claim can't silently regress — not blocking.
- **Q5 (L1, review-assertion only, no test): APPROVED.** Best-effort scrub is unobservable post-GC; review-assertion suffices.
- **Q6 (§1 GREEN bar): CONFIRMED** as the exec reviewer's bar — all three tests demonstrably fail-before on `3a23dbb` and route the named production functions (proven above).

### Mandate 4 — Scope / caller fan-out

- **Firmware-only: CONFIRMED.** All edits in `bundle/`+`gui/`; no `me`/CLI/`me-preview`/schema/NDEF/codec surface. No new program/screen. Track A files have no build tags (platform-neutral pure Go).
- **`wipeBytes` called, not edited: CONFIRMED.** No new-helper need.
- **Track B's `codex32_polish.go:103` NOT touched: CONFIRMED disjoint.**
- **TinyGo-safe: plausibly yes** (two extra `int` returns, an index loop, existing `wipeBytes`; no reflection/goroutines/generics added). Could not run the actual TinyGo device build (no tinygo in this env); the spec correctly defers that to the plan's final pass. Not an R0 blocker.

### Mandate 5 — No false protocol/crypto claim

codex32 prefix/language semantics, chunk index-ordering (`split`), `mk.Decode` order-tolerance (reassembles by index — so H1's multi-chunk mk1 readback is not subject to an H2-class false-FAIL), and BIP-39 language load-bearingness all verified against source. No false claim found.

---

## Critical
None.

## Important
None.

## Minor (fold before authoring the plan — none block the GREEN verdict)

1. **Caller fan-out table (§5) is factually wrong in two rows — correct it so the plan's grep is trustworthy:**
   - **`extractSuppliedMd1` has TWO callers, not one.** Besides `gui/multisig_verify.go:60`, it is called at **`gui/multisig.go:71`** (`supplyMultisigPolicyFlow`, the engrave/supply flow). The spec's §5 "Only caller is `gui/multisig_verify.go:60`" and §3.H1 option-(a) rationale ("no engrave/supply caller") are false. This does not change the ruling — it strengthens the case for option (b) (a new helper), since widening `extractSuppliedMd1` would now demonstrably affect the live engrave flow. Fold the corrected fact.
   - **`md1Gatherer.collected()` has THREE call sites, not two.** Besides `md1_gather.go:77,140`, it is called at **`gui/bundle.go:234`** (`offerChunkedMD1`, where `sub` is a `*md1Gatherer`). The spec's §2 hedge — "verify flows read md1 via `bundleGatherFlow` → `bundleCard.strings` … NOT via this `md1Gatherer`" — is incorrect: a multi-chunk md1's `bundleCard.strings` is populated from `sub.collected()` at `bundle.go:234`, so the verify-flow readback DOES route through `md1Gatherer.collected()`. Good news (the H2 fix at `collected()` fixes the real verify path, not just inspect), but the claim must be corrected and `bundle.go:234` added to the fan-out. The full gui suite passed under the H2 fix, so the third consumer is unaffected by the order change.

2. **T-H1 should assert the decodable-but-wrong discrimination, not only an undecodable mutation.** A char-flipped chunk often yields an *undecode* (caught at the mk1-decode leg) rather than exercising the fingerprint/xpub/origin/stub compares (`verify.go:52-60`). The spec's §3.H1 test sketch (flip a char) is acceptable but the plan should additionally include a decodable-but-wrong mk1 case (e.g. a valid foreign card) — proven here to FAIL via the stub-binding leg. This makes the masking-proof airtight.

---

## Verified-correct list

- H1/H2/M1/L1/L2 every cited `file:line` accurate at `3a23dbb` (except the two §5/§2 fan-out misstatements above).
- The three flow-level tests are load-bearing: fail-before on `3a23dbb`, pass-after, route production functions (probed directly).
- `TestVerifyMultisig` masking pattern (`:30` self-feeds `derived.MK1`; `:35-41` hands `verifyMultisig` a `bad` arg directly, bypassing flow wiring) and `TestVerifyBundleMd1Reordered` (encodes H2 as intended) confirmed exactly as the spec describes.
- Q1–Q6 rulings all defensible against source; the locked decisions (H2 fix at `collected()`, M1 language-compare, H1 option-b new helper + keep param) are correct.
- `mk.Decode` order-tolerance ⇒ H1's multi-chunk mk1 readback is safe from H2-class false-FAILs.
- All three `collected()` call sites guard with `complete()` ⇒ index-walk has no gaps.
- Firmware-only / disjoint from Track B (`codex32_polish.go:103`) / `wipeBytes` not edited / no new program — all confirmed.
- Full `go test ./...` green on pristine `3a23dbb`; full `gui` + `bundle` suites green under each fix applied in isolation.

---

## Bottom line

The spec is technically sound and converges to 0C/0I. All five findings are real at `3a23dbb`, the three flow-level tests genuinely catch the live bugs through the production paths (proven by probe, not assertion), and every locked design decision (Q1–Q6) is correct against source. The two Minor source-citation errors (under-counted callers for `extractSuppliedMd1` and `collected()`) do not change any fix design — if anything they reinforce option (b) and confirm the H2 fix location reaches the real verify path — but they must be folded so the plan's mandatory pre-impl grep starts from accurate facts. Fold the two Minor corrections + the T-H1 decodable-wrong companion, then proceed to plan authoring.

**Verdict: GREEN (0 Critical / 0 Important).** Minor folds recommended; not gating.

**Fork hygiene:** confirmed left clean on `main @ 3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`; throwaway worktree `/tmp/r0-trackA` removed and pruned; pre-existing `seedhammer-wt-bip39` worktree untouched.
