# Phase B2a plan — R0 round 2 (independent fold verification, sonnet)

Artifact: `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a.md` at commit `6f8b5c4`.
Scope: **did the round-1 fold correctly apply each of the 9 findings, and are the factual claims it added TRUE?** Mechanical verification, not a design review.
Primary object: `git diff bdec841..6f8b5c4`.
Fork under review: `/scratch/code/shibboleth/seedhammer` @ `78949e7`.
Tier: sonnet, per the reviewer-tiering rule — rounds 0 and 1 (opus) did the design-level judgment; what remained was mechanical.
Persisted verbatim, before any fold. HTML entities from the transport layer have been restored; nothing else is altered.

---

VERDICT: 0 Critical / 1 Important / 0 Minor / 2 Nit

| finding | applied? | verified how |
| --- | --- | --- |
| I1 | **PARTIAL** | §6c's three claims all correctly fixed and verified TRUE against the fork: `platform_sh2.go:398-417` (loop only returns touch-derived `gui.PointerEvent`, or `p.stdin` — which only gets a writer if the `debug` build tag sets `initHook = dbgInit` in `debug_sh2.go:13`); `debug_sh2.go:82-83` is the only `gui.ButtonEvent{` producer in the repo (aside from a directional-repeat synthesis in `gui/gui.go:127` restricted to Up/Down/Left/Right, irrelevant to `Button2`/`Center`); `gui/gui.go:2467` `op.Input(&ctx.B, &s.words[i])` confirmed verbatim; `gui/event.go:155-159` confirmed `buttonEvent` case checks only `e.Button != f.btn`, no bounds check; `gui/widget.go:70` confirmed `Clickable.Clicked` calls `ctx.Router.Next(ButtonFilter(c.Button), ButtonFilter(c.AltButton), PointerFilter(c))` regardless of layout. **But** I1's own WHERE field named **both** "§6b comment and §6c" — and the identical false claim ("on a touch-only SH2 a CENTRE TAP opens the word editor") still ships unfixed in §6b's whole-file block, `unlockEngraveMnemonic`'s comment (plan line 2104). See defect below. |
| I2 | YES | Task 5.3 (lines 1855-1862) adds "write the vector-E negative... Task 8's `ct_len == 0` row names this test," **and** Task 8's row (line 2431) is repointed by name to "Task 5.3's vector-E negative." Both halves present. |
| M1 | YES | Global Constraints bullet (lines 153-162) now correctly attributes the seam to `newDeriver`/§5c, not Task 3. Verified in fork: `Opener.Unlock` (`seal/open.go:176-205`) still uses `o.KDF`, but the plan's `UnlockWithKey` (line 1241) takes a pre-derived key and never touches `Opener.KDF` — confirmed by reading its body. Measured `seal.DeriveKey(..., 100000)` on host: **~10.4ms** (3 runs, 10.37/10.53/10.40ms), confirming "tens of milliseconds." `kdfStepIterations = 500` and the "≥256 frames" figure are internally consistent with the ≥200-frame floor stated at line 1865. |
| M2 | YES | New C1 text ("`SecretsResident()` goes false when the LAST secret's plate is built") verified against plan code: `SecretsResident` (line 1348) scans **all** `p.Secret` records; `unlockSecretSession` (line 1982) offers/wipes one at a time via `unlockSecretPlate`, whose `defer p.WipeSecretAt(i)` (line 2001) only wipes index `i`. SPEC §10.2.2 line 1268-1269 quoted exactly: "~21 minutes for single-sig, ~63 for a 2-of-3." Vector counts re-measured from `seal/testdata/vectors.json`: vector F's secret section has exactly 3 `ms1` + 12 `mk1`/`md1` records; vector G's secret section is 3 records, all `ms1`. Matches "Vectors F and G each carry three `ms1` records" exactly. |
| M3 | YES | Task 7.1 (line 2377) now reads `sealBlobForTest`, with the explanation that `sealForTest` is unreachable from `package gui`. |
| M4 | YES | `gui/unlock_session.go` header comment (§6b, lines 1949-1952) now reads "...ONCE THE ENGRAVE SCREEN IS LEFT. A plate merely PAUSED resumes from the spline..." — in the correct shipped-comment location. |
| N1 | YES | C1 section now quotes `newEngraverJob(ctx.Platform, plate.Spline, plate.Conf, 0)` (line 206) — matches `gui/gui.go:2633` in the fork byte-for-byte. |
| N2 | YES | Task 5.3 (lines 1870-1875) adds "but only for tests that do not need the unlock to SUCCEED... it is wrong for Task 7's `(sealed)` test." |
| N3 | YES | Task 5.1 (lines 1834-1838) now includes "Also write `gui/seal_fixture_test.go` and assert it round-trips... in both shapes," placed before 5.2. |

---

## [I1] The false "centre tap" claim I1 flagged is still shipping into the fork, in the location I1's own WHERE field named

**WHERE:** Plan §6b's whole-file block, `unlockEngraveMnemonic`, line 2104 (`design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a.md`).

**CLAIM (unchanged, still in the folded plan):**
> `// ship without it: on a touch-only SH2 a CENTRE TAP opens the word editor, / // and editing an authoritative payload seed produces a self-consistent plate / // that does not restore the payload's wallet.`

**DEFECT:** This is the exact false claim I1 recorded as false and fixed in §6c — but §6c is a *different* location from where this sentence actually lives. The round-1 report's own "WHERE" field reads "Plan §6b comment and §6c," naming both locations up front. The fold fixed §6c (rationale, test, guard placement, all independently verified true against the fork above) but left the identical sentence untouched in §6b, inside the whole-file Go block that will ship verbatim into `gui/unlock_session.go` as a production source comment. Grepping the folded plan for "CENTRE TAP" / "touch-only SH2" finds it at line 2104 (unfixed) and line 2162 (the §6c blockquote correctly explaining it was false) — one of the two locations the finding named is still wrong.

**CONSEQUENCE:** A shipped, security-adjacent comment in the actual codebase asserts a false threat model ("centre tap opens the editor") for the exact guard I1 required — the same class of defect M4 fixed elsewhere in this same fold (a shipped comment stating the opposite of what the plan decided). Low operational risk since the comment sits beside — not instead of — the correct §6c guard code, but it is exactly the "reads as authoritative to the next reader" pattern this feature keeps re-teaching, and it is the one place the build gate cannot catch (a comment, not code).

**FIX:** Replace lines 2104-2106 with the corrected rationale already written for §6c: "Do not ship without it: on a touch-only SH2 the edit nav button (`Button2`, or a tap on its slot) opens the word editor, and editing an authoritative payload seed produces a self-consistent plate that does not restore the payload's wallet."

## [N] Two small citation drifts in I1's newly-added evidence

- `gui/gui.go:2330` (line 2156) — the actual `if editBtn.Clicked(ctx) {` in the unmodified fork is at **line 2331**, not 2330 (verified by direct read). Off by one.
- `gui/event.go:153-159` (line 2158) — the `buttonEvent` case (the substantive evidence for "no bounds check") is at **lines 155-159**; 153-154 is the tail of the preceding `pointerEvent` case. Close but not exact.

Neither changes the substance of any claim; both are inside the passage this brief asked to scrutinize hardest, so noted for completeness.

---

Everything else — I2, M1, M2, M3, M4, N1, N2, N3 — is correctly and completely applied, and every factual claim I checked in the fold's volunteered text (the I1 chain of PointerEvent/ButtonEvent production, the M2 residency mechanics and SPEC/vector figures, the M1 KDF timing, measured directly) is true. The single Important above is narrowly scoped: I1's fix is complete in §6c but incomplete in §6b, at the exact second location the finding's own report already pointed to.
