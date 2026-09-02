# SPEC — SeedHammer II `sysw` consumption, including transactions

**Status:** DRAFT, pre-R0. Written 2026-08-27 against the working trees at
`mnemonic-engrave` `master` = `25102c5` and `seedhammer` `main` = `0b656d7`.

**Every fact in §1 was produced by running a command, and the command is beside
the fact.** Nothing in this document is taken from an older design doc. Where a
committed document says otherwise, the disagreement is named at the point it
occurs.

---

## 0. The finding that reframes this cycle

**The capability this spec was commissioned to specify is already shipped.**

The brief was *"teach the SeedHammer II to consume `sysw` packages and engrave
from them — including transactions."* Measured against the current fork:

- the device has a complete `sysw` reader (`seedhammer/sysw/`, 1,827 lines);
- it is **wired**, not a library nobody calls — **18 non-test files** import it,
  including `gui/sysw_load.go`, `gui/sysw_session.go`, `gui/sysw_admit.go` and
  `gui/transaction.go`;
- `gui/transaction.go` is **1,464 lines** implementing gather → review →
  plate-planning → engrave → post-cut for **both** transaction forms;
- the host producer `me sysw pack` is shipped in `me 0.7.0` and its help text
  already reads *"the flash image a SeedHammer II engraves a transaction or a
  wallet backup from"*;
- **109 test functions** across `sysw`, `mt`, `txqr` and the `gui`
  transaction/payload surface pass today.

So this document is **not** a specification for new construction. It is the
normative record of a surface that was built across the `sysw` and tx-engraving
cycles and never written down in one place, **plus** the specification of the
three things measurement shows are genuinely not done.

**Read §7 first if you want the scope answer.** This is not a cycle. It is
**two defects and one operator ruling**, and one of the two defects needs a
Rust-first change.

### 0.1 Three premises in the brief that measurement contradicts

Recorded here rather than silently corrected, because each one would have
shaped a wrong design.

| brief premise | measured |
| --- | --- |
| *"If SH2 meets a sealed container it cannot open…"* | SH2 **opens sealed containers today**. `gui/sysw_load.go` prompts for a BIP-39-word passphrase, confirms the word count before the KDF, and calls `sysw.Open`. There is no "cannot open" state except a wrong passphrase, which already refuses with a specific sentence (§4). |
| *"…a truncated NFC transfer"* | **A `sysw` container never arrives over NFC.** Every `sysw.Reader` implementation reads flash or an embedded blob (§1.4). NFC carries *bare records*, bounded by a different cap. The refusal the brief asks for has no reachable trigger at container level. |
| *"encrypted payloads are out of scope"* | Correct as a statement about **new work**, and already true of transactions by construction — but not because the device lacks the capability. It has it. §6 restates the boundary in terms that are true. |

---

## 1. Measured inventory

### 1.1 The container, as the code defines it

`crates/me-cli/src/sysw/wire.rs`, read directly:

| field | value |
| --- | --- |
| magic | `MNEMSYSW`, 8 bytes at offset 0 |
| region address | `0x10D00000`, fixed and normative |
| region length | 65,536 (16 × 4 KiB sectors) |
| header length | 52 bytes |
| `MAX_SECTION_LEN` | **32,734** = `(65536 − 52 − 16) / 2` |
| sealed discriminator | `ct_len > 0` |
| KDF / AEAD | PBKDF2-SHA256 / AES-256-GCM, 100,000–2,000,000 iterations |

The cap is an ugly number on purpose: two maxed sections plus header plus tag
must still fit the region, and both halves of that reasoning are **compile-time**
assertions in Rust (`const _: () = assert!(…)`) and negative-array-length
assertions in Go (`var _ [RegionLen - (HeaderLen + 2*MaxSectionLen + TagLen)]struct{}`).

### 1.2 Record classes — the full list

`crates/me-cli/src/sysw/record.rs:45-64`:

`Mnemonic`, `Codex32Secret`, `Passphrase`, `FreeText`, `Descriptor`, `MdMk`,
`Mt`, `Tx`, `Address`, `Unknown`.

Three predicates, all in `record.rs:74-108`:

- `is_secret()` = `Mnemonic | Codex32Secret | Passphrase`
- `is_bearer()` = **`Mt | Tx`, exactly**
- `is_argv_forbidden()` = `is_secret() || is_bearer()`

`Descriptor` and `Address` are **deliberately never returned** by `classify` on
either side — classifying them needs a descriptor parser and an address decoder.
An unclassifiable record is `Unknown` and the caller fails closed. This means
two of the ten classes are declared but unreachable, on both sides.

### 1.3 Transactions are unsealed by construction

`decide_sealing()` keys on whether any record `is_secret()`. Bearer is not
secret, so a payload holding only a transaction has no secret record and is not
sealed. **Verified by running it**, not by reading:

```
$ mt encode --qr --in tx.hex > qr.rec
$ me sysw pack --in qr.rec --no-passphrase --out qr.bin
sealing:  NOT SEALED — no record in this payload is secret material, so there
      is nothing to encrypt.
digest:   c282 6ca8 4f21 2887 02cc 70f0 91d7 5d34

$ me sysw show qr.bin
sealed:   false
pub_len:  447
ct_len:   0
identity: a9c3197d5fcc21e5f984b8eb3c26607a2e0ace1f39d9f1d0c7d6b67155aca3c8
public record 0: raw signed transaction — txid 2dcf2b97…72ebf630, 222 bytes
```

The operator's stated preference is the shipped behaviour, and it is structural
rather than a default that could be flipped.

### 1.4 Delivery is flash, never NFC — and the negative is bounded

```
$ grep -rn 'sysw.Reader\|SyswReader' --include="*.go" .
$ grep -rln 'sysw' --include="*.go" nfc/          # no output
```

Every `sysw.Reader` implementation found by that search:

| implementation | source |
| --- | --- |
| `sysw.XIPReader` (`read_tinygo.go`) | flash at `0x10D00000`, execute-in-place |
| `sysw.FileReader` (`read_host.go`) | a host file, for tests |
| `embeddedSyswReader`, `syswCardsReader` (`cmd/emu/`) | blobs compiled into the emulator |
| `countingSyswReader`, `testPlatform.sysw` | test fakes |

**Scope of the negative:** every `*.go` file in the `seedhammer` repository was
searched for `sysw.Reader`/`SyswReader`, and every `*.go` file under `nfc/` was
searched for `sysw`. Within that scope there is **no path by which an NDEF
message becomes a `sysw` container.** The absolute address appears in
`read_tinygo.go` and nowhere else, by the same rule `seal` follows.

The two caps belong to two transports and must not be confused:

| transport | carries | cap |
| --- | --- | --- |
| `picotool` → `0x10D00000` | a whole **container** | `MAX_SECTION_LEN` = 32,734 per section |
| NFC tag | one **bare record** | `gui/scan.go`'s 8 KiB scan buffer, **8191** |

### 1.5 The device is wired, not merely capable

```
$ grep -rln 'seedhammer.com/sysw' --include="*.go" . | grep -v _test.go
```

18 files. The ones that carry the flow:

| file | lines | role |
| --- | --- | --- |
| `gui/sysw_load.go` | 293 | read region → open → digest compare → fill session |
| `gui/sysw_session.go` | 223 | the loaded records and their flags |
| `gui/sysw_admit.go` | 134 | which class each program may take |
| `gui/sysw_source.go` | 137 | provenance naming and the acceptance screen |
| `gui/transaction.go` | 1,464 | the whole transaction program |

Plus wirings into `bundle_flow.go`, `derive_xpub.go`, `freetext_flow.go`,
`passphrase_flow.go`, `multisig.go`, `wallet_policy.go`, `scan.go`.

### 1.6 The suite is green

```
$ go test ./sysw/ ./mt/ ./txqr/            # EXIT=0
ok  seedhammer.com/sysw    0.113s
ok  seedhammer.com/mt      (cached)
ok  seedhammer.com/txqr    2.139s
$ go test -run 'Sysw|Transaction|Payload' ./gui/    # EXIT=0
ok  seedhammer.com/gui     2.298s
```

41 test functions in `sysw`+`mt`+`txqr`, 68 matching in `gui` — **109 total**,
counted with `go test -list`, not by hand.

> **Method note.** The first attempt at this returned exit **127** — `go` is not
> on `PATH` on this box; the toolchain is at
> `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`. Reported
> because an exit code read through a pipe would have shown an empty result and
> been recorded as "no tests".

### 1.7 How an engraving is driven, and what the container can supply

Measured by a dispatched recon agent
(`design/agent-reports/RECON-sh2-sysw-device-paths.md`) and **independently
re-checked line by line** before being written here.

The unit the machine cuts is one struct (`gui/gui.go:743`):

```go
type Plate struct {
    Duration uint64             // whole tick count; uint64 because the widest
                                // real plate is past MaxUint32 on a 32-bit target
    Spline   bspline.Curve      // the planned motion
    Conf     engrave.StepperConfig  // snapshotted so the run cannot disagree
                                    // with the plan
    id       ...
}
```

The path is `toPlate` (`gui/gui.go:3515`) → `NewEngraveScreen` →
`engraveJob.Start` → `runEngraving` (`gui/engraver.go:184`) → `Platform.Engraver()`
(`cmd/controller/platform_sh2.go:591`), which streams stepper knots.

**What the container supplies, and what it does not.** A `Plate` is
**geometry** — a spline and a motion config. The container supplies **none** of
it. It supplies *records*; `gui/transaction.go` turns a record into glyphs or QR
modules and `toPlate` turns those into a spline. `toPlate` returns `ErrTooLarge`
when the result does not fit the plate's safety margin, which is why a record
that packs on the host can still fail to cut — the two limits are unrelated, and
neither is `MAX_SECTION_LEN`.

**Carousel placement.** `loadPayload` is an **unconditional** navigable program
(`gui/gui.go:230`), deliberately unlike `unlockPayload` (the Sealed Payload),
which is conditional and appended last. Load Payload reports an empty region
itself rather than disappearing, so an operator who wrote a payload and sees
nothing gets a sentence instead of a missing menu entry.

---

## 2. Normative — what the device does today

Stated as behaviour, and asserted to be what the code does as measured above.
Anything in this section that a future change breaks is a **regression**, not a
design choice.

**N1. The device consumes; it never produces a container.** Package `sysw` has
no `pack`. This is stated in the package comment as a deliberate omission: it
removes the possibility of the device disagreeing with the host about how to
build a container it should never build.

**N2. The device never signs, never emits a PSBT, never returns anything over
NFC.** Its only output is an engraving. (Operator ruling; matches the code —
there is no write path.)

**N3. Bounds are checked before any KDF work.** `ParseHeader` rejects an
out-of-range iteration count, an over-long section, a bad magic and an unknown
version before anything expensive runs. The firmware has no active watchdog, so
an unbounded iteration count would be a hang rather than a slow open.

**N4. Section lengths are compared as `uint32`, never widened to `int`.** `int`
is 32 bits on the Cortex-M33 target, so `int(pub_len)` is negative for any
length with the top bit set and the cap would be bypassed. This is recorded as a
**Go-only porting error already fixed** (`sysw/header.go`); the Rust primary
compares `as usize` and was always correct.

**N5. A payload is loaded if and only if it was authenticated.** `[compared]`
has exactly two routes — a successful AEAD open, or the operator comparing the
displayed digest. Declining the comparison **unloads**: `ctx.sysw = nil`.

**N6. The digest is shown when and only when `pub_len > 0`.** The digest of an
empty record set is a constant every such payload shares, so showing it would
invite a comparison that proves nothing.

**N7. The public-data digest is computed, never stored.** A hash carried inside
the payload is rewritten by whoever rewrites the records.

**N8. The AEAD binds `header || public section` as AAD.** Binding only the
ciphertext's framing would let an attacker swap a public record for one encoding
their xpub with the tag still verifying.

**N9. Reserved prefixes fail closed.** `text:`, `pass:`, `tx:` and, since the
composer's Stage 1, `key:`, `hash:` and `now:` require
**lowercase** hex bodies; a body that is not is `Unknown` and refused, never
quietly treated as free text. Uppercase is rejected because the digest is taken
over the record as it appears on the wire, so two spellings would be two digests.

**N10. `tx:` additionally requires a structural transaction parse.** The prefix
cannot smuggle arbitrary bytes into a non-secret class.

**N11. Classification is stricter than entry.** `bip39.Parse` and `codex32.New`
are forgiving because a human is typing on a touchscreen. Classification is not
entry: the input is a payload someone else may have written, and forgiving there
means the device hands a program a secret the host tool would have refused.

**N12. An unconfirmed `md1`/`mk1`/`mt1` record counts as SECRET for flag
evaluation, and refuses nothing.** It loads, and the operator is warned.

**N13. Confirmation is by the real decoder, never by a checksum.** Any complete
set of BCH-valid `mt1` strings reassembles — the payload is opaque bytes — so
`mt.Decode` must also parse the bytes as one transaction *and* match the 20-bit
chunk-set id to the top 20 bits of the display txid.

**N14. Every input must carry a non-empty scriptSig or at least one witness
item.** This is the only thing separating a signature-stripped transaction from
the honest one it came from, since stripping the witness is exactly what the
txid ignores. Enforced on both the `tx:` and `mt1` classes, on both sides.

---

## 3. Normative — what the host supplies

**N15. `me sysw pack` is the only producer.** `me` consumes constellation
strings and manufactures none: `mt encode --qr` emits the `tx:` record, bare
`mt encode` emits the `mt1` strings.

**N16. `--expect` checks completeness, not merely presence.** A half-transmitted
`md1` or `mt1` set is present and still cannot be restored from. Vocabulary:
`descriptor, cosigner, transaction, mnemonic, secret`. `address` and
`passphrase` are deliberately absent — neither can ever be satisfied here, and a
kind that cannot be satisfied turns a gate into a permanent refusal.

**N17. Bearer and secret records are refused on argv by default.** Ruling
2026-08-26: uniform treatment. `--allow-argv-secret` exists because the threat
model does not bite on a single-user air-gapped box, and is greppable on purpose.

**N18. `--region` pads to 65,536 with `0xFF`** — the erased state of NOR flash,
so the image is byte-for-byte what the sector looks like with only the container
written. Verified: `wc -c` = 65536.

---

## 4. Refusals — what the operator SEES

This is the heart of a consume-only device. Every string below is **verbatim
from the current source or from a run**, not paraphrased.

### 4.1 On the device, at load (`gui/sysw_load.go`)

| trigger | what the operator sees |
| --- | --- |
| no reader, or `Probe()` false, invoked from the menu | `No payload found at 0x10D00000. Write one with `me sysw pack --region`.` |
| same, **at boot** | **nothing** — silent by design; a machine without a payload must behave exactly as it did before |
| `Read()` fails | `Could not read the payload region.` |
| `ParseHeader` fails (bad magic, version, section too long, iterations) | `There is no systemwide container at 0x10D00000.` |
| blob shorter than the header declares — **the truncation case** | `The payload is shorter than its header declares. Nothing was loaded.` |
| sealed, and the AEAD open fails | `That passphrase did not open this payload.` |
| unsealed, and `Open` fails (non-UTF-8 records) | `This payload could not be read.` |
| operator declines the digest comparison | `Digest not compared.\nNothing was loaded.` |

Note the deliberate wording rule already in the code: **never the words "payload
unreadable" for a structural failure**, because that phrase teaches the operator
to read a wrong file as tampering.

### 4.2 On the device, warnings that refuse nothing

Shown once per payload over the classes actually present, de-duplicated by
**(flag, cause)** rather than by flag — because a flag with two causes needs two
sentences:

- `A SECRET is stored unencrypted in flash.`
- `An md1/mk1 the device could not confirm - treated as a secret - is stored unencrypted in flash.`
- `The passphrase is below the word-count floor.`
- `An md1/mk1 the device could not confirm - treated as a secret - is protected by a passphrase below the word-count floor.`

The first of these is followed by a `KEEP` / `UNLOAD` choice, offered at the
moment the operator learns a secret is sitting unencrypted in flash.

### 4.3 On the device, in the transaction program (`gui/transaction.go`)

| trigger | what the operator sees |
| --- | --- |
| no payload loaded | `No payload is loaded.\n\nLoad one with Load Payload, or write one with `me sysw pack --region`.` |
| loaded but not compared | `This payload has not been checked, so nothing may be taken from it.\n\nCompare its digest at Load Payload.` |
| compared, but holds no transaction | `This payload holds no transaction.\n\nIt holds: <n class, n class…>.` |
| plus incomplete `mt1` strings | `<n> mt1 string(s) belong to no complete set. Pack every string of the set with `me sysw pack`.` |

The inventory is spelled out because *"the operator cannot otherwise tell a
payload with the WRONG contents from an empty one, and those have different
fixes — re-pack versus pack at all."* Class names are operator-facing words
(`raw transaction`, `md1/mk1 card`, `mt1 chunk`, `unrecognised record`) and live
in fork-native UI code, **not** in package `sysw`, because the Rust primary has
no such function and adding one to the port would be the port leading.

### 4.4 On the host — measured by running each one

| invocation | exit | stderr |
| --- | --- | --- |
| `--expect transaction,descriptor` on a tx-only payload | **4** | `me: --expect descriptor was not met: NO record of that kind is in the stream.` / `Looking for an md1 descriptor card.` / `Nothing was written -- a container built without it would flash and engrave, and the gap would only show when someone tried to restore.` |
| `--expect transaction` on 5 of 6 `mt1` strings | **4** | `me: --expect transaction was not met: records of that kind ARE present, but the set does not reassemble.` / `Unconfirmed at record 0, 1, 2, 3, 4 (records count from 0).` / `A partial set is not a backup: it passes every checksum it carries and still cannot be restored from. Nothing was written.` |
| an unclassifiable record | **4** | `me: record 0 (records count from 0) is not a form this container can place: not a BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:` record.` |
| `--expect transaction` on a complete set | **0** | — |

### 4.5 On the device, at the NFC record path

Not container delivery (§1.4) — this is the transport that *does* exist, and it
is where the brief's "truncated transfer" question has a real answer. The single
funnel is `(*scanner).Scan` (`gui/scan.go:29`), fed from 7 call sites of
`startScanner`; it classifies tag bytes into typed values and reports status
(`gui/gui.go:2274-2281`):

| status | what the operator sees |
| --- | --- |
| `scanFailed` | `Scan error` |
| `scanOverflow` — **the oversize/truncation case** | `Content too large` |
| `scanUnknownFormat` | `Unknown format` |
| `scanStarted` | `Scanning...` |

Per-flow refusals, verbatim and each verified at its line:

- `Not an md1/mk1 card.` — `gui/bundle_flow.go:83`
- `That tag is not a seed.` — `gui/derive_xpub.go:362`
- `Not a recognized address.` — `gui/verify_address.go:93`
- `Not an mt1 string.` — `gui/transaction.go:691` and `:765`

`scanOverflow` fires when the 8 KiB scan buffer is exactly full — the **8191**
boundary of §1.4, and the reason F-247 (§5.4) is about that number and not
about 32,734.

### 4.6 The refusal the brief asked for that has no trigger

**A sealed container the device cannot open.** There is no such state at
container level: if `ct_len > 0` the device prompts for a passphrase and either
opens it or says `That passphrase did not open this payload.` The nearest thing
to the brief's case is a container whose KDF or AEAD byte is unknown, which
`ParseHeader` rejects **before** any prompt — and the operator sees
`There is no systemwide container at 0x10D00000.`

**That sentence is wrong for that cause**, and it is the one genuine refusal gap
this document found. See §5.3.

---

## 5. What is NOT done

Three items. Two are defects; one is an unbuilt operator ruling.

### 5.1 G-P3.10 — a transaction is silently DROPPED (funds-visible)

**Live at `gui/transaction.go:467`** — re-measured today, because the follow-up
cites line 449 and the file has moved since:

```go
if c.confirmed && c.tx.TxidDisplay == tx.TxidDisplay {
    continue next // merged: the set candidate already carries the bytes
}
```

The merge is keyed on the **derived txid**, never on the bytes. Two consequences:

1. A `tx:` record byte-different from an existing candidate but sharing its txid
   is **skipped entirely** — not merged, not flagged, **dropped**. The operator
   is never told it existed. The pair that does this is not exotic: a
   transaction and its own signature-stripped form share a txid *by construction*.
2. Because each accepted `tx:` candidate is appended to `cands` and the inner
   loop ranges over `cands`, **two `tx:` records sharing a txid** collapse to one
   the same way.

**The operator has already ruled on this** (ruling 2026-08-25c, recorded in
`FOLLOWUPS.md`):

> *"If two transactions in a payload have same txid, we can just engrave both
> without much concern. The odds are low and we can't be responsible for every
> edge case."*

**The code does not do this, and is worse than the ruling assumes** — the ruling
addresses two confusing rows, the code produces one row and silent data loss.
The legitimate half of the merge must stay: the *same* transaction delivered
both as `mt1` strings and as a `tx:` record is one transaction in two forms, and
should present once.

**Normative statement required:** candidates merge on **bytes**. Two candidates
whose bytes differ are two candidates, whatever their txids agree on.

**Rust-primary:** **exempt.** This is fork-native GUI candidate-assembly with no
Rust counterpart. `payloadTransactions` has no analogue in `me-cli`.

### 5.2 G-P3.14 — the review screen shows nothing about where the money goes

The screen an operator confirms before cutting shows **no outputs, no amounts,
no fee, no locktime, no nSequence and no network.** Measured cause:

```go
type Tx struct {                     // seedhammer/mt/mt.go:130
    Raw, TxidDisplay, Inputs, Outputs, SegWit, EveryInputSigned, UnsignedInputs
}
```

`Outputs` is a **count**, not the outputs. This is a **parser** limitation, not
a screen limitation.

**This is where the Rust-primary rule bites, and the answer is not the obvious
one.** The Go `mt` package pins its provenance to
`me-cli/src/sysw/mt.rs and tx.rs`, so those are primary. Measured there:

```rust
pub struct TxSummary {               // crates/me-cli/src/sysw/tx.rs:35
    txid_display, size, inputs, outputs, segwit,
    every_input_signed, unsigned_inputs,
}
```

**The Rust primary carries no output values, addresses, locktime or network
either** — and its doc comment claims *"Everything a review screen needs"*,
which G-P3.14 measures as false. So this is **not** a convergence port that the
Go-defect exemption would cover.

> A caution against the obvious rebuttal: `mt inspect` *does* print addresses
> and amounts (verified — it printed `bc1qc80qm4p…  0.05000000 BTC` during this
> recon). That parser lives in the **`mnemonic-transaction`** repo, not in
> `me-cli::sysw::tx`, and it is not the one the device's port tracks. Citing it
> as evidence that "Rust already has this" would be reading the right fact off
> the wrong module.

**Verdict: this needs a normative Rust change, landed first, with test vectors.**
Owner: **`mnemonic-engrave`**, `crates/me-cli/src/sysw/tx.rs` — extend
`TxSummary` with per-output value and script/address, plus `locktime`, and let
the Go port converge afterwards. Whether `mnemonic-transaction`'s richer parser
should be the source of that shape, or whether `sysw::tx` should grow it
independently, is an **open design question this spec does not settle** (§8).

**It may also be legitimately reduced rather than built.** The acceptance sheet
already allows that: *"§3.4/§3.5's derived/asserted split is built, **or** the
reduction is ruled and this sheet amended."* The device engraves a **bearer**
instrument whose legend already says `BEARER - ANYONE HOLDING THIS CAN BROADCAST
IT`; an argument that the operator verified the transaction on the host, where
`mt inspect` shows everything, is available and consistent with **all
verification is host-side**. That argument should be made or rejected
explicitly, not left as a NOT-MET row.

### 5.3 The KDF/AEAD refusal names the wrong cause

A container with an unknown KDF or AEAD byte is rejected by `ParseHeader`, and
the operator is told `There is no systemwide container at 0x10D00000.` There
**is** one; it is a container this firmware is too old to open. The correct
shape already exists three rows above it in the same function — a specific
sentence per structural cause.

**This is the encrypted-payload refusal the brief asked for**, in the only form
that has a reachable trigger. It is bounded: one screen, one sentence, no
passphrase-entry surface, no new capability.

**Normative statement required:** a container whose version, KDF or AEAD this
firmware does not recognise refuses with a sentence naming **that** cause and
distinguishing it from "there is no container here".

**Rust-primary:** exempt — the Go `ParseHeader` already returns distinct error
values (`ErrVersion`, `ErrKDF`, `ErrAEAD`, `ErrSectionTooLong`, `ErrIterations`)
and the primary does too (`WireError::UnknownKdf`, …). Only the **GUI's
rendering** of them is uniform where it should not be. No wire, identity,
validation or admission behaviour changes.

### 5.4 Not a defect, but unresolved: F-247

`mt encode --qr` does not say whether the record fits an **NFC tag** (8191, not
32,734). The operator deferred it — *"skip nfc stuff for now"* — and it needs a
**ruling, not an implementation**. It is recorded here because §1.4's two-cap
distinction is exactly what makes it hard: `mt` has no `--out`, so it cannot
know the record's destination, and a line saying "fits an NFC tag" is noise on
every run of the commoner (picotool) journey.

---

## 6. Out of scope — and why

| item | why it is out |
| --- | --- |
| **Encrypted payload authoring** | Out per operator ruling. Transactions are already unsealed *structurally* (§1.3), not by a default. The device's existing sealed-open path is **not** removed — it is shipped, tested and load-bearing for secret-carrying payloads. What is out is any new work on sealing, passphrase entry, or KDF choice. §5.3 is the one bounded exception, and it adds a **sentence**, not a surface. |
| **Signing** | The device consumes only. Operator ruling 1: its only output is an engraving. There is no signing code and none is proposed. |
| **On-device verification** | Operator ruling 2: all verification is host-side, in the Rust constellation utilities. This is not merely a simplification — **SH2 has no camera**, so it can never read back a QR it cut, and an on-device "verify" would be checking its own memory against itself. The device receives, decides what to engrave, and engraves. |
| **P4, the cross-tool operator journey** | Deferred by operator ruling; F-370 schedules it as the **first post-release item**, not "eventually". |
| **`Descriptor` and `Address` classification** | Both classes are declared and neither is ever returned (§1.2). Making them reachable needs a descriptor parser and an address decoder on the device. Out of scope, and named so a reader does not mistake the declared class for a working one. |
| **NFC container delivery** | Not out by choice — it does not exist (§1.4) and nothing in this spec proposes it. Raising it later means designing a chunked transport for a 32,734-byte section over an 8,191-byte buffer. |

---

## 7. Scope — the honest read

**This is not a cycle. It is a burndown.**

| item | size | gate |
| --- | --- | --- |
| §5.1 G-P3.10 — merge on bytes | small; one predicate and its tests | funds-visible: silent loss of a transaction the operator packed |
| §5.3 KDF/AEAD refusal sentence | very small; one screen | no |
| §5.2 G-P3.14 — outputs on the review screen | **either** a Rust-first parser change plus a Go port plus a screen, **or** a written ruling that reduces it | blocks the acceptance sheet either way |
| §5.4 F-247 | a ruling, then possibly one stderr line | no |

If §5.2 is **ruled down**, the remaining work is roughly a day and belongs in
the existing tx-engraving cycle's burndown rather than a new one. If §5.2 is
**built**, it is a Rust-first change with vectors, a Go convergence port, and a
screen — call it a small cycle of its own, and it is the only part that
justifies the word.

**Phases, if it is built:**

| phase | content | owner |
| --- | --- | --- |
| S0 | operator rules §5.2 (build or reduce) and §5.4 | operator |
| S1 | §5.1 + §5.3 — both fork-native, no Rust dependency, ship independently | fork |
| S2 | *only if S0 says build:* `TxSummary` grows outputs/locktime in Rust, with vectors | `mnemonic-engrave` |
| S3 | Go convergence port of S2; review screen renders it | fork |
| S4 | the live journey walk (P3b) and the S0 hardware plate (P4) | operator |

**S1 does not wait on S0.** Neither of its items touches the parser.

---

## 8. What is NOT verified

Stated plainly, because a section resting on an unverified fact is worse than
one flagged as unknown.

1. **No hardware was involved.** Every device-side fact is from source and from
   host-run Go tests. Nothing here was observed on a SeedHammer II, and the
   engraving path in particular has an emulator and a test suite standing in for
   steel.
2. **I did not run the emulator walk.** The `cmd/emu` payload fixtures exist and
   the gui tests pass, but I did not drive the UI end to end. A gate that has
   never executed is a hypothesis — this document's §2 claims rest on unit
   tests, not on a walk.
3. **The `mt1` text-plate engraving path was not exercised**, only the `tx:`/QR
   pack path (§1.3). `planTransactionTextPlates` exists and is tested; I did not
   run it.
4. **I did not read `SPEC_systemwide_payloads.md` (93,890 bytes) or
   `SPEC_engrave_transaction.md` (100,400 bytes) in full** — only their section
   lists, and targeted greps. Both are large, both are normative, and this
   document may restate something one of them already says better. §2's
   statements were derived from **code**, so a disagreement between this
   document and either spec is a real finding, not a transcription slip.
5. **Whether §5.2 should draw on `mnemonic-transaction`'s parser** is an open
   design question. I established that the two parsers differ and which one the
   device tracks; I did not read `mnemonic-transaction`'s source (the brief said
   not to touch other repos, and several are mid-push).
6. **The three-premise correction in §0.1 is bounded by §1.4's search scope.**
   If an NDEF ingest path exists that names neither `sysw.Reader` nor
   `SyswReader`, nor mentions `sysw` under `nfc/`, I did not find it.
7. **`Descriptor`/`Address` unreachability** was established from `classify` on
   both sides. I did not exhaustively prove no other code path constructs those
   class values.

---

## 9. Provenance

| artifact | revision |
| --- | --- |
| `mnemonic-engrave` | `master` = `25102c5`, 57 commits ahead of `origin/master` |
| `seedhammer` | `main` = `0b656d7` = `origin/main` = `ship/tx-engraving` |
| `me` | 0.7.0 (`target/debug/me`) |
| `mt` | `/home/bcg/.cargo/bin/mt`, built 2026-08-26 |
| Go | 1.26.3, `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3` |

**A stale document this recon corrects:** `CONTINUITY_tx_engraving_2026-08-25.md`
says *"**Nothing pushed.**"* Measured today, the fork's `main`, `origin/main` and
`ship/tx-engraving` are all `0b656d7` — the tx-engraving work **is** pushed and
**is** on `main`. Its phase table is also behind: it shows P3a running, and the
fork has since taken P5 review folds (`cace554`, `2bb2b73`, `18c7522`, `0b656d7`).
