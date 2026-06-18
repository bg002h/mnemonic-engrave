# cycle-prep recon — 2026-06-18 — seedhammer-mstar-correction

**Fork `main` SHA at recon time:** `04a1e95` (Seed XOR shipped). Design repo HEAD: `e1c3743`.
**Slug:** `seedhammer-mstar-correction` — teach the SeedHammer fork to **error-CORRECT** mistyped
`md1`/`mk1`/`ms1` (codex32-family BCH) strings, not just detect — porting the constellation's
existing BCH decoders. The device's codex32 layer is detection-only today.

Recon = three parallel agents (constellation port-source; authoritative BIP-93/codex32 facts;
fork integration). BCH facts verified against **BIP-93** + the reference impls + the constellation
code, per the verify-external-protocol-facts rule.

---

## Verdict: **GO — but DECOMPOSE (Phase A decoder / Phase B UX) and SCOPE to the typed ms1/codex32 path.** Architect panel before spec (user-endorsed).

A genuine **crypto-decoder cycle** (new BCH error-correction over GF(1024)), touching
secret-bearing strings → the project's full gated treatment + the panel. Net-new ≈ a BCH decoder
(~300–500 LoC) + a **GF(1024)=GF(32²) extension field** (~100–200 LoC) + a "suggest→confirm" UX
(~100–200 LoC). It's an **extension** of the fork's existing, parity-tested codex32 layer (reuses
GF(32), the polymod engine, the per-code params), not a from-scratch field impl.

---

## 1. The correction guarantee (BIP-93, quoted/verified)

- **Short code (≤93 chars — the ms1/md1/mk1-*regular* case):** correct **up to 4 character
  substitutions**, **up to 8 erasures** (known-position unknowns), or **up to 13 consecutive
  erasures (burst)**. **Long code (mk1-long, 125–127):** 4 subs / 8 erasures / 15 burst.
- **Detection** (the status quo): any error in ≤8 chars guaranteed; <3e-20 (short)/3e-23 (long)
  false-accept beyond.
- **Min distance d=8** is *inferred* from the `BCH(93,80,8)`/`BCH(108,93,8)` params + the
  8-consecutive-root generator window — **BIP-93 prose does NOT print `d` numerically** (don't
  cite "BIP-93 states d=8"; cite the construction).
- `t=4` from `2t<d`; "we only guarantee to correct 4 characters no matter how long the string."

## 2. The algorithm + decode field (verified definitively)

BIP-93 deliberately does **not** specify a decode pipeline — established from the reference impls
(`rust-bitcoin/bech32 primitives::correction`, the `codex32` crate) + the constellation decoder:
**syndromes → Berlekamp-Massey (error-locator Λ) → Chien search (positions) → Forney (magnitudes)
→ apply → RE-VERIFY.** The decode runs over the **extension field GF(1024)=GF(32²)**
(`ζ²=ζ+1`), NOT GF(32) — confirmed in BIP-93's Mathematical Companion + the code (β order 93
regular / γ order 1023 long; 8-syndrome window; `deg(Λ)>4 ⇒ reject`). Error *magnitudes* are
GF(32). **The fork has only GF(32) — GF(1024) is the main net-new crypto piece.**

## 3. ONE parameterized decoder serves all three m\*1

The constellation's mk-codec `decode_errors` is a single constant-agnostic core; the toolkit's
`repair.rs` already drives it for ms/md/mk by XOR-ing the per-code target. **md1/mk1 INHERIT
codex32's BCH code** — *identical generator* (`GEN_REGULAR`/`GEN_LONG` = the BIP-93 canonical
values, unit-tested), same `d=8`, same `t=4`/8-erasure capacity. They differ **only** in HRP +
target-residue constant (and, per §6, the initial residue). So: **port ONE parameterized
decoder.**

## 4. The "suggest → confirm" rule (BIP-93, quoted) + the safety landmines

- BIP-93: *"a string without a valid checksum MUST NOT be used"*; *"implementations SHOULD provide
  a corrected valid string if possible … SHOULD NOT automatically proceed with a corrected string
  without user confirmation."* Suggestions tied to the guaranteed radius (≤4 subs / ≤8 erasures).
- **Miscorrection beyond radius (the catastrophic case):** a 5+-error string can yield a bogus
  degree-≤4 locator with valid roots → "correct" to a **different valid codeword** (a wrong seed/
  descriptor/key). The constellation's decoder + tests acknowledge this
  (`five_errors_either_rejects_or_returns_bogus_recovery`).
- **Mandatory defensive RE-VERIFY:** `bch_correct_*` re-runs the verifier after applying and
  rejects any non-valid result. **Skipping the re-verify ships the wrong-but-valid bug.** The Go
  port MUST replicate it. Use the toolkit `repair.rs` model (return positions + was/now, re-verify)
  — NOT mk-codec's silent auto-applying `decode_string`.
- For seed material a silent wrong-correction is catastrophic → "suggest + confirm + re-verify",
  never auto-apply, is the only safe posture.

## 5. SCOPE finding — correction is valuable on the TYPED path; md/mk are NFC-only

- **`ms1`/codex32 — HAND-TYPED** (`inputCodex32Flow` via the CODEX32 menu; the codex32 keypad).
  Typos happen → correction is **genuinely valuable** here. (Recovery-share entry routes through
  the same flow → benefits too.)
- **`md1`/`mk1` — NFC-ONLY** (`gui/scan.go:70` `ValidMD/ValidMK` → `mdmkText`). **No hand-typed
  md/mk entry path exists.** NFC is a digital, framed, error-free transport → transcription-
  correction is **moot for md1/mk1 as they arrive today.** Making md/mk correction useful requires
  **first ADDING a typed md/mk entry path** (new menu + keypad + live-gate) — net-new GUI,
  independent of the decoder, with no current user.
- **Recommendation:** scope the cycle to **correct the typed ms1/codex32 path**; treat **typed
  md/mk entry as a separate, optional follow-up**. (The decoder is built parameterized so md/mk
  correction is *ready* the day a typed md/mk entry lands.)
- Erasure/damaged-plate re-read angle: BIP-93's 8-erasure capacity could help re-reading a
  partially-unreadable engraved plate, but **no plate re-entry path exists**, so that benefit is
  hypothetical without new scope.

## 6. The #1 porting landmine — the per-code initial residue (RESOLVED here; verify in spec)

The two recon agents **disagreed** on `POLYMOD_INIT` (the verify-facts class). Resolved against
the fork's own parity-tested code: **codex32/ms1 uses initial residue `1`** (`codex32/checksum.go`
short residue `[q×12, p]` = 1), **md/mk use `0x23181b3`** (`codex32/mdmk.go:39`). Pairing ms with
`0x23181b3` was a documented constellation bug (`ms-codec/bch.rs` + `BUG_decode_with_correction_
length_divergence.md`). The mk1 65-bit/75-bit target hi/lo splits are already in `mdmk.go:54-63`.
**The fork's existing per-code params are correct + parity-tested**, and the decoder *reuses* them
(it operates on `residue ⊕ target`, which the fork already computes) — so the risk is contained,
but the spec must state the per-code init/target table exactly and pin it with **Rust-generated
parity vectors, never Go-self-generated**.

## 7. What the constellation decoder does NOT do (net-new if wanted)

- The **shipping** mk-codec v0.4.0 decoder is **substitutions-only** — **no erasure path** (0 of
  the 8-erasure guarantee), despite BIP-93 recommending wrong-case/`?`/non-bech32 → erasure
  conversion. Erasures (known position, worth 8) vs substitutions (unknown position, worth 4) are
  different operations; the hand-typed device naturally presents *substitutions*. **Decide:**
  subs-only (port-faithful, simpler) vs also build erasure-decode + the wrong-case/`?`-as-erasure
  UX (net-new, but matches BIP-93's transcription-friendly intent and the device's hand-entry
  reality).
- The constellation CLI is **pristine-only** (rejects any string needing correction —
  `me-cli/validate.rs MkCorrected`), and PR2's firmware md/mk path is verify-only. So **correction
  is a genuine net-new capability** relative to what ships.

## 8. Reuse vs net-new + decomposition

**Reuse (exists, parity-tested):** GF(32) (`gf32.go`), polymod engine (`checksum.go`), all
per-code params incl. `POLYMOD_INIT`/NUMS targets/generators/length-brackets/hi-lo-splits
(`checksum.go`+`mdmk.go`), the fail-soft parser + `Describe` feedback (`polish.go`), the codex32
keypad, and the confirm/error/choice-screen idioms incl. the **Button2-drain** pattern.
**Net-new:** GF(1024) field (~100–200), the BCH decoder syndromes+BM+Chien+Forney+re-verify
(~300–500), the suggest→confirm UX in `inputCodex32Flow` (~100–200), (optional) erasure path,
(optional, separate) typed md/mk entry (~150–250).

**Decomposition (both agents):** **Phase A** = the crypto decoder port (pure `codex32` package:
GF(1024) + BM/Chien/Forney + unique-correction-within-radius + mandatory re-verify; TDD against
Rust golden vectors — the high-risk, authoritative-bound piece, own R0/review/merge). **Phase B**
= the suggest→confirm UX wired into `inputCodex32Flow` (the typed ms1/codex32 path), own gate.

## 9. Open questions for the architect panel (convening next, user-endorsed)

1. **Scope:** ms1-typed-only (recommended) vs all-three (forces a typed md/mk entry prerequisite).
2. **Subs-only vs +erasures** (the BIP-93 wrong-case/`?` transcription UX) — port-faithful vs
   net-new but more useful on a hand-typed device.
3. **The miscorrection-beyond-radius safety model** — mandatory re-verify + suggest-confirm; how
   to frame the "Did you mean?" so a user can't accept a wrong correction blindly (cf. the Seed-XOR
   fingerprint-gate reasoning); only-suggest-when-unique-within-radius.
4. **GF(32) impl** — build GF(1024) on the fork's existing log-table `fe.Mul` (reuse audited
   field) vs the Rust's carryless-multiply.
5. **Decomposition + per-phase gating** confirmation.

## 11. Architect-panel synthesis — LOCKED decisions (2026-06-18, 3 lenses)

Panel (crypto-security / firmware-resource / design-decomposition) persisted verbatim to
`design/agent-reports/seedhammer-mstar-correction-panel-*`. Unanimous: **proceed, decomposed and
ms1-scoped, with a non-skippable safety model.** Locked decisions for the spec:

- **Decompose, strict A→B, each its own gate.** **Phase A** = a pure, parameterized `codex32`
  decoder package (GF(1024) + syndromes→Berlekamp-Massey→Chien→Forney→**mandatory re-verify**,
  subs-only, **unique-within-radius or return nothing**), TDD against **Rust-generated parity
  vectors**, **merged DORMANT** (no caller — the SLIP-39-D1 precedent; covers all three m\*1 by
  construction). **Phase B** = the suggest→confirm UX on the typed ms1/codex32 path. B calls A's
  API, so A must land first.
- **Build `Gf1024{lo,hi}` on the fork's audited GF(32) `fe.Mul`** (~0 new tables, no `math/big`,
  no 128-bit; decoder internals `uint8`/`uint16`, `uint64` only at the residue boundary).
- **The three-layer safety model is MANDATORY (Critical if violated):** (1) decoder-internal
  guards (`deg(Λ)>4`/root-count/bad-magnitude → reject — port faithfully); (2) **re-verify after
  apply**; (3) **human confirmation of the resulting string, never auto-apply.** Re-verify does
  NOT close the residual "wrong-but-valid" hole (a >t-error string can decode to a *different
  valid* codeword that re-checksums clean) — only the human gate does, and only if framed right.
- **The confirm gate must show the DIFF, not the blob:** per-position `pos N: 'x'→'y'` (≤4 lines)
  + the decoded header fields (`id · thr · share`) as the human-checkable anchor — the Seed-XOR-
  fingerprint discipline (you can't eyeball a high-entropy string, but you CAN check a 1–2-char
  edit against your card). A **new** screen (NOT `confirmCodex32Flow`, whose Button3 engraves);
  Button2-drain; accept → replace fragment → re-validate through the **existing** `New`/OK path;
  reject → keep editing.
- **Run the decoder ON-DEMAND, never per-keystroke:** a "Fix?" affordance shown only when the
  string is **complete-but-invalid-in-a-valid-length-window** (mirrors `codex32Feedback`'s
  suppress-until-window discipline); a mid-typed prefix would yield flickering garbage.
- **Suppress entirely** when the decoder returns nothing, or the re-verify fails, or the
  correction isn't unique within radius — show the existing "bad checksum", let the user re-type.
  **No multi-candidate / "A or B?" for secret material.**
- **Subs-only for v1.** Erasures (`?`/wrong-case → erasure) need marking-UX + a damaged-plate
  re-read path that doesn't exist; the forced-uppercase, b/i/o-dimmed keypad makes substitutions
  the only realistic error anyway. Keep the BM core erasure-amenable; defer the path.
- **Per-code-constant integrity:** ONE source of truth — the decode path consumes the
  verifier's existing parity-tested `residue ⊕ target` (no second copy of `POLYMOD_INIT`/targets/
  hi-lo). Pin with **Rust-generated vectors only**, plus a **negative cross-constant test** (an
  `ms1` string must NOT "correct-and-verify" under the `md` constants) and the field self-tests
  (β order 93, γ order 1023, generator roots) as build-time conformance. Add
  `tinygo build -target=pico-plus2` of `codex32` to CI (the Slice-1 lesson).
- **Scope v1 = ALL m\*1, typed entry + correction (USER DECISION 2026-06-18 — overrode the
  panel's ms1-only recommendation).** Phase A's decoder already covers all three; Phase B adds a
  **typed md/mk entry** so md1/mk1 correction is reachable on-device too, not just ms1. The
  panel's ms1-only was a cost-benefit call (md/mk arrive only over error-free NFC, no current
  typed user); the user wants full all-m\*1 typed correction. **Cleanest realization:** md1/mk1
  are codex32-family bech32 strings → the existing codex32 keypad charset serves all three, so
  Phase B becomes an **HRP-dispatched typed-entry**: the user types `ms1…`/`md1…`/`mk1…`; validate
  per parsed HRP (`codex32.New` for ms1, `ValidMD`/`ValidMK` for md/mk); the shared decoder
  corrects per-HRP via the same suggest→confirm gate; engrave routes by type (`engraveCodex32`
  for ms1, `mdmkFlow` for md/mk — the existing `engraveObjectFlow` dispatch). Erasure UX still
  deferred (subs-only v1).

## 10. TDD oracle
The constellation's BCH-correction integration tests (`ms/md/mk-codec .../tests/bch_decode.rs`,
`bch_all_lengths.rs`, `bch_adversarial.rs` — corrupt-then-correct with known positions/magnitudes)
+ the fork's existing clean codewords (`codex32/mdmk_test.go` `md1Regular`/`mk1Regular`/`mk1Long`)
to corrupt. **Rust-sourced parity vectors only** (never Go-self-generated) — the per-code-const
landmine (§6).
