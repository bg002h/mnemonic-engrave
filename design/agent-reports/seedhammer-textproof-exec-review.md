# Opus architect — TEXTPROOF! execution review

Diff `05ebece..5de1999`. Source verified untouched; all probing on copies; baseline green.

## VERDICT
**NOT GREEN (0C / 2I)** + 6 Minor, 8 Nit. **The plate it produces today is correct** —
re-measured independently. Both findings are about what is UNGUARDED and what the proof
FAILS TO SAMPLE, not a wrong plate in hand.

## I1 — The flow-level wiring has ZERO coverage; six mutations survive, two wrong-plate-capable
`gui/freetext_flow.go:150-160`, `:379-380`. The author's six mutations were all inside
`freetext_proof.go`; nothing exercises `ftTextEntryFlow`'s integration.

| mutation | result | consequence |
|---|---|---|
| F1 drop the `kbd.Fragment` re-seed | **SURVIVED** | field keeps `TEXTPROOF!`; declining engraves body `TEXTPROOF!` at 6.0mm **with the proof's title and footer** |
| F2 drop the `continue` | **SURVIVED** | one tap jumps to Title; operator never sees what landed |
| F3 pass `nil` loader | **SURVIVED** | trigger dead |
| F4 swap `&footer, &title` | **SURVIVED** | title and footer engrave transposed |
| F5 re-seed with `ftProofFor(!*useQR)` | **SURVIVED** | with a QR chosen, 758-char variant → refusal loop |
| **F6 delete the entire trigger block** | **SURVIVED** | **the whole feature is absent and the suite is green** |

Wrote the missing E2E test (drives `engraveTextFlow` by touch through `freetextPlateHook`) and
confirmed it kills F1/F2/F4/F5, asserting `size=3 title="TEXTPROOF 3.0mm 44"
footer="gjpqy 0O 1lI| rn m" lines=20 qr=false` / `lines=21 qr=true`.

## I2 — Uppercase absent from running text, on a machine whose every other plate is UPPERCASE
Measured in the prose: lowercase **26/26**; uppercase **7/26** (`DEHLPTU`) no-QR, **4/26**
(`HLPT`) with-QR. Fourteen-to-seventeen capitals appear only inside the dense unspaced sweep.
Meanwhile `backup.EngraveSeed` calls `strings.ToUpper` on the seed and on **every mnemonic word**
(`backup.go:126,163,308`), and descriptor titles engrave at `plateSmallFontSize = 3.0` — exactly
the size this plate exists to qualify. Reading `ABANDON` in steel is a different task from reading
`…ABCDEFGHIJKLMNOP…` where every neighbour is a capital and there is no word shape.
**The fix is free** — measured: no-QR 758→844 stays 3.0mm (22/24 lines); with-QR 436→481 stays
3.0mm (23/24).

## M1 — Confusable adjacency asserted on the SOURCE string, never the engraved lines
Lengthening a group so it wraps survives the suite. Today every group is intact on both variants
(all 20/21 lines dumped and checked) — latent, not live. Assert on `Fit`'s `lines`.

## M2 — The plate's own label is not bound to the measurement
`ftProofTitle = "TEXTPROOF 6.0mm 44"` and `"TEXTPROOF 3.0mm 99"` both survive. On permanent steel
the title is the only record of what was tested.

## M3 — A degenerate pattern passes everything
`sweep + confusables + strings.Repeat("xy ", 200)` — no prose, no pangrams, no wrap variety —
passes the whole suite. Nothing pins the prose.

## M4 — The prompt's semantics are entirely unpinned
Three survivors: swapping the nav icons so **the checkmark means "no"**; `noBtn` returning true so
**Back also loads**; and replacing the "REPLACES ALL THREE" copy. `ftProofNav`'s own comment says
it is "named so a test can assert WHICH ICON sits on WHICH answer" — no such test exists.

## M5 — The "Test Pattern" title rationale is wrong on BOTH halves
Measured: `uiContains(content, "Test Pattern")` returns **true with the title not drawn at all**,
because the body's "…test pattern?" survives space-stripping — so the choice does not make a title
assertion non-vacuous, it moves the trap one level down. And the named trap is unreachable anyway:
`NewTextKeyboard` starts unrevealed, so the readout extracts as `**********`, never `TEXTPROOF!`.

## M6 — The confirm screen CANNOT SHOW THIS PLATE; the safety copy is off-panel
`ftConfirmBody` measures **637px** (no QR) / **701px** (QR) against `ppConfirmArea`'s **270px** on
the 480×320 panel — 136% / 160% overflow. The size line starts at y=510 and all three warnings
(`ftWarnNotBackup`, `ftWarnTiming`, `ftWarnQR`) at y=537: **entirely off-screen**.
`TestFTConfirmCarriesTheSafetyCopy` is a **false PASS** for exactly the reason `ftProofBody`'s own
comment states — ExtractText ignores occlusion. Pre-existing for any long free text; `TEXTPROOF!`
is now the one-tap route to it.

## NITS
N1 case-insensitive trigger survives. N2 footer descenders and the sweep's leading space survive.
N3 the doc comment's "816 without a QR / 516 with" — measured hard capacity is **1032** and 520;
816 is 26% low. N4 "the only place the plate shows a space" is false. **N5 the confusable separator
is ` | ` while `|` is itself a member of the `1lI|` group, so the plate reads `1lI| | 5S` —
ambiguous at exactly the size under test.** N6 "the only place inset centring is exercised at its
limit" — at 3.0mm the inset span is 36 and the title is 18, half. N7 no clear-all key, so loading
the no-QR proof then adding a QR strands 758 chars. N8 declining ADVANCES the flow, so the Back
glyph moves forward.

## CHECKED AND CLEAN
Both patterns land at **exactly 3.0mm** at true production params (measured via
`newPlatform().EngraverParams()`, not the test file's hand-rolled params). Title and footer engrave
at the same `fontMM` as the body, so the plate's "3.0mm" is true of the title too. All 95 printable
ASCII present in both, derived from the codepoint range. **Every body rune decodes in `font/sh`** —
checked individually and both plates built end to end under `recover()`, since `engrave.String`
panics on an unknown rune mid-plate. **No confusable pair is split by the wrap on either variant**
(all lines dumped), including where the QR narrows the budget to 14–16 characters. Trigger safety:
whole-field equality, one call site, `ftLineEntryFlow` never calls it, nil loader short-circuits.
Declining leaves `TEXTPROOF!` usable (driven E2E). No button double-consumption across the nested
prompt. The loader never touches the QR choice. **All six of the author's claimed mutations are
genuinely killed** — each re-run and attributed.
