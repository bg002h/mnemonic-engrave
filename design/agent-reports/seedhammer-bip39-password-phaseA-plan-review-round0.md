# Fable architect — Phase A plan review, round 0

- **Reviewer role:** adversarial fable architect reviewing an implementation plan before execution.
- **Plan under review:** `design/IMPLEMENTATION_PLAN_seedhammer_bip39_password_phaseA_substrate.md` @ `23439dd`
- **Brief:** one question — if an engineer with no prior context executes this plan literally, do they end up correct, or does the plan mislead them? Spec decisions D1–D8 and the two-phase split declared settled and out of scope; empirical facts verified across the three spec review rounds taken as given.
- **Date:** 2026-08-03

Persisted verbatim before folding, per the project standard.

---

## VERDICT
**NOT GREEN (2C / 2I)** — plus 5 Minor, 3 Nit.

## EXECUTABILITY CHECK
- **Task 1 — OK** (skip-placement caveat, M1)
- **Task 2 — OK** — all steps verified, including empirically: `nix develop` from `font/constant` resolves the flake by searching up, and `go generate` reproduces `constant.bin`/`constant.go` byte-identically today
- **Task 3 — BROKEN** — worked-example coordinates sit in occupied slots; the generator's order-derived slot rule is never stated (C1); the Step 1 dump command drops stray generated files at the repo root (I1)
- **Task 4 — OK** — refactor is behaviour-preserving; alphabet literal is exactly 96 runes, contiguous 0x1F,0x20–0x7E ascending (verified by count); Step 6 uses nonexistent helpers (M2)
- **Task 5 — BROKEN** — the measurement procedure cannot run as ordered (C2); the cross-repo `git add` in Step 6 is fatal (I2)

## FINDINGS

### C1 — Task 3's coordinate examples are wrong, and the load-bearing slot rule is never stated
**Where:** Task 3, house style + Steps 4–6.
**Defect:** `parseChars` (`cmd/vectorfont/main.go:414-428`) assigns each glyph's x-offset by **document order**: after every glyph element it does `offx -= adv`, so the k-th glyph element (0-based) is normalized against slot `x ∈ [6k, 6k+6)`. The drawn x-position does not choose the slot; element order does. The current SVG has **51** glyph elements (space is synthetic, metrics markers are skipped by id), so `dash` occupies slot 50 (x 300–306, matching `viewBox` width 306) and the next free slot is **x 306–312**. The plan's worked examples place `o` at x 289–293 (inside `star`'s slot 48) and `p` at x 295–299 (inside `at`'s slot 49). Appended as elements 52 and 53, both normalize to x = **−17..−13** — 17 units left of their cells.
**Failure:** Every mechanical acceptance test still passes — advance is the uniform `meta.Advance` regardless, `Decode` succeeds, the start point isn't (0,0) — so the engineer gets a font where the only two glyphs the plan drew for them are engraved ~3 cells to the left of where the passphrase layout expects them. Worse, the plan affirmatively teaches that x-coordinates are freely choosable ("place each glyph at the next free x-slot"), so an engineer who inserts new glyphs mid-file (e.g. alphabetically after `Z` — the natural order) silently shifts the normalization of **every subsequent existing glyph**, corrupting existing plate output (the Step 9 goldens would catch that variant; nothing automated catches the new-glyph misplacement — only the eyeball loop, which the plan has just miscalibrated).
**Fix:** State the rule explicitly in the house-style section: *the k-th glyph element in document order occupies x ∈ [6k, 6k+6); markers and the synthetic space consume no slot; new glyphs MUST be appended after `dash` and before the markers, first at x 306–312; never reorder or insert mid-file.* Correct both examples (+17: `o` → `307,5 308,4 310,4 311,5 311,7 310,8 308,8 307,7 307,5`; `p` → base 312) and add a one-line cross-check ("after appending N glyphs the viewBox width must read 306+6N").

### C2 — Task 5's measurement procedure is inverted and halts itself: maxima for dims 37/41 are unobservable as written
**Where:** Task 5, Steps 2–4.
**Defect:** The plan raises the fuzz entropy cap to 120 and runs the fuzzer *before* touching `engrave.go` (Step 4 conditions the extension on the fuzz result). But `ConstantQR` rejects `dim > 33` at `engrave/engrave.go:408-413` before any module counting, and even without that guard `constantTimeQRModules` returns 0 → the `len(modules) > nmod` error at `:479`. `FuzzConstantQR` calls `t.Fatalf` on any `ConstantQR` error (`engrave/engrave_test.go:439-441, :449-451`), and any input > 78 bytes at `qr.L` (v5, dim 37) or > 62 bytes at `qr.Q` triggers it. The 10-minute run of Step 3 dies within seconds as a "crasher", having observed nothing. Additionally, the 120-byte cap breaks the fuzzer **permanently**: at `qr.Q`, 87–120 bytes produce v8/v9 (dims 49/53), which even a successful v5/v6 extension must reject — the Q branch `Fatalf`s forever.
**Failure:** Step 3 is unexecutable; the engineer either abandons the task or hand-rolls unplanned surgery on `engrave.go`, and the O6 decision gets made on no data.
**Fix:** Reorder: (1) make a **provisional** extension first — raise the guard to 41, add `case 37, 41: alignMarkers = append(..., bezier.Pt(dim-9, dim-9))` to `bitmapForQRStatic`, and return a generous provisional bound (e.g. `dim*dim`) from `constantTimeQRModules` for 37/41; (2) in the fuzz target, treat `ConstantQR` errors for `dim > 41` as a skip (`return`), not `Fatalf`, and log `qrc.Size` + `len(cmd.plan)` on success; (3) fuzz, with a **decidable** convergence criterion (e.g. no new per-dim maximum in the last 5 of 10 minutes) and a sanity ceiling (`dim²` minus static-feature modules); (4) then either replace the provisional values with maxima+`extra` or revert the extension entirely and record the 78-char cap. The overall approach — fuzz-derived maxima + `extra`, backed by the fail-closed `:479` error — is defensible: it is exactly how the existing v1–v4 constants were derived, and an underestimate produces a refusal, never a wrong plate. The procedure, not the philosophy, is what's broken.

### I1 — Task 3 Step 1's dump command writes stray generated files into the repo root
**Where:** Task 3, Step 1 (and every iteration of the authoring loop).
**Defect:** `run()` in `cmd/vectorfont/main.go:88-94` writes `name+".go"`/`name+".bin"` to the **current directory**. The plan's command runs from the repo root, so each loop iteration drops `./constant.go` and `./constant.bin` at the root (verified empirically — both files appeared) while `font/constant/`'s real outputs go stale relative to the SVG being edited.
**Failure:** Silent repo pollution (exit 0, dump appears fine), plus a mid-loop trap: any `go test ./font/constant/` run during authoring judges the stale binary, telling the engineer their glyph edits "didn't take". The stray root package also gets swept into `go build/test ./...`.
**Fix:** Run the dump from `font/constant`: `cd font/constant && nix develop --command go run seedhammer.com/cmd/vectorfont -package constant -scale 100 -dump /tmp/font-dump.svg constant.svg constant` — which also keeps the real `.bin`/`.go` fresh during the loop.

### I2 — Task 5 Step 6's `git add` crosses repository boundaries and is fatal
**Where:** Task 5, Step 6.
**Defect:** `git add engrave/engrave.go engrave/engrave_test.go ../mnemonic-engrave/design/SPEC_...md` — verified: `fatal: ... is outside repository at '/scratch/code/shibboleth/seedhammer'`. Nothing is staged; the subsequent `git commit` fails too.
**Failure:** The whole Task 5 outcome sits uncommitted; the likely recovery improvisations are committing only the seedhammer side (dropping the O6 record from the spec) or reaching for `git add -A` (forbidden by project convention).
**Fix:** Two commits, two repos: stage/commit `engrave/*` in seedhammer; separately commit the spec §9 O6 resolution in `mnemonic-engrave`.

### M1 — Task 1 Step 3: skip placement is unstated and order-sensitive
The snippet must be inserted **after the loop but before** the `len(missing) > 0` error block. Go's testing package treats a test that fails and then skips as failed, so appending it after the `Errorf` yields FAIL, not the expected SKIP. Step 4 self-catches this, but one sentence ("insert between the loop and the error report") removes the stumble.

### M2 — Task 4 Step 6 uses nonexistent helpers `testParams()`/`mmTest`
The real helpers are `params()` (`engrave/engrave_test.go:131`) and `mm` (`:113`) — which Step 1 of the same task uses correctly. As written, Step 6 does not compile. Rename.

### M3 — Task 3's acceptance criterion 1 overclaims: no test in the task enforces that 0x1F decodes
`TestPrintableASCIICoverage` spans 0x20–0x7E only, and both rule tests `continue` on `!ok`. A forgotten `space_mark` glyph sails through Task 3's gate and surfaces only as Task 4's construction panic. Add a one-line `Font.Decode(0x1F)` assertion to Task 3.

### M4 — O5 "must be identical to their pre-change values" has no captured baseline
Nothing before Task 3 records the shared stringer's `runeDuration`/`center`. The real proof is structural (existing glyph splines are byte-unchanged — regen determinism verified — plus the goldens, which do exist behind a real `-update` flag). Either capture the two numbers in Task 1, or state explicitly that the goldens are the proof and the Task 4 log is merely the record.

### M5 — the worked `o` is the same stroke path as the existing `O` at reduced height
`O` is `85,3 86,2 88,2 89,3 89,7 88,8 86,8 85,7 85,3` — the identical octagon. The plan's sole lowercase example is thus a by-construction instance of the "case-only confusable" trap its own confusables section flags for the 0/O/o triple. Step 10's mandatory inspection gates it, but the starting point should model the differentiation convention, not the anti-pattern.

### N1 — `TestUniformAdvance` is near-vacuous
The generator assigns `meta.Advance` to every glyph (`parseChars`), so per-glyph variance is impossible; it guards only the `advance` marker line, which the global constraints already freeze.

### N2 — Task 5 Step 5's `cmd == nil` branch is unreachable
`ConstantQR` never returns `nil, nil`; the fail-closed mechanism it means to document is the `:479` error.

### N3 — Minor line-drift in citations
`mm` at :113 not :112; `FuzzConstantQR` at :426-453. Immaterial.

## WHAT I CHECKED AND FOUND CLEAN
- **All load-bearing citations verified:** `constantAlphabet` :750; panics :1210/:1215/:1218; paddedString panic :1286, sentinel :1296; `String` panic :1365; `mapChar` :704-771 with shared `return r, true` and `lt`/`gt` precedent as described; `constantTimeQRModules` :349 with `extra = 5` and 0-default; `bitmapForQRStatic` :384; `dim > 33` guard :408; threshold :479; duration sites :641/:649; `FuzzConstantQR` and 40-byte truncation; `-update` flags exist in engrave/backup/gui tests; `newBitmap` panics >64 (41 is safe); spec file and all three R0 report files exist under `mnemonic-engrave/design/`.
- **APIs:** `Font.Decode(rune) (int, UniformBSpline, bool)`; `Next() (Knot, bool)`; `Knot{Ctrl bezier.Point; Line bool}`; face indexes 0..126 (`indexLen = unicode.MaxASCII`), so 0x1F is addressable and currently free (ok = `Advance > 0`); `engrave_test.go` already imports `font/constant` — no import cycle; Go 1.25.10, so the range-over-func `runes` iterator is valid. All four proposed test files compile against these APIs (modulo M2).
- **Empirical:** `nix develop` from `font/constant` searches upward and finds the flake; `go generate` is deterministic and currently byte-identical (Task 2 Step 6's premise holds); the dump command works (and exposed I1).
- **Task 4 core:** `passphraseAlphabet` literal is exactly 96 runes, contiguous 0x1F, 0x20–0x7E, strictly ascending; the `newConstantStringer` extraction is a pure parameterization of `:1202`/`:1208` — behaviour-preserving. Including 0x20 (empty synthetic spline) is safe: `engraveSpline` on an empty spline yields nothing and returns true; construction and `paddedString` handle it without panic.
- **Counts:** 52 decodable = 26 A–Z + 10 digits + synthetic space + 15 drawn symbols; the 43-missing list and the 26+17+1=44 new-glyph arithmetic are exact; `;`, `<`, `>` are named in `mapChar` but glyphless, as the plan states.
- **Task 5 side facts:** v5/v6 take exactly one alignment pattern with corner at (dim−9, dim−9) — correct; `strings.Repeat("Xy7#", 30)` at `qr.L` is v6 (dim 41) — correct.
- **Ordering:** Task ordering and the staging note are sound — nothing constructs the passphrase stringer before Task 4's own test, so glyphs may precede the alphabet; Task 3's append-only edits can't disturb the shared stringer (and Step 9's goldens gate the reorder hazard); Task 5 is independent of Tasks 1–4.
- **TDD:** Task 1's RED (43 missing) and Task 2's RED ("space_mark not recognised") are genuine; Task 4's RED is a real compile failure; Task 3's rule tests are honestly presented as pre-passing regression nets, with Task 1's coverage test as the flipping gate.

The plan's substrate architecture is faithful to the spec and most of it is executable to the letter. The two Criticals are concentrated where the brief suspected: Task 3's coordinate space and Task 5's decision procedure. Both have mechanical fixes that don't disturb the plan's structure.
