# R5 — fold-verification pass over the R4 fold (`01b280d`, `322bbb5`)

Scope: mechanical check, not a fresh audit. Spec frozen and verified at
`322bbb5f7ebc04dca3ef95b92c3be86f2a2bd026` throughout this review (`git log -1`
re-checked at the end of the pass, unchanged, clean tree). Inputs: the three R4
lens reports (`mt-spec-R4-codec-assumptions.md`, `mt-spec-R4-cli-assumptions.md`,
`mt-spec-R4-fold-check.md`), the two fold commits' full messages and diffs, and
`design/SPEC_mt_v0_1.md` read in the relevant sections at current HEAD.

**No process failure to report** — the freeze held.

## Verdict

**R4 Critical/Important dispositions: 34 items scored** (8 from lens 1 codec,
15 from lens 2 CLI/transport, 2 lens-3-native findings — the bech32 refutation
and N-1 — plus N-2/N-3 included for completeness though Minor). Breakdown:

- **FIXED: 9** (A-1, A-2, A-3, A-7, A-9(i), B-1, the bech32-refutation retraction,
  N-1, N-2)
- **PARTIALLY FIXED: 2** (A-5, B-2)
- **NOT FIXED: 20** (A-4, A-6, A-8, A-9(ii), A-10, B-3, B-4, B-5, B-7, B-8, B-9,
  B-10, B-11, B-13, B-14, B-15, N-3, plus B-16–B-23 not independently re-checked
  beyond grep — consistent with the fold's own stated scope)
- **STALE: 2** (B-6 — acknowledged by the fold commit; B-12 — same class, **not**
  acknowledged by the fold commit, see below)
- **OBSOLETE: 0. DEFERRED: 1** (A-1's domain-string sub-question was DEFERRED
  in `01b280d` and then FIXED in `322bbb5` — counted as FIXED above, the
  deferral itself was honest and closed one commit later)

**One new contradiction found, Important**: §11 (line 1873) still asserts
"chunk sizing is a flat 40 payload bytes," directly contradicting §3b's own
corrected text three sections earlier, which the same fold cycle rewrote to say
`md-codec` **balances**. This is the exact defect class A-5 was filed against,
surviving in the second of the two sites A-5 itself named to fix.

**The EPD §6.4 retraction holds.** Verified directly against the primary
source file, not just against the lens-3 report's quotation of it — see below.

## R4 finding disposition

| Source | ID | Sev | Finding (short) | Verdict | Evidence |
| --- | --- | --- | --- | --- | --- |
| Lens 1 (codec) | A-1 | Crit | `mt1`'s NUMS domain string not derivable, only the derivation rule is | **F** | `322bbb5` rules `"shibbolethnumstransaction"` → `MT_REGULAR_CONST = 0x1a2fc877f9528d7c1`, appears at exactly 3 sites (`grep -c`: 1634, 1641, 1835). **Recomputed independently in Python**: `sha256("shibbolethnumstransaction").hexdigest() == "d17e43bf...97978f6"`, top 65 bits `== 0x1a2fc877f9528d7c1`, `bit_length() == 65`, differs from both `MD_REGULAR_CONST` and `MK_REGULAR_CONST`. All four of the commit's claimed checks reproduce exactly. |
| Lens 1 | A-2 | Crit | Is `count` stored as `count−1`? Fold widened the field, deleted the answer | **F** | `01b280d`, §10.13(a2) field table, line 1661: *"`count` \| 8 \| **`count − 1`**, matching `md-codec`'s offset convention: a set of 1 stores `0`, a set of 256 stores `255`"*; `index` stated plain/zero-based (line 1662). |
| Lens 1 | A-3 | Crit | 4-bit `version` value unassigned | **F** | Line 1658: *"`version` \| 4 \| **`0b0001`** — `mt1` wire v1. Not inherited from `md1`..."* |
| Lens 1 | A-4 | Crit | Last chunk's payload length has no signal for `mt qr`; no `MessageLen` decision | **NF** | `grep -n "MessageLen\|payload_byte_count\|symbol_aligned_bit_count"` at current HEAD: the only `MessageLen` hit (line 1437) is inside the retracted-UR historical note. No framing decision (zero-pad vs length-delimit vs explicit field) was added for `mt qr`. Not claimed fixed by either commit. |
| Lens 1 | A-5 | Imp | "Flat 40 bytes per chunk" mis-describes `md-codec`, which balances; A-5 named **two** sites, §3b and §11 | **PF** | §3b (lines 366–378) rewritten correctly: *"An earlier version of this box called it 'a flat 40 bytes per chunk', and that mis-describes the chunker — R4 lens 1"*... *"`mt1` balances too."* **§11 (line 1873) was not touched** and still reads: *"§3b's correction established that chunk sizing is a flat 40 payload bytes (`crates/md-codec/src/chunk.rs:224,253-254`), so the count is *exact*"* — the opposite of what §3b now says, citing the same lines. See "New contradictions" below. |
| Lens 1 | A-6 | Imp | One transaction, two chunk sets (raw tx via `mt string`, PSBT via `mt qr`) share one `chunk_set_id`; no discriminator | **NF** | `grep -n "sniff\|payload-type discriminator\|which parser"`: no hits. §10.13(c) (line 1691) still derives `chunk_set_id` from "the EXTRACTED transaction's txid" for **both** verbs, with no mechanism to tell a raw-tx chunk set from a PSBT chunk set apart. Not claimed fixed. |
| Lens 1 | A-7 | Imp | `chunked` bit: no stated value, no stated function | **F** | Line 1659: *"`chunked` \| 1 \| **`1`, always, and RETAINED** even though `mt1` is always chunked — see below"*, with the rationale (dropping it silently shifts every later field) at lines 1670–1675. |
| Lens 1 | A-8 | Imp | Reassembly semantics (duplicates, gaps, mismatch) unwritten | **NF** | `grep -n "duplicate chunk\|ChunkSetIncomplete\|re-scan"`: no hits. Not claimed fixed. |
| Lens 1 | A-9(i) | Minor | Bit order/padding unstated | **F** | Lines 1677–1681: *"Fields are written most-significant-bit first... followed immediately by the chunk payload with **no padding between them**; padding appears only once, at the end of a chunk..."* |
| Lens 1 | A-9(ii) | Minor | §10.13(b) names HRP as `mt1` where the checksum wants `mt` | **NF** | Line 1650 unchanged: *"**(b) Its own HRP**, `mt1`, currently hardcoded at four sites in `md-codec`."* Still conflates the checksum-domain HRP with the printed prefix; no `hrp_expand("mt")` correction added. `grep -n hrp_expand`: 0 hits. |
| Lens 1 | A-10 | Nit | Codex32 capacity margin (73/80 symbols) unrecorded | **NF** | `grep -n "73 data symbols\|REGULAR_DATA_SYMBOLS_MAX"`: 0 hits. |
| Lens 2 (CLI) | B-1 | Crit | Zero CLI flags named; 7 operator inputs unrouted | **F** | §10.10 (lines 1561–1584): *"THE SPEC NAMES ZERO FLAGS while requiring SEVEN operator inputs the PSBT cannot supply — R4 lens 2"*, with a table of 7 non-PSBT inputs (plate budget, `FROM`, `TO` id, `TO` free text, input values, module size, node location) each with an absent-behavior. Matches B-1's own 7-row table one-for-one. |
| Lens 2 | B-2 | Crit | 4 candidate `sysw` record framings give 4 different §8.7c ceilings; the conformant one refuses the spec's own largest artifact | **PF** | §8.7c (lines 1228–1245) retracts its numeric ceiling and states all four candidate numbers (3,671/4,094/4,476/4,525) plus the 322 B shortfall, explicitly deferring to "§10.9's record framing is a prerequisite." The **false number is gone** and the dependency is now honestly named — but the underlying decision (which framing) is still undecided, and no new open-question item tracks it for closure (§10 runs 1–22, none titled "record framing"). |
| Lens 2 | B-3 | Crit | §4's chosen config and §5's legend have no channel into the payload | **NF** | Not addressed; no new record/field mechanism described. |
| Lens 2 | B-4 | Crit | `mt qr` output encapsulation/destination undecided (bare/region/UF2; stdout/`--out`) | **NF** | `grep -n "UF2\|REGION_LEN\|--region"`: 0 hits in the spec. |
| Lens 2 | B-5 | Crit | Node location, credentials, timeout, non-answer classification all unspecified | **NF** | `grep -n "rpc-url\|rpc-cookie\|cookie file\|timeout"`: 0 hits. |
| Lens 2 | B-6 | Crit | Engraved `~<year>` depended on operator's network (live node vs embedded constant) | **STALE, acknowledged** | Fold commit explicitly: *"STALE THROUGH MY OWN DRIFT... True at its 4527cbc baseline, fixed two commits later by the 'embedded timestamp only ever' ruling."* Confirmed: current spec (line 1067 area) computes the estimate from `MT_REF_HEIGHT`/`MT_REF_TIME` only, no live-node branch for the *estimate* (`488a270`, pre-dating all three R4 lenses). |
| Lens 2 | B-7 | Imp | Refusal format unspecified; `§8` numbering has `7c` before `7b`, and item 1/item 3 are the same check under two numbers | **NF** | §8's numbering unchanged: `1, 2, 2b, 2c, 2d, 3, 4, 5, 6, 7, 7c, 7b, 8, 9` (confirmed via `sed`/grep at current HEAD). Item 1 ("Not fully finalized → refuse") and item 3 ("An unsigned or unfinalized transaction... → refuse") both still present, unmerged. Not claimed fixed by either commit. |
| Lens 2 | B-8 | Imp | Exit codes unspecified | **NF** | Line 1586: *"**Still unspecified:** the flag spellings themselves, exit codes, and the format of the refusal messages..."* — unchanged from before the fold. |
| Lens 2 | B-9 | Imp | Input encoding (binary/base64/sniff) and file-vs-stdin shape unspecified | **NF** | No change to §10.10's `"a finalized PSBT... from a file or stdin, equivalently"`; no sniff/encoding rule added. |
| Lens 2 | B-10 | Imp | §8.9 "Secrets → refuse" has no defined subject on a PSBT input | **NF** | `grep -n "is_seed\|8\.9"`: item 9 (line 1263) unchanged: *"**Secrets** → refuse, as `me` already does for `ms1`."* No scoping to operator-supplied strings added. |
| Lens 2 | B-11 | Imp | Success report: no row format/ordering rule; change-detection unsatisfiable from a `FROM WALLET` hint; provenance enumeration incomplete (segwit `witness_utxo`+no-node case) | **NF** | The 7-row table itself predates this fold (added in `52ad001`/R3). None of B-11's sub-issues (format pinning beyond the locktime line, ordering vs. warnings, the change-detection derivation problem, the missing 4th provenance state) were touched — table text at lines 1551–1559 unchanged from before `01b280d`. |
| Lens 2 | B-12 | Imp | Negative block-height delta (`target_height < reference_height`) had 4 divergent candidate behaviors, incl. a debug-build panic | **STALE, unacknowledged** | Same underlying fix as B-6: `488a270` (pre-dating all three R4 lenses) added the explicit `target_height < MT_REF_HEIGHT` branch → `NO TIMELOCK` (confirmed at lines 1067, 1074). This is the same staleness class as B-6 but the fold commit message names **only** B-6 as stale — B-12 isn't mentioned at all, despite being resolved by the identical prior commit. Not a spec defect (the fix is real and correct), but the fold's own accounting of "what was stale vs. what I fixed" undercounts by one. |
| Lens 2 | B-13 | Imp | §4's tie-break still not a total order: 2×3 vs 3×2 tiling orientation ties | **NF** | Objective list (lines 458–462) unchanged at 5 steps; no 6th key for orientation added. |
| Lens 2 | B-14 | Imp | `Class::is_secret()` for the new `mt1` class, and the F1-flag consequence, undecided/unstated | **NF** | `grep -n "is_secret\|flagSecretInPlaintext"`: 0 hits. |
| Lens 2 | B-15 | Imp | Teaching the shared classifier `mt1` silently changes `me convert`/`bundle`/`seal` behavior | **NF** | `grep -n "me convert\|RefusedSecret\|Format::Mt"`: 0 hits. |
| Lens 3 (fold-check) | bech32 refutation | Crit | `52ad001` wrongly refuted R3's EPD §6.4 all-lowercase Critical, checking a secondary document instead of the cited primary source | **F, and correct** | `01b280d` retracts it in §3 (lines 226–253), quoting the primary source's actual clause. **Independently re-verified against `design/SPEC_encrypted_payload_delivery.md:806-825` directly** (not just against the lens-3 report's quote of it) — the "All-lowercase... Pinned here at §6.4, not inside §6.6" bullet is present, word-for-word. See "The EPD §6.4 retraction" below. |
| Lens 3 | N-1 | Imp | §10.8 (item 8) still cited `md-codec`'s unwidened, 37-bit `ChunkHeader` as the shared header's definition | **F** | Item 8 (line 1430) now reads: *"It is `mt1`'s own 41-bit header, not `md-codec`'s 37-bit one (§3): the latter's 6-bit `count` caps a set at 64 chunks, which `mt qr` exceeds."* |
| Lens 3 | N-2 | Minor | §4's "three unmodelled inputs" note (line 468, pre-fold) still said "37-bit `mt1` chunk header" | **F** | Now reads "the **41-bit `mt1` chunk header** per symbol" (line 509). |
| Lens 3 | N-3 | Minor/Imp (borderline) | "`mt string` keeps the 64-chunk limit because that is a property of the codex32 container... not of the header" — unsupported by the cited source | **NF** | §3's correction box (lines 189–191) unchanged verbatim: *"`mt string` keeps the **64-chunk limit** because that is a property of the codex32 container it is engraved into (§3b), not of the header."* Same unverified claim lens 3 flagged; not touched by either commit. |

## The EPD §6.4 retraction

**Holds.** Two independent checks:

1. **Against the lens-3 report's own quotation** — `01b280d`'s spec text
   (§3, lines 234–239) quotes the primary source's "All-lowercase... Pinned
   here at EPD §6.4, not inside EPD §6.6" sentence, matching the lens-3
   report's quotation of the same passage verbatim.
2. **Against the primary file itself, read directly** (not trusting either
   commit message or the lens-3 report to transcribe it correctly):
   `design/SPEC_encrypted_payload_delivery.md:814-825` —

   > "**All-lowercase.** The validators accept a consistently-uppercased
   > string (`engine.setCase`, `codex32/checksum.go:132`; `verifyMDMK` folds
   > case on the HRP), so without this the same wallet has two spec-legal
   > encodings — and therefore two different §6.6 hashes. Verified:
   > `md1qqqsyqcyq5rq…` and `MD1QQQSYQCYQ5RQ…` both return `ValidMD = true`
   > and hash differently. ... Lowercase is what `mnemonic bundle
   > --group-size 0` emits. Pinned here at §6.4, not inside §6.6, so the
   > engraved artefact and the hash agree by construction."

   This is exactly the clause the original `52ad001` refutation denied
   existed, exactly where R3 lens 3 said it was (line range 806–825 in that
   file — the "All-lowercase" bullet sits at 814–825, inside the 806-825
   range cited).

**§3's constraint table now states both clauses accurately** (lines 264–265):
row 1 covers "no interior spaces" (✓, satisfied by the 32-character bech32
alphabet); row 2, labeled explicitly *"EPD §6.4 — ALL-LOWERCASE, a second
clause of the same rule"*, states it is satisfied "only because the record
stores lowercase," with bech32's uppercase form reserved for the QR and
"never reach[ing] a record." No false claim that the clause doesn't exist
remains anywhere I searched (`grep -n "no lowercase\|NO lowercase clause"` →
0 hits at current HEAD).

**The retraction correctly declines to over-claim the practical design is
verified**: both the lens-3 report and the current spec text agree the
uppercase-for-QR/lowercase-for-record mechanism is *unverified* pending
§10.9's still-open `sysw` record-class admission work (consistent with B-2's
PARTIALLY-FIXED disposition above — the same open dependency).

## Commit-claim audit

| commit | claim | verified? |
| --- | --- | --- |
| `01b280d` | "All three R4 lenses folded in one commit" | **Yes** — `git show --stat` touches only `design/SPEC_mt_v0_1.md` (169 insertions, 16 deletions). |
| `01b280d` | Retraction of the bech32/EPD §6.4 refutation | **Yes, and correct** — verified against primary source directly, above. |
| `01b280d` | "The header's exact 41-bit layout is now stated field by field: version = 0b0001, chunked = 1 and RETAINED, chunk_set_id, count stored as COUNT-1, index plain and zero-based, MSB-first, no padding" | **Yes** — field table at lines 1656–1662 and prose at 1664–1681 match this description exactly, field by field. |
| `01b280d` | "'FLAT 40 BYTES PER CHUNK' MIS-DESCRIBED md-codec... Chunk COUNTS are unaffected... per-chunk SIZES differ" | **Yes, but only in §3b.** §11 (line 1873) still asserts the pre-correction claim verbatim — see "New contradictions." |
| `01b280d` | "8.7c LOSES ITS NUMBER... Four candidates give four ceilings -- 3,671/4,094/4,476/4,525 B... only EPD-conformant one refuses section 4's own largest artifact by 322 B" | **Yes** — all four numbers and the 322 B figure appear verbatim at lines 1230–1236. |
| `01b280d` | "THE SPEC NAMED ZERO FLAGS... All seven are now tabulated" | **Yes** — 7-row table at lines 1568–1580, matching B-1's own 7 inputs one-for-one. |
| `01b280d` | "10.9's GATE CLAIM CORRECTED BEFORE IT LANDED... MaxRecords=24/MaxRecordLen=512 are SEAL gates, not sysw ones... never reached this spec" | **Yes** — item 9's box (lines 1479–1493) states this correction, and independently, `grep -n "MaxRecords\|MaxRecordLen"` over the spec returns 0 hits — confirming the wrong numbers indeed never landed in the spec text (only the correction narrative mentions them by name, which is itself accurate reporting, not a live false claim). |
| `01b280d` | "FILED, NOT FOLDED: 10.22... Operator decision" | **Yes** — item 22 pre-`322bbb5` was the placeholder; `322bbb5` closes it one commit later, as designed. |
| `01b280d` | "STALE THROUGH MY OWN DRIFT... lens 2's B-6" | **Yes, correctly identified as stale** — but see B-12 in the disposition table: the same staleness class exists and is unacknowledged. |
| `01b280d` | Gate output: `spec-structure-check.sh → STRUCTURE OK`, `plan-cite-check.sh → 30/30, 0 dangling` | **Consistent with the task brief's "already settled" figures** (30/30, STRUCTURE OK) — not independently re-run per the brief's instruction not to re-derive machine-gated facts. |
| `322bbb5` | `sha256("shibbolethnumstransaction") = d17e43bf...97978f6`; top 65 bits `0x1a2fc877f9528d7c1`; bit length 65; differs from both siblings | **Yes, recomputed independently in Python, exact match on every figure.** |
| `322bbb5` | "Constant appears at exactly three sites: 10.13(a) ruling, 10.13(a) recomputation, 10.22 closure" | **Yes** — `grep -c "0x1a2fc877f9528d7c1"` → 3, at lines 1634, 1641, 1835, matching the described locations exactly. |
| `322bbb5` | "10.13 now has no undecided input left" | **Consistent with spec text** — item 13 now rules (a) NUMS, (b) HRP, (a2) header layout, (c) content id, with no remaining "undecided" language in that item. (Separately, other open items — A-4, A-6, A-8, B-2 through B-15 above — remain genuinely open elsewhere in the spec; the claim is scoped to §10.13 specifically and is accurate as scoped.) |

## New contradictions

### C-1 — §11 still asserts "chunk sizing is a flat 40 payload bytes," directly contradicting §3b's own correction three sections earlier

**Severity: Important. Section: §11 (line 1873) vs §3b (lines 366–378).**

§3b, as corrected by this fold: *"An earlier version of this box called it 'a
flat 40 bytes per chunk', and that mis-describes the chunker — R4 lens 1...
`md-codec` computes `chunks_needed` against the 320-bit ceiling and then
**balances** the payload across that many chunks (`crates/md-codec/src/chunk.rs:267`)... the **per-chunk sizes** differ."*

§11, untouched by either commit, three sections later:

> "§3b's correction established that chunk sizing is a **flat 40 payload
> bytes** (`crates/md-codec/src/chunk.rs:224,253-254`), so the count is
> *exact* for a given payload size."

This cites the exact same source lines (`chunk.rs:224,253-254`) that §3b's
new correction explicitly says supports only the *ceiling*, not a flat
per-chunk size, and asserts the opposite of what §3b now says "established."
The *conclusion* §11 draws from it (chunk count is exact/unaffected) happens
to still be true — that part is a separate, correct claim already stated
independently in §3b ("chunk COUNTS are unaffected"). But the *premise* §11
states as settled fact ("chunk sizing is a flat 40 payload bytes") is the
identical mis-description A-5 was filed against, and A-5's own text named
**both** §3b and §11 as the sites needing the fix — only one was edited.

An implementer or reviewer reading §11 in isolation (a "provenance of the
numbers" section, plausibly read on its own to sanity-check a citation) will
conclude chunk sizes are flat and get the same wrong per-chunk-size model
A-5 exists to prevent — while §3's earlier correction box has already
retracted that exact conclusion.

**Not caused by `322bbb5`** — this line was untouched by both commits under
review; it predates the R4 cycle. It is reported here because the task asked
specifically whether "any surviving text [is] still describing a flat size,"
and this is a live, current-tense assertion of exactly that, not a
historical/quoted one (compare to the §3 line-174 and §10.13 line-1626
occurrences of "37 bits," both explicitly framed as describing a past,
superseded state, and correctly left alone).

No other new contradictions found. Everything else the task named as a
propagation surface — the 41-bit header layout across §3/§3a/§4/§8.7b/§10.8,
the NUMS constant's three sites, §8.7c's absence of a numeric ceiling, and
the seven-input CLI table — checked out consistent.

## Coverage note

Machine-gated facts (30/30 citations, STRUCTURE OK) taken as given per the
brief, not re-run. `grep`/`sed` searches used throughout are stated inline
with what was searched; where a negative is reported ("0 hits"), the search
pattern is given so the scope is auditable. The R3-era disposition table
inside `mt-spec-R4-fold-check.md` (F-1, F-2, C-1 through C-14, I-1 through
I-13) was **not** re-derived here — that is R3 material already dispositioned
by that report itself; this pass covers only the R4-native findings (lens 1's
A-series, lens 2's B-series, and lens 3's own bech32-refutation/N-series
findings) against the two commits that respond to them. B-16 through B-23
(Minor/Nit in lens 2) were grep-spot-checked only where a hit would have
changed the Critical/Important picture; none did, and none is claimed fixed
by either commit's own message, so they are omitted from the disposition
table rather than padded in as unverified NFs.
