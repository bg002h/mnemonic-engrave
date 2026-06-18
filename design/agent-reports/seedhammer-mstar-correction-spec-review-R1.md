<!--
Persisted verbatim. opus-architect R1 gate of the m*1 BCH-correction spec
(SPEC_seedhammer_mstar_correction.md @ 5b7b7b3, after folding the R0 findings). Reviewer agentId
ab1ec5be1c3463acc. Verdict: GREEN 0C/0I. Re-review after the R0 fold (folds can introduce drift):
all three R0 Importants verified closed against fork source 04a1e95 — I-1 (HRP-dispatched per-frame
validation + dual-coordinate per-HRP window table, both real hazard cases md1Regular=24<48 and
mk1Long=111-in-dead-zone now armed), I-2 (ParsePrefix confirmed to fail on md/mk data so the header
line is genuinely ms-only; the per-position diff confirmed HRP-neutral/producible so the load-
bearing anchor exists for md/mk), I-3 (eager perr surface localized + gated to ms). Folds M-1/M-3
verified; no drift (cross-refs resolve, manifest matches §4.1/§4.2, Phase A §3 undisturbed); no new
C/I. Disposition: GREEN — proceed to the Phase A plan + its own R0 gate.
The text below is the agent's report exactly as returned; do not edit.
-->

# R1 GATE REVIEW — m*1 BCH correction (spec)

**Reviewer:** opus architect (adversarial R1 re-review, read-only)
**Spec:** `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_mstar_correction.md` (folded; reviewed as-on-disk)
**Prior R0:** `design/agent-reports/seedhammer-mstar-correction-spec-review-R0.md` (NOT GREEN, 0C/3I+3m)
**Authoritative base:** fork `04a1e95` (verified: `git log --oneline -1` → `04a1e95 Merge feat/seedxor-combine`)
**Date:** 2026-06-18

---

## Verification Results

### Task 1 — §4.1(b) window-table numbers vs source bytes

I verified every cell of the §4.1(b) table (spec lines 161-166) against the fork source:

- **`ms` short total 48..93** — `shortCodeMinLength = 48`, `shortCodeMaxLength = 93` (`codex32/codex32.go:41-42`), re-exported `ShortCodeMinLength/MaxLength` (`codex32/polish.go:17-18`). ✔ matches table.
- **`ms` long total 125..127** — `longCodeMinLength = 125`, `longCodeMaxLength = 127` (`codex32.go:43-44`); `LongCodeMin/MaxLength` (`polish.go:19-20`). ✔
- **`md` regular data ≥13 ⇒ total ≥16** — `ValidMD` calls `verifyMDMK(..., mdmkShortSyms)` with `mdmkShortSyms = 13` (`mdmk.go:41,124-126`); `verifyMDMK` rejects `len(data) < n` i.e. `< 13` (`mdmk.go:100`), and the doc-comment confirms md-codec "applies no further data-part length bracket" (`mdmk.go:122-123`). Data ≥13 ⇒ total = data+3 ≥ **16**. ✔ table cell correct, including the deliberately open-ended upper bound.
- **`mk` regular data 14..93 ⇒ total 17..96** — `mkRegularMinLen = 14`, `mkRegularMaxLen = 93` (`mdmk.go:47-48`); `ValidMK` case `n >= 14 && n <= 93` (`mdmk.go:139`). +3 ⇒ **17..96**. ✔
- **`mk` long data 96..108 ⇒ total 99..111** — `mkLongMinLen = 96`, `mkLongMaxLen = 108` (`mdmk.go:49-50`); `ValidMK` case `n >= 96 && n <= 108` (`mdmk.go:142`). +3 ⇒ **99..111**. ✔
- **`mk` reserved-invalid data 94..95 ⇒ total 97..98** — `ValidMK` `default: return false` covers the 94..95 gap (`mdmk.go:145-147`); doc-comment "Lengths 94..=95 … reserved-invalid and rejected here" (`mdmk.go:133-135`). +3 ⇒ **97..98**. ✔ The spec correctly states this band is "rejected, never corrected."

The `+3` prefix accounting is correct for all three HRPs: `md1`/`mk1`/`ms1` are each 3 chars (HRP 2 + the `1` separator); `splitHRP` cuts on the first `1` (`codex32.go:453-458`), so "data-part" = chars after `xx1`. No off-by-one found.

**I-1 hazard was real, and the fix covers it.** Measured against the canonical test vectors (`codex32/mdmk_test.go:13-15`):
- `md1Regular` = `"md1yqpqqxqq8xtwhw4xwn4qh"` → **total 24** (data-part 21). 24 < 48 ⇒ under the old codex32-only `ShortCodeMinLength=48` window it would *never* arm feedback or the "Fix?" trigger. Hazard real; the new `md` window (total ≥16) arms it.
- `mk1Long` = total **111** (data-part 108). 111 ∈ [94,124] — squarely in the codex32 dead zone (`codex32StatusLine` line 27-28: `n < LongCodeMinLength` → "keep typing", never an arming window). Under the old code it would never arm. The new `mk` long window (99..111) arms it. Hazard real, fix covers it.
- (Aside: `mk1Regular` = total 80, which *coincidentally* falls in codex32's 48..93 — confirming the R0 observation that md/mk breakage was length-class-specific, not total. The spec's table is keyed on the generic `mkRegular` brackets, not this single example, so there is no conflict.)

### Task 2 — I-2 genuinely closed (anchor both load-bearing AND producible)

Both halves verified:

- **`ParsePrefix` genuinely fails on md/mk data** (so the `id·thr·share` header line truly cannot be produced for md/mk, vindicating the "ms-only" restriction). `ParsePrefix` applies the codex32 **share** schema unconditionally: `data[0]` must be a threshold ∈ {0,2..9}, else it returns `errInvalidThreshold` (`polish.go:102-109`). For `md1yqp…`, `data[0]='y'` (not a digit) ⇒ `errInvalidThreshold` the moment the first data char is typed. Confirmed: there is no HRP guard in `ParsePrefix`; it does not branch on `f.HRP`. The spec's §2.3 and §4.2 claim ("`ParsePrefix` returns `errInvalidThreshold` on md/mk data") is exactly correct (spec lines 68-70, 202-204).
- **The per-position diff is HRP-neutral and genuinely producible for md/mk.** §3.2 defines `CorrectionResult{Corrected string; Edits []Edit{Pos,Was,Now}}` (spec lines 113-124) returned by the pure decoder `Correct(frag)`, parameterized internally by HRP but emitting `Edits` regardless of HRP — the edits are positional substitutions from the BCH apply step, carrying no codex32-share-schema assumption. So the universal anchor (spec §2.3 lines 64-67; §4.2 lines 200-205) is producible for all three m*1. The safety argument holds: the load-bearing anchor (per-position diff "compare each changed position to your source card") exists and is producible for md/mk, while the inapplicable codex32 header line is correctly demoted to an ms-only *secondary* anchor.

I-2 is closed. The spec no longer mandates an unproducible anchor for md/mk; it explicitly states (line 70, 204) the diff IS the md/mk anchor and that no md/mk header parser exists / is out of scope.

### Task 3 — I-3 suppression correctly specified, consistent with I-1

§4.1(c) (spec lines 172-178) correctly identifies the root cause: `codex32Feedback` surfaces `perr` eagerly and unconditionally — `if perr != nil { return codex32.Describe(perr) }` at the very top of the function, *before* the length-window gate (`gui/codex32_polish.go:56-59`). I confirmed `inputCodex32Flow` feeds `perr` from `ParsePrefix(kbd.Fragment)` into `codex32Feedback` every frame (`gui.go:731,783`), and `Describe(errInvalidThreshold)` → `"bad threshold"` (`polish.go:38-39`). So a valid `md1y…` would show a spurious "bad threshold" the instant `data[0]` is typed — exactly as the spec states (lines 174-176). The mandated fix (gate the `ParsePrefix`/`codex32FieldLine` feedback to `HRP=="ms"`; for md/mk use HRP + length-window + per-HRP validity) is correct and **consistent** with I-1: §4.1(a) dispatches *validity* by HRP (`New`/`ValidMD`/`ValidMK`), and §4.1(c) gates the *header feedback* to ms — these compose without contradiction (validity is computed for all HRPs; only the codex32-share-schema *header* surface is ms-gated). No conflict.

### Task 4 — Drift hunt (whole-spec fresh read)

- **Cross-references all resolve.** Every `§` and table reference in the folded text points to a real target: `§2.3` (invariant #3, lines 56-72), `§2.5` (#5, lines 80-83), `§2.6` (#6, lines 84-87), `§4.1`/`§4.1(b)`/`§4.2`/`§7` all exist. The fold's new "§4.1(b) table" label (lines 158-166, referenced from lines 171, 194, 234) is internally consistent. No dangling refs.
- **Manifest matches §4.1/§4.2.** §6 `gui/codex32_polish.go` row (line 233) lists per-HRP length window + `New`/`ValidMD`/`ValidMK` dispatch + suppress-`ParsePrefix`-for-md/mk + the diff-confirm screen (universal diff, `id·thr·share` ms-only, Button2-drain) — matches §4.1(a)(c) and §4.2. `gui/gui.go` row (line 234) lists HRP-dispatched per-frame validation + §4.1(b) length windows + `any` return + menu relabel — matches §4.1. The earlier I-1 finding ("manifest only marked gui.go 'dispatch + any return; menu relabel', not the per-frame validation rework") is now closed: the manifest explicitly says "HRP-dispatched per-frame validation (`New`/`ValidMD`/`ValidMK`) + HRP-aware length windows."
- **§5 "ms only" note matches §2.3/§4.2.** §5 (lines 220-224): "the `id · thr · share` header line (**ms only**)" + "the per-position diff (all m*1)" — consistent with §2.3 (line 67-70) and §4.2 (lines 200-205). No contradiction.
- **`any`-return ripple still sound.** Verified `engraveObjectFlow` already routes both `case codex32.String:` → `engraveCodex32` and `case mdmkText:` → `mdmkFlow` (`gui.go:1861,1865-1866`); `mdmkText` is a real type (`gui/scan.go:78`). Menu `case 2` (`gui.go:2037-2041`) assigns `inputCodex32Flow`'s result into the `(any, bool)` return of the surrounding flow, so the `any` change ripples only to `recoverCodex32Flow` (`gui/codex32_polish.go:161,171`) — which the spec enumerates (lines 180-189) with two valid resolution options. Unchanged from R0; not weakened.
- **TDD §7 consistent.** §7 (lines 241-251) covers HRP-dispatch entry, the Fix→confirm→accept→re-validate path (asserts diff shown + corrected string engraves), reject-keeps-editing, suppress-when-uncorrectable, Button2-drain no-hang, `recoverCodex32Flow` green. Aligns with the folded §4.1/§4.2. No drift.

### Task 5 — Phase A (§3) not disturbed

§3 (lines 99-135) is byte-identical in intent to the R0-VERIFIED-SOUND text: GF(1024)=GF(32²) on `fe.Mul`, β/γ/J_START constants, the syndromes→BM→Chien→Forney→re-verify pipeline, `deg(Λ)>4 ⇒ fail`, Chien root-count==deg, Forney-magnitude-in-GF(32)-nonzero, subs-only, unique-within-radius-or-nothing, pure (no GUI/RNG). The `CorrectionResult`/`Edits` API (§3.2 lines 112-124) is unchanged and is what makes I-2's universal anchor producible — so the fold *relies on* Phase A unchanged rather than weakening it. Security invariants §2 (#1 no-auto-apply, #2 mandatory re-verify, #4 suppress-unless-unique, #5 per-code-const integrity, #6 orientation pin, #7 TinyGo + the M-3 length-8 buffer note now folded into #7 lines 91-93) are intact. M-3 fold verified: line 92 now reads "BM's working `prev`/`Λ′` and the `Ω` buffer are length-**8** … the plan sizes `[8]`-wide arrays, not `[5]`," citing `bch_decode.rs:446` — exactly the R0 M-3 ask.

### Task 6 — New Critical/Important surfaced by the fold

None found. The fold is additive and constrained to Phase B's md/mk realization plus the two §2.7/§2.3 notes; it introduced no new code paths, no new constants, and no new contradictions. The "ms-only header / universal diff" split is the minimal correct resolution and does not regress the ms path (ms still gets both anchors).

---

## Findings

### CRITICAL — none.

### IMPORTANT — none.

All three R0 Importants are genuinely closed against source:
- **I-1** closed — §4.1(a)(b) + manifest now specify HRP-dispatched per-frame validation and a dual-coordinate (data-part vs total) per-HRP window table whose every cell I verified against `mdmk.go`/`codex32.go`; the two real hazard cases (`md1Regular`=24<48, `mk1Long`=111 in the 94..124 dead zone) are now armed.
- **I-2** closed — `ParsePrefix` confirmed to fail on md/mk data (header line genuinely ms-only), and the per-position diff confirmed HRP-neutral/producible (decoder emits `Edits` regardless of HRP); the load-bearing safety anchor exists for md/mk.
- **I-3** closed — §4.1(c) correctly localizes the eager `if perr != nil` surface (`codex32_polish.go:56-59`) and mandates gating it to ms, consistent with the I-1 validity-dispatch.

### MINOR

- **m-1 (informational, not blocking).** The minor numbering convention mixes "§2.N" (subsection-of-the-numbered-list, e.g. §2.6 = invariant #6) with §-as-section elsewhere; every reference resolves correctly, so this is purely stylistic. No action required for GREEN.
- **m-2 (carried, cosmetic).** R0 M-2 (recon citation drift) is partially addressed — line 6 now cites recon `47c9ad7` with "recon-time source SHA `04a1e95`," and the base pin `04a1e95` is verified correct against fork HEAD. The remaining recon-internal HEAD/SHA bookkeeping is cosmetic and pre-merge-verifiable; not a gate item.

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

All three R0 Importants (I-1/I-2/I-3) and the two folded minors (M-1 dual-coordinate window table, M-3 length-8 BM buffers) are closed and verified against the authoritative fork source at `04a1e95`. The folds introduced no drift: cross-references resolve, the §6 manifest matches §4.1/§4.2, §5's "ms-only header" note matches §2.3/§4.2, and the security invariants in §2 are intact. Phase A (§3) is undisturbed and is in fact the mechanism that makes the I-2 universal anchor producible. No new Critical or Important surfaced.
