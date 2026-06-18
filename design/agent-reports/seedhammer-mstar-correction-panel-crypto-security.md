<!--
Persisted verbatim. Architect panel — crypto-security lens (agentId ac3516393d63ebc65). Verdict: proceed-with-changes — non-skippable 3-layer safety (no auto-apply, mandatory re-verify, a DIFF-framed 'confirm the edit' gate showing per-position x->y + decoded header fields, Seed-XOR-fingerprint analogue); suppress any non-unique/out-of-radius suggestion entirely; subs-only v1; pin every per-code const with Rust-generated parity vectors + a negative cross-constant test.
The text below is the agent's report exactly as returned; do not edit.
-->

# Design Input — BCH Error-CORRECTION for codex32-family (`ms1`/`md1`/`mk1`) on SeedHammer II

**Lens:** crypto-correctness + safety of *correcting secret-bearing strings*. Read against `design/cycle-prep-recon-mstar-correction.md`, the constellation port source, and the fork's existing detection layer. This locks the spec's open decisions (§9) before it is written.

---

## 1. The miscorrection-beyond-radius hazard — the crux

**Reasoning.** This is the whole risk surface. The decoder is a bounded-distance decoder: it guarantees a *unique correct* answer only when the true error weight is ≤ t (= 4 substitutions). Past that, the recon's `five_errors_either_rejects_or_returns_bogus_recovery` is the documented reality — a 5+-error word can produce a degree-≤4 locator with 4 valid Chien roots and GF(32)-valid Forney magnitudes, i.e. a *structurally perfect but semantically wrong* codeword. For a normal bech32 string that is a nuisance; for one that becomes a **seed / descriptor / key** it is the catastrophic case: the device confidently engraves a different secret than the user holds, and *the engraving itself re-checksums clean*, so nothing downstream ever flags it.

Critically, **re-verify does NOT close this hole.** I read the constellation's two layered defenses and they are not equivalent in strength:

- The **algorithmic guards** inside `decode_errors` (bch_decode.rs:485-492) — `deg(Λ) > 4 ⇒ None`, `mag.hi != 0 ⇒ None`, `mag.lo == 0 ⇒ None` — reject *most* >t patterns.
- The **mandatory re-verify** after apply (`bch_correct_regular` bch.rs:429; toolkit `repair_chunk_one` repair.rs:733) recomputes the polymod over the corrected word and rejects anything that isn't a true codeword.

But a wrong-but-valid result *is a true codeword* — re-verify passes it by construction. Re-verify only catches the case where Forney produced an *inconsistent* magnitude set (the common >t outcome); it provides **zero protection** against the residual case where the 5+-error word genuinely sits within distance-4 of a *different* valid codeword. That residual probability is small but **nonzero and not bounded by re-verify**; it is bounded only by the code's minimum distance and the fraction of the 5+-error coset that decodes. Re-verify is necessary but is the wrong thing to lean on as *the* safety property.

So the safety model is correct but must be stated as **three independent, non-skippable layers**, and the burden of the residual hazard must land on the **human**, not the math:

1. **Decoder-internal guards** (port faithfully; do not soften).
2. **Mandatory post-apply re-verify** (treat skipping as Critical).
3. **Human confirmation of the *resulting string*, never auto-proceed** — this is the only layer that defends the residual wrong-but-valid case, and only if framed so the human can actually catch it.

**On "how does the user verify a high-entropy correction they can't eyeball":** they can't verify the *whole string* — and any UX that implies they can is security theater. But they don't need to. The leverage is that the **edit is tiny and local**. The user typed the string from a physical source (a plate / card they are holding). The right gate shows **exactly the diff**, in source coordinates, and asks them to confirm *the edit*, not the string:

> `pos 17: 'b' → '8'`
> `pos 43: 'k' → 'h'`
> "Did you mean this? Compare each changed position to your card."

A 1-2 char substitution diff is humanly checkable against the physical source ("position 17 on my card really is an 8, I just fingered b"). This is **directly analogous to the Seed-XOR mandatory fingerprint gate** the fork already ships: you cannot eyeball 256 bits of XOR output, so the device makes you verify a *small, derived, human-comparable* artifact (the fingerprint) before it will proceed. Here the human-comparable artifact is **the position-list diff**. The recon and the toolkit already produce exactly this — `RepairDetail.corrected_positions: Vec<(usize, char, char)>` (repair.rs:430) is the `(position, was, now)` triple — and mk-codec's `DecodedString::corrected_char_at` (bch.rs:619) surfaces the corrected character including inside the checksum region. The data structure for a faithful diff gate already exists in the port; the cycle ports it, it does not invent it.

**Recommendation.**
- **MANDATORY, enforced, Critical-if-violated:** (a) never auto-apply a correction into the engrave path; (b) always re-verify after apply; (c) the user-facing gate shows the **per-position diff in card coordinates** and requires explicit confirmation before the corrected string can be engraved.
- **Frame the gate as "confirm the EDIT," not "confirm the string."** Show `pos N: 'x'→'y'` for every changed position. Do NOT show only the full corrected string as a wall of bech32 and ask "looks right?" — that invites blind acceptance and is the failure mode this gate exists to prevent.
- **The correction screen is a distinct, new screen — it MUST NOT reuse `confirmCodex32Flow` as-is.** That flow's Button3 is `IconHammer` → straight to `backupSeedStringFlow` (engrave). A corrected string must route through a *prior* "accept this correction?" screen whose accept path lands the user back on the normal confirm screen with the now-clean string. Re-use the **Button2-drain idiom** (codex32_polish.go:106-111) so the new screen's third action can't wedge the router queue.
- Position indexing must be in **the coordinate system the user typed** (chars after `hrp1`, the toolkit's convention), and must be unambiguous about checksum-region edits (a correction can land in the 13/15 checksum chars; the diff should still show it honestly, as `corrected_char_at` does).

This is **sufficient** *only* with all three layers enforced and the diff-framed gate. A skippable re-verify or any auto-apply into engrave is **Critical**.

---

## 2. Only-suggest-when-unique-and-within-radius

**Reasoning.** BIP-93 ties the obligation to ≤4 subs / ≤8 erasures and says implementations SHOULD suggest *if possible* and MUST NOT use an unchecksummed string. "If possible" means **uniquely and within radius** — a bounded-distance decoder returns *at most one* candidate within distance t, so "unique within radius" is the decoder's native contract; the danger is when the input is *outside* radius and the decoder *guesses* anyway (the §1 residual). The port already encodes the gating I want: `decode_errors` returns `None` for `deg(Λ)==0 || deg(Λ) > 4` (bch_decode.rs:566), `None` if Chien roots ≠ deg(Λ) (the "not enough distinct roots ⇒ not a clean t-error pattern" check, bch_decode.rs:415/572), and `None` on bad magnitudes. The toolkit then surfaces `None` as `TooManyErrors { bound: 8 }` → **suppress, do not guess** (repair.rs:704). That is exactly the posture.

**Recommendation — exact gating (suppress the suggestion entirely otherwise):**
1. **Decoder returns `None`** (deg 0, deg > 4, Chien-root-count mismatch, or bad/zero Forney magnitude) ⇒ **no suggestion.** Show the existing "bad checksum" feedback (`Describe(errInvalidChecksum)`, polish.go:31) and let the user re-type. Never display a partial or best-effort guess.
2. **Decoder returns `Some` but post-apply re-verify fails** ⇒ **no suggestion** (it was a >t pattern that fooled BM). Same "bad checksum" terminal state.
3. **Decoder returns `Some` and re-verify passes** ⇒ this is the unique within-radius candidate ⇒ **offer it via the §1 diff gate.** Only here.
4. **Do not implement multi-candidate / list-decoding.** Bounded-distance decoding is single-candidate by construction; presenting "did you mean A or B?" for a secret invites the user to pick the wrong one. If it isn't uniquely correctable within radius, the answer is "re-type," full stop.

"Silence beats a guess" is already the fork's established discipline (`suggest_hrp` returns `None` when 0 or 2+ HRP neighbors exist, repair.rs:519). Apply the same principle to the BCH suggestion.

---

## 3. Subs-only vs erasures (security/transcription view)

**Reasoning.** The shipping constellation decoder is **substitutions-only** (recon §7; confirmed — the port has no erasure path, no syndrome-side erasure-locator seeding, no UX to mark a position unknown). Subs cost 4 of the distance budget per error (unknown position *and* value); erasures cost 2 (known position, unknown value) — so the same code corrects 4 subs *or* 8 erasures. The device's hand-entry reality (`inputCodex32Flow`, forced-uppercase keypad, b/i/o statically dimmed) produces almost exclusively **substitutions**: a fat-finger swaps one valid bech32 char for another. The classic erasure sources BIP-93 names — wrong case, a literal `?`, a non-bech32 char — are **already structurally impossible or already rejected** on this device: the keypad force-uppercases (no case ambiguity), can't emit `?`, and dims b/i/o. So the erasure path's natural inputs barely arise on the typed path.

**Security difference (this is the important part):** erasures are **strictly safer per unit of correction power**, because a known-position unknown-value carries *more information* — the decoder isn't guessing *where* the error is. An erasure-aware decoder that converts the user's explicitly-marked "I can't read this char" positions into erasures would (a) extend reach to damaged-plate re-reads and (b) reduce miscorrection probability for those positions. **But** there is no UX on the device today to *mark* a position as an erasure (no plate-re-read entry path exists either — recon §5), and adding an "unknown char" key + erasure-locator seeding in BM is genuine net-new crypto on the most safety-sensitive code path. Net-new crypto on the secret path is exactly where I want the *least* surface in a first cycle.

**Recommendation.** **Subs-only for v1 — port-faithful, and it matches the typed path's actual error distribution.** Defer erasures to a separate follow-up gated on a concrete consumer (a plate-re-read or partial-card-recovery entry path), where the known-position safety advantage is realized rather than hypothetical. There is **no security regression** in shipping subs-only: it is the more conservative correction power (4 vs 8), it is the already-audited algorithm, and the natural erasure triggers don't occur on this keypad. Build the decoder so an erasure-locator seed *could* be added later (the BM core is standard), but do not wire it or expose UX in v1.

---

## 4. Per-code-constant integrity

**Reasoning.** This is the documented landmine and I confirm the recon's resolution against the fork's own code. The fork's `mdmk.go` is explicit and correct: **md/mk use `POLYMOD_INIT = 0x23181b3`**, **codex32/ms1 uses initial residue `1`** (`newShortChecksum`/`newLongChecksum` residue field = `[feQ×12, feP]` = 1, checksum.go:36-45), and the NUMS targets + the 65/75-bit hi/lo splits are spelled out (mdmk.go:39, 54-63) with the bit widths and the "do NOT copy codex32's residue and only swap target" warning baked into the comment. The constellation's own history (`ms-codec/bch.rs` + the `BUG_decode_with_correction_length_divergence.md` referenced in recon §6) is the cautionary tale: pairing ms with `0x23181b3` was a real bug.

The structural safety here is strong because **the decoder operates on `residue ⊕ target`** (bch_decode.rs takes `residue_xor_const`; the fork already computes the residue via the `engine` and holds `target`). The decoder is **constant-agnostic** — it never re-derives a target or an init; it consumes the difference the verifier already produces. So the integrity question reduces to: *the verifier's existing per-code params must be the single source of truth, and the decode path must consume them, not a re-typed copy.*

**Recommendation — the discipline to mandate:**
1. **Reuse the fork's existing, parity-tested params verbatim.** The decode path computes the syndrome input from the *same* `engine` residue + `target` the `verifyMDMK` path uses (checksum.go / mdmk.go). **No second copy** of POLYMOD_INIT, targets, generators, length brackets, or hi/lo splits anywhere in the new decode code. One definition, both paths.
2. **Pin everything with Rust-generated parity vectors — never Go-self-generated.** This is non-negotiable and is the §6/§10 finding: a Go-self-generated vector would pass even if Go and its own vector shared the same wrong constant (the false-consensus class). The golden vectors (corrupt-then-correct, with known positions + magnitudes) must originate from the Rust codecs' `bch_decode` tests (`ms/md/mk-codec .../tests/bch_decode.rs`, `bch_adversarial.rs`).
3. **Add three things the recon's "reuse existing params" doesn't fully cover, and that I'd treat as spec-mandatory:**
   - **A negative cross-constant vector per code:** a string valid under *one* HRP's target must fail to "correct-and-verify" under another's — the explicit regression test for the ms-init/md-init mixup. (E.g. an `ms1` string fed to the `md` constants must not produce a clean correction.)
   - **The reserved-invalid length band [94,95] and out-of-range must reject *before* decode runs** (the fork's `ValidMK` already gates this, mdmk.go:145; the toolkit gates it as `ReservedInvalidLength`, repair.rs:629). The decode path must inherit the same length gate, not just the verify path — otherwise a length-confused input could pick the wrong generator.
   - **`α`-element / `j_start` / generator-window self-test as a build-time assertion** (the port's `beta_has_order_93`, `gamma_has_order_1023`, `generator_polynomial_evaluates_to_zero_at_specified_roots` tests, bch_decode.rs:657-708). Port these as conformance tests so a GF(1024) construction error can't ship silently. **Note the field-impl decision (recon §9.4):** the fork has a *log-table* GF(32) (`fe.Mul`, gf32.go:96) while the Rust port uses *carryless* GF(32) (`gf32_mul`, bch_decode.rs:102). Whichever GF(1024) base is chosen, cross-check the GF(32) layer against the fork's existing `invLogTbl` (the port already does this — `gf32_alpha_powers_match_bech32_log_inv_table`, bch_decode.rs:631) so the two field implementations are proven identical before GF(1024) is built on top.

---

## 5. Top security risks, ranked

**Risk 1 — Wrong-but-valid miscorrection silently engraved (the §1 residual). [HIGHEST]**
A >t-error input decodes to a *different valid codeword* → device engraves a wrong secret that re-checksums clean. Re-verify does not bound this; it is bounded only by min-distance + human review.
*Mitigation:* enforce the three-layer model (§1) with the **diff-framed confirm gate** ("confirm the EDIT," per-position `'x'→'y'`, compare to your card), never auto-apply, mandatory re-verify, and the only-unique-within-radius suppression (§2). Distinct correction-accept screen, not a re-skinned engrave confirm.

**Risk 2 — Per-code constant drift (init/target/hi-lo) on the decode path. [HIGH]**
The documented landmine; a single wrong constant mis-corrects every string of that code, and a Go-self-generated test would not catch it.
*Mitigation:* single source of truth (reuse the verifier's params; decoder consumes `residue ⊕ target`), Rust-generated parity vectors only, plus the negative cross-constant vector and the GF-layer cross-check (§4).

**Risk 3 — GF(1024) net-new field arithmetic defects. [MEDIUM]**
GF(1024)=GF(32²) is the main net-new crypto piece (recon §2); a subtle multiply/reduction/`inv` bug surfaces only on multi-error inputs and could turn a correctable string into a wrong correction.
*Mitigation:* port the field self-tests as build-time conformance (ζ³=1, β order 93, γ order 1023, generator-root checks), cross-check the GF(32) base against the fork's `invLogTbl`, and gate the cycle behind the project's mandatory adversarial post-implementation review over the whole diff (CLAUDE.md per-phase policy) — TDD against golden vectors plus the independent execution review, since this is precisely the "implementation-introduced regression TDD misses" case.

**Decomposition note (concur with recon §8):** Phase A (pure decoder: GF(1024) + BM/Chien/Forney + unique-within-radius + mandatory re-verify, TDD against Rust vectors, own R0/review) gated *before* Phase B (the diff-confirm UX in `inputCodex32Flow`, own gate). Scope to the **typed ms1/codex32 path** (recon §5) — md/mk arrive only via error-free NFC today, so their correction is moot until a typed md/mk entry path exists; build the decoder parameterized so it's *ready* but don't add the md/mk GUI in this cycle.

---

## Bottom line

**Proceed-with-changes:** GO on the decomposed, ms1-typed-scoped cycle — but the spec MUST make the three-layer safety model non-skippable (no auto-apply, mandatory re-verify, and a **diff-framed "confirm the edit" gate** modeled on the Seed-XOR fingerprint discipline), suppress any non-unique/out-of-radius suggestion entirely, ship subs-only for v1, and pin every per-code constant with **Rust-generated parity vectors plus a negative cross-constant test** — with the mandatory adversarial post-implementation review treated as a hard gate, not deferrable.
