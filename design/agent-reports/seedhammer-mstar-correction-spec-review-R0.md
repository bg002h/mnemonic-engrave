<!--
Persisted verbatim. opus-architect R0 gate of the m*1 BCH-correction spec
(SPEC_seedhammer_mstar_correction.md @ 47c9ad7). Reviewer agentId a528133472b51291c. Verdict:
NOT GREEN 0C/3I+3m. Phase A decoder VERIFIED sound (field design, constants, pipeline,
orientation pin, re-verify, per-code-const integrity — no findings). All 3 Importants are in
Phase B's md/mk realization (under-scoped): I-1 inputCodex32Flow's per-frame codex32.New + the
codex32-total length windows are ms-only -> md/mk correction non-functional for several lengths;
I-2 the confirm-gate anchor (id/thr/share via ParsePrefix) is codex32-share-schema-specific ->
unproducible for md/mk, removing the ONLY defense against engraving a wrong secret; I-3 ParsePrefix
fires spurious 'bad threshold' on md/mk while typing. Disposition: folded (HRP-aware validation/
windows/feedback; the per-position DIFF is the universal anchor, id/thr/share codex32-only).
The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — m*1 BCH correction (spec)

**Reviewer:** opus architect (adversarial R0 gate, read-only)
**Spec:** `design/SPEC_seedhammer_mstar_correction.md` (cited commit `47c9ad7`)
**Base:** fork `main` `04a1e95` (verified: `git log` HEAD = `04a1e95`, "Merge feat/seedxor-combine")
**Oracle:** `mk-codec .../bch_decode.rs`, `bch.rs`, toolkit `repair.rs`
**Date:** 2026-06-18

---

## Verification Results

**Decoder fidelity (Phase A §3) — SOUND.**
- GF(1024)=GF(32²), `ζ²=ζ+1`, built on the fork's GF(32) is a faithful, near-mechanical port. The Rust `Gf1024{lo,hi}.mul` (bch_decode.rs:163-172) uses the 4-subfield identity `lo=ll^hh, hi=lh^hl^hh`; swapping Rust's carryless `gf32_mul` for the fork's log-table `fe.Mul` (gf32.go:96-103) is justified because Rust's own `gf32_alpha_powers_match_bech32_log_inv_table` (bch_decode.rs:631-645) cross-checks the carryless powers against *exactly* the fork's `invLogTbl` (gf32.go:22-27). The SPEC mandating that cross-check as a Go conformance test (§3.1) is correct and actionable.
- Field constants port **verbatim** — I verified the fork's `fe` numbering against the Rust constants: `feG=8` ⇒ `BETA={lo:0,hi:8}`; `feE=25`, `feX=6` ⇒ `GAMMA={lo:25,hi:6}`; `feP=1` ⇒ `ONE.lo=1`. Exact match. `REGULAR_J_START=77`/`LONG_J_START=1019` (bch_decode.rs:212-217) and the β-order-93 / γ-order-1023 / ζ³=1 / generator-root self-tests (bch_decode.rs:657-708) are real and portable.
- Pipeline (syndromes→BM→Chien→Forney→re-verify) faithfully matches `decode_errors` (bch_decode.rs:550-599). Guards `deg(Λ)=0||>4` (566), Chien root-count==deg (415,572), Forney `mag.hi!=0`/`mag.lo==0` (485-492) are all present and the SPEC §3.2 reproduces them. `Correct`'s API (unique within-radius, re-verified, with `Edits`, else false) faithfully mirrors `bch_correct_*` (bch.rs:429,482 re-verify) and the `repair.rs` `(pos,was,now)` triple model — both verified by subagent against source (RepairDetail.corrected_positions = `Vec<(usize,char,char)>`, repair.rs:424-431).

**Symbol-orientation (§2.6) — correctly identified and accurately stated.** Rust `compute_syndromes` unpacks LSB-first: `coeffs[i]=(residue>>(5*i))&0x1F` ⇒ `coeffs[0]`=x⁰ (bch_decode.rs:305-306). Fork `unpackSyms` is MSB-first: `out[0]`=top 5 bits=highest power (mdmk.go:65-71); `engine.residue` index 0 = highest power (checksum.go:13-18). These are opposite array orders. The SPEC's "ONE canonical orientation + boundary conversion, pinned by a Rust parity vector" is the right mandate.

**3-layer safety model (§2) — correctly non-skippable and correctly reasoned.** The SPEC correctly states re-verify does NOT close the residual wrong-but-valid hole (a >t string decoding to a *different* valid codeword re-checksums clean — confirmed by `five_errors_either_rejects_or_returns_bogus_recovery`, bch_decode.rs:811-848) and that only the human diff-gate closes it. The new-screen / Button2-drain / never-`confirmCodex32Flow` mandate is sound (confirmCodex32Flow Button3=`IconHammer`→engrave, codex32_polish.go:122,210-213).

**Per-code-constant integrity (§2.5) — verified.** ms1 init residue=1 (`[feQ×12,feP]`, checksum.go:36-45), md/mk `POLYMOD_INIT=0x23181b3` (mdmk.go:39); targets/hi-lo splits at mdmk.go:54-63. Decode-consumes-`residue⊕target`, Rust-vectors-only, negative-cross-constant test: all correct.

**Wiring (§4.1) — claims verified TRUE by subagent.** `inputCodex32Flow` returns `(codex32.String,bool)` (gui.go:721); exactly two callers (menu `case 2` gui.go:2038, `recoverCodex32Flow` codex32_polish.go:171); `engraveObjectFlow` already has BOTH `case codex32.String:` and `case mdmkText:` (gui.go:1861-1866); keypad permits m/d/k/1 (codex32_polish.go:222); `ParsePrefix` exposes `HRP` early on partial fragments (polish.go:64,99). The menu path is already `any` (`newInputFlow` returns `(any,bool)`, gui.go:1855), so the `any` return ripples only to `recoverCodex32Flow` as the SPEC says; both resolution options are real.

---

## Findings

### CRITICAL — none.

### IMPORTANT

**I-1. The md/mk live-entry path reuses codex32/ms-specific machinery that is structurally wrong for md/mk — the SPEC under-scopes §4.1 to "dispatch + `any` return + menu relabel."** `inputCodex32Flow` is wired entirely to codex32/ms internals that do not generalize by HRP:
- **Per-frame validation** calls `codex32.New(kbd.Fragment)` (gui.go:730), which uses the codex32 short/long checksum constants (init=1, codex32 NUMS target) for *any* HRP — `New` does not gate on HRP (codex32.go:98-124). A correct `md1`/`mk1` string therefore fails `New` with `errInvalidChecksum`. The SPEC says "validate per the parsed HRP" but the file manifest only marks gui.go "modify — HRP-dispatch + `any` return; menu relabel," not the per-frame validation rework.
- **Length windows are codex32-total-length-specific.** `codex32Feedback`/`codex32StatusLine` and the "complete-but-invalid-in-window" trigger the SPEC reuses (§4.2) gate on `ShortCodeMinLength..ShortCodeMaxLength`=**48..93** and `LongCodeMinLength..LongCodeMaxLength`=**125..127** (polish.go:17-20; codex32.go:43-44). These are ms/codex32 *total string* windows. md/mk valid totals are different: `md1Regular`=24 chars (< 48), `mk1Regular`=80 (coincidentally in-window), `mk1Long`=111 (in the 94..124 dead zone). So a short `md1` and any `mk1Long` never enter the window that arms the "Fix?" trigger or even the live "bad checksum" feedback. md/mk correction — the codes the user expanded scope to include — is non-functional through the specified gate for those length classes.

  *Required fix:* §4.1/§4.2 + the file manifest must specify HRP-aware validation (dispatch `New`/`ValidMD`/`ValidMK` by parsed HRP per frame) and HRP-aware length windows / status / feedback for the md/mk brackets (regular data 14..93 ⇒ total 17..96; long data 96..108 ⇒ total 99..111; md regular data ≥13 ⇒ total ≥16; reserved 94..95 data). This is a substantive `inputCodex32Flow` rework, not a signature change.

**I-2. The mandated confirm-gate human anchor (`id · thr · share` via `codex32FieldLine`/`ParsePrefix`) is structurally inapplicable to md/mk — and it is the only layer that closes the catastrophic wrong-but-valid hazard for them.** Security invariant #3 / §2.3 / §4.2 require the diff gate to show the decoded header fields as the human-checkable anchor. `ParsePrefix` applies the **codex32 share header schema** unconditionally: `data[0]`=threshold∈{0,2..9}, `data[1:5]`=id, `data[5]`=share index (polish.go:101-138). md/mk strings do not have this layout — `md1Regular`'s data part begins `yqp...` (mdmk_test.go:13), so `data[0]='y'` ⇒ `ParsePrefix` returns `errInvalidThreshold`, and `codex32FieldLine` yields nothing usable. There is **no md/mk header/field parser in the fork** (grep: none; `mdmkFlow` engraves verbatim with no anchor, gui.go:1917-1935). So for md/mk the SPEC mandates an anchor that cannot be produced, leaving either (a) no human anchor for md/mk — weakening the *only* defense against silently engraving a wrong secret, for exactly the expanded-scope codes — or (b) a net-new md/mk header parser that is not in §3/§6.

  *Required fix:* §2.3/§4.2 must specify the md/mk confirm-gate anchor explicitly. Acceptable resolutions: define an HRP-aware anchor (e.g. md/mk header decode, if a parser is added — then scope it in the manifest), OR, if no md/mk parse exists, the gate must fall back to a still-meaningful human-checkable anchor for md/mk (at minimum the per-position `x→y` diff in card coordinates, which IS applicable, plus an explicit statement that the id/thr/share line is codex32-only). The SPEC must not leave the md/mk anchor unspecified given it is the load-bearing safety layer.

**I-3. The `ParsePrefix`-driven live feedback fires spurious errors for md/mk.** In `inputCodex32Flow`, `perr` from `ParsePrefix` is surfaced *eagerly, regardless of window* (`codex32Feedback`: `if perr != nil { return Describe(perr) }`, codex32_polish.go:56-59). For an md/mk fragment `ParsePrefix` returns `errInvalidThreshold` as soon as `data[0]` is typed (e.g. `md1y`), so the user sees a spurious "bad threshold" while typing a valid md/mk string. This is the same root cause as I-1/I-2 (codex32-only header assumptions) and must be resolved in the same §4.1 rework — the spec should call it out so the implementer does not ship a misleading live error.

### MINOR

**M-1. Length-window prose inconsistency in the recon, inherited risk.** The recon §1 says "Long code (mk1-long, 125–127)" while the decode field and mdmk.go use the *data-part* bracket 96–108 (total 99–111). The fork's 125–127 is the codex32/ms *total* long window. These are different coordinate systems; the SPEC should state windows in explicit (data-part vs total) terms to prevent the implementer conflating them (this conflation is the mechanism behind I-1). Not blocking on its own.

**M-2. Recon citation drift.** SPEC header cites recon at `71ab189`; the file is `design/cycle-prep-recon-mstar-correction.md` and its own header reads "recon time `04a1e95`, design HEAD `e1c3743`." Cosmetic; verify the commit pin before merge.

**M-3. §2.7 stack-array sizing.** The SPEC says BM "fixed-size stack arrays … max Λ degree 5." Note Λ has degree ≤4 (5 coefficients incl. constant) but BM's working `prev`/intermediate and Ω are length-8; the plan (not the spec) must size `[8]`-wide buffers for Ω/Λ′ (Rust `omega=vec![ZERO;8]`, bch_decode.rs:446). Flag for the plan, not the spec gate.

---

## Verdict

**NOT GREEN — 0 Critical / 3 Important.**

The Phase A decoder spec is sound and faithfully ported (field design, constants, pipeline, orientation pin, re-verify, per-code-constant integrity all verified against source — no findings). The block is entirely in Phase B's md/mk realization: the SPEC treats `inputCodex32Flow`'s validation, length windows, live feedback, and confirm-gate header anchor as HRP-neutral, but every one of them is codex32/ms-specific in the fork. As written, md/mk correction (the user's explicit scope expansion) is non-functional for several length classes (I-1), surfaces spurious live errors (I-3), and — most seriously — has no producible human-checkable anchor on the confirm gate (I-2), which is the spec's own designated *only* defense against silently engraving a wrong secret.

**Required fixes before GREEN:**
1. **I-1:** Rewrite §4.1/§4.2 + the file manifest to specify HRP-aware per-frame validation (`New`/`ValidMD`/`ValidMK` by parsed HRP) and HRP-aware length windows / status / feedback / "Fix?"-trigger covering the md/mk data-part brackets — not merely a signature change.
2. **I-2:** Specify the md/mk confirm-gate human anchor explicitly (HRP-aware header decode if a parser is added and scoped, or an explicit md/mk fallback to the per-position diff with a stated note that id/thr/share is codex32-only) so the load-bearing safety layer exists for md/mk.
3. **I-3:** Specify suppression of the codex32-schema `ParsePrefix` feedback for md/mk fragments in the same rework.

Fold, persist this review verbatim to `design/agent-reports/`, and re-dispatch the R0 gate (re-review after every fold — folds can introduce drift). Address M-1/M-2/M-3 opportunistically.
