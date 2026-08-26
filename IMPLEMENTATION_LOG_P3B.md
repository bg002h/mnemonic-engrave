# IMPLEMENTATION LOG — P3b, the `me tx` → `mt encode --record` graft

**Executed 2026-08-25.** One capability moved between two tools: the CLI surface
came from a parallel implementation, the wire format did not.

| repo | branch | commits |
| --- | --- | --- |
| `mnemonic-transaction` | `p3b/mt-record` (from `p1/mt-inspect-raw`) | `0d3a361`, `5fcfa4d` |
| `mnemonic-engrave` | `p3b/drop-me-tx` (from `p3/ui-walk`) | `d328237`, this file |

Not pushed. Neither live checkout was written to; the fork was read once, for a
`git status`, and not at all by the journey run.

---

## 1. What landed

### `mnemonic-transaction` — the producer

`mt encode` gains `--record`, `--raw` and `--chunks`.

- **`--record --raw`** emits `tx:` + the transaction's canonical serialization
  in lowercase hex, on one line. **Concatenation, and that is the entire wire
  format** — it is `me`'s `sysw::record::TX_PREFIX` + `hex_lower`, restated in
  three lines because the repos do not depend on each other, and pinned by one
  string equality so the two cannot drift silently.
- **`--record --chunks`** emits **exactly what `mt encode` already emitted,
  byte for byte.** That is a finding, not a design choice — see §3.
- **`--record` with neither form** is refused, and the refusal teaches: it names
  both forms and what each *produces*, and it runs before the transaction is
  read, so a refusal costs no work. That is spec R3.
- **`--record` conflicts with `--group-size` and `--elide-prefix`**, structurally
  via clap. Added here and *not* present in the parallel arm: a record is
  engraved verbatim and EPD §6.4 requires the canonical unbroken string, so a
  grouped record is one `me sysw pack` cannot classify — the operator would meet
  "record 3 unrecognised", naming the wrong tool. Silently ignoring a flag they
  typed is the worse alternative.
- **R3 is registered** in `crates/mt-cli/tests/refusals.toml` and in
  `scripts/check-refusal-coverage.sh`'s seeded set — the only entry whose rule is
  ruled outside `SPEC_mt_v0_1`, so it is cited **by document**:
  `SPEC_engrave §2.2`, not a bare `§2.2`, which resolves to a real and entirely
  unrelated section of `mt`'s own spec. `./scripts/mutate-refusals.sh` reports
  all 33 refusal tests go red when their check is removed.
- **`blocks::Form`** — the second commit; see §4.

### `mnemonic-engrave` — the consumer

- **`me tx` is deleted**: `Command::Tx`, `run_tx_cli`, its two exit-code table
  rows, its two `sysw_cli` tests, and `record::encode_tx` with it.
- **The parser and the signature predicate stay.** `sysw::classify` still
  decodes the body, still parses it as a transaction and still requires every
  input signed; `pack` still refuses a `tx:` record on argv at exit 3. Admission
  is not delegated to another binary. Only the manufacturing moved.
- **Every pipeline mention** now reads `mt encode --record --raw | me sysw pack`
  — the `sysw pack` help, the argv refusal's suggested private channel, the
  `NotATransaction` remedy, `record.rs`'s class doc, and the journey.
- **`design/ACCEPTANCE_engrave_transaction.md`**: R3 → MET, "no `--record`
  default" → MET, "`mt` emits the record, `me` packs" → MET, R7 widened,
  **G-P3.19 CLOSED**, bearer-posture row no longer cites a deleted verb, counts
  re-measured by `scripts/acceptance-count.py`.
- **`design/FOLLOWUPS.md`**: F-246 filed (the NFC-fit line — see §6).

---

## 2. What was taken from arm A, and what was deliberately left

Arm A is `_experiment/A/mnemonic-transaction`, branch `exp/tx-plan-driven`,
commit `fc7072a`. Read-only throughout.

**Taken.**

- **The clap surface**, verbatim in shape: `#[arg(long, requires = "record")]` on
  `raw` and `#[arg(long, requires = "record", conflicts_with = "raw")]` on
  `chunks`. This is genuinely better than a fresh attempt — "a form needs
  `--record`" and "not both" become *structural*, so `record_form_guard` handles
  only the case clap cannot express, and is the remainder rather than a re-check.
- **The refusal's teaching text**, which is R3's own wording from the spec.
- **The test scaffolding** from its 404-line `tests/tx_record.rs`: the `mt()`,
  `corpus()`, `even()`, `tmp_with()` and per-form helpers; the R3 refusal tests;
  and the four-route "a failing run contributes nothing to stdout" test, which
  names the invariant the whole pipeline rests on (`fish` reports a pipeline's
  status as the last command's).

**Deliberately left.**

- **All 224 lines of `crates/mt-cli/src/tx_record.rs`.** It is an **`MTX1`
  framing encoder** — magic, version, form byte, txid, **wtxid**, flags — and
  that format is **retired**. Grafting it would reintroduce the wire format this
  project replaced, including the `wtxid` field the signature predicate
  superseded. The acceptance sheet already records why nothing survives for a
  frame to carry: the txid is derivable from the bytes by anyone holding them,
  and the metadata record was dropped in P1.
- **Its cross-implementation oracle test**, which reproduces
  `tx_record_vectors.json` byte for byte. A fine idea; the file it judges against
  pins the retired framing, so the gate would have been three implementations
  agreeing on the wrong thing. (`me`'s side is pinned instead by
  `the_record_mt_emits_packs_on_stdin` constructing `tx:` + hex and packing it,
  against `mt`'s `the_raw_form_is_the_prefix_and_the_transaction_hex_and_nothing_else`
  asserting the identical string from the other side, with no shared code.)
- **Its NFC-fit reporting.** Out of scope by ruling — see §6.
- **Its `mt inspect` half.** Already grafted in P1 (`df8d6d0`); untouched.
- **Its `--from` fingerprint parsing and legend slots**, which exist only to fill
  frame fields that no longer exist.

---

## 3. What the empirical check found

**The pipeline, run end to end**, logs read rather than exit codes:

```
honest 222-byte vector (mt1_v1.json, "even")
  mt encode --record --raw | me sysw pack --no-passphrase --out p.bin   -> exit 0
  me sysw show p.bin
    public record 0: raw signed transaction — txid 2dcf2b97…f630, 222 bytes

113-byte witness-stripped form (same txid, no signatures)
  mt encode --record --raw ...                     -> REFUSED §8.3,
                                                      "1 of 1 inputs carry no
                                                       signature (input 0)",
                                                      0 bytes on stdout
  | me sysw pack ...                               -> "no records", nothing written

mt encode --record --chunks | me sysw pack         -> exit 0
  me sysw show: 6 × "mt1 chunk — confirmed", "mt set 2dcf2: … 6 string(s)"
```

**The §8.3 guard fires on the new path**, and that is what the move buys. It is
one call site reached by both forms, not two.

**`--record --chunks` is exactly today's output.** Checked rather than assumed:
the acceptance sheet's row 2.1 records that the `tx:` **metadata record** an
earlier draft placed beside the chunks was dropped in P1 — "no txid (derived),
no wtxid (retired), and legend fields ride as ordinary `text:` records" — so a
chunk set rides the container as **bare** `mt1` records, the same route
`md1`/`mk1` already take. Arm A emits a metadata record there. **Wrapping the
strings would have invented a container the consumer does not parse.** The test
asserts byte equality with `mt encode` and says so in its own words.

---

## 4. Defects found on the way

### D1 — `--record --raw` described the artifact it did not make *(mine; fixed, `5fcfa4d`)*

The first version passed every test I had written and printed, on stderr, beside
a record bound for **one QR plate**:

> *"mt corrects up to 4 wrong CHARACTERS per string … strings 1-6 are 87"* ·
> *"Type the strings back from the steel and run `mt verify < typed-from-steel.txt`"* ·
> `CUT 6 strings, 522 characters` · `PREFIX all 6 strings begin mt1p9h8jqq9` ·
> `FORMAT: mt1 codex32` · *"on EACH plate, its number: 1/6, 2/6, … 6/6"* ·
> *"These strings ARE the engraving"* · *"the strings just left this terminal"*

Every one is an instruction for a **21-minute-per-plate, irreversible** cut, and
every one names a different artifact than the one on stdout. You cannot type a QR
back; `mt verify` reads `mt1` and would refuse what the scanner hands you; and
`mt` does not know how many plates a raw record becomes — the **device** searches
for a layout — so `1/6` was a plate count invented for permanent steel.

**Nothing was wrong in any section, and no unit test could see it**: the strings
really are built either way, and nothing asserted a relationship between what
stdout carried and what stderr claimed. **It was visible only in the regenerated
journey transcript** — which is the whole argument for the house rule about
walking a journey rather than re-reading sections.

Fixed with `blocks::Form`, asked by exactly the blocks that make a claim about
the artifact. `verify_the_steel` now says SCAN + `mt inspect --in scanned.hex` on
the raw form — the raw subject P1 added for this step, and the command the
device's own post-cut screen names. **Every assertion is paired with its control
on the same fixture**: the chunks form must still say all seven things, because
it does cut six strings. A one-sided test passes an implementation that prints
both.

### D2 — R7 was on stdin only; `--in` packed an empty container at exit 0 *(pre-existing; fixed)*

```
$ me sysw pack --no-passphrase --out p.bin --in <empty file>
exit=0    -rw------- 52 p.bin        digest: none — this payload has no public section
$ printf '' | me sysw pack --no-passphrase --out p.bin
exit=2    me: no records: …
```

R7's own stated reason applies verbatim to `--in`: a failed upstream leaves a
**0-byte file** as readily as an empty pipe. **I hit the exact sequence within a
minute of the first end-to-end run** — `mt encode --record --raw > rec.txt` is
refused by §8.2h, because `>` creates 0644 under the usual umask, and `rec.txt`
is left empty. The empty container is the worse outcome: it flashes, it boots,
and the device offers nothing — P3's F1 reached from the host side. Both channels
now go through one `no_records_guard` that names the file. Mutating it green
kills **four** tests, including the two pre-existing stdin ones.

### D3 — the acceptance counter reads prose *(fixed in the sheet)*

`scripts/acceptance-count.py` matches `MET-DIFF` **anywhere in a row**. A row
whose verdict is `MET` and whose prose said *"the earlier MET-DIFF was true of…"*
counted as DIFF, and §4.1 printed 10/5 after R3 had already been flipped. The
count is measured; the row is still prose, and prose can lie to the measurement.
Recorded in the sheet so the next editor does not spend a round on it.

### D4 — a sentence the change falsified without touching *(fixed)*

§4.5 said the 19 walk-found gates *"are the 19 counted above"* — true only while
"walk-found" and "still open" happened to coincide. G-P3.19 was walk-found and is
now closed, so it is **19 walk-found, 18 open**, and they are different sets from
here on.

### D5 — the journey generator could not run at all here *(worked around, loudly)*

`scripts/gen-tx-journey.sh` checked for `go` up front, and **there is no Go
toolchain in this environment.** That is not a property of the document: only
Part 2 needs Go. The check moved to the step that needs it, and a `REUSE_FRAMES=1`
escape reuses the committed `frames.md`. It **fails closed** (the file must
exist), is never the default, and prints a banner.

> **Stated plainly, because it is a gate that did not run.** Part 2 of the
> journey was **not recaptured** in this cycle, and the fork was not consulted at
> all. Part 2 is trustworthy only while the firmware screens are unchanged.
> Re-run `scripts/gen-tx-journey.sh` without the variable, with `go` on PATH,
> before relying on it. Noted separately and untouched: `third_party/seedhammer`
> pins `713aee2e` while the live fork sits at `422acba`.

### D6 — the journey gained the divergence that found D2

The regenerated Part 1 now shows the **obvious two-step form being refused**
before the pipe that works, because that is what an operator reaches for first
and it is where D2 lives.

---

## 5. Verification

```
mnemonic-transaction
  cargo nextest run --locked             233 passed, 0 skipped   (219 before)
  cargo clippy --all-targets --locked    clean
  scripts/check-refusal-coverage.sh      33 tests over 19 ruled refusals
  scripts/mutate-refusals.sh             all 33 red without their check

mnemonic-engrave
  cargo nextest run --locked             367 passed, 1 skipped   (365 before)
  cargo clippy --all-targets --locked    0 warnings
  scripts/acceptance-count.py            pasted into §4.5 verbatim
```

**14 mutants, all killed**, and where the mutated line's execution was in doubt
it was proved to run with an `eprintln!`:

| # | mutation | tests that went red |
| --- | --- | --- |
| M1 | `finalized_guard_raw` neutered | 2 (raw + chunks inherit §8.3) |
| M2 | `record_form_guard` neutered | 2 (R3, and R3-runs-first) |
| M3 | a frame prepended to the record | 1 |
| M4 | clap `requires = "record"` dropped | 1 |
| M5 | the `--group-size` conflict dropped | 1 |
| M6 | `Form` pinned to `Strings` | 2 |
| M7 | `verify_the_steel` ignores the form | 2 |
| M8 | legend `FORMAT` always `mt1 codex32` | 2 |
| M9 | `correction_coverage` ungated | 1 |
| M10 | `CUT`/`PREFIX` ungated | 1 |
| M11 | plate numbering ignores the form | 2 |
| M12–M14 | the three noun substitutions reverted | 1 each |
| — | `no_records_guard` neutered (`me`) | 4, incl. both pre-existing stdin tests |

---

## 6. Out of scope, and left that way

**The NFC-fit line.** `SPEC_engrave_transaction.md` §2.3 item 2 (line 450) and
the §6 P2 scope row both say `mt encode --record` must state whether its record
fits an **NFC tag** — `gui/scan.go`'s 8 KB buffer, 8191, and explicitly not
`MaxSectionLen`. **The operator has not ruled on it, so it is not built.** Filed
as **F-246** with the reference implementation named (arm A built it) and a
warning not to graft that version blind: its line measured its own `MTX1`
framing, whose length is not this record's length. `mt` has no `--out`, so it
cannot know the record's destination, and a "fits an NFC tag" line is noise on
every run of the commoner journey — which is what the ruling has to settle.

The raw form does print `RECORD    one tx: record, N characters — for QR plates`.
That is the artifact's **length and nothing more**, in the column `CUT` occupies
for the other form. It is deliberately **not** a fit/does-not-fit verdict against
any ceiling, and the code says so where it is written.

**Also untouched**, as scheduled: `mt inspect` (P1), the fork's Go side (P3),
the device (P4–P5), and G-P3.10 / G-P3.14, which remain open on the operator's
journey walk.
