# IMPLEMENTATION PLAN — SeedHammer fork, Refugium F5 and F7

> **Status: DRAFT 3 (2026-10-05).** Draft 1 (972665e): R0 round 1, opus 0C/8I/10M/3N
> (`design/agent-reports/refugium-F5-F7-plan-R0.md`). Draft 2 (bdc87b3): round 2, opus
> 0C/3I/8M/3N (`refugium-F5-F7-plan-R0-round2.md`). Draft 3 folds round 2 (fold table
> §7), and the device ELF gate has now been run once (§4.5). Biggest change: in the Refugium build NFC is off for
> the whole power cycle, not latched on first secret (R0 I-2 to I-6).
> FOLLOWUPS F-702 (companion `mr-gui-f-refugium-features` in `bg002h/refugium-wallet`).
> Parent plan: refugium-wallet `design/IMPLEMENTATION_PLAN_mr_gui_v1.md` (draft 7,
> f929084), lane F. F5 and F7 are the two lane-F phases with no upstream dependency;
> F1 waits on F-699 (E1), F2/F3/F4/F6 wait on `refugium-codes` (A7) and get their own
> plan later. Risk set (seeds, an OTP writer's removal): R0 to 0 Critical / 0 Important
> before code; one implementer per phase; an independent adversarial review of each
> phase's whole diff after.

## 0. Baselines

| What | Where | Revision |
|---|---|---|
| Fork | `bg002h/seedhammer` main | be00ef8 |
| This repo | `bg002h/mnemonic-engrave` master | fde7841 (draft 1 was planned at cb29b05; nothing in between touches the fork) |
| UI spec | refugium-wallet `design/SPEC_ui_refugium_wallet.md` | draft 20 (§4.4 "QR codes on plates", §4.6.2, §14 item 8) |
| Recon | `design/agent-reports/refugium-F5-recon.md`, `refugium-F7-recon.md` | 2026-10-05, at be00ef8 |
| Standard SeedQR | SeedSigner `docs/seed_qr/README.md` (fetched verbatim by the F5 recon) | dev, 2026-10-05 |
| Rust ms1 reference | `ms-codec` at mnemonic-engrave's pin, git rev `e5dff6f` (0.10.0; not on crates.io) | recorded in the vector provenance |
| TinyGo | 0.41.1 (the flake's overlay version); release tarball used here because the flake's GitHub inputs are unreachable from this container | measured 2026-10-05 |

## 1. What the spec asks

- **F5** (UI spec §4.4): ms1 plates carry a **Standard SeedQR** of the seed's BIP-39
  words (4-digit zero-padded indices, numeric mode), **never a QR of the ms1 string**.
  The QR is encoded from the same held words that passed the typed read-back, with
  test vectors. Parent Done-when: layout vectors, and a 96-digit SeedQR fits the QR
  size cap.
- **F7** (UI spec §4.6.2 and §14 item 8, [P-E7]): a Refugium build in which NFC is off
  while any secret is held (single-sig verify's NFC gatherer included); the NFC
  `lock-boot` OTP writer and the `FOREVERLAURA!` QA command removed; BIP-39 passphrase
  entry off in the seed sitting. Parent Done-when: a build test asserts both commands
  are absent from the binary, and an emulator test shows NFC off from seed entry to
  power-off.

## 2. Decisions this plan takes

- **D1. The profile is a Go build tag, `refugium`.** A `//go:build refugium` /
  `//go:build !refugium` file pair defines `const refugiumProfile`; the device build
  passes `-tags refugium` (`nix run .#build-firmware -- -tags refugium`; `flake.nix:118`
  forwards `"$@"`, verified in R0). `-ldflags -X` cannot remove code. The untagged build
  keeps upstream behaviour byte for byte except where §4.1's characterization-tested
  refactor moves code without changing it.
- **D2. F5 adds a layout; it replaces none.** Today's `EngraveSeedString` (QR of the
  uppercase string) stays byte-identical, so goldens `codex32-0/1`, the seal's
  `MaxEngraveableCodex32Len = 90` and `backup/engraveable_test.go` hold. F5 wires no
  existing flow; the consumer is the F3/F4 seed sitting. **In the Refugium build,
  F7 removes every producer of a QR of an ms1 string** (§4.4), so the spec's "never"
  holds there from F7 on, not from F3.
- **D3. NFC is off for the whole power cycle in the Refugium build** (replaces draft
  1's latch). Neither the UI spec nor the parent plan gives the Refugium build any use
  for NFC: payloads arrive in the systemwide container over USB, seeds are typed, and
  NFC-sourced secrets would otherwise need refusing (R0 I-4). A latch is a denylist of
  secret entry points and R0 found three it missed (I-3); "never on" needs no list. It
  also removes the reader-fault cases (I-2), since the field is never turned on. This
  is stricter than the spec's "while a secret is held"; Brian is asked to confirm on a
  decision card, and work proceeds on it meanwhile.
- **D4. Passphrases are off in the Refugium build, loudly.** The seed sitting does not
  exist yet (F2 to F4) and nothing the Refugium build offers in v1 needs one (closing
  runs and `verify` with a passphrase are F8, after v1). Every place that would ask
  shows a notice instead of silently skipping (§4.3). Revisit at F8.

## 3. F5 — ms1 plus Standard SeedQR plate (PR 1)

**Facts (recon and R0, verified):** `seedqr.QR(m)` writes `%04d` per word index
(`seedqr/seedqr.go:22-33`); the vendored `qr.Encode` picks numeric mode for an
all-digit string; widths at `qr.M`: 48 digits 25 modules, 96 digits 29, against the
seed-plate cap `seedQRMaxSize = 33` at `qrScale = 3` (`backup/backup.go:154-156,194`);
SeedSigner's spec agrees. `codex32.EncodeMS1(entropy)` =
`NewSeed("ms",0,"entr",'s',[0x00]‖entropy)`. `backup` may import `codex32`, `seedqr`
and `bip39` without a cycle (R0 checked).

**Delivers:**

1. `backup.EngraveSeedStringSeedQR(params engrave.Params, plate SeedString, m bip39.Mnemonic) (engrave.Engraving, error)`.
   It is a general layout function; the word-count policy (24, or 12 by override) is
   Refugium's and lives in F3, not here (R0 M-7). It:
   - refuses unless `m` is a valid English BIP-39 mnemonic whose SeedQR fits the cap
     (every valid length does: 24 words is 29 modules);
   - recomputes `codex32.EncodeMS1(m.Entropy())` and refuses unless it equals
     `strings.ToLower(plate.Seed)` ("ms1 string and words disagree");
   - if `plate.MasterFingerprint` is non-zero, recomputes it from the words (no
     passphrase) and refuses a mismatch, so the fingerprint row cannot come from
     another seed (R0 M-7). The derivation is `masterFingerprintFor`'s
     (`gui/gui.go:894-913`), moved into a non-gui package both callers use, with its
     wipes of the 64-byte BIP-39 seed and the master private key kept (round 2 M-6);
   - encodes `qr.Encode(string(seedqr.QR(m)), qr.M)` (upstream's word-plate level),
     refuses above `seedQRMaxSize`, and reuses `engraveSeedString` unchanged.
   - **Wiping:** the entropy buffer from `m.Entropy()` and `EncodeMS1`'s internal
     payload are wiped with a helper the implementer names (gui's `wipeBytes` is not in
     `backup`; add a local one or move it, R0 M-8), as are the BIP-39 seed and master
     key of the fingerprint check. The comment beside the function
     lists the copies that stay live and cannot be wiped: the SeedQR digit string, the
     `qr.Code` bitmap, the recomputed ms1 string and the fingerprint input (R0 M-8).
2. **Vectors** `backup/testdata/ms1_seedqr_vectors.json` plus `.provenance.json`.
   Rows: SeedSigner's published Standard SeedQR examples (digit streams copied
   verbatim) and BIP-39's standard English test vectors at 12 and 24 words (no M15
   rows, R0 M-9). Columns: words, entropy hex, ms1 string, SeedQR digits, QR width at
   `qr.M`. **The ms1 column is generated by Rust**: `ms-codec` at mnemonic-engrave's
   pin (git rev `e5dff6f`, 0.10.0), `encode(Tag::ENTR, &Payload::Entr(e))`, through a
   small generator committed under `backup/testdata/gen/` (its own `Cargo.toml`, not a
   workspace member); never by the Go encoder it tests. The provenance names the repo,
   rev and command (R0 I-8b, round 2 M-5; round 2 confirmed crates.io 0.9.0 matches Go
   byte for byte, so either source is sound, and the pin is used). The SeedQR
   digits for BIP-39 rows come from SeedSigner's rule applied by the generator in Rust
   or Python, not by `seedqr.QR`. Per row the test asserts: `seedqr.QR(words)` equals
   the digits; `seedqr.Parse(digits)` gives the words; `EncodeMS1(entropy)` equals the
   ms1 string; the width is as recorded and at most 33.
3. **Goldens** `backup/testdata/ms1-seedqr-24.bin` and `ms1-seedqr-12.bin` via
   `compareGolden` (`-update` regenerates), each with a correct non-zero fingerprint
   and a title, from vector rows.
4. **Refusal tests:** one word changed (string and words disagree); a wrong
   fingerprint; an invalid checksum; a non-English mnemonic if the type can carry one,
   else a sentence in the report saying why it cannot reach the function. Today's
   `codex32-0/1` goldens pass untouched.
5. **Preview:** an entry in `gui/preview.go` (`previewBuilders`) so `cmd/plateview`
   renders the new plate.

**Done when:** the vector test, both goldens and the refusal tests pass; `go test
./backup/ ./seedqr/ ./codex32/` green; the default gui suite still passes (sharded).

## 4. F7 — Refugium build profile (PR 2)

### 4.1 Profile switch and debug commands

- `gui/profile_refugium.go` / `gui/profile_default.go` declare `refugiumProfile`.
  `gui/tinygo_split_test.go` finds only `_tinygo.go` pairs, so it neither sees nor
  blocks this pair (R0 M-2). A new guard does for refugium pairs what that test does
  for tinygo pairs: each `*_refugium.go` file's constraint is `X && refugium` (X empty
  in gui, `tinygo && rp` in `cmd/controller`) with a twin carrying `X && !refugium`,
  and no `*_refugium.go` file has a string literal on §4.5's absent list (round 2 M-2).
- **Characterization tests first** (R0 M-6), in the default build: a `lock-boot` tag
  on the start screen calls `Platform.LockBoot` once and stays on the start screen; a
  `FOREVERLAURA!` tag returns `qaProgram`. Then move the `debugCommand` arm
  (`gui/gui.go:2264-2277`) into `handleDebugCommand` returning a small result enum
  (stay / return action), in a tag pair: `debugcmd_default.go` holds today's code;
  `debugcmd_refugium.go` logs and stays, with neither literal. With NFC off (§4.2) no
  tag reaches it in the Refugium build; it is removed anyway.
- **QA programs** (R0 M-5): under the profile the start screen never dispatches
  `qaProgram` (`gui.go:2148`), and the QA proof triggers `PASSPROOF!`
  (`passphrase_passproof.go:52-65`), `TEXTPROOF!`, `CONSTPROOF!` and the other
  `ftProofTrigger*` triggers are absent from the Refugium build: a
  `const proofTriggersEnabled` pair gates each comparison, and the trigger constants
  move to `!refugium` files. A trigger is never disabled by setting it to `""`, which
  would fire on an empty passphrase or text (round 2 M-1). The `qaProgram` enum
  value stays (guard at `gui.go:268`); whether `qaEngraveFlow` code still links is
  measured by §4.5 and reported.
- `cmd/controller/platform_sh2.go`'s `LockBoot` and `writeOTPValues` (`:553`, `:724`)
  move to a `!refugium` file; the `refugium` twin's `LockBoot` returns an error and
  links no OTP writer. `signKeyHash` stays in the shared file, since
  `isSecureBootEnabled` (`:770-771`) still uses it (R0 N-2). The comment at
  `seal/record.go:212-219` citing `gui.go:1672` and `platform_sh2.go:545` is refreshed
  in the same commit (R0 N-3). R0 confirmed nothing else in firmware calls `AddBootKey`,
  `EnableSecureBoot` or the white-label writers; the implementer re-greps and records.

### 4.2 NFC off

Enforced twice, so neither layer alone carries it (round 2 I-3):

- **Device platform.** Under the profile, `cmd/controller` does not report
  `gui.FeatureNFC` (`platform_sh2.go:313`) and `NFCReader()` returns an untyped `nil`
  (not a typed nil inside the interface). At boot it calls
  `(*st25r3916.Device).Close()` once, which writes `regOpCtrl` (0x02) = 0, clearing
  `en`, `rx_en`, `tx_en` and `wu` (`driver/st25r3916/st25r3916.go:297-301,791`); a warm
  reboot does not reset the chip, so the write is needed. It is the firmware's only
  contact with the chip in this build. If it fails, the platform records an NFC fault;
  `gui.Run` checks the platform's fault before anything else and shows one non-secret
  screen, "NFC could not be turned off. Power off.", that takes no input (round 2 M-7).
- **gui.** Under the profile `startScanner` treats every reader as nil, and one helper
  `ctx.nfcAvailable()` (false under the profile, else `Features().Has(FeatureNFC)`)
  replaces every `FeatureNFC` read and keys every scan offer. A source-parse test
  fails if `Features().Has(FeatureNFC)` appears outside `nfcAvailable`. A tagged test
  gives `testPlatform` a counting reader and drives each of the seven `startScanner`
  sites (`gui.go:2236`, `bundle_flow.go:225`, `derive_xpub.go:343`,
  `verify_address.go:77`, `mk1_inspect.go:204`, `md1_gather.go:111`,
  `transaction.go:750`): zero reads. The untagged run of the same test reads
  (positive control).
- **Scan offers.** Under the profile no screen offers or asks for a scan. The floor
  list, keyed on `FeatureNFC` today: `bundle_flow.go:195`, `transaction.go:316`,
  `derive_xpub.go:287`; not keyed today (round 2 M-3): `verify_address.go:20` (Scan or
  Type choice; under the profile Type only), `composer_door.go:127` ("Scan cards"),
  `sysw_session.go:237` (`syswAltScan`, the decline arm of `syswChoose`),
  `md1_gather.go:142` and `mk1_inspect.go:232` ("Scan the next chunk."). Each moves onto
  `nfcAvailable()` and gets a tagged test that the offer is absent. The implementer adds
  any further site found and lists, per flow, where its card comes from instead (the
  systemwide payload where the flow already takes one, else a screen saying this build
  takes cards from the payload only). No flow may wait on a reader that does not exist.
- **Emulator walk** (parent Done-when; round 2 I-1). `cmd/emu`'s `presentedCount`
  counts tags the page presents, not reads (`cmd/emu/nfc.go:94-104`). So F7 first adds
  a `deliveredCount`, incremented in the emulated reader's `Read` (`nfc.go:145-162`) and
  exposed as `shNFC.delivered()`, with a host test in `cmd/emu` for it. `cmd/emu`'s
  platform gets its own `refugium` twin (it cannot see gui's unexported constant): no
  `FeatureNFC` (today always reported, `platform.go:350`), nil reader. The walk: power
  on, start the single-sig engrave flow, type a seed, engrave its cards, and end on that
  flow's final done screen (gui has no power-off prompt; the end of the flow is the end
  of the session), presenting a tag at every screen. Refugium build: `presented() > 0`
  and `delivered() == 0` at the end. Default build: `delivered() > 0`. It runs headless
  through Playwright on the preinstalled Chromium (`/opt/pw-browsers`); the implementer
  runs the default-build walk **first**, before any profile code, and a failing control
  stops the phase. CI gains `GOOS=js GOARCH=wasm go vet -tags refugium ./cmd/emu/`
  (round 2 M-8); the walk itself is run by the implementer and the reviewer, and its
  output goes in the implementation report.

### 4.3 Passphrases off

Under the profile (R0 I-7):
- The eight "Add a BIP-39 passphrase?" prompts (`gui.go:2818`, `derive_xpub.go:426`,
  `singlesig.go:120`, `singlesig_verify.go:116`, `multisig.go:144`,
  `multisig_verify.go:921`, `multisig_build_slots.go:886`, `bip85.go:324`) are each
  replaced by a notice the user acknowledges: "This build takes no BIP-39 passphrase.
  The seed is used without one." The flow then continues as the no-passphrase branch.
- The core `passphraseFlow`/`passphraseFlowTitled` and
  `syswPassphraseFlow`/`syswPassphraseFlowTitled` return `("", false)` under the
  profile (the implementer confirms at each call site that `false` reads as "no
  passphrase given", not "cancel", and changes the pin if a site reads it otherwise),
  so a missed prompt still cannot take one.
- **SLIP-39** (`slip39_polish.go:280-295`): skipping its passphrase would recover a
  different valid seed. Under the profile, the SLIP-39 passphrase question stays, and
  "yes, the shares have a passphrase" ends in a stop: "This build cannot take a SLIP-39
  passphrase. Recover these shares on another build." Only "no passphrase" continues.
- The passphrase engrave program is hidden from the menu, and a payload holding a
  `pass:` record is refused whole at load, with a message naming the record (dropping
  one record silently would change what the payload's digest covers; round 2 nit). The
  default build is unchanged.
- Tests per site under the tag.

### 4.4 No QR of an ms1 string in the Refugium build

R0 I-8a: under the profile, the producers of a QR of an ms1 string are cut to text
only: the codex32 engrave flow (`codex32_polish.go:218-251`, which uses
`EngraveSeedString`), the sealed-unlock codex32 plate (`unlock_session.go:189-246`),
bundle ms1 cards' "TEXT + QR" and "QR ONLY" choices, reached through
`validateMdmkStrings` (`gui.go:2667-2740`) from `singlesig_engrave.go:24`,
`multisig_engrave.go:36` and the composer (`composer_flow.go:584-647`), and the Engrave
Text program, which today only warns on ms1-shaped text (`freetext_flow.go:1068`,
`sysw_session.go:308-317`) and still offers "Add QR" (`freetext_flow.go:538`) (round 2
M-4). `validateMdmkStrings` takes no card kind, so the gate tests each string's HRP
(`ms`, case-insensitive, the same predicate as the free-text warning) rather than
threading a kind through. Each producer offers text only under the profile, or, where
the plate function needs a QR, the implementer adds a text-only variant to that path
with a golden; ms1-shaped free text gets "No QR" forced. A tagged test walks each
producer and asserts no QR whose content is an ms1 string is emitted. F3/F4 later add
the SeedQR plate (F5) to the seed sitting.

### 4.5 Build tests

R0 I-1 showed byte scans of switch literals fail: gc compiles short `case` strings to
integer compares. So:
- **Source level, every `go test`:** the Refugium file sets are
  `go list -tags refugium -f '{{.GoFiles}}' ./gui` and
  `go list -e -tags tinygo,rp,refugium -f '{{.GoFiles}}' ./cmd/controller` (round 2
  M-2; verified to run on host). A test parses those files with `go/parser` and
  asserts no **string literal** (`*ast.BasicLit`) equals or contains `FOREVERLAURA!`,
  `lock-boot`, `PASSPROOF!`, `TEXTPROOF!`, `CONSTPROOF!` or
  `https://seedhammer.com/doc/?d=SHII`, and no **call** names `writeOTPValues`,
  `AddBootKey`, `EnableSecureBoot` or a white-label writer (comments are ignored, round
  2 M-1). Positive control: the default sets (`-tags ''` and `-tags tinygo,rp`) contain
  each.
- **Device, TinyGo ELF** (round 2 I-2). Measured 2026-10-05 on be00ef8, TinyGo 0.41.1
  release tarball with the flake's flags (`-target pico-plus2 -stack-size 16kb -gc
  precise -opt 2 -scheduler tasks`, ELF output): `writeOTPValues`, `AddBootKey` and
  `EnableSecureBoot` have **no symbols** (inlined into `LockBoot`'s caller), and
  `FOREVERLAURA!` occurs 0 times. What is present in the default ELF and so usable as
  markers: data literals `lock-boot`, `seedhammer.com/doc/?d=SHII` (the OTP redirect URL
  `writeOTPValues` writes), `PASSPROOF`, `TEXTPROOF`, `CONSTPROOF`; symbols
  `(*seedhammer.com/nfc/poller.Poller).Read` and `seedhammer.com/gui.qaEngraveFlow`.
  The check: in the Refugium ELF every marker is absent; in the default ELF every
  marker is present (positive control, measured above). The OTP writer's absence rests
  on the redirect literal plus the source-level call check, since it has no symbol of
  its own. The implementer re-measures on the PR's base, records the marker list and
  any `st25r3916` symbol (`Detect`, `reset`) that disappears with the reader, and runs
  both builds locally before review. In CI, `test.yml`'s TinyGo job builds to `-o
  /dev/null` today (`test.yml:139`); it gains two ELF builds and the marker check.
- **Tagged tests in CI:** `GOFLAGS=-tags=refugium` with the shard script (it does not
  forward `-tags` itself, R0 M-3). The tagged run uses an anchored test list, and a
  `-v` count of `--- PASS` lines must reach the expected number (R0 M-4). The
  implementer lists which default-suite tests cannot pass under the tag (NFC-driven,
  passphrase-driven, ms1-QR-driven) and how they are excluded (a `!refugium` constraint
  on the test file, or a skip that names the profile), so the rest of the suite runs
  tagged. Plus `GOOS=js GOARCH=wasm go vet -tags refugium ./cmd/emu/` and `cmd/emu`'s
  host tests.

**Done when:** every §4 test passes under the tag; the default gui suite passes
unchanged (sharded); both ELF checks pass locally and in CI; the emulator walk passes
on both builds.

## 5. Order and delivery

Two PRs to `bg002h/seedhammer` main (R0 M-10): PR 1 F5 (library, vectors, goldens),
PR 2 F7 (profile, CI). Each phase: re-validate this plan against the tree, one
implementer in a worktree (tests first), the gofmt five-file baseline, then an
independent adversarial review of the whole diff (opus), report verbatim to
`design/agent-reports/refugium-F5-exec-review.md` and `refugium-F7-exec-review.md`,
folds to 0C/0I. Merges go through the Merging PRs thread on Brian's typed go-ahead.
No OTP, signing or hardware step is in this plan.

## 6. Risks

- D3 is stricter than the spec. If Brian wants NFC for public data in this build,
  §4.2 becomes draft 1's latch plus R0's I-2 to I-5 fixes, and this plan goes back to
  review.
- A screen still promises a scan. Mitigated by §4.2's per-site list and the emulator
  walk.
- The F5 function lands without a consumer until F3/F4. Accepted: vectors pin it, and
  F7 already removes the old ms1 QR from the Refugium build.

## 7. Fold tables

### R0 round 1

| Finding | Fold |
|---|---|
| I-1 host byte scan fails its control; device gate never run | §4.5: source-level file-set check, ELF symbol and data-literal check, run locally via nix, positive controls |
| I-2 reader faults leave the field on | D3: NFC never on in the build; §4.2 explicit field-off at boot, fail closed |
| I-3 latch misses entry points | D3 replaces the latch |
| I-4 NFC-sourced secrets accepted | D3: no tag is ever read |
| I-5 screens promise a scan | §4.2 wording sites and per-screen list |
| I-6 emulator test relabelled | §4.2 `cmd/emu` walk with positive control |
| I-7 passphrase gating silent, SLIP-39 wrong-seed, ambiguous result | §4.3 notices, SLIP-39 stop, `("", false)` pinned and checked, program hidden, `pass:` refused |
| I-8a ms1-string QR still produced | D2 and §4.4 |
| I-8b circular ms1 vectors | §3 item 2: Rust-generated with provenance |
| M-1 seven sites | §4.2 says seven |
| M-2 split test blind to the pair | §4.1 corrected, new guard |
| M-3 shard script and `-tags` | §4.5 `GOFLAGS` |
| M-4 vacuous `-run` | §4.5 anchored list and PASS count; tagged-suite exclusions listed |
| M-5 QA triggers | §4.1 all absent; `qaProgram` dispatch gated |
| M-6 refactor without tests | §4.1 characterization tests first |
| M-7 policy in `backup`; fingerprint unchecked | §3 item 1 |
| M-8 copy inventory | §3 item 1 |
| M-9 M15 seeds | dropped |
| M-10 one PR | §5 two PRs |
| N-1 `:290` | §4.2 cites `:287` |
| N-2 `signKeyHash` | §4.1 |
| N-3 stale comment | §4.1 |

### R0 round 2

| Finding | Fold |
|---|---|
| I-1 emulator walk cannot pass; nothing counts reads | §4.2 `deliveredCount`, emu `refugium` platform twin, named end screen, headless Playwright run, default walk first |
| I-2 ELF gate never run; markers may be inlined | §4.5: measured with TinyGo 0.41.1, inlined symbols replaced by measured markers, CI keeps ELFs |
| I-3 NFC-off untested on the device side | §4.2 two layers: gui `startScanner` and `nfcAvailable()` gates with a counting-reader test, plus `Poller.Read` in the ELF absence set |
| M-1 comment hits; `""` trigger trap | §4.5 AST string literals; §4.1 `proofTriggersEnabled` |
| M-2 controller pair constraint; `go list` form | §4.1 guard rule; §4.5 command |
| M-3 unkeyed scan offers | §4.2 floor list |
| M-4 Engrave Text QR; `validateMdmkStrings` kind | §4.4 |
| M-5 ms-codec source | §0 and §3: git rev `e5dff6f` |
| M-6 fingerprint derivation wipes | §3 item 1 |
| M-7 field-off call and fault screen | §4.2 `Close()`, fault flag, `gui.Run` screen |
| M-8 tagged emulator CI | §4.5 |
| Nits (untyped nil, emu twin, `pass:` whole-payload refusal) | §4.2, §4.3 |
