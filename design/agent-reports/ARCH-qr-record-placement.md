# ARCH ruling — where the `tx:` record is constructed (QR out of `m*-cli`)

**Date:** 2026-08-26. **Second ruling by the D5 architect agent; D5 is not
reopened.** Constraints fixed by the operator, verbatim: *"In and out are very
good for all m\*-cli binaries. But QR code stuff probably doesn't belong in
m\*-cli, including mt"* and *"Psbt handling belongs in mt… because the end
result is a string that can be used for engraving."* The spec has already
carved the deferral (§9a "DEFERRED to a tier-placement cycle"); this report is
the ruling that closes it. Written to its own file rather than appended to
`ARCH-toolkit-vs-shared-crate.md`: separate question, separate fold, separate
diff.

---

## 1. The ruling

**The `tx:` record is constructed by `me sysw pack`, and only there. `mt` stops
emitting it: `mt encode --qr` becomes `mt encode --raw-tx`, which emits the
transaction's canonical serialization as bare lowercase hex — no prefix, no
container vocabulary, no "QR" token anywhere in `m*-cli`.** The pipeline
becomes:

```
mt encode                    -> mt1 strings            (text plates / hand engraving)
mt encode --raw-tx           -> canonical signed tx hex (PSBT finalize+extract, or pass-through)
… | me sysw pack             -> pack classifies the hex, wraps it into the tx: record itself
```

The coordinator's proposed shape is adopted after testing (§7 has the
alternatives and why each loses). The deciding principle, which is also the
durable test for question 6: **an artifact is produced by the repo whose spec
defines its grammar.** `mt1` strings are `SPEC_mt_v0_1`'s grammar — `mt` writes
them. The `tx:` record class is `SPEC_systemwide_payloads` §3.3.1/§5.3.1's
grammar — that spec lives in `mnemonic-engrave`, so `me` writes it. Canonical
transaction bytes are Bitcoin's grammar — `mt` produces them from a PSBT
(finalize + extract is encoding work, per the operator's ruling) and `me`
independently validates them at admission (`sysw::tx::parse`,
`every_input_signed`, `--allow-unsigned-inputs` — all already shipped, measured
by the coordinator). Today's arrangement has `mt` restating `me`'s record
format in its own source — `encode_tx_record` at `mt-cli/src/main.rs:732`,
whose own doc comment admits it is `me`'s format *"re-stated in three lines
rather than shared"* with a pinned test *"so the two cannot drift silently"* —
which is the tier reach in one sentence: a cross-repo duplicate of a grammar
`me` owns, held together by a drift test. After the ruling there is exactly
**one writer per grammar** and the drift pin is deleted rather than maintained.

Details of the surface, both sides:

- **`mt encode --raw-tx`** inherits `--qr`'s exact slot: same position in the
  one-pipeline flow (the §8.3 empty-satisfaction guard and every other refusal
  run on the *transaction* before any form is rendered — verified in source:
  the form choice at `main.rs:603` happens after the guards), same
  `conflicts_with = "group-size"/"elide-prefix"` (restated reason: hex is
  machine input for `pack` and `bitcoin-cli`, grouping breaks both), same
  stderr `blocks::Form` split (rename `Form::RawRecord` → `Form::RawHex`; the
  correction-budget suppression logic is unchanged — BCH budgets still do not
  apply to a QR path). `--qr` is **not** kept as an alias, by `mt`'s own
  shipped philosophy from the very commit that created it (1282260: *"a stale
  script must fail loudly, not quietly mean something else"*).
- **`me sysw pack`** gains one recognizer arm in `classify_with`
  (`sysw/mod.rs`): a line that is pure lowercase hex AND `tx::parse`s AND
  passes the same `every_input_signed || adm.allow_unsigned_inputs` predicate
  as the existing `tx:` arm classifies `Class::Tx`; pack writes `TX_PREFIX` +
  body into the container at pack time. **The container bytes are unchanged**
  — the flash format, the device, and the fork are untouched. The recognizer
  is unambiguous against every existing class: prefixed classes are
  prefix-gated, `mt1`/`ms1`/`md1`/`mk1` carry HRPs no hex string has, a BIP-39
  line has words, and a hex line that fails `tx::parse` stays `Unknown` and is
  refused with its index, exactly as today. Uppercase hex stays refused
  (canonical-lowercase rule, EPD §6.6).
- **`me` keeps READING `tx:` records forever.** The prefix arm of
  `classify_with` stays: a record pasted back from a payload or transcribed
  from a plate must still classify in recovery flows. Only production moves.
- **`record.rs`'s "me READS this class and does not WRITE it" paragraph is
  rewritten**, not deleted — it becomes "me is this class's ONLY writer", and
  its original argument (*"a second encoder is a second thing to drift"*) now
  argues **for** the new arrangement instead of against it: the ruling reduces
  the format's writers from two to one.

## 2. Does the D5 answer change?

**No.** The foundation crate, its near-zero dependency rule, its crates.io
distribution, its five consumers and its host are all untouched — the `tx:`
record never was in the crate's scope. One boundary sentence is added to the
D5 report's negative scope, same class as the terminal-gate exclusion: **the
foundation crate must not grow record construction or container vocabulary**
— that is payload-tier grammar, and this ruling is precisely about keeping it
in its tier. (`mt`'s guard extraction in P1 is unaffected; the argv guard,
write gate and channel helpers carry no record knowledge.)

## 3. The blast radius, enumerated

**`mnemonic-transaction` (`mt`):**

- Flag rename + rendering: `EncodeArgs.qr` → `raw_tx`; `encode_tx_record`
  (main.rs:732) loses the prefix — becomes a plain lowercase-hex render — and
  its doc comment's cross-repo restatement disclaimer dies with it.
- Token sweep, measured: **6 src files** (`main.rs`, `blocks.rs`, `report.rs`,
  `validate.rs`, `input.rs`, `locktime.rs`) and **6 test files** carry
  `--qr`/`RawRecord`/`tx:` references; **25 `--qr` occurrences in tests**, 10
  `RawRecord` in src. `tests/tx_record.rs` (8 `tx:` references) — the
  cross-repo drift pin — is **replaced** by a pin of the bare-hex output
  against the corpus vector's own bytes, and stops naming `me`'s format.
- The pasted-`tx:` recogniser (24b8cef, cf17591 M-5) **stays** — an operator
  may hold a `tx:` record from a payload — but its wording changes from
  "your own output" to naming it a payload record and pointing at the hex.
- `Command::Encode`'s doc and the `--qr` long help (main.rs:49, 160-178):
  rewritten without QR/record vocabulary.

**`mnemonic-engrave` (`me`):**

- `classify_with`: the bare-hex arm (one match arm reusing the existing
  predicate). `pack`'s write path: prefix the classified hex. Both sit in the
  same admission code P0 already touches for `--expect`.
- **10 measured source sites** name `mt encode --qr`
  (`record.rs:34`; `main.rs:152, 169, 375, 1889, 1911, 1984, 1987, 2119,
  2193`) — six of them operator-facing message text from 516d5d0. All become
  `mt encode --raw-tx` with the pipe unchanged.
- The argv gate needs **zero change** and gets stronger for free:
  `is_argv_forbidden` keys on class, so bare transaction hex on `me`'s argv
  now classifies `Tx` and draws the bearer refusal with purge remedies,
  instead of today's less-specific `Unknown` refusal.

**The fork / device: zero changes.** The container's bytes are identical; the
device's QR path reads `Class::Tx` records from flash exactly as before.
`seal/session.go` is untouched.

**The spec (`SPEC_constellation_cli_uniformity.md`):** the §9a deferral block
(lines ~1105-1125) is replaced by this ruling; the two `--qr` reproductions
(§1 line 44, §6d line 440) are dated evidence and stay as history with the
flag's rename noted once; §6g's C-1 rationale is restated (§4 below); §10's
acceptance stage swaps `--qr` for `--raw-tx`; the §7 measurement-provenance
notes (lines 923-931) are history and stay.

**The journey:** `scripts/gen-tx-journey.sh` — **3 `--qr` call sites** — and
the 3 rendered sites in `design/JOURNEY_engrave_transaction.md` regenerate
from the driver. Owned by the same phase as `mt`'s rename (§5), so P4's
"journey regenerates" gate is never left unsatisfiable — the I-10 lesson,
applied in advance this time.

**Shipped P5 work that survives untouched:** `Class::is_bearer()` (`Mt` and
`Tx`) and the argv gate keyed on it; `me`'s admission validation
(`tx::parse`, signature predicate, `--allow-unsigned-inputs`); the device QR
path. The reversal moves one producer; it does not disturb one reader.

## 4. C-1's `transaction` binding survives, restated

**The union `Class::Mt | Class::Tx` survives verbatim — the mechanism is
untouched and only the rationale sentence is rewritten.** §6g should read:

> the kind `transaction` is satisfied by `Class::Mt` OR `Class::Tx`: a
> transaction reaches `pack` in two forms — bare `mt1` chunk records
> (`mt encode`, the text-plate path), classified `Mt`, and canonical
> transaction hex (`mt encode --raw-tx`, the QR path), which `pack` itself
> classifies `Tx` and wraps into the `tx:` record only `pack` writes. Bind the
> kind to a single class and one of the two paths takes a false refusal.

Everything else in §6g — kinds not counts, the incomplete-set refusal, opt-in
— is unaffected. The union's teeth are unchanged: a refusing producer leaves
neither form, and `--expect transaction` still catches it.

## 5. Phase impact (§7 P0-P4)

- **P0** — gains `me`'s half: the bare-hex recognizer arm and pack-side prefix
  write, landed with `--expect` (same admission code, same tests, one review).
  §6g rationale restated per §4. The foundation crate's negative scope gains
  the no-container-vocabulary line.
- **P1** — gains `mt`'s half: `--qr` → `--raw-tx`, the `Form` rename, the test
  replacement, and the journey driver's 3 call sites + regeneration. P1's gate
  ("diff enumerated, each edit justified by a named §6 ruling") absorbs this
  cleanly — each edit cites this ruling.
- **Between P0 and P1 the constellation is coherent by construction**: `me`
  accepts *both* the `tx:` record (which `mt` still emits) and bare hex; no
  flag-day, no broken intermediate, consistent with the spec's stated
  mixed-states policy.
- **P2, P3 — unaffected. P4 — unaffected in content**; its regeneration gate
  now exercises the new pipeline because P1 already moved the driver.
- No phase is added or deleted.

## 6. What the spec must RECORD, so it is not moved a third time

The verb's history, in one table, and the test that ends it:

| when | producer of the engravable transaction record | the local reason |
| --- | --- | --- |
| pre-2026-08-24 | `me` (a `tx:` metadata wrapper; `sysw` owned encode) | container work belongs to the container tool |
| 2026-08-25 (1282260) | `mt` (`--qr` emits the record directly) | "every constellation string comes from its own tool" |
| this ruling | **bytes: `mt`. record: `me`.** | an artifact is produced by the repo whose spec defines its grammar |

Both earlier positions were locally right and globally wrong, and the operator
named why: *"We had too narrow a view of the constellation when we started."*
The first position put PSBT vocabulary in the payload tier; the second put
container vocabulary in the encoding tier. The analogy that justified the
second move — every string from its own tool — was **correct and misapplied**:
`mt1` is an encoding-tier grammar and is `mt`'s; `tx:` is not a constellation
string at all but a **container record class**, defined in
`SPEC_systemwide_payloads`, so *its* own tool is `me`. The spec must state the
test in one line — **find the spec that defines the artifact's grammar; its
repo is the producer** — and note that the test also derives D5's boundary
(the foundation crate defines no grammar, so it produces no artifact) and the
display-grouping ruling (presentation of a string belongs to that string's
producer). A future proposal to move the record a third time must first say
which spec's grammar moved.

## 7. Alternatives tested and declined

- **A new verb (`mt extract`)** — breaks the cycle's own finding that the four
  verbs are already uniform across `md`/`mk`/`ms`/`mt` (§2); a fifth verb in
  one tool re-opens exactly the asymmetry this cycle exists to close;
  duplicates the guard/stderr wiring `encode` already has; and is wrong for
  raw-hex input, where extraction is a pass-through.
- **`me sysw pack` accepts hex; `mt` changes nothing** — leaves `--qr` and the
  record emission in `mt`, which the operator has now ruled against twice.
- **The record stays `mt`'s with a documented exception** — a documented tier
  violation is still a tier violation, and the drift pin it requires
  (`tests/tx_record.rs` naming `me`'s prefix) is permanent maintenance for a
  boundary the operator has rejected.
- **Keeping `--qr` as an alias for `--raw-tx`** — rejected by `mt`'s own
  shipped rule from 1282260: stale scripts must fail loudly.
- **`--hex` as the flag name** — ambiguous with the *input* being hex;
  `--raw-tx` names the artifact (`bitcoin-cli` vocabulary: `getrawtransaction`
  / `sendrawtransaction`), stays in the encoding tier's language, and pairs
  with `mt decode`'s existing "broadcastable hex" artifact — after the change
  `encode --raw-tx` and `decode` emit the same grammar from opposite inputs.

## 8. Facts verified during this ruling (2026-08-26)

| claim | how verified |
| --- | --- |
| `mt`'s record construction is 3 lines + a drift pin | `mt-cli/src/main.rs:732` `encode_tx_record`; its doc: "re-stated in three lines rather than shared"; `tests/tx_record.rs` pins the prefix |
| guards run before form rendering in `mt` | `main.rs:603` form chosen after §8.3/argv/stdout guards; "the pipeline is one pipeline" comment at 599 |
| `me` validates transactions itself | `sysw/mod.rs` `classify_with`: `tx::parse` + `every_input_signed \|\| adm.allow_unsigned_inputs` at the `tx:` arm |
| `me` currently writes no `tx:` record | `sysw/record.rs:34-41`: "me READS this class and does not WRITE it… There is no `encode_tx` here" |
| bare hex cannot collide with existing classes | `classify_with` read in full: prefix-gated classes, HRP-gated strings, word-gated mnemonic; unrecognized → `Unknown`, refused |
| blast counts | 6+6 `mt` files, 25 test `--qr` refs, 10 `RawRecord` refs; 10 `me` sites naming `mt encode --qr`; 3 driver + 3 journey sites; spec lines 44, 440, 694, 769-772, 923-931, 1107-1117 |
| C-1 binding is spec text, not shipped code | §6g (fold of 2026-08-26); P0 has not run — no `--expect` in `me-cli/src` |
| the spec already carries the deferral + history hook | §9a "DEFERRED to a tier-placement cycle" block, lines ~1105-1125 |
| `--qr` shipped today; alias philosophy | `mnemonic-transaction` 1282260 commit message |

— architect agent, second ruling, same dispatch.
