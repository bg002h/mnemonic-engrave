# Pre-ship adversarial execution review — `proof-sizes-and-equals` — round 0

Reviewer: independent opus subagent. Dispatched 2026-08-05 against
`seedhammer` branch `proof-sizes-and-equals` @ `6064aa8`, base `main` @ `0983e3c`
(6 commits, ~1746 insertions, 24 files). Persisted VERBATIM before folding, per
the R0 gate.

Scope given: *"is there anything in this diff that would produce a WRONG PLATE,
a wrong-looking glyph on steel, a false-passing test, or a broken firmware
build?"* — with the already-settled facts stated so they would not be
re-derived.

---

VERDICT: RED (0 Critical / 2 Important)

**No Critical findings.** I could not produce a wrong plate or a wrong glyph from this diff. Specifically verified, not assumed:

- `constant.bin` is **not stale**: `go generate ./font/constant` reproduces byte-identical output (md5 `656230a32820cf728425077af53b3f99`, `git status` clean after regen).
- `'='` compiled to `[{100 -435}{500 -460}]` / `[{100 -165}{500 -140}]` — narrow-end ink gap 180 font units = 0.600mm = exactly 2.0 stroke widths, wide end 2.56; divergence 7.153° vs `'n'` 7.125°. `'f'` compiled to 8 vertices, crossbar 7.187°, hook 6.843° (one quantisation step), left arm 2.00 / right 1.25, all x within 100..500. Neither glyph changed its **run count** (`'='` is still 2 polylines, `'f'` still 1) and neither crosses the side bearing.
- TinyGo `pico-plus2` firmware build: exit 0. `internal/sh2` is imported only by `cmd/plateview` and `cmd/emu` (`js`); `gui/preview.go` is `!tinygo`. Nothing new enters the image.
- Full `go test ./...`: green. 14 mutations applied and reverted; 12 were caught (including reverting `'='` to flat bars, `'f'`'s crossbar to y4.5, growing `'f'` outward past x500 — caught by all four passphrase goldens moving, the off-by-one face split, the stale plan split, an auto-fit `ftFitAt`, and a promiscuous suffix parser). Two survived; both are below.
- Worktree left clean, HEAD unchanged at `6064aa8`.

---

**Important — gui/freetext_proof_sizes_test.go:107** — `TestMixedProofLabelsStateTheRungTheyAreCutAt` checks the mixed-proof title by comparing `ftBothAt`'s output against `ftProofTitleBothAt(size)`, the same function that produced it, so the assertion cannot fail. — Replace the body of `ftProofTitleBothAt` (gui/freetext_proof.go:331-333) with `return "SH+CONST 3.0mm"`: `go test ./...` stays **fully green**, and `BOTHPROOF!6` then cuts a 6.0mm plate titled `SH+CONST 3.0mm` — permanent steel whose only record of what was tested states a size the plate does not have, which is what the comment at :328 calls "worse than no title". The same file already fixed this exact class for the *labels* at :117-121 ("comparing a function against itself passes however wrong it is, and the first version of this test did exactly that") and left the title two lines above unfixed. — Fix: build the expected title from primitives, `want := fmt.Sprintf("SH+CONST %.1fmm", size)`, as the label branch already does.

**Important — gui/freetext_proof.go:498-503 (`ftProofReplaces`)** — the consent prompt misstates two of the three fields it claims to replace whenever a rung is named, because it prints `p.Title` and `ftProofFooter` (the untrimmed pattern's) rather than what the loader will write. — Measured, `BOTHPROOF!4.4` prompts: *"…Title becomes SH+CONST 3.0mm, Footer becomes gjpqy 0O 1lI| rn m. The plate is cut in sh+constant at 4.4mm, with the pattern trimmed to fit."* The actual title is `SH+CONST 4.4mm`; the sentence contradicts itself. At `BOTHPROOF!6` the actual footer is `TOP SH / BOT CONST`, not the one stated. `TestProofPromptSaysWhatItWillDo` (gui/freetext_proof_test.go:965) and `TestProofWholePlateDropsTheQR` only ever call `ftProofReplaces(p, 0)`, so nothing covers the rung path. — Fix: when `size != 0`, take the title from `ftProofTitleBothAt(size)` and the footer from `ftBothHalves`/`ftBothAt`, and loop the prompt tests over `backup.FontSizes`.

**Minor — gui/freetext_proof_sizes_test.go:147** — `TestMixedProofNamesTheFacesWhenLabelsGo` asserts `footer != ftProofFooterFaceMap`, comparing against the very constant whose *content* the test's name claims to verify. — Mutating `ftProofFooterFaceMap` (gui/freetext_proof.go:243) to `"gjpqy 0O 1lI| rn m"` leaves `go test ./...` green; the 6.0mm plate (the only rung that reaches `ftDropLabels` — measured drop levels are 0,1,2,3,3,4 for 3.0→6.0) would then carry a footer naming no face, and nothing on the plate would say which half is which. Weaker than the two above because the test does still catch the *wiring* regression (footer not swapped) and the constant is a static literal. — Fix: also assert `strings.Contains(ftProofFooterFaceMap, "SH")` and `"CONST"`.

**Minor — gui/freetext_proof.go:635-641** — the fall-through comment is false and the path silently downgrades the rung: when `ftBothAt` fails for a named rung the loader loads the untrimmed pattern and sets `*size = 0` (auto-fit, 3.0mm), after the prompt already said "at 4.4mm, with the pattern trimmed to fit". The comment claims "the untrimmed pattern, whose own size the prompt already stated" — the prompt stated the *named* rung. — Unreachable today (all six rungs build; verified), so no plate is wrong. — Fix: return the error so the operator gets a refusal, or correct the comment.

**Minor — backup/fit.go:210-214** — `FitBlocksAt` has no test in package `backup` at all; the `slices.Contains(FontSizes, size)` guard and the `len(blocks) == 0` early return are entirely unexercised (every reference outside `fit.go` is in `gui`). — The guard is live: `cmd/plateview -size 4.5` reaches it, and removing it would lay a plate out at a rung no capacity figure in the package is measured against, silently. — Fix: add a `backup` test for the rejection and for "the rung asked for is the rung returned".

**Minor — gui/freetext_proof_test.go:1008** — `TestProofPromptFitsPanel` measures the prompt only at `size == 0`, but the rung prompt is 18px taller. — Measured on the real 480x320 panel: 221px of 270px available at every rung, so it fits today with 49px to spare; the guard simply does not cover the longer string, and the comment at :990 notes overflow is unreadable because the scroller is bound to buttons the machine lacks. — Fix: loop the size argument over `append([]float32{0}, backup.FontSizes...)`.

**Nit — gui/freetext_proof.go:420-427** — the file is no longer gofmt-clean (`gofmt -l` lists it; the same file on `main` is clean); the `BOTHPROOF!` struct literal's keys are misaligned after `Sizeable: true` was inserted. CI does not gate gofmt, so nothing breaks.

**Nit — gui/freetext_proof.go:510-516** — `ftRungLabel` was inserted between `ftProofKeep`'s doc comment and `ftProofKeep` (:523), so that comment now documents the wrong function and `ftProofKeep` has none.

**Nit — font/constant/glyph_rules_test.go:381-386** — `const ascender, descender = -600.0, 100.0` and its failure message call that "the alphabet's" vertical extent, but the true extent is **-700..100**: `'#'` (constant.svg:106) and `'|'` (constant.svg:132) both reach svg y=1. The check itself is sound (a conservative bound that `'='` clears); only the stated range is wrong.
