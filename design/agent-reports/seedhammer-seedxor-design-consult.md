<!--
Persisted verbatim. Architect design-input consult for the Seed XOR combine cycle.
Agent agentId a457b750cd8a65849. Locked decisions: (1) NO two-way fork / interpretation
hold-to-confirm (Seed XOR result is unambiguously a BIP-39 seed -> backupWalletFlow direct);
(2) MANDATORY Seed-XOR-specific fingerprint gate with 'no built-in check' wording (the ONLY
safety net — no auth tag); (3) ask N up front, first part fixes length, reject mismatches;
(4) Path A menu hook (SEED XOR peer entry -> bip39.Mnemonic -> existing dispatch); (5) restrict
to Coldcard-interop 12/18/24-word (needs a deliberate guard; bip39.New doesn't enforce it).
Split deferred. Combine-only, S. The text below is the agent's report exactly as returned; do not edit.
-->

# Design brief — Seed XOR combine (SeedHammer fork, cycle prep)

Read: recon `design/cycle-prep-recon-seedxor.md`; port `mnemonic-toolkit/src/seed_xor.rs` + `cmd/seed_xor.rs`; fork `gui/slip39_polish.go`, `gui/codex32_polish.go`, `gui/gui.go`, `bip39/bip39.go` (all at fork `main` `bc63caa`). Bottom line at the end.

---

## 1. Engrave-artifact ambiguity — CONFIRMED ABSENT. Drop the two-way fork.

**Confirmed, and this is the single biggest simplification vs Cycle D.** The SLIP-39 fork in `engraveRecoveredSLIP39` (`slip39_polish.go:388–428`) exists because of a genuine type ambiguity the recon for that cycle flagged: a SLIP-39 master secret is a *raw secret* that is *either* BIP-39 entropy (constellation/Coldcard convention → engrave as a derived BIP-39 seed plate) *or* an opaque BIP-32 seed (Trezor convention → engrave the shares verbatim). The device cannot tell which from the bytes, so it asks the user, framed by "how was this backup made."

Seed XOR has no such ambiguity **by construction**: every input is a BIP-39 mnemonic, the operation is XOR of BIP-39 *entropy*, and `bip39.New(result)` (`bip39.go:228`) re-derives a BIP-39 mnemonic with a valid checksum. Input type = output type = `bip39.Mnemonic`. There is exactly one correct engrave path — `backupWalletFlow` (`gui.go:1929`) — and no interpretation question to put to the user.

**Recommendation:** No "Trezor/verbatim" fork, no interpretation hold-to-confirm. After combine, call `backupWalletFlow(ctx, th, m)` directly. The flow is closer to `recoverCodex32Flow` (`codex32_polish.go:161`, which has no interpretation fork) than to the SLIP-39 recovery — but flatter still, because codex32 must re-confirm/re-engrave the *recovered codex32 string* whereas Seed XOR just hands a plain mnemonic to the existing dispatch.

One nuance the spec should state explicitly so it isn't mistaken for an omission: `backupWalletFlow` already runs its own `SeedScreen.Confirm` + `masterFingerprintFor` + optional passphrase. So the recovered seed *is* shown word-by-word and fingerprinted by the existing path. The only question (Q2) is whether Seed XOR needs an *additional* gate before that.

---

## 2. No-auth-tag safety — WORSE than SLIP-39. Make the fingerprint gate MANDATORY and Seed-XOR-specific in wording.

**The recon is right that this is the weakest member of the recovery family, and I'd state it more sharply than the recon does.** Compare the three:

- **codex32** (`recoverCodex32Flow`): Lagrange interpolation over a fixed share set with a BCH checksum; `ConsistentShares` (`codex32_polish.go:180`) rejects shares that don't belong to the same secret. A substituted share is *caught*.
- **SLIP-39** (`recoverSLIP39Flow`): `ConsistentShares` (`slip39_polish.go:234`) + a per-share digest; an inconsistent set is caught. The fingerprint gate (`confirmSLIP39Fingerprint`, `:433`) is a *second* belt against the residual passphrase/plausible-deniability case.
- **Seed XOR**: *nothing.* Any N equal-length BIP-39 mnemonics XOR to a valid BIP-39 wallet. There is no consistency relation among "correct" parts to check — the decoy property (recon §1) is exactly the statement that every part-set is internally consistent. A wrong-but-valid substituted part produces a different valid wallet, silently.

So for Seed XOR the fingerprint gate is not a second belt — **it is the only safety net, and it sits over a strictly weaker primitive.** The toolkit CLI already frames it at maximum strength: the combine path emits *"Seed XOR has no authentication tag; verify the recovered wallet's expected derived address before trusting; if a share was substituted... the result will validate but derive the wrong wallet"* (`cmd/seed_xor.rs:372–375`). The firmware should match that register.

**Recommendation — three concrete asks for the spec:**
1. **MANDATORY, not optional.** The recovered-fingerprint confirm screen must be on the only success path to `backupWalletFlow`, with no skip. (Structurally: a thin `seedxor_polish.go` clone of `confirmSLIP39Fingerprint`, or call that helper directly — it's already generic over `mfp uint32`.) Note this is *in addition to* `backupWalletFlow`'s own internal flow; do the explicit Seed-XOR fingerprint confirm **before** the handoff, so the framing copy is Seed-XOR-specific rather than the generic seed-plate confirm.
2. **Stronger wording than SLIP-39's.** `confirmSLIP39Fingerprint` says "Confirm this matches your wallet records before engraving." For Seed XOR, name the absence of a check: e.g. *"Seed XOR has no built-in check. Any wrong part still produces a valid-looking wallet. Confirm this fingerprint matches your records before engraving."* This is the meaningful framing difference vs SLIP-39: SLIP-39's gate guards a residual edge case after a digest already passed; Seed XOR's gate is load-bearing for the whole operation, and the copy should say so.
3. **The user must already know their target fingerprint** — that is the implicit precondition, and the spec should state it as a documented assumption (an operator who doesn't have their fingerprint on hand gets zero protection from the gate; that's inherent to the primitive, not a fixable UX gap).

The Button2-drain idiom (`slip39_polish.go:104–110`, `:445`) must be replicated on any new confirm screen — this is the documented Cycle-B R0-C1 EventRouter footgun and it's non-obvious. Flag it in the spec so the implementer doesn't drop it.

---

## 3. Combine entry UX — ask N up front (Option a), first part fixes the length, reject mismatches at input time.

**Recommend Option (a): a "How many parts?" `ChoiceScreen` up front, then collect exactly N.** Reasoning:

- **Symmetry / least-surprise.** SLIP-39 already establishes "tell the device the shape up front" via `slip39LengthPick` (`slip39_polish.go:40`). The collection-loop idiom in both `recoverSLIP39Flow` and `recoverCodex32Flow` is a fixed-count `for len(shares) < k` loop with a live "Share i of N" progress title (`codex32_polish.go:169–170`). Option (a) drops straight into that proven pattern; Option (b) (collect-until-done) is a *new* interaction shape the codebase doesn't have, which means new edge cases (what does "done" look like; can you finish at N=1?) and more test surface — against the S sizing.
- **The decoy property makes "collect until done" actively dangerous.** Because every subset is a valid wallet, a collect-until-done flow has no signal to stop at the right count — if the user fat-fingers "done" one part early, the device cheerfully combines N−1 parts into a *valid wallet with a different fingerprint*. Asking N up front turns "I have a pile of 3" into a committed count the device enforces, and the mandatory fingerprint gate (Q2) is the backstop if the count itself was wrong. Collect-until-done has neither guard.
- **Holding a pile of equal-looking parts:** the user knows how many parts they made (it's the one piece of metadata that lives in their head / their records, since the parts carry none). "How many parts?" maps directly to that knowledge. There's no scanning-order or which-is-which problem because combine is order-independent (recon §1).

**Picker shape:** a small `ChoiceScreen` titled e.g. "Seed XOR" / "How many parts?" with choices `2, 3, 4, 5` (mirror `slip39LengthPick`'s return-0-on-back convention). 2–5 covers every realistic Seed XOR setup; you can offer more but there's no real demand. `seed_xor_combine` requires `MIN_SHARES = 2` (`seed_xor.rs:27`), so 2 is the floor.

**Length enforcement — first part sets it, reject mismatches at entry, no separate length picker.** Unlike SLIP-39 (where `slip39LengthPick` is needed because `inputSLIP39Flow` fills a *pre-sized* slice), Seed XOR should NOT ask word-length up front:

- Ask only N. Collect part 1 with the existing 12/24 (and possibly 18 — see Q5) BIP-39 entry; whatever length the user enters fixes `L`.
- Parts 2..N: enforce `len(part) == L`. The cleanest enforcement is to allocate `emptyBIP39Mnemonic(L)` for each subsequent part so the entry UI only *permits* L words — the user physically cannot enter a mismatched length, which is better than entering then rejecting. (If, for Q5 reasons, you let part 1 be any of several lengths, derive L from part 1 and size every later part's `emptyBIP39Mnemonic(L)` accordingly.)
- Keep the toolkit's explicit mismatch refusal (`cmd/seed_xor.rs:328–339`, the library's `MismatchedShareLengths`, `seed_xor.rs:166`) as a defensive `showError` even though the pre-sized-slice approach should make it unreachable — defense-in-depth, matching how `selectForCombine` keeps a defensive `ok=false` branch the loop "guarantees" can't hit (`slip39_polish.go:248`).

So the flow is: **N picker → part 1 (length L emerges) → parts 2..N each pre-sized to L via `inputWordsFlow` → `Entropy()` each (`bip39.go:158`) → XOR fold → `bip39.New(result)` (`:228`) → mandatory Seed-XOR fingerprint gate → `backupWalletFlow`.** This is the recon's flow with the length-picker removed (the recon's §3 mention of "how many parts" picker is right; it should NOT also add a length picker).

A combine helper should live as a tiny pure function (the ~50 LoC Go port of `seed_xor_combine`) so it's unit-testable against the Coldcard vectors and the toolkit G1 byte-pin independently of the GUI.

---

## 4. Menu hook — Path A confirmed; "SEED XOR" is a proper peer.

**Confirmed: Path A (a new choice on the input `ChoiceScreen`, `gui.go:2012–2016`).** It's the natural seam:

- That `ChoiceScreen` is explicitly the "what are you inputting" menu — `{12 WORDS, 24 WORDS, CODEX32, SLIP-39}`. Seed XOR is exactly another *input modality* that yields a seed, so it belongs there as a peer. CODEX32 and SLIP-39 are already non-word-count entries on the same menu, so a `"SEED XOR"` peer is consistent, not a category error.
- The combine flow returns a `bip39.Mnemonic`, and `newInputFlow` returns `(any, bool)` to `engraveObjectFlow` (`gui.go:1847`), whose `case bip39.Mnemonic:` already routes to `backupWalletFlow`. **Zero new dispatch.** This is the same property that made the 12/24-word cases trivial.

**Reject the alternative (an action on the BIP-39 confirm screen).** Seed XOR combine is *initiated* before you have any single mnemonic to confirm — it consumes N inputs and produces one. Bolting "combine with more parts" onto the post-entry confirm screen (the SLIP-39/codex32 Button2-Recover pattern) is the wrong fit: those schemes Recover *from a parsed share that already carries set metadata* (threshold, id). A bare BIP-39 mnemonic carries no "I am part 1 of a Seed XOR set" signal, so there's nothing to key a Recover affordance off — you'd be offering "combine" on *every* BIP-39 entry, which is noise. Entry-menu peer is correct.

Menu copy: `"SEED XOR"` matches the existing all-caps style. Place it after SLIP-39 (end of the recovery family).

---

## 5. Length scope — restrict to Coldcard-interop **12 / 18 / 24** word. Exclude 15/21. This needs an explicit guard.

**This is the decision with a real footgun, and the firmware does NOT enforce it for free — the spec must add a guard.** Key facts I verified in the firmware:

- `bip39.New` (`bip39.go:228`) accepts **any** entropy of 16–32 bytes in multiples of 4 — i.e. 16/20/24/28/32 = **12/15/18/21/24 words**. It will silently produce a 15- or 21-word mnemonic.
- `emptyBIP39Mnemonic(nwords)` and `inputWordsFlow` are word-count-generic; the only thing pinning the existing menu to 12/24 is the literal `[]int{12, 24}[choice]` (`gui.go:2024`). There is no firmware-level interop restriction.
- The toolkit explicitly treats 15/21 as **non-interop extensions** (`seed_xor.rs:5–7`: "Coldcard interop is pinned at lengths {16,24,32}... 20/28-byte are toolkit-only") and the CLI emits a loud advisory that 15/21 shares "will NOT round-trip a Coldcard device" (`cmd/seed_xor.rs:224–232`).

**Recommendation: restrict to 12/18/24 (16/24/32-byte).** A SeedHammer engraver's *entire purpose* is to reproduce a seed that some hardware wallet will restore. A 15- or 21-word Seed XOR result restores on essentially nothing in the field (Coldcard, the originator, is 12/18/24 only). Engraving a 15/21-word Seed XOR plate is a near-pure footgun: it costs a real plate to materialize a backup that the user's actual wallet can't ingest, and the device has no way to warn at restore time. The CLI can afford to support 15/21 behind a stderr advisory because a CLI user is a power user reading warnings; the engraver user is mid-physical-operation and a stderr-equivalent warning has far less stopping power.

Concretely the spec should:
- Offer only 12/18/24-word entry for Seed XOR parts (add 18 to the entry options for this flow — it isn't on the current menu but is Coldcard-standard and the entry code supports it). Note this is a *superset* of the current 12/24 menu in the 18-word direction — fine, because Coldcard standardizes 18.
- **Reject** a part whose length isn't 16/24/32 bytes with a `showError`, even though pre-sizing the entry slice should make it unreachable for parts 2..N (defense-in-depth, and it guards part 1).

If the user genuinely wants a 15/21-word Seed XOR, the toolkit CLI already serves that niche with eyes open. Keep the firmware Coldcard-clean. **This is the one place I'd deviate from leaving it open** — the recon (§1, §4) lists it as a decision to "decide"; I'm recommending *decide = restrict*, and noting it requires a deliberate guard rather than relying on absent enforcement.

---

## 6. Top risks + sizing

1. **Silent wrong-wallet on a substituted/miscounted part (the core hazard).** *Mitigation:* mandatory Seed-XOR-specific fingerprint gate with "no built-in check" wording (Q2) + N-asked-up-front so the count is committed and enforced (Q3). This is the cycle's defining safety requirement, not a nice-to-have; the R0 review should treat a skippable/soft gate as a Critical.
2. **Dropped Button2-drain on the new confirm/fingerprint screen → router hang.** *Mitigation:* reuse `confirmSLIP39Fingerprint` verbatim if possible, or copy its unconditional `drainBtn.Clicked(ctx)` (`slip39_polish.go:445`); call it out in the spec and make it an R0 checklist item. This is a known, recurring, non-obvious footgun in this codebase.
3. **Scope-creep into 15/21-word support or collect-until-done UX.** *Mitigation:* the spec locks 12/18/24-only (Q5) and N-up-front (Q3) as decisions, not options, so the implementer doesn't reopen them. Secondary: someone proposes on-device *split* — explicitly out of scope (no CSPRNG; recon §3 DECISIVE finding).

**Sizing sanity check:** Combine-only is genuinely **S** — ~50 LoC pure XOR helper + a thinned `seedxor_polish.go` (no threshold parse, no roster, no `ConsistentShares`, no interpretation fork — strictly less than codex32) + N-picker + one menu line; tests cross-checked against the captured Coldcard vectors and the toolkit G1 byte-pin. The fingerprint gate and Button2-drain are reuse, not new design.

---

## Bottom line

**Proceed as the recon recommends (GO, combine-only, S), with three decisions locked into the spec:** (1) mandatory Seed-XOR-specific fingerprint gate with explicit "no built-in check" wording — it is the *only* safety net over a strictly weaker primitive, stronger framing than SLIP-39's; (2) ask N up front, first part fixes length, reject mismatches at entry, **no** length picker and **no** Trezor/verbatim interpretation fork (both unnecessary here); (3) restrict to Coldcard-interop **12/18/24** word — this needs a deliberate guard because `bip39.New` and the entry flows do *not* enforce it. Path A menu hook confirmed. Split stays deferred.
