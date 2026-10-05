# Recon F7: Refugium build profile (seedhammer @ be00ef8)

## 1. Build, tags, CI, toolchain

- **Firmware build:** `flake.nix:93-127` `build-firmware` runs `tinygo build -o firmware.uf2 -ldflags="-X main.Version=$VERSION" ${tinygo-flags} "$@" ./cmd/controller` (line 118). The flags are at `flake.nix:80`: `-target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks`. It then runs picotool seal plus `cmd/picosign sign -clear`. **`"$@"` already passes extra flags through**, so `nix run .#build-firmware -- -tags refugium` works with no flake change (inferred: tinygo accepts `-tags`). `flash-firmware` at `flake.nix:142` has the same shape.
- **Build tags are already in use.** The closest precedent is the `debug` tag:
  - `gui/debug.go:1` `//go:build debug` → `const debug = true`
  - `gui/nodebug.go:1` `//go:build !debug` → `const debug = false`
  - `cmd/controller/debug_sh2.go:1` `//go:build tinygo && rp && debug`
  
  Other tags in use: tinygo/!tinygo hook pairs in gui (`plate_hook*.go`, `frame_hook*.go`, `engraved_hook*.go`, `composer_state_hook*.go`), which are policed by `gui/tinygo_split_test.go:15-140`. That test discovers every `//go:build` pair in gui and expects each to be an interface hook (`:216`). **A new `refugium`/`!refugium` pair in gui could trip it**, so check it before adding the files. There are also `oraclelive` (`gui/multisig_build_oracle_live_test.go:1`, `oracle/live_test.go:1`) and `js` (`cmd/emu/*_js.go`).
- **Choice of mechanism:** use a build tag. `-ldflags -X` can only set string vars, so the code and literals stay in the binary and absence can't be proven. A separate cmd would duplicate `cmd/controller` (inferred). The tag pattern would be `gui/refugium.go` (`const refugium = true`) plus `gui/norefugium.go`, with the debug-command arms and `LockBoot` moved into `!refugium` files. A `const` bool guarding an `if` would let the compiler dead-strip the literal (inferred, not verified for TinyGo). Separate files are the provable form.
- **CI** (`.github/workflows/`):
  - `image.yml:20` runs `nix run .#build-firmware`.
  - `test.yml:100` runs `CGO_ENABLED=0 go test -timeout 20m ./...`.
  - `test.yml:128` compiles the oraclelive-tagged tests (`go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/ ./cmd/emu/`). This is the precedent for "a tag-gated build must be compiled in CI".
  - `test.yml:137` runs `scripts/test-32bit.sh`; `test.yml:148` runs `GOOS=js GOARCH=wasm go vet ./cmd/emu/`.
  - `test.yml:149-164` is the `tinygo-device-build` job: `nix develop --command tinygo build ... -o /dev/null ... ./cmd/controller`. A refugium variant goes here, with `-tags refugium -o fw.elf` and a byte scan of the ELF. **Scan the ELF, not the .uf2:** a .uf2 splits the payload into 256-byte chunks inside 512-byte blocks, so a literal can straddle a block boundary and a scan would miss it (inferred).
- **Toolchain in this container:**
  - `which tinygo` finds nothing, and `/nix/store` has no tinygo or picotool. `nix` exists (`/nix/var/nix/profiles/default/bin/nix`), so `nix develop` would have to download the toolchain (not tried).
  - Host Go is 1.25.10.
  - **`cmd/controller` does not build on the host:** `go build ./cmd/controller` fails with "build constraints exclude all Go files" (`main.go`/`platform_sh2.go` are `tinygo && rp`).
  - `cmd/emu/confinement_test.go:89-110` records why `go list -deps ./cmd/controller` cannot prove absence either: with no tags it prints nothing, and with `-tags tinygo,rp` the graph is partial because `machine` and `device/rp` are not in the Go stdlib.
- **Host-side proof of absence (inferred design):** both literals live in **package gui** (`gui/gui.go:2266,2268`) plus the scanner's `"command: "` prefix (`gui/scan.go:58`). Any host binary that links gui and reaches `StartScreen.Flow` (e.g. a tiny `package main` test helper calling `gui.Run`/`NewContext`, or `GOOS=js GOARCH=wasm go build -tags refugium ./cmd/emu`) can therefore be built from a `_test.go` with `exec go build -tags refugium` and `bytes.Contains`-scanned for `FOREVERLAURA!` and `lock-boot`.
  - Pair it with a positive control: the same build without the tag must contain both strings, otherwise the test passes vacuously. That is house style; see the "INCONCLUSIVE" checks in `confinement_test.go:41`.
  - `LockBoot`/`writeOTPValues` themselves live in `cmd/controller` (`platform_sh2.go:553,724`), so only TinyGo can prove they are gone. That proof belongs in the CI tinygo job; `confinement_test.go:107-110` says the same for its own blob scan.

## 2. NFC start/stop, interface, secret-held flows

- **Platform interface:** `gui/gui.go:3608-3638`: `LockBoot() error` (`:3609`), `NFCReader() io.ReadCloser` (`:3613`), `Features()`. `FeatureNFC` (`:3641-3660`) is a capability bit, used so a screen can decide whether to show a SCAN row without consuming a tag (`:3646`).
- **Controller:**
  - `cmd/controller/platform_sh2.go:572-574` `NFCReader()` returns `poller.New(p.nfc)`. This does no chip I/O (`nfc/poller/poller.go:43-50`).
  - `platform_sh2.go:306-313` creates the device and always sets `FeatureNFC`.
  - `st25r3916.New` touches no registers (`driver/st25r3916/st25r3916.go:88-96`). The chip is reset and armed only inside `Detect()` (`:212-224`), which `Poller.Read` calls (`poller.go:66`).
  - `Device.Close()` writes `regOpCtrl=0`, i.e. field and oscillator off (`st25r3916.go:297-301`).
  - **Caveat for (a):** when the close times out, `Poller.Close` returns `ErrCloseTimeout` **without touching the chip** (`poller.go:124-138`). `startScanner`'s stop function then only logs and abandons the goroutine (`gui/nfc_scan.go` stop closure, after line ~120). The chip can therefore still be active after "stop". A Refugium latch has to treat an abandoned reader as a fault (e.g. refuse to proceed or hard reset). (inferred)
  - The poller also runs **tag emulation** (`type4.NewTag`, `poller.go:46,84-86`) whenever an external field is present. Even reader-only use exposes an emulated tag to a phone.
- **Single NFC choke point:** `startScanner(ctx, r)` in `gui/nfc_scan.go:56`. It is the only poll loop (F-126 consolidation, `:12-31`); a nil reader produces a channel that never delivers (`:57-59`). All 8 call sites pass `ctx.Platform.NFCReader()`:
  - `gui/gui.go:2236` (StartScreen.Flow)
  - `gui/bundle_flow.go:225` (bundleGatherFlowResume)
  - `gui/derive_xpub.go:343` (scanSeedFlow)
  - `gui/verify_address.go:77` (scanAddressFlow)
  - `gui/mk1_inspect.go:204` (mk1GatherFlow)
  - `gui/md1_gather.go:111` (md1GatherFlow)
  - `gui/transaction.go:750` (transactionGatherFlow)
  
  The argument is evaluated before `startScanner` runs. On the emulator, `NFCReader()` consumes a pending tag (`cmd/emu/nfc.go:30`, `gui/derive_xpub.go:262-266`). A latch should therefore wrap the reader acquisition (e.g. a `ctx.nfcReader()` helper) and the `Features().Has(FeatureNFC)` SCAN-row probe (`derive_xpub.go:290`), not only `startScanner` itself (inferred).
- **Latch home:** `type Context` (`gui/gui.go:64-93`) already carries per-session secret state: `sysw *syswSession` (`:66`) and `wipe *wipeGuard` (`:86`, bracketed by `unlockSecretSession`, `gui/wipe_guard.go:1-60`, `gui/unlock_session.go:81`). The wipeGuard is scoped to a session, not sticky. "Seed entry until power-off" needs a new sticky bool on Context that nothing clears (inferred). There is **no existing "secret held" flag.**
- **Flows where a secret is in memory while NFC runs today:**
  1. **Single-sig verify:** `gui/singlesig_verify.go:102` re-types the seed (`reMnemonic`, zeroed by defer at `:104-108`), then the passphrase prompt (`:116-120`), then `bundleGatherFlow` at `:145`, so NFC runs with seed and passphrase held. The outer `engraveSingleSigFlow` also still holds the original `mnemonic` (defer-zeroed at `gui/singlesig.go:169-172`, and the verify call is at `:291`).
  2. **Multisig engrave verify:** the seed comes in at `gui/multisig.go:127`, defer-zeroed at `:136-139`, and verify is called at `:348` → `multisigVerifyFlow` → `bundleGatherFlow` at `gui/multisig_verify.go:781`. The seed is held throughout. The re-typed seed and passphrase come after the gather (`:921-954`).
  3. **Multisig build verify:** `seedRegistry` is created at `gui/multisig_build.go:308` with `defer reg.scrub()` at `:309`. Seeds are entered via `buildSeedForSlot` at `:331`/`:756-758`, and verify runs at `:561` → `multisig_verify.go:781`, with seeds held. The cosigner gather at `:202` runs before the seeds are entered, so it is clean.
  4. **StartScreen after a payload unlock or load:** the StartScreen scanner (`gui.go:2236`) runs whenever `ctx.sysw` holds secret classes (ClassMnemonic, Codex32Secret, Passphrase) from a loaded payload (`gui/derive_xpub.go:284` reads `ctx.sysw.has(sysw.ClassMnemonic)`).
  5. **After any secret flow returns to StartScreen:** buffers are defer-zeroed, but under "until power-off" semantics NFC must stay off from then on (inferred requirement).
  6. **NFC used as a secret source** (NFC on before the secret exists; it becomes the secret's origin):
     - `scanSeedFlow` via the "SCAN" row of the seed picker (`derive_xpub.go:289-310`).
     - At the start screen, scanned `bip39.Mnemonic` / `codex32.String` / SLIP-39 share / `passScan` objects are dispatched by `engraveObjectFlow` (`gui/gui.go:2590-2613`; `passScan` → `engravePassphraseFlowFrom(..., srcNFC)` at `:2613`).
     
     The Refugium profile probably wants all of these refused, since the "seed entry" itself would be NFC.
- **Flows that are clean (public data, no secret yet):**
  - `supplyMultisigPolicyFlow` gathers before the seed (`gui/multisig.go:85-102`, commented "BEFORE any seed is typed").
  - `bundleFlow` (`bundle_flow.go:24-41`) and wallet policy (`wallet_policy.go:131`).
  - Transaction (`transaction.go:300-311`; mt1 is public).
  - mk1/md1 gathers from `mdmkFlow` (`gui.go:2773,2785`).
  - `verifyAddressFlow` from `descriptorFlow` (`gui.go:3270`).
  
  These are clean only if the sticky latch is not set, i.e. no earlier secret this power cycle and no sysw secrets loaded.
- **Flows with secrets but no NFC:** bip85 (`gui/bip85.go:274,324`), codex32 input (`gui.go:1274`, keyboard only), composer/hashlock (`composer_hashlock.go:342` uses the PassphraseKeyboard for the preimage), and the passphrase program. None of them call `startScanner`.
- **How the GUI tests drive NFC:**
  - `testPlatform` (`gui/gui_test.go:~400-500`) has an `nfc func() io.ReadCloser` field. `Features()` returns `FeatureNFC` iff `p.nfc != nil` (`:464-468`); `NFCReader()` returns `p.nfc()` or nil (`:479-484`); `LockBoot()` is at `:471`.
  - Fakes include `fakeNFC` (`gui/payload_door_walk_test.go:130`, `gui/run_reentry_test.go:358`) and `nfcTag(...)` (`gui/sysw_source_test.go:36`).
  - Scanner tests are `gui/nfc_scan_test.go` and `gui/nfc_scan_abandon_test.go`. `gui/scan_test.go:78` exercises `debugCommand`.
  - So a "secret held ⇒ `p.nfc` never called" test can count calls on the `p.nfc` func (inferred).

## 3. Debug-command dispatch and LockBoot call graph

- **Parse:** `gui/scan.go:58-61`: an NDEF payload starting with `"command: "` becomes `debugCommand{rest}` (type at `scan.go:138-140`). It is classified before every sniffer, so it is reachable from any tag.
- **Dispatch:** only in `StartScreen.Flow`, `gui/gui.go:2264-2277`, on the start-screen scanner (`:2236`). It handles exactly two commands, and anything else logs `"unknown debug command"` (`:2276`):
  - `"FOREVERLAURA!"` → `qaProgram` (`:2266-2267`) → `qaEngraveFlow(ctx)` (`gui.go:2148`, defined in `gui/qa.go:14`).
  - `"lock-boot"` → `ctx.Platform.LockBoot()` (`:2268-2274`).
  
  No other flow handles `debugCommand`; in other gathers it is just an unrecognised object (inferred from the grep: `debugCommand` appears only in `scan.go` and `gui.go:2264`).
- **`qaProgram` enum** (`gui/gui.go:254`) is load-bearing: there is a compile-time guard `var _ [1]struct{} = [qaProgram - unlockPayload]struct{}{}` (`:268`), and it is referenced in program tests. Keep the enum; under the tag, drop only the trigger arm and the `case qaProgram` dispatch, or `qa.go`.
- **LockBoot call graph** (sole caller is `gui/gui.go:2270`):
  - `cmd/controller/platform_sh2.go:553` `LockBoot` → `writeOTPValues()` (`:724-752`):
    - `otp.WriteWhiteLabelAddr` (`:729`)
    - 8× `otp.WriteWhiteLabelString` (`:746`)
    - `otp.AddBootKey(signKeyHash)` (`:750`)
  - then `otp.EnableSecureBoot()` (`:557`), then `machine.CPUReset()` (`:559`).
  
  Other implementations: `cmd/emu/platform.go:357` (refuses) and `gui/gui_test.go:471`. The OTP functions are defined in `driver/otp/otp.go:93` (EnableSecureBoot), `:102` (AddBootKey), `:172`/`:204` (white label). `boardVersion()` (`platform_sh2.go:755`) also reads OTP (read only) and is used by `HardwareVersion` (`:565`), so it stays.
- **`passphrase_passproof.go:52`:** only a comment. The `PASSPROOF!` keyboard trigger (`ppPassProofTrigger`, `:65`) "mirrors the NFC debugCommand precedent". It is a typed-literal QA pattern inside the passphrase program, which becomes moot if passphrase entry is removed (§4).
- **`seal/record.go:212-219`:** a comment explaining that `cmdPrefix` mirrors `gui/scan.go`'s, because a decrypted plaintext `"command: lock-boot"` would reach LockBoot. **Its line references are stale:** it cites `gui.go:1672` (now `:2268-2274`) and `platform_sh2.go:545` (now `:553`). The literal `"lock-boot"` also appears as test data in `seal/record_test.go`, `seal/open_test.go` and `seal/engraveable_test.go`. Those are test files and are not linked into firmware.

## 4. Passphrase entry points

- **Core:** `passphraseFlow` / `passphraseFlowTitled` (`gui/gui.go:918,927`), plus `NewPassphraseKeyboard` (`gui/passphrase_keyboard.go:80`).
- **Payload-or-keyboard wrapper:** `syswPassphraseFlow` / `syswPassphraseFlowTitled` (`gui/sysw_source.go:86,98`). It offers a sysw ClassPassphrase first (`:103`) and falls back to the keyboard (`:112`).
- **"Add a BIP-39 passphrase?" prompts (seed sittings):**
  - `gui/gui.go:2818-2820` (backupWalletFlow → `passphraseFlow`)
  - `gui/derive_xpub.go:426-432` (→ syswPassphraseFlow)
  - `gui/singlesig.go:120,132` (→ syswPassphraseFlow)
  - `gui/singlesig_verify.go:116-118` (→ passphraseFlow)
  - `gui/multisig.go:144-150` (→ syswPassphraseFlow)
  - `gui/multisig_verify.go:921-954` (→ passphraseFlow)
  - `gui/multisig_build_slots.go:886-902` (→ syswPassphraseFlowTitled)
  - `gui/bip85.go:324` (master passphrase)
- **Other entry points:**
  - SLIP-39 passphrase: `gui/slip39_polish.go:285-291` (→ passphraseFlow). It is not BIP-39, but it is the same keyboard and also a secret.
  - Passphrase engrave program: `engravePassphrase` → `engravePassphraseFlow` (`gui/gui.go:2179`, `passphrase_flow.go:641`); NFC `passScan` → `engravePassphraseFlowFrom` (`gui.go:2613`); `engravePassphraseFlowPreloaded` from `gui/singlesig.go:466`; keyboard at `passphrase_flow.go:75`.
  - sysw admission table grants ClassPassphrase to these programs: `gui/sysw_admit.go:34,36,45,46,69,85`.
- **Not BIP-39:** `unlockPassphraseFlow` (`gui/unlock_kdf.go:109`, payload KDF password), `NewAddressKeyboard` (`passphrase_keyboard.go:206`), and the hashlock preimage (`composer_hashlock.go:342`).
- **Making it unreachable (inferred design):**
  - Add one `const allowBIP39Passphrase` from a tagged file pair.
  - Gate each "Add a BIP-39 passphrase?" `ChoiceScreen`: skip it and keep `passphrase := ""`.
  - Gate `syswPassphraseFlowTitled` and `passphraseFlowTitled` themselves, returning `("", false)` defensively.
  - Hide the `engravePassphrase` carousel entry and the `passScan` dispatch.
  - Remove ClassPassphrase from `sysw_admit`.
  
  Gating at the two core functions alone would still show the prompt. Gating at the prompts alone would leave the core reachable from `slip39_polish` and the engrave program, so do both. A test could assert, under the tag, that no `ChoiceScreen` ever carries the lead "Add a BIP-39 passphrase?" by walking each seed program.

## 5. Test infrastructure

- **`go test ./gui/`** runs single-core: the package has zero `t.Parallel()`, and its runtime grew from 440.6s to 496.8s in one phase (`.github/workflows/test.yml:60-75`). CI runs it serially with `-timeout 20m` (`:100`).
- **Shard script:** `/home/claude/mnemonic-engrave/scripts/gui-shard-test.sh <pkg> <shards> <timeout>` (defaults `./gui/ 6 20m`, lines ~27-30). It enumerates tests from `go test -list` and asserts the partition covers every test before running. Running Refugium-tagged gui tests means passing `-tags refugium` through; check whether the script forwards extra go flags (not verified).
- **cmd/emu:**
  - Built with `GOOS=js GOARCH=wasm` (`cmd/emu/build.sh`); CI only vets it (`test.yml:148`).
  - On the host it builds a `main_notjs.go` stub.
  - Its platform reports `FeatureNFC` unconditionally (`cmd/emu/platform.go:351`) and refuses `LockBoot` (`:357`).
  - Host tests in cmd/emu include the confinement tests, a precedent for source-level "must not be in the firmware" gates (`cmd/emu/confinement_test.go`, `embed_confinement_test.go`).
