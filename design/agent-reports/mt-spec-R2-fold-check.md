# mt spec — R2 fold-verification pass

Scope: mechanical fold-check, not a fresh audit. Artifact: `design/SPEC_mt_v0_1.md`
at commit `b1790a4` (= current HEAD; working tree verified byte-identical to
`git show b1790a4:design/SPEC_mt_v0_1.md`). Commit range examined: `e0bbc27..b1790a4`
(30 commits). Reports read in full: all seven R0/R1 lens reports plus the two
push-log notes in `design/agent-reports/`.

Machine-gated facts taken as given, not re-derived: `plan-cite-check.sh` → 26/26
resolved, 0 dangling (re-ran, confirmed); `spec-structure-check.sh` → 15 sections,
33 cross-refs, STRUCTURE OK (re-ran, confirmed).

## Verdict

- **Commit-claim audit:** 24 spec-content commits examined. **23 verified**
  against current spec text. **1 FALSE CLAIM found**: `83d2a72` claims *"Every
  '136' in the spec updated to 130"* — one instance survives unchanged (line 461).
  This is a third instance of the failure mode `2924903`/`14ddab0` document,
  uncaught by any later commit.
- **Contradictions found: 3, all Important, 0 Critical.** All three are
  incomplete-propagation defects: a later ruling landed correctly in the section
  it targeted and was never carried into a sibling section stating the same fact.
- Severity: 0 Critical / 3 Important / 1 Minor (the arithmetic slip in
  `feb56a4`'s commit message, noted in the table, not a spec-text defect).

## Commit-claim audit

| commit | claim | verified? |
| --- | --- | --- |
| `b1790a4` | New `spec-structure-check.sh`; fixed 1 real dangling ref (§11's stale §6c/§6d) + 2 script blind-spots (§6a); rewrote §10.8 off UR; fixed 1 stale `SPENDABLE AFTER BLOCK` in §8.4. Gate: 15 sections/33 cross-refs OK, 26/26 cites, 9-term sweep clean. | **Yes.** Re-ran both gates myself, identical output. §10.8's rewritten text (lines 1066-1070) contains no live UR reference. `SPENDABLE AFTER BLOCK` — 0 occurrences. §6c/§6d — only the reworded prose form at line 1354. **But** the sweep's 9 terms did not include bare "UR" or "136", which is exactly where the residual defects below live (F-2) — the commit doesn't claim to have covered those terms, so this isn't a false claim, just an incompletely-scoped one. |
| `feb56a4` | §10.4 closed: TO allows a flagged free-text label, warns when blank. "20 items, 14 closed." | **Content yes** — three-state TO table (line 1000-1004) and the "flag is the point" reasoning (1006-1011) match verbatim. **Arithmetic off by one (Minor):** 20 items total (counted directly), but "14 closed" + "of the 7 remaining" double-counts item 7, which the same sentence calls "already-closed." 14 + 7 = 21 ≠ 20; the real split is 14 closed / 6 open. Does not affect spec content, only the commit message's own tally. |
| `14ddab0` | Warn below 10 sat/vB; repairs `5bbfd1d`'s no-op on the "NO minimum fee" wording. | **Yes.** §8.2b (lines 690-703) carries the warning verbatim, including the exact stderr text quoted in the commit. |
| `7b47941` | Legacy-input warning states arithmetic, not advice. | **Yes.** §8.2c (lines 717-751) matches the quoted warning block exactly, including the BTC figures. |
| `5bbfd1d` | Legacy inputs accepted; no minimum fee; CPFP is the only fee-bump that works; RBF omitted because it would misfire on the RCW's own fixture. | **Yes for what's in the spec.** §8.6's "Legacy inputs are ACCEPTED" block and §7's "Pinned fee" row match. §9 lists RBF/CPFP as out-of-scope (mt implements neither), consistent with the commit's "we don't care about rbf or cpfp" framing. The RBF-misfire *reasoning* (nSequence 0xFFFFFFFE on the RCW) is in the commit message only, correctly not asserted in the spec body. **But** this is also where the "non_witness_utxo binds the amount" framing that creates F-1 below gets its "Legacy inputs are ACCEPTED" wrapper — the claim that landed is accurate to what the commit intended, the *contradiction* is with older/newer text elsewhere (see F-1). |
| `9ccf5c3` | §8.2 removed; consensus-engine dependency dropped; §7 records the accepted hazard; weakens §8.6 to structural-only. | **Yes.** §8 item 2 (lines 651-676) and §7's "Well-formed but INVALID" row (629) match verbatim, including the "structural heuristic, not a proof" language reused at §8.6 (877-884). |
| `2924903` | Input is always a finalized PSBT; §6 gained a cross-reference (self-corrected after a first silent no-op). | **Yes.** §6's blockquote (566-569) and §10.10's table (1112) both state it. |
| `1893607` | §10.13 ruled: NUMS constant, HRP, content id = txid, 20 bits, `mt-codec` forks (not shares) into a new repo. | **Yes.** §10 item 13 (1174-1231) matches point-for-point, including the "I was wrong about where this lands" correction. |
| `83d2a72` | Legend reports two numbers not a verdict (`LOCKED TO BLOCK`/`NO BLOCK TIMELOCK`); legend re-measured at 130 chars/6 lines, down from 136; **"Every '136' in the spec updated to 130."** | **Partially FALSE.** The `stderr` report block (799-803) and §8 item 4 verbiage are correct and present. The 130-char table sum checks out (41+20+23+34+12=130). **But the "every 136" claim is false**: `git show 83d2a72 -- design/SPEC_mt_v0_1.md` shows the diff touched exactly one "136"→"130" (the field-table header sentence); the phrase "not part of the **136**-character budget above" at what is now line 461 is untouched in that commit's diff and remains "136" in the current file. This is the same failure class as `2924903`/`14ddab0` (a claimed sweep that missed an instance) — but unlike those two, **it was never caught or corrected**. See F-2. |
| `753dd58` | `mt` reads locktime FIELDS only; legend drops `IMMEDIATELY SPENDABLE` for a fact-only line; new §10.18 flags §8.2's survival as undecided. | **Yes.** §8 item 4 (771-850) matches, including the exact `nSequence` reasoning and the OP_CSV over-warn disclosure. (§10.18 was later closed by `9ccf5c3` — expected evolution, not a defect.) |
| `b594ab3` | `--timelocked`/`--immediate` flags removed; `mt` derives and warns instead. | **Superseded as expected, not false.** This commit's own legend text (`SPENDABLE AFTER BLOCK`/`IMMEDIATELY SPENDABLE`) was itself replaced two commits later by `753dd58`/`83d2a72`. Current spec correctly reflects the *final* state, not this intermediate one — that's the normal shape of an iterating fold, not a defect. |
| `829f328` | base45 chosen for the QR; new §3a "medium-appropriate ECC" principle; codex32-in-QR measured and rejected. | **Yes.** §3's table (185-189) and §3a (212-244) match the commit's numbers (63-65% / 85.5-86% / 88.4-88.8%) exactly. |
| `1ef961a` | Citation paths qualified with `crates/` prefixes; no spec CONTENT changed. | **Yes** — citation-only, confirmed no prose diff beyond paths (per commit's own diff scope, and current cite-check is clean). |
| `b515de9` | UR dropped entirely; 8 more §10 rulings folded (12 answered NO-fill, 3 closed by the UR drop, 4/5/7/14/15 dispositioned). | **Yes.** §3's three-position retraction box (132-163) and §10 items 2, 3, 6, 7, 12, 15 all match. |
| `fc4179c` | 4 rulings: `sysw` payload (unencrypted), stdin+file input, **module size becomes the operator's choice** (§8 item 8 rewritten from a hard refusal to "operator's choice, defaulting to 0.60mm"), static-scan reader out of scope. | **Yes for what it touched.** §8 item 8 (903-908) and §10 item 1 (932-952) match the commit's diff exactly (confirmed via `git show fc4179c -- design/SPEC_mt_v0_1.md`, which only edits those two spots). **The commit does not claim to have touched §4**, and didn't — but §4 states the *opposite* rule unchanged since the original draft. See F-3. |
| `ce678aa` | Every symbol gets its own `n/m` label (not just `PLATE n OF m`); labels cost is unpriced and per-symbol. | **Accurate at the time.** This commit is what introduced the now-stale "136-character budget" and "UR part" phrasing into §5 (both correct when written — UR was still current, budget was still 136). Neither phrase was updated by the two later commits that individually falsified each of them (`b515de9` dropping UR; `83d2a72` changing the budget). See F-2. |
| `4932f30` | §11's binary/results-file counts and "floor" caveat corrected to match the current probe crate and the 40-byte chunk fix. | **Yes.** §11 (1309-1358) states both probe runs with their real counts (12/13 binaries, 12 results files) and the reworded caveat about framing overhead, matching the commit's stated reasoning. |
| `612a46c` | Retraction: raw tx CAN represent an unsigned tx; new refusal 2b (value-blind acceptance); legacy-refusal premise was false; two "falsified nearby text" fixes (§5 TO truncation, mt1/md1/mk1 indistinguishability). | **Yes for what survives.** §3's retraction (line ~1051 equivalent — now embedded in §8 item 1's framing) and §8 item 2b (677-716) match. The TO-truncation and indistinguishability items are both still present, correctly evolved by later commits (`feb56a4` for TO, unchanged for §7's indistinguishability row at line 624). |
| `a709976` | Chunk is 40B not 45(→"363-bit model"); ceiling corrected 2,904B→2,560B; all 13 probe binaries rebuilt; two more "falsified nearby text" fixes (§8.2 prevout-always-arrives claim split by verb). | **Yes.** §3b's table (270-286) shows the corrected counts (162B→5 chunks, etc., matching the commit's before/after numbers exactly) and the correction box (288-312) states the 320-bit constant and 2,560B ceiling. |
| `042cb65` | Foundational two-verb fold: `ur:psbt` (at the time), `mt string` added, 4 operator rulings + T-1/T-2/T-3/T-4/T-6 folded, §6a/6c/6d/8.3a/1a deleted. | **Structurally yes**, as the base the later commits iterate on. Its `ur:psbt` content was itself superseded three commits later by `b515de9` — expected, documented in §3's own retraction box. |
| `097f388`, `5b99055`, `7116df6`, `239020d`, `bc8b0c1`, `52b91a3`, `d68d454` | Report-persist commits (R0/R1 lenses). No spec-content claims — each explicitly states "nothing folded in this commit." | **N/A** — verified each is report-only by checking the commit touches only `design/agent-reports/*.md`. |
| `4d74f7e`, `18d6a73`, `1048a29`, `099a516`, `6d2fb3d`, `ee43143` | Triage / measurement / citation-gate / continuity commits. | **N/A to spec.md content claims** — `ee43143` touches the continuity doc (not in scope here); the rest touch `design/measurements/` or run gates without spec prose changes beyond what's covered above. |
| `fefe901`, `30b2d64`, `1e74d4b` | String-form scope rulings (plate layout is the user's; stderr-only warning; 10.11 answered from fork font metrics). | **Yes.** §3b's "Layout on steel is the user's, not `mt`'s" (314-333) and "one thing `mt string` does say" (335-358) subsections match both commits verbatim, including the exact quoted rulings. |

## Contradictions

### F-1 — Legacy input value: "bound" (§8.6, §10.16) vs. "checked against nothing" (§6, §7, §8.2c)

**Severity: Important.**

Three sites assert the legacy input's claimed value **is** structurally bound /
verifiable, using this reasoning to justify *removing* the old refusal:

- §8.6 (line 891-893): *"The previous draft refused them, and its stated reason
  was false: it claimed a legacy amount is unverifiable because the sighash does
  not commit to it. The first clause is true; the conclusion does not follow,
  since BIP-174 requires `non_witness_utxo` for a legacy input — the **whole
  previous transaction** — so hashing it and matching the txid **binds the
  amount** without any help from the sighash."*
- §10.16 (line 1257-1258): *"the original refusal's premise was false
  (`non_witness_utxo` **binds** a legacy amount by txid)."*

Three other sites, describing the exact same fact, say the opposite — that the
value is checked against **nothing** and cannot be verified at all:

- §6 (line 573-575): *"For legacy inputs **nothing commits to them at all** —
  R0 lens 2's finding, and it survives the scope cut... **§8.6 is the rule.**"*
- §7's "Wrong input value" row (line 628): *"**not detectable by `mt`.** §8.2's
  removal means no signature is verified, and a legacy sighash never committed
  to the amount anyway."*
- §8.2c (line 739) and its blockquote (line 753-768): *"`mt` **CANNOT VERIFY
  THAT VALUE**... for a legacy input the claimed value is checked against
  **nothing**."*

Both cannot be simultaneously operative: if hashing `non_witness_utxo` and
matching the txid really does bind the amount (§8.6/§10.16's claim, used to
justify accepting legacy inputs at all), then `mt` performing that hash-check
would make the value verifiable, and the elaborate §8.2c warning would not need
to treat it as an unrecoverable hazard. But §8.2c is explicit that `mt`
implements **no such check** ("`mt` verifies neither" the PSBT's UTXO record nor
operator-supplied values) — so in the *actual* design, the value genuinely is
checked against nothing, and §8.6/§10.16's "binds the amount" framing overstates
what `mt` does.

**Root cause, traced via `git log -S`:** §6's "§8.6 is the rule" pointer dates
to the very first two-verb fold (`042cb65`), when §8 item 6 was only the sighash
rule — before §8.2c existed at all. The "non_witness_utxo binds the amount"
rebuttal was added later (`612a46c`, then extended by `5bbfd1d`) as a rebuttal to
the *old* refusal's premise, without being reconciled against the operative
warning mechanism that `5bbfd1d`/`7b47941` built at §8.2c one and two commits
later. The pointer in §6 was never updated to name §8.2c, which is where the
actual rule now lives.

**Recommendation (not binding, for the next fold):** either state plainly in
§8.6/§10.16 that the `non_witness_utxo` hash-check is a theoretical capability
`mt` v0.1 chooses not to implement (so the rebuttal doesn't overclaim), or repoint
§6's "§8.6 is the rule" to §8.2c, which is where the operative behavior lives.

### F-2 — §5's per-symbol label references a dropped envelope and a superseded number

**Severity: Important.**

Lines 461-462 (the paragraph immediately following §5's legend field table):

> "Plus, **not part of the 136-character budget above**, one `n/m` label
> engraved beside **each QR symbol**, naming the UR part it carries (§10.8's
> ruling)."

Two separate facts here are stale:

1. **"136-character budget"** — the legend budget was re-measured and changed to
   **130 characters** by `83d2a72` (confirmed: the field table two paragraphs
   above, line 452, correctly reads "130 characters"). `83d2a72`'s own commit
   message claims *"Every '136' in the spec updated to 130"* — false; this is
   the one instance the sweep missed (see commit-claim audit above).
2. **"naming the UR part it carries"** — UR was dropped entirely (`b515de9`),
   and §10.8 itself, the very ruling this sentence cites, was rewritten by
   `b1790a4` to read "for the **chunk** it holds" (line 1067) with no UR
   reference. The cross-reference target was fixed; the sentence pointing at it
   was not.

Both are the same failure mode the brief describes: the fix landed in the
section that was directly being edited (§5's field table for (1); §10.8 itself
for (2)) and never propagated to this adjacent sentence describing the same
fact.

**Grep control**, confirming this is the only surviving instance of each: `grep
-n "136" design/SPEC_mt_v0_1.md` returns exactly this one line; `grep -n -w "UR"
design/SPEC_mt_v0_1.md` returns 18 hits, of which every other one is inside a
correction/retraction box or the §1/§2/§3 "UR is dropped" discussion — line 462
is the only live (non-historical) sentence still treating UR as an active
mechanism.

### F-3 — §4 still states module size as a hard floor mt "must not select" below; §8 item 8 and §10.1 say it's the operator's choice

**Severity: Important.**

§4, "Module size" (lines 439-443), unchanged since the very first pre-R0 draft
(`5a4389f`, confirmed via `git log -S`):

> "Whether a camera reads 0.30 mm modules off brushed steel is a hardware
> question, gated on the test plate in F-234. **Until that plate exists, `mt`
> must not select a module below 0.60 mm** (two strokes)."

This is a hard, unconditional prohibition. But §8 item 8 (lines 903-908), folded
by `fc4179c` in response to the operator's explicit ruling *"User picks from all
available options, suggesting 0.6"*, reads:

> "**Module size is the operator's choice, defaulting to 0.60 mm** — not a
> refusal. Ruling 2026-08-23 (§10.1): `mt` offers **every size it can engrave**
> and suggests 0.60 mm... Sizes below that are optically unvalidated, and `mt`
> says so at the point of choice **rather than refusing**."

And §10.1 (line 936) states explicitly that this *replaced* the floor described
in §4: *"So §8.8's hard refusal below 0.60 mm becomes a default and a
recommendation, not a floor: `mt` offers every module size it can engrave... and
the operator decides."*

**Verified via diff, not inference:** `git show fc4179c -- design/SPEC_mt_v0_1.md`
shows this commit rewrote §8 item 8 and §10 item 1 only; it never touches §4.
No commit in the 30-commit range touches §4's "Module size" paragraph at all
(`git log -S"must not select a module below" -- design/SPEC_mt_v0_1.md` returns
only the original `5a4389f`). So this ruling — one of the operator's ~20 — landed
in two of the three places that state it and was never carried into the third,
which still describes the exact behavior (`mt` categorically refusing sub-0.60mm)
that §8/§10.1 say was replaced.

An implementer reading §4 in isolation would build a hard refusal; reading §8/§10
in isolation would build an operator-selectable option with a warning. These are
different behaviors, not a wording nuance.
