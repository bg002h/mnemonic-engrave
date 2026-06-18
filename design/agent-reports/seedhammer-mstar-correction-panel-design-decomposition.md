<!--
Persisted verbatim. Architect panel — design/decomposition/UX lens (agentId aaff8859a383ef80f). Verdict: proceed, decompose Phase A (pure parameterized subs-only decoder + mandatory re-verify, TDD vs Rust vectors, merged DORMANT — SLIP-39-D1 precedent) -> Phase B (suggest->confirm UX on the typed ms1/codex32 path); scope v1 to ms1-typed, defer typed-md/mk-entry + erasures as FOLLOWUPs; 'Fix?' affordance only when complete-but-invalid-in-window, never live.
The text below is the agent's report exactly as returned; do not edit.
-->

# Design Brief — BCH Error-Correction for `m*1` Strings (suggest → confirm)

**Inputs read:** recon `design/cycle-prep-recon-mstar-correction.md` (§1–10); `gui/gui.go` `inputCodex32Flow` (:721, per-frame `codex32.New`/`ParsePrefix` + `codex32Feedback` hook :783–787); `gui/codex32_polish.go` `confirmCodex32Flow` (:83, Button2-drain :106–111, `codex32Feedback`/`Describe` :56). Lens: scope, decomposition, UX.

---

## Q1 — Scope: ms1-typed-only vs all-three

**Reasoning.** The recon's §5 finding is decisive and I see no counter-argument. `md1`/`mk1` arrive over NFC only (`gui/scan.go` `ValidMD/ValidMK` → `mdmkText`); NFC is a framed, error-free digital transport, so transcription correction has **no error to correct** for md/mk *as they arrive today*. Correction is only valuable where humans introduce substitutions — the hand-typed `ms1`/codex32 path (`inputCodex32Flow`), which recovery-share entry also routes through (`recoverCodex32Flow` :171 calls `inputCodex32Flow`), so the recovery flow inherits the benefit for free. "Build typed md/mk entry now" would mean shipping a net-new menu + keypad + live-gate (~150–250 LoC of GUI per §8) **for which there is no user** — speculative scope that delays the genuinely-useful piece.

**What "all m\*1 correction" actually requires** (so the trade is explicit): (a) the parameterized decoder — which we build anyway and which already serves md/mk by construction (§3: one constant-agnostic core, md/mk *inherit* codex32's BCH code, differing only in HRP + target residue, with the per-code init/target table from §6); plus (b) a **net-new typed md/mk entry path** — new menu entry, a keypad (md/mk alphabet differs from the codex32 bech32 keypad), a live length/structure gate, and its own confirm screen. (a) is in scope; (b) is the only incremental cost, and it has no consumer.

**Recommendation.** **Scope v1 to correcting the typed ms1/codex32 path.** Build the decoder parameterized (per §3/§8) so md/mk correction is *dormant-ready* the day a typed md/mk entry path lands. Treat typed md/mk entry as a **separate, optional follow-up** (own FOLLOWUP slug), explicitly deferred — not silently dropped.

---

## Q2 — Decomposition + gating

**Reasoning.** Phase A (the pure `codex32` decoder package: GF(1024)=GF(32²), Berlekamp-Massey → Chien → Forney → apply → **mandatory re-verify**, unique-within-radius) is the high-risk, authoritative-bound piece (§2, §4, §6). It is provable in isolation against Rust golden vectors (§10), has zero GUI surface, and carries the catastrophic-miscorrection landmine (§4: a 5+-error string yielding a bogus valid codeword → wrong seed). Phase B (suggest→confirm UX in `inputCodex32Flow`) is pure orchestration over Phase A's pure API and is meaningless to merge without it. This is a textbook crypto-core / UX-shell split.

**Is Phase A independently mergeable/useful?** **Yes — exactly the SLIP-39 D1 dormant-decoder precedent.** A pure decoder package with a Rust-parity test suite is mergeable, reviewable, and valuable on its own: it locks the highest-risk code behind its own R0/review/merge gate *before* any UX exists to misuse it, and it is the shared engine that a future typed-md/mk path (Q1) reuses verbatim. Ship it dormant (no caller) with full tests.

**Order forced?** **Yes, strictly A→B.** Phase B's "suggest" call and its accept/re-validate loop are literally calls into Phase A's API; you cannot TDD or even type-check B against a non-existent decoder. The gating is the project standard: each phase gets its own R0 (0C/0I) gate, persisted verbatim to `design/agent-reports/`, with the mandatory post-implementation adversarial review over each phase's diff.

**Recommendation.** **Confirm the A→B split as proposed.** Phase A = pure decoder package, TDD against Rust-sourced parity vectors (never Go-self-generated — §6, §10), merged dormant. Phase B = suggest→confirm UX on the typed path, gated separately. One caveat: the GF(1024) field-construction decision (§9 Q4 — build on the fork's audited log-table `fe.Mul` vs port the Rust carryless-multiply) belongs **inside Phase A's spec**, decided at A's R0, not left open.

---

## Q3 — The suggest→confirm UX (the core design)

This is where I'll be most opinionated, because the failure mode here is **a user accepting a wrong-but-valid correction to a string they cannot eyeball** — i.e. engraving the wrong seed.

**When does the suggestion appear?** **Not live.** A per-keystroke "did you mean?" is actively harmful: mid-typing, every prefix is "invalid," the decoder would fire on garbage, and a partially-typed string is exactly the wrong input to a fixed-radius decoder (it'll "correct" toward noise). The existing feedback hook already encodes this discipline — `codex32Feedback` (:56–67) deliberately **suppresses the `New` checksum error until the fragment is in a valid length window**, precisely so a half-typed string isn't flagged. Mirror that: **offer correction only when the string is complete-but-invalid** — i.e. length is in a valid window (`ShortCodeMinLength..ShortCodeMaxLength` or the long window) AND `nerr != nil`. At that exact state, today the OK button is hidden (`valid` is false at :737/:790). So the natural trigger is a **dedicated action that appears only in the complete-but-invalid state** — a "Fix?" affordance — rather than firing on OK (OK is gated on `valid` and never reachable while invalid) and rather than running on every frame.

**Concrete mapping against `inputCodex32Flow`:**
- The per-frame parse already exists (`share, nerr := codex32.New(...)` at :730). Add a sibling computation **only in the complete-but-invalid state**: `inWindow && nerr != nil`. Computing the decode every frame is wasteful and could lag the keypad; gate it behind the explicit "Fix?" press (Phase A's decode is the expensive call), not the render loop.
- When invalid-in-window, the nav currently shows Back only (OK hidden). **Add a third affordance in that state** — a "Fix?" button (mirror how `valid` conditionally adds the OK nav at :790–793). On press, call Phase A's decoder.
- Decoder returns one of: **no correction within radius** → show a transient feedback line via the existing `addLine`/`Describe` channel ("No fix within 4 changes — check your typing"); keep editing. **A unique correction within radius** → route to a **confirm screen**.

**How to present a correction the user can't eyeball.** This is the crux. The string is high-entropy; a human cannot verify a 48–93-char bech32 blob is "the right one." So the confirm screen must make the **delta** legible, not the whole string:
- Show the **changed positions explicitly**: `pos 17: 'b' → '8'`, one line per substitution (≤4 lines, which fits the existing centered-line body layout in `confirmCodex32Flow` :131–135).
- Show the **decoded header fields** of the corrected string (`codex32FieldLine` / `ParsePrefix`: `id NAME · thr 2 · share C`) — this is the one part a user *can* sanity-check against what they intended to type, analogous to how Seed-XOR uses the fingerprint as the human-checkable anchor.
- Require **explicit accept** on a **mandatory confirm gate** modeled on the Seed-XOR recovery framing and rendered with the **Button2-drain idiom** (`codex32_polish.go` :106–111): always drain the unused button event so an unconsumed event can't block the router queue head in direct-call context (the R0 C1 reason cited at :107). Buttons: Button1 = reject/keep-editing, Button3 = accept-correction. Frame the title as a question, not a statement — "Apply this correction?" — so accept is a deliberate act.
- **On accept:** replace `kbd.Fragment` with the corrected string and **re-run `codex32.New`** (Phase A already re-verified internally, but re-validating through the same `New` path the OK button trusts means the now-valid string flows through the *unchanged* accept path at :737 — no special-cased "trust me" branch). Then the normal OK→`confirmCodex32Flow`→engrave path takes over. **On reject:** discard the suggestion, return to editing with the fragment untouched.
- **Never auto-apply** (BIP-93 §4: "SHOULD NOT automatically proceed … without user confirmation"; for seed material a silent wrong-correction is catastrophic). The re-verify in Phase A is non-negotiable (§4: skipping it ships the wrong-but-valid bug); the confirm gate is the second line of defense against the within-radius-but-wrong-codeword case.

**Recommendation.** Trigger = explicit "Fix?" affordance, shown **only when complete-but-invalid-in-window**, never live. Present = changed-positions delta + decoded header fields + full corrected string, on a mandatory Button2-drain confirm gate framed as a question. Accept replaces the fragment and re-validates through the existing `New`/OK path; reject keeps editing.

---

## Q4 — Subs-only vs erasures (UX)

**Reasoning.** Erasures buy a higher guarantee (≤8 vs ≤4, §1) but require the user to **mark unknown positions** — a `?` key on the codex32 keypad, plus the BIP-93-recommended wrong-case/non-bech32 → erasure conversion (§7). That is net-new keypad UI, net-new decoder mode (the shipping constellation decoder is **substitutions-only**, §7 — so the erasure path is also net-new *crypto*, not just UX), and a conceptual burden ("mark the characters you're unsure of") on a tiny-screen device. Critically, **the hand-typed device naturally presents substitutions, not erasures** (§7): a user who mistypes produces a wrong character at a known-to-them-but-unmarked position, not a flagged blank. The erasure model fits a *damaged-plate re-read* scenario — but §5 confirms **no plate re-entry path exists**, so that benefit is hypothetical without yet more new scope. The whole erasure feature is solving a problem the v1 user doesn't have.

**Recommendation.** **Ship subs-only for v1.** The user fixes the flagged string by re-reading their source and the decoder finds ≤4 subs — which covers the realistic typo count for a hand-typed string. Keep the decoder's internal structure erasure-amenable (it's the same syndrome machinery) so a `?`-as-erasure UX is an additive follow-up if a damaged-plate re-entry path is ever built. Defer erasures explicitly to FOLLOWUPS, with the rationale (no plate-re-read consumer) recorded so it isn't re-litigated.

---

## Q5 — Top risks, decomposition, bottom line

### Top 3 design/scope risks (ranked)

1. **Miscorrection beyond the guaranteed radius → wrong-but-valid seed engraved (catastrophic).** A 5+-error string can produce a bogus degree-≤4 locator with valid roots and "correct" to a *different valid codeword* (§4; the constellation's own `five_errors_either_rejects_or_returns_bogus_recovery` test). On a device that engraves seed material, this is the worst outcome. **Mitigation:** (a) Phase A **mandatory internal re-verify** after applying (port the toolkit `repair.rs` model, **not** mk-codec's silent auto-applying `decode_string` — §4); (b) the Phase B confirm gate surfaces the **decoded header fields** as a human-checkable anchor (Q3) so the user has *something* to verify even though the body is unreadable; (c) suggest only a **unique correction within radius**, never a guess. Re-verify and the confirm gate are independent layers — both required.

2. **Per-code initial-residue / target-constant landmine (false-consensus class).** The two recon agents *disagreed* on `POLYMOD_INIT` (§6) — exactly the "plausible-but-wrong fact" failure the project's verify-external-facts rule guards against. ms1 uses initial residue `1`; md/mk use `0x23181b3`; pairing ms with the md/mk constant was a real constellation bug. **Mitigation:** the spec must state the per-code init/target table *exactly* and pin it with **Rust-generated parity vectors, never Go-self-generated** (§6, §10). Even though v1 is ms1-only, build/verify the table for all three now (the decoder is parameterized) so the dormant md/mk path is correct-by-test the day it's wired.

3. **Scope creep into speculative net-new GUI (typed md/mk entry; erasure marking).** Both are tempting "while we're here" additions; both serve no current user (Q1, Q4). **Mitigation:** hard-scope v1 to ms1-typed subs-only correction; record typed-md/mk-entry and erasure-UX as explicit deferred FOLLOWUPs with rationale, so they're conscious deferrals, not gaps. *(Lesser, noted: live-decode performance on the keypad render loop — mitigated structurally by Q3's "Fix?"-gated decode, never per-frame.)*

### Recommended decomposition

- **Phase A — pure `codex32` decoder package (own R0 → review → merge, dormant).** GF(1024)=GF(32²) (field-construction choice decided *at A's R0*, §9 Q4), syndromes → Berlekamp-Massey → Chien → Forney → apply → **mandatory re-verify**, unique-correction-within-radius, **substitutions-only**. Parameterized over all three codes (per-code init/target table per §6). TDD against **Rust-sourced golden vectors** (§10). Ships: a tested, dormant decoder + the exact per-code constant table. *Mergeable and useful standalone (SLIP-39 D1 precedent).*
- **Phase B — suggest→confirm UX in `inputCodex32Flow` (own gate), ms1/codex32 typed path only.** "Fix?" affordance shown only when complete-but-invalid-in-window; mandatory Button2-drain confirm gate showing changed-positions delta + decoded header fields; accept → replace fragment → re-validate through the existing `New`/OK path; reject → keep editing. Recovery-share entry inherits this for free via `recoverCodex32Flow`.
- **Deferred follow-ups (explicit, not dropped):** typed md/mk entry path (unblocks dormant md/mk correction); erasure-decode + `?`-as-erasure UX (unblocked by a future damaged-plate re-read path).
- **Order: strictly A → B** (B's code calls A's API; cannot TDD B without A).

### Bottom line

**Proceed, decompose as follows:** Phase A (pure parameterized subs-only BCH decoder with mandatory re-verify, TDD against Rust parity vectors, merged dormant) → Phase B (suggest→confirm UX on the typed ms1/codex32 path only, with a "Fix?"-gated, delta-displaying, mandatory confirm gate) — scoping v1 to ms1, deferring typed-md/mk-entry and erasure-UX as recorded follow-ups.
