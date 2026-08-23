# mt spec — R4 fold-verification pass

Scope: mechanical fold-check of the three commits (`0bfd132`, `52ad001`,
`d2d1a58`) that respond to the three R3 lens reports
(`design/agent-reports/mt-spec-R3-{fold-check,information,implementability}.md`).
Not a fresh audit.

**Baseline moved twice during this review**, both times flagged to me
mid-task and both times operator rulings folded after dispatch, not defects:
`d2d1a58` (stated baseline) → `4527cbc` (reference pair ruled a source
constant, not build-derived) → `62a0df6` (pinned `MT_REF_HEIGHT=963_759`,
`MT_REF_TIME` from a live node) → `488a270` (removed the run-time
node-branch for the unlock estimate entirely; negative-subtraction now
warns `NO TIMELOCK` instead of printing a past year). **This report checks
against `488a270`, the actual HEAD at completion**, not `d2d1a58`. Per the
coordinator's instruction, propagation of the two originally-flagged
commits (`4527cbc`, `62a0df6`) was checked; the fourth commit (`488a270`)
was discovered only by re-running the log after finishing that check and is
folded into the same pass below.

Machine-gated facts taken as given, not re-derived, and re-confirmed at
final HEAD `488a270`: `plan-cite-check.sh` → 27/27 resolved, 0 dangling;
`spec-structure-check.sh` → 15 sections, 37 cross-refs, STRUCTURE OK
(including the new wrap-aware `SUPERSEDED_TERMS.txt` check).

## Verdict

**8 Critical+Important findings from the three R3 reports assessed for
disposition** (the ones the fold commits could plausibly have touched;
see coverage note): **6 FIXED, 0 PARTIALLY FIXED, 2 NOT FIXED
(REFUTED-as-claimed, see below), 0 OBSOLETE, 0 DEFERRED.** In addition,
**21 R3 Important findings the fold commits never claimed to touch remain
NOT FIXED** (listed in the disposition table; consistent with the fold
commits' own scoped claims).

- **The bech32 refutation does NOT hold as argued.** `52ad001` refutes R3
  lens 3's C-4 (bech32 UPPERCASE collides with EPD §6.4's all-lowercase
  clause) by checking `design/SPEC_systemwide_payloads.md` and concluding
  "EPD §6.4 ... carries NO lowercase clause ... 6.6 ADDITIONALLY hashes
  lowercase, which confirms 6.4 does not." **The primary EPD document,
  `design/SPEC_encrypted_payload_delivery.md:806-825`, directly
  contradicts this**: its §6.4 bullet list includes **"All-lowercase,"**
  with the explicit sentence *"Lowercase is what `mnemonic bundle
  --group-size 0` emits. **Pinned here at §6.4, not inside §6.6**, so the
  engraved artefact and the hash agree by construction."* R3 lens 3's
  citation (`SPEC_encrypted_payload_delivery.md:806-825`) was exact; the
  fold checked a different, secondary document instead of the primary
  source its own reviewer had cited by line number, and that secondary
  document's paraphrase was read too literally. **This is a Critical
  finding on its own** — see "The bech32 refutation" section below for the
  full analysis, including why the spec's *practical* design (store
  lowercase, uppercase only for the QR) may still be safe despite the wrong
  argument.
- **One new, still-live propagation gap in the header widening**: §10.8
  (lines 1373-1375) still describes the shared chunk header by citing
  `crates/md-codec/src/chunk.rs` verbatim — the exact 6-bit-count/37-bit
  header that `52ad001`'s own correction box (§3, lines 171-191) says is
  **no longer what `mt1` uses**. An implementer who reads only §10.8 (a
  RULED, "ready to build" open-question item) reconstructs the identical
  64-chunk-cap bug `52ad001` just fixed.
- **§4's "three unmodelled inputs" note (line 468) still says "37-bit
  `mt1` chunk header"** — stale by exactly the amount `52ad001` widened it
  (41-bit). One-line propagation miss, same class as the base45 sweep.
- **A new, unverified claim in `52ad001`'s own correction box**: "`mt
  string` keeps the 64-chunk limit because that is a property of the
  codex32 container it is engraved into (§3b), not of the header." Checked
  against `descriptor-mnemonic/crates/md-codec/src/chunk.rs`: the *only*
  places `64` appears as a chunk-**count** cap are `ChunkHeader::write()`'s
  own `(1..=64)` range check (a direct consequence of the 6-bit
  `count-1` field) and `split()`'s literal `chunks_needed > 64` (which
  exists to pre-empt that same header check). Nothing independent of
  header field width caps chunk *count* in the cited source — the 40
  bytes/chunk figure §3b cites (`SINGLE_STRING_PAYLOAD_BIT_LIMIT`) is a
  per-chunk *size* cap, a different constraint. The claim may describe a
  legitimate forward design choice for `mt-codec`'s own `split()`, but as
  written it asserts a source-grounded fact that the cited source does not
  support.
- Everything the brief specifically asked me to check propagated cleanly
  otherwise: the 41-bit header is consistent in §3/§3b/§8.7b's *numbers*
  (only the two citations above are stale); §8.7c's headroom is 15.4%
  everywhere, no stray "40%"; the seven-row success-path report landed in
  §10.10; `gettxout`'s value comparison landed in §6a; §8.2c's warning is
  narrowed correctly; the no-node warning enumerates the three skipped
  checks; bech32-stored-lowercase/uppercased-for-QR is stated once,
  consistently; the build-vs-run node language from `4527cbc`/`62a0df6`
  has zero stale "build-derived" residue after `488a270`'s further
  simplification.
- One stale illustrative number, downgraded to **Minor**: the `stderr`
  example box (§8.4, lines 957/959) and the "already passed" example
  (line 1065) still show `current height 963663`, the pre-`62a0df6`
  measurement figure, while the current embedded constant is
  `MT_REF_HEIGHT = 963_759`. `488a270` explicitly decouples these two
  concepts ("current height" is a live-node fact; `MT_REF_HEIGHT` is a
  build-time estimate input, "not an input to this estimate"), so this is
  a dated illustrative placeholder rather than a live contradiction — but
  it is the same "one commit updates a shared number, a duplicate three
  lines away is missed" pattern this cycle has already been bitten by four
  times, so it is worth naming.
- One residual R3 finding (C-9, `NO BLOCK TIMELOCK` vs `NO TIMELOCK`
  spelled two ways for the same legend field) is confirmed still live at
  final HEAD and was never in scope of any of the four commits reviewed.

## R3 finding disposition

Legend: **F**=FIXED, **PF**=PARTIALLY FIXED, **NF**=NOT FIXED,
**REFUTED-WRONG**=fold claims refutation, refutation does not hold.

| Source | ID | Sev | Finding (short) | Verdict | Evidence |
| --- | --- | --- | --- | --- | --- |
| R3 lens 1 | commit-claim / F-2 | Imp | `93df0f2` claimed 1 survivor of "engraved out-of-band reminder"; 2 existed (§10.16 wrapped) | **F** | `0bfd132` rewrote §10.16 (line ~1631: *"Nothing reaches an `mt qr` plate: §5's legend is full (§8.2c). Recorded in §7."*). Whitespace-normalised sweep for the phrase, current HEAD: 1 hit, inside §8.2c's correction box quoting *"§7 named 'the engraved out-of-band reminder' as the mitigation"* — correction context, matches `SUPERSEDED_TERMS.txt` + gate result. |
| R3 lens 1 | F-1 | Imp | base45 survives in 3 live sites (§3a diagram, §10.3 CLOSED ruling, §10.8) after bech32 reversal | **F** | `0bfd132` diff rewrites all 3: §3a's diagram now `bech32U`; §10.3 rewritten with reversal history and a bech32-specific test-plate instruction; §10.8 now says "bech32-uppercase payload." `grep -n base45`, current HEAD: 7 hits, all inside §3's comparison table / correction box / §10.3's own retraction prose — none asserting base45 as current. |
| R3 lens 3 | C-1 | Crit | Same base45 finding as F-1, independently found | **F** | Same fix, same evidence. |
| R3 lens 3 | C-3 | Crit | Ruled encoding (96 chunks) cannot be written by the ruled 6-bit-count header (64 max) | **F** | `52ad001` widens `count`/`index` to 8 bits each (41-bit header, 256-chunk ceiling), documented in §3's correction box (lines 171-191). |
| R3 lens 3 | C-4 | Crit | bech32 UPPERCASE collides with EPD §6.4's all-lowercase clause | **REFUTED-WRONG** | See "The bech32 refutation" section. The stated argument is factually wrong against the primary EPD source. The design change made alongside it (store lowercase, uppercase only for QR) may independently avoid the practical collision, but that is not verified against any admission code for a not-yet-existing `mt1` record class (§10.9/C-2 gap). |
| R3 lens 3 | C-7 (§8.7c headroom) | Imp | "roughly 40% headroom" compares QR-capacity bytes against a record-text-byte cap; real headroom 16.2%/15.4% | **F** | `52ad001` recomputes to "15.4% headroom... largest PSBT that fits is roughly 4,537 B." `grep -n "40%"`, current HEAD: only inside the correction box explaining the old figure was wrong. No live "40%" claim remains. |
| R3 lens 2 | I-1 | Crit | "mt prints every output in full at encode time" asserted twice, specified nowhere | **F** | `52ad001` adds the 7-row SUCCESS-PATH REPORT table to §10.10, first row: *"every output — address in full, amount, and which are change if a wallet was supplied."* |
| R3 lens 2 | I-2 | Crit | `gettxout`'s `value` fetched, only its null-ness used; the chain's own answer for a segwit input's value was thrown away | **F** | `52ad001` adds to §6a: *"`mt` compares the fetched `value` against the PSBT's UTXO record for that input and **refuses on mismatch**, naming both numbers."* |
| R3 lens 2 | I-3 | Imp | §8.2c's legacy warning fires on every legacy input while its body asserts a binding §8.2d now performs — the common case prints a false 11-line block | **F** | `52ad001` rewrites the firing condition: *"The legacy warning fires only when the value is UNBOUND — not on every legacy input... So it fires when, and only when, the value is bound by nothing: no `non_witness_utxo` (§8.2d), no chain fetch (§6a)."* |
| R3 lens 2 | I-4 | Imp | No node ⇒ §8.5's unspent check silently does not run, no message | **F** | `d2d1a58` adds the "NO NODE IS A WARNING, NOT A SILENCE" box in §6a, enumerating all 3 skipped checks (§8.5 unspent, §6a value match, §8.4 locktime) plus a plate-time reminder. |
| R3 lens 2 | I-5 | Imp | Plate count reported only on refusal, never on success | **F** | Success-path report row: *"the plate count — and, since a plate is ~21 minutes (F-225), the engraving time."* |
| R3 lens 2 | I-6 | Imp | Fee never stated on the success path | **F** | Success-path report row: *"the fee — absolute and as sat/vB... printed whether or not a warning fires."* |
| R3 lens 2 | I-7 | Imp | `mt string`'s chunk/character count never told to the operator on success | **PF** | Success-path row states *"the headroom — chunks against 64 (`mt string`)..."*, i.e. chunk-count proximity is now reported. The specific magnitude I-7 flagged as the dominant, unstated cost (~5,900 raw characters for a 63-chunk artifact) is still not named — only the chunk ratio is. |
| R3 lens 2 | I-8 | Imp | `mt string`'s bearer warning states the fact, withholds an "ask the operator to add a reminder" action §8.2c already uses for a lesser hazard | **NF** | §7's bearer row (unchanged): *"`mt` emits a string, not an engraving, so it **has no mechanism** to put a warning on hand-cut steel."* Two rows later, §7's wrong-input-value row (unchanged): *"An `mt string` operator controls their own plate and **may add a reminder**."* Same contradiction, neither side touched by any of the 4 commits reviewed. |
| R3 lens 2 | I-9 | Imp | `TO <amount>` semantics undefined (total vs to-wallet vs single output) | **NF** | §5's field row (unchanged): `TO <wallet id, fp or label>  <amount>`, no definition of what `<amount>` sums over. Not touched. |
| R3 lens 2 | I-10 | Imp | Nothing dates the plate; dropped-fields table's "recoverable how" claim is false for date | **NF** | §5's dropped-fields table (unchanged): `fee rate and date \| inputs − outputs, and the PSBT carries the input amounts`. Not touched. |
| R3 lens 2 | I-11 | Imp | `PLATE n OF m` doesn't say all m are required; no encode-time nudge toward duplicates | **NF** | §5's field table (unchanged), no "ALL PLATES REQUIRED" text anywhere. Not touched. |
| R3 lens 2 | I-12 | Imp | Module-size notice fires "at the point of choice"; no such point exists in the CLI | **NF** | §10.10's CLI table (unchanged): `flags \| none for locktime (§8.4)` — no module-size flag or selection surface added. Not touched. |
| R3 lens 2 | I-13 | Imp | Up to 10 stderr blocks, no ordering/severity/summary; not even tracked as unspecified | **NF, and volume increased** | `52ad001` added a 7-row report and `d2d1a58` added a 4-line no-node box — more stderr content, still no ordering/severity/summary rule, and §10.10's "Still unspecified" line still only names exit codes and refusal-message format. The finding is unaddressed and its premise (stderr volume) is now larger. |
| R3 lens 3 | C-2 | Crit | `mt qr`'s record framing undefined; `MaxRecords=24`/`MaxRecordLen=512`/public allow-list/EPD §6.3 card-set decode all unnamed and refuse every obvious framing | **NF** | `grep -n "MaxRecords\|MaxRecordLen\|512\b"`, current HEAD: 0 hits. §10.9 unchanged, still *"There is no transaction class... Adding one is the work."* |
| R3 lens 3 | C-5 | Crit | Byte/bit framing of the (now 41-bit) header unspecified; the measurement silently picked byte-alignment | **NF** | `grep -n "MSB\|byte-align\|bit-pack"`, current HEAD: 0 hits. Widening the header field widths (`52ad001`) did not add a framing rule. |
| R3 lens 3 | C-6 | Crit | 4-bit `version` field value for `mt1` never assigned; decoder hard-refuses a wrong one | **NF** | §10.13 (unchanged aside from unrelated text shifts) still rules only (a) NUMS, (b) HRP, (c) content id. No numeric `version` assignment anywhere; `grep` for an assignment returns nothing. |
| R3 lens 3 | C-8 | Imp | §4 searches/tie-breaks on module size; §8.8/§10.1 give it to the operator | **NF** | §4's objective (unchanged): `4. TIE-BREAK: maximise MODULE SIZE`, alongside §4's correction box still citing the 41-tie measurement that only matters if module size is searched. Not touched. |
| R3 lens 3 | C-9 | Imp | Legend's no-timelock line spelled two ways: `NO BLOCK TIMELOCK` (§5, §8.4 closing note) vs `NO TIMELOCK` (§8.4's own `Legend:` bullet, its `stderr` example) | **NF** | Both forms confirmed still present: `grep -n "NO BLOCK TIMELOCK\|NO TIMELOCK"` → 4 hits, 2 of each, none reconciled. Not touched by any of the 4 commits (the timelock-estimate work in `4527cbc`/`62a0df6`/`488a270` is adjacent but did not touch this specific line pair). |
| R3 lens 3 | C-10 | Imp | No rule for target height below reference height; worked example drops `~<year>`; potential `u32` underflow | **F** (superseded by `488a270`, not by the 3 commits under review) | `488a270` — landed after my dispatch, addressed in the mid-review delta — adds an explicit `target_height < MT_REF_HEIGHT` branch that warns and reads `NO TIMELOCK` instead of a past year, closing the underflow/false-projection risk C-10 flagged. Noted for completeness; not one of the 3 commits this task scoped, but directly answers a named R3 Critical-adjacent gap. |
| R3 lens 3 | C-11 | Imp | `mt string`'s stdout delimiter/grouping/casing unspecified | **NF** | §3b (unchanged): *"`mt string` emits a string. That is the whole of its output."* No delimiter/grouping/casing rule added. |
| R3 lens 3 | C-12 | Imp | `TO <amount>` semantics undefined (duplicate of I-9) | **NF** | Same as I-9. |
| R3 lens 3 | C-13 | Imp | §5 calls `FROM WALLET` mandatory; field table and §10.4 make it optional | **NF** | §5's prose (unchanged, line ~596-599): *"`FROM WALLET` is a mandatory field sized into §4's reservation..."* directly contradicts the field table's own "Optional — loudly warned when absent" two paragraphs above. Not touched. |
| R3 lens 3 | C-14 | Imp | Per-symbol `n/m` label has no placement rule relative to the 4-module quiet zone | **NF** | §10.8 (unchanged): "beside" is the only placement word; no quiet-zone interaction stated. |

## The bech32 refutation

**Does not hold, as argued.**

`52ad001`'s commit message and the spec text it produced (§3's constraint
table, line ~240: *"the `sysw` record stores **lowercase**, and `mt`
uppercases only when encoding the QR symbol... EPD §6.6 hash a form
differing from the stored bytes, leaving 'canonical' ambiguous"*) refute
R3 lens 3's C-4 with this reasoning, quoted verbatim from the commit:

> "EPD 6.4 as quoted in SPEC_systemwide_payloads carries NO lowercase
> clause: it is 'no interior spaces, no hyphens, no grouping of any
> kind', and the spec says 6.6 'ADDITIONALLY' hashes lowercase, which
> confirms 6.4 does not."

I checked this against `design/SPEC_systemwide_payloads.md:532-573`
(§5.3.1), which does read the way the commit describes — it paraphrases
"EPD§6.4 is normative and emphatic: '...no interior spaces, no hyphens, no
grouping of any kind'" and separately notes "EPD§6.6 additionally hashes
'canonical LOWERCASE records'." Read in isolation, this paraphrase is
ambiguous enough to support the fold's reading.

**But `SPEC_systemwide_payloads.md` is not EPD — it is a document that
cites EPD.** The primary source is `design/SPEC_encrypted_payload_delivery.md`,
and R3 lens 3 cited it by exact line range: `806-825`. I read that range
directly. §6.4 ("Record container — NORMATIVE"), under "Normative
constraints, all checked before any record is acted on" (line 806), lists
five bullets. The fourth (lines 814-825) is:

> "**All-lowercase.** The validators accept a consistently-uppercased
> string (`engine.setCase`, `codex32/checksum.go:132`; `verifyMDMK` folds
> case on the HRP), so without this the same wallet has two spec-legal
> encodings — and therefore two different §6.6 hashes. Verified:
> `md1qqqsyqcyq5rq…` and `MD1QQQSYQCYQ5RQ…` both return `ValidMD = true`
> and hash differently. This is not hypothetical: the device's own
> keyboard-entry path emits **uppercase**... An operator re-deriving with
> `me hash`... would then see a mismatch on an untampered payload — and
> learn that mismatches are normal, which disarms the single control §6.6
> exists to provide. Lowercase is what `mnemonic bundle --group-size 0`
> emits. **Pinned here at §6.4, not inside §6.6**, so the engraved
> artefact and the hash agree by construction."

This is unambiguous and deliberately so — the EPD author wrote "pinned
here at §6.4, not inside §6.6" specifically to foreclose the exact
misreading `52ad001` makes. R3 lens 3's citation (evidence, wording, even
the `md1qqqsyqcyq5rq…`/`MD1QQQSYQCYQ5RQ…` example) is byte-for-byte
accurate to this source. **The refutation checked a secondary,
paraphrasing document instead of the primary source its own reviewer had
already cited by line number, and the paraphrase's brevity was read as
license for a conclusion the primary source explicitly rules out.**

**Does the practical design still avoid the collision anyway?** Possibly,
on separate grounds the fold also introduced: the spec's new row states
the `sysw` record itself stores lowercase, and `mt` uppercases only when
rendering the QR symbol — i.e., the case transform happens strictly after
whatever validates the record against EPD §6.4/§6.6, not as part of the
on-wire bytes. If that mechanism is implemented exactly as described, the
bytes actually subject to §6.4's all-lowercase check never carry
uppercase, so the *design* may not collide even though the *stated reason*
for believing so is wrong. This is not verified: §10.9/C-2 (no `mt1`
sysw record class exists yet, and none of the four admission gates for it
— `MaxRecords`, `MaxRecordLen`, the public allow-list, EPD §6.3's card-set
decode — are named anywhere in this spec) means there is no code path yet
to confirm the uppercase transform genuinely happens downstream of
admission rather than upstream of it.

**Severity: Critical.** A false claim about an external, security-relevant
normative document, now embedded as live prose in a DRAFT spec column
(§3's constraint table, "EPD §6.4" row, which lists only the no-space
clause and omits the all-lowercase clause that same section normatively
carries). Per the standing "a wrong refutation is worse than a wrong
fold" instruction, this needs correcting regardless of whether the
underlying design accidentally still holds — the current text will teach
a future reader (or reviewer trusting a "REFUTED" label) that EPD §6.4 is
narrower than it is.

## Commit-claim audit

| commit | claim | verified? |
| --- | --- | --- |
| `11c6dd6` | Report-only, "nothing folded" | **Yes** — `git diff-tree --name-only 11c6dd6` touches only `design/agent-reports/mt-spec-R3-fold-check.md`. |
| `0bfd132` | New wrap-aware `SUPERSEDED_TERMS.txt` gate; fixes 4 base45/reminder sites; "Superseded-term positive control: injected live base45 in ordinary prose, gate FIRES" | **Yes.** `spec-structure-check.sh` re-run at current HEAD: STRUCTURE OK, and the gate's own logic (read, not re-derived per the brief) matches WHITESPACE-NORMALISED, case-sensitive matching against `SUPERSEDED_TERMS.txt` — confirmed the file exists with 15 terms including `base45` and `engraved out-of-band reminder`. All four named sites (§3a diagram, §10.3, §10.8, §10.16) confirmed changed in the diff and confirmed still changed at final HEAD. |
| `c34ecfc` | Report-only, "nothing folded" | **Yes** — touches only `design/agent-reports/mt-spec-R3-information.md`. |
| `cc4fdc0` | Report-only, "nothing folded" | **Yes** — touches only `design/agent-reports/mt-spec-R3-implementability.md`. |
| `52ad001` | Header widened to 41 bits/256 chunks; bech32-lowercase-for-record/uppercase-for-QR design added; C-4 "REFUTED"; §8.7c headroom recomputed 40%→15.4%; success-path report added; `gettxout` value comparison added; §8.2c warning narrowed | **Header widening: Yes**, confirmed in §3's correction box and consistent (modulo the two stale citations at §4:468 and §10.8:1373-1375, above) everywhere else the brief named (§3a, §3b, §8.7b). **§8.7c/success-report/gettxout/§8.2c claims: Yes**, all confirmed present and matching the commit's own description, verbatim, at current HEAD. **C-4 "REFUTED": claim verified as MADE, but the refutation itself does not hold** — see above. This is the one claim in this commit that is false, and it is the highest-stakes one in the commit. |
| `d2d1a58` | No-node warning enumerating 3 skipped checks, not a refusal | **Yes** — confirmed present verbatim in §6a, matches the commit's own quoted box exactly. |

**Gate-output lines**: all four commits' claimed `spec-structure-check.sh`
→ STRUCTURE OK and `plan-cite-check.sh` → 27/27 are independently
re-confirmed at final HEAD `488a270` (also 27/27, STRUCTURE OK), so none
of the later commits regressed either gate.

## New contradictions

### N-1 — §10.8 still cites `md-codec`'s unwidened `ChunkHeader` as the definition of the shared header

**Severity: Important. Section: §10.8 (lines 1373-1375) vs §3 (lines
171-191).**

§3's correction box is explicit: *"What is shared is `mt1`'s header,
identically across both verbs — not `md-codec`'s... `mt1` therefore uses 8
bits each for `count` and `index` — a 41-bit header admitting 256
chunks."* §10.8, ruling on how a recoverer learns fragment parameters,
still reads:

> "**Machine-readably this holds for both verbs, because §3 made them
> share one header.** `ChunkHeader` carries `count` and `index` — n-of-m
> — plus a 20-bit `chunk_set_id` so pieces of different transactions
> cannot be combined (`crates/md-codec/src/chunk.rs`). For `mt string`
> that header sits inside the BCH-protected chunk; for `mt qr` it rides
> in the bech32-uppercase payload. **One mechanism, both media.**"

This cites `crates/md-codec/src/chunk.rs` as the *definition* of the
shared header — i.e. the 6-bit-count, 37-bit struct that `write()` refuses
above 64. §3's own correction box exists specifically because that struct
cannot express `mt qr`'s 96-chunk artifact. §10.8 was not swept when §3
was corrected. An implementer who builds from §10.8 alone (a numbered,
RULED open-question item — exactly the kind of section this spec's own
numbering exists to make authoritative) reconstructs the identical
unbuildable-encoding bug `52ad001` fixed.

**Control, confirming the citation predates the fix and was not touched
by it:** `git show 52ad001 -- design/SPEC_mt_v0_1.md` (already read in
full for this task) does not touch any line in the §10.8 range; `git log
-p -S 'md-codec/src/chunk.rs' -- design/SPEC_mt_v0_1.md` behaviour is
consistent with this citation being unmodified since before the R3 cycle.

### N-2 — §4's "three unmodelled inputs" note still says the header is 37-bit

**Severity: Minor. Section: §4, line 468.**

§4's plate-table caveat: *"the **37-bit `mt1` chunk header per symbol**
(§3)..."* — stale by exactly the 4 bits `52ad001` added. `grep -n
"37-bit\|41-bit"`, current HEAD: the only other "37 bits" occurrences
(lines 174, 1518) are inside §3's own correction box (historical,
describing the OLD unbuildable header) and §10.13's historical account of
what R0 round 1 originally read from `md-codec` — both legitimate. Line
468 is the one live, non-historical assertion of the old width, and it
sits in a note whose whole purpose is enumerating what §4's table does
not yet model — an odd place for a stale number to survive unnoticed,
since §10.14's required regeneration (which this note explicitly points
at) will need the *correct* header size to compute its own additive
input.

Minor rather than Important: this is a documentation-caveat number, not a
buildable-artifact-determining one — nothing downstream keys off "37" vs
"41" here the way §10.8's citation does.

### N-3 — the "codex32 container, not the header" justification for `mt string`'s continued 64-chunk cap is not supported by the source it is built on

**Severity: Minor/Important (borderline — see reasoning). Section: §3,
lines 188-191.**

§3's correction box: *"`mt string` keeps the **64-chunk limit** because
that is a property of the codex32 container it is engraved into (§3b),
not of the header."* I read `descriptor-mnemonic/crates/md-codec/src/chunk.rs`
directly (the file cited two paragraphs earlier in the same box). The only
two places `64` functions as a chunk-**count** ceiling are:

- `ChunkHeader::write()`: `if !(1..=64).contains(&(self.count as u32))` —
  a direct consequence of the 6-bit `count-1` field width;
- `split()`: `if chunks_needed > 64 { return Err(ChunkCountExceedsMax...) }`
  — a literal mirroring the header's own cap, not an independent
  container-level rule.

`SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64 * 5 = 320` bits (which §3b cites for
the "40 payload bytes/chunk" figure) is a **per-chunk size** cap tied to
codex32's 80-symbol long-form ceiling, not a per-set **count** cap — a
different constraint entirely. I found no source-level evidence for a
codex32-container property that limits total chunk *count* independent of
header field width. The claim may describe a legitimate forward design
decision (mt-codec's own fork of `split()` could simply choose to keep
enforcing ≤64 for `mt string` regardless of the wider header, e.g. to
preserve §3b's already-measured tables), but as written it asserts this is
a property of "the container," which the cited source does not
demonstrate, and the commit message's own framing ("Every finding
verified against source or recomputed before editing") sets the bar this
specific sentence does not clear. I rate this Minor rather than Important
because it produces no implementer-visible divergence on its own — both
readings agree on the number (64) — but flag it because it is a fresh,
unverified factual claim in the exact section that just fixed an
adjacent, structurally identical error (asserting a header property
without checking field widths against source).

## Coverage note

Per the brief, citations (27/27) and structure (including the
whitespace-normalised superseded-term check) are machine-gated and were
re-confirmed at final HEAD, not re-derived. The `SPEC_systemwide_payloads.md`
vs `SPEC_encrypted_payload_delivery.md` distinction was resolved by reading
both documents directly rather than trusting either commit message's
description of what either says. Not covered: R3's Minor/Nit findings
(spot-checked only where directly relevant to a Critical/Important
disposition above); whether any NOT FIXED item is safe to leave open
versus load-bearing enough to block the next gate — that is a severity and
scheduling call for whoever runs the next round, not a fold-verification
question; and full re-derivation of the §8.7c bit-arithmetic (34,656 bits
/ 6,932 characters) against R3 lens 3's own differently-derived 6,863
figure — both land in the same 15-16% headroom range and I could not
determine from the spec text alone which per-chunk padding convention
each used, so I report the discrepancy exists (visible by comparing
`RESULTS_ecc_selection_2026-08-22.txt`-cited figures against the fold's
recomputation) without asserting either is wrong.
