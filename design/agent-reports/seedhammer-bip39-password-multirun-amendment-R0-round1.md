# Opus architect — R0 round 1 — §3.5.0 multi-run amendment (fold verification)

- **Under review:** `SPEC_seedhammer_engrave_bip39_password.md` §3.5.0 / §3.4 @ `9a926f8`
- **Round 0:** NOT GREEN (2C/4I) — `…multirun-amendment-R0-round0.md`
- **Brief:** did the fold fix each round-0 finding, and did it introduce a new defect? Round 0's connectivity classification and clean-list declared established; the user's re-acceptance of the row-granularity disclosure declared out of scope, but its *correctness* in scope.
- **Date:** 2026-08-03

Persisted verbatim before folding.

---

## VERDICT

**NOT GREEN (0C / 1I)**

Both Criticals genuinely closed. One Important: the corrected disclosure is still not complete or accurate against §4.

## FOLD VERIFICATION

**C1 FIXED** — row-granularity restatement accurate on every point checkable against source. `stringColumn` (`backup/backup.go:268-276`) issues exactly one `constant.String` call per `groupLen=10` group. The three bookend blocks are real and distinguishable: un-padded `Move` (`engrave.go:1292`, scaled 1:1 because `ts.rem==0` → `ts.Reset(dur,dur)` at `:1038-1040`), `centerDur` (`:1296-1297`), `padDur` (`:1294`, applied `:1343`). All differ from `advDur`. `n_row = T_row − 10` is exact for a full row. The "do not restate as an aggregate" guard rail is a good addition. See I-1.

**C2 FIXED** — mechanism traced end to end and correct. `DelayMove(yield, conf, totalDur+runeDuration, pen, start)` emits `Delay(moveDur, totalDur+runeDuration)` then `Move(start)` (`:226-230`). `Reset` cannot trip `invalid scale` because `moveDur ≤ advDur ≤ totalDur` is the existing inter-slot invariant. The `Move`'s 3 clamped knots hit `len(spline)==5 → appendLine` (`:1021-1023`), whose `computeSCurve` durations sum to exactly the `timeMove` denominator, so `rem` drives to 0 — no `unaligned delay`, and `ts.Done()` at the next slot — no `scale already in effect`. Zero-distance sub-case also works via the `len(knots)==0` filler (`:1141-1143`) and the `denom==0` branch (`:1093-1095`). Timing identical to a normal slot. Both assertions sufficient and mutually necessary — `passphraseAlphabet` (`:772-775`) does contain `0x20`. *Nits:* call omits the leading `yield` arg; names `dot` where the code's target is `start`; "the `denom == 0` path then applies correctly" mis-attributes — the `denom != 0` path normally applies, and works.

**I1 FIXED (Minor residue)** — (i)-(iv) all correct against source. Confirmed (ii) is not merely necessary but *sufficient*: with bounds over every run, `dist(pen,start) ≤ advDist + startEndDist` still holds for both inter- and intra-glyph moves. No fifth *external* consumer — `constantPlan`/`constantRune`/`timeConstantPath` appear only in `engrave.go`. Residue: (a) `maxDur = max(inf.Duration, maxDur)` (`:1255`) is a fifth code site that must iterate the slice; (b) **the pad target for the intra-glyph inter-run move is never stated normatively** — if padded to anything other than `advDur`, the leak upgrades from a per-row count to per-row *positions* of the eight glyphs, materially worse than what was accepted; (c) §3.5.1.1 was not updated as round 0 asked.

**I2 FIXED** — decidable predicate. The absolute 181080 is em/`StepperConfig`-dependent and the em isn't stated, but the constraint is em-relative and survives.

**I3 FIXED (Minor residue)** — all three properties present. The zero-run bullet contradicts itself: "through the real plate path" is unreachable (§3.3 strips `0x20`), then "must exercise the stringer directly" — the latter is correct, delete the former. It also asserts no-panic only, not the normative content (the slot costs exactly `advDur + runeDuration`).

**I4 FIXED (Minor residue)** — analysis right; nothing in §4's passphrase layout routes through `PaddedString` (`stringColumn` → `String`; only `wordColumn` uses `PaddedString`, on the shared single-run instance). Minor: the prohibition is self-defeating — `String` *is* `PaddedString(yield, txt, n, n)` (`:1272-1275`). Precise rule: "MUST NOT be called with `shortest != longest` on a multi-run alphabet", and per this spec's own doctrine it deserves a guard, not a bare MUST NOT.

**M1 NOT FIXED** — the drift was re-broken in a new direction: round 0 supplied *panic* lines, the fold put them in the **Site** column and invented a Panic column. Correct values preserving the original convention: `1386/1388`, `1305/1309`, `1240/1241`, `1232/1233`, `1237/1238`, `1180/1181`, `1076/1077`, `1073/1074`.

**M2 FIXED** — rows 7/8 accurate; construction/engrave split now an explicit column and right for all eight rows. Related-panic cites verify.

**M3 FIXED** — row 1 correctly labelled metadata-only; `paddedString`'s `panic("unreachable")` (`:1315`) named as the passphrase's decode gate.

## NEW FINDINGS

### I-1 (Important) — the corrected disclosure still misstates §4, and its enumeration is incomplete on three further channels

1. **§4.2 contradicts the row length.** §3.5.0 fixes `T_row = 10 + n_row` and §7(c) hard-codes 10, but §4.2 mandates **5 rows × 20 characters** for the QR layout. Wrong for one of two normative layouts — same class as C1 (a claim about §4 that §4 doesn't support), though here it errs conservative. Fix: `T_row = rowLen + n_row`, `rowLen ∈ {10, 20}`.
2. **`L` is disclosed exactly, not "to within the final row".** `paddedString`'s park is length-dependent — `mid2 := longest + shortest - 1; dot = Pt(mid2*advDist/2, baseline)` (`:1340-1341`). The move *to* park is padded, but the **next** element's approach is not (`:1292`), so its duration is a function of the previous row's character count. For the final partial row that yields `L_last` exactly — so `L` is exact and `n_last` becomes exact. The spec's own I4 paragraph already says the `String` path "leaks `L` exactly", contradicting the bullet.
3. **The conditional legend leaks a content bit.** §4.3: the legend is engraved "whenever the passphrase contains a space". Presence/absence is a large, positionally distinct, non-constant-time block → a timing-only observer learns "contains ≥1 space".
4. **With QR on, the QR version leaks a length bracket and a charset class.** Module count varies 33/37 with byte length *and* alphanumeric-subset membership; `ConstantQR` is constant-time given the version, but the version is chosen from content.

None of 2-4 is created by the amendment and none is larger in kind than what was accepted — hence Important, not Critical. But the section is framed as "what an observer actually recovers" and carries a standing instruction not to under-state.

## CONSISTENCY

- **Disclosure vs §4 — FAIL** (I-1.1).
- **I2 budget vs `2L` — PASS.**
- **Required code changes vs §3.5.1.1 — PASS (no contradiction), Minor gap:** §3.5.1.1 not updated; origin constraint still per-glyph; its cites are all pre-`adff081`, so the document now mixes two numbering bases.
- **Internal counts — FAIL (Minor).** "FIVE charset checks" (`:263`) vs an eight-row table vs "eight KNOWN checks" vs §3.5.0's "sixth row … six known checks" in imperative voice for an applied change (round 0's N1 unfixed). Rows 7-8 are timing, not charset, checks — the framing needs rewording, not just a number.
- **`%` classification — FAIL (Minor).** Table lists four Reducible (`x # * $`) with `%` Resolved at k=2, but the prose still says "The **five** reducible glyphs … MUST be redrawn as single strokes" — which would send a font author back to make `%` single-stroke, settled at 2 parts by `211e896` after two failed attempts.
