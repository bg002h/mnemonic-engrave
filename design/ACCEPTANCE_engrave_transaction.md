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
| `tx:` body not lowercase hex | **REFUSE** exit 4 | `ClassUnknown` | R1 |
| `tx:` body hex but not a transaction | **REFUSE** exit 4 | `ClassUnknown` | R9 |
| `tx:` body parses, an input unsigned | **REFUSE** exit 4 | **nothing — no predicate exists** | L2 / G-P3.1 |
| damaged / corrected / elided `mt1` string | **REFUSE** exit 4 | `ClassUnknown` | R5, L3 |
| **incomplete** `mt1` set | **REPORT-AND-PACK** exit 0 | **ENGRAVE**, legend replaced by `INCOMPLETE - MISSING STRINGS - RE-ENCODE PAYLOAD` | **ruling 2026-08-25** |
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
| R2 | NOT-MET (P3) | **measured:** `me sysw pack --no-passphrase "tx:<hex>"` packs at exit 0. `read_records` (`main.rs:1400-1416`) accepts argv unconditionally |
| R3 | MET-DIFF | `mt encode --record` was never built. The form is chosen by **which tool you run** — `mt encode` → `mt1` chunks → TEXT plates; `me tx` → `tx:` record → QR plates. There is no defaultable flag, so the refusal R3 exists to teach cannot arise |
| R4′ | MET-DIFF | there is no `form` byte and no combined record: the two forms are distinct record **classes** (`Class::Mt`, `Class::Tx`). "One record carrying both" is unrepresentable, so the comparison R4′ prescribes has nothing to compare |
| R5 | MET | strict admission both sides (L3); **measured:** an elided `mt1` string is refused at exit 4 |
| R6 | MET | `main.rs:1088-1093` names the 32,734-byte cap. *Minor:* "Split them across two payloads" is not actionable for one transaction |
| R7 | NOT-MET (P3) | subsumed by the missing stdin path — see §8-10 below |
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
| 3.2 | the compare screen must name `me sysw show` | NOT-MET (P3) | `gui/sysw_load.go:164-176` still says *"Compare this against what `me sysw pack` printed:"*; `me sysw show` is referenced nowhere in the fork |
| 3.2 | the `me sysw pack` line must go | NOT-MET (P3) | same site; and `pack`'s own digest line (`main.rs:1250`) carries no pointer either |
| 3.3 | menu gains content entries **and** boot invokes it | NOT-MET (P3) | `syswPayloadMenu` offers only `{LOAD AGAIN, UNLOAD}` (`gui/sysw_unload.go:42`); boot calls `syswLoadFlow` directly (`gui/gui.go:2019`) |
| 3.6b | extracting `chunk_set_id` without a decoder is P4's, and not zero | SUPERSEDED | ruling 2026-08-25b requires the device to compute confirmation itself, which needs the whole decoder — shipped as `mt/mt.go` + `codex32.ValidMT` |
| 3.6b | R15: 20 bits refutes, never confirms | MET-DIFF | see R15 |
| 3.6b | `chunk_set_id` is the BINDING mechanism | MET | grouping key in `sysw/mt.rs:97-121` and `sysw.MTUnconfirmed` |
| 3.6b | refuse an R17 collision at pack, naming both txids | SUPERSEDED | ruling 2026-08-25b |
| 4.1a | R16 | MET | see R16 |
| 4.2a | 16 symbols is a hard bound | MET | `txqr/txqr.go:38,50-52,106-111` |
| 4.2a | discard any configuration above 16 symbols | MET | plate search bounded at `txqr.MaxSymbols+1` (`gui/transaction.go:692-724`) |
| 4.2a | a phase must own Structured Append | MET | `txqr` implements it through the vendored library's exported `Encoding` seam — **no fork of kortschak-qr** |
| 4.2d | S0 cuts the SA pair; P5's gate has two halves | NOT-MET (P4) | fixture committed and pinned; **no gate consumes it**. Half (b)'s *decode* leg is met by ZXing; the module-for-module leg and the steel leg are not |
| 4.3a | the per-plate instruction is a function of what is ON THAT PLATE | NOT-MET (P3) | the engraved legend is **job-level** (`gui/transaction.go:665-677`); a legend-only plate still reads *"scan all qr, any order"*; TEXT plates carry no instruction at all |
| 4.3a | per plate it scans; per job `mt inspect` once; never on a partial set | NOT-MET (P3) | no post-cut screen exists; `mt inspect` is referenced nowhere in the fork. (The "never on a partial set" half holds vacuously) |
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
| `mt` emits the record, `me` packs | MET-DIFF | `mt encode` emits the `mt1` strings, which **are** the records; `me tx` emits the `tx:` record; `me` packs. `mt` was left untouched |
| no new secrecy class | MET | L8 |
| `MaxSectionLen` → 32,734; NFC keeps 8191 | MET | L9 |
| the QR's byte encoding stays a parameter until the test plate | MET-DIFF | byte mode is now fixed and **proven decodable by an independent decoder**; the steel half is still S0's (G-P4.1) |
| the journey walk is the review of this spec | MET | `design/JOURNEY_WALK_engrave_transaction.md` |
| `me sysw pack` gains stdin | NOT-MET (P3) | **measured:** `printf '' \| me sysw pack --no-passphrase` → exit 2, *"no records: pass them on argv or with --in"* — the exact sentence §1.1 ruled must change |
| a `tx:` record on argv is refused | NOT-MET (P3) | R2 |
| no `--record` default; the refusal teaches | MET-DIFF | R3 |
| chunks engraved verbatim — **no `mt1` decoder in v1** | SUPERSEDED | ruling 2026-08-25b requires the device to compute the confirmation, so the decoder was ported. Chunks are still engraved verbatim |
| world-readable output refused + override, `me` and `mt` | MET | L13 |
| sealing decided by content | NOT-MET (P3) | `main.rs:1021` `let sealing = !*no_passphrase;` — flag-decided, and no line states which way it went or why |
| overwriting the region is intended — a courier | MET | `me sysw wipe`, `--region`, `{LOAD AGAIN, UNLOAD}` |
| the device names `me sysw show` under the digest | NOT-MET (P3) | §3.2 above |
| the txid is for recognition, never claimed as proof | NOT-MET (P3) | the review screen shows the full txid with **no statement of its limit**; §3.5 requires the device carry the same caveat `mt verify` does |
| show a total, allow skip | NOT-MET (P3) | no outputs, amounts or total exist: `mt.Tx` (`mt/mt.go:122-129`) carries only `Raw`, `TxidDisplay`, `Inputs`, `Outputs`, `SegWit` |
| the total is never spelled as a destination amount | MET-DIFF | satisfied by **absence** — there is no total to mislabel. It becomes live the moment the row above is built |
| the device says "test the plate"; it never tests it | NOT-MET (P3) | no post-cut screen; the engraved legend carries a job-level *"scan, then broadcast"* |
| `mt inspect` gains a raw-transaction subject | NOT-MET (P5) | landed on `mnemonic-transaction@df8d6d0`, branch `p1/mt-inspect-raw`, **not merged to `main`** (`git branch --contains` → that branch only) |
| the carousel is payload-independent | MET-DIFF | `engraveTransaction` is unconditional (`gui/gui.go:222`); applicability is expressed inside the program by `txNothingToEngrave` rather than by the payload menu |
| the payload menu appears right after a successful load | NOT-MET (P3) | §3.3 above |
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
| all four lockstep sites, with the three silent ones asserted by test | NOT-MET (P3) — the four **sites** are all present (§4.2); the condition also demands the three silent ones be asserted by test, and they are not — G-P3.8 |
| both pipeline invariants asserted, each with a phase | NOT-MET (P5) — no test names *"`mt encode` writes nothing to stdout on a failure path"*; `decode`/`verify`/`inspect` have one (`decode_writes_nothing_to_stdout_on_failure`) |

### 4.5 Counts

| | MET | MET-DIFF | NOT-MET | SUPERSEDED | total |
| --- | --- | --- | --- | --- | --- |
| §4.1 refusals | 8 | 5 | 2 | 2 | **17** |
| §4.2 NORMATIVE | 10 | 5 | 7 | 2 | **24** |
| §4.3 rulings | 12 | 7 | 10 | 1 | **30** |
| §4.4 close conditions | 2 | 1 | 7 | 1 | **11** |
| **spec items** | **32** | **18** | **26** | **6** | **82** |
| §6 defects found by walking the code, not stated as spec lines | — | — | **19** | — | **19** |
| **TOTAL** | **32** | **18** | **45** | **6** | **101** |

**NOT-MET by owning phase: P3 = 28, P4 = 8, P5 = 9.** No NOT-MET is unowned.

*How the two halves join.* §6 lists **36** gates. **17** of them are the §4 NOT-METs
regrouped — several spec lines say one thing, so e.g. G-P3.16 discharges three
(§3.2's two NORMATIVE statements and the walk-I ruling) and G-P3.4 discharges
two (R7 and the stdin ruling); G-P4.5 and G-P5.2 are extra gates on items already
counted. The other **19** are defects the code walk found that no spec line
states, and they are the 19 counted above.

---

## 5. THE FUNDS-SAFETY BAR (`FORWARD_PLAN` §2), restated concretely

This feature engraves an **already-signed** transaction. It never signs, never
derives, never touches seed-class records. The failure class is narrow: **an
artifact that passes every check and is worthless — or dangerous — in steel.**

| # | check | status |
| --- | --- | --- |
| 1 | signature-presence predicate, per input | **HOST ONLY** — `tx:` class guarded (L2). **The `mt1` chunk class is guarded by neither side** (`set_confirmed` never reads `every_input_signed`), and the **device has no predicate at all**. G-P3.1/G-P3.2 |
| 2 | strict admission, no BCH correction ever | **MET** — L3, both sides agree by construction |
| 3 | semantic confirmation computed ON DEVICE | **MET** — L4/L5, conformance-tested with the Rust answers |
| 4 | independent decode proof for every QR class emitted | **MET in the suite** (ZXing, k∈{1,2,3,6}, reverse order). Off steel: G-P4.1 |
| 5 | cross-language seam tests | **MET** — `sysw_mt_payload.bin`, packed by the Rust binary, read by Go |
| 6 | txid truth at every surface | **PARTIAL** — full display-order txid on the review screen and the legend; the post-cut recompute path needs `mt inspect`'s raw subject (unmerged) and a phone scan (P4) |
| 7 | bearer posture | **MET** — raw hex never on argv for `me tx`/`mt encode`; review screen states *"BEARER: anyone holding the plates can broadcast it."*; no echo before a refusal |
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
  false-positives of check 1. The override that names them is not built (G-P3.3).
- **Regenerability of the payload.** Ruled 2026-08-25: report loudly, pack.
- **A second `tx:` record that is byte-different but shares a txid** (witness
  malleation). Both pass; the picker cannot tell them apart. G-P3.10.

---

## 6. THE GATES — every NOT-MET, with its owning phase and done-condition

**P3 — the UI walk and the journey** (read here as *the last software phase*:
host CLI and device screens both, because a journey starts at the host).

| id | gate | done when |
| --- | --- | --- |
| G-P3.1 | the signature predicate has **no Go counterpart** (`grep` for `every_input_signed` over the fork: 0 hits; `mt/mt.go:263` skips the scriptSig without inspecting it) | `mt.ParseTx` returns a per-input signedness flag; a `tx:` record or reassembled set with an unsigned input reaches the device flagged with the mandatory legend substitution; RED-tested with the 113-byte stripped vector |
| G-P3.2 | the predicate does not guard the **`mt1` chunk class** on either side — `sysw::mt::set_confirmed` (`sysw/mt.rs:124-138`) checks parse + binding only. (`mt encode` refuses unsigned input at §8.3 — but that is a different tool, not this container's admission boundary) | `set_confirmed` and `mt.Decode` both consult the predicate, or the sheet records a ruling that they deliberately do not |
| G-P3.3 | `--allow-unsigned-inputs` (`FORWARD_PLAN` §2.1) does not exist — **measured:** clap rejects it. Overdue from P0 | the flag exists, names the failing input indices, and has a test |
| G-P3.4 | `me sysw pack` has **no stdin path**; the ruled pipeline `mt encode \| me sysw pack` cannot be typed | stdin is read when neither argv nor `--in` is given; **empty** stdin joins the exit-2 path (R7); both tested |
| G-P3.5 | a `tx:` record on **argv** is not refused (R2) | argv carrying a `tx:` record exits non-zero **before** the record is echoed anywhere |
| G-P3.6 | sealing is **flag-decided**, not content-decided, and says nothing | `pack` seals iff some record is `Class::is_secret()`, and prints which way it went and why, every time |
| G-P3.7 | the incomplete-set warning does not name **the set** or **every missing index** — ruling 2026-08-25 makes "loudly" normative | the stderr line names the `chunk_set_id` and every missing index against the header's `count`; `me sysw show` does the same |
| G-P3.8 | no scanner-level test drives an `mt1` string through `scanner.Scan`; the three silently-failing lockstep sites are asserted only indirectly | `gui/scan_test.go`'s table gains an `mt1` row; a test drives `engraveObjectFlow`'s `mtText` case |
| G-P3.9 | the **NFC gather** drops a complete-but-non-decoding set (`gui/transaction.go:391-398`) while the **payload** path engraves it — two behaviours for one condition, and the drop contradicts ruling 2026-08-25b | both paths behave identically, or the divergence is ruled and recorded |
| G-P3.10 | two byte-different transactions sharing a derived txid present as two identical picker rows | the picker distinguishes them (size, or a content digest) — or the case is ruled not-our-concern in writing |
| G-P3.11 | R11′'s third branch ("payload not yet compared") is untested | the test table covers all three |
| G-P3.12 | R16's message does not name the **module size**, which §4.1a requires | it names module size, byte count and the ceiling at that module |
| G-P3.13 | the device never says to **discard** a plate abandoned mid-cut (§4.4) | the stop screen says so |
| G-P3.14 | the review screen shows **no outputs, amounts, locktime, nSequence, fee, network or total**, and states no limit on the txid | §3.4/§3.5's derived/asserted split is built, or the reduction is ruled and this sheet amended. **The `mt.Tx` struct carries none of these fields, so this is a parser change, not a screen change** |
| G-P3.15 | the payload menu gains no content-derived entries and the boot path does not invoke it | boot calls it on a successful load, and it lists what the payload holds |
| G-P3.16 | the compare screen names `me sysw pack` (the re-pack path); `pack`'s digest line carries no pointer | the screen names `me sysw show <file>` and the `pack` line is gone; `pack` prints the same pointer |
| G-P3.17 | no post-cut instruction screen; the engraved instruction is job-level even on a plate with no QR on it | the per-plate instruction is a function of that plate's contents; the per-job instruction names `mt inspect` once, after the last plate, and says order does not matter |
| G-P3.18 | no cut-TIME estimate before commit, though `gui/transaction.go:530-532` claims the operator budgets by it | the confirm screen states plate count **and** time, or the comment is corrected |
| G-P3.19 | `me tx` emits a `tx:` record for an **unsigned** transaction at exit 0, and `pack` refuses the same bytes at exit 4 one step later | ruled in the journey walk: either `me tx` applies the predicate, or it warns and the sequencing is documented |
| G-P3.20 | **no end-to-end UI walk exists** for the transaction program (`runUITouch` is used in 39 other test files, not this one) | the walk drives choice → review → plan-confirm → engrave loop for TEXT and QR, including the legend-substitution screens, with golden images and an emulator journey |

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

*Superseded by this sheet: `design/IMPLEMENTATION_PLAN_P1_me_container.md`.
Companions: `design/FORWARD_PLAN_post_experiment.md` (phases),
`design/SPEC_engrave_transaction.md` (requirements and their reasons),
`design/FOLLOWUPS.md` (rulings 2026-08-25 and 2026-08-25b),
`IMPLEMENTATION_LOG_P1.md`, `design/agent-reports/P2-recon-{gui,codec-qr}.md`
(the latter two in the `seedhammer` repo).*
