# mt spec — R3 fold-verification pass

Scope: mechanical fold-check of the four commits (`b1790a4..9907348`) that
respond to R2's three lens reports. Not a fresh audit. Artifact:
`design/SPEC_mt_v0_1.md` at `9907348` (1585 lines, read in full).

Machine-gated facts taken as given, not re-derived: `plan-cite-check.sh` →
27/27 resolved, 0 dangling (re-ran, confirmed); `spec-structure-check.sh` →
15 sections, 37 cross-refs, STRUCTURE OK, including the new table-cell-count
check (re-ran, confirmed).

## Verdict

**27 Critical+Important findings from the three R2 reports assessed:**
**13 FIXED, 3 PARTIALLY FIXED, 11 NOT FIXED, 0 OBSOLETE, 0 DEFERRED.**

- **Commit-claim audit: 1 new FALSE CLAIM found.** `93df0f2` asserts *"the
  single surviving 'engraved out-of-band reminder' is inside the correction
  box quoting what section 7 used to say."* **Two** survive, both at that
  commit and unchanged today — §10.16 carries a second, live instance the
  commit's own verification grep missed because the phrase wraps across a
  line break. This is a **fourth** instance of the silent-incomplete-sweep
  class the brief asked me to hunt for (joining `2924903`, `14ddab0`,
  `83d2a72`).
- **2 new contradictions found**, both from incomplete propagation of the two
  newest reversals: base45→bech32 (3 live leftover mentions, one inside a
  "CLOSED" ruling) and the §8.2d legacy-value-binding addition (§6 and §7
  still make the old blanket "checked against nothing" claim §8.2d
  contradicts for part of its scope).
- The two report-only commits (`3718bc4`, `a5a8d8e`) are confirmed to touch
  only `design/agent-reports/*.md`.
- **11 NOT FIXED findings are mostly ones this fold cycle never claimed to
  touch** — the four fold commits' own messages scope themselves to R2's
  Criticals plus a handful of named Importants (S-1..S-4, S-6, S-7, S-8,
  F-1..F-3, plus the table-gate/8.6-shape/MAX_SECTION_LEN/content-id work).
  Most of the NOT FIXED items below are Importants the commits never claimed
  to have folded, so they are not false claims — they are open debt, listed
  here because the brief asks for disposition of every Critical/Important in
  the three reports regardless of whether a commit claimed to address it.

## R2 finding disposition

Legend: **F**=FIXED, **PF**=PARTIALLY FIXED, **NF**=NOT FIXED, **OBS**=OBSOLETE.

| ID | Sev | Finding (short) | Verdict | Evidence / open text |
| --- | --- | --- | --- | --- |
| S-1 | Crit | §8.6 witness-only; legacy `scriptSig` never examined | **F** | §8.6b box, lines 1009-1019: *"BOTH SPENDING STRUCTURES, not just the witness... `mt` inspects `scriptSig` and witness alike."* `scriptSig` now appears in §8.6 (7 total occurrences spec-wide, `grep -c scriptSig` = 7, up from 3). |
| S-2 | Crit | §6a/§8.5 discard the true value; §7/§8.2c say undetectable | **PF** | §8.2d (added) binds the value via hash-match **only when `non_witness_utxo` is present**. But §8.5 (line 983) still refuses *only* on `gettxout` returning `null` — the `value` field it returns is still never cross-checked against the claimed/PSBT value for an input **without** `non_witness_utxo`. The original S-2 scenario (node available, true value differs from claimed, no cross-check) survives for that residual case. |
| S-3 | Crit | §7 names an "engraved reminder" §5 has no field for | **F** (at cited sites) | §7's row now reads *"**Nothing reaches the steel for `mt qr`** — §5's legend is full (§8.2c)"* (line 656); §8.2c's box explicitly retracts the old framing (lines 776-785). **But see new contradiction F-2 below** — the same retracted phrase survives, unfixed, at §10.16. |
| S-4 | Crit | `nLockTime` timestamp/height conflation; false-reassurance re-opened | **F** | §8.4 now branches on `LOCK_TIME_THRESHOLD = 500_000_000` (1 occurrence), compares a timestamp against median-time-past (1 occurrence), legend reads `LOCKED UNTIL <t>` (3 occurrences) vs `LOCKED TO BLOCK <n> ~<year>`. Confirmed via grep, matches commit `93df0f2`'s claims. |
| S-5 | Imp | §7 "Pinned destination" applies the `TO` line to `mt string` too, no verb marking | **NF** | §7's row (line 651) is still unmarked by verb: *"§5's `TO` line names the destination **wallet**... `mt` displays every output in full at encode time; the plate carries a summary."* `mt string` has no §5 legend (§3b), and §7's preamble (*"Every mitigation below names a field §5 actually engraves"*) still does not require a per-row verb tag. |
| S-6 | Imp | §7 "Pinned fee" row: 3 cells in a 2-col table, GFM drops the 3rd | **F** | Row now reads as one 2-column entry (line 653) with the `mt string` fee-blindness fact folded into the mitigation cell. Structure gate now checks table-cell counts and reports 0 malformed rows (machine-gated, re-confirmed). |
| S-7 | Imp | §8.6 recognizer: 65-byte Schnorr sig vs 65-byte control block collide | **F** | Box at lines 1021-1046: shape-based recognizer added — *"last element is the control block, second-last the leaf script"* — explicitly still called a heuristic, not a proof. |
| S-8 | Imp | §10.13 content id: which txid (unsigned_tx vs extracted), which bits | **F** | §10.13(c) (lines 1390-1406): *"the id derives from the EXTRACTED transaction's txid... The top 20 bits of the txid in its standard display form."* No remaining `unsigned_tx`-as-source-of-id text found (`grep -n unsigned_tx` → only the resolution's own explanatory prose, lines 1392/1400). |
| S-9 | Imp | Operator-supplied input value never engraved; `extract_tx()` refuses the result | **NF** | §8.2c's box (lines 739-743) still only states that `extract_tx()` refuses on 3 counts and that §8 "adopted the first and ignored the other two" — no text anywhere says whether an operator-supplied value gets written into the payload. `grep -n "extract_tx"` → 2 hits, neither resolves this. |
| S-10 | Imp | Absurd-fee refusal (§8.2b) only catches the harmless (over-claim) direction | **NF** | §8.2b's text (lines 713-717) still states the `AbsurdFeeRate` refusal as if it were relevant to §8.2c's hazard (*"it is what a wrong input value produces (§8.2c)"*) with no disclosure that it structurally cannot catch the loss-causing (under-claim) direction. |
| S-11 | Imp | §6a discloses only the `null` side of `gettxout`'s ambiguity, not a stale-node non-null false negative | **NF** | §6a's box (lines 628-633) still discloses only mempool-exclusion and null-ambiguity: *"a `null` cannot distinguish 'already spent' from 'this node is still syncing, or is on the wrong chain'."* No mention of a stale/behind-tip node returning a stale-but-non-null value. |
| S-12 | Imp | `TO <wallet id> <amount>` — `<amount>` undefined for multi-output tx | **NF** | §5's field row (line 486) and §10.4 (lines 1184-1217) both discuss `TO` at length; neither defines what `<amount>` sums over when a transaction has more than one output (change, multiple counterparties). |
| C-1 | Crit | `mt qr` payload undefined; base45 collides with EPD §6.4 (spaces) | **PF** | The EPD §6.4 collision itself is **fixed** — the switch to bech32 uppercase is explicitly justified against §6.4 (§3's correction box, lines 191-221). The broader "no `sysw` record class, no channel for §4's chosen params, no legend delivery" gap is still open, and is honestly disclosed as unresolved work in §10.9 (*"There is no transaction class... Adding one is the work"*, lines 1288-1289) and §10.17. |
| C-2 | Crit | `mt qr`'s per-chunk payload byte-count is never stated | **NF** | §3b's "40 payload bytes / 64 chunks" rule is stated only for `mt string`. No text distinguishes an `mt qr` chunk's capacity from that rule, or ties it to symbol capacity. `grep -n "40 payload bytes\|per chunk"` finds no `mt qr`-specific chunk-size statement. |
| C-3 | Crit | Byte-domain framing of the 37-bit header is unspecified; no `mt1` version value | **NF** | `grep -n "padding\|MSB\|framing"` over the current spec returns no hits inside the normative text. `version` is named as a `ChunkHeader` field (lines 165, 1245) but no numeric value for `mt1` is ever assigned. |
| C-4 | Crit | Content id: which txid, which bits, which end | **F** | Same fix as S-8 above. |
| C-5 | Crit | §4 vs §8.8/§10.1: module size search vs operator's choice | **F** | Already fixed in `bdd7438`, before lens 3's report arrived (the report says so itself). Confirmed current: §4's module-size paragraph (lines 458-465) now reads *"0.60 mm... is what `mt` SUGGESTS — not a floor it enforces"*, with a correction box naming the earlier miss. `grep -n "must not select a module"` → 0 hits. |
| C-6 | Crit | §8.7's "plate budget" undefined; `MAX_SECTION_LEN` unmentioned | **F** | §8's refusal 7 now defines it: *"'Plate budget' means the operator's stated maximum plate count"* (line 1064). New refusal 7c cites `MAX_SECTION_LEN = 8191` (lines 1068-1074). |
| I-1 | Imp | Per-symbol `n/m` label geometry vs the quiet zone is unstated | **NF** | §10.8 (lines 1238-1276) is unchanged on this point — states the label exists and is "unpriced," never says where it sits relative to §4's 4-module quiet zone. |
| I-2 | Imp | §5 calls `FROM WALLET` mandatory while §10.4 closes it optional | **NF** | Line 571 still reads: *"`FROM WALLET` is a **mandatory field** sized into §4's reservation, and nothing says what supplies it or what happens when it is absent."* This directly contradicts the field table's own *"Optional — loudly warned when absent (§10.4)"* two paragraphs above (line 484) and §10.4 itself, which is CLOSED as optional (lines 1184-1188). Neither commit in this cycle touched this sentence. |
| I-3 | Imp | `TO <amount>` semantics undefined | **NF** | Duplicate of S-12. |
| I-4 | Imp | §7 credits an "engraved out-of-band reminder" §5 does not engrave | **F** (at cited sites) | Same as S-3. See new contradiction F-2. |
| I-5 | Imp | v0.1 ships no decoder; recoverer walk cannot complete | **F** (as a disclosure) | §9 (lines 1115-1120) now states plainly: *"a plate cut by `mt` v0.1 **cannot be read back by `mt` v0.1**"*, and files the format-naming gap as new open question §10.21 (lines 1529-1535). This is the kind of gap a spec edit can only disclose, not close — and it is now disclosed, which is what I-5 asked for. |
| I-6 | Imp | `mt string`'s stdout delimiter/grouping of a multi-chunk set is unspecified | **NF** | §3b (lines 338-342) still only says *"`mt string` emits a string. That is the whole of its output"* — no delimiter, separator, or grouping rule for the 5-to-89-chunk case. |
| F-1 | Imp | Legacy value "bound" (§8.6/§10.16) vs "checked against nothing" (§6/§7/§8.2c) | **PF** | §8.2d now makes the "bound" framing literally true for inputs carrying `non_witness_utxo`, and §8.2c's warning text was narrowed correctly (*"NOTHING HAS VERIFIED THAT VALUE. This input carries no `non_witness_utxo`..."*, lines 767-768). **But §6's box (line 601) and §7's "Wrong input value" row (line 656) were not updated** and still make the old blanket claim: §6 — *"For legacy inputs nothing commits to them at all... §8.6 is the rule"*; §7 — *"**not detectable by `mt`.** §8.2's removal means no signature is verified, and a legacy sighash never committed to the amount anyway."* Neither acknowledges §8.2d exists. |
| F-2 | Imp | §5's per-symbol label cites a 136-budget (stale, was 130) and "the UR part" | **F** | Now consistently reads *"136-character budget"* (re-measured, correct — see below) and *"naming the `mt1` chunk it carries"* (line 490), no UR reference. |
| F-3 | Imp | §4 states a hard 0.60mm floor; §8/§10.1 make it the operator's choice | **F** | Same fix as C-5. |

**On the apparent "130 vs 136" flip:** R2 lens 1's F-2 was filed against a state where §5 said 130 (current) but a sibling sentence still said 136 (stale). `1ffbccb` then **re-measured the whole legend** (the unlock-year estimate added characters) and the correct current value became 136 everywhere — a different, later, and independently-verified re-measurement, not a reversion of the fix. Confirmed: current file has exactly 4 occurrences of "136" (all correct — field-table sum, per-symbol-label sentence, the "goes from 130 to 136" transition sentence, and §10.21's new mention) and exactly 1 occurrence of "130" (the transition sentence, correctly historical). Field-table sum re-added: 41+20+29+34+12 = 136 ✓ (the `LOCKED TO BLOCK <n> ~<year>` field grew from 23 to 29 characters with the year estimate).

### Minor/Nit findings — spot-checked, not exhaustive

| ID | Sev | Verdict | Note |
| --- | --- | --- | --- |
| S-13 / M-1 | Minor | **F** | §8.7b now says 89 chunks (line 1077), matching §3b's table. `grep -n "78 chunks"` → 0 hits. |
| S-14 | Minor | **NF** (partially) | §8.4's own box now correctly frames `IMMEDIATELY SPENDABLE` as *"an earlier draft"* (line 977) — fixed there. **But §1 item 7 (lines 85-90) is untouched** and still reads *"It reads the transaction and warns if the plate would be immediately spendable"* as a current decision, which §8.4 explicitly superseded with fact-only reporting. |
| S-15 | Minor | **NF** | §8.4's box (lines 964-969) still frames the relative-timelock gap as *"lives in the witness script as `OP_CSV`"* / *"evaluating the sending wallet's script"* — the wrong-protocol-fact framing R2 flagged (BIP-68 relative locks live in `nSequence`/`nVersion`, transaction fields; `OP_CSV`/BIP-112 is the separate script-level opcode) is unchanged. |
| S-16 | Minor | **NF** | `"MIN form"` (line 752) and `` "`lean` PSBT form" `` (line 739) both still appear, undefined by the current §3 (which dropped these terms when UR was removed). |
| S-17 / M-3 | Nit | **F** | Superseded by the 130→136 re-measurement above; no longer applicable in its original form. |
| M-2 | Minor | **F** | Duplicate of F-2. |
| M-4 | Minor | **F** | Duplicate of S-6. |
| M-5 | Minor | **NF** | Same `(version, chunk_set_id)` for one transaction's two different-count chunk sets (`mt qr` vs `mt string`) is unaddressed. |
| M-6 | Minor | **NF** | QR encoding mode (alphanumeric vs byte) still absent from §4's search space. |
| M-7 | Minor | **NF** | Symbol-ordering-within-tiling still unspecified. |

## Commit-claim audit

| commit | claim | verified? |
| --- | --- | --- |
| `b1790a4` | (pre-cycle baseline; audited by R2 lens 1, not re-audited here) | — |
| `971d3fa` | Report-only, "nothing folded" | **Yes** — `git diff-tree --name-only 971d3fa` touches only `design/agent-reports/mt-spec-R2-fold-check.md`. |
| `bdd7438` | F-1/F-2/F-3 fixed; §8.2d added; post-write greps `"136-character" 0, "UR part" 0, "must not select a module" 0, refusal 8.2d present 1` | **Yes**, all four verified against the file *as it stood at that commit* (`git show bdd7438:design/SPEC_mt_v0_1.md`) and each condition still holds in current HEAD except "136-character," which correctly changed to 136 in a later, independent, non-contradicting fold (`1ffbccb`). |
| `3718bc4` | Report-only, "nothing folded" | **Yes** — touches only `design/agent-reports/mt-spec-R2-funds-safety.md` (1 file, 796 insertions). |
| `a5a8d8e` | Report-only, "nothing folded" | **Yes** — touches only `design/agent-reports/mt-spec-R2-implementability.md`. |
| `93df0f2` | S-1..S-4 fixed; 78→89 chunks; post-write greps include `"engrave a reminder" 0`, and **"the single surviving 'engraved out-of-band reminder' is inside the correction box quoting what section 7 used to say"** | **S-1/S-2(partial)/S-3(partial)/S-4 verified as claimed for their cited sites. The "single surviving" grep claim is FALSE.** `tr '\n' ' ' \| grep -o 'engraved *out-of-band reminder'` at commit `93df0f2` returns **2** matches, not 1: the correction box (§8.2c, expected) and a second, live, unretracted instance at §10.16 (*"The residual risk is handled by §8.2c's engraved out-of-band reminder and recorded in §7"*), present unchanged in the commit's parent and never touched by `93df0f2`'s diff. The commit's own grep was presumably line-based and missed it because the phrase wraps across a markdown line break at exactly that spot (`"engraved\n    out-of-band reminder"`). **This is the fourth instance this cycle of a commit asserting a sweep result its own search methodology could not have supported** — joining `2924903`, `14ddab0` (both pre-R2), and `83d2a72`'s "every 136... updated to 130" (caught by R2 lens 1). See new contradiction F-2 below. |
| `1ffbccb` | bech32 replaces base45; unlock-year estimate added; legend 130→136; post-write grep `three "136" (all correct sites), one surviving "130"` | **Yes**, verified against the file *as it stood at that commit* — exactly 3× "136" and 1× "130", matching the claim precisely. **But the commit's title and content claim a wholesale replacement of base45 by bech32 for the QR payload, and that replacement is incomplete** — see new contradiction F-1 below. The commit's own diff (`git show 1ffbccb -- design/SPEC_mt_v0_1.md`) rewrites §3's table and correction box, but never touches §3a's pipeline diagram, §10.3, or §10.8, all three of which still describe the payload as base45. |
| `9907348` | Table-gate added; §8.6 recognizes scriptSig+witness by shape; §8.7c (`MAX_SECTION_LEN`) added; commitment checks enumerated; §10.13(c) content-id ambiguity resolved; decoder-gap + §10.21 disclosed | **Yes for all named items** — each confirmed present in current text (S-7, C-6, S-8/C-4, I-5 rows above). Structure gate (machine-gated, re-confirmed: 15 sections, 37 cross-refs, STRUCTURE OK, 0 malformed table rows) and cite gate (27/27) both pass as claimed. |

## New contradictions

### F-1 — base45 survives in three live (non-retracted) locations after the bech32 reversal

**Severity: Important. Sections: §3a, §10.3, §10.8.**

`1ffbccb` reverses the QR payload encoding from base45 to bech32 uppercase,
and rewrites §3's table and correction box thoroughly and correctly. Three
other places describing the *same* current pipeline were never touched, and
all three are live prose, not inside a retraction/correction blockquote:

- **§3a's pipeline diagram** (line 262, a fenced code block, not a
  blockquote):

  ```
  mt qr:      chunk header + payload -> base45 -> QR (Reed-Solomon) -> modules
  ```

  This is the section's own summary of "what legitimately crosses both
  media," directly contradicting §3's chosen encoding two sections earlier.

- **§10.3, an entire open question marked CLOSED** (lines 1175-1182):

  > "~~Is UR worth its expansion? What goes in the QR?~~ **CLOSED.** UR is
  > dropped (§3), and the QR payload is **`mt1` chunks, base45-encoded** —
  > operator ruling 2026-08-23. ... base45 was chosen over 3%-denser raw
  > binary for scanner compatibility... **§10.1's test plate should still
  > confirm scanners read base45 off engraved steel** — the choice is made,
  > the optical validation is not."

  This is the most serious of the three: a reader who trusts a **CLOSED**
  ruling (exactly the class of text §10's numbering exists to make
  authoritative) is told the wrong encoding was chosen, complete with a
  now-wrong justification (scanner compatibility, corrupted-triple
  detection — neither property bech32 uppercase was chosen for) and a
  now-wrong validation instruction (confirm base45 scans, not bech32).

- **§10.8** (line 1248): *"for `mt qr` it rides in the base45 payload."*

**Control, confirming these are the only three live instances:** every other
`base45` occurrence in the file (lines 160, 187, 192, 209, 221 — 5 of 8
non-listed hits) is inside §3's own correction/comparison apparatus,
correctly using base45 as a *rejected* candidate or a historical reference
point. `grep -n base45 design/SPEC_mt_v0_1.md` returns 12 total hits; 3 are
the ones above (§3a diagram is a code block so it doesn't read as a
blockquote to a casual grep, but a reader has no such filter), 4 are inside
§3's correction box discussing the reversal itself, 5 are comparison-table
entries showing base45 was considered and rejected.

### F-2 — §10.16 still cites the retracted "engraved out-of-band reminder," and the commit that fixed the phrase elsewhere miscounted its own survivors

**Severity: Important. Section: §10.16 (also documented under the
commit-claim audit above, since it doubles as a false verification claim).**

§8.2c's correction box (folded by `93df0f2`) explicitly retracts the claim
that `mt qr` can carry an engraved reminder: *"`mt` CANNOT put that reminder
on a `mt qr` plate... nothing reaches the steel."* §7's row was corrected to
match. §10.16 — the open question ruling out a legacy-input refusal — was
not:

> "16. ~~Should `mt` refuse legacy (non-segwit) inputs at all?~~ **CLOSED —
> NO**... The residual risk is handled by §8.2c's engraved out-of-band
> reminder and recorded in §7."
> (lines 1474-1480)

This both (a) restates the exact retracted mechanism as the closing
statement of a CLOSED ruling, and (b) is now also inaccurate about *what*
handles the residual risk — as of `bdd7438`, §8.2d handles part of it via
hash-binding, which §10.16's own text one sentence earlier correctly
describes (*"the original refusal's premise was false (`non_witness_utxo`
binds a legacy amount by txid)"*) without naming §8.2d or updating the
"handled by" clause to match.

**Root cause:** the phrase wraps across a markdown line break
(`"...§8.2c's engraved\n    out-of-band reminder..."`), which is exactly why
`93df0f2`'s own single-line verification grep reported one surviving
instance instead of two. Confirmed by re-running the same grep shape against
that commit's tree: line-based, 1 match; newline-collapsed, 2 matches.

## Coverage note

Per the brief's already-settled list, citations (27/27) and structure
(including the new table-cell-count check) are machine-gated and were
re-confirmed, not re-derived. Everything above was checked by reading the
cited spec text directly and, where a numeric or textual claim was
checkable, by grep against both current HEAD and the commit tree the claim
was made at. Not covered: whether any NOT FIXED item is *safe to leave*
open versus load-bearing enough to block a re-review — that is a severity
and gating call for whoever runs the next R-round, not a fold-verification
question.
