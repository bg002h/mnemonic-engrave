# R7 — fold verification of the R6 three-lens review (6C / 27I)

**Scope, per brief:** two questions only. (1) Did the fold in commits `4c74e0e`,
`3baa0a1`, `196e924`, `a4d0197` actually fix each of the 33 findings from
`R6-lens-fold-propagation.md` (0C/8I), `R6-lens-adversarial.md` (4C/8I) and
`R6-lens-implementability.md` (2C/11I)? (2) Did the fold introduce a new defect?
No fresh audit, no re-litigation of settled rulings. Base commit `aa0bd68`
(pre-fold) diffed against `a4d0197` (HEAD). Structure/citation gates re-run and
matched the commit messages' claims (`STRUCTURE OK, 16 sections, 43 cross-refs`;
`32/32 citations resolved, 0 dangling`) — not re-litigated here.

**Result: 30 FIXED / 3 PARTIAL / 0 NOT FIXED**, out of 6 Critical + 27 Important.
Two new Minor-severity issues found (Section B), neither Critical/Important.

---

## Section A — per-finding verdicts

### Criticals (6/6 FIXED)

| lens | id | verdict | evidence |
| --- | --- | --- | --- |
| adversarial | C-1 (TX labelled txid, specified as wtxid) | **FIXED** | Line 658: `TX` row now reads *"the txid — double-SHA-256 of the decoded transaction with marker, flag and witnesses stripped... Not a hash of the engraved bytes"*. Second site, lines 1965–1971 (§6a), states the wtxid/txid distinction and the 162B/94B example verbatim. |
| adversarial | C-2 (content-id blind to 92% of payload) | **FIXED**, by operator ruling, not redesign | Lines 421–447: *"THE CONTENT ID IDENTIFIES THE TRANSACTION. IT DOES NOT PROVE THE BYTES."* Operator ruling explicitly retracts the "only thing that can" / "funds-load-bearing invariant" language; design (txid-based content id) is deliberately unchanged per the operator's own words quoted in the box. This matches the "settled — do not reopen" ruling in the review brief. `verify`'s FAILED report (lines 449–461) now states the limit plainly. |
| adversarial | C-3 (DEAD reported for merely-unconfirmed parent) | **FIXED**, both sites | Lines 563–568: DEAD now requires `getrawtransaction <parent> true` returning `confirmations ≥ 1`; PENDING covers not-found-or-mempool-only. Second site, §8.5 refusal at lines 2673–2685: now requires `null` **and** confirmed parent, with an explicit note that a mempool-only parent is a warning not a refusal. |
| adversarial | C-4 (BEARER "cannot redirect" guarantee false) | **FIXED** | Lines 1679–1719: unqualified "cannot redirect the money" replaced with "**in the ordinary case**... `mt` GUARANTEES NOTHING HERE", the 64-byte-preimage counter-example is spelled out, and the `stderr` warning now states both what was checked and that it reads shape not script. (Minor residual: §7's threat-model row for "Non-`ALL` sighash", line 2052, does not itself add the grindability caveat verbatim — but §8.6, which it cites, carries the caveat in full at lines 2726–2751, so the hazard is not lost. Graded FIXED since the core objection — an unqualified guarantee used to pick permanent wording — is resolved.) |
| implementability | C-1 (§11 vs §3b: two chunk splits) | **FIXED** | Lines 3628–3648 (§11): explicit retraction box, *"THE REPLACEMENT REASON WAS ITSELF WRONG, AND IT INVERTED THE CITATION IT LEANED ON"*, states the normative rule is §3b's. |
| implementability | C-2 (chunk-count formula never stated for `mt1`) | **FIXED** | Lines 1406–1434 (§3b, "The chunking rule — NORMATIVE"): `count = ceil(payload_len/40)`, `bytes_per_chunk = ceil(payload_len/count)`, stated as pseudocode, with the two-constants distinction spelled out. Reproduces every row of §3b's own chunk table (verified: 162→5, 405→11, 535→14, 742→19, 2498→63, 3538→89, all `ceil(bytes/40)`). |

### Fold-propagation Importants (6 FIXED / 2 PARTIAL)

| id | verdict | evidence |
| --- | --- | --- |
| I-1 (flat-40 rule survives in 3 places) | **PARTIAL** | §11 main site (line ~3634) and §3b "What fits" (line 1438, now "at most 40 payload bytes") are fixed. **Third site named in the finding, §10.12 line 3321, is unchanged**: *"A 535 B transaction balanced at **40 B/chunk** is 14 chunks."* §3b's own normative rule (line 1428) gives `bytes_per_chunk = 39` for this exact 535-byte payload — the two numbers directly contradict each other on the same page. Confirmed byte-identical to the pre-fold text at old line 2752 (`git show aa0bd68` vs current); none of the four fold commits touch this line. |
| I-2 ("PSBT-only" survives in 2 places) | **PARTIAL** | §10.10 heading and prose fixed (line 3112: *"Why a PSBT is PREFERRED — not required"*), §6 site fixed (line 1871: *"`mt` PREFERS a finalized PSBT and accepts a raw signed transaction..."*). **The sub-table the finding also named (old line 2598, now lines 3125–3130) is untouched**: `§8.2b value balance? | ... | **cannot run** — no input amounts` still states this as an absolute for "raw signed transaction", with no node column — contradicting §8.2e's own table two sections earlier (line 2334: `raw, **node** | **✓ via `gettxout`**`). Confirmed byte-identical to pre-fold text. |
| I-3 ("ZERO FLAGS" false claim) | **FIXED** | Lines 3184–3212: *"THE SPEC NAMES TWO FLAGS while requiring SEVEN operator inputs"*, with the 7-row operator-input table added. |
| I-4 (§5 "five fields" vs §0a's six) | **FIXED** | Line 1663: *"prints these SIX fields"*, with a note explaining "five" was true only "for the rest of that day." |
| I-5 (§8.2c "five fields... no room for a sixth", "mt string") | **FIXED** | Lines 2205–2213 area: argument now states it holds "at five fields, at six, and at whatever the deferred cycle settles on" — both brittle numbers and the old verb name removed. |
| I-6 (4 sites still say "spendable") | **FIXED**, all 4 sites | §7 thesis (line 2037: "broadcastable by whoever holds it"), §7 table row (line 2049: "moves money when whoever picks it up broadcasts it"), decision 7 (line 1077: "immediately broadcastable"), §5 rationale cell (line 1677: "the plate carries a transaction anyone holding it can broadcast"). |
| I-7 (§9 states §10.21 gap as open after it closed) | **FIXED** | Lines 2880–2885: *"§10.21 closed the version of it stated here, while the hazard survives on other grounds"* — restated on §0a's no-legend-on-realistic-plate grounds, exactly as the original finding's note for the fold recommended. |
| I-8 (normative report block drops SET-PREFIX row) | **FIXED** | Lines 716–736: `PREFIX` row added below `CUT`, both shown ("two extra rows" now shows two), with a retraction note explaining the earlier drop. |

### Adversarial Importants (8/8 FIXED)

| id | verdict | evidence |
| --- | --- | --- |
| I-1 (`--transaction` claims "prove identity", compares 20 bits) | **FIXED** | Lines 503–517: full 32-byte txid compare, PSBT compared against its extracted transaction, "prove identity" wording only survives inside the retraction quoting the old text. |
| I-2 (`decode` failure behaviour unspecified; `xargs` blind to it) | **FIXED** | Lines 786–811: *"WRITES NOTHING TO STDOUT UNLESS EVERY CHECK... PASSES, and exits non-zero otherwise"*; one-liner changed to `> tx.hex && ...` (line 779–780). Only 2 remaining `xargs` hits, both inside the retraction narrating the old defect. |
| I-3 (LIVE says "broadcast it" for a mempool-conflicted input) | **FIXED** | Line 565 qualifies LIVE as "unspent in the UTXO set", with the mempool caveat inline; box at 577–599 explains the fix and rejects re-querying with `include_mempool=true`. |
| I-4 (report prints `— PASSED`, forbidden by §8.4) | **FIXED** | `— PASSED` deleted from the normative block (line 649 example now ends at `current height 1402887` with no verdict); box at 665–684 explains and binds the row to §8.4 by reference. Zero remaining `PASSED` hits outside the retraction narrating it. |
| I-5 (PSBT-sourced FEE in the "verified" column, unchecked) | **FIXED** | Lines 692–715: rule 2 now names three provenance classes (chain-fetched / txid-bound / operator-asserted); `FEE` row (line 660) carries weakest provenance inline, `(CLAIMED — no input value verified)`. |
| I-6 ("or the total across all inputs" — two readings, off by a whole input) | **FIXED** | Lines 2150–2166: alternative deleted, *"mt requires the operator to supply that input's value, per input"*, with the 1.0/1.99/2.0 BTC scenario spelled out as the reason. |
| I-7 (duplicate resolution: 2-candidate table, row 1 "has proof") | **FIXED** | Lines 233–291: table kept for the 2-candidate illustrative cases, but the rule is restated over `n` candidates with majority vote explicitly forbidden (line 253), and row 1 now "announces" rather than silently proceeding (lines 266–278). |
| I-8 (DEAD printed for a transaction that already confirmed) | **FIXED** | Lines 543–561: new first-class check, `getrawtransaction <our txid> true` → `SPENT — ALREADY CONFIRMED`, run **before** any per-input classification, with the box explaining why it must go first. |

### Implementability Importants (10 FIXED / 1 PARTIAL)

| id | verdict | evidence |
| --- | --- | --- |
| I-1 (stdout case never stated) | **FIXED** | Line 956: `"normalise to LOWERCASE"` as step 1, with box at 969–985 explaining the reasoning and that the correction table is read after normalisation. |
| I-2 ("strip whitespace before anything else" unbuildable) | **FIXED** | Lines 883–887: ordered split rule — split on newline-containing whitespace runs, strip intra-line whitespace, split a line at each `mt1`/`MT1` prefix. |
| I-3 (mandatory length check circular at decode time) | **FIXED** | Lines 928–937: modal-length rule stated (*"the most common string length in the set IS the expected one"*), final chunk explicitly deferred until the set is complete. |
| I-4 (duplicate comparison basis unstated) | **FIXED** | Line 239: *"'BYTES' MEANS THE POST-CORRECTION PAYLOAD, NEVER THE AS-TYPED CHARACTERS."* |
| I-5 (two position conventions, one broken example) | **PARTIAL** — see Section B for detail | Rule is fixed and unambiguous: line 987, *"POSITIONS IN OUTPUT ARE 1-BASED... A BCH codeword index `k` is position `k + 4`"*. The `pos 16`/`b` example (lines 1047–1052) is verified correct by direct character count (position 16 = `b`, caret column checked programmatically). **The other worked example the finding named (chunk 7's correction detail, lines 348–362) was only partially regenerated**: the first number was bumped from the old 0-based `pos 12` to the new 1-based `pos 13` (correct — `q` really is at position 13), but the other three cited positions (34, 35, 78) were left unchanged from the pre-fold text and do not match the displayed string — computed positions for the `v` and `8` corrections are 29 and 30, not 34 and 35 (a consistent off-by-5, verified programmatically, treating each `[x>y]` bracket as one symbol). This is the identical failure mode I-5 itself was filed to fix — an operator checking this position against their steel finds neither the claimed value nor learns anything. |
| I-6 (0-based rules, 1-based reports, "chunk 7" ambiguous) | **FIXED** | Line 204: *"positions and chunk numbers are both 1-based in output, wire fields are 0-based"*; line 211's completeness rule restated as "chunks 1 through `count` present." |
| I-7 (raw hex: §8.2e accepts, §10.10/§6 refuse) | **FIXED** | §10.10 (line 3112) and §6 (line 1871) both now state PSBT-preferred/raw-hex-accepted, matching §8.2e. |
| I-8 (past locktime: two engraved legends) | **FIXED** | Lines 2580–2599: `NO TIMELOCK` reserved for `nLockTime = 0`/final inputs; a past-but-enforced locktime keeps `LOCKED TO BLOCK <n>` and drops only the `~<year>` estimate, argued on substance (the height is a fact about the transaction's fields) not just precedence. |
| I-9 (§8.5 refuses on any null; §1.1 allows PENDING) | **FIXED** | Closed by the C-3 fold — §8.5 now refuses only on DEAD (null **and** confirmed parent); confirmed at lines 2673–2690. |
| I-10 (CLI names zero flags; grouping/`--quiet` scope unruled) | **FIXED** | Lines 3270–3299: grouping-affects-stdout and `--quiet`-scope both ruled; exit code `0 = every check passed` fixed, explicitly reconciling with the "exit codes still unspecified" sentence two lines earlier (that sentence is about the *non-zero* code space only). |
| I-11 (input sniffing: table of recognisers, not a procedure) | **FIXED** | Lines 2280–2308: ordered 4-step procedure, binary tested before whitespace strip (with the reason stated), hex-encoded-PSBT ambiguity called out with its required refusal wording. |

---

## Section B — new defects found

**1. (Minor) Stale "four-state" description after I-8's fix added a fifth top-level state.**
Adversarial I-8's fold (196e924) added a new pre-check at lines 543–546: `getrawtransaction`
on the transaction's own txid, which — if confirmed — reports `SPENT — ALREADY CONFIRMED`
and skips per-input classification entirely. This is a state the STATUS row can now show
that is outside the four rows of the per-input classification table. Two nearby sentences
were not swept to reflect it: line 537, *"PLATE LIVENESS is its own row, and it has FOUR
states, not two"*, and line 663 (the normative report block), *"`STATUS` | always... | the
four-state liveness table above"*. Both predate this fold (present verbatim in `aa0bd68`)
and are still literally true of the sub-table they name, but neither now mentions the
`SPENT — ALREADY CONFIRMED` pre-check that can also appear in that same `STATUS` field.
Low practical risk — the check is described in full, with example wording, directly above
— but it is exactly the propagation pattern the brief asked to hunt for, so it is recorded.
No fix required to close this review; flagging for the next pass.

**2. (Documentation accuracy) The `a4d0197` commit message overclaims for one of its two "regenerated" examples.**
The message says *"TWO POSITION CONVENTIONS, NEITHER STATED, AND ONE EXAMPLE BROKEN... Both
regenerated from COMPUTED offsets."* This is true of the `pos 16`/`b` example (lines
1047–1052, verified character-by-character above) and **not** true of the chunk-7
correction-detail example (lines 348–362): only the leading position number was
recomputed; the interior three were left at their pre-fold values, which were already
inconsistent with the displayed string before this cycle (confirmed via `git show aa0bd68`
— lines 34/35/78 are byte-identical to the pre-fold text, only "pos 12" became "pos 13").
This is the direct cause of I-5 (implementability)'s PARTIAL grade above, recorded here
separately because it is a claim about the fold's own completeness, not a claim about the
spec's content.

---

## Section C — checked and found correct (not to be re-derived)

- **All "spend"/"spendable" sites**: comprehensive grep across the document found no
  remaining unqualified claim that a holder can "spend" or is granted unqualified power;
  every live use is "broadcast" or explicitly qualified ("in the ordinary case").
- **`xargs` and `— PASSED`**: every remaining occurrence of both strings is inside a
  retraction box narrating the old, now-fixed defect — none is a live assertion.
- **Chunking formula (C-1/C-2 impl) self-consistency**: `count = ceil(len/40)` and
  `bytes_per_chunk = ceil(len/count)` reproduce every row of §3b's own chunk table (162→5,
  405→11, 535→14, 742→19, 2498→63, 3538→89) and every row of §1e's length table (33B→79/74,
  37B→85/82, 39B→89/71, 40B→90/61, 40B→90/90, 40B→90/55).
- **The `pos 16`/`b` autocorrect example** (lines 1047–1052): verified by direct character
  count and column-alignment check (caret column minus string-start column = 16, which is
  `b` in `mt1qzrf8xk2v9d7b4...`). Correct.
- **Position 13 in the chunk-7 example**: `q` is genuinely the 13th character of
  `MT1QZRF8XK2V[q>p]...` under 1-based counting. Correct (see Section B for the other three
  numbers in the same example).
- **Content-id design**: confirmed the operator ruling (adversarial C-2) deliberately kept
  the content id as the txid rather than switching to wtxid, consistent with the "settled —
  do not reopen" instruction in the review brief. The retraction is a claim change, not a
  design change, everywhere it is stated.
- **1-based numbering, `position = k + 4`**: only one site states the BCH-index-to-string-position
  mapping; no stray `k + 3` or other offset survives.
- **Structure/citation gates**: `spec-structure-check.sh` → `STRUCTURE OK, 16 sections, 43
  cross-refs`; `plan-cite-check.sh` → `32 / 32 resolved, 0 dangling, 0 ambiguous`. Matches
  what all four commit messages claim.

**Two pre-existing, unfixed Minor findings noticed in passing (out of the 33-finding scope,
not graded, not new — flagged only for completeness):**
- Fold-propagation M-2 / implementability-adjacent: §5's legend field table (line 1732)
  gives the `LOCKTIME` field as 35 characters; the actual measured string
  (`RESULTS_legend_budget_2026-08-22.txt`) and the current spec's own worked example
  (`LOCKED TO BLOCK 1383520 ~FALL 2034`) are both 34 characters. Byte-identical to the
  pre-fold text; none of the four commits touch it.
- Implementability M-2: the worked report's `mt1 SET 0x0e17e` does not match its own `TX`
  row's txid (`9a3f21c0...` truncates to `0x9a3f2`, not `0x0e17e`). Byte-identical to the
  pre-fold text; none of the four commits touch it.

---

*Report written by the R7 fold-verification agent as its final action, per the standing
agent-persists-its-own-report rule. Nothing in the spec was edited by this review.*
