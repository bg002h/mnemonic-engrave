# SeedHammer fork — own-code adversarial bug hunt (VERBATIM synthesis report)

**Workflow:** `wf_f304fe52-1b8` (13-finder full-parallel fan-out → refute-by-default verify → synthesis); 22 agents, ~1.6M tokens.
**Scope:** OUR fork additions vs upstream merge-base `86a58ab` (fork `main` @ `3a23dbb`) — the m-format codecs (`md`/`mk`/`codex32`/`slip39`/`seedxor`/`bundle`) + the `gui` flow layer + `bip85`. NOT the inherited engraving subsystem (that was the prior `seedhammer-engrave-bughunt.md`).
**Date:** 2026-06-20.
**Result:** 8 candidates → **8 confirmed**, 0 uncertain; full 13-subsystem coverage (no finder failures). Severity: **2 High / 4 Medium / 2 Low.**
**Method note:** two earlier launches were aborted by transient Anthropic API overload/rate-limiting (529s); this run completed at full parallelism once the API recovered. Each confirmed finding survived a refute-by-default verifier; protocol/crypto facts checked against authoritative source.

---

All load-bearing facts confirmed against HEAD `3a23dbb`:

- **GSD-1 (single-sig false-FAIL):** `md1Gatherer.collected()` ranges the Go map `g.set` (random order); `chunk.go:145` builds chunks in index order; `equalStrings` is positional. Both halves verified — confirmed HIGH, fails-closed.
- **MSV-1 (multisig mk1 self-compare):** call site passes `reDerived.MK1` (not a readback) and `verifyMultisig`'s `derived` param is used but mk1 is identical on both sides; `extractSuppliedMd1` refuses any `cardMK1`; single-sig sibling correctly reads both cards via `singleSigReadbackCards`. Confirmed HIGH, conditional on a coincident mis-engraving.

Both HIGHs survive the adversarial pass. Now writing the report.

---

# Bug-Hunt Report — SeedHammer Fork (m-format / T-series / bip85 divergence)

## 1. Scope & Method

- **Target:** OUR-OWN additions to the SeedHammer II firmware fork — the m-format / T-series / bip85 code. Scope = our divergence from upstream merge-base `86a58ab`; fork main @ **`3a23dbb`** (verified live: `3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`). **READ-ONLY** audit.
- **Counts:** 8 candidates → **8 confirmed**, **0 uncertain**, 0 refuted.
- **Coverage:** **COMPLETE** — all 13 finder subsystems produced results; no failed/aborted finder runs.
- **Method:** 13-finder batched adversarial fan-out → **refute-by-default** verification (each finding re-checked against actual source; external protocol facts — BIP-39 seed/wordlist semantics, SLIP-0039 group-share structure, secp256k1 key types, Go map-iteration semantics — verified against authoritative source text, not the draft) → synthesis with a final adversarial sanity pass on every HIGH.
- **Severity calibration:** air-gapped device, no network exfil path. The escalators are reachable panic / wrong-output / NFC-secret transmission; memory-hygiene scrub gaps are bounded to medium/low because TinyGo's GC may copy/retain regardless (the codebase's own `wipeBytes` is documented best-effort). Verify-integrity holes that fail-closed or require a coincident operator fault are bounded to high (not critical).

---

## 2. Confirmed Findings (deduplicated, ranked severity → severity)

After dedup, **8 distinct root causes** remain. Two findings touch `gui/multisig_verify.go` but are genuinely **distinct root causes** (one is the mk1 self-compare / never-read-back; the other is the md1+mk1 tautology-with-over-claiming-copy), so both are kept. The four `secret-scrub`/intermediate-wipe findings live in four different packages with independent root causes and are kept separate.

### HIGH

---

#### H1 — Multisig verify-bundle never reads back the operator mk1 plate (compares re-derived mk1 against itself)
- **Severity:** HIGH · **Class:** verify-hole · **File:** `gui/multisig_verify.go:100` (with `gui/multisig_supply.go:18-34`, `bundle/verify.go:52-60`)
- **Finder:** gui-multisig (`MSV-1`)
- **Description:** `multisigVerifyFlow` re-types the seed, re-derives the operator leg into `reDerived`, reads back **only the md1** over NFC (`extractSuppliedMd1` structurally **refuses** if any `cardMK1` is present), then calls `verifyMultisig(reDerived, ms1Readback, reDerived.MK1, suppliedMd1)`. The mk1 argument is `reDerived.MK1` — the freshly re-derived value, **not** an NFC read-back of the engraved plate. Inside `bundle.Verify` both `derived.MK1` and `readback.MK1` are `reDerived.MK1`, so the fingerprint/xpub/origin-path comparison (`verify.go:52-60`) is a tautology. The operator's own key card — the single most restore-critical public plate — is compared to itself.
- **Trigger:** Complete a "Supply policy" or "Build policy" multisig engrave → "Verify now" → re-type seed → scan back md1 (only card the flow accepts) → hand-type ms1 (full). If the engraved **mk1** plate has any transcription/engraving error (wrong xpub char, wrong fingerprint, wrong policy stub), the flow still shows "Verify OK — The engraved bundle matches the seed."
- **Verifier reasoning/repro:** Confirmed at HEAD: call site passes `reDerived.MK1` on the readback side (verified live); `extractSuppliedMd1` returns `ok=false` on any `cardMK1` (verified — `case cardMK1, cardMS1: return nil, false`), so reading the mk1 back is structurally impossible. The docstring even *claims* the flow "gather[s] the supplied md1 + operator mk1 over NFC" — which it does not. The single-sig sibling does it correctly (`singleSigReadbackCards` requires both mk1 and md1 from NFC; passes the read-back mk1 at `singlesig_verify.go:123`), proving this is a divergence/regression, not a design choice. The unit test only calls `verifyMultisig` directly with explicit distinct mk1 args, so the flow's self-comparison wiring is never exercised.
- **Why HIGH not critical:** No panic, no secret leak, no NFC-secret transmission — a silent integrity hole in the safety net, harmful only on a coincident physical mk1 mis-engraving.
- **Suggested fix:** Make `extractSuppliedMd1` (or a dedicated gather) also accept the operator `cardMK1` and pass the **read-back** mk1 into `verifyMultisig`, mirroring `singleSigReadbackCards`. Drop the dead `derived bundle.Bundle` parameter or wire it through. Add a flow-level test that mutates the engraved mk1 plate bytes and asserts FAIL.

---

#### H2 — Single-sig verify-bundle FALSE-FAILS a correct multi-chunk md1 (order-sensitive compare vs map-random readback)
- **Severity:** HIGH · **Class:** verify-hole · **File:** `bundle/verify.go:64` (with `gui/md1_gather.go:57-63`, `md/chunk.go:145`)
- **Finder:** gui-scan-dispatch (`GSD-1`)
- **Description:** `bundle.Verify` compares the md1 leg with positional `equalStrings(derived.MD1, readback.MD1)` (verified: order-sensitive exact-string compare). The two sides have **different chunk orderings**: `derived.MD1` is produced by `split()` in **index order** (`for index := 0; index < count; index++`, verified at `chunk.go:145`); `readback.MD1` comes from `md1Gatherer.collected()`, which **ranges the Go map `g.set`** (verified — `for _, s := range g.set`) in Go's deliberately-randomized map-iteration order. No sort exists anywhere between the gatherer and Verify. A real wpkh single-sig md1 is 3 chunks (payload 81 bytes, 320-bit chunk limit), so the readback is a 3-element slice in random order vs the index-ordered derived side.
- **Trigger:** Engrave single-sig → "Verify now" → re-type seed → scan back the 3 md1 chunk plates over NFC. `equalStrings` compares random-order readback against index-order derived and reports "md1 string mismatch" → "Verify Failed" on a **correct, faithfully-engraved** backup. Affects both full and watch-only single-sig.
- **Verifier reasoning/repro:** Every link verified at HEAD. For a 3-chunk set the orders coincide in at most 1/6 of runs and are re-randomized each run, so a correct verify FAILs on the large majority of attempts/retries. `TestVerifyBundleMd1Reordered` proves the order-sensitivity is *intentional* (and that `Reassemble` is order-tolerant), so the comparator genuinely rejects a correctly-reassemblable but reordered set. The single-sig unit tests mask it by passing the derived (index-ordered) slice as the readback on both sides, never exercising the map-order gather path. Multisig is unaffected (both sides share `suppliedMd1`, same order).
- **Why HIGH not critical:** It **fails closed** — it never *passes* a true mismatch — so no fund-loss path. But it breaks the operator's only on-device confidence check and may cause a correct steel backup to be distrusted/discarded.
- **Suggested fix:** Sort both sides by chunk index before comparison, OR have `collected()` return chunks in `ChunkIndex` order (iterate `0..total-1` over the map rather than ranging it). Add a flow-level test that gathers chunks in shuffled order and asserts PASS.

---

### MEDIUM

---

#### M1 — ms1 verify compares only recovered entropy, silently ignoring codex32 prefix/language (non-English `mnem` readback with matching entropy falsely PASSES)
- **Severity:** MEDIUM · **Class:** verify-hole · **File:** `bundle/verify.go:83-97,122-136`
- **Finder:** bundle (`BND-1`)
- **Description:** `ms1Entropy(s)` calls `codex32.DecodeMS1` (returns prefix, language, entropy, err) but **discards prefix and language**, returning only entropy bytes; `Verify` then does `bytes.Equal(dEnt, rEnt)`. The BIP-39 **language byte** is load-bearing: identical entropy under a different wordlist yields a different mnemonic sentence → different PBKDF2 seed → different wallet (verified against BIP-39 and the firmware's own `MnemonicSeed` at `bip39.go:217-226`, which feeds wordlist word-strings into PBKDF2). The device only ever *engraves* an `entr`/language-0 ms1, and the derived side is always `entr`; but the read-back ms1 is **hand-typed** and gated only on `DecodeMS1` success, which admits the `mnem` prefix (0x02) with language 1..9.
- **Trigger:** Full verify-bundle (single- or multi-sig): operator re-types the seed (derived = `entr`/English, entropy E), then hand-types a valid `mnem`-prefix ms1 with a **non-English** language byte but the same entropy E. `bytes.Equal(E,E)` is true → "Verify OK," yet the typed-back secret recovers a different wallet.
- **Verifier reasoning/repro:** Every link confirmed. `codex32.New` validates only length/HRP/BCH-checksum/structure — no prefix/language gate — so a non-English `mnem` ms1 is fully reachable. `gui/ms1_decode.go:37-44` itself treats a non-English `mnem` as a different-wordlist wallet ("Restore with a `<name>` BIP-39 wallet"), confirming this is a material wallet change, not the "incidental string difference" the `verify.go:81-82` comment intends to tolerate. Tester can construct the malicious ms1 via `codex32.NewSeed("ms",0,"entr",'s',[]byte{0x02,0x01,<E...>})`.
- **Why MEDIUM:** Weakens a verification guarantee (false PASS) but requires a self-inflicted operator action (hand-typing a deliberately different secret); not a panic, leak, or remote path.
- **Suggested fix:** Have `ms1Entropy` also return prefix+language and compare them in `Verify`, OR reject any non-`entr`/non-language-0 ms1 at the verify-readback gate (the device never engraves anything else).

---

#### M2 — SLIP-39 `Combine`: recovered group-share secrets leak unwiped on multi-group error/abort paths
- **Severity:** MEDIUM · **Class:** secret-scrub · **File:** `slip39/combine.go:101-117` (also `:138-141`)
- **Finder:** slip39
- **Description:** `Combine` wipes `groupShares[].v` and `ems` only on the **success** path (lines 119-122). Three error returns skip the scrub: (a) `:103` when a later group's `recoverSecret` fails its digest — prior groups' `gv` stay live; (b) `:108` `errInsufficientShares` when recovered-group count ≠ `GroupThreshold` — all `gv` leak; (c) `:116` when the group-layer `recoverSecret` digest fails — every `groupShares[].v` leaks. Secondary: `recoverSecret`'s digest-fail branch (`:138-141`) wipes `s` but leaves `d` (interpolated digest‖random, secret-derived) unwiped — `d` is wiped only on the success branch (`:142`). The leaked `gv` buffers are **fresh** secret-bearing allocations (threshold==1 → `append([]byte(nil), shares[0].y...)`; threshold>1 → `interpolateSecretAt`'s `make([]byte,n)`), not aliases of input shares.
- **Trigger:** On-device multi-group SLIP-39 Recover (`GroupThreshold ≥ 2`). Cleanest path: a member-threshold-1 group has **no member digest** (SLIP-0039: a 1-member group carries the group share directly, no member-level Shamir split), so a transcription error that survives the detection-only RS1024 BCH checksum "recovers" at the member layer, then fails the group-layer digest at `:114-116`, leaking all prior groups' recovered secrets. Symmetric: a 2-group set where A verifies and B fails its digest leaks A's `gv` via `:103`.
- **Verifier reasoning/repro:** All code verified verbatim against `combine.go`/`lagrange.go`/`feistel.go`/`share.go`; SLIP-0039 group-share structure and RS1024-is-detection-only verified against the spec. `TestRecoverSecretWipesOnDigestFail` asserts only `recoverSecret`'s own `s`-wipe, so the `Combine`-level gap and the `d` gap are uncovered.
- **Why MEDIUM:** Leaked value is a group-level Shamir share of the EMS (not the raw master secret); wipe is best-effort under TinyGo GC (`feistel.go:15-16`); air-gapped, transient in-RAM, no exfil.
- **Suggested fix:** Move the `groupShares[].v` (and `ems`) scrub into a `defer` covering all return paths; on `recoverSecret`'s digest-fail branch also wipe `d`. Add tests asserting all `gv` are zeroed after each of the three error returns and that `d` is zeroed on digest-fail.

---

#### M3 — `seedxor.Combine`: per-part BIP-39 entropy intermediates left un-wiped (breaks the package's own scrub contract)
- **Severity:** MEDIUM · **Class:** secret-scrub (nfc-secret roster tag) · **File:** `seedxor/seedxor.go:38,44`
- **Finder:** seedxor
- **Description:** `Combine` deliberately scrubs the accumulator `out` on every exit (L40, L46, L54) and documents itself as a port of the Zeroizing-everywhere Rust reference — but the per-part entropy copies are never wiped: (1) `parts[0].Entropy()` at L38 returns a fresh secret heap slice (`bip39.splitMnemonic` → `big.Int.Bytes()` then `append(padding, entBytes...)`, a new allocation per call) copied into `out` then abandoned; (2) `e := p.Entropy()` at L44 allocates a fresh secret slice for **each** of `parts[1..N-1]`, none ever wiped — not on success, not on the `errMismatchedLengths` path (L46-47 wipes `out` but leaves the mismatched part's `e` live). For an N-part combine, N un-scrubbed raw share-entropy copies are left on the heap.
- **Trigger:** Main menu → input flow → "SEED XOR" (`gui.go:2160` → `combineSeedXORFlow` → `seedxor.Combine`). Every successful N-part combine on the air-gapped device leaks N raw-entropy copies.
- **Verifier reasoning/repro:** `Entropy()` provably returns a fresh allocation (verified against `bip39.go:177-197`); `TestCombineNoCallerMutation` relies on exactly that copy semantics, confirming `e`/`parts[0]` are distinct live copies, not aliases. OUR OWN code already does the missing pattern (`singlesig_derive.go:85-87`: `entropy := m.Entropy(); …; wipeBytes(entropy)`; `ms1_decode.go:29`), so `Combine` is the outlier against an established convention.
- **Why MEDIUM:** Real reachable missed scrub against the package's own contract, but no wrong-key, no NFC transmission; scrub is best-effort under TinyGo GC.
- **Suggested fix:** Bind each `Entropy()` to a named var and `wipe()` it after the XOR (and wipe `e` alongside `out` on the `errMismatchedLengths` path), mirroring `singlesig_derive.go:85-87`.

---

#### M4 — BIP-85 leaf EC private key object (`pkey`) is never `Zero()`'d — secret scalar survives `deriveBip85Child` return
- **Severity:** MEDIUM · **Class:** secret-scrub · **File:** `gui/bip85.go:101-108`
- **Finder:** gui-bip85
- **Description:** `deriveBip85Child` does `pkey, _ := k.ECPrivKey(); priv := pkey.Serialize(); k.Zero(); defer wipeBytes(priv)`. It scrubs `priv` (serialized 32 bytes) and `k` (the ExtendedKey) but **never calls `pkey.Zero()`**. `pkey` is a `*btcec.PrivateKey` (= `secp256k1.PrivateKey{ Key ModNScalar }`) holding the raw leaf scalar — the exact secret BIP-85 feeds into HMAC. Worse, `Serialize()` has a **value receiver** (`func (p PrivateKey) Serialize()`), so it copies the whole scalar; the copy is what `priv` reads while `pkey.Key` stays untouched. After return, the live `PrivateKey` object (which has a dedicated `.Zero()` precisely for memory-scraping defense) retains the secret. This contradicts the function's own SECURITY docstring (L56-59), which enumerates the scrubbed buffers and omits the privkey object.
- **Trigger:** Main menu → BIP-85 child-seed derive (`gui.go:1519`) → type master mnemonic → pick child word count + index → `deriveBip85Child` runs, leaving the leaf scalar resident in `pkey` (plus a value-receiver copy) unscrubbed.
- **Verifier reasoning/repro:** Verified against authoritative source — decred `secp256k1/v4@v4.4.1/privkey.go`: `type PrivateKey struct{ Key ModNScalar }`, `func (p *PrivateKey) Zero()` documented for memory-scraping, `func (p PrivateKey) Serialize()` value receiver; `btcec/v2@v2.4.0` aliases the decred type; `hdkeychain` `ECPrivKey()` materializes the scalar. The sibling `deriveAccountXpub` uses `ECPubKey()/Neuter()` and never materializes a `*PrivateKey`, so the gap is bip85-specific. Under TinyGo's non-moving GC the object simply persists unscrubbed (worse for the finding).
- **Why MEDIUM:** Best-effort memory-hygiene gap against the function's own committed contract; no wrong-key, no NFC leak.
- **Suggested fix:** Add `defer pkey.Zero()` after `Serialize()`.

---

### LOW

---

#### L1 — `DecodeMS1` probe discards secret BIP-39 entropy without scrubbing (3 call sites)
- **Severity:** LOW (down from claimed medium) · **Class:** secret-scrub · **File:** `gui/codex32_polish.go:103` (also `gui/singlesig_verify.go:116`, `gui/multisig_verify.go:93`)
- **Finder:** codex32 (`CX32-SCRUB-01`)
- **Description:** Three sites use `codex32.DecodeMS1` purely as a validity probe and discard the returned entropy with `_`. `DecodeMS1` → `Seed()` → `parts().data()` allocates a **fresh** `[]byte` ([prefix][full seed entropy]) on every call (`String` caches only `s string`; `data()` does `make+append`, no caching — verified). The 16-32 byte heap buffer carrying the master seed entropy is left for the GC, never zeroed — deviating from the codebase's own convention (`bundle/verify.go:ms1Entropy` copies-then-`wipe()`; `ms1_decode.go:29` `defer wipeBytes`).
- **Trigger:** Codex32 confirm screen with an unshared ms1 secret; or single/multi-sig bundle Verify "full" flow where the ms1 readback is hand-typed.
- **Verifier reasoning:** Mechanics all verified; genuinely reachable. **Downgraded medium → low:** at all three sites the same secret is *already* resident, for strictly longer and in fundamentally un-scrubbable form, as the codex32 Go **string** itself (`scan codex32.String` lives the whole confirm loop and is engraved verbatim; verify flows store `s.String()` into `ms1Readback`). Zeroing the short-lived decode buffer does not meaningfully shrink the RAM-exposure window the immutable string dominates.
- **Suggested fix:** Capture and `wipeBytes()` the probe's entropy return at all three sites, matching `ms1_decode.go:29` and `verify.go:134`. Low-cost consistency fix.

---

#### L2 — Multisig verify md1/mk1 legs are tautological yet success copy over-claims "engraved bundle matches the seed"
- **Severity:** LOW · **Class:** flow-logic · **File:** `gui/multisig_verify.go:26-29,100-104` (with `gui/multisig_derive.go:60`)
- **Finder:** bundle (`BND-2`)
- **Description:** The readback bundle is `{MS1: ms1Readback, MK1: reDerived.MK1, MD1: suppliedMd1}` while the derived side is `reDerived`, whose `.MD1` is `clone(suppliedMd1)` verbatim and `.MK1` is a stub computed from `suppliedMd1`. So inside `Verify`: `checkStubBinding` passes by construction; the mk1 legs compare `reDerived.MK1` vs itself; the md1 exact-string leg compares `clone(suppliedMd1)` vs `suppliedMd1`. The md1/mk1 legs **can never fail**, so a corrupted-but-decodable read-back md1 altering a **non-operator cosigner** key is not caught — yet the flow shows "Verify OK / The engraved bundle matches the seed." Only `findUserSlot` (operator-as-cosigner + operator xpub re-derivation) and the ms1 entropy leg are real checks.
- **Verifier reasoning:** All links verified. Partly inherent: the air-gapped device holds only the operator's seed and has **no independent source of truth** for other cosigners' public keys — so the verify gap for those legs is an unfixable limitation, not an exploitable acceptance bug. The genuine defect is the **UX/copy over-claim**. (Distinct from H1: this is the md1/mk1 self-clone tautology + over-claiming message; H1 is the structural never-read-back-the-mk1-plate wiring.)
- **Suggested fix:** Soften the success copy to scope the guarantee honestly (e.g. "Operator key + secret verified; other cosigners' public keys are taken as given"). No crypto change is possible for the public legs on an air-gapped device.

---

## 3. Severity Ranking (confirmed)

| # | Severity | Title | File:line |
|---|----------|-------|-----------|
| H1 | **HIGH** | Multisig verify never reads back the operator mk1 plate (compares re-derived mk1 to itself) | `gui/multisig_verify.go:100` |
| H2 | **HIGH** | Single-sig verify FALSE-FAILS a correct multi-chunk md1 (positional compare vs map-random readback) | `bundle/verify.go:64` |
| M1 | **MEDIUM** | ms1 verify ignores codex32 prefix/language — non-English `mnem` readback falsely PASSES | `bundle/verify.go:83-97,122-136` |
| M2 | **MEDIUM** | SLIP-39 `Combine` leaks recovered group-share secrets on error/abort paths | `slip39/combine.go:101-117` |
| M3 | **MEDIUM** | `seedxor.Combine` leaves per-part entropy intermediates un-wiped | `seedxor/seedxor.go:38,44` |
| M4 | **MEDIUM** | BIP-85 leaf privkey object `pkey` never `Zero()`'d | `gui/bip85.go:101-108` |
| L1 | **LOW** | `DecodeMS1` probe discards secret entropy without scrubbing (3 sites) | `gui/codex32_polish.go:103` |
| L2 | **LOW** | Multisig verify md1/mk1 legs tautological; success copy over-claims | `gui/multisig_verify.go:26-29,100-104` |

**Theme:** 4 of 8 are in the **verify-bundle** subsystem (`bundle/verify.go` + the gui verify flows). Both HIGHs and both verify-hole MEDIUMs/LOW concentrate there — the verify path is the weakest cluster and warrants a dedicated hardening pass + flow-level (not just unit) tests that exercise the real NFC-gather → Verify wiring.

## 4. Uncertain Findings (worth a human look)

**None.** All 8 candidates resolved to a definitive `confirmed` verdict under refute-by-default; no items were left in the uncertain bucket.

## 5. Refuted Candidates (audit trail)

**None.** Every candidate that reached verification was confirmed (with two severity downgrades: CX32-SCRUB-01 medium→low; all others held at claimed severity). There were no refutations in this round.

| Candidate | Verdict | Note |
|-----------|---------|------|
| *(none refuted)* | — | 8/8 candidates confirmed; 2 severity adjustments noted inline |

## 6. Final Adversarial Sanity Pass (HIGH/critical)

Both HIGHs were re-checked against live source at HEAD `3a23dbb` and **survive**:

- **H1 (mk1 self-compare):** Re-verified the call site passes `reDerived.MK1` on the readback side and that `extractSuppliedMd1` returns `false` on any `cardMK1` (so reading the mk1 back is structurally impossible), while the single-sig sibling correctly requires both cards via `singleSigReadbackCards`. The bug is real and not a deliberate design choice (the docstring claims a mk1 read-back that the code never performs). Held at HIGH: it is conditional on a coincident physical mk1 mis-engraving, fail-closed otherwise (no false PASS without that fault), so **not** critical. **Not downgraded.**
- **H2 (single-sig false-FAIL):** Re-verified `collected()` ranges the Go map (random order), `split()` emits index order, and `equalStrings` is positional with no intervening sort. The effect is deterministic-in-aggregate (≤1/6 chance of accidental agreement per run, re-randomized). Held at HIGH: it **fails closed** (never passes a true mismatch → no fund-loss), so not critical, but it breaks the operator's only on-device confidence check. **Not downgraded.**

No HIGH was found questionable on the final pass. No critical findings exist in this round.

## 7. Residual Risk / Not Covered

- **Coverage was COMPLETE** — all 13 finder subsystems ran and produced results; **no failed-finder coverage gap** requires a re-run. The hunt is complete with respect to its planned fan-out.
- **Inherent, non-fixable limit (documented, not a bug):** multisig verify cannot validate non-operator cosigner public keys on an air-gapped device (no independent source of truth) — see L2. Treat as a UX-honesty item, not a crypto fix.
- **Best-effort scrub caveat applies to all secret-scrub findings (M2/M3/M4/L1):** TinyGo's GC may copy/retain regardless, so the fixes are defense-in-depth, not guarantees. They are still worth applying for consistency with the codebase's stated contract and to minimize the residency window. Note that under TinyGo's non-moving GC, the un-scrubbed objects simply *persist* — which strengthens (not weakens) the M4/L1 case.
- **Scope boundary:** this audit covered only our divergence from merge-base `86a58ab`. Upstream SeedHammer code and the `me-cli`/`me-preview` host-side crates were out of scope and not examined here.
- **Test-masking pattern observed:** H1, H2, and the unit suites for the verify flows all pass derived/synthetic data as the "readback," so unit tests are systematically blind to the real gather→Verify wiring. Recommend adding **flow-level** verify tests (shuffled-chunk gather; mutated engraved plate bytes) before closing the verify-bundle hardening cycle.
