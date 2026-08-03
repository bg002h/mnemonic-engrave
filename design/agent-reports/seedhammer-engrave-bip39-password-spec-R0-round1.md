# Opus architect — R0 round 1 — SPEC_seedhammer_engrave_bip39_password.md

- **Reviewer role:** adversarial opus architect, round 1 of the mandatory pre-implementation R0 gate (must reach 0C/0I).
- **Spec under review:** `design/SPEC_seedhammer_engrave_bip39_password.md` @ `7154176` (post-fold)
- **Brief:** scoped re-review — *did the fold fix each round-0 finding, and did it introduce a new defect?* Round 0's eight CONFIRMED claim verifications and its clean-list were declared established fact and explicitly out of scope. D1–D8 settled.
- **Round 0 verdict:** NOT GREEN (2C/6I) — `seedhammer-engrave-bip39-password-spec-R0-round0.md`
- **Date:** 2026-08-03

Persisted verbatim before folding, per the project standard.

---

## VERDICT
NOT GREEN (0C/4I)

The two round-0 Criticals are genuinely closed. The four findings below are all in material the fold **added** (§3.5.1, §3.5.2, §4.3, §7) and none of them re-litigate settled ground.

## FOLD VERIFICATION

- **C1 FIXED** — §4.3 — legend line `<mark> = SPACE` is now mandatory whenever the passphrase contains a space, engraved *with the real mark glyph* ("matching shapes, not descriptions"); §7 adds a space-fidelity assertion that the legend is emitted whenever any space is present; O4 absorbs the wording.
- **C2 FIXED** — §3.4 + new §3.5/§3.5.1 — the false "single load-bearing gate" claim is explicitly retracted in-text, replaced by a check table naming `ConstantStringer`'s face-independent binary search (`engrave.go:1282`/`1286`), and §3.5 mandates `ConstantStringer`/`ConstantQR` and forbids the non-constant variants. (Enumeration is still incomplete — see new **I4**, which is a narrower defect than C2.)
- **I1 PARTIAL** — §3.5.2 — mandates extending `bitmapForQRStatic` to versions 5–6 and correctly rejects the `engrave.QR` fallback, but omits a second required change without which nothing works. See new **I1**.
- **I2 FIXED (introduces new defect)** — §4.3 — metadata relocated to the 10 mm margin bands with the height-budget rationale stated, and §7's fit test extended to `100 chars + QR + both fingerprints + legend + footer`. The relocation does not fit in the worst case — see new **I2**.
- **I3 FIXED** — §5.1 — confirm screen must render spaces with the visible mark *and* display derived counts (`100 chars · 3 spaces · 1 trailing`), with leading/trailing called out by name; §7 adds "Confirm-screen space surfacing".
- **I4 FIXED** — §5.0 + §5 step 1 — keyboard extension to all 32 ASCII symbols is now a prerequisite, the 13 untypeable characters are enumerated correctly (`% * < > [ \ ] ^ ` { | } ~`), and §7's three-way alignment test asserts typeability.
- **I5 FIXED** — §7 — QR byte-exactness test present and specific: encode→decode, assert byte-identity with the passphrase as entered across leading/trailing/interior/repeated spaces and all 95 chars, plus the separate assertion that the engraved text stream contains zero `0x20` glyph indices.
- **I6 FIXED** — §3.2.1 — new hard requirement with the no-redundancy rationale, an enumerated pair list, and O1 amended to check every enumerated pair by eye on metal. (List omits the case-only pairs — filed as **M1**, non-blocking.)
- **M1 FIXED** — §5.3 — rewritten: `[]byte` accumulation wiped on exit and abort, honest statement that the existing string widget leaves unwipeable prefixes, implementation must state where residuals remain.
- **M2 FIXED** — §4.2 — "variable, bounded by 37 modules", reserved envelope, "tests must not assert exactly 37", plus §7's 33/37/41 variability test.
- **M3 FIXED** — §3.3 — the `mapChar` generator-extension prerequisite is stated with the `cmd/vectorfont/main.go:704-771` cite.
- **N1 FIXED** — `backup/backup.go:49-61` (§3.3), `engrave.go:1375-1377` (D5), `bip32/bip32.go:38` (D1) — all three corrected. (One new imprecise cite, see **N1** below.)

## NEW FINDINGS

### I1 — §3.5.2's QR extension is incomplete: `constantTimeQRModules` returns 0 for dims 37/41, so every version-5/6 QR still fails
**Where:** §3.5.2, O6
**Defect:** Reaching version 6 requires **three** changes, not one. §3.5.2 names only `bitmapForQRStatic`. The third is `constantTimeQRModules` (`/scratch/code/shibboleth/seedhammer/engrave/engrave.go:349-365`), a hardcoded switch over 21/25/29/33 that falls through to `// Not supported, return a low number to force error.` / `return 0` (`:363-364`). `ConstantQR` reads it at `:430` and rejects at `:479` (`if len(modules) > nmod`), and `ConstantQRCmd.Engrave` re-reads it at `:641` to drive the constant-time loop `for range nmod` (`:649`). With `nmod == 0`, every 37- or 41-module QR errors out even after the guard at `:411` is relaxed and the `bitmapForQRStatic` switch is extended.
**Failure:** O6 ("confirm the extension is feasible") is scoped to the wrong change and will be answered against an incomplete picture. The omitted constant is not a mechanical switch case: the repo documents its values as "maximum numbers found through fuzzing… Add a bit more to account for outliers not yet found" (`:350-352`), and it simultaneously sets the failure threshold *and* the engraving duration for every QR of that size. Too small → content-dependent engrave-time failures; too large → every QR of that size gets slower. This shifts the feasibility judgement materially toward the 78-char cap.
**Fix:** §3.5.2 must name all three sites (`:411` guard, `:393-401` switch, `:349-365` module counts) and state that the version-5/6 module maxima must be derived by extending the existing fuzz corpus (`engrave/testdata/fuzz`), not estimated. Note in O6 that fail-closed behaviour is preserved either way (`:479` errors, it does not truncate) — that part is sound.
*(Verified separately and correct: dims 37/41 for versions 5/6, and each takes exactly one alignment marker at `(dim-9, dim-9)` — v5 centre (30,30), v6 centre (34,34) — so the `case 25, 29, 33:` line extends cleanly. `newBitmap` panics only above width 64, so 41 is safe. The 78-byte cap is right: QR v4-L byte capacity is exactly 78.)*

### I2 — §4.3's mandated placement does not fit: three metadata lines need 9 mm, the margin band offers 7 mm
**Where:** §4.3 vs §4.1
**Defect:** §4.3 mandates legend + `SEED FP:` + `EXPECTED COMB FP:` — **three** lines — in the top band, and says explicitly that metadata goes in the 10 mm bands "**not** the usable area". At `plateSmallFontSize` 3 mm (`backup/backup.go:88`) with `LineHeight` 1, three lines are 9 mm. The band is `innerMargin` 10 mm (`backup.go:47`) minus `outerMargin` 3 mm (`backup.go:46`) = **7 mm** engraveable. Measured against the §4.1 layout instead of the band definition it is no better: a 60 mm block centred on 85 mm leaves 12.5 mm above it, minus the 4 mm `metaMargin` the existing code uses (`backup.go:117`, `:161`) = 8.5 mm for 9 mm of text — the top line runs off the plate edge. Existing practice places exactly **one** 3 mm line per band and already reaches y≈2.7 mm, i.e. slightly inside the 3 mm outer margin.
**Failure:** The relocation introduced to fix I2 reproduces I2 in the other direction: worst case (both fingerprints present + a space in the passphrase) clips at the plate edge or lands in the corner screw-hole band. §7's worst-case fit test would catch it — after the layout has been built the way the spec mandates.
**Fix:** Split the load: fingerprints in the top band (2 lines, 6 mm ≤ 7 mm), legend **and** footer in the bottom band (2 lines, 6 mm). State the per-band line budget (max 2 lines at 3 mm) as a normative constraint so the test has something to assert against. Widths are fine as-is: the longest line (`FINGERPRINTS TYPED, NOT VERIFIED`, 32 chars × 2.0 mm advance = 64 mm, centred) spans x∈[10.5, 74.5] and clears the 10 mm corner bands — by 0.5 mm, worth pinning explicitly.

### I3 — §3.5.1 (widen the shared `constantAlphabet`) is incompatible with §7's byte-identical goldens, and its impact analysis names only one of three shared effects
**Where:** §3.5.1 / O5 vs §7 "Existing-output invariance" (and D5's "existing output provably unaffected")
**Defect:** §3.5.1 treats `runeDuration` as the only shared cost. `NewConstantStringer` derives **three** values from the alphabet: `runeDuration`, `startEndDist` = `ManhattanDist(bounds.Min, bounds.Max)` (`engrave.go:1234`), and `center` = midpoint of those bounds (`engrave.go:1235`), where `bounds` is seeded at (0,0) and accumulates every alphabet glyph's path start/end (`engrave.go:1196`, `:1224-1227`). I probed the compiled face: all 52 current glyphs have ctrl-Y in **[-600, 0]**, so today `bounds.Max.Y` is exactly 0. D5 mandates descenders **1 unit below the baseline** for `g j p q y`; a single one of those glyphs whose stroke starts or ends at the tail pushes `bounds.Max.Y` positive, moving `center` for every constant-time string on the machine. `center` sets the data-independent start and park positions (`engrave.go:1274`, `:1319`), and `startEndDist` sets `padDur`/`advDur`/`centerDur` (`:1277-1279`).
**Failure:** `golden.CompareBSpline` compares both control points and tick timings — `knotsCloseEnough` is `k1.Engrave == k2.Engrave && k1.T == k2.T && pointsCloseEnough(...)` (`internal/golden/golden.go:72-75`). So widening the shared alphabet changes the goldens for **every existing plate type** (seed, slip39, codex32), and §7's "goldens must be byte-identical" is unsatisfiable as specified. The likely implementer response is `-update`, which silently discards the only guard proving the change did not touch existing output — the guarantee D5 rests its whole rationale on. (Engraved artwork is unchanged; glyph `dot` positions do not depend on `center`. It is the plan and the guard that change.)
**Fix:** Make the **separate `ConstantStringer` instance for the passphrase alphabet the default**, not the conditional retreat — it satisfies §7 and D5 unconditionally and costs nothing. If the shared alphabet is widened anyway, §7 must say which goldens are expected to change and why, and O5 must be broadened from "measure `runeDuration`" to "measure `runeDuration`, `startEndDist` and `center`".

### I4 — §3.4's check table is again presented as complete and is not; `constantAlphabet` has an unstated ordering constraint
**Where:** §3.4, §3.5.1
**Defect:** §3.4 asserts "there are **THREE** independent charset checks on the path, not one… All three must be satisfied or the device panics". There are **five**, and the two omitted ones both fire in `NewConstantStringer` — i.e. for *every* plate type, not just this flow:
- `engrave.go:1210` `panic("unsorted alphabet")` — the alphabet must be in ascending codepoint order (`sort.Find` at `:1282` depends on it). §3.5.1 says "all 95 printable ASCII **plus** the visible-space mark", and §3.3 puts the mark at `0x00–0x1F` — i.e. it must come **first**, not appended. The spec never says so.
- `engrave.go:1215` `panic("unsupported rune")` — every rune in `constantAlphabet` must decode in the face at construction time. This makes the font work and the alphabet widening a single atomic change: extending `constantAlphabet` to 95 characters before the 44 glyphs exist bricks `NewConstantStringer` for seed plates too. The spec's §3.2 (author glyphs) / §3.5.1 (widen alphabet) split invites exactly that staging.
**Failure:** Both are loud construction-time panics, so neither can reach a plate — but §3.4 was rewritten specifically to stop being a false-completeness claim ("An earlier draft… That was **false**, and is corrected here"), and a reader will trust the table. The ordering constraint in particular is invisible until the panic.
**Fix:** Two sentences. Add rows 4 and 5 to the §3.4 table (`engrave.go:1210` ordering, `engrave.go:1215` alphabet⊆face), and state in §3.5.1 that the alphabet is a single ascending-codepoint string with the mark first, extended in the same commit as the glyphs.

### M1 — §3.2.1's confusable list omits the case-only pairs, which are the class this feature creates
**Where:** §3.2.1, O1
**Defect:** The enumerated pairs are lowercase-vs-digit/punctuation collisions. The class the plate actually introduces is `C/c O/o S/s U/u V/v W/w X/x Z/z K/k` — letters whose lowercase form is the same stroke path at x-height instead of cap height, distinguished by size alone (measured ratio 4 units vs ~2.67 at 6 mm em). D5 fixes the metrics, so these are same-shape-by-construction unless the author deliberately differentiates. The plate is case-sensitive with no checksum, so `C`→`c` is a different wallet. Non-blocking because the list is prefaced "At minimum" and O1 says "including", so the requirement is not falsely closed.
**Fix:** Add the case-only pair class to the §3.2.1 enumeration so O1's by-eye check covers it.

### N1 — cite imprecision in §3.5.2
`bitmapForQRStatic` is `engrave/engrave.go:384-401`; the cited `:406-413` is `ConstantQR`'s guard. Substance correct.

### N2 — glyph-origin trap for new glyphs
`paddedString` uses `inf.Start != (bezier.Point{})` as the sentinel for "has a leading move segment" (`engrave.go:1288-1295`). All 52 current glyphs keep X ∈ [100,500] so this is safe; a new glyph drawn with its first engraved point exactly at the origin (plausible for `_`) would take the wrong branch. Worth one line in the font-authoring requirements.

## CONSISTENCY

- **§3.4's three-check table vs §3.5 — FAIL.** The table and §3.5 agree with each other, but both omit the two construction-time checks at `engrave.go:1210` and `:1215` (finding I4).
- **§4.1/§4.2 height budgets vs §4.3's relocation — FAIL on arithmetic.** Intent is consistent and clearly stated (budgets cover text + QR only, metadata lives in the bands); the bands cannot hold the three lines §4.3 assigns to the top one (finding I2).
- **§1.1's 100-char max vs §3.5.2's 78-char fallback — PASS.** The cap is an explicitly conditional, O6-gated, user-visible retreat, not a competing limit. Nit: §1.1's table would read better with a footnote pointing at it.

Also checked and clean: §3.5.1's uniform-advance premise is airtight — I probed the compiled face and **all 52 supported glyphs, including space, have advance 600** with `Metrics{Ascent:800, Height:900}`, so widening the alphabet cannot trip `panic("variable width font")` at `engrave.go:1218`; space's zero-knot spline degrades correctly through `paddedString` (empty engrave, full `Delay` padding), so including `0x20` in the alphabet is safe.
