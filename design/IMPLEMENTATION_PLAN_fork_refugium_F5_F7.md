# IMPLEMENTATION PLAN — SeedHammer fork, Refugium F5 and F7

> **Status: DRAFT 2 (2026-10-05).** Draft 1 (972665e) was reviewed in R0 round 1
> (opus: 0C/8I/10M/3N, `design/agent-reports/refugium-F5-F7-plan-R0.md`); draft 2 folds
> every finding (fold table, §7). Biggest change: in the Refugium build NFC is off for
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
| Rust ms1 reference | `ms-codec` (crates.io), the version mnemonic-engrave pins | recorded in the vector provenance |

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
     another seed (R0 M-7);
   - encodes `qr.Encode(string(seedqr.QR(m)), qr.M)` (upstream's word-plate level),
     refuses above `seedQRMaxSize`, and reuses `engraveSeedString` unchanged.
   - **Wiping:** the entropy buffer from `m.Entropy()` and `EncodeMS1`'s internal
     payload are wiped with a helper the implementer names (gui's `wipeBytes` is not in
     `backup`; add a local one or move it, R0 M-8). The comment beside the function
     lists the copies that stay live and cannot be wiped: the SeedQR digit string, the
     `qr.Code` bitmap, the recomputed ms1 string and the fingerprint input (R0 M-8).
2. **Vectors** `backup/testdata/ms1_seedqr_vectors.json` plus `.provenance.json`.
   Rows: SeedSigner's published Standard SeedQR examples (digit streams copied
   verbatim) and BIP-39's standard English test vectors at 12 and 24 words (no M15
   rows, R0 M-9). Columns: words, entropy hex, ms1 string, SeedQR digits, QR width at
   `qr.M`. **The ms1 column is generated by Rust** (`ms-codec` from crates.io, at the
   version mnemonic-engrave pins, through a small generator committed under
   `backup/testdata/gen/` or run from mnemonic-engrave), never by the Go encoder it
   tests; the provenance names the crate, version and command (R0 I-8b). The SeedQR
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
  for tinygo pairs: each `*_refugium.go` file has exactly `//go:build refugium` and a
  twin with `//go:build !refugium`, and no `*_refugium.go` file contains any literal on
  §4.5's absent list.
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
  `ftProofTrigger*` triggers are absent from the Refugium build. The `qaProgram` enum
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

- Under the profile, `cmd/controller` does not report `gui.FeatureNFC`
  (`platform_sh2.go:313`) and `NFCReader()` returns nil. At boot it puts the
  ST25R3916 in its field-off state explicitly once (the implementer cites the
  datasheet register and value, and the driver call that writes it), so the field's
  state does not rest on a reset default. If that write fails, boot shows a non-secret
  "NFC could not be turned off. Power off." screen and stops.
- `gui` already handles a platform without NFC: every reader call goes through
  `startScanner`, which accepts a nil reader (`gui/nfc_scan.go:56`), and operator
  wording is keyed on `FeatureNFC` (`bundle_flow.go:195`, `transaction.go:316`,
  `derive_xpub.go:287`; R0 I-5). The implementer confirms each of the seven
  `startScanner` sites and the three wording sites behave as on a no-NFC platform
  (the emulator before a tag source exists is that case today) and lists any screen
  whose text still says "scan"; each such screen is fixed and tested.
- Flows whose only input for a card was NFC (single-sig and multisig verify's
  gatherers, the mk1 inspect and md1 gather screens, transaction scan) take the card
  from the systemwide payload where they already can, or say that this build takes
  cards from the payload only. The implementer lists, per flow, which; no flow may be
  left waiting on a reader that does not exist.
- **Emulator test** (parent Done-when; R0 I-6): `cmd/emu` built with `-tags refugium`
  runs a walk script from power-on through typed seed entry and engraving to the
  power-off prompt, presenting a tag at each step, and asserts with its existing
  observability (`presentedCount`, `assertNoNFC`) that no tag is ever read. A default
  build of the same walk reads the tag (positive control). `cmd/emu`'s own platform
  must honour the profile the same way (no `FeatureNFC`, nil reader), or the walk
  proves nothing; the implementer says how the emulator platform takes the tag.

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
- The passphrase engrave program is hidden from the menu, and a payload `pass:`
  record is refused at load with a message naming the record (the default build is
  unchanged).
- Tests per site under the tag.

### 4.4 No QR of an ms1 string in the Refugium build

R0 I-8a: under the profile, the producers of a QR of an ms1 string are cut to text
only: the codex32 engrave flow (`codex32_polish.go:218-251`, which uses
`EngraveSeedString`), the sealed-unlock codex32 plate (`unlock_session.go:189-246`),
bundle ms1 cards' "TEXT + QR" and "QR ONLY" choices (`gui.go:2667-2740`) and composer
secret cards (`composer_flow.go:584-647`). Each offers text only, or, where the plate
function needs a QR, the implementer adds a text-only variant to that path with a
golden. A test under the tag walks each producer and asserts no ms1-string QR is
emitted. F3/F4 later add the SeedQR plate (F5) to the seed sitting.

### 4.5 Build tests

R0 I-1 showed byte scans of switch literals fail: gc compiles short `case` strings to
integer compares. So:
- **Source level, every `go test`:** `go list -tags refugium -f '{{.GoFiles}}'` for
  `gui` and `cmd/controller` (the latter with `GOOS`/`GOARCH`/tags the TinyGo build
  uses, or by parsing build constraints directly if `go list` cannot resolve them on
  host) gives the Refugium file set; a test asserts no file in it contains
  `FOREVERLAURA!`, `lock-boot`, `PASSPROOF!`, `TEXTPROOF!`, `CONSTPROOF!`, or calls
  `LockBoot` body code, `writeOTPValues`, `AddBootKey`, `EnableSecureBoot` or a
  white-label writer. Positive control: the default file set contains each.
- **Device, TinyGo ELF:** build `./cmd/controller` with and without `-tags refugium`
  via the flake (nix is installed here; the implementer runs it locally before
  review, not only in CI). In the Refugium ELF, assert absent: symbols
  `main.writeOTPValues`, `seedhammer.com/driver/otp.AddBootKey`,
  `…otp.EnableSecureBoot`, the white-label writers, and a data-only literal such as
  `"https://seedhammer.com/doc/?d=SHII"`; in the default ELF, assert all present
  (positive control). Exact symbol names are taken from the default ELF's symbol table
  and recorded. The same check runs in `.github/workflows/test.yml`'s TinyGo job.
- **Tagged tests in CI:** `GOFLAGS=-tags=refugium` with the shard script (it does not
  forward `-tags` itself, R0 M-3). The tagged run uses an anchored test list, and a
  `-v` count of `--- PASS` lines must reach the expected number (R0 M-4). The
  implementer lists which default-suite tests cannot pass under the tag (NFC-driven,
  passphrase-driven, ms1-QR-driven) and how they are excluded (a `!refugium` constraint
  on the test file, or a skip that names the profile), so the rest of the suite runs
  tagged.

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

## 7. Fold table (R0 round 1)

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
