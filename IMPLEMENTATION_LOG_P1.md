# IMPLEMENTATION LOG — P1, the six grafts

**Written 2026-08-25 by the implementing agent.** Phase P1 of
`design/FORWARD_PLAN_post_experiment.md` §4: graft the six mechanisms from arm A
(`_experiment/A`, `exp/tx-plan-driven`) that survive the loss of A's `MTX1`
record framing, onto the arm-B base.

Branches: `p1/grafts` in `mnemonic-engrave` and `seedhammer`,
`p1/mt-inspect-raw` in `mnemonic-transaction`. Nothing pushed.

**Six landed. None skipped.** Two were larger than the brief expected, because
running the gate found a defect the brief did not know about (grafts 5 and 6).

---

## 0. Result

| # | graft | verdict | test |
| --- | --- | --- | --- |
| 1 | section cap 8191 → 32,734, both sides, cross-repo formula test | **landed** | `sysw::wire::tests::the_section_cap_is_raised_here_and_frozen_in_seal`, `sysw_cli::a_payload_past_the_old_8191_cap_packs_and_reads_back`, `sysw.TestTheSectionCapMatchesTheRustPrimary`, `sysw.TestASectionPastTheOldCapIsAccepted` |
| 2 | the exit-code vocabulary (2 / 3 / 4) | **landed** | `cli::the_exit_code_vocabulary_is_one_vocabulary` |
| 3 | R11′'s two messages | **landed** | `gui.TestNoPayloadAndNoTransactionAreDifferentMessages` |
| 4 | legend cut LAST, emission-order test | **landed** | `backup.TestTheTitleAndFooterAreEmittedLast`, `gui.TestTransactionPlateCutsItsTitleLast` |
| 5 | `capgate` over the base's QR capacity | **landed — and it found a live defect** | `txqr.TestTheLibraryCapacityTablesMatchThePublishedLimits`, `TestStructuredAppendCapacityCostsExactlyTheHeader`, `TestOneSymbolIsByteModeToo`, `TestTheQRDeliveryCeilingIsWhatWeThinkItIs` |
| 6 | `mt inspect` gains a raw-transaction subject | **landed** | `inspect::inspect_reads_a_raw_transaction_and_reports_its_txid` + 4 more |

Commits, in order:

```
mnemonic-engrave  p1/grafts
  5538f3b  lock: record the mt-codec git rev -- 9a0427a left Cargo.lock behind
  b82e28c  sysw: MaxSectionLen 8191 -> 32,734 -- the delivery ceiling, lifted (graft 1)
  9f4a72e  me: one exit-code vocabulary, named at every site and pinned by a table (graft 2)
  (this commit)  plan: P1 landed -- the delivery ceiling, measured; this file

seedhammer        p1/grafts
  a411dc6  sysw: MaxSectionLen 8191 -> 32,734, and a test that reads the primary's source
  5023ff8  gui: R11' -- "no payload" and "no transaction in the payload" are two screens
  28f4134  backup: the plate's claim about itself is cut LAST (graft 4)
  5fed302  txqr: the mode-segmentation gate, run against THIS encoder -- and what it found

mnemonic-transaction  p1/mt-inspect-raw
  df8d6d0  mt inspect: a RAW-TRANSACTION subject, because the post-cut step needs one
```

---

## 1. THE DELIVERY CEILING, MEASURED

Every figure below was produced by running the shipped code. None is derived on
paper except where the derivation is shown and its inputs are measured.

| what | before | after | how |
| --- | --- | --- | --- |
| container section (`MAX_SECTION_LEN`) | 8,191 B | **32,734 B** | `(65536 − 52 − 16) / 2`, const-asserted in Rust and Go |
| one raw `tx:` record | 4,094 B of transaction | **16,365 B** | `tx:` + 2N hex ≤ 32,734 |
| an `mt1` chunk set | 3,720 B (93 chunks) | **14,840 B (371 chunks)** | **measured** — see below |
| QR plates, device side | — | **37,264 B** at ECC M | **measured** by the new capgate |

The chunk-set figure, measured against the real binary rather than computed:

```
$ python3 -c 'rec = "text:" + "61"*41; print(len(rec))'      # 87, a full mt1 chunk string
87
$ me sysw pack --no-passphrase --in n371.txt --out n371.bin   # 371*87 + 370 = 32,647
exit=0
$ me sysw show n371.bin
sealed:   false
pub_len:  32647
$ me sysw pack --no-passphrase --in n372.txt                  # 372*87 + 371 = 32,735
exit=4
```

40 payload bytes per chunk is `payload_ceiling_bytes` in
`design/vectors/mt1_v1_vectors.json`; 87 characters per full chunk is measured
off that file's 222-byte vector. 371 × 40 = **14,840 bytes**. At the old cap,
`88n − 1 ≤ 8191` gives n ≤ 93 and 93 × 40 = 3,720 — which is the "~3.5 KB
delivery ceiling" the experiment write-up recorded.

The QR figures, from `txqr.TestTheQRDeliveryCeilingIsWhatWeThinkItIs`:

```
v40-L:  2951 bytes/symbol x 16 symbols =  47216 bytes
v40-M:  2329 bytes/symbol x 16 symbols =  37264 bytes
v40-Q:  1661 bytes/symbol x 16 symbols =  26576 bytes
v40-H:  1271 bytes/symbol x 16 symbols =  20336 bytes
```

**So the container binds at 32,734 bytes, not the QR path**, and the ~8 KB
pathological spend is deliverable in either form. In chunk form, at 8191, it was
deliverable in neither.

---

## 2. Per-graft record

### Graft 1 — the section cap, both sides

`sysw::wire::MAX_SECTION_LEN` and `sysw.MaxSectionLen` are now the formula
`(REGION_LEN − HEADER_LEN − TAG_LEN) / 2`, with the property they exist to
preserve — two maxed sections plus header plus tag fit the 64 KiB region — asserted
at COMPILE time in both languages. `seal`'s cap stays 8191 and stays frozen.

The cross-repo test reads the FORMULA out of the Rust source rather than
comparing two retyped literals, and additionally reads the primary's
`seal/wire.rs` to prove the raise did not land on the wrong constant.

**It does not skip itself into silence.** The constant, the region geometry and
the "32,768 would not fit" corollary are asserted unconditionally; only the
cross-repo half depends on the sibling checkout, and when absent it logs both
paths it tried. It resolves in both layouts this repo lives in (side-by-side, and
`third_party/seedhammer` inside `me`).

RED, watched:

```
assertion `left == right` failed: sysw's cap, raised from 8191
  left: 8191
 right: 32734

cap_test.go:41: this port says 8191; the primary's formula gives 32734
cap_test.go:86: a 20,000-byte section was refused: sysw: section too long: pub_len=20000 ct_len=0 cap=8191
```

Mutations:

| # | mutation | result |
| --- | --- | --- |
| M1 | Rust cap = `32_768` | **BUILD FAILS** — `evaluation panicked: assertion failed: HEADER_LEN + 2 * MAX_SECTION_LEN + TAG_LEN <= REGION_LEN` |
| M2 | Rust cap back to 8191 | both CLI ceiling tests RED |
| M3 | Go port retypes the literal instead of the formula | cross-repo test RED, both assertions |
| G1 | Go cap = `32768` | **BUILD FAILS** — `invalid array length RegionLen - (HeaderLen + 2 * MaxSectionLen + TagLen) (untyped int constant -68)` |

**A SECOND CAP ON THE GO SIDE: there is none.** Checked, because a device-side
ceiling the host does not know about reintroduces exactly the divergence the
raise exists to prevent. `MaxSectionLen` is consulted once, in `ParseHeader`;
`boundBlob` and both `Reader` implementations bound by `RegionLen` (65,536) and
by the header's declared total; `sysw` has no `MaxRecords` (`seal` does, and
`seal` is untouched). `gui/scan.go`'s `8*1024` buffer is the NFC path and carries
records, never containers.

### Graft 2 — the exit-code vocabulary

The constants `EXIT_OK/USAGE/REFUSED/INVALID` already existed in the base; the
whole tx-engraving path (`run_sysw`, `emit`) returned bare integers. All ten sites
are now named, and the vocabulary is documented once on the constants.

The table found **two real disagreements** — see §3, defects 3 and 4. Both were
invisible to any per-subcommand test, which is why the graft is a table and not a
set of scattered `.code()` calls.

**Not changed, deliberately:** the unsigned-input refusal stays at exit 4. It has
the shape of a policy refusal (§2.1 of the forward plan even schedules an
override flag for it), but that section rules exit 4 and the library models it as
inadmissible input. A ruled number is not a defect.

### Graft 3 — R11′'s two messages

Two situations with two different fixes had collapsed into one sentence, and
**the sentence was unreachable on the real machine**: it sat behind
`if hasReader { gather; return }`, and the SH2 has an NFC reader soldered to every
board, so `Features()` always reports `FeatureNFC`. The operator with no payload
was dropped into a scanner with no statement of why.

A third arm was added for a sentence that would otherwise be a lie: a session
loaded but not `compared` gets `(nil, 0)` from `payloadTransactions` because
`takeAll` refuses it, and "this payload holds no transaction" would then be a
claim about contents the session is not allowed to read.

`txClassName` lives in fork-native UI code, not in package `sysw`: the Rust
primary has no such function and adding one to the port would be the port
leading. These are words on a screen, not normative behaviour.

### Graft 4 — the legend cut last

`EngraveText` emitted the title as its FIRST yield. `Engraving` is
`iter.Seq[Command]`, executed in emission order, so a plate abandoned mid-cut
already carried `TX 2DCF2B97 1/2` and looked finished.

Both tests assert the ORDER of emitted operations, because **a finished plate
looks identical either way** — every bounds test, every golden and every
rendering in the tree passes under both orders. That the whole `backup` package
including its goldens stayed green is the evidence that the artifact is unchanged.

`gui.TestTransactionPlateCutsItsTitleLast` asserts the same property of the real
`Plate`, through `toPlate` and `engrave.PlanEngraving` — because "PlanEngraving is
a streaming transform, so order survives it" is an assumption worth a test rather
than a comment.

### Graft 5 — capgate

Arm A's own log, item 7: *"`capgate` was never run against my Go capacity
function… I assert it is monotonic and reachable; I did not assert it equals the
published limits."* So the mechanism existed and had never been pointed at the
thing that matters.

The existing Rust `capgate` was run first and passes 8/8 — but it gates the
`qrcode` crate inside a measurement probe. This base has **no `saCapacity` at
all**: its capacity computation is the vendored kortschak-qr version walk inside
`txqr`, and that is what a plate count depends on. The gate was ported there.

12/12 exact against the published ISO/IEC 18004 v40 limits, in all three modes at
all four EC levels. **No mismatch in the arithmetic — and one live defect in the
package**, see §3 defect 7.

### Graft 6 — `mt inspect` raw subject

The device's post-cut step is *"scan the QR, then run `mt inspect` on what you
get"*, and a scanner hands back the transaction's bytes. `mt inspect` could only
read `mt1` strings, so the machine was about to instruct a step no tool could
perform. It now routes on the literal `mt1` — safe by a bech32 property: the data
charset excludes `1`, `b`, `i` and `o`, so `1` occurs in an `mt1` string only as
the HRP separator, and a hex transaction contains no `m` or `t` at all.

It reuses `report::Report` verbatim. A second report would give the
pre-engraving view and the post-cut view two implementations of one thing, free
to disagree — which is exactly the disagreement the post-cut step exists to
detect. The SET rows are absent because there are no chunks.

---

## 3. What was found wrong — in the base, and in arm A

Numbered so they can be cited. All are fixed unless marked otherwise.

1. **`Cargo.lock` was stale and `--locked` could not build at all.** Commit
   `9a0427a` moved `mt-codec` from a path dependency to a git dependency pinned
   by rev and did not commit the lock update, so every `cargo … --locked`
   invocation died before compiling a line — including CI's. Found by running the
   baseline suite before touching anything. Fixed in its own commit, `5538f3b`.

2. **Two shipped test fixtures went vacuous the moment the cap rose.**
   `pack_never_emits_what_the_reader_would_refuse` and
   `pack_refuses_a_section_too_long_for_its_own_parser` both built a literal 30
   records = 24,179 bytes of section. That was past 8191 and is not past 32,734,
   so `pack` was right to accept it and both went red. Their counts are now
   DERIVED from `MAX_SECTION_LEN`, with an assertion that the fixture exceeds the
   cap — the only form that cannot go quietly vacuous the next time the constant
   moves.

3. **`me` with nothing piped in exited 4** saying *"not a bech32 string (no '1'
   separator / empty HRP)"* — describing input the operator never gave as
   malformed. `me tx` has always said *"no transaction hex given (pipe it in, or
   --in FILE)"* at exit 2 for the same situation. This is a new user's literal
   first action. Now exit 2, with the same shape of message.

4. **`me sysw pack --iterations 5` exited 4; `me seal --iterations 5` exits 2** —
   the same typo, on the same flag, with the same range, in the same sentence.
   A flag value out of range is usage: nothing has been read at the point it is
   caught. Fixed on the `sysw` side.

5. **R11′'s message was unreachable on the shipped hardware** (graft 3 above).
   Not merely collapsed — behind an `if hasReader` that is always false on a
   machine that always has a reader.

6. **`backup.EngraveText` cut the plate's claim about itself first** (graft 4).

7. **`txqr` documented an invariant half its own paths did not hold.** Its
   package comment says *"Byte mode always: the payload is raw transaction bytes,
   and a per-part mode choice would make symbol boundaries observable in the
   decoded text."* True of `k ≥ 2`. **`k = 1` went through `qr.Encode`, which
   picks numeric, then alphanumeric, then byte** — so one symbol's capacity was a
   fact about the payload's character distribution rather than its length.
   Measured: 3,000 `A` bytes fit one v40-M symbol whose byte capacity is 2,331;
   3,000 `7` bytes did too. Fixed: both paths now share one byte-mode version
   walk. **The output is unchanged for real transactions**, and that is measured,
   not argued: the pre-existing `TestSingleSymbolMatchesThePlainEncoder` compares
   the pinned 222-byte vector's symbol against `qr.Encode`'s byte for byte, and
   still passes.

8. **`mt`'s `input::sniff` hard-codes the verb `encode` in every refusal it
   builds.** Routed unchanged, `mt inspect` over a truncated hex string would have
   printed `mt encode: REFUSED` — naming a command the operator did not run, on
   the one screen a recoverer reads in a panic. **`mt verify --transaction` had
   the identical wart and is fixed with it**: repairing one half of a defect is
   how the other half survives review.

9. **`mt`'s `no_node_warning` claims strings that may not exist.** It says *"mt
   read this transaction from the strings"* and *"read from the engraving
   itself"*. On a raw subject there are no strings. It now takes a `ReadFrom`, and
   the strings path's wording is byte-for-byte what it was.

### Found and NOT changed — recorded so nobody has to re-derive them

10. **`go vet ./gui/` fails, pre-existing and untouched by this work:**
    `gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires
    go1.26 or later (file is go1.25)`. `go.mod` declares `go 1.25.10` and the
    installed toolchain is 1.26.3. `go test` is unaffected and green. Fixing it
    means bumping the language version in `go.mod`, which is a decision about the
    firmware's toolchain floor and not P1's to make.

11. **`me tx` does not apply the signature predicate.** It emits a `tx:` record
    for a witness-stripped transaction at exit 0, and `me sysw pack` then refuses
    the same bytes at exit 4. The gate is at pack, which is the right place — but
    the journey shows the operator a success and then a refusal one step later,
    for something the first step could have named. Worth a journey-walk decision
    in P3, not a unilateral change here.

12. **`--allow-unsigned-inputs` (forward plan §2.1) is not implemented.** The
    predicate refuses with no override. Out of P1's scope; flagged because §2.1
    lists the flag as part of the check.

13. **The `incomplete > 0` branch in `payloadTransactions` is very nearly dead.**
    It counts `ClassMt` records whose `mt.ParseHeader` fails, and classification
    already required `ValidMT`, so the case is close to unreachable. Folded into
    the R11′ inventory message as a suffix rather than deleted, because "close to"
    is not "is".

14. **Arm A's `capgate` had never been run against arm A's own capacity
    function** (its log, item 7). The mechanism is good; it was pointed at the
    wrong thing in both arms until now.

15. **Mutation C3 is undetectable by any capacity probe, and that is a fact worth
    keeping.** Setting `saHeader.Bits()` to 0 does not move the measured capacity
    by one byte, because `coding.Plan.Encode` catches the overflow afterwards
    (`cannot encode 23656 bits into 23648-bit code`). The encoder is a backstop,
    so the mutation is harmless — but invisible. What it changes is WHO refuses,
    and that is now what the gate asserts.

### Where a test was not good enough the first time

Recorded because it is the whole argument for mutation testing:

- **Graft 6, mutation I2** — dropping the verb rewrite was NOT caught. Both
  refusal tests used inputs refused by `decode_tx` and by the PSBT-parse arm, and
  both of those name their own verb, so nothing reached a refusal that `sniff`
  itself built. A third test now uses a hex string that lost a character and a
  hex-encoded PSBT, both of which `sniff` refuses. Re-run: RED.
- **Graft 5, mutation C3** — see item 15.
- Three of graft 6's tests **passed the moment they were written**. Two were
  vacuous (they asserted a verb that a different code path was already supplying)
  and were strengthened until the mutation caught them; the third is a deliberate
  guard that the strings path is unchanged, and is supposed to stay green.

---

## 4. Raw test output

### `mnemonic-engrave` (Rust)

```
$ cargo nextest run --locked
     Summary [  12.695s] 333 tests run: 333 passed, 1 skipped

$ cargo clippy --all-targets --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
```

Baseline before P1: 330 passed, 1 skipped.

### `mnemonic-transaction` (Rust)

```
$ cargo nextest run --locked
     Summary [   0.132s] 219 tests run: 219 passed, 0 skipped

$ cargo clippy --all-targets --locked
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.30s
```

Baseline before P1: 212 passed.

### `seedhammer` (Go)

Everything but `gui`, uncached — 52 packages with tests, 20 with none, 0 failures:

```
$ go list ./... | grep -v '/gui$' | xargs go test -count=1
ok  	seedhammer.com/backup	3.549s
ok  	seedhammer.com/engrave	0.919s
ok  	seedhammer.com/md	0.038s
ok  	seedhammer.com/mk	0.061s
ok  	seedhammer.com/mt	0.006s
ok  	seedhammer.com/seal	17.383s
ok  	seedhammer.com/sysw	0.036s
ok  	seedhammer.com/txqr	1.926s
…
```

The `gui` package, sharded 24 ways (it is ~93% of the suite and runs on one core
otherwise):

```
$ scripts/gui-shard-test.sh ./gui/ 24 20m
=== enumerating tests in ./gui/ ===
    932 top-level tests
    partition verified exhaustive: 932 == 932
=== running 24 shards in parallel (timeout 20m each) ===
  shard 0: ok    38 tests  ok  	seedhammer.com/gui	115.278s
  …
  shard 23: ok    37 tests  ok  	seedhammer.com/gui	0.488s
=== wall: 116s ===
RESULT: ok -- all 932 tests ran across 24 shards
```

**The log was read, not the exit code**: the partition is asserted exhaustive
before anything runs (932 == 932), every shard reports `ok`, and no shard wrote
to stderr. A truncated `go test` run exits 0, which is why the count and the
per-shard lines are what is checked.

The `txqr` capgate, in full:

```
$ go test -run 'TestTheLibraryCapacityTables|TestStructuredAppendCapacity|TestTheQRDeliveryCeiling|TestOneSymbolIsByteModeToo' -v ./txqr/
--- PASS: TestTheLibraryCapacityTablesMatchThePublishedLimits (0.90s)
--- PASS: TestStructuredAppendCapacityCostsExactlyTheHeader (0.46s)
=== RUN   TestTheQRDeliveryCeilingIsWhatWeThinkItIs
    v40-L:  2951 bytes/symbol x 16 symbols =  47216 bytes
    v40-M:  2329 bytes/symbol x 16 symbols =  37264 bytes
    v40-Q:  1661 bytes/symbol x 16 symbols =  26576 bytes
    v40-H:  1271 bytes/symbol x 16 symbols =  20336 bytes
--- PASS: TestTheQRDeliveryCeilingIsWhatWeThinkItIs (0.38s)
--- PASS: TestOneSymbolIsByteModeToo (0.01s)
```

The Rust `capgate`, run once as the brief asked, against the measurement probe it
was written for:

```
$ cargo run --bin capgate            # design/measurements/mt-size-probe
mode-segmentation gate -- select2.rs's cap() vs published v40 limits
  v40-L: alnum 4296/4296 OK  | byte 2953/2953 OK
  v40-M: alnum 3391/3391 OK  | byte 2331/2331 OK
  v40-Q: alnum 2420/2420 OK  | byte 1663/1663 OK
  v40-H: alnum 1852/1852 OK  | byte 1273/1273 OK

gate PASSES: cap() measures the mode it claims, at every EC level.
```

And the independent decode proof still holds after the `txqr` change —
`ZXingReader` (zxing-cpp) merges the set back to the transaction byte-identically
at k = 1, 2, 3 and 6, in reverse scan order:

```
--- PASS: TestZxingMergesTheSetBackToTheTransaction (0.03s)
```

---

## 5. What P1 did NOT do

- **Operator to/from labels.** Out of scope by the brief: they ride as ordinary
  `text:` records, documentation only, no format change. No format was invented.
- **Anything from A's `MTX1` framing.** No `wtxid`, no carried txid, no 75-byte
  header. The mechanisms were grafted; the container was not.
- **The UI walk (P3), the hardware session (P4), the acceptance sheet (P2).**
  Untouched, as scheduled.
