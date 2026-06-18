<!--
Persisted verbatim. Architect panel (design/decomposition/UX lens), agentId a9abd39929fbc8e43. Verdict: decompose into D1 (crypto, mergeable) + D2 (GUI); redesign two-level collection UX as a group-satisfaction roster; Rust split side as checked-in round-trip fixture oracle; reuse backupWalletFlow + disambiguation + fingerprint.
The text below is the agent's report exactly as returned; do not edit.
-->

# Design-input brief — Cycle D: on-device SLIP-0039 secret recovery

Sources reviewed: `design/SPEC_seedhammer_slip39_recovery.md`, `design/cycle-prep-recon-slip39-recovery.md`, the Cycle-B precedent `gui/codex32_polish.go` + `design/SPEC_seedhammer_codex32_multishare_recovery.md`, the fork's `gui/gui.go` (`backupWalletFlow:1929`, `SeedScreen.Confirm:2062`), and the Rust port `mnemonic-toolkit/.../slip39/{mod,gf256,lagrange,feistel,share}.rs`.

The spec is unusually strong: protocol facts are verified against authoritative source, the security invariants in §2 are the right ones, and the "what to engrave" decision (§3) is correctly reasoned and is the single highest-risk semantic call — it's resolved well. My input is about **structure**, not correctness of the resolved decisions.

---

## Q1 — Scope: one cycle, or decompose? **Decompose. Two phases, hard gate between.**

Cycle B was one cycle because it added **zero crypto** — its own SPEC §1 says "Crypto already exists... this is a GUI-flow + small-API cycle." Cycle D is the opposite: it ships ~600-900 LoC of **new, from-scratch finite-field cryptography** (GF(256), Lagrange, 4-round Feistel/PBKDF2, two-level combine, bit-packing share extraction) *plus* a harder-than-B two-level GUI flow *plus* an entropy→BIP-39 engrave bridge. Bundling a fresh cryptosystem and a novel UX behind a single R0 gate and a single whole-diff review means the adversarial reviewer is amortizing attention across two very different risk domains at once — exactly when the crypto half deserves undivided scrutiny.

The natural fracture line is already in the file manifest (§8): the four new crypto files + `share.go` are pure, deterministic, oracle-testable, and have **no GUI dependency**; the GUI half imports them through one function, `Combine`. They are independently reviewable and independently shippable.

**Recommended decomposition:**

- **Phase D1 — crypto port + share-value extraction (no GUI).** Ships: `slip39/{gf256,lagrange,feistel,combine}.go`, the `share.go` widening to {20,23,27,30,33} words + `Value []byte` + padding validation, `ConsistentShares`, `Describe` extensions, and the full §7 vector + negative + panic-safety test suite diffed against the Rust oracle. **Gate: every official vector passes, every negative returns the right sentinel, panic-safety proven, 0C/0I review.** This phase is mergeable on its own — it widens `ParseShare` (which also unblocks the filed Cycle-C all-lengths follow-on) and adds dormant-but-tested crypto. Nothing in the GUI calls `Combine` yet, so merging it cannot regress any user-facing flow.
- **Phase D2 — GUI recover flow + engrave bridge.** Ships: the `slip39ConfirmAction` enum + Recover button + Button2-drain, `recoverSLIP39Flow`, the `inputSLIP39Flow` title param, the two-level collection UX (Q2), the passphrase choice, and the `backupWalletFlow` dispatch. Gate: GUI tests + a second whole-diff review focused on UX/event-routing correctness against a now-trusted crypto core.

**Order matters and is forced:** D1 before D2, because D2's collection sufficiency logic and `recoverSLIP39Flow` are written *against* `Combine`'s preconditions. You cannot meaningfully review the GUI's "is the set sufficient yet?" logic until `Combine`'s contract is frozen and trusted. This also means that if D1 slips, D2 doesn't drag a half-trusted crypto core into a UX review.

The spec/recon both currently say "single XL cycle" (SPEC §9, recon §7). My recommendation is to override that to **two gated phases under one cycle umbrella** — same pipeline, same branch lineage, but two R0 gates and two execution reviews with a merge of D1 in between.

---

## Q2 — The two-level collection UX (SPEC §5.2). **The weakest part of the spec. Critique + concrete fix.**

This is where mirroring Cycle B is actively misleading, and the spec under-specifies it. Codex32's prompt is `"Share i of k"` — a *flat, fully-known* target: the first share tells you k, and you count up to k. **SLIP-39 has no single k the device can show.** The device knows `groupThreshold` (GT) and `groupCount` from share 0, but it does **not** know, from any one share, *which* groups the user holds shares for, nor the member thresholds of groups it hasn't seen a share from yet. The total number of shares required is `Σ memberThreshold` over the GT groups the user chooses to satisfy — and the user, not the device, chooses which groups.

The spec's proposed title `"Group g · share m of t"` plus "groups still needed" is a reasonable *atom* but it papers over the navigation problem: a user holding a physical pile of shares of unknown grouping needs to know **which pile to reach for next and when they can stop**, and the §5.2 design's sufficiency definition ("exactly groupThreshold distinct groups each holding exactly its memberThreshold shares") is stated as a terminal predicate, not as live guidance.

There's also a real **abort hazard** buried here: §5.2 step 2 says collect "until the set is sufficient," but with two-level structure a user can wander into a state that can *never* become sufficient with the shares they hold (e.g. they have full member thresholds for only GT−1 groups). The flat codex32 loop can't hit this; the SLIP-39 loop can, and the spec has no "you cannot complete with these shares / start over" affordance — only per-share `ConsistentShares` rejection and a final `Combine` error.

**Concrete recommendation — make the collection screen a live tally, not a counter:**

1. After share 0, show a **standing roster**: `Need <GT> of <groupCount> groups`. As shares arrive, render one line per group *seen*: `Group <GI>: <m>/<memberThreshold> ✓|…`. A group flips to satisfied (✓) at `m == memberThreshold`. A top-level line shows `<satisfied>/<GT> groups complete`.
2. The "done" condition is reached when `satisfied == GT` — and at that point the device should **stop prompting and offer Continue**, not require the user to guess they're finished. (Note the Rust `Combine` requires *exactly* GT groups and *exactly* memberThreshold each — SPEC §4 step 4/5 — so the UX must actively *stop accepting* shares for already-satisfied groups and for groups beyond GT, or surface `errInsufficientShares`/too-many cleanly. The spec's "exactly" semantics make over-collection an error, which the flat-counter UX never had to handle.)
3. Add an explicit **per-share group/member readout on entry confirmation** (`This is Group <GI>, member <I> of threshold <t>`) so the user can sort their physical pile against the roster.
4. Keep `Back` as "remove last share," but add a recoverable path for the dead-end state (Q6 risk #2).

This is enough additional UX surface that it reinforces Q1: it deserves its own phase and its own review.

---

## Q3 — The double-passphrase flow. **Reuse `backupWalletFlow` for the engrave — but it is a genuine UX hazard, mitigate explicitly.**

Reuse is the right call architecturally. `backupWalletFlow` (`gui.go:1929`) is exactly the post-recovery path you want: it does `SeedScreen.Confirm` → optional BIP-39 passphrase → fingerprint choice → `engraveSeed`, all already gated and golden-tested. Re-implementing a parallel confirm+engrave for recovered seeds would duplicate the most safety-critical screen in the firmware and create two code paths that can drift. Don't do that.

**But** the spec is right to flag this as R0-relevant, and it under-weights the hazard. The user enters a SLIP-39 EMS passphrase during recovery, then `backupWalletFlow` *immediately* offers a second, differently-purposed passphrase ("Add a BIP-39 passphrase?" at `gui.go:1941`). Two optional passphrases at two stages, feeding two different algorithms, is a textbook conflation footgun — and it composes with invariant §2.3 and the **silent-wrong-secret property**: a wrong SLIP-39 passphrase yields a *different valid* master secret with no error, which then flows into `backupWalletFlow` and engraves a wrong-but-plausible seed. The user gets no signal.

Recommendation: **route through `backupWalletFlow` unchanged, but bracket it with disambiguation.**
- Label the SLIP-39 passphrase prompt unmistakably (the spec's "SLIP-39 passphrase (not a BIP-39 passphrase)" is good — keep it).
- Surface the resulting **fingerprint** on the recovered-seed confirm before any engrave, and document (on-device note or doc) that a wrong SLIP-39 passphrase silently produces a different wallet — the device cannot detect it. This is the only real defense against the deniability property, and it's free because `masterFingerprintFor` is already computed in `backupWalletFlow`.
- Do **not** try to make `backupWalletFlow` "aware" it's handling a recovered seed (e.g. suppressing the BIP-39 passphrase offer). The BIP-39 25th word is legitimately independent and may be wanted. Suppressing it would be a wrong simplification. Keep the flow generic; fix the problem with labeling + fingerprint visibility.

---

## Q4 — Mirroring Cycle B: what transfers vs what's a trap.

**Genuinely transfers (copy with confidence):**
- The **confirm-action enum + unconditional Button2-drain** pattern (`codex32_polish.go:101-122`). This is a real, hard-won EventRouter footgun (Cycle-B R0-C1) and the SLIP-39 analogue is identical. The spec correctly carries it (§5.1).
- The **engrave dispatch loop** shape (`engraveCodex32`, lines 198-217): Back returns `true` (recognized, not "Unknown format"), Recover loops back to confirm on abort. Directly applicable.
- The **eager per-candidate `ConsistentShares` + `Describe`-labeled ErrorScreen** validation idiom. The *idiom* transfers; the *checker* is more complex (Q2).
- **Reusing the existing engrave path** rather than building a new one (B reused `backupSeedStringFlow`; D reuses `backupWalletFlow`). Same principle.

**Traps — where copying B's structure misleads:**
1. **`"Share i of k"` → flat counter (Q2).** B's collection is a known-length count-up. Copying that title shape is the single biggest trap: SLIP-39 has no device-knowable single k. This must be redesigned, not ported.
2. **`Interpolate`-as-final-authority defense-in-depth.** B's `recoverCodex32Flow` leans on `Interpolate`'s own internal re-checks as a backstop (B SPEC §5). D's `Combine` has the **digest gate** (§4.3) which is a *cryptographic* authority, not just a structural one — but it only fires for threshold ≥ 2 layers, and **threshold==1 layers have no digest** (SPEC §4.3, Rust `recover_secret`). Treating `Combine` error as the sole correctness backstop the way B treats `Interpolate` is misleading: for a 1-of-1 group/1-of-1 outer config there is *no* integrity check at all, and a wrong SLIP-39 passphrase never errors anywhere. B's "the crypto will catch it" mental model does not hold for D.
3. **Engrave artifact identity.** B engraves the recovered string **verbatim** (GF(32), same alphabet in and out — `engraveCodex32` just re-confirms the same `codex32.String`). D engraves a **transformed** artifact: recovered bytes → `bip39.New(secret)` → a *different representation* (words + SeedQR) under a *convention assumption* (§3). The "recover then re-confirm the same thing" loop in `engraveCodex32` does not map — D's confirm is of a derived BIP-39 mnemonic the user never typed, which raises the stakes on the fingerprint-visibility point in Q3. Copying B's "loop re-confirms it" comment would be semantically wrong here.

---

## Q5 — Rust-as-oracle TDD. **Right approach. One real gap: porting only `combine` forfeits the split→combine round-trip oracle.**

Porting an already-audited, constellation-gated implementation (`SPEC_slip39_v0_13_0.md`) and diffing the Go against it on the official `vectors.json` is the correct way to de-risk crypto correctness — it's strictly better than re-deriving from the spec, and the recon already independently reproduced 15/15 valid vectors and caught the two classic bugs (generator must be 3; Feistel decrypt returns `r||l` reversed). Provenance is high.

**The gap the spec doesn't fully confront:** the Rust source contains **both** `slip39_split` (`mod.rs:93`) and `slip39_combine` (`mod.rs:206`), and its own test suite uses the split→combine **round-trip** as an oracle (e.g. `split_secret_*` tests at `mod.rs:496/676`). By porting **combine only** (SPEC §1, In-scope), Phase D1 loses the ability to generate *fresh* random share sets and prove they round-trip. You are left with only the **static, public-corpus vectors** — which are a fixed 45 entries (recon §3) and notably **thin on the unusual lengths**: the spec itself admits (§7) "Add a 256-bit (33-word) and a 23/27/30-word length case **if present in the corpus; otherwise synthesize via the Rust oracle**." That "otherwise synthesize" is doing a lot of quiet work — for the 20/23/27/30/33-word bit-packing/padding logic in the new `share.go`, the official corpus may not exercise every length, and combine-only Go cannot generate them itself.

**Recommendation — keep combine-only in the firmware, but make the *Rust split side* a first-class test-fixture generator, not an afterthought:**
- In D1's test harness, drive the **Rust `slip39_split`** (it's right there) over each of the five master-secret lengths × a couple of group topologies to emit share sets + expected master secrets as committed Go fixtures. This restores the round-trip property as an *offline* oracle: Rust splits → Go combines → must equal the Rust input. This closes the length-coverage hole the static corpus leaves, especially for the 23/27/30-word intermediate lengths that almost certainly aren't all in `vectors.json`.
- Make this generation **reproducible and checked-in** (a small documented Rust harness + the resulting fixtures), so the oracle relationship is auditable and doesn't depend on a developer's local toolkit checkout at review time.
- Don't port `split` into the firmware — that would pull in RNG + Feistel-encrypt for no shippable benefit and violates the in-scope boundary. The split side belongs only in the test fixture pipeline.

Secondary note: the spec's panic-safety requirement (§4.4) is correct and load-bearing — the Rust uses `assert!`-as-precondition and the firmware's share *set* is attacker/typo-controlled. The oracle does **not** cover this (the Rust asserts; the Go must return errors), so the §7 panic-safety test is not redundant with the vector diff — keep it explicitly.

---

## Top-3 design/scope risks (ranked)

1. **Single-cycle scope concentrates a fresh cryptosystem and a novel UX behind one gate, diluting crypto scrutiny.** *Mitigation:* decompose into D1 (crypto + share-value, independently mergeable and oracle-tested) and D2 (GUI), with a hard 0C/0I gate and a dedicated whole-diff review between them. (Q1.)
2. **Two-level collection is genuinely harder than codex32 and the spec mirrors the flat "i of k" model, leaving the user unable to navigate an unknown-grouping pile — and able to enter a never-completable dead-end state.** *Mitigation:* replace the counter with a live group-satisfaction roster (`satisfied/GT` + per-group `m/threshold`), stop-and-offer-Continue at sufficiency, per-share group/member readout, and an explicit dead-end/start-over affordance. (Q2.)
3. **Silent wrong-secret on a mismatched SLIP-39 passphrase, compounded by the double-passphrase flow and the entropy→BIP-39 *transform* (vs codex32's verbatim engrave).** A wrong EMS passphrase never errors and engraves a plausible wrong wallet; threshold-1 layers have no digest gate either. *Mitigation:* surface the recovered **fingerprint** on the post-recovery confirm (free — `backupWalletFlow` already computes it), label the two passphrases unmistakably, and document the deniability caveat on-device. (Q3, Q4 trap #2, Q5.)

---

## Overall verdict

**Decompose as follows:** run Cycle D as **two gated phases under one cycle** — **D1 = crypto port + `share.go` value extraction + `Combine`/`ConsistentShares`, fixture-generated round-trip via the Rust *split* side, merged on its own once vectors+negatives+panic-safety are GREEN; then D2 = the redesigned two-level collection UX + passphrase disambiguation + `backupWalletFlow` engrave bridge, with fingerprint visibility as a first-class confirm element.** The crypto-sourcing and "what to engrave" decisions are sound and should not be reopened; the work to de-risk is structural (split the gate) and UX (the §5.2 collection model and the double-passphrase confirm), not algorithmic.
