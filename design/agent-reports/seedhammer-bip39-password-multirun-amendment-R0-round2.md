# Sonnet verification — R0 round 2 — §3.5.0 multi-run amendment

- **Under review:** `SPEC_seedhammer_engrave_bip39_password.md` §3.5.0 / §3.4 @ `beb4a3b`
- **Rounds:** 0 = NOT GREEN (2C/4I), 1 = NOT GREEN (0C/1I), this = round 2
- **Brief:** did the round-1 fold fix I-1 and the minors, and did it introduce a new defect? Verification pass, not an audit; a clean result closes the gate.
- **Model tier:** sonnet, per the project rule — the design questions were settled in rounds 0-1, leaving a mechanical fold check.
- **Date:** 2026-08-03

Persisted verbatim.

---

## VERDICT
GREEN (0C/0I)

## FOLD VERIFICATION

**I-1.1 FIXED** — §3.5.0 states `T_row = rowLen + n_row` with `rowLen` **10** in §4.1 and **20** in §4.2 (lines 461-464), and §7(c) asserts the bound for both (939-940). Matches §4.1's "10 rows × 10 characters" and §4.2's "5 rows × 20 characters" exactly.

**I-1.2 FIXED** — Verified against committed `engrave.go` (HEAD `211e896`): `mid2 := longest + shortest - 1` at 1339-1340, `dot = bezier.Pt(...)` at 1341, unpadded `cont := yield(Move(pen))` at **1292** exactly as cited. The spec now states `L` is disclosed exactly, and explicitly resolves the prior self-contradiction with I4's "leaks `L` exactly" note (478).

**I-1.3 FIXED** — conditional-legend bit stated as a disclosure item (481-485): presence/absence leaks "whether the passphrase contains at least one space."

**I-1.4 FIXED** — QR-version leak stated (486-490): module count (33/37) varies with byte length and alphanumeric-subset membership; `ConstantQR` constant-time only *given* the version.

**I1-residue FIXED** — `(v) maxDur (:1255)` enumerated as a fifth required change (384-385); verified at line 1255 in committed source. The intra-glyph inter-run move is pinned normatively to `advDur` (390-394) with the leak-upgrade rationale stated.

**I4-residue FIXED** — restated precisely as "`PaddedString` MUST NOT be called with `shortest != longest` on a multi-run alphabet," with a construction-time guard specified (511-517).

**I3-residue FIXED** — §7's zero-run bullet (942-947) no longer claims the real plate path exercises it, and asserts the normative slot cost (`advDur + runeDuration`), not merely no-panic.

**M1 FIXED** — all eight site/panic pairs match expected values exactly, verified against `git show HEAD:engrave/engrave.go`: `1386/1388`, `1305/1309`, `1240/1241`, `1232/1233`, `1237/1238`, `1180/1181`, `1076/1077`, `1073/1074`. (A naive grep of the *working tree* shows these ~10 lines higher — traced to an uncommitted concurrent Task 5 edit in the same file. Against committed HEAD, which is what the spec cites, every pair is exact. Not a spec defect.)

**Minors — PARTIAL.**
- "Four reducible glyphs (`x # * $`)" — FIXED (396), consistent with the table's 4+1+2+6=13 split.
- Check-count consistency — **NOT fully fixed.** §3.4 correctly says "Eight are known" (264, 295), but §3.5.0's "Consequences for §3.4" (524-527) still says "the count is now **six** known checks" alongside "table gained rows 6-8". Residual Minor.
- N1 imperative voice — **NOT fully fixed.** Same sentence mixes past tense ("gained") with imperative ("drop").

## NEW FINDINGS
None.

## CONSISTENCY
- §3.5.0 disclosure vs §4.1/§4.2 row length — **Pass**.
- Round-1 additions vs §3.5.1.1 / Required-code-changes — **Pass**, no contradiction.
- §3.5.1.1 pre-`adff081` cites vs HEAD — **Fail (Minor, not addressed).** §3.5.1.1 cites `1208-1210`/`1213-1215`/`1216-1218`/`1294-1296` for checks now at `1232-1233`/`1237-1238`/`1240-1241`/`1319`. Document mixes two numbering bases. Not blocking.

Both residual Minors are documentation-only — the normative tables and code-change enumerations they describe are correct and verified against source. **Gate closes GREEN; implementation may resume.**
