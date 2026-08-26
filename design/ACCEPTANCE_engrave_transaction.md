# ACCEPTANCE — Engrave a Transaction

**Written 2026-08-25 (phase P2 of `FORWARD_PLAN_post_experiment.md` §4).
THIS IS THE NORMATIVE SURFACE FROM NOW ON.** It replaces
`design/IMPLEMENTATION_PLAN_P1_me_container.md` (retired, superseded marker at
its head) and supersedes `design/SPEC_engrave_transaction.md` **as the statement
of what must be true to ship** — the spec keeps its authority as the statement of
*why*, and every one of its requirements is classified below.

Two operator rulings dated **2026-08-25** (`design/FOLLOWUPS.md`) bind over the
spec wherever they disagree. A requirement they changed is `SUPERSEDED`, not
`NOT-MET`.

Every MET below carries a file:symbol or a test name. Every NOT-MET carries an
owning phase and a done-condition (§6). **Machine-checked at the time of
writing**, logs read rather than exit codes:

```
me:          cargo nextest run --locked   -> 333 tests run: 333 passed, 1 skipped
fork:        go test ./sysw/ ./mt/ ./txqr/ ./codex32/   -> 4x ok
fork:        go test -run TestZxing -v ./txqr/   -> PASS (RAN; ZXingReader present)
fork gui:    16/16 transaction tests PASS, no skips
mt:          scripts/check-provenance.sh  -> every copied file matches its source
both repos:  MTX1 in *.go/*.rs -> 0 files;  wtxid -> 1 doc comment, 0 code
```

**Re-measured at the end of phase P3a**, logs read rather than exit codes:

```
me:    cargo nextest run --locked --no-fail-fast -> 366 tests run: 366 passed, 1 skipped
       cargo clippy --all-targets --locked       -> 0 warnings
fork:  gui-shard-test.sh ./gui/ 24 -> all 982 tests ran across 24 shards, ok
       go test $(go list ./... | grep -v /gui$) -> every package ok
sheet: scripts/acceptance-count.py -> §4.5's numbers, measured
```

**Two `me` gates that had been skipping silently** — `cross_lang` and
`preview_cross_lang` return early when `go` is off `PATH`, and they also need
`third_party/seedhammer`, which `git worktree add` does not create. Both RUN
and pass in the figure above. Same class as G-P5.8.

---

## 1. THE LIVE RULE LIST — what the code actually enforces

Not what the spec proposed. Sources: `crates/me-cli/src/sysw/{mod,record,mt,tx,wire}.rs`
(Rust primary) and `seedhammer.com/{sysw,mt,txqr,codex32,gui}` (port).

| # | rule | where |
| --- | --- | --- |
| L1 | A `tx:` record's body must be **lowercase hex AND parse as one serialized transaction AND carry a signature on every input**; anything else is `Class::Unknown` and refused at pack. | `sysw/mod.rs:150-168`, `sysw/tx.rs:182-259` |
| L2 | **Every input carries a non-empty scriptSig or ≥1 witness item** (`every_input_signed`). This replaces the retired E17/`wtxid`. It is per-INPUT, not per-transaction. | `sysw/tx.rs:59,257`; RED tests `a_signature_stripped_transaction_is_refused`, `one_signed_input_does_not_vouch_for_the_others` |
| L3 | An `mt1` record is admitted by **strict** validity — exact BCH, consistent case, parseable header. **Error correction is never applied**, on either side. | `sysw/mt.rs:47-84`; `codex32/mtdata.go:35`; `codex32/correct.go:44-76` has no `"mt"` case |
| L4 | A chunk set is **confirmed** only if it reassembles with zero corrections, the bytes **parse** as a transaction, and the set's `chunk_set_id` **equals the top 20 bits of the derived display txid**. | `sysw/mt.rs:124-138`; device `mt/mt.go:218-224` |
| L5 | **The device computes L4 itself.** No confirmed-flag byte exists in the wire format. | `gui/transaction.go:287,314,391`; `sysw/confirm.go:151-186` |
| L6 | **The txid is always DERIVED, never carried.** No code path displays a txid the device did not recompute. `txCandidate.confirmed` zero-value is `false`. | `mt/mt.go:303-317`; `gui/transaction.go:481-488` |
| L7 | **Nothing in the chunk path refuses.** An incomplete or non-decoding set packs, reaches the device, and engraves under an **un-overridable substituted legend**. | ruling 2026-08-25/25b; `main.rs:1272-1283`; `gui/transaction.go:99-107,284-299,605-623` |
| L8 | Neither `Class::Mt` nor `Class::Tx` is **secret**. | `sysw/record.rs:65-70`; `sysw/record.go:53-55`, tested `sysw/mt_records_test.go:45` |
| L9 | Container section cap is the **formula** `(65536−52−16)/2 = 32,734`, compile-asserted in both languages; `seal` stays frozen at 8191; NFC's 8 KiB buffer is untouched. | `sysw/wire.rs:61,65,68`; `sysw/wire.go:64,86,90`; `gui/scan.go:31` |
| L10 | QR carries **raw transaction bytes in byte mode always**, one symbol or Structured Append for 2..16, hard 16-symbol cap enforced twice. | `txqr/txqr.go:38,50-52,74-141,106-111` |
| L11 | The plate search minimises **plates, then symbols**, with **ECC floor M as a constraint** (`H,Q,M` — M is the lowest ever tried) and module 0.9 mm before 0.6 mm. **0.3 mm is never emitted.** | `gui/transaction.go:677-684,692-724`; ECC order `:708` (`{qr.H, qr.Q, qr.M}` — M is the lowest ever tried), module order `:709` (`{3, 2}` = 0.9 mm then 0.6 mm) |
| L12 | The **legend and title are cut LAST**, asserted by knot order, not by appearance. | `backup/backup.go:493-517`; `TestTheTitleAndFooterAreEmittedLast`, `TestTransactionPlateCutsItsTitleLast` |
| L13 | Output destinations that are group- or other-readable are refused unless `--allow-world-readable`; character devices exempt; `--out` creates and `fchmod`s to 0600. | `main.rs:923-943,1425-1430`; `tests/world_readable_output.rs` |
| L14 | Text plates pack as many `mt1` strings as fit, in index order, at the **3.0 mm tested floor**, with the **real layout** (`toPlate`) as the fitting oracle. | `gui/transaction.go:38,594-637` |

---

## 2. THE VECTOR LIST — what exists, and what each proves

| vector / fixture | proves |
| --- | --- |
| `EVEN_RAW_HEX` (222 B, node-produced, `sysw/tx.rs:268`) + `EVEN_TXID` | the structural parser reproduces a real node's txid, and `chunk_set_id() == 0x2dcf2` |
| `EVEN_STRIPPED_HEX` (113 B, `sysw/tx.rs:279`) | **the artifact the whole cycle turns on** — same txid as the honest body, parses, round-trips, and fails the signature predicate |
| `MIXED_STRIPPED_HEX` (`sysw/tx.rs:286`) | the predicate must be per-INPUT: an `any` mutation leaves the first test green and turns this one red |
| the 6-string `EVEN` mt1 set (`sysw/mt.rs:166`), cross-checked against a fresh encode | the pinned corpus and the encoder cannot drift apart silently |
| entropy-as-a-complete-set (`entropy_wrapped_as_a_complete_mt_set_is_unconfirmed`) | the C3 smuggling channel: reassembles, does not parse, unconfirmed |
| real tx under a foreign set id (`a_real_tx_under_a_foreign_set_id_is_unconfirmed`) | the txid↔`chunk_set_id` binding, at 2^-20 |
| `mt/mt_test.go` `even` / `uneven` / `smuggled` / `foreign` | the same four properties on the device side, with the **Rust answers verbatim** |
| `codex32/mtdata_test.go:68` `TestMTTargetReproducesFromDomain` | mt1's NUMS target is re-derived from `SHA-256("shibbolethnumstransaction")` and differs from md's and mk's |
| `gui/testdata/sysw_mt_payload.bin` + `TestHostPackedMtPayloadLoadsAndConfirms` | **the cross-language seam** — bytes the Rust `me` binary wrote, read end-to-end by device code. The F-212 class |
| `TestZxingMergesTheSetBackToTheTransaction` | an **independent mainstream decoder** (ZXing) merges our Structured Append set back byte-identically, k ∈ {1,2,3,6}, **reverse** scan order |
| `txqr/capgate_test.go` (4 tests) | the vendored library's real capacity equals the **published ISO/IEC 18004 v40** limits in 3 modes × 4 EC levels; SA costs exactly its header; k=1 is byte mode too (F-234 regression) |
| `sysw/cap_test.go` `TestTheSectionCapMatchesTheRustPrimary` | the port reads the **primary's source text** for the formula — a retyped constant is how a port silently forks |
| `design/vectors/mt1_v1_vectors.json`, `scripts/gen-mt1-vectors.py` | an encoder independent of `mt-codec` generated the corpus, so the fixture can falsify the code it checks |
| `design/measurements/SA_FIXTURE.txt`, `scripts/gen-sa-fixture.py` | an independent `segno` Structured-Append pair, **committed and pinned but not yet consumed by any gate** — see G-P4.2 / G-P5.2 |

**Not covered by a vector:** `sysw_vectors.json` — the container's own vendored
conformance corpus — has **zero** `mt1`/`tx:` rows on either side (G-P5.3).

---

## 3. REFUSAL / FLAG COVERAGE

| condition | host | device | authority |
| --- | --- | --- | --- |
| `tx:` record **on argv** | **REFUSE** exit 3, naming `--in`/stdin | n/a | R2, G-P3.5 |
| `tx:` body not lowercase hex | **REFUSE** exit 4 | `ClassUnknown` | R1 |
| `tx:` body hex but not a transaction | **REFUSE** exit 4 | `ClassUnknown` | R9 |
| `tx:` body parses, an input unsigned | **REFUSE** exit 4 | **nothing — no predicate exists** | L2 / G-P3.1 |
| damaged / corrected / elided `mt1` string | **REFUSE** exit 4 | `ClassUnknown` | R5, L3 |
| **incomplete** `mt1` set | **REPORT-AND-PACK** exit 0, naming the set and **every** missing string 1-based against the header's count (G-P3.7) | **ENGRAVE**, legend replaced by `INCOMPLETE - MISSING STRINGS - RE-ENCODE PAYLOAD` | **ruling 2026-08-25** |
| complete set that is **not a transaction** | **REPORT-AND-PACK** exit 0 | **ENGRAVE**, legend replaced by `INCOMPLETE - DOES NOT DECODE - RE-ENCODE PAYLOAD`; QR choice withheld | **ruling 2026-08-25b** |
| set id collides with another set's (R17) | **REPORT-AND-PACK** (`AmbiguousChunk` → unconfirmed) | as above | ruling 2026-08-25b |
| section > 32,734 B | **REFUSE** exit 4, naming the cap | `ParseHeader` refuses | R6 |
| world-readable stdout / `--out` | **REFUSE** exit 2 unless overridden | n/a | R8 |
| iterations out of range | **REFUSE** exit 2 (usage) | n/a | exit-code table, `main.rs:250-266` |
| no records given | **REFUSE** exit 2 | n/a | R7 (but see G-P3.2) |
| QR search returns no configuration | n/a | **REFUSE the QR path**, name the byte count and the 16-symbol cap, re-offer TEXT | R16 |
| >16 symbols | n/a | **REFUSE** (`errTooManySymbols`) | R13/N15 |
| complete-but-non-decoding set gathered over **NFC** | n/a | **DROPPED** with `Set complete but does not confirm as one transaction. Dropped.` | divergence from the payload path — G-P3.9 |
| `tx:` record arriving over NFC | n/a | `errScanUnknownFormat` — `tx:` is a container-only class | R12 |

**Exit-code vocabulary** (`main.rs:250-266`, pinned by
`cli::the_exit_code_vocabulary_is_one_vocabulary`): 2 = usage, 3 = policy
refusal, 4 = invalid input.

---

## 4. CLASSIFICATION — every spec requirement

Verdicts: **MET** · **MET-DIFF** (purpose satisfied another way, reason given) ·
**NOT-MET (Pn)** · **SUPERSEDED** (a 2026-08-25 ruling changed it).

### 4.1 The refusals (`SPEC` §5)

| # | verdict | evidence / reason |
| --- | --- | --- |
| R1 | MET | `sysw/mod.rs:152-167`; `an_uppercase_hex_body_is_still_a_non_hex_body` |
| R2 | **MET (P3a)** | `read_records` refuses any argv record whose trimmed, case-folded form begins `tx:`, at **exit 3**, before the passphrase ceremony and before any output; `a_tx_record_on_argv_is_refused`, `the_argv_refusal_echoes_neither_the_transaction_nor_a_passphrase` |
| R3 | **MET (P3b)** | `mt encode --record` is built. The form is a FLAG with no default: `record_form_guard` (`mt-cli/src/main.rs`) refuses `--record` with neither `--raw` nor `--chunks`, names both forms and says what each PRODUCES, and runs before the transaction is read. Registered in `mt`'s `refusals.toml` and proven non-vacuous by `scripts/mutate-refusals.sh`. Clap carries the other two halves structurally (`requires`/`conflicts_with`), so the guard is the remainder, not a re-check; `record_without_a_form_is_refused_and_the_refusal_teaches`. **The earlier verdict here was true of a world where `me tx` existed** |
| R4′ | MET-DIFF | there is no `form` byte and no combined record: the two forms are distinct record **classes** (`Class::Mt`, `Class::Tx`). "One record carrying both" is unrepresentable, so the comparison R4′ prescribes has nothing to compare |
| R5 | MET | strict admission both sides (L3); **measured:** an elided `mt1` string is refused at exit 4 |
| R6 | MET | `main.rs:1088-1093` names the 32,734-byte cap. *Minor:* "Split them across two payloads" is not actionable for one transaction |
| R7 | **MET (P3a; WIDENED P3b)** | `read_records` reads stdin when neither argv nor `--in` is given, and EMPTY or whitespace-only input joins the same exit-2 path. **P3b found the guard was on stdin ALONE**: `--in` an empty file exited **0** and wrote a 52-byte container holding nothing. Both channels now go through one `no_records_guard`, which names the file when there is one; `empty_stdin_is_the_exit_2_path_not_an_empty_container`, `an_empty_in_file_is_the_exit_2_path_too`, `the_empty_in_refusal_names_the_file` |
| R8 | MET | L13; `mt`'s half at `mnemonic-transaction@542b391` (§8.2h) |
| R9 | MET | `pack_refuses_a_tx_record_that_is_not_a_transaction`, `me_tx_refuses_non_transactions` |
| R10 | MET-DIFF | the txid is **derived**, never asserted, so R10's stated hazard (two records *asserting* one txid over different bytes) cannot arise from an assertion. Duplicate candidates merge on **bytes**, not on the txid (`gui/transaction.go:263-327`, `TestPayloadTransactionsConfirmsAndMerges`), so identical twins collapse safely and different ones stay two candidates. Residual: G-P3.10 |
| R11′ | MET | `txNothingToEngrave` (`gui/transaction.go:115-134`) — three messages, not two. Test gap: G-P3.11 |
| R12 | MET-DIFF | `tx:` was **never added** to `isSyswEncoded` (`gui/scan.go:107-110`), so C3's precondition does not exist. Consequence: `tx:` cannot travel NFC at all (G-P5.4) |
| R13 | MET | Structured Append exists (`txqr/txqr.go:74-141`) and decodes under ZXing, so the refusal's condition never arises; the 16-symbol bound is enforced twice |
| R14 | SUPERSEDED | retired by the spec itself; several transactions in one payload work (`payloadTransactions`) |
| R15 | MET-DIFF | **strictly stronger than specified.** Nothing is carried; the device *reassembles, parses and derives* the txid, then binds it to `chunk_set_id` (`mt/mt.go:218-224`). A refutation-only check on a carried value is replaced by a computation. Incomplete sets are flagged, not refused, per ruling |
| R16 | MET | `gui/transaction.go:721-723` names the byte count and the symbol cap and re-offers TEXT. *Does not name the module size* — G-P3.12 |
| R17 | SUPERSEDED | **ruling 2026-08-25b: nothing in the chunk path refuses.** The collision is still *detected* — one `chunk_set_id` group, `mt_codec` returns `AmbiguousChunk`, the set reports unconfirmed and packs |

### 4.2 The NORMATIVE statements (24)

| § | statement | verdict | evidence / reason |
| --- | --- | --- | --- |
| 2.1 | per-form record layout (metadata record + bare siblings) | MET-DIFF | bare `mt1` siblings are exactly as ruled. The `tx:` **metadata** record was dropped because nothing survives to carry: no txid (derived), no wtxid (retired), and legend fields ride as ordinary `text:` records |
| 2.1a | prefix without a branch is the defect | MET | the prefix was never added to the scanner (R12) |
| 2.1b | P1 defines the wire layout before anything reads it | MET-DIFF | no new layout exists; the layout is the existing record model — `TX_PREFIX` + lowercase hex, and bare `mt1` — stated at `sysw/record.rs:24-52`, ported at `sysw/record.go:21,39,43`, conformance-tested with the Rust answers |
| 2.2a | chunks path engraves TEXT ONLY; may not call `validateMdmk` | MET | `planTransactionTextPlates` (`gui/transaction.go:597-660`); `validateMdmk` is not called; QR is offered only for a confirmed candidate |
| 3.1a | P4 owns the `txScan` case in `engraveObjectFlow` | MET-DIFF | the carrier type is `mtText` (`gui/scan.go:134`) with its case at `gui/gui.go:2492-2495`. No `txScan` exists because `tx:` never travels NFC |
| 3.1a | all four lockstep sites | MET | `gui/gui.go:2056-2058`, `:2214-2215`, `:2442-2444`, `:2492-2495`. Test gap: G-P3.8 |
| 3.2 | the compare screen must name `me sysw show` | **MET (P3a)** | G-P3.16; `TestTheCompareScreenNamesTheReadPath` |
| 3.2 | the `me sysw pack` line must go | **MET (P3a)** | G-P3.16; the same test asserts its absence, and `pack` prints the pointer |
| 3.3 | menu gains content entries **and** boot invokes it | **MET (P3a)** | G-P3.15, both halves; `TestTheBootLoadEndsAtThePayloadMenu` drives the real `uiFlow` |
| 3.6b | extracting `chunk_set_id` without a decoder is P4's, and not zero | SUPERSEDED | ruling 2026-08-25b requires the device to compute confirmation itself, which needs the whole decoder — shipped as `mt/mt.go` + `codex32.ValidMT` |
| 3.6b | R15: 20 bits refutes, never confirms | MET-DIFF | see R15 |
| 3.6b | `chunk_set_id` is the BINDING mechanism | MET | grouping key in `sysw/mt.rs:97-121` and `sysw.MTUnconfirmed` |
| 3.6b | refuse an R17 collision at pack, naming both txids | SUPERSEDED | ruling 2026-08-25b |
| 4.1a | R16 | MET | see R16 |
| 4.2a | 16 symbols is a hard bound | MET | `txqr/txqr.go:38,50-52,106-111` |
| 4.2a | discard any configuration above 16 symbols | MET | plate search bounded at `txqr.MaxSymbols+1` (`gui/transaction.go:692-724`) |
| 4.2a | a phase must own Structured Append | MET | `txqr` implements it through the vendored library's exported `Encoding` seam — **no fork of kortschak-qr** |
| 4.2d | S0 cuts the SA pair; P5's gate has two halves | NOT-MET (P4) | fixture committed and pinned; **no gate consumes it**. Half (b)'s *decode* leg is met by ZXing; the module-for-module leg and the steel leg are not |
| 4.3a | the per-plate instruction is a function of what is ON THAT PLATE | **MET-DIFF (P3a)** | G-P3.17(a): `transactionLegend(…, plateHasQR)`. TEXT plates still carry no on-plate instruction, deliberately — a line trades against the brief's own priority (fewest plates), and their instruction is the post-cut screen |
| 4.3a | per plate it scans; per job `mt inspect` once; never on a partial set | **MET (P3a)** | G-P3.17(b): `transactionPostCutFlow`. The partial-set half is now live rather than vacuous — an unconfirmed set gets a different sentence |
| 4.4a | reorder the emission, and the gate asserts the order | MET | `backup/backup.go:493-517`; knot-order tests both at mechanism and artifact level |
| 4.5a | the reservation is computed PER PLATE from that plate's field set | NOT-MET (P4) | shipped legend is a fixed 3-line block plus a title row; no packed five-field reservation, no plate-1-vs-2..m split. The device has no wallet-level data source |
| 5/R4′ | the XOR is PER TRANSACTION | MET-DIFF | a payload may hold both forms; the device merges byte-identical twins into one candidate |
| 5/R11′ | two distinct messages | MET | `gui/transaction.go:115-134` |

### 4.3 The rulings not to be re-litigated (`SPEC` §8, 30 rows)

| ruling | verdict | note |
| --- | --- | --- |
| QR carries the standard form, never codex32 | MET | `txqr` encodes `tx.Raw` |
| the device comprehends before it cuts | MET | `mt.Decode`/`ParseTx` before the review screen |
| plate default is QR + legend; text optional | MET-DIFF | the operator **chooses**; a `tx:`-only candidate offers QR only, an unconfirmed set offers TEXT only. An explicit choice is not worse than a default |
| payload carries raw XOR chunks | MET-DIFF | per transaction; both may coexist and merge |
| `mt` emits the record, `me` packs | **MET (P3b)** | `mt encode --record --raw` emits the `tx:` record and `--chunks` emits the bare `mt1` records; `me` packs both and manufactures neither. **`me tx` is deleted** — `me` had no other verb that produced a constellation string, and the one it had disagreed with the consumer one step downstream (G-P3.19) |
| no new secrecy class | MET | L8 |
| `MaxSectionLen` → 32,734; NFC keeps 8191 | MET | L9 |
| the QR's byte encoding stays a parameter until the test plate | MET-DIFF | byte mode is now fixed and **proven decodable by an independent decoder**; the steel half is still S0's (G-P4.1) |
| the journey walk is the review of this spec | MET | `design/JOURNEY_WALK_engrave_transaction.md` |
| `me sysw pack` gains stdin | **MET (P3a)** | G-P3.4. `pack_reads_records_from_stdin_when_neither_argv_nor_in_is_given`; the sentence now names three channels (`the_no_records_message_names_stdin`). A TTY stdin prints what it is waiting for rather than looking like a hang |
| a `tx:` record on argv is refused | **MET (P3a)** | R2 / G-P3.5 |
| no `--record` default; the refusal teaches | **MET (P3b)** | R3 |
| chunks engraved verbatim — **no `mt1` decoder in v1** | SUPERSEDED | ruling 2026-08-25b requires the device to compute the confirmation, so the decoder was ported. Chunks are still engraved verbatim |
| world-readable output refused + override, `me` and `mt` | MET | L13 |
| sealing decided by content | **MET (P3a)** | `decide_sealing` (`main.rs`): seals iff some record is `Class::is_secret()`, and prints which way and why on **every** invocation. G-P3.6 |
| overwriting the region is intended — a courier | MET | `me sysw wipe`, `--region`, `{LOAD AGAIN, UNLOAD}` |
| the device names `me sysw show` under the digest | **MET (P3a)** | §3.2 above |
| the txid is for recognition, never claimed as proof | NOT-MET (P3, **G-P3.14 — not P3a's**) | the review screen shows the full txid with **no statement of its limit**; §3.5 requires the device carry the same caveat `mt verify` does. *P3a note:* the UNSIGNED review screen now does carry it — *"the txid above is the same one a signed version would have"* — so the shape exists; the confirmed screen still does not |
| show a total, allow skip | NOT-MET (P3, **G-P3.14 — not P3a's**) | no outputs, amounts or total exist: `mt.Tx` carries only `Raw`, `TxidDisplay`, `Inputs`, `Outputs`, `SegWit`, `EveryInputSigned`, `UnsignedInputs`. A parser change, not a screen change |
| the total is never spelled as a destination amount | MET-DIFF | satisfied by **absence** — there is no total to mislabel. It becomes live the moment the row above is built |
| the device says "test the plate"; it never tests it | **MET (P3a)** | the post-cut screen says TEST THEM NOW and says why the device cannot: it has no camera |
| `mt inspect` gains a raw-transaction subject | NOT-MET (P5) | landed on `mnemonic-transaction@df8d6d0`, branch `p1/mt-inspect-raw`, **not merged to `main`** (`git branch --contains` → that branch only) |
| the carousel is payload-independent | MET-DIFF | `engraveTransaction` is unconditional (`gui/gui.go:222`); applicability is expressed inside the program by `txNothingToEngrave` rather than by the payload menu |
| the payload menu appears right after a successful load | **MET (P3a)** | §3.3 above |
| the picker is keyed on the txid; the prefix never verifies | MET | keyed on `TxidDisplay`, full 64 hex on screen (`gui/transaction.go:483`). An **unconfirmed** candidate is keyed on the 5-hex `csid` — honest, since no txid exists |
| legend cut last; incomplete plates discarded; no resume | MET | L12; `gui/transaction.go:557-561` states a re-run starts at plate 1. It does not say *discard the plate* — G-P3.13 |
| text+QR is never offered for a transaction | MET | the choice is TEXT **or** QR; no combined variant exists |
| multi-symbol QR uses Structured Append | MET | L10, ZXing-proven |
| the legend is packed and its reservation computed; 3.0 mm is the tested floor | NOT-MET (P4) | the 3.0 mm floor is MET (`gui/transaction.go:38`); the packed/computed reservation is not (§4.5a) |
| symbol count outranks ECC; ECC floors at M | MET | L11 |

### 4.4 What must be true to close (`SPEC` §7, 11 conditions)

| condition | verdict |
| --- | --- |
| 0C/0I under an enumerated lens set | NOT-MET (P5) — the **shipped code** has had no whole-diff review; the spec's own R0 is GREEN |
| the mode-segmentation gate (`capgate`) is green | MET — Rust probe 8/8; fork `txqr` capgate 4 tests, 12/12 against the published v40 limits |
| the test plate is cut and read (S0) | NOT-MET (P4) |
| §4.2c's two Structured-Append gates | NOT-MET (P4 physics, P5 module-for-module); the *decode-our-own-rendering* leg is MET |
| the legend reservation is computed, not hard-coded | NOT-MET (P4) |
| the gate asserts the legend's EMISSION ORDER | MET |
| `check-provenance.sh` green across both repos | MET-DIFF — green in `mnemonic-transaction`; the fork carries in-tree package pins plus `TestVendoredVectorsMatchTheirProvenancePin`. No such script exists in `me` or the fork |
| refusal coverage is a bijection; every refusal test goes red without its check | NOT-MET (P5) — `refusals.toml` / `check-refusal-coverage.sh` / `mutate-refusals.sh` exist **only** in `mnemonic-transaction` |
| the carried txid and R15 implemented; the chunks picker uses the ASSERTED voice | SUPERSEDED — nothing is carried, so there is no asserted voice to render (L6) |
| all four lockstep sites, with the three silent ones asserted by test | **MET (P3a)** — G-P3.8; each of the four was deleted in turn and reddens 1, 1, 2 and 2 tests |
| both pipeline invariants asserted, each with a phase | NOT-MET (P5) — no test names *"`mt encode` writes nothing to stdout on a failure path"*; `decode`/`verify`/`inspect` have one (`decode_writes_nothing_to_stdout_on_failure`) |

### 4.5 Counts

**These are MEASURED, not hand-entered.** `scripts/acceptance-count.py` reads
the tables above and prints them; the first recount after P3a got three of the
five rows wrong before that script existed, which is what the standing rule
about never hand-counting is for. Output, pasted:

```
section                   MET  DIFF  NOT  SUP  rows
4.1 refusals               11     4    0    2    17
4.2 NORMATIVE              14     6    2    2    24
4.3 rulings                20     5    4    1    30
4.4 close conditions        3     1    6    1    11
spec items                 48    16   12    6    82

gates listed in §6: 36  (unique 36)
gates marked CLOSED: 18
gates still open:    18 -> G-P3.10 G-P3.14
                          G-P4.1..G-P4.6  G-P5.1..G-P5.10
```

**Re-measured after P3b** (the `me tx` → `mt encode --record` graft), by the
same script. Three verdicts moved and one gate closed: R3 and the two
`--record` rows became **MET** because the flag now exists to have no default,
and **G-P3.19 closed by construction**. Nothing was re-classified by reading.

> **A wording trap this recount walked into**, recorded because it costs a
> whole round to find twice: the script matches `MET-DIFF` **anywhere in a
> row**, so a row whose verdict is `MET` and whose *prose* says "the earlier
> MET-DIFF was true of…" counts as DIFF. It printed 10/5 for §4.1 after R3 had
> already been flipped. **The count is measured; the row is still prose, and
> prose can lie to the measurement.**

**Every remaining §4 NOT-MET is owned by P4 or P5, or by one of the two P3
gates held back for the operator's journey walk** (G-P3.10 the txid-collision
picker, G-P3.14 the review screen's missing outputs/fee/network). No NOT-MET is
unowned. **G-P3.19 was the third and is closed** — P3b.

**P3b closed G-P3.19.** **P3a closed 17 gates** — G-P3.3…G-P3.9, G-P3.11…G-P3.13, G-P3.15…G-P3.18,
G-P3.20 — and verified G-P3.1/G-P3.2, **completing the half of G-P3.1 that was
still open**. §7 records six defects none of the gates asked for.

*How the two halves join.* §6 lists **36** gates. **17** of them are the §4 NOT-METs
regrouped — several spec lines say one thing, so e.g. G-P3.16 discharges three
(§3.2's two NORMATIVE statements and the walk-I ruling) and G-P3.4 discharges
two (R7 and the stdin ruling); G-P4.5 and G-P5.2 are extra gates on items already
counted. The other **19** are defects the code walk found that no spec line
states.

**That second number stopped equalling the open count at P3b**, and the
sentence here used to lean on their agreeing. Through P3a the 19 walk-found
gates *were* exactly the 19 still open; G-P3.19 was walk-found and is now
closed, so it is **19 walk-found, 18 still open**, and the two are different
sets from here on. Closing 17 §4-derived gates plus G-P3.19 is the 18 CLOSED
above.

---

## 5. THE FUNDS-SAFETY BAR (`FORWARD_PLAN` §2), restated concretely

This feature engraves an **already-signed** transaction. It never signs, never
derives, never touches seed-class records. The failure class is narrow: **an
artifact that passes every check and is worthless — or dangerous — in steel.**

| # | check | status |
| --- | --- | --- |
| 1 | signature-presence predicate, per input | **MET, all four cells (P3a).** Host `tx:` (L2), host `mt1` (`sysw::mt::diagnose`), device `mt1` (`mt.Decode` → `ErrUnsignedInputs`), device `tx:` (`payloadTransactions`, the half G-P3.1 left open). Override `--allow-unsigned-inputs` names the failing inputs at pack, in `me sysw show`, on the device screen and on the plate |
| 2 | strict admission, no BCH correction ever | **MET** — L3, both sides agree by construction |
| 3 | semantic confirmation computed ON DEVICE | **MET** — L4/L5, conformance-tested with the Rust answers |
| 4 | independent decode proof for every QR class emitted | **MET in the suite** (ZXing, k∈{1,2,3,6}, reverse order). Off steel: G-P4.1 |
| 5 | cross-language seam tests | **MET** — `sysw_mt_payload.bin`, packed by the Rust binary, read by Go |
| 6 | txid truth at every surface | **PARTIAL** — full display-order txid on the review screen and the legend; the post-cut recompute path needs `mt inspect`'s raw subject (unmerged) and a phone scan (P4) |
| 7 | bearer posture | **MET** — raw hex never on argv for `mt encode` (§8.2f, before clap) and never accepted on argv by `me sysw pack` (R2, exit 3); review screen states *"BEARER: anyone holding the plates can broadcast it."*; no echo before a refusal |
| 8 | rulings conformance | **MET** — L7; both legends substituted, un-overridably; nothing in the chunk path refuses |
| 9 | never emit unvalidated geometry | **MET** — L11; 0.3 mm is never emitted |

**Deliberately NOT guarded, and why that is acceptable:**

- **Signature VALIDITY.** Needs prevout scripts and amounts an offline device
  cannot have and can never fetch. Garbage witness bytes of plausible structure
  pass. The signing wallet is the authority; our guarantee is *presence and
  structure*, and the review screen says so. Guarding it would make the device a
  verifier it cannot be — this is the line "reasonable effort" draws.
- **Witness tampering with intact structure, pre-pack.** The retired `wtxid`
  could not see this either. Post-pack, BCH on chunks and QR ECC cover transit.
- **Fee, amount, recipient sanity.** The wallet showed these before signing.
  *Note:* the device shows none of them either (§4.3) — so the operator's only
  identity check is the txid, which is why check 6 matters.
- **An adversary holding the machine, or forging plates.** Physical custody is
  assumed, as everywhere in this product.
- **Honest empty/empty inputs** (P2A anchor spends and similar exotica) —
  false-positives of check 1. `--allow-unsigned-inputs` is the override, and it
  names every failing input index at pack time and again in `me sysw show`
  (G-P3.3, closed P3a). It is host-only by construction: there is no flag byte
  in the wire format, so the DEVICE re-derives signedness for itself.
- **Regenerability of the payload.** Ruled 2026-08-25: report loudly, pack.
- **A second `tx:` record that is byte-different but shares a txid** (witness
  malleation). Both pass; the picker cannot tell them apart. G-P3.10.

---

## 6. THE GATES — every NOT-MET, with its owning phase and done-condition

**P3 — the UI walk and the journey** (read here as *the last software phase*:
host CLI and device screens both, because a journey starts at the host).

| id | gate | done when |
| --- | --- | --- |
| **G-P3.1 — CLOSED (P3a; HALF of it was still open when P3a began)** | the signature predicate has **no Go counterpart** | **VERIFIED and COMPLETED.** The chunk half was done (`mt.Decode` → `ErrUnsignedInputs`). The **`tx:` RECORD half was not**, and the gate names it explicitly: `sysw.Classify` requires only a structural parse, so an unsigned `tx:` record was `ClassTx`, reached `payloadTransactions`, and became a candidate **with no flag of any kind** — with the honest transaction's txid, because stripping signatures is exactly what the txid ignores. §5 check 1 was unmet for the class that carries the QR path. Now flagged (not refused — it arrives only via `--allow-unsigned-inputs`) with `legendUnsigned` and the failing input named on screen and on the plate |
| **G-P3.2 — CLOSED, verified (P3a)** | the predicate does not guard the **`mt1` chunk class** on either side | **VERIFIED.** Rust: `sysw::mt::diagnose` (the single implementation both `mt_unconfirmed` and `decode_confirmed` now ask — they were two copies of one walk, which is how the predicate came to be added to each separately). Go: `mt.Decode` → `ErrUnsignedInputs`. Mutation-tested on the collapsed Rust implementation: disabling it reddens 3 |
| **G-P3.3 — CLOSED (P3a)** | `--allow-unsigned-inputs` (`FORWARD_PLAN` §2.1) does not exist — **measured:** clap rejects it. Overdue from P0 | **DONE.** `TxSummary::unsigned_inputs` carries the indices and `every_input_signed` is now *defined* from it, so the two cannot drift. `sysw::Admission` is the seam (`classify_with`/`pack_with`); the refusal, the override warning and `me sysw show` all name the failing inputs by number. **Scoped:** it loosens nothing else, is silent on a fully-signed transaction, and deliberately does NOT reach the `mt1` chunk class — nothing there refuses, and the device recomputes confirmation itself, so a host flag could only make the two disagree. Mutation-tested twice (per-input → whole-transaction reddens 4; flag ignored in `classify` reddens 3) |
| **G-P3.4 — CLOSED (P3a)** | `me sysw pack` has **no stdin path**; the ruled pipeline `mt encode \| me sysw pack` cannot be typed | **DONE.** `main.rs:split_record_stream`/`read_records`; precedence `--in` > argv > stdin. Five tests in `tests/sysw_cli.rs`: stdin is read, blank lines skipped as with `--in`, empty and whitespace-only stdin exit 2, the message names stdin, argv still wins. Mutation-tested: removing the empty guard reddens 2 |
| **G-P3.5 — CLOSED (P3a)** | a `tx:` record on **argv** is not refused (R2) | **DONE.** Exit 3 (policy refusal — the record is well-formed, the channel is not), raised in `read_records` before anything else `pack` does. Five tests: refused, located by argv index, echoes neither the body nor a generated passphrase, still packs from `--in` and stdin (byte-identical containers), and is scoped to the `tx:` class alone. The exit-code table gained a row. Mutation-tested: `if false &&` reddens 4 |
| **G-P3.6 — CLOSED (P3a)** | sealing is **flag-decided**, not content-decided, and says nothing | **DONE.** `decide_sealing`; three verdicts, each naming its reason and the secret-carrying records BY CLASS (never by content — a `pass:` body is a passphrase). **A defect surfaced doing it, and it predates the gate:** `me sysw pack --passphrase-words 4 <md1>` minted a passphrase, told the operator to store it apart from the machine, and wrote `sealed: false, ct_len: 0` — `pack` encrypts only secret-class records, so the plaintext was empty, `sealed()` is `ct_len > 0`, and the 16-byte AEAD tag landed past `total_len()` unauthenticated. The flag is now reported IGNORED instead. Pinned by `what_pack_says_about_sealing_is_what_show_reads_back`, which makes a second program agree with the message rather than asserting its wording |
| **G-P3.7 — CLOSED (P3a)** | the incomplete-set warning does not name **the set** or **every missing index** — ruling 2026-08-25 makes "loudly" normative | **DONE.** `sysw::mt::set_problems` groups once and diagnoses per set; `SetProblem` distinguishes **five** failures whose remedies are not close (missing / does-not-reassemble / not-a-transaction / txid-does-not-bind / unsigned-inputs). `pack` prints `mt1 set 2dcf2 (records 0, 1, 2 …) did NOT confirm … MISSING strings 2, 4 and 5 of 6`, and `me sysw show` prints `mt set 2dcf2: INCOMPLETE — … MISSING strings 2, 4 and 5 of 6`. Chunk numbers are **1-based**, `mt`'s operator-facing convention. `mt_unconfirmed` is now defined from the same grouping, and `the_diagnosis_and_the_verdict_are_one_answer` asserts the two never disagree. Mutation-tested: `.take(1)` on the missing list reddens 3 |
| **G-P3.8 — CLOSED (P3a)** | no scanner-level test drives an `mt1` string through `scanner.Scan`; the three silently-failing lockstep sites are asserted only indirectly | **DONE.** `gui/scan_test.go` gains the `mt1` row; `gui/transaction_lockstep_test.go` adds five. `TestEveryNavigableProgramHasATitleAndAPlate` sweeps `0..lastNav()` rather than naming one program, so the next program added is covered. Mutation-tested by deleting each of the four sites: mtText case → 1 red, title case → 1, `layoutMainPlates` → 2, scanner `ValidMT` branch → 2 |
| **G-P3.9 — CLOSED (P3a)** | the **NFC gather** drops a complete-but-non-decoding set while the **payload** path engraves it — two behaviours for one condition, and the drop contradicts ruling 2026-08-25b | **DONE.** `substitutionFor` is the one function both paths ask; the gather offers the set instead of dropping it. The gather's decision moved out of the frame loop into `txGather.offer`, which is why no test could reach it before. A **third** substitution fell out: a set that reassembles, parses and BINDS and still cannot be broadcast is not "DOES NOT DECODE" — `mt.ErrUnsignedInputs` is exported and earns `legendUnsigned`. **Scoped to COMPLETE sets**, and the scoping is recorded as a finding: an INCOMPLETE set is not a divergence (more scanning can fix it), but an operator holding 3 of 6 tags still has no way to engrave the three from the gather, though the payload path offers exactly that — that needs a button inside a live scanning loop and an operator ruling on what Back then means |
| G-P3.10 | two byte-different transactions sharing a derived txid present as two identical picker rows | the picker distinguishes them (size, or a content digest) — or the case is ruled not-our-concern in writing |
| **G-P3.11 — CLOSED (P3a)** | R11′'s third branch ("payload not yet compared") is untested | **DONE.** `TestR11HasThreeDistinctMessages` plus `TestOrphanStringsAreASuffixNotAMessage`. **Recorded:** the branch is DEFENSIVE, not reachable from the load flow — `syswLoadFlow` nils an uncompared session — which is why it had no test: nobody could get to it to notice it was wrong |
| **G-P3.12 — CLOSED (P3a)** | R16's message does not name the **module size**, which §4.1a requires | **DONE.** *"20064 bytes is too large for QR plates. At 0.6mm modules — the smallest this machine cuts — 16 Structured Append symbols at ECC M hold at most 17968 bytes. Use TEXT plates."* The ceiling is **measured by search**, not written down. **Two findings:** (a) the first draft printed a ceiling of **0**, because `EncodeSet` refuses a payload it cannot split into 16 non-empty parts, so a bottom-up search never leaves the ground; (b) **R16 is UNREACHABLE through the container** — the QR ceiling is 17,968 B and the largest `tx:` record a section can carry is (32,734−3)/2 = 16,365 B, so `pack` refuses first. `TestTheQRCeilingIsAboveWhatTheContainerCanDeliver` asserts the relation so the day it inverts is a failing test |
| **G-P3.13 — CLOSED (P3a)** | the device never says to **discard** a plate abandoned mid-cut (§4.4) | **DONE.** The stop screen says DISCARD, says why (*"half cut and nothing will finish it"*), and says what keeping it costs: a re-run starts at plate 1, so the drawer ends up with two plates numbered n/m that are not the same, on a machine with no camera to tell them apart |
| G-P3.14 | the review screen shows **no outputs, amounts, locktime, nSequence, fee, network or total**, and states no limit on the txid | §3.4/§3.5's derived/asserted split is built, or the reduction is ruled and this sheet amended. **The `mt.Tx` struct carries none of these fields, so this is a parser change, not a screen change** |
| **G-P3.15 — CLOSED (P3a)** | the payload menu gains no content-derived entries and the boot path does not invoke it | **DONE**, both halves. The lead reads *"Loaded. It holds: 6 mt1 chunk, 1 free text."*, and one content-derived entry (ENGRAVE TRANSACTION) appears only when the payload holds a class `progTransaction` admits — asked through the admission table, not a second list. `uiFlow` calls `syswPayloadMenu` on a successful boot load; BACK exits, as §3.3 requires. `TestTheBootLoadEndsAtThePayloadMenu` drives the REAL `uiFlow` from power-on over a real region, because calling the menu directly is exactly what cannot tell the two halves apart |
| **G-P3.16 — CLOSED (P3a)** | the compare screen names `me sysw pack` (the re-pack path); `pack`'s digest line carries no pointer | **DONE.** The screen names `me sysw show <file>`; `pack` prints `re-print it with: me sysw show <path>` with the path filled in, and says *"the file you just wrote"* on stdout rather than inventing one. `the_named_command_prints_the_same_digest` RUNS the named command and compares — a pointer to a command that prints something else makes the operator read a mismatch as tampering |
| **G-P3.17 — CLOSED (P3a)** | no post-cut instruction screen; the engraved instruction is job-level even on a plate with no QR on it | **DONE.** `transactionLegend` takes `plateHasQR`: the legend-only plate says where the symbols are, an inline one says scan these, a single-symbol plate does not mention order. `transactionPostCutFlow` runs once after the last plate, names ONE command per plate kind (`mt inspect` / `mt verify`+`mt decode`), and for an unconfirmed set says the set did not confirm rather than sending the operator to check a txid it never produced. **It was a modal and the modal TRUNCATED it** — `ErrorScreen` does not page, so *"this machine has no camera"* was unreachable with three assertions on its wording passing; it pages now and a test pages it. **Scoped, recorded:** TEXT plates gain no on-plate instruction — a line trades against the brief's own priority (fewest plates) and the `mt1` hrp is self-describing. **Depends on G-P5.7:** `mt inspect`'s raw subject is unmerged, so the named command is not yet on `main` |
| **G-P3.18 — CLOSED (P3a)** | no cut-TIME estimate before commit, though the code claims the operator budgets by it | **DONE.** `transactionJobTime` sums `Plate.Duration` over `TicksPerSecond` — the same clock the live remaining-time readout uses, so two clocks cannot disagree in front of the operator — and says *"unknown"* at tps 0 rather than dividing by it on a confirm screen. The pinned vector reports *"about 30 min of cutting"* |
| **G-P3.19 — CLOSED (P3b)** | `me tx` emits a `tx:` record for an **unsigned** transaction at exit 0, and `pack` refuses the same bytes at exit 4 one step later | **DONE — BY CONSTRUCTION, which is why neither prescribed remedy was taken.** The walk offered two: make `me tx` apply the predicate, or make it warn and document the sequencing. Both keep two implementations of one question. The verb moved to `mt` instead, where the transaction vocabulary already lives, and `mt encode --record --raw` inherits **§8.3** — an input with neither scriptSig nor witness on any input is refused, per input, **before a record exists**. There is no exit-0-then-exit-4 disagreement left to sequence, because the producer cannot emit what the consumer refuses. **Measured end to end:** the 113-byte witness-stripped form of the pinned `even` vector → `mt` REFUSED §8.3, 0 bytes on stdout → `me sysw pack` "no records", nothing written. `the_raw_form_inherits_the_signature_guard` (+ its honest control), `the_chunks_form_inherits_the_signature_guard_too` |
| **G-P3.20 — CLOSED (P3a)** | **no end-to-end UI walk exists** for the transaction program (`runUITouch` is used in 39 other test files, not this one) | **DONE.** Five walks in `gui/transaction_walk_test.go` drive the real flow through real screens and finish the engrave through a real `EngraveScreen`: QR from a `tx:` record, TEXT from a confirmed set, the two legend-substitution paths, and the picker. Four **goldens** of the plates themselves (`tx-qr`, `tx-text`, `tx-unconfirmed`, `tx-unsigned-qr`) — mutation-proven to catch a lost warning at the artifact rather than at a string. **Journey:** `design/JOURNEY_engrave_transaction.md`, regenerated byte-identically by `scripts/gen-tx-journey.sh`; its device screens come from `TestCaptureTransactionJourney`, which is the walk instrumented, so document and test cannot drift. **Its one limit, stated in it:** the frames are the firmware's op tree, not the emulator's framebuffer — what the device SAYS, not how it LOOKS. That capture needs WASM + playwright and belongs with P4, beside a photograph of steel. **THREE DEFECTS THE WALK FOUND** are recorded in §7 |

**P4 — S0, the hardware session.** *This gate cannot be simulated and the
release does not ship before it.*

| id | gate | done when |
| --- | --- | --- |
| G-P4.1 | no engraved QR has ever been scanned | an engraved plate round-trips through ≥2 phone scanners to the correct txid, at 0.9 mm and 0.6 mm |
| G-P4.2 | the Structured-Append **physics** gate has never run | the committed `SA_FIXTURE` pair is cut and a real scanner reassembles it off steel |
| G-P4.3 | the legend reservation is hard-coded, and no face below 3.0 mm is tested | S0 answers the face; the reservation is then computed per plate from that plate's field set, with a test that plate 1 and a legend-only plate charge differently |
| G-P4.4 | 0.3 mm is optically unvalidated (and correctly never emitted) | S0 rules it in or out; if in, the encodeable ceiling moves out of the container's range |
| G-P4.5 | the byte ENCODING is proven only against a software decoder | a raw-octet symbol is read off steel by a phone app |
| G-P4.6 | the post-cut verify path has never been walked | one chunk string hand-typed into `mt` verifies; one scanned QR's bytes go through `mt inspect` |

**P5 — ship.**

| id | gate | done when |
| --- | --- | --- |
| G-P5.1 | the shipped code has had **no whole-diff independent review** | one opus review over the merged feature closes 0C/0I, its brief naming what §5's gates already machine-verified |
| G-P5.2 | P5's SA gate — reproduce the S0 fixture **module-for-module**, per symbol (the mask is per symbol) — does not exist | `txqr`'s output is compared byte-for-byte against `SA_FIXTURE` at pinned version/level/mask |
| G-P5.3 | `sysw_vectors.json` has **zero** `mt1`/`tx:` coverage on either side; the container's own conformance mechanism was not extended | the corpus carries at least one confirmed set, one incomplete set and one `tx:` record, regenerated Rust-primary-first |
| G-P5.4 | `tx:` records cannot travel NFC (`isSyswEncoded` excludes the prefix; `me`'s NDEF short-record cap is 249 B) — true and undocumented | stated in the README/help, or ruled a defect and fixed |
| G-P5.5 | refusal coverage is not a bijection: no `refusals.toml` equivalent exists in `me` or the fork | each refusal in §3 has a test that goes RED when its check is removed, driven by a committed table |
| G-P5.6 | no test names *"`mt encode` writes nothing to stdout on a failure path"* | that test exists in `mt-cli/tests/encode.rs` |
| G-P5.7 | `mt inspect`'s raw subject is unmerged (`p1/mt-inspect-raw`) | merged to `mnemonic-transaction@main`, and the device's post-cut instruction can name a command that exists |
| G-P5.8 | the ZXing round-trip `t.Skip`s when `ZXingReader` is absent, so fork CI proves nothing | CI installs the binary, or the skip fails the build on the CI runner |
| G-P5.9 | three doc comments still claim an unconfirmed `mt` record "counts as SECRET" — `sysw/mt.rs:21,282`, `sysw/record.rs:45`; mirrored at `sysw/confirm.go:33,150` — contradicting ruling 2026-08-25b, which the **operator-facing message** was already fixed to match (`9a0427a`) | the prose matches the code and the ruling |
| G-P5.10 | `mnemonic-engrave`'s CI does not exercise the fork's transaction packages (its submodule pin predates them); the fork's own CI does | stated, or the pin is advanced as part of the release |

---

---

## 7. WHAT P3a FOUND THAT NO GATE ASKED FOR

Six defects, none of them a gate row. Five were found by **executing** the
thing rather than reading it; one by writing a message that would have had to
be true.

| # | severity | what | where it came from |
| --- | --- | --- | --- |
| F1 | **Critical** | **The `tx:` record path was inert.** `payloadTransactions` built the candidate without setting `confirmed`, whose zero value is false — so a signed transaction got the *"UNCONFIRMED SET / Set 00000, 0 string(s) / QR plates are unavailable"* screen, and then `transactionReviewAndEngrave` found no TEXT (no strings) and no QR (`!confirmed`), so `len(choices)==0` and it **returned silently**. `me tx \| me sysw pack` → device produced nothing, with no screen. 16 transaction tests green throughout; not one drove a `tx:` record past `payloadTransactions` | G-P3.20's walk |
| F2 | **Critical** | **The program PANICKED** — `slice bounds out of range [:8] with length 0`. The picker built a row per candidate reading `c.tx.TxidDisplay[:8]`; an unconfirmed candidate has the zero-value `mt.Tx`. Rows are built for ALL candidates *before* `len(choices) > 1` decides whether to show the screen, so **one incomplete `mt1` set crashed the program with no picker ever displayed**. Live since the ruling-2026-08-25 fold; the triggering payload is the ordinary one that ruling exists FOR. This sheet recorded the row as MET | G-P3.20's walk |
| F3 | **Critical** (funds-safety) | **No signature predicate on the `tx:` class on the device** — G-P3.1's other half. See its row | verifying a gate marked closed |
| F4 | Important | **The BEARER warning was below the fold.** `confirmReviewScreen` pages, and the warning sat last, so page 1 held the question and the txid and nothing else: an operator pressing Continue from the screen showing the number they came to check never saw it. A **position**, not a wording — every assertion on the sentence passed | G-P3.20's walk |
| F5 | Important | **`--passphrase-words` on a payload with nothing secret printed a passphrase, told the operator to store it apart from the machine, and wrote `sealed: false, ct_len: 0`.** `pack` encrypts only secret-class records, so the plaintext was empty and the 16-byte AEAD tag landed past `total_len()` unauthenticated. The passphrase protected nothing and opened nothing. Predates G-P3.6; the gate's new message would have asserted the protection in words | writing G-P3.6's message |
| F6 | Important | **The post-cut screen was truncated by its own modal.** `ErrorScreen` does not page, so *"check the txid against…"* and *"this machine has no camera"* were unreachable — with three assertions on their wording passing. F-151's shape one step along: there, text submitted and not drawn; here, drawn and not shown | G-P3.20's walk |

**Two things this sheet asserts that are FALSE, measured:**

1. **§4.1 R10** — *"Duplicate candidates merge on **bytes**, not on the txid … so identical twins collapse safely and different ones stay two candidates"*, and **G-P3.10** — *"two byte-different transactions sharing a derived txid present as two identical picker rows"*. The merge reads `c.tx.TxidDisplay`. A byte-different transaction sharing a txid is **DROPPED, not duplicated**, and the pair that does it is not exotic — a transaction and its own signature-stripped form share a txid by construction. Pinned by `TestTheMergeIsKeyedOnTheTxidNotOnTheBytes` **without changing the behaviour**, so G-P3.10's operator ruling starts from what the code does. *(G-P3.10 is out of P3a's scope by instruction.)*
2. **R16 is unreachable through the container.** See G-P3.12.

**Two gates that skip silently, same class as G-P5.8.** `me`'s `cross_lang` and
`preview_cross_lang` — the constellation's Rust↔Go seam tests — `return` early
when `go` is not on `PATH`, and `go` is not on `PATH` on this box by default.
They also need `third_party/seedhammer`, which `git worktree add` does not
create. Both pass once present; neither runs otherwise.

**One thing for P4.** The pinned 222-byte vector plans to **ECC H at 0.6 mm**,
not 0.9 mm — L11 working as specified, since ECC outranks module size. But it
means the default plate for the constellation's own reference transaction uses
the smaller face, and 0.6 mm has never been read off steel. G-P4.1 should cut
*this* plate.

---

*Superseded by this sheet: `design/IMPLEMENTATION_PLAN_P1_me_container.md`.
Companions: `design/FORWARD_PLAN_post_experiment.md` (phases),
`design/SPEC_engrave_transaction.md` (requirements and their reasons),
`design/FOLLOWUPS.md` (rulings 2026-08-25 and 2026-08-25b),
`IMPLEMENTATION_LOG_P1.md`, `design/agent-reports/P2-recon-{gui,codec-qr}.md`
(the latter two in the `seedhammer` repo).*
