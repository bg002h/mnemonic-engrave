# RECON: sysw wiring, NFC ingest, refusal screens, engrave pipeline, mt/txqr

Repo: /scratch/code/shibboleth/seedhammer (read-only recon)
Module (go.mod:1): `module seedhammer.com` — so the sysw import path is `seedhammer.com/sysw`.

---

## 1. Who imports `sysw`?

Command run:
```
grep -rln 'seedhammer\.com/sysw' --include='*.go' .
```
37 files import it. Split by test/non-test:

### Non-test importers (17)

| File | Calling function(s) that touch `sysw.*` |
|---|---|
| `/scratch/code/shibboleth/seedhammer/gui/sysw_load.go` | `syswLoadFlow` (gui/sysw_load.go:25) — calls `sysw.ParseHeader` (:63), `sysw.Identity` (:75), `sysw.Open` (:146), `sysw.PublicDataHash`/`sysw.FormatHash` (:166,177), `sysw.CliffAbove` (:184), `sysw.HeaderLen` (:165). Also `syswHasFlag` (:240), `syswLoadWarnings` (:261). |
| `/scratch/code/shibboleth/seedhammer/gui/sysw_session.go` | `syswSession.load` (:80) calls `sysw.MDMKUnconfirmed` (:99), `sysw.MTUnconfirmed` (:106), `sysw.Classify` (:111); `syswSession.take` (:123), `syswSession.takeAll` (:167), `syswSession.has` (:182); `syswOffer` (:204), `syswOfferTitled` (:210) — both typed on `sysw.Class`. |
| `/scratch/code/shibboleth/seedhammer/gui/sysw_source.go` | `syswPassphraseFlowTitled` (:98) calls `sysw.DecodeBody` (:104); `syswSourceAccept` (:115) is typed on `sysw.Class`. |
| `/scratch/code/shibboleth/seedhammer/gui/sysw_admit.go` | package-level table `admitted` (:32-51, keyed on `sysw.Class*`), `admits` (:60), `syswFlags` (:115). Line 51: `progTransaction: {sysw.ClassMt: true, sysw.ClassTx: true}`. |
| `/scratch/code/shibboleth/seedhammer/gui/scan.go` | `(*scanner).Scan` (:29) calls `sysw.DecodeBody` (:72), checks `sysw.PassPrefix`/`sysw.TextPrefix` (:76,108-109); `isSyswEncoded` (:107). |
| `/scratch/code/shibboleth/seedhammer/gui/sysw_unload.go` | `syswPayloadMenu` (:34), `syswPayloadHasTransaction` (:84, walks `s.records`, `r.class`), `syswUnloadFlow` (:105), `syswReloadCost` (:150) — all operate on the `*syswSession` (sysw-typed records), not directly on package `sysw` funcs. |
| `/scratch/code/shibboleth/seedhammer/gui/bundle_flow.go` | `bundleFlow` (:25) via `syswOffer(ctx, th, sysw.ClassMDMK, ...)` (:25 area). |
| `/scratch/code/shibboleth/seedhammer/gui/derive_xpub.go` | `syswSeedPickerTitled` (:283, uses `sysw.Class`). |
| `/scratch/code/shibboleth/seedhammer/gui/multisig.go` | `supplyMultisigPolicyFlow` (:96) via `syswOffer(ctx, th, sysw.ClassMDMK, "First card from where?")`. |
| `/scratch/code/shibboleth/seedhammer/gui/multisig_build_payload.go` | `buildCosignerSource` (:67). |
| `/scratch/code/shibboleth/seedhammer/gui/passphrase_flow.go` | `engravePassphraseFlowFrom` (:662) via `syswOffer(ctx, th, sysw.ClassPassphrase, "Password from where?")` (:662); `engravePassphraseFlowPreloaded` (:815). |
| `/scratch/code/shibboleth/seedhammer/gui/wallet_policy.go` | `walletPolicyFlow` (:39) via `syswOffer(ctx, th, sysw.ClassMDMK, "First card from where?")`. |
| `/scratch/code/shibboleth/seedhammer/gui/freetext_flow.go` | `engraveTextFlowFrom` (:1496) via `syswOffer(ctx, th, sysw.ClassFreeText, "Text from where?")`. |
| `/scratch/code/shibboleth/seedhammer/gui/gui.go` | `uiFlow` (:1988) — calls `syswLoadFlow(ctx, th, ctx.Platform.SyswReader(), true)` at **gui.go:2031**, then `syswPayloadMenu(ctx, th)` at **:2032**; `newInputFlow` (:2747) via `syswOffer(ctx, th, sysw.ClassMnemonic, "Seed from where?")` (:2747) and `syswOffer(ctx, th, sysw.ClassCodex32Secret, ...)` (:2763). |
| `/scratch/code/shibboleth/seedhammer/gui/transaction.go` | `payloadTransactions` (:408) via `ctx.sysw.takeAll(sysw.ClassMt)` (:408) and `sysw.ClassTx` (:451); `txClassName` (:276, `sysw.Class` param); switch on `sysw.ClassMt`/`sysw.ClassTx` at :292/:294. |
| `/scratch/code/shibboleth/seedhammer/cmd/controller/platform_sh2.go` | `(*Platform).SyswReader` (:587) — `return sysw.XIPReader{}`. **This is the real-hardware implementation** (build tag `tinygo && rp`, see §2/§4 below). |
| `/scratch/code/shibboleth/seedhammer/cmd/emu/platform.go` | `(*platform).SyswReader` (:313, build tag `js`) — returns `syswCardsReader{}` / `nil` / `embeddedSyswReader{}` depending on `p.syswChoice`. This is the **browser/wasm emulator**, not the device. |
| `/scratch/code/shibboleth/seedhammer/cmd/emu/sysw_cards_payload.go` | `syswCardsReader.Probe`/`.Read` (:77-88, build tag `js`) — wraps an embedded test-blob (`//go:embed sysw_cards_payload.bin`), test fixture, not production data. |
| `/scratch/code/shibboleth/seedhammer/cmd/emu/sysw_test_payload.go` | `embeddedSyswReader.Probe`/`.Read` (:84-98ish, build tag `js`) — same shape, second embedded test blob. |

### Test importers (20)
`cmd/emu/gaterecord_anchor_test.go`, `cmd/emu/sysw_cards_payload_host_test.go`, `cmd/emu/sysw_test_payload_host_test.go`, `gui/blanking_ink_test.go`, `gui/gui_test.go`, `gui/multisig_build_payload_test.go`, `gui/multisig_supply_passphrase_test.go`, `gui/raster_test.go`, `gui/s6b_passphrase_plate_test.go`, `gui/sysw_admit_oracle_test.go`, `gui/sysw_admit_test.go`, `gui/sysw_cells_test.go`, `gui/sysw_confirm_test.go`, `gui/sysw_coverage_witness_test.go`, `gui/sysw_load_test.go`, `gui/sysw_payload_menu_test.go`, `gui/sysw_picker_test.go`, `gui/sysw_takeall_test.go`, `gui/sysw_unload_test.go`, `gui/transaction_crosslang_test.go`, `gui/transaction_messages_test.go`, `gui/transaction_test.go`.

### Verdict on Q1
`sysw` is **wired in**, heavily — it is not an orphaned library. It backs a real, unconditional carousel entry (`loadPayload`, "Load Payload", program constant at gui/gui.go:230) offered automatically at boot (`uiFlow`, gui.go:2031) and reachable any time from the menu. Every one of the device's other programs (`progBackupWallet`, `progPassword`, `progText`, `progXpub`, `progBundle`, `progSingleSig`, `progMultisig`, `progWalletPolicy`, `progBip85`, **`progTransaction`**) asks `ctx.sysw` for records via `syswOffer`/`syswOfferTitled`/`takeAll` (gui/sysw_admit.go:32-51). The real firmware's `sysw.Reader` is `sysw.XIPReader{}` returned from `cmd/controller/platform_sh2.go:587-589`, reading the "SYSTEMWIDE" flash region at **0x10D00000** (see comments at gui/sysw_load.go:15-22, :31-32, :66). This is a *different* container from the "Sealed Payload" (region 0xE1000000, `seal.XIPReader`, program `unlockPayload`) — the two are deliberately kept apart per in-repo comments (gui/sysw_load.go:19-22).

---

## 2. The NFC ingest path

Chain (real hardware, `cmd/controller`):

1. `cmd/controller/platform_sh2.go:572-574`: `(*Platform).NFCReader() io.ReadCloser { return poller.New(p.nfc) }` — `p.nfc` wraps an `st25r3916.Device` (ISO14443a/ISO15693 reader chip).
2. `/scratch/code/shibboleth/seedhammer/nfc/poller/poller.go:41-90` (`func New`, `(*Poller).Read`): on each `Read`, polls for a tag (`p.poll`, :103-120, tries ISO15693 via `nfc/type5`, then ISO14443a via `nfc/type2`), wraps the tag's raw byte stream in `ndef.NewMessageReader` (:83) then `ndef.NewRecordReader` (:88).
3. `/scratch/code/shibboleth/seedhammer/nfc/ndef/ndef.go`: `MessageReader.Read` (:36) unwraps NDEF TLV blocks; `RecordReader.Read` (:95) parses NDEF record headers and, for a **Text** well-known record (`case 'T':`, :183), skips the IANA language code and yields the payload bytes only.
4. **Entry point that receives the decoded bytes:** `/scratch/code/shibboleth/seedhammer/gui/nfc_scan.go:45` `func startScanner(ctx *Context, r io.ReadCloser) (chan scanResult, func())` runs a goroutine that calls `s.Scan(r)` (:62) where `s` is a `*scanner` (gui/scan.go). `startScanner` is invoked with `ctx.Platform.NFCReader()` from seven call sites: `gui/md1_gather.go:86`, `gui/mk1_inspect.go:183`, `gui/verify_address.go:77`, `gui/derive_xpub.go:343`, `gui/bundle_flow.go:207`, `gui/gui.go:2140` (the top-level StartScreen carousel scanner), `gui/transaction.go:750` (the mt1-chunk gather scanner).
5. `(*scanner).Scan(r io.Reader) (any, error)` at `/scratch/code/shibboleth/seedhammer/gui/scan.go:29` is **the function that receives the decoded NDEF text bytes** and classifies them. What it does next (gui/scan.go:53-104): checks a `command: ` debug prefix; then `isSyswEncoded` (sysw `text:`/`pass:` hex-prefixed records, decoded via `sysw.DecodeBody`); then tries, in order, `bip39.Parse` (seed mnemonic), `nonstandard.OutputDescriptor`, `codex32.New` (single codex32 share), `codex32.ValidMD`/`ValidMK` (md1/mk1 chunk), `codex32.ValidMT` (mt1 transaction chunk → routed to the transaction gather, comment at scan.go:94-97), `btcaddr.DecodeAddress` (mainnet/testnet). Anything matching none of these returns `errScanUnknownFormat` (scan.go:103).

So the single funnel from "tag physically read" to "typed Go value the rest of the GUI consumes" is `(*scanner).Scan`, and the typed results it returns (`freeTextScan`, `passScan`, `addressText`, `mdmkText`, `mtText`, `bip39.Mnemonic`, `*bip380.Descriptor`, `*codex32.String`, `debugCommand`) are consumed by whichever flow started that particular `startScanner` call (e.g. `gui/transaction.go:759-766` type-asserts `scan.Object.(mtText)`).

---

## 3. What happens to unparseable input — the operator-visible refusal screens

Search commands run:
```
grep -rn 'scanUnknownFormat\|scanFailed\|scanIdle\|scanStarted' --include='*.go' gui/ | grep -v _test.go
grep -rn 'showError(ctx, th,' --include='*.go' gui/ | grep -v _test.go | grep -iE 'unknown|unreadable|malform|invalid|not recognized|could not|corrupt|refus'
```

**Scan-level status strings** — `scanStatus` enum at `/scratch/code/shibboleth/seedhammer/gui/gui.go:3550-3558` (`scanIdle, scanStarted, scanOverflow, scanUnknownFormat, scanFailed`), rendered as a transient subtitle in the top-level carousel by `(*StartScreen).draw`:
```
gui/gui.go:2274   case scanFailed:        sttxt = "Scan error"
gui/gui.go:2276   case scanOverflow:      sttxt = "Content too large"
gui/gui.go:2278   case scanStarted:       sttxt = "Scanning..."
gui/gui.go:2280   case scanUnknownFormat: sttxt = "Unknown format"
```

**Per-flow messages** shown while gathering (each of these is drawn as body text on that flow's own screen, not a modal):

- `gui/verify_address.go` (`scanAddressFlow`, :74-120):
  - `:93` `"Not a recognized address."` (scanUnknownFormat)
  - `:95` `"Scan failed - try again."` (scanFailed)
- `gui/derive_xpub.go` (`scanSeedFlow`, ~:340-390):
  - `:362` `"That tag is not a seed."` (recognized, wrong class)
  - `:364` `"Unrecognized tag."` (scanUnknownFormat)
  - `:366` `"Scan failed - try again."` (scanFailed)
- `gui/md1_gather.go` (:100-115):
  - `:106` `"Different descriptor - rescan the right chunks."`
  - `:108` `"Already captured that chunk."`
  - `:110` `"Not an md1 descriptor chunk."`
- `gui/mk1_inspect.go` (:197-211):
  - `:202` `"Different key - rescan the right card."`
  - `:204` `"Already captured that chunk."`
  - `:206` `"Not an mk1 key chunk."`
- `gui/transaction.go` (`(*txGather).offer`, :686-738, and `transactionGatherFlow`, :743-793):
  - `:691` and `:765` `"Not an mt1 string."`
  - `:696` `"Already scanned that string."`
  - `:709-710` `fmt.Sprintf("String %d of %d. %d to go.", ...)` (incomplete set, still gathering)
- `gui/bundle_flow.go` (`(*bundleGatherScreen).feedback`, :67-87):
  - `:70` `"Type the ms1 share on-device, never over NFC."` (explicit refusal — ms1 must never travel over NFC)
  - `:73` `"Incomplete key card: the payload is missing some of its chunks."` / `:75` `"Incomplete key card: scan all its chunks."`
  - `:81` `"Already captured that card."`
  - `:83` `"Not an md1/mk1 card."` (the generic "recognized-but-wrong-class-or-dropped" refusal for the bundle gather)
  - and (:221-224) `"No complete cards yet. Scan a card's chunks first."` / `"No complete cards. Pack them on the host with `me sysw pack` and load the payload again."`

**Container-level refusals** (sysw SYSTEMWIDE payload malformed/unreadable), all via `showError(ctx, th, "Load Payload", ...)` in `/scratch/code/shibboleth/seedhammer/gui/sysw_load.go`:
- `:32` `"No payload found at 0x10D00000. Write one with `me sysw pack --region`."`
- `:60` `"Could not read the payload region."`
- `:66` `"There is no systemwide container at 0x10D00000."`
- `:71` `"The payload is shorter than its header declares. Nothing was loaded."`
- `:150` `"That passphrase did not open this payload."`
- `:152` `"This payload could not be read."`
- `:205` `"Digest not compared.\nNothing was loaded."`

**Sealed Payload container refusals** (different container, region 0xE1000000): `"Payload unreadable."` at `gui/unlock_flow.go:35,43,77` and `gui/unlock_kdf.go:454`.

**Transaction-specific "recognized but does not confirm" screens** (not a scan refusal but the analogous "we can't make sense of this as a transaction" surface): `transactionReviewLines` in `gui/transaction.go:824-872` — headline `"UNCONFIRMED SET"` (:828), with body `"This did not confirm as a transaction on this device.\nThe strings are engraveable and each is valid.\n\nThis device does not parse every transaction. Know what you are engraving."` (:853-864). Note this is a *deliberate* design choice (ruling 2026-08-25/26 per in-code comments): an unconfirmed/unparseable transaction set is **not refused**, it is reported loudly and still engraveable — see §5.

Command used for the sysw error grep is reproduced above; ran from repo root `/scratch/code/shibboleth/seedhammer`.

---

## 4. How an engraving is driven — "operator confirms" → "machine cuts"

The data structure handed to the engraver is `gui.Plate`, defined at `/scratch/code/shibboleth/seedhammer/gui/gui.go:743-785`:
```go
type Plate struct {
    Duration uint64                  // total engraver tick count (uint64: firmware target is 32-bit, widest plate exceeds MaxUint32)
    Spline   bspline.Curve           // the actual cut geometry
    Conf     engrave.StepperConfig   // motion config the Spline was PLANNED with (snapshotted, not re-read from the platform)
    id       uint64                  // unexported; correlates a completed engrave back to the string validated (0 unless built via validateMdmk)
}
```
It is built by `toPlate(plan engrave.Engraving, params engrave.Params) (Plate, error)` at **gui/gui.go:3515-3529**, which calls `engrave.PlanEngraving(params.StepperConfig, plan)` to get the `Spline`, `bspline.Measure(spline)` for `Duration`, and checks the geometry fits within `SquarePlate` bounds (returns `ErrTooLarge` if not).

Pipeline once a `Plate` exists:
1. Operator confirmation screen — a `ChoiceScreen{Choices: []string{"ENGRAVE"}}` with a hold-to-confirm delay (`ConfirmDelay`, `confirmDelay = 1*time.Second`, gui.go:419) — e.g. `engraveTransactionPlates` at `gui/transaction.go:1093-1126`.
2. `NewEngraveScreen(ctx, plate)` at **gui/gui.go:3191-3198** builds `job := newEngraverJob(ctx.Platform, plate.Spline, plate.Conf, 0)`.
3. `(*EngraveScreen).Engrave(ctx, th) bool` at **gui/gui.go:3212-3288** drives the UI state machine; on the confirm-delay completing it calls `s.job.Start()` (:3257).
4. `(*engraveJob).Start` (`gui/engraver.go:95-113`) launches `e.runEngraving(quit, progress)` on a goroutine.
5. `(*engraveJob).runEngraving` (`gui/engraver.go:184-224`) is where the physical cut happens: `d, err := e.pl.Engraver(stall)` (:186) obtains the real `Engraver` from the `Platform` interface, then `drv := stepper.NewDriver(d)` (:205) and `for k := range e.spline { ...; t, err := res.Knot(k); ...; reportProgress(...) }` (:208-222) streams each B-spline knot to the driver, finishing with `drv.Flush()` (:223).
6. On real hardware, `Platform.Engraver(stall bool)` is `/scratch/code/shibboleth/seedhammer/cmd/controller/platform_sh2.go:591-604`, returning a `*homingEngraver` whose `Write(steps []uint32)` (:613-624) homes the axes then calls the physical stepper/needle driver's `Write`, driving the GPIO-attached TMC2209 steppers and needle solenoid.

**What must be populated for a plate to be cut**: a non-zero `Plate.Spline` (the actual toolpath) and a non-zero `Plate.Conf` (`engrave.StepperConfig{TicksPerSecond, Speed, EngravingSpeed, Acceleration, Jerk}` — see comment at gui/gui.go:759-766: "A ZERO Conf IS A PROGRAMMING ERROR... Jerk=0 divides by zero inside SafePointer.Resume"). `Duration` is derived/measured, not independently meaningful; `id` is optional bookkeeping. In practice every caller reaches `Plate` only through `toPlate`, never a hand-built literal (comment gui.go:763).

---

## 5. Is there a transaction-engraving path already? YES — fully wired, not stubbed.

`mt/` (`/scratch/code/shibboleth/seedhammer/mt/mt.go`) — package doc comment (mt.go:1-23): "decodes mt1 (signed-transaction) constellation strings: the 11-symbol chunk header, chunk-set reassembly, and the structural transaction parse + txid binding that CONFIRMS a set." It is explicitly a **decode-only PORT** (comment: "PORT, NOT PRIMARY... may never be led by this port", referencing `mnemonic-transaction/crates/mt-codec` and `mnemonic-engrave/crates/me-cli`). Key API: `ParseHeader` (:74), `Decode(in []string) (Tx, error)` (:192, the confirmation routine — reassembles, parses as a Bitcoin tx, and checks the chunk_set_id binds to the txid), `ParseTx(raw []byte) (Tx, error)` (:272, for standalone `tx:` records). **There is deliberately no mt1 encoder** in Go (comment at gui/sysw_source.go and repeated in gui/transaction.go:22-27,64-65: "the device deliberately has no mt1 ENCODER: a Go encoder would be a second implementation of a normative format").

`txqr/` (`/scratch/code/shibboleth/seedhammer/txqr/txqr.go`) — package doc (:1): "encodes a raw signed Bitcoin transaction as QR symbols for [Structured Append]." This is the **outbound/engrave-time encoder**: `EncodeSet(data []byte, k int, level qr.Level) ([]*qr.Code, error)` (:46) turns confirmed transaction bytes into a set of QR codes for the plate.

Who calls them (search: `grep -rln 'seedhammer\.com/mt"' . ` / `grep -rln 'seedhammer\.com/txqr"' .`):
- `mt`: `sysw/classify.go:53` (`mt.ParseHeader`, to classify a record as `sysw.ClassMt`), `sysw/record.go:115` (`mt.ParseTx`, to classify a `tx:` record as `sysw.ClassTx`), `sysw/confirm.go:160,180` (`mt.ParseHeader`/`mt.Decode`, used by `MTUnconfirmed` to flag incomplete sets), and **only** `gui/transaction.go` outside `sysw` and tests.
- `txqr`: **only** `gui/transaction.go` (`planTransactionQRPlates`, :1288+).

**The on-device flow that exists today**, entirely in `/scratch/code/shibboleth/seedhammer/gui/transaction.go` (1464 lines):
- Menu entry: `engraveTransaction` program constant (gui/gui.go:222, titled `"Engrave Transaction"` at gui.go:2226), wired into the carousel dispatch at `gui/gui.go:2070-2072` (`case engraveTransaction: engraveTransactionFlow(ctx, th)`), and reachable a second way from the Load Payload menu when the loaded payload holds transaction records (`gui/sysw_unload.go:48-50,64-67`, `"ENGRAVE TRANSACTION"` choice).
- `engraveTransactionFlow`/`engraveTransactionFlowSeeded` (gui/transaction.go:300-361): sources candidates either from the loaded sysw payload (`payloadTransactions`, :404-...) or from a live NFC gather (`transactionGatherFlow`, :743-793, which itself uses the same `startScanner`/NFC path as §2, filtering for `mtText`).
- `(*txGather).offer` (:686-738) accumulates mt1 chunks per `chunk_set_id`, calls `mt.Decode` once the declared chunk count is reached, and produces a `txCandidate` (:59-100) — `confirmed bool` is the "this device parsed it and the txid binds" flag.
- `transactionReviewAndEngrave` (:906-963) shows a review (`transactionReviewLines`, :800-904, with an explicit **"BEARER: anyone holding the plates can broadcast it."** warning, :884), lets the operator pick TEXT or QR plates, plans them (`planTransactionTextPlates` :1153, or `planTransactionQRPlates` :1288, which is where `txqr.EncodeSet` is called), and engraves via `engraveTransactionPlates` (:1093-1126) — the exact same `NewEngraveScreen(...).Engrave(...)` pipeline described in §4.
- Post-cut instructions telling the operator to verify off-device with `mt inspect`/`mt verify`/`mt decode` CLI tools (`transactionPostCutLines`, :1016-1087) since "This machine has no camera and can never read a plate back."

**Explicitly out of scope / does NOT exist here** (stated in the file's own header comment, gui/transaction.go:22-33): *"NOTHING HERE SIGNS, BUILDS OR BROADCASTS. The device engraves what it was handed."* So: no signing, no PSBT construction, no transaction building, and no broadcast anywhere on-device — confirmed by grepping (`grep -rn 'Broadcast\|BroadcastTx\|SignTx\|psbt\.' gui/ cmd/` returns nothing relevant beyond the comment/UI text above). The device is a pure **display-and-cut** consumer of externally-produced mt1 strings or `tx:` records.

**What is genuinely absent**: an on-device *signing ceremony* or PSBT flow feeding this — the mt1/tx: material is assumed to arrive already finalized (via the sysw payload written by `me sysw pack` on a host, or scanned NFC chunks produced by the same host tooling / `mt-codec`/`me-cli`). This recon searched only `mt/`, `txqr/`, `sysw/`, `gui/`, `cmd/` for anything signing- or broadcast-shaped and found none; a "there is no signing path" claim is bounded to those directories.

---

## 6. Screen/flow inventory for payload loading ("Load Payload"-style flows)

Command run: `grep -rn 'progTransaction\|syswLoadFlow\|syswOffer\b\|syswOfferTitled\|loadPayload\b' --include='*.go' gui/ cmd/ | grep -v _test.go`, plus reading the `program` enum.

Full carousel menu (`type program int`, `/scratch/code/shibboleth/seedhammer/gui/gui.go:175-243`, titles from the `titleTxt` switch at gui.go:2209-2236), in navigation order:
1. `backupWallet` → **"Backup Wallet"**
2. `engravePassphrase` → **"BIP-39 Password"**
3. `engraveText` → **"Engrave Text"**
4. `engraveXpub` → **"Account Xpub"**
5. `engraveBundle` → **"Engrave Bundle"**
6. `engraveSingleSig` → **"Engrave Single-Sig"**
7. `engraveMultisig` → **"Engrave Multisig"**
8. `walletPolicy` → **"Wallet Policy"**
9. `engraveTransaction` → **"Engrave Transaction"** (gui.go:222, transaction engraving — see §5)
10. `loadPayload` → **"Load Payload"** (gui.go:230, unconditional; dispatches to `syswPayloadMenu`, gui/sysw_unload.go:34)
11. `bip85Derive` → **"BIP-85 Child Seed"**
12. `unlockPayload` → **"Sealed Payload"** (gui.go:241, conditional — only shown when `Platform.PayloadReader().Probe()` finds the *other* container; different subsystem, `seal/`, frozen per repo comments)
13. `qaProgram` — non-navigable (QA-only, excluded from the carousel bound by the compile-time guard at gui.go:256)

**"Load Payload" itself** (`syswPayloadMenu`, gui/sysw_unload.go:34-76) is the single menu entry that ingests external data wholesale: it reads the SYSTEMWIDE flash region (`ctx.Platform.SyswReader()`), and if nothing is loaded yet, IS the load flow (`syswLoadFlow`, gui/sysw_load.go:25). Once loaded, it offers `"LOAD AGAIN"` / `"UNLOAD"`, plus a content-derived `"ENGRAVE TRANSACTION"` entry when the payload holds anything the transaction program admits (`syswPayloadHasTransaction`, sysw_unload.go:84-94, via the admission table `admitted[progTransaction]` at gui/sysw_admit.go:51).

**Every other "…from where?" input picker** in the other 8 engrave programs (Backup Wallet, BIP-39 Password, Engrave Text, Account Xpub, Engrave Bundle, Single-Sig, Multisig, Wallet Policy) offers the *same* loaded sysw payload as one of its sources via `syswOffer`/`syswOfferTitled` (gui/sysw_session.go:204-222) — e.g. `gui/gui.go:2747` `"Seed from where?"`, `gui/multisig.go:96` and `gui/bundle_flow.go:25` and `gui/wallet_policy.go:39` `"First card from where?"`, `gui/passphrase_flow.go:662` `"Password from where?"`, `gui/freetext_flow.go:1496` `"Text from where?"`. These are the *consuming* picker screens, not separate "Load Payload"-style ingest flows — the one and only bulk-ingest flow is `syswLoadFlow`/`syswPayloadMenu` under the "Load Payload" carousel entry (plus the NFC scan-per-flow entry points from §2 for anything not sourced from the payload).

---

## Bounding statement for negative claims in this report

- "sysw has no signing/broadcast path": bounded to `grep -rn` over `mt/`, `txqr/`, `sysw/`, `gui/`, `cmd/` in this repo for signing/broadcast-shaped identifiers; not exhaustive of every string in the tree.
- "nfc/ndef, nfc/type2/4/5 are only reached via nfc/poller": based on `grep -rn 'nfc/ndef\|nfc/type2\|nfc/type4\|nfc/type5\|nfc/poller' --include='*.go' . | grep -v '^\./nfc/'`, which found only `cmd/controller/platform_sh2.go` (imports `nfc/poller`, `nfc/type5` directly) and `nfc/poller/poller.go` itself (imports the type2/type4/type5/ndef siblings). No other file in the tree imports the type2/type4/type5/ndef packages directly.
- "cmd/emu is not the real firmware": based on build tags — `cmd/emu/*.go` carries `//go:build js` (browser/wasm emulator), `cmd/controller/*.go` (the files examined) carries `//go:build tinygo && rp` (real RP2350 hardware). `cmd/controller/main.go` is the real `func main()`.
