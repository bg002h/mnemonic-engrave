# Opus architect — R0 round 0 — §3.5.0 multi-run glyph amendment

- **Reviewer role:** adversarial opus architect, R0 gate on a spec **amendment** before implementation resumes.
- **Under review:** `design/SPEC_seedhammer_engrave_bip39_password.md` §3.5.0 (new) and §3.4 (revised), @ `8a22180`
- **Brief:** is the amendment correct, complete and safe to implement? Original D1–D8, the charset, and the requirement to use constant-time primitives declared settled. The user's *acceptance* of the `T = L + n` disclosure declared out of scope — but whether the spec states it **correctly** was explicitly in scope.
- **Date:** 2026-08-03

Persisted verbatim before folding, per the project standard.

---

## VERDICT

**NOT GREEN (2C / 4I)**

## CLAIM VERIFICATION

1. **Disclosure analysis — FALSE.** The arithmetic (`T = L + n`, `T ∈ [L,2L]`, `L ∈ [T/2,T]`) is internally correct given max k=2, but the threat statement is not. §4 engraves the passphrase as 10-character rows through `stringColumn` (`backup/backup.go:268-276`, one `constant.String` call per `groupLen=10` group), and `paddedString` bookends every call with distinguishable blocks — un-padded `Move` (`engrave/engrave.go:1292`), `centerDur` first block (`:1296-1297`, measured 18216), `padDur` last block (`:1343`, measured 22948) vs `advDur` between glyphs. Rows are therefore separable in the tick stream and `L_row = 10` is public by construction. See C1.
2. **`k = max(runs,1)` — FALSE.** Measured: `newConstantStringer(constant.Font, params(), 1000, " 0123456789")` constructs fine, then engraving `"0 1"` panics **`scale already in effect`** (`engrave/engrave.go:1077`). See C2.
3. **Connectivity rule — CONFIRMED.** Verified against `font/constant/constant.svg` *and* the compiled font. Exactly 13 runes have ≥2 engrave runs (`#`=4, `*`=4, `%`=3, the other ten =2), plus `0x20` with 0. No glyph is misclassified in either direction; the "8 irreducible" count is right.
4. **Implementable against real code — IMPRECISE.** Implementable, but only after four changes/invariants the spec does not name (I1), and one step is not implementable as written (C2).
5. **Interactions with settled work — IMPRECISE.** §3.5.1 survives cleanly (verified empirically); §4 does not (C1); §7 was not updated (I3); `PaddedString`'s stronger guarantee is silently lost (I4).
6. **§3.4's six — CONFIRMED accurate, IMPRECISE cites.** All six name real panics at HEAD (`:1388, :1309, :1241, :1233, :1238, :1181`), but the table mixes pre- and post-`adff081` line numbers (M1). A seventh **does** exist and it is the one §3.5.0 trips (M2).

## FINDINGS

### C1 — The disclosure is stated per plate; the layout leaks it per 10-character row, where `L` is public by construction

**Where:** §3.5.0 "The accepted disclosure" (against §4).

**Defect:** The spec's three load-bearing claims — "An observer measures **only `T`** — a blend", "Extracting `n` requires already knowing `L`", and "in the no-side-knowledge case this discloses *less* about length" — are all false under §4's own layout. §4 mandates "one 10-character group per row, reusing the existing column machinery"; that machinery is `stringColumn` (`backup/backup.go:268-276`), which issues **one `ConstantStringer.String` call per row**. Each call is bracketed by blocks whose durations differ from the intra-row `advDur` (`engrave/engrave.go:1292`, `:1296-1297`, `:1343`), so rows are trivially segmentable from ticks alone.

**Failure:** The attacker does not need "independent knowledge of the length" — §4 hands it to them: every full row is exactly 10 characters ("position implies index"). They read `K_row` per row and recover `n_row = K_row − 10` **exactly**. For a 100-character passphrase that is ten exact counts of `= " i j ! : ; ?` per decade, plus `ceil(L/10)` from the row count itself — not one aggregate blend, and not "less about length than today". The stated attack precondition is void and the disclosed quantity is ~10× richer than the spec admits.

**Fix:** Restate the disclosure at the granularity it is actually observable: `T_row = 10 + n_row` with `L_row` public, so `n` is disclosed per 10-character window and `L` to within the last row. Then either re-obtain the user's acceptance against *that* statement, or engrave the whole passphrase through one padded call so only the aggregate is observable.

### C2 — `k = max(runs, 1)` does not close the zero-run hole; it reproduces an engrave-time panic, and its stated baseline is false

**Where:** §3.5.0 "Zero-run glyphs — space MUST cost one unit".

**Defect:** The spec asserts "Under the per-glyph model it still costs a full unit (an empty engrave plus full `Delay` padding) … Space therefore costs one unit, **as it does today**." It does not. `Decode(' ')` returns advance 600 with an **empty spline** (`cmd/vectorfont/main.go:331-333` sets `Index[' '] = Glyph{Advance: meta.Advance}`, Start=End=0), so `paddedString` emits `Delay(0, runeDuration)` (`engrave/engrave.go:1328`) followed by **no knots at all**. `timeScaler` is left holding `rem = runeDuration`, and the next `DelayMove`'s delay hits `Reset`'s `if s.rem > 0` guard (`:1076-1078`). The `denom == 0` special case at `:1093-1095` rescues a zero-*distance* move (which still emits knots), not a zero-*knot* spline. Measured directly: panic **`scale already in effect`**.

**Failure:** (a) The rule's justification rests on a behaviour that does not exist. (b) "emits one empty padded run", implemented literally, *is* the panicking sequence — so the rule as written is unimplementable. (c) The panic fires at **engrave** time, not construction: `NewPassphraseStringer` builds, §7's alphabet test passes, and the device crashes when a plate is engraved. It is masked today only by §3.3's `0x20 → 0x1F` translation — an invariant §3.5.0 never states as load-bearing while simultaneously forbidding removal of `0x20` from the alphabet.

**Fix:** Specify the mechanism instead of the outcome: for a zero-run glyph emit a single `DelayMove(conf, totalDur + runeDuration, pen, dot)` and **no** separate `Delay`, so the move's knots absorb the whole unit (the `denom == 0` path then applies correctly). Add a construction-time assertion that no alphabet rune has zero runs unless that path is taken, and state explicitly that `0x20` must never reach the stringer.

### I1 — The amendment does not name the code changes it requires, and three of them are invariants

**Where:** §3.5.0 "The rule" ("reuses the existing padding machinery … a smaller change").

**Defect:** Four required changes are unstated:
- (i) `timeConstantPath` (`engrave/engrave.go:1170-1189`) must return one `constantPlan` **per run**; `constantRune.Info` (`:786`) becomes a slice.
- (ii) `newConstantStringer`'s bounds accumulation (`:1247-1250`) must cover **every** run's start/end, not the first start and last end — `startEndDist`/`center` bound every padded move (`:1294-1296`).
- (iii) `paddedString` must split the spline at run boundaries, and the boundary is encoded as a `Line`-flag flip **inside a shared clamped control-point triple** (measured on `:` — the run-1 end triple is flagged T,T,F and the run-2 start triple F,F,T). The existing `for range 3 { spline.Next() }` skip (`:1320-1324`) does not generalise, and §3.5.1.1's "no glyph may start at (0,0)" must now apply to *every run's* start.
- (iv) `dot.X` must advance once per **glyph**, not per run (`:1336`), or §4's "position implies index" breaks and the plate is drawn wrong.

Additionally each run's `Delay` denominator must equal that run's flush duration exactly, or `timeScaler` panics `unaligned delay` (`:1099`) / `scale already in effect` (`:1077`).

**Fix:** Enumerate (i)-(iv) in §3.5.0 as normative, and add (ii) and (iv) to §3.5.1.1's constraint list.

### I2 — "Reduce k before quantizing" can raise `runeDuration`, and that is unbudgeted

**Where:** §3.5.0 "Reduce k before quantizing".

**Defect:** Measured `runeDuration` = 181080 ticks, set by `8`. The mandated single-stroke redraws convert pen-up moves (move speed) into retraced engraving (engraving speed) plus extra retraced length. `$` is today runs `[135176, 22948]` plus a 20852-tick move = 178976 total; a single-stroke `$` must retrace much of the S to reach the bar and plausibly exceeds 181080, becoming the new `runeDuration` — which multiplies the cost of **all 96 glyphs**, since every run is padded to it. The spec's "worst case is bounded at `2L`" is expressed in a unit the redraw can itself inflate, and the cost comparison that justifies per-run over the rejected uniform-2-unit scheme (also `2L`, but fully constant) then no longer holds.

**Fix:** Require `runeDuration` to be re-measured after the redraws with a hard budget — the redrawn glyphs must not become the longest single run — and restate the worst case in absolute time.

### I3 — §7 gains no test for the new normative timing behaviour

**Where:** §7 / §3.5.0.

**Defect:** The amendment changes constant-time semantics and §7 is unchanged. Nothing asserts (a) that two passphrases of equal length with equal multi-run counts produce identical `ProfileSpline` patterns, (b) that per-run blocks are uniform so no *position* leak arises within a row, or (c) the disclosure bound itself. The idiom already exists — `TestConstantWords` (`engrave/engrave_test.go:192-215`) asserts `refProf.Equal(prof)` across all BIP-39 words, and that is precisely the assertion per-run quantization weakens; the weakened form must be written down or the property is unguarded. Separately, §7's "No-panic guarantee" bullet does **not** catch C2, because §3.3's translation strips `0x20` before layout.

### I4 — `PaddedString`'s stronger guarantee is silently lost, and "today leaks `L` exactly" is only true of `String`

**Where:** §3.5.0 "The accepted disclosure".

**Defect:** `paddedString` runs exactly `longest` slots regardless of content, repeating runes to fill (`engrave/engrave.go:1303-1338`). Today that makes a `PaddedString` call's duration independent of the string *including its length within `[shortest, longest]`* — the property seed plates rely on and `TestConstantWords` asserts. Under per-run, `T = Σ k(rune at slot)` over `longest` slots, which depends on content **and** on which runes get repeated. So (a) "Today's scheme leaks `L` exactly" is false for the `PaddedString` path (it leaks nothing there), and (b) the amendment silently removes a guarantee of an exported API. Harmless today only because `constantAlphabet` contains no multi-run glyph.

**Fix:** State the changed guarantee, and forbid `PaddedString` on a multi-run alphabet (or specify what it now guarantees).

### M1 — §3.4's table mixes pre- and post-`adff081` line numbers
Rows 1-5 cite the pre-commit file (`1363/1365`, `1282/1286`, `1216/1218`, `1208/1210`, `1213/1215`); row 6 cites post-commit (`1178/1181`). At HEAD they are `1388`, `1309`, `1241`, `1233`, `1238`, `1181`.

### M2 — the seventh check exists, and it is the one §3.5.0 trips
`timeScaler.Reset` panics `invalid scale` (`engrave/engrave.go:1073-1075`, actual duration exceeds its pad target) and `scale already in effect` (`:1076-1078`, a padded block left unconsumed) — plus `delay during spline` (`:1052-1054`), `unaligned delay` (`:1098-1100`) and `paddedString`'s `unclamped spline` (`:1322`). Unlike rows 3-6 these fire at **engrave** time; C2 is an instance. The table should gain at least the `timeScaler.Reset` row and mark the construction/engrave split.

### M3 — row 1 of §3.4 points at `engrave.String`, which §3.5 forbids for the passphrase
`engrave.go:1363/1365` is `StringCmd.engrave`. The secret path's decode check is `paddedString`'s (`:1311/:1315` at HEAD, `panic("unreachable")`). Row 1 is correct for the metadata lines but mislabels which gate protects the passphrase.

### M4 — the `%` redesign is asserted, not validated
`%` is 3 parts (measured), and the only route to k=1 is moving both dots onto the slash — a glyph whose legibility and confusability nothing checks (§3.2.1's list and O1/O3 don't cover it). If the redesign is rejected on legibility grounds, `%` stays at k=3 and "max k = 2 / worst case `2L`" is false.

### N1 — §3.5.0's "Consequences for §3.4" is written in the imperative for a change already applied
§3.4 already carries the sixth row and the non-exhaustiveness disclaimer.

## WHAT I CHECKED AND FOUND CLEAN

- **Connectivity classification is exactly right**, verified against both the SVG and the compiled font. `x`'s two strokes cross at (447,6) — each control polygon is 180°-symmetric about that point, so both curves pass through it; `#`'s verticals cross both horizontals; `*`'s four lines are concurrent at (291,3); `$`'s bar crosses the S's middle segment at (477,5). `%`'s dots genuinely miss the slash (at y∈[2,3] the slash is at x∈[484.3,485] vs dot1 x∈[481,482]; at y∈[7,8] it is at x∈[481,481.7] vs dot2 x∈[484,485]). All eight of `= " i j ! : ; ?` have a truly detached part. Counts 13 and 8 are both correct; `#`/`*` are 4 parts as stated.
- **Retracing precedent holds** — uppercase `X` (`font/constant/constant.svg:30`) already retraces, so single-stroke redraws match the font's existing idiom.
- **Intra-glyph moves fit the existing pad budget.** Max measured intra-glyph move is 666 machine units (`#`, `$`, `%`) against `advDist + startEndDist` = 666 + 888 = 1554 for the passphrase alphabet, so `advDur` bounds every intra-glyph move and `invalid scale` will not fire on that account.
- **Every run duration is ≤ the current `runeDuration`** (181080) before the redraws, so `Delay(runDur, runeDuration)` is well-formed per run.
- **§3.5.1's separate-instance argument survives the amendment.** Every glyph in the shared `constantAlphabet` is single-run, so a per-run `timeConstantPath` is a no-op there. Verified empirically: I patched `timeConstantPath` in a scratch clone and `go test ./engrave/... ./backup/... ./font/... ./stepper/...` was green with no `-update`.
- **All six §3.4 checks name real panics** that exist at HEAD.
- **The rejected alternative is correctly costed.** "Uniform k-unit padding for every glyph … doubles engraving time" is accurate (2L blocks, genuinely fully constant). I also measured the obvious third option — padding each *glyph's total* to `runeDuration` — and it is free (max glyph total including intra-glyph moves is 181080, exactly `8`'s single run), **but it is not fully constant**: the engrave/move transition pattern inside the slot becomes content-dependent, leaking glyph *positions* rather than a count. So the spec's rejection stands and per-run is the better of the two.
- **§3.5.2 (QR) is untouched** by the amendment.
