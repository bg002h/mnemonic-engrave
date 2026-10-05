# IMPLEMENTATION PLAN — SeedHammer fork, Refugium F5 and F7

> **Status: DRAFT 1 (2026-10-05).** FOLLOWUPS F-702 (companion
> `mr-gui-f-refugium-features` in `bg002h/refugium-wallet`). Parent plan:
> refugium-wallet `design/IMPLEMENTATION_PLAN_mr_gui_v1.md` (draft 7, f929084), lane F.
> F5 and F7 are the two lane-F phases with no upstream dependency; F1 waits on E1,
> F2/F3/F4/F6 wait on `refugium-codes` (A7) and get their own plan later.
> Risk set (seeds, an OTP writer's removal): R0 to 0 Critical / 0 Important before
> code; one implementer; an independent adversarial review of the whole diff after.

## 0. Baselines

| What | Where | Revision |
|---|---|---|
| Fork | `bg002h/seedhammer` main | be00ef8 |
| This repo | `bg002h/mnemonic-engrave` master | cb29b05 |
| UI spec | refugium-wallet `design/SPEC_ui_refugium_wallet.md` | draft 20 (§4.4 "QR codes on plates", §4.6.2, §14 item 8) |
| Recon | `design/agent-reports/refugium-F5-recon.md`, `refugium-F7-recon.md` | 2026-10-05, at be00ef8 |
| Standard SeedQR | SeedSigner `docs/seed_qr/README.md` (fetched verbatim by the F5 recon) | dev, 2026-10-05 |

## 1. What the spec asks

- **F5** (UI spec §4.4): ms1 plates carry a **Standard SeedQR** of the seed's BIP-39
  words (4-digit zero-padded indices, numeric mode), never a QR of the ms1 string. The
  QR is encoded **from the same held words** that passed the typed read-back, with
  test vectors. Parent Done-when: layout vectors, and a 96-digit SeedQR fits the QR
  size cap.
- **F7** (UI spec §4.6.2 and §14 item 8, [P-E7]): a Refugium build in which NFC is off
  while any secret is held (single-sig verify's NFC gatherer included); the NFC
  `lock-boot` OTP writer and the `FOREVERLAURA!` QA command removed; BIP-39 passphrase
  entry off in the seed sitting. Parent Done-when: a build test asserts both commands
  are absent from the binary, and an emulator test shows NFC off from seed entry to
  power-off.

## 2. Decisions this plan takes (defaults; the reviewer may challenge)

- **D1. The profile is a Go build tag, `refugium`.** A `//go:build refugium` /
  `//go:build !refugium` file pair in `gui` defines `const refugiumProfile`; the
  device build passes `-tags refugium` (`nix run .#build-firmware -- -tags refugium`;
  `flake.nix:118` forwards `"$@"`, to be confirmed by the implementer). `-ldflags -X`
  is rejected: it cannot remove code or literals. The default (untagged) build keeps
  upstream behaviour byte for byte where this plan does not say otherwise.
- **D2. F5 adds a layout; it replaces none.** Today's ms1 plate
  (`EngraveSeedString`, QR of the uppercase string) stays byte-identical, so goldens
  `codex32-0/1`, the seal's `MaxEngraveableCodex32Len = 90` and
  `backup/engraveable_test.go` hold. F5 does **not** rewire existing flows (codex32
  flow, sealed unlock, single-sig/multisig/composer bundle cards): the consumer is the
  F3/F4 seed sitting, which holds the words. Wiring is that plan's job.
- **D3. The NFC rule is a one-way latch per power cycle in the Refugium build.** Once
  any secret has entered memory, NFC stays off until power-off (the spec's "from seed
  entry to power-off"). A latch, not a "held now" counter, because tracking release
  across ~20 flows is where a counter would leak, and the sitting ends in power-off
  anyway. In the default build the latch records but gates nothing.
- **D4. Passphrase entry is off in the whole Refugium build,** not only in the seed
  sitting: the seed sitting does not exist yet (F2 to F4), and the Refugium build
  offers nothing that needs one in v1 (closing runs and `verify` with a passphrase are
  F8, after v1; parent plan §1). Revisit at F8.

## 3. F5 — ms1 plus Standard SeedQR plate

**Facts (recon, verified):** `seedqr.QR(m)` writes `%04d` per word index
(`seedqr/seedqr.go:22-33`); the vendored `qr.Encode` picks numeric mode for an all-digit
string; measured widths at `qr.M`: 12 words (48 digits) 25 modules, 24 words (96 digits)
29 modules, against the seed-plate cap `seedQRMaxSize = 33` at `qrScale = 3`
(`backup/backup.go:154-156,194`). SeedSigner's spec agrees (48 digits 25x25, 96 digits
29x29). `codex32.EncodeMS1(entropy)` = `NewSeed("ms",0,"entr",'s',[0x00]‖entropy)`.

**Delivers:**

1. `backup.EngraveSeedStringSeedQR(params engrave.Params, plate SeedString, m bip39.Mnemonic) (engrave.Engraving, error)`
   (name may change if it collides with fork style). It:
   - refuses unless `m` is a valid English BIP-39 mnemonic of **12 or 24** words
     (Refugium's two lengths; 15/18/21 refused with a message naming the count);
   - recomputes `codex32.EncodeMS1(m.Entropy())` and refuses unless it equals
     `strings.ToLower(plate.Seed)` ("ms1 string and words disagree"), so the string row
     and the QR cannot come from different seeds. The recomputed entropy buffer is
     wiped as the existing callers wipe theirs;
   - encodes `qr.Encode(string(seedqr.QR(m)), qr.M)` (the level upstream's word plate
     uses, `gui/gui.go:851-873`), refuses above `seedQRMaxSize`, and reuses
     `engraveSeedString` unchanged for geometry (string columns, fingerprint row, title
     row). The title is the caller's (the seed letter label is F3's decision).
   If importing `codex32`/`seedqr`/`bip39` into `backup` creates an import cycle, the
   function lives in a small new package instead; the checks do not move to callers.
2. **Vectors** `backup/testdata/ms1_seedqr_vectors.json` (+ a `.provenance.json`
   naming each source): per row the words, entropy hex, the ms1 string, the SeedQR
   digit string and the QR width at `qr.M`. Rows: the SeedSigner spec's published
   Standard SeedQR examples (their digit strings copied verbatim, so the digits are
   checked against the external authority, not against our own encoder), BIP-39's
   standard English test vectors at 12 and 24 words, and the M15 public signet test
   seeds that are 12 or 24 words. A test asserts, per row: `seedqr.QR(words)` equals
   the digits; `seedqr.Parse(digits)` gives the words back; `EncodeMS1(entropy)` equals
   the ms1 string; the decoded ms1 entropy equals the words' entropy; the width is as
   recorded and at most 33.
3. **Goldens:** `backup/testdata/ms1-seedqr-24.bin` and `ms1-seedqr-12.bin` through
   `compareGolden` (`-update` regenerates), each with a non-zero fingerprint and a
   title, from vector rows.
4. **Refusal tests:** one word changed (string and words disagree); 18 words; a
   non-English mnemonic if the type can carry one, else a note why not reachable;
   today's `EngraveSeedString` goldens unchanged (`codex32-0/1` pass untouched).
5. **Preview:** an entry in `gui/preview.go` (`previewBuilders`) so `cmd/plateview`
   renders the new plate.
6. **Secret-copy inventory:** the SeedQR digit string and `qr.Code` bitmap are
   unwipeable copies of the seed; name them in a comment beside the new function, as
   `gui/unlock_session.go:262` does for its own.

**Done when:** the vector test, both goldens and the refusal tests pass; `go test
./backup/ ./seedqr/ ./codex32/` green; the default gui suite unaffected.

## 4. F7 — Refugium build profile

### 4.1 Profile switch

`gui/profile_refugium.go` (`//go:build refugium`) and `gui/profile_default.go`
(`//go:build !refugium`) declare `const refugiumProfile = true/false`. Whatever
`gui/tinygo_split_test.go` requires of a new `//go:build` pair (it discovers every
pair) is satisfied explicitly, with its reason recorded in that test's exception map
if needed — never by weakening the scan.

### 4.2 Debug commands removed

- Move the `debugCommand` arm of the start screen (`gui/gui.go:2264-2277`) behind a
  function `handleDebugCommand` in a tag pair: `debugcmd_default.go` holds today's
  code verbatim (both literals); `debugcmd_refugium.go` logs "debug commands are not in
  this build" and returns to idle, with **neither literal** in the file. The
  `qaProgram` enum value stays (the guard at `gui/gui.go:268` needs it).
- `cmd/controller/platform_sh2.go`'s `LockBoot` and `writeOTPValues` (`:553`, `:724`)
  move to a `!refugium` file; the `refugium` twin's `LockBoot` returns an error and
  links no OTP writer. Nothing else calls `EnableSecureBoot` or `AddBootKey` from
  firmware code (recon §3); the implementer re-greps and records the result.
- `PASSPROOF!` (`gui/passphrase_passproof.go:65`): if it is a test/QA trigger, it is
  absent from the Refugium build too; if it is a user feature, it is gated with the
  passphrase (D4). The implementer states which.

### 4.3 NFC latch

- `Context` gains a latch (an `atomic.Bool`, since the scanner goroutine reads it) and
  `ctx.noteSecret()` sets it. `ctx.nfcReader()` returns nil when
  `refugiumProfile && latched`, else `ctx.Platform.NFCReader()`. **All eight**
  `startScanner(ctx, ctx.Platform.NFCReader())` sites (`gui.go:2236`,
  `bundle_flow.go:225`, `derive_xpub.go:343`, `verify_address.go:77`,
  `mk1_inspect.go:204`, `md1_gather.go:111`, `transaction.go:750`, and the eighth the
  recon counted) switch to `ctx.nfcReader()`. A test parses the gui sources and fails
  if `Platform.NFCReader()` is called anywhere except `nfcReader`.
- The poll loop in `startScanner` checks the latch every iteration and exits when it
  is set under the profile, so a scanner already running when a secret arrives stops
  without waiting for its screen to leave. The `FeatureNFC` SCAN row
  (`derive_xpub.go:290`) is hidden while latched.
- **Fail closed on a stuck reader.** `Device.Close` can time out (`ErrCloseTimeout`,
  `nfc/poller/poller.go:101-138`) and leave the field on. Under the profile, a stop
  that reports a close timeout while latched shows a non-secret screen "NFC did not
  turn off. Power off now." and offers nothing else. The implementer finds how the
  timeout surfaces to `startScanner`'s stop function (`gui/nfc_scan.go:127`) and wires
  it; if it cannot surface, the plan comes back with that finding before code.
- **Where `noteSecret` is called** (every place a secret enters memory): typed words
  (`inputWordsFlow` and the seed-entry flows in `derive_xpub.go`), typed codex32,
  SLIP-39 entry, the NFC seed SCAN row's result (`scanSeedFlow`), start-screen scans of
  mnemonic, codex32, SLIP-39 or `pass:` records (`gui.go:2590-2613`), a systemwide
  payload holding any secret class (`ctx.sysw`), the sealed-unlock session, and any
  further entry the implementer finds. The implementer greps for every constructor of
  `bip39.Mnemonic`, every `codex32` decode and every passphrase sink in `gui`, and
  records the list with file:line in the implementation report; the reviewer checks it.
- **Tests** (`//go:build refugium`, run with `-tags refugium`): for each entry point,
  drive it on `testPlatform` with a counting fake NFC reader, then assert the reader is
  not read again in that power cycle (later screens included: single-sig verify's
  gatherer at `singlesig_verify.go:145`, multisig verify, multisig build verify, the
  start screen). A default-build test asserts the same walk still scans (upstream
  behaviour kept). This is the parent plan's "emulator test": `testPlatform` is the
  emulator's model of the platform; the implementer also adds a `cmd/emu` walk script
  if one can observe NFC state, and says if it cannot.

### 4.4 Passphrase entry off

Under the profile: the eight "Add a BIP-39 passphrase?" prompts (`gui.go:2818`,
`derive_xpub.go:426`, `singlesig.go:120`, `singlesig_verify.go:116`, `multisig.go:144`,
`multisig_verify.go:921`, `multisig_build_slots.go:886`, `bip85.go:324`) are skipped as
"no passphrase"; the core `passphraseFlow`/`passphraseFlowTitled` and
`syswPassphraseFlow`/`syswPassphraseFlowTitled` refuse (return the no-passphrase result
and log) so a missed prompt still cannot reach them; an NFC or payload `pass:` record
is never applied to a seed (it may still be shown or engraved as a record if that
path does not attach it to a seed; the implementer states which paths attach). SLIP-39
passphrases (`slip39_polish.go:291`) are gated the same way. Tests per prompt under the
tag.

### 4.5 Build tests

- **Host, every `go test`:** a test builds a host binary that links `gui` with
  `-tags refugium` (a tiny `main` under `gui/testdata/` or `cmd/emu` for `GOOS=js`;
  the implementer picks what builds on host) and asserts the bytes contain neither
  `FOREVERLAURA!` nor `lock-boot`; a positive control builds the same binary untagged
  and asserts both are present, so the test cannot pass vacuously.
- **Device, CI:** `.github/workflows/test.yml`'s TinyGo device build gains a
  `-tags refugium` build of `./cmd/controller` and a scan of the **ELF** (not the
  `.uf2`, where a literal can straddle a block) for both literals and for a unique
  literal of `writeOTPValues` (a white-label string), with the same positive control
  on the default build. TinyGo is not in this container, so this step is proven green
  in CI before review closes.
- CI also runs `go test -tags refugium ./gui/ -run Refugium` and `./backup/`.

**Done when:** all of §4's tests pass under the tag; the default gui suite passes
unchanged (sharded with `mnemonic-engrave/scripts/gui-shard-test.sh`, confirmed to
forward `-tags`, else run serially); CI's device step is green on both builds.

## 5. Order and delivery

One PR to `bg002h/seedhammer` main from `claude/project-thread-59q2a6`, two commits
(F5, then F7). Gofmt baseline: diff `gofmt -l .` against the known five-file set.
Then the independent adversarial review of the whole diff (opus), its report verbatim
to `design/agent-reports/refugium-F5-F7-exec-review.md`, folds to 0C/0I. Merging goes
through the Merging PRs thread on Brian's typed go-ahead. No OTP, signing or hardware
step is in this plan.

## 6. Risks

- The latch misses an entry point. Mitigated by the grep inventory in the report, the
  reviewer's independent grep, and the source-parse test that every scanner gets its
  reader through `nfcReader`.
- The tag pair trips the split test, or the shard script drops `-tags`. Found in the
  first run; fixed in the test harness, never by narrowing a scan.
- The F5 function lands without a consumer until F3/F4. Accepted: the vectors pin it,
  and the seed sitting plan wires it.
