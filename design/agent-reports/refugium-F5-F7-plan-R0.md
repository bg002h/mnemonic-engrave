# R0 review: IMPLEMENTATION_PLAN_fork_refugium_F5_F7.md

- **Reviewer:** independent adversarial architect (opus), R0 gate
- **Date:** 2026-10-05
- **Plan revision:** 972665e (draft 1)
- **Code baseline:** bg002h/seedhammer main be00ef8 (read-only)
- **Counts:** 0 Critical / 8 Important / 10 Minor / 3 Nits
- **Verdict:** NOT GREEN. Fold the Importants and re-review (scope: did each fold fix the finding and add no new defect).

Lenses applied, in order: (1) correctness against the code, (2) funds/seed safety, (3) spec coverage and D1-D4, (4) gates that have never run, (5) process. Facts were checked against the tree at be00ef8. One gate was prototyped: the host absence test (I-1).

---

## Important

### I-1. The host absence gate does not work as written. Its positive control fails on today's tree, because `FOREVERLAURA!` is never stored contiguously in a gc binary. The device gate has never been run. (lenses 1, 4)

**Evidence.** I built a host binary that links `gui` and keeps `gui.Run` reachable (a scratch `package main` calling `gui.Run(nil, "")` behind an `os.Args` guard, built untagged at be00ef8). Then I counted byte occurrences:

```
b'FOREVERLAURA!' 0      <- the positive control the plan specifies FAILS
b'FOREVERL'      1      (as an 8-byte little-endian immediate)
b'lock-boot'     1      (only via the log format "lock-boot: %v", gui.go:2271)
```

The gc compiler lowers a `switch` against short constant strings (`gui/gui.go:2266`, `:2268`) into length checks plus immediate integer compares. So the case literals never appear as contiguous bytes. Two consequences:

- §4.5's host test ("positive control builds the same binary untagged and asserts both are present") fails on its first run. An implementer under pressure "fixes" that by dropping the control or changing the marker. This is the vacuity path the control exists to block.
- `lock-boot` passes the control only by accident, through a log string. If the implementer reworded the log line, the absence check would pass vacuously on the case literal.

The TinyGo/LLVM device build may or may not inline the same way, so the ELF scan in §4.5 is unvalidated too. CLAUDE.md says "a plan may not close while any of its own gates has never been run". Nix is on this box (`/nix/var/nix/profiles/default/bin/nix`), and CLAUDE.md gives the exact `nix develop -c tinygo build` recipe. "Proven green in CI before review closes" therefore defers a gate that could be run now.

**Fix.**

- (a) Base the binary proof on markers the compiler cannot split:
  - symbols, via `debug/elf` / `go tool nm`. In the default ELF, `main.writeOTPValues` and `seedhammer.com/driver/otp.{AddBootKey,EnableSecureBoot,WriteWhiteLabelString,WriteWhiteLabelAddr}`, plus a distinctively named default-only debug handler, must be present (control). In the refugium ELF they must be absent.
  - long data-only literals, for example `otpRedirectURL = "https://seedhammer.com/doc/?d=SHII"` (`cmd/controller/platform_sh2.go:76`, used only by `writeOTPValues`).
- (b) Add a source-level proof that cannot be fooled by codegen. Under `-tags refugium`, list the package's build file set (`go list -tags refugium -f '{{.GoFiles}}' ./gui ./cmd/controller`, with `GOOS`/`GOARCH`/tags matching the device for controller via `go/build.Context`). Assert that no file in it contains `FOREVERLAURA!` or `lock-boot`. Positive control: the untagged set does contain them.
- (c) Before the plan closes, run the TinyGo control locally on be00ef8 (`nix develop -c tinygo build -o fw.elf … ./cmd/controller`) and record which markers survive. Choose the markers from that measurement, not from inference.

### I-2. NFC fault states: an abandoned reader from before the latch is never handled, any `Close` error leaves the field state unknown, and the "poll loop exits on the latch" mechanism does not turn anything off. (lenses 1, 2)

**Evidence.**

- `startScanner`'s stop (`gui/nfc_scan.go:127-140`) treats **any** `r.Close()` error the same way: log, return, abandon the goroutine. A timeout returns before touching the chip (`nfc/poller/poller.go:124-138`). A non-timeout error means `d.Close`'s `writeReg(regOpCtrl, 0)` itself failed (`driver/st25r3916/st25r3916.go:297-301`). Either way the field-off write did not happen.
- The plan only handles "a stop that reports a close timeout **while latched**". The dangerous ordering is the reverse: a reader is abandoned on a public screen (no latch yet, so no screen is shown), and the operator then types a seed. That secret is held with the ST25R3916 possibly still in detect/wake-up mode, and with type-4 tag emulation answering an external field (`poller.go:46,74-85`).
- §4.3 says "the poll loop … exits when [the latch] is set … so a scanner already running when a secret arrives stops without waiting for its screen to leave". The goroutine spends its time parked inside `s.Scan(r)` → `Poller.Read` → `Detect()` (`st25r3916.go:212+`), and only reaches the top of the loop when a tag or interrupt arrives. Exiting the goroutine also does not write `regOpCtrl=0`; only `Close` does. The stated mechanism does not do what it claims.
- The "implementer finds how the timeout surfaces … if it cannot surface, the plan comes back" step is already answerable. It cannot surface today: stop returns nothing, and gui deliberately does not import the poller (`nfc_scan.go:128-131`).

**Fix.** Specify this now:

- (a) stop sets a sticky `ctx.nfcFault` on **any** `Close` error;
- (b) under the profile, `noteSecret()` checks `nfcFault`, and the place that records a fault checks the latch. Either order leads to the non-secret "NFC did not turn off. Power off now." screen, before any secret-taking screen is drawn;
- (c) drop the poll-loop latch check. If a scanner must be stoppable mid-screen, `noteSecret` must call the active scanner's stop (Interrupt and Close), not wait for the goroutine to notice;
- (d) add tests: a fake reader whose `Close` errors on a public screen, followed by seed entry, must end on the power-off screen; and the reverse order must too.

### I-3. The latch is a denylist of entry points, which fails open on a missed one, and the plan's own list already misses some. (lens 2)

**Evidence.** These are my independent enumeration of secret sinks in `gui` (non-test). Entries the plan's §4.3 list does not name:

- `passphrase_flow.go:75`: the engravePassphrase program builds `NewPassphraseKeyboard` directly. It bypasses `passphraseFlow`, so neither the latch list nor §4.4's "core functions refuse" covers it.
- `composer_hashlock.go:342`: the hashlock preimage, typed on `NewPassphraseKeyboard`. A preimage is a spending secret.
- `seedxor_polish.go:40-61` (SEED XOR combine) and `bip85.go:79-126` (master seed, then child via `bip39.New`). Both enter through `inputWordsFlow`, so they are covered only if the latch sits in `inputWordsFlow` rather than in "the seed-entry flows in derive_xpub.go" as written.
- `unlock_kdf.go:146-160`: unlock passphrase words through `inputWordsFlow`. `sysw_load.go:119` likewise.
- `gui.go:1274`: typed codex32, on its own keyboard.

The grep-and-report mitigation depends on two people's diligence. The only mechanical test (§4.3) checks that `NFCReader()` goes through `nfcReader`. It does not check that every secret sink calls `noteSecret`.

**Fix.** Make the latch fail closed. Either:

- (a) **latch at program dispatch** (`gui/gui.go:~2148-2185`) for every program **not** on an explicit public allowlist (for example md1/mk1 inspect, bundle without seeds, transaction, address verify), plus at `sysw_load.go:185` when a secret class loads, plus in the scanner on any secret-class object; or
- (b) **NFC entirely off in the Refugium build.** `nfcReader()` always returns nil and the NFC capability is reported false.

Nothing in the UI spec or parent plan gives the Refugium build a use for NFC: seeds are typed (§4.4), the plan never rides a payload, and read-back is typed (§4.4 "Read-back is typed"). Option (b) satisfies "NFC off while a secret is held" by construction, removes the tag-emulation exposure, and makes I-2 and most of I-4 moot. D3 does not consider it. At minimum, record why it is rejected.

### I-4. NFC-sourced secrets are accepted under the Refugium build, which contradicts "NFC off while a secret is held" at the moment the secret arrives. (lenses 2, 3)

**Evidence.**

- §4.3 lists "the NFC seed SCAN row's result (`scanSeedFlow`)" and "start-screen scans of mnemonic, codex32, SLIP-39 or `pass:` records" as places to *call `noteSecret`*. They are therefore still **accepted**. The scanner parses the secret in its goroutine while the field is on (`gui/scan.go:72-77` passScan, `:78` `bip39.Parse`, `:89` `codex32.New`), and `engraveObjectFlow` routes it (`gui/gui.go:2590-2613`).
- The F7 recon (§2 item 6) said "The Refugium profile probably wants all of these refused". The plan dropped that without saying so.
- UI spec §4.4: "ms1 plates are made on the SeedHammer from the words typed on its touchscreen". Accepting NFC seeds is a user-visible behaviour the spec does not ask for, and the plan does not declare it.

**Fix.** Under the profile:

- the scanner drops secret classes (mnemonic, codex32 secret, SLIP-39 share, `pass:`), scrubs `buf`, and reports a refusal status ("Secrets are never read over NFC in this build");
- the seed picker's SCAN row is hidden unconditionally (not only while latched);
- tests cover each class.

If I-3(b) is adopted, this becomes automatic.

### I-5. Screens that tell the operator to scan are not gated, so the latched build directs operators to a dead reader. The plan says nothing about what Refugium verify flows now do. (lenses 1, 3)

**Evidence.** Reader-aware wording is keyed on `Features().Has(FeatureNFC)` at three sites. The plan gates only one of them (and cites it as `:290`; it is `:287`):

- `gui/bundle_flow.go:195` (`hasReader`, which drives `tally()` and `bundleDoneEmpty`, `:76-89,128-133,236-240`)
- `gui/transaction.go:316`
- `gui/derive_xpub.go:287`

With the latch set, single-sig verify (`singlesig_verify.go:145`) and multisig verify (`multisig_verify.go:781`) still say "Scan a card's chunks", but the reader is nil (`nfc_scan.go:57-59`), so the only exits are a payload or Back. The parent plan names "single-sig verify's NFC gatherer" explicitly. The resulting operator experience is a user-visible change the plan neither states nor tests.

**Fix.**

- Add `ctx.nfcAvailable()`, equal to `FeatureNFC && !(refugiumProfile && latched)` (or always false under I-3(b)), and use it at all three `Features().Has(FeatureNFC)` sites.
- Extend the source-parse test so that `Features().Has(FeatureNFC)` appears only inside `nfcAvailable`.
- State and test what single-sig and multisig verify show under the profile.

### I-6. The parent plan's "emulator test" Done-when is re-labelled, not met, and the justification is factually wrong. (lenses 3, 4)

**Evidence.** §4.3 says "`testPlatform` is the emulator's model of the platform". It is not. `testPlatform` is gui's unit-test fake (`gui/gui_test.go:~400-500`); the emulator is `cmd/emu`, with its own `platform` (`cmd/emu/platform.go:281,350`). The emulator already has NFC observability that walk scripts assert on (`cmd/emu/nfc.go` `presentedCount`; `walk_build_policy.js:258-277` `assertNoNFC`), so the hedge "if one can observe NFC state, and says if it cannot" is weaker than the tree allows. `cmd/emu/build.sh` does not forward tags; `GOFLAGS=-tags=refugium` would.

**Fix.** Either:

- (a) commit to a `cmd/emu` walk built with `-tags refugium`. Add a reader-fetch counter or a "delivered" counter beside `presentedCount`, present a tag after seed entry, and assert that it is never delivered and that the reader is never fetched again. Add CI vet of the tagged wasm build (`GOOS=js GOARCH=wasm go vet -tags refugium ./cmd/emu/`); or
- (b) amend the parent plan's F7 Done-when through the FOLLOWUPS companion, explicitly, rather than re-labelling a unit test as the emulator.

### I-7. Gating the SLIP-39 passphrase "the same way" silently recovers a different seed, an invented behaviour with a funds consequence. Skipping the BIP-39 prompts in engrave flows is also silent. (lenses 2, 3)

**Evidence.** `slip39_polish.go:280-295` documents that a wrong (here: absent) SLIP-39 passphrase "silently recovers a different valid seed". §4.4 extends D4 (a BIP-39 rule) to SLIP-39 with "gated the same way". An operator whose shares carry a passphrase would get a valid but wrong seed, and could engrave an ms1 plate of it.

Similarly, skipping the "Add a BIP-39 passphrase?" prompt in `singlesig.go:120` and `multisig.go:144` makes the engrave flows derive the no-passphrase xpub with no notice. For a passphrase wallet, that engraves md1/mk1 for a different wallet. Upstream has the same silence, but this build is choosing it, so it should be said.

The core-function refusal is also underspecified: "return the no-passphrase result". `("", true)` and `("", false)` mean different things at the call sites (`slip39_polish.go:292` treats `!ok` as abort; `multisig_verify.go:954` treats `!ok` as "no passphrase").

**Fix.**

- SLIP-39 under the profile: keep the choice screen, but "Enter passphrase" ends in an explicit stop ("This build cannot recover a passphrase-protected SLIP-39 set"). Never a silent skip. Or refuse SLIP-39 entry outright.
- BIP-39 prompts: replace the prompt with a one-line acknowledged notice ("This build uses no passphrase") rather than nothing. State it as a user-visible change.
- Pin the exact refusal return value: `("", false)`, with each caller's handling of it listed.
- Cover the engravePassphrase program and the `passScan` route explicitly. D4 says "off in the whole build", but §4.4 still lets `pass:` records be "shown or engraved", and the program's keyboard (I-3) is unguarded. Hide the program and refuse the record under the profile, or say why not.

### I-8. Two loose ends in the requirements: no owner for "never a QR of the ms1 string" in the Refugium build, and a circular source for F5's ms1 vector column. (lenses 3, 5)

**Evidence.**

- (a) UI spec §4.4: ms1 plates carry a Standard SeedQR, "never a QR of the `ms1` string". D2 leaves every existing producer of a QR-of-string ms1 plate reachable in the Refugium build:
  - `engraveCodex32` → `EngraveSeedString` (`gui/codex32_polish.go:218-251`);
  - bundle cardMS1 TEXT+QR and QR-ONLY at qr.L (`gui/gui.go:2667-2740`);
  - composer secret cards (`gui/composer_flow.go:584-647`).

  D2 says "wiring is that plan's job", but the parent plan's F3/F4 text does not assign removing these, so F9 could ship a Refugium image that engraves the forbidden plate.
- (b) F5 vectors check "`EncodeMS1(entropy)` equals the ms1 string", but the plan does not name a source for the ms1 column. If it is generated with the Go `EncodeMS1`, the check is circular. The only existing external pin for `EncodeMS1` is one all-zero vector (`codex32/msencode_test.go:10-12`). The Rust-primary rule makes `ms-codec` the authority for ms1 encoding.

**Fix.**

- (a) Add an explicit hand-off: a FOLLOWUPS entry (with companion) plus a line in §3. It says that, in the Refugium build, every ms1 plate producer either uses the new layout or is unreachable, and that F9 is gated on a test asserting this. Alternatively, gate those three producers under `refugium` in F7 now.
- (b) Generate the ms1 column with `ms-cli`/`ms-codec` (mnemonic-secret, version and SHA in `.provenance.json`), and the words↔entropy column from BIP-39's published vectors. State in the plan that no column is produced by the code under test.

---

## Minor

- **M-1.** "All eight `startScanner` sites … and the eighth the recon counted". There are **seven** (`gui.go:2236`, `bundle_flow.go:225`, `derive_xpub.go:343`, `verify_address.go:77`, `mk1_inspect.go:204`, `md1_gather.go:111`, `transaction.go:750`). The recon also says "All 8" and lists 7. Fix the count. The source-parse test, not the list, is the guard; make its positive control "finds exactly 1 call, inside nfcReader".
- **M-2.** §4.1 says `tinygo_split_test.go` "discovers every pair". It discovers pairs only by the `_tinygo.go` filename suffix (`gui/tinygo_split_test.go:118-126`), so a `refugium`/`!refugium` pair is invisible to it and needs no exception. Correct the claim. Consider adding a parallel guard for refugium pairs: the refugium file must contain neither literal, and the pair's constraints must be exactly `refugium`/`!refugium`.
- **M-3.** The shard script does **not** forward `-tags` (`scripts/gui-shard-test.sh:39,82`). `GOFLAGS=-tags=refugium` works for both `-list` and the run. Say so instead of "confirmed … else run serially".
- **M-4.** CI's `go test -tags refugium ./gui/ -run Refugium` passes vacuously if no test name contains "Refugium" (`ok … [no tests to run]`). Use an anchored list, or `-v` with a count of `--- PASS` lines that must be at least N. Also state which existing default-suite tests cannot pass under the tag and how they are excluded, so that the rest of the suite can run tagged.
- **M-5.** Decide `PASSPROOF!` now. It is a QA trigger inside the passphrase program (`gui/passphrase_passproof.go:52-65`), so it disappears with that program under I-7. The free-text program has sibling QA triggers (`TEXTPROOF!`, `CONSTPROOF!`, the `ftProofTrigger*` in `gui/preview.go:99-111`). Say whether they stay. Also note that `qaEngraveFlow` (`gui/qa.go`) stays linked through `case qaProgram` (`gui.go:2148`) even with its trigger gone. State that this is accepted or gate the dispatch.
- **M-6.** Moving the `debugCommand` arm into `handleDebugCommand` refactors default-build code that has no test: no test dispatches `FOREVERLAURA!` or `lock-boot`, and `gui/scan_test.go:78` covers only parsing. Write characterization tests first (default build: `lock-boot` calls `LockBoot` once and stays on the start screen; `FOREVERLAURA!` returns `qaProgram`), then refactor. The arm uses both `continue` and `return` inside `StartScreen.Flow`, so the function needs a result enum.
- **M-7.** F5's 12/24-word rule is Refugium policy, but D2 places it inside a general `backup` function that F-455's composer is expected to reuse. Either take an allowed-lengths parameter, or document that the function is Refugium-scoped. Also consider checking `plate.MasterFingerprint` against the words, or recording why the caller is trusted. Today it is an unchecked caller input printed on a seed plate.
- **M-8.** Extend the secret-copy inventory. `EncodeMS1`'s internal `payload` (`codex32/msencode.go:23-25`) and its returned string are LIVE copies. `backup` has no `wipeBytes` (that helper is in gui), so name the helper used to wipe `m.Entropy()`'s buffer.
- **M-9.** "M15 public signet test seeds" has no location. Cite the file and revision (and that they are public) in `.provenance.json`, or drop the row.
- **M-10.** PR shape: F5 (pure library and goldens) and F7 (profile, CI) are independent. F7's merge waits on a TinyGo CI gate and two more R0 rounds. Two PRs let F5 land without that coupling and keep each diff small, as the house rule asks for fork PRs.

## Nits

- **N-1.** `derive_xpub.go:290` should be `:287` for the `FeatureNFC` probe.
- **N-2.** §4.2 "`writeOTPValues` (`:724`)": the function starts at `:724`, and its doc comment at `:722`. Fine. Also mention that `signKeyHash` stays, because `isSecureBootEnabled` (`:770-771`) still uses it, so the implementer does not move it into the `!refugium` file.
- **N-3.** `seal/record.go:212-219` cites stale line numbers (`gui.go:1672`, `platform_sh2.go:545`; the recon noted this). The F7 commit touches those exact lines, so refresh them there.

---

## Facts verified true (no action)

- seedqr `%04d` and panic on invalid (`seedqr/seedqr.go:22-33`).
- `qr.M` word plate at `gui/gui.go:851-873`.
- `seedQRLevel`/`seedQRMaxSize` (`backup/backup.go:154-156`) and `qrScale = 3` (`:194`).
- `EngraveSeedString`/`engraveSeedString` shape (`:158-245`).
- `EncodeMS1` recipe (`codex32/msencode.go:17-31`).
- The SeedSigner README carries verbatim digit streams for 12 and 24 words (saved spec, lines 60, 342, 370, 394, …).
- **No import cycle:** `codex32` imports only std, `seedqr` imports `bip39`, `bip39` imports std plus x/crypto, and none import `backup`.
- `flake.nix:118` forwards `"$@"`.
- `LockBoot` has exactly one caller (`gui/gui.go:2270`).
- The only OTP writers are in `cmd/controller/platform_sh2.go:553-559,724-752`.
- The eight "Add a BIP-39 passphrase?" prompts are exactly as listed.
- The `qaProgram` guard is at `gui.go:268`.
- Type-4 emulation serves a writable *empty* tag (`nfc/type4/type4.go:14`), so it leaks nothing stored.

## D1-D4

- **D1** (build tag): sound, and `-X` is rightly rejected. Pair it with the I-1 proof method.
- **D2** (add a layout, replace none): sound for F5's scope, provided I-8(a) assigns an owner for the forbidden plates.
- **D3** (one-way latch): better than a counter. The simpler, fail-closed alternative, NFC off for the whole Refugium build, was not considered (I-3).
- **D4** (passphrase off in the whole build): acceptable as a superset of the spec, but incomplete (the engravePassphrase program, `pass:` records) and over-extended to SLIP-39 with an unsafe default (I-7).

## Process

- **Rust-primary:** no normative codec behaviour changes in Go. F5 is a fork-native layout and F7 is GUI/firmware; both are exempt under (b). The one Rust-primary touchpoint is the vector oracle (I-8b).
- **The plan has no executable code blocks,** so `plan-build-gate.sh` does not apply. The host absence gate was prototyped by this reviewer and fails as written (I-1).
