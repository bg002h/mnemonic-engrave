# SPEC — SeedHammer verify-correctness cluster fixes (Track A)

**Author:** single-author brainstorm spec (read-only on the fork). **Base:** fork `bg002h/seedhammer` branch `main` @ HEAD `3a23dbb` (`3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`, verified live). **Findings source:** `design/agent-reports/seedhammer-fork-own-code-bughunt.md` (H1, H2, M1, L1, L2). **Orchestration:** `design/seedhammer-own-code-fix-orchestration-plan.md` (Track A = `fix/verify-cluster`). **Date:** 2026-06-20.
**Status:** PRE-R0. **No code may be written until an opus architect R0 review converges to 0 Critical / 0 Important.** This is a spec, not a plan.

---

## 0. Scope, non-goals, and firmware-only confirmation

Track A hardens the on-device **verify-bundle** path (the operator's only post-engrave confidence check) across single-sig and multisig. It fixes **five** findings that all concentrate in `bundle/verify.go` + the two gui verify flows:

| ID | Sev | One-line | Primary site (`3a23dbb`) |
|----|-----|----------|--------------------------|
| H1 | HIGH | Multisig verify never reads back the operator mk1 plate — compares re-derived mk1 to itself → mis-engraved mk1 silently PASSES | `gui/multisig_verify.go:100` + `gui/multisig_supply.go:26` |
| H2 | HIGH | Multi-chunk md1 readback FALSE-FAILS — positional compare vs Go-map-random gather order | `gui/md1_gather.go:57-63` (`collected()`) + `bundle/verify.go:64` |
| M1 | MED | ms1 verify compares only recovered entropy, ignoring codex32 prefix/language → non-English `mnem` readback with matching entropy FALSE-PASSES | `bundle/verify.go:83-97,122-136` |
| L2 | LOW | Multisig md1/mk1 legs tautological; success copy over-claims "matches the seed" | `gui/multisig_verify.go:26-29,100-104` |
| L1 | LOW | `DecodeMS1` validity-probe discards secret entropy unscrubbed — **2 verify-flow sites only** | `gui/singlesig_verify.go:116`, `gui/multisig_verify.go:93` |

**Files Track A may edit:** `bundle/verify.go`, `gui/multisig_verify.go`, `gui/multisig_supply.go`, `gui/singlesig_verify.go`, `gui/md1_gather.go` and their `_test.go` siblings. (`bundle/verify_test.go`, `gui/multisig_verify_test.go`, `gui/md1_gather_test.go`, `gui/singlesig_verify_test.go`.)

**Out of scope / non-goals:**
- The **third L1 site** `gui/codex32_polish.go:103` belongs to **Track B** (`fix/scrub-batch`). Track A MUST NOT touch it. This split is what keeps the two tracks file-disjoint (the basis for A∥B concurrency).
- M2 / M3 / M4 (secret-scrub batch) are Track B.
- The shared helper `wipeBytes` (`gui/slip39_polish.go:344`) is *called* by Track A, *edited* by neither track. A "needs a new helper" urge is a design smell to flag at R0.
- No new crypto check for non-operator cosigner public keys (L2): an air-gapped device holds only the operator's seed and has **no independent source of truth** for other cosigners' xpubs. This is an inherent, non-fixable limitation — the L2 fix is a UX-honesty copy change, not a crypto change. Do NOT over-engineer.

**Firmware-only / surface confirmation:**
- **No `me` / CLI / `me-preview` surface.** All changed files are in the `seedhammer.com` firmware module (`bundle/`, `gui/`). The host-side Rust `me-cli` and Go `me-preview` crates are untouched. No schema, NDEF, or wire-format change — the m-format codecs (`md`/`mk`/`codex32`) are not edited.
- **No new program / no new screen.** All fixes are internal to existing flows; the only user-visible change is one success-message string (L2).
- **Secret-hygiene preserved or improved.** L1 *adds* scrubs. No fix introduces a new long-lived secret buffer. H1 reads back a **public** mk1 plate (no secret). M1 may read prefix/language ints (non-secret) alongside the already-scrubbed entropy.
- **TinyGo-safe.** No new dependencies, no reflection, no goroutines, no generics beyond what the package already uses. The H2 fix is a loop reorder; M1 returns two extra `int`s; L1 binds a return var and calls existing `wipeBytes`. All compile under the TinyGo device target (the *real* CI gate — host `go build` is not sufficient; the plan's final pass MUST run the TinyGo device build).

---

## 1. The headline risk this spec is built to defeat: TEST-MASKING

The bug hunt's central lesson: **every existing verify unit test passes synthetic/derived data as the "readback," so it stays GREEN even if H1/H2/M1 are mis-fixed.** Verified at `3a23dbb`:

- `gui/multisig_verify_test.go:30` — `verifyMultisig(derived, derived.MS1, derived.MK1, derived.MD1)` passes `derived.MK1` on the readback side. This is *exactly* the H1 bug pattern; the test cannot observe it. Its "mutated mk1 FAIL" subtest (`:35-41`) hands `verifyMultisig` a distinct `bad` arg directly, **never exercising the flow wiring** that always supplies `reDerived.MK1`.
- `bundle/verify_test.go:117 TestVerifyBundleMd1Reordered` asserts a **reordered** md1 readback FAILS — encoding the H2 bug as *intended* behaviour. This test's semantics MUST flip under the H2 fix (see §3.H2).
- `gui/singlesig_verify_test.go:14 readbackCards(b)` builds the readback `[]bundleCard` *from the derived bundle* — same chunk order, so it never trips H2 at the gather seam.

**Therefore Track A's load-bearing GREEN bar is three NEW flow-level tests**, each of which MUST:
1. **FAIL when run against `3a23dbb` (current code)** — proving it catches the live bug, and
2. **PASS after the fix.**
3. Exercise the **production extraction/gather path** (`collected()` / `extractSuppliedMd1` / `ms1Entropy`), NOT a synthetic stub that re-derives both sides.

| Test | Bug | Behaviour today (`3a23dbb`) | Behaviour after fix | Lives in |
|------|-----|------------------------------|---------------------|----------|
| **T-H2** shuffled/map-ordered multi-chunk md1 gather → `Verify` | H2 | FALSE-FAIL (PASS expected, gets mismatch) | PASS | `gui/md1_gather_test.go` |
| **T-H1** mutated/wrong engraved operator mk1 plate readback → `Verify` | H1 | FALSE-PASS (the bug) | FAIL | `gui/multisig_verify_test.go` |
| **T-M1** non-English `mnem` ms1 readback, entropy matches → `Verify` | M1 | FALSE-PASS | FAIL | `bundle/verify_test.go` |

The R0 reviewer and the post-implementation exec reviewer MUST each confirm: (a) all three tests fail on the pre-fix tree, and (b) they route through the named production functions, not a re-derived-both-sides stub.

---

## 2. Verified call-graph facts (live `file:line` @ `3a23dbb`)

These are the load-bearing facts the fixes rest on; all read directly from source.

**H1 wiring:**
- `gui/multisig_verify.go:100` — `verifyMultisig(reDerived, ms1Readback, reDerived.MK1, suppliedMd1)`. The 3rd arg (mk1) is `reDerived.MK1`, the freshly **re-derived** value — NOT an NFC readback.
- `gui/multisig_verify.go:26-28` — `verifyMultisig(derived bundle.Bundle, ms1Readback string, mk1, md1 []string)` builds `readback := bundle.Bundle{MS1: ms1Readback, MK1: mk1, MD1: md1}` and calls `bundle.Verify(derived, readback)`.
- `gui/multisig_verify.go:60` — the flow only ever calls `extractSuppliedMd1(cards)`; it never extracts an mk1 from `cards`.
- `gui/multisig_supply.go:26` — `case cardMK1, cardMS1: return nil, false` — `extractSuppliedMd1` **structurally refuses** any card set containing a `cardMK1`, so the operator mk1 plate can never be read back through this path.
- Inside `bundle.Verify`, `derived.MK1` and `readback.MK1` are *both* `reDerived.MK1` → `bundle/verify.go:52-60` (fingerprint/xpub/origin-path compare) is a tautology.
- The function's own docstring (`gui/multisig_verify.go:32-33`) *claims* the flow "gather[s] the supplied md1 + operator mk1 over NFC" — a claim the code does not honour.
- **Correct sibling to mirror:** `gui/singlesig_verify.go:23-42 singleSigReadbackCards` requires BOTH a `cardMK1` and a `cardMD1` from the gathered set; `gui/singlesig_verify.go:98,123` extracts the real readback mk1 and passes it to `verifySingleSig`. This proves H1 is a divergence/regression, not a design choice.

**H2 wiring:**
- `gui/md1_gather.go:23-28` — `md1Gatherer.set` is a `map[int]string` (key = `ChunkIndex`, set at `:51` `g.set[h.ChunkIndex] = s`).
- `gui/md1_gather.go:57-63` — `collected()` does `for _, s := range g.set { out = append(out, s) }` — ranges the Go map in **deliberately randomized** iteration order. No sort anywhere downstream.
- `md/chunk.go:140-145` — `split()` emits chunks in **index order** (`for index := 0; index < count; index++`). So `derived.MD1` is index-ordered.
- `bundle/verify.go:64,138-148` — the md1 leg is `equalStrings(derived.MD1, readback.MD1)`, a **positional** exact compare (length + element-by-element). Random readback order vs index-ordered derived → false-FAIL whenever the random order ≠ index order (≤1/6 agreement for a 3-chunk set, re-randomized each run).
- A real wpkh single-sig md1 is 3 chunks; the multisig `wsh(sortedmulti(2,3))` fixture is 6 chunks (`gui/md1_gather_test.go:14-21`) — both multi-chunk, both affected at the single-sig gather seam. (Multisig verify is unaffected by H2 *today* only because both `Verify` sides share the same `suppliedMd1` slice in the same order; the H2 fix at `collected()` is the right place regardless — see §3.H2 lock.)
- **The gather seam routes through `collected()`** at `gui/md1_gather.go:77,140` (both call sites feed `gatheredDescriptorFlow(ctx, th, g.collected())`). Note: today the *verify* flows read md1 via `bundleGatherFlow` → `bundleCard.strings` (index order, `gui/bundle.go:31-36`), NOT via this `md1Gatherer`. The H2 defect as reported is in `md1Gatherer.collected()` and its consumers; fixing `collected()` to be order-deterministic is correct and removes the latent map-order hazard at its root. (See §3.H2 for why the fix lives here and the test exercises `collected()`.)

**M1 wiring:**
- `bundle/verify.go:122-136 ms1Entropy(s)` calls `codex32.DecodeMS1(str)` (`:127`) which returns `(prefix, language, entropy, err)` but **discards prefix and language** (`_, _, entropy, err :=`), returning only the entropy copy.
- `bundle/verify.go:83-97` — `Verify` compares only `bytes.Equal(dEnt, rEnt)`.
- `codex32/mspayload.go:34-60 DecodeMS1` — payload `[0x00][entropy]` (entr, language always 0) OR `[0x02][language][entropy]` (mnem, `language` 0..9). `codex32.New` validates only length/HRP/BCH/structure — it admits a `mnem`-prefix (0x02) string with `language` 1..9.
- The BIP-39 **language byte is load-bearing**: identical entropy under a different wordlist → different mnemonic words → different PBKDF2 seed → **different wallet**. `gui/ms1_decode.go:37-44` itself treats a non-English `mnem` as a different-wordlist wallet ("Restore with a `<name>` BIP-39 wallet"), confirming this is a material wallet change, not the "incidental string difference" `bundle/verify.go:81-82` intends to tolerate.
- The device only ever *engraves* `entr`/English (`codex32.EncodeMS1`, `codex32/msencode.go:17-31`, hard-codes `msPrefixEntr`), and the derived side is always `entr`. Only the **hand-typed readback** can carry a non-English `mnem`.

**L1 wiring (2 in-scope sites):**
- `gui/singlesig_verify.go:116` — `if _, _, _, err := codex32.DecodeMS1(s); err != nil { ... }` — discards the returned secret `entropy` (`[]byte`) with `_`, no scrub.
- `gui/multisig_verify.go:93` — identical pattern, same discard.
- Correct convention to mirror: `gui/ms1_decode.go:22,29` (`_, language, entropy, err := codex32.DecodeMS1(scan); ...; defer wipeBytes(entropy)`) and `bundle/verify.go:131-134` (copy-then-`wipe`). `DecodeMS1`→`Seed()`→`parts().data()` allocates a fresh `[]byte` each call (no caching), so a real secret buffer is abandoned today.

**L2 wiring:**
- `gui/multisig_derive.go:60` — `md1 := append([]string(nil), suppliedMd1...)` (a verbatim clone of the supplied md1), placed into `reDerived.MD1`.
- `gui/multisig_verify.go:100` — the readback bundle's `MD1` is `suppliedMd1`. So inside `Verify` the md1 leg compares `clone(suppliedMd1)` vs `suppliedMd1` → can never fail. The mk1 leg is the H1 self-compare (also can never fail today).
- `gui/multisig_verify.go:104` — success copy `showNotice(ctx, th, "Verify OK", "The engraved bundle matches the seed.")` — over-claims a full-bundle guarantee.
- The genuine *real* checks in the multisig flow are `findUserSlot` (operator-as-cosigner + operator xpub re-derivation, `gui/multisig_verify.go:70`) and the ms1 entropy leg.

---

## 3. Per-finding fix design

### H1 — Read back the operator mk1 plate and compare the REAL readback

**Invariant established:** *A mis-engraved operator mk1 plate (wrong xpub/fingerprint/origin/policy-stub) makes multisig verify FAIL.* The mk1 legs of `bundle.Verify` (`:52-60`) must compare an **NFC-read-back** mk1 against the re-derived mk1 — never the re-derived value against itself.

**Fix design:**
1. Add an extraction that gathers BOTH the operator `cardMK1` and the wallet-policy `cardMD1` from the NFC card set, mirroring `singleSigReadbackCards` (`gui/singlesig_verify.go:23-42`). Options for R0 to choose between (spec recommends **(b)**):
   - (a) Relax `extractSuppliedMd1` to also return the mk1 — but its contract (and docstring, `gui/multisig_supply.go:12-17`) is specifically "exactly one md1, zero key cards"; widening it risks the supply/engrave callers (see §5 fan-out). **Not recommended.**
   - (b) **Recommended:** add a new helper in `gui/multisig_supply.go` (e.g. `extractSuppliedMd1AndMk1(cards) (md1, mk1 []string, ok bool)`) that requires exactly one md1 AND exactly one operator mk1 (reuse the `singleSigReadbackCards` shape). Leave `extractSuppliedMd1` and its other callers untouched. The verify flow calls the new helper; the operator now scans both the md1 policy card(s) and the mk1 key card(s) over NFC (`bundleGatherFlow` already yields both kinds — `gui/bundle.go:31-36`).
2. In `gui/multisig_verify.go:100`, pass the **read-back** mk1 (from step 1) as the 3rd argument: `verifyMultisig(reDerived, ms1Readback, mk1Readback, suppliedMd1)`.
3. Update the error-path copy at `gui/multisig_verify.go:62`/the new gate so the operator is told to read back BOTH cards (mirror `gui/singlesig_verify.go:100` "Need one key card (mk1) and one descriptor (md1) read back.").
4. **Dead `derived` param question:** the `derived bundle.Bundle` first param of `verifyMultisig` is NOT dead — it carries `reDerived.MK1`/`reDerived.MD1`/`reDerived.MS1` (the comparator baseline), exactly as `verifySingleSig` uses it. It must STAY. After the fix it stops being self-compared because the readback side now carries the real read-back mk1. **Resolution: keep the param; the bug was the *argument* (`reDerived.MK1`), not the parameter.** (This corrects the bug-hunt's tentative "drop the dead param" suggestion — verified against `verifySingleSig`'s identical signature usage.)
5. Fix the docstring at `gui/multisig_verify.go:31-35` so it matches the now-true behaviour (gathers md1 **and** operator mk1 over NFC).

**Acceptance test (T-H1), `gui/multisig_verify_test.go`:**
- Derive the correct multisig leg (`deriveMultisigLeg`, as the existing test does at `:24`).
- Build a `[]bundleCard` readback set via a helper (model on `singlesig_verify_test.go:14 readbackCards`) containing the correct md1 card(s) AND an operator **mk1 card whose chunk strings are MUTATED** (flip a char in a chunk, as `multisig_verify_test.go:37` does for the synthetic case — but here put it in the `cardMK1` of the gathered set).
- Run through the **production extraction** (`extractSuppliedMd1AndMk1`, or whatever the new helper is named) → `verifyMultisig`.
- **Assert FAIL.** On `3a23dbb` this test cannot even be written against the current flow (there is no mk1 extraction); written against the fixed helper it must FAIL on a mutated mk1, and a companion "correct readback → PASS" sub-test must PASS. The masking proof: a sibling assertion that feeding the *re-derived* mk1 (matching) PASSES, and the mutated one FAILS — the discrimination the current flow lacks.
- A complementary direct check: assert the production flow extracts the mk1 from the card set at all (today `extractSuppliedMd1` returns `false` on any `cardMK1`, so a card set with an mk1 is rejected — the new helper must accept it).

---

### H2 — Make `collected()` return chunks in `ChunkIndex` order [LOCKED LOCATION]

**LOCKED FIX LOCATION: `gui/md1_gather.go:collected()`** — NOT `bundle/verify.go`. **Rationale (load-bearing, must hold through plan-R0):** a `[]string` in `bundle/verify.go` carries no chunk index, so making `equalStrings` order-tolerant there would force a signature change (pass indices, or sort by re-parsing each chunk header inside `bundle`) that ripples into every `Verify` consumer and re-introduces parsing into the deterministic comparator core. `collected()` already owns the indexed map (`set map[int]string`) and is the single producer of the unordered slice — fixing it at the source makes EVERY consumer (verify, inspect-descriptor display, `gatheredDescriptorFlow`) deterministic with zero signature churn. The comparator stays a pure positional compare, which is correct once both sides are canonically ordered.

**Invariant established:** *`md1Gatherer.collected()` returns chunk strings in ascending `ChunkIndex` order (0..total-1), deterministically, regardless of scan/arrival order.* Consequently a correctly-engraved multi-chunk md1, scanned in any order, compares equal to the index-ordered derived md1 and verify PASSES.

**Fix design:** in `gui/md1_gather.go:57-63`, replace the map-range with an index-ordered walk:
```
func (g *md1Gatherer) collected() []string {
    out := make([]string, 0, len(g.set))
    for i := 0; i < g.total; i++ {
        out = append(out, g.set[i])
    }
    return out
}
```
- `collected()` is only ever called when `complete()` is true (`gui/md1_gather.go:76,139` guard with `if g.complete()`), and `complete()` requires `len(g.set) == g.total` with all indices `0..total-1` present (each `offer` keys by `h.ChunkIndex`). So every index lookup is populated — no zero-value `""` gaps. R0 should confirm there is no path that calls `collected()` before `complete()`. (If R0 finds one, fall back to: collect present keys, sort ascending — but the index walk is preferred for clarity given the `complete()` precondition.)

**Acceptance test (T-H2), `gui/md1_gather_test.go`:**
- Use the existing 6-chunk `wshSortedmultiChunks` fixture (`:14-21`).
- `offer()` the chunks in a **deliberately shuffled / reverse / rotated** order into a fresh `md1Gatherer` (the gatherer keys by parsed `ChunkIndex`, so arrival order is what we vary).
- Assert `collected()` returns the chunks in **index order** == the canonical `wshSortedmultiChunks` slice (which is index-ordered). Today (`3a23dbb`) this assertion FAILS for most shuffles (map-random order ≠ index order); after the fix it PASSES for every arrival order.
- Strengthen against the existing `TestMD1Gatherer` (`:84-86`) which only checks `len(collected())` — the new test asserts **element-wise equality in index order**, which is the discriminating assertion.
- **End-to-end flavour (recommended):** also assert that a correct multi-chunk md1, gathered shuffled, then run through `Verify` against an index-ordered derived md1, **PASSES** — exercising `collected()` → comparator, not just `collected()` in isolation.

**Existing-test semantics flip (MANDATORY, must be called out at R0):** `bundle/verify_test.go:117 TestVerifyBundleMd1Reordered` currently asserts a *reordered* md1 readback FAILS — it encodes the H2 bug as intended. **This is a comparator-level test, not a gather-level test:** `bundle.Verify` still does a positional compare and remains *correct* to reject an out-of-order `[]string` (the contract is "both sides are canonically ordered"). So `TestVerifyBundleMd1Reordered` can legitimately STAY as a comparator-contract test, BUT its name/comment must be reframed to "the comparator is positional by contract; canonical ordering is the gather layer's responsibility (`collected()`)," cross-referencing T-H2. R0 to confirm we are not deleting a real guarantee — we are relocating the ordering responsibility to `collected()` and documenting the comparator's positional contract. **Do not weaken `bundle.Verify` to sort internally** (that would re-introduce parsing into the comparator and contradict the H2 lock).

---

### M1 — Compare codex32 prefix + language, not just entropy

**Invariant established:** *A readback ms1 whose recovered entropy matches the derived ms1 but whose codex32 prefix or BIP-39 language byte differs makes verify FAIL.* Identical entropy under a different wordlist is a different wallet and must not PASS.

**Fix design — choose ONE at R0 (spec recommends (a)):**
- **(a) Recommended: compare prefix + language in `Verify`.** Have `ms1Entropy` return `(prefix, language int, entropy []byte, err error)` (it already gets all three from `DecodeMS1` and discards two). In `Verify` (`bundle/verify.go:83-97`), after decoding both sides, compare `dPrefix == rPrefix && dLang == rLang` and error with a clear `"verify: ms1 wordlist/language mismatch"` if they differ, in addition to the entropy compare. Keep the entropy scrub (`wipe(dEnt)`/`wipe(rEnt)`) on every path. This is the most faithful: it verifies the readback recovers the **same wallet** (same words), not merely the same entropy.
- **(b) Alternative: reject non-`entr`/non-language-0 ms1 at the readback gate.** Since the device only ever engraves `entr`/English, gate the hand-typed readback at `gui/singlesig_verify.go:116` and `gui/multisig_verify.go:93`: after `DecodeMS1`, if `prefix != msPrefixEntr` (or `language != 0`), show "That isn't an English ms1 secret share." and refuse. Simpler, but `msPrefixEntr`/the prefix consts are package-private to `codex32` (`codex32/mspayload.go:9-11`) — the gate would need an exported predicate or to compare `language != 0` only (English-`mnem` would still pass, which is acceptable since English-`mnem` recovers the same words as `entr`). **Risk:** this duplicates the gate across two flow sites and leaves `bundle.Verify` still entropy-only (defence is at the gate, not the comparator), so a future caller of `Verify` is unprotected. **(a) is preferred** because it hardens the comparator itself.

**Rationale for (a):** the comparator is the single deterministic core (`bundle/verify.go:4`). Putting the language check there protects every present and future caller; the prefix/language are already in hand at `ms1Entropy`. The two extra `int` returns are non-secret. TinyGo-safe (no new types).

**Acceptance test (T-M1), `bundle/verify_test.go`:**
- Derived side: the correct `entr`/English bundle (`correctBundle()`), entropy E (the zero-16 vector).
- Readback side: construct a **non-English `mnem`** ms1 with the **same entropy E** but `language = 1` (Japanese). `EncodeMS1` only emits `entr`, so build it directly via `codex32.NewSeed("ms", 0, "entr", 's', append([]byte{0x02, 0x01}, E...))` (the `0x02` mnem prefix + `0x01` language byte + the 16-byte entropy) — confirmed reachable: `codex32.NewSeed` is exported (`codex32/codex32.go:279`), `DecodeMS1` admits `mnem`/language 1..9 (`mspayload.go:42-50`), and `gui/ms1_decode.go:37-44` confirms the firmware itself treats this as a different-wallet readback. Set `readback.MS1` to that string.
- **Assert FAIL** (the error names ms1 language/wordlist). On `3a23dbb` `bytes.Equal(E,E)` is true so `Verify` PASSES (the bug); after fix it FAILS.
- Companion PASS test: an English-`entr` readback with the same entropy still PASSES (don't over-reject English/`entr`). Optionally an English-`mnem` (prefix 0x02, language 0) readback with same entropy — under (a) this FAILS on prefix mismatch unless we treat language-0 as wordlist-equivalent; **R0 decision:** since English-`mnem` and `entr` recover identical words, prefer comparing on **language only** (both language 0) rather than raw prefix, so legitimate English readbacks aren't rejected on an incidental prefix difference. Lock this nuance at R0.

---

### L2 — Soften the over-claiming success copy (no crypto change)

**Invariant established (honest scoping):** *The multisig "Verify OK" message must not claim a guarantee the air-gapped device cannot provide.* After H1, the operator-key (mk1) leg becomes a real readback compare and the ms1 entropy/language leg (M1) is real; but the md1 leg remains a `clone(suppliedMd1)` vs `suppliedMd1` tautology (`gui/multisig_derive.go:60`), and non-operator cosigner public keys are taken as given (no source of truth). The honest guarantee is: *operator key + operator xpub/origin + the secret are verified; the wallet policy and other cosigners' public keys are taken as supplied.*

**Fix design:**
- Determine from source what CAN be cross-checked on the md1 leg: `findUserSlot` (`gui/multisig_verify.go:70`) already re-derives and matches the **operator's** xpub against the supplied policy, and `checkStubBinding` (`bundle/verify.go:103-118`) verifies the operator mk1 binds to the supplied md1's policy id. So the operator's slot IS cross-checked; the gap is the *other* cosigners' xpubs and the verbatim md1 string for non-operator data. This gap is **inherent** (no source of truth on an air-gapped device) — do NOT add a crypto check.
- **Change only the copy at `gui/multisig_verify.go:104`** to scope the guarantee honestly, e.g.: `"Verify OK"` / `"Operator key + secret verified. Other cosigners' keys are taken as supplied."` (final wording at R0). Optionally add a one-line code comment documenting the inherent air-gapped limitation.
- After the H1 fix, do NOT additionally claim the md1 leg is independently verified — it is a clone tautology by construction (the supplied md1 IS the input). The honest framing is "operator key + secret verified."

**Acceptance:** No new flow-level pass/fail test is mandated for L2 (it is a copy/UX change, not a logic bug). A light test asserting the multisig success notice text contains the scoped wording (e.g. `uiContains(content, "taken as supplied")` and does NOT contain the bare over-claim) is OPTIONAL and may be added if cheap, modelled on the `uiContains` assertions in `gui/md1_gather_test.go:118,164`. R0 to decide whether to require it. L2 must not regress any H1/M1 assertion.

---

### L1 — Scrub the `DecodeMS1` validity-probe entropy (2 verify-flow sites)

**Invariant established:** *The ms1 validity probe in each verify flow does not abandon an unscrubbed secret entropy buffer* — matching the codebase's own convention (`gui/ms1_decode.go:29`, `bundle/verify.go:131-134`).

**Fix design (both sites identical):** at `gui/singlesig_verify.go:116` and `gui/multisig_verify.go:93`, capture the entropy return and wipe it:
```
if _, _, ent, err := codex32.DecodeMS1(s); err != nil {
    showError(...)
    return
} else {
    wipeBytes(ent)
}
```
or bind before the `if` and `defer wipeBytes(ent)`. Use the existing `wipeBytes` (`gui/slip39_polish.go:344`) — do NOT add a new helper. Note (from the hunt): this is a defence-in-depth consistency fix — the same secret already lives longer as the immutable `codex32.String`/`ms1Readback` Go string, so this is LOW; still worth doing for convention. **Track A owns ONLY these two sites.** The third site `gui/codex32_polish.go:103` is Track B's — do not touch.

**Acceptance:** No dedicated flow test (best-effort scrub is not observable post-GC and TinyGo may retain regardless). A static/review assertion suffices: the exec reviewer confirms both sites now capture-and-`wipeBytes` the probe entropy. Optionally a test that the probe path is reached (already covered by the full-verify flow tests). R0 to confirm no test is needed.

---

## 4. Cross-cutting invariants & interaction

- **Single implementer, TDD, one worktree** (`fix/verify-cluster`), strictly serial within the track (no parallel re-implementations). Tests authored before impl per finding; reviewer loop to 0C/0I after every fold.
- **H1 + L2 co-located in `gui/multisig_verify.go`** — design and implement together (L2's honest copy depends on H1 making the mk1 leg real). H2 + M1 both touch `bundle/verify.go`'s legs (M1) and `md1_gather.go` (H2) — co-located edits, one implementer.
- **No interaction with Track B:** Track B only adds zeroing of already-discarded buffers in disjoint files (`slip39/`, `seedxor/`, `gui/bip85.go`, `gui/codex32_polish.go`); no signature/return/control-flow change any Track-A test observes. A∥B concurrency holds.
- **Order of comparison legs in `bundle.Verify` is preserved** — H2/M1 do not reorder the existing stub→mk1→md1→ms1 sequence; they make the md1 leg's *inputs* canonical (H2, at the gather layer) and the ms1 leg stricter (M1, language compare added after the entropy compare).

---

## 5. Scope & caller fan-out (must be checked by the implementer + exec reviewer)

| Changed function | Callers to verify unaffected (verified @ `3a23dbb`) |
|------------------|------------------------------------------------------|
| `ms1Entropy` (signature grows 2 returns, M1) | Only `bundle/verify.go:83,87` (both inside `Verify`). No other caller — `grep` confirms it is package-private and called only twice. Update both call sites. |
| `collected()` (body only, H2 — **no signature change**) | `gui/md1_gather.go:77` and `:140` (both `gatheredDescriptorFlow(ctx, th, g.collected())`). Behaviour change is order-only; `gatheredDescriptorFlow`→`ExpandWalletPolicyChunks`→`Reassemble` is order-tolerant, so no caller breaks; the verify path strictly improves. Confirm no third caller. |
| `extractSuppliedMd1` (UNCHANGED under recommended option (b)) | Only caller is `gui/multisig_verify.go:60`. If option (a) is chosen instead, re-audit: `extractSuppliedMd1` is referenced only there (no engrave/supply caller) — verify via `grep` before changing its contract. |
| new `extractSuppliedMd1AndMk1` (H1, option (b)) | New function; sole caller is the multisig verify flow. Model its multi-card acceptance on `singleSigReadbackCards` (`gui/singlesig_verify.go:23-42`). |
| `verifyMultisig` (param kept; argument changed, H1) | Only caller is `gui/multisig_verify.go:100`. The test `gui/multisig_verify_test.go:30,38,46,55,65` calls it directly with explicit args — those stay valid; ADD the flow-level T-H1 alongside. |
| success copy `gui/multisig_verify.go:104` (L2) | No callers; string-only. |
| `DecodeMS1` probe sites (L1) | Local to each flow; no signature change. |

**Mandatory `grep` checks for the implementer (pre-impl):** `grep -rn "ms1Entropy\|\.collected()\|extractSuppliedMd1\|verifyMultisig" --include=*.go` to confirm the fan-out table is exhaustive at implement time (citations decay).

---

## 6. Risks

1. **Test-masking recurrence (highest):** if the new tests re-derive both sides or stub the gather, they regress to GREEN-on-broken. Mitigation: the three flow-level tests MUST route through `collected()` / `extractSuppliedMd1AndMk1` / `ms1Entropy` and MUST be demonstrated to FAIL on `3a23dbb` first. The exec reviewer verifies both.
2. **H2 fix-location drift:** an implementer "fixing" H2 in `bundle/verify.go` (sorting inside the comparator) would re-introduce parsing into the deterministic core and force a signature change. The lock in §3.H2 is binding through plan-R0.
3. **`TestVerifyBundleMd1Reordered` mishandling:** deleting it (losing the positional-contract guarantee) OR leaving its comment claiming reorder-rejection is the intended *product* behaviour. Resolution: keep it as a relabelled comparator-contract test that cross-references T-H2. R0 to confirm.
4. **M1 over-rejection:** comparing raw `prefix` (not `language`) could reject a legitimate English-`mnem` readback (prefix 0x02, language 0) that recovers identical words. Mitigation: compare on `language` (and treat language-0 `mnem`≡`entr`). Locked at R0.
5. **`collected()` pre-`complete()` call:** the index walk assumes all `0..total-1` present. If any caller invokes `collected()` before `complete()`, an index gap yields `""`. Verified today both callers guard with `complete()`; R0 to re-confirm, else fall back to sort-present-keys.
6. **Widening `extractSuppliedMd1` (if option (a) chosen):** could pollute the one-md1/zero-mk1 contract used by the supply flow. Mitigation: option (b) (new helper) avoids touching the existing contract — recommended.
7. **TinyGo gate:** all changes must compile on the device target; the plan's final pass runs the TinyGo device build, not just host `go build`/`go test`/`go vet`.
8. **Agent-API dispatch failure** (the 529 class that twice aborted the hunt): if R0/exec dispatch fails mid-session, flag explicitly and defer the formal gate to API recovery — never silently substitute inline self-review.

---

## 7. R0 gate (mandatory, before any plan or code)

This spec MUST pass an opus architect R0 review to **0 Critical / 0 Important** before an implementation plan is authored. Fold findings → persist the review verbatim to `design/agent-reports/` → re-dispatch after every fold until GREEN. No code before GREEN. Open R0 questions for the architect:
- **Q1 (H1):** confirm option (b) (new `extractSuppliedMd1AndMk1`, leave `extractSuppliedMd1` intact) over option (a); confirm the `derived` param stays (it is the comparator baseline, not dead).
- **Q2 (M1):** confirm option (a) (language compare in `Verify`) over (b) (gate-level reject); confirm compare on **language** (language-0 `mnem`≡`entr`) not raw prefix, to avoid over-rejecting English readbacks.
- **Q3 (H2):** confirm the index-walk `collected()` (vs sort-present-keys) given the `complete()` precondition; confirm `TestVerifyBundleMd1Reordered` is relabelled (kept), not deleted or inverted.
- **Q4 (L2):** approve the final scoped success-message wording; decide whether the optional UI-copy test is required.
- **Q5 (L1):** confirm no dedicated test is required for the two scrub sites (review-assertion only).
- **Q6:** confirm the §1 test-masking bar — all three flow-level tests demonstrably FAIL on `3a23dbb` and route the production functions — is the GREEN bar the exec reviewer must check.
