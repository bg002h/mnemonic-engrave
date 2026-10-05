# R0 review, round 2: IMPLEMENTATION_PLAN_fork_refugium_F5_F7.md

- **Reviewer:** independent adversarial architect (opus), R0 gate, round 2
- **Date:** 2026-10-05
- **Plan revision:** bdc87b3 (draft 2)
- **Round 1:** `refugium-F5-F7-plan-R0.md` (0C/8I/10M/3N, at 972665e)
- **Code baseline:** bg002h/seedhammer main be00ef8 (read-only)
- **Counts:** 0 Critical / 3 Important / 8 Minor / 3 Nits
- **Verdict:** NOT GREEN. Two of the Importants are gates that cannot run as written (the emulator walk) or have still not run (the TinyGo ELF). The third is a missing test of the one property F7 exists for.

Scope: (1) whether each round-1 finding is folded; (2) whether the fold added defects, D3 above all; (3) remaining Critical or Important gaps. Settled facts were not re-derived.

Things I executed:
- `go list` on the controller with host tags.
- `nix develop … -c tinygo version`, which failed.
- An ms-codec (crates.io 0.9.0) vector generator, compared against Go `EncodeMS1`. To run the Go side I put a scratch `_test.go` file in `codex32/` and deleted it straight after. `git status` is clean.

---

## Fold table check (round 1, finding by finding)

| R1 | Folded? | Note |
|---|---|---|
| I-1 | **Partly** | The source-level file-set check and the ELF symbol/data-literal check are both specified. Item I-1(c), "run the TinyGo control **before the plan closes** and choose markers by measurement", is still deferred to the implementer. The plan's reason it can be run here ("nix is installed here") is false today. See **I-2**. A "contains" scan also trips on comments; see **M-1**. |
| I-2 | Yes | D3 removes the reader-fault cases. The field-off write at boot is specified; see M-7 for the driver detail and the failure-screen mechanism. |
| I-3 | Yes | D3 (no denylist). |
| I-4 | Yes, but only at the platform | See **I-3**: nothing in gui or on the device tests it. |
| I-5 | **Partly** | The three `FeatureNFC` sites are cited correctly. Scan offers that are not keyed on `FeatureNFC` exist, and the plan's claim about the emulator is wrong. See **M-3**. |
| I-6 | **No (regressed)** | Round 1 said `presentedCount` is not a read counter and asked for a delivered or fetch counter. Draft 2 dropped that and relies on `presentedCount`/`assertNoNFC`, which fail on any presented tag. See **I-1**. Round 1's tagged-wasm CI vet was also dropped. |
| I-7 | Yes | Notices, the SLIP-39 stop, `("", false)` with a check at each call site, the program hidden, and `pass:` refused. Detail in N-3. |
| I-8a | **Mostly** | The Engrave Text program is still an ms1-string-QR producer. See **M-4**. |
| I-8b | Yes, with an inconsistency | Rust generation works (I ran it). The provenance wording contradicts the pin. See **M-5**. |
| M-1 to M-10, N-1 to N-3 | Yes | M-2's new guard wording conflicts with the controller pair (**M-2**). M-7's fingerprint check adds secret copies the plan does not list (**M-6**). |

---

## Important

### I-1. The §4.2 emulator walk cannot pass as written. `presentedCount`/`assertNoNFC` count *presentations*, not reads, so "present a tag at each step and assert no tag is read" fails in both builds. The positive control cannot tell "read" from "presented". The plan also gives no way to run the walk. (Round-1 I-6 regressed.)

**Evidence.**

- `cmd/emu/nfc.go:94-104`: `set()` does `n.queue = append(...)` then `n.presentedCount++`. The count goes up when the page presents a record, whether or not anything reads it. The doc comment says the same: "how many records the PAGE has offered".
- `cmd/emu/walk_build_policy.js:270-283`: `assertNoNFC` throws when `shNFC.presented() !== 0`. A walk that "presents a tag at each step" therefore throws at its first assertion, in the Refugium build and the default build alike. If the emulator twin returns nil via `detached` (`nfc.go:130-137`), the queue is never drained, but `presented()` still rises.
- Nothing counts deliveries. `Read` (`nfc.go:145-162`) dequeues without a counter. So the default-build positive control ("reads the tag") has no observable either.
- There is no headless runner. Walks are browser-console scripts (`walk_verify.js:5-6`: `const w = await import(...)`). This box has `node` but no Chromium or Playwright. `mnemonic-engrave/scripts/gen-tx-journey.sh:15,196` says framebuffer capture "needs a WASM build and playwright" and belongs to a hardware session. Under CLAUDE.md, "a plan may not close while any of its own gates has never been run". This gate has not run, and the plan does not say who runs it or how.
- §4.2's walk ends "to the power-off prompt". No such screen exists in gui (`grep -i "power off"` finds only a comment in `sysw_unload.go:20`).

**Fix.**

- Add a `deliveredCount` to `nfcSource`, incremented in `Read` when a record is taken off the queue (`n.cur, n.queue = n.queue[0], …`). Expose it as `shNFC.delivered()`, with a host test in the untagged `nfc_test.go`, which is host-testable.
- Refugium walk: present a tag at each step and assert `presented() > 0` (non-vacuity) and `delivered() === 0` at each step and at the end. Default walk: assert `delivered() > 0`.
- Name the walk's end screen (for example, the start screen after the engrave screen returns).
- Name who runs the walk and how, with the console output saved under `design/agent-reports/`. For example, Brian in a browser, or Playwright if the toolchain is approved. Run it once on the default build **before this plan closes**, to show that the instrument works.
- Restore round-1's CI line: `GOOS=js GOARCH=wasm go vet -tags refugium ./cmd/emu/` beside the existing `test.yml:123`.

### I-2. The device ELF gate has still never run, and it cannot run locally. `nix develop` fails here, so "the implementer runs it locally" cannot happen as stated. Round-1 I-1(c) is still open. (Gate never executed.)

**Evidence.**

- `PATH=/nix/var/nix/profiles/default/bin:$PATH nix develop /home/claude/seedhammer -c tinygo version` fails in about 1 s. The error is: `unable to download 'https://github.com/numtide/flake-utils/archive/11707dc….tar.gz': HTTP error 403 … GitHub access to this repository is not enabled for this session. Use add_repo to request access.`
- `flake.lock` pins three GitHub inputs: NixOS/nixpkgs, numtide/flake-utils and nix-systems/default. A direct GitHub archive fetch also returns 403. `cache.nixos.org` is reachable (`nix-cache-info` answered), so the binary substituter is not the problem; only the flake-input tarballs are.
- No `tinygo` is in `/nix/store` or on PATH.
- The CI job (`.github/workflows/test.yml:139`) builds with `-o /dev/null`, so even CI keeps no ELF to scan today.
- What is unmeasured: whether TinyGo `-opt 2` keeps `main.writeOTPValues` as a symbol. It has a single caller (`platform_sh2.go:553-555`) and may be inlined into `LockBoot`, in which case the positive control fails on its first run. Round-1 I-1 described exactly this failure for gc.

**Fix.** Before the plan closes, do one of:

- (a) request access to the three flake-input repos (add_repo for `NixOS/nixpkgs`, `numtide/flake-utils`, `nix-systems/default`), then run the default build locally with `-o fw.elf` and record `nm`; or
- (b) push a probe branch through the `ci/staging` route, with a workflow step that builds both ELFs (`-o fw-default.elf`, `-o fw-refugium.elf`), dumps `llvm-nm`/`go tool nm` plus a `strings | grep` of the data literal, and uploads them as artifacts.

Then choose the markers from that output and write the chosen names into the plan. Also state the fallback if `writeOTPValues` is inlined: assert on `otp.AddBootKey`/`otp.EnableSecureBoot`/`otp.WriteWhiteLabel*` and the `otpRedirectURL` literal (`platform_sh2.go:76`). Change the CI job to keep the ELF and run the absence/presence check.

### I-3. The property F7 exists for, NFC off on the device, has no executed test anywhere. D3 lives only in TinyGo-only controller code that no host test compiles. gui keeps no profile gate of its own. The emulator walk tests a separate emulator twin. (Lenses: funds/seed safety, spec Done-when.)

**Evidence.**

- D3 is implemented as `cmd/controller` not setting `FeatureNFC` (`platform_sh2.go:313`) and `NFCReader()` returning nil (`:572-574`). Every file there is `//go:build tinygo && rp` (`platform_sh2.go:1`, `main.go:2`), so no `go test` compiles it.
- §4.5's source check looks only for forbidden literals and OTP calls. It does not check that the Refugium `NFCReader` returns nil or that `FeatureNFC` is unset.
- The ELF check lists only OTP symbols. A Refugium twin that kept `return poller.New(p.nfc)` (a copy-paste slip) passes every gate in the plan.
- gui obeys whatever the platform says. The seven `startScanner` sites call `ctx.Platform.NFCReader()` directly, and `startScanner` gates only on `r == nil` (`gui/nfc_scan.go:56-59`). A gui-level tagged test with `testPlatform{nfc: …}` (`gui_test.go:464-484`) would read tags under `-tags refugium` today, and the plan adds nothing that would make it fail.
- The emulator walk (I-1) exercises `cmd/emu/platform.go`'s twin, a separate implementation from the controller's. A green walk therefore says nothing about the device image.

**Fix.** Enforce the property in two layers, and test both:

- (1) **gui:** under `refugiumProfile`, `startScanner` treats any reader as nil, and the `FeatureNFC` reads go through one `ctx.nfcAvailable()` that returns false under the profile. Round-1 I-5 proposed this helper. The tests:
  - a tagged gui test with a `testPlatform` that **does** supply a counting reader walks each of the seven sites and asserts zero `Read` calls;
  - the untagged control asserts reads happen;
  - a source test asserts `Features().Has(FeatureNFC)` appears only inside the helper.
- (2) **device ELF:** add to the Refugium absence set `seedhammer.com/nfc/poller.(*Poller).Read` (or `poller.New`) and `seedhammer.com/driver/st25r3916.(*Device).Detect`/`reset`. Keep `(*Device).Close`, which is needed for the field-off write. The default ELF is the positive control. Take the exact names from the I-2 measurement.

---

## Minor

- **M-1. The §4.5 "file contains literal" scan fails on comments in files that must stay in the Refugium set, and the obvious twin for a trigger constant is a trap.**
  - `PASSPROOF!` appears in comments in `gui/passphrase_flow.go:98,274,553,855`. That program is only hidden (§4.3), so the file stays.
  - `TEXTPROOF!`/`CONSTPROOF!` appear in comments in `gui/freetext_flow.go:731,796-797,1051`.
  - The constants are defined in `gui/freetext_proof.go:47,54,69` and `gui/passphrase_passproof.go:62`, and code in the shared files uses them.

  Fix:
  - Scan string literals with `go/parser` (`*ast.BasicLit`), not raw bytes, and scan identifiers for the OTP calls.
  - In the Refugium twin, disable the triggers with a `const proofTriggersEnabled = false` that the comparison checks. **Never** set the trigger to `""`: `kbd.Fragment == ""` would then fire on an empty passphrase or empty text.

- **M-2. The new refugium-pair guard (§4.1), "exactly `//go:build refugium` / `!refugium`", rejects the controller pair, which must be `tinygo && rp && refugium` / `tinygo && rp && !refugium`.** Scope the guard per package, or require "the constraint is X && refugium with an X && !refugium twin". For §4.5's controller file set, the invocation that works on the host is `go list -e -tags tinygo,rp[,refugium] -f '{{.GoFiles}}' ./cmd/controller`. I ran it at be00ef8 and got `[engraver.go main.go platform_sh2.go]`. Untagged it fails with "build constraints exclude all Go files". Write that command into the plan.

- **M-3. The §4.2 claim "operator wording is keyed on FeatureNFC" (three sites) is incomplete, and its emulator parenthetical is wrong.**
  - Scan offers not keyed on `FeatureNFC`:
    - `verify_address.go:20` (`"Scan"`/`"Type"` choice; leads to a Back-only "Scan the address QR." screen, `:99`);
    - `composer_door.go:127` (`"Scan cards"`, unconditional);
    - `sysw_session.go:237` (`syswAltScan = "SCAN CARDS"`, the decline arm of `syswChoose`);
    - `md1_gather.go:142` and `mk1_inspect.go:232` (`"Scan the next chunk."`).
  - "(the emulator before a tag source exists is that case today)" is false. `cmd/emu/platform.go:350` always reports `FeatureNFC`, and `NFCReader` is non-nil unless detached (`nfc.go:130-137`). The no-NFC model is gui's `testPlatform` with `nfc == nil` (`gui_test.go:464-484`).

  Put this list in §4.2 as the floor for the implementer's list. After I-3's fix these become `nfcAvailable()` sites.

- **M-4. §4.4 misses the Engrave Text program as an ms1-string-QR producer.** An ms1-shaped text only *warns* (`freetext_flow.go:1068` → `syswWarnMS1Shaped`, `sysw_session.go:308-317`: "NEVER A REFUSAL"), and the program offers "Add QR" (`freetext_flow.go:538`). Under the profile, ms1-shaped text should force "No QR" (or be refused), and the §4.4 test should cover it.

  Also note that the bundle producers reach the QR choices through `validateMdmkStrings` from `singlesig_engrave.go:24` and `multisig_engrave.go:36` as well as the composer. That function takes no card kind (`gui.go:2667`), so the gate has to thread the kind through or test the `ms1` HRP. Name which.

- **M-5. §0 and §3's vector provenance contradicts itself.** "`ms-codec` (crates.io), the version mnemonic-engrave pins": mnemonic-engrave pins ms-codec **by git rev** `e5dff6f…` (0.10.0, "not on crates.io and deliberately so", `crates/me-cli/Cargo.toml:73-83`, `Cargo.lock:618-620`). The crates.io maximum is 0.9.0. Pick one source and record it.

  Feasibility is confirmed. `ms-codec = "=0.9.0"`'s `encode(Tag::ENTR, &Payload::Entr(e))` gives:
  - `ms10entrsqqqq…cj9sxraq34v7f` for 16×00;
  - `ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6` for 16×7f;
  - `ms10entrsqrlll…7ydtcvhdp9ycqe` for 32×ff.

  These are byte-identical to Go `EncodeMS1` on the same inputs. The git repo is also reachable (`git ls-remote` succeeded).

- **M-6. The fingerprint check in §3 brings BIP-32 master-key derivation into `backup`, and the copy inventory does not list what that creates.** It creates the 64-byte BIP-39 seed from `MnemonicSeed` (`bip39.go:261`) and the master private key. Today `backup` imports neither (`go list` imports: engrave, fonts, passphrase, qr, std). Reuse `masterFingerprintFor`'s pattern (`gui.go:894-913`: `defer mk.Zero()`, fingerprint taken before zeroing) by moving it to a shared package rather than writing a second derivation. Add both buffers to the inventory, with how each is wiped.

- **M-7. §4.2's field-off detail.** The plan leaves the driver call and register for the implementer to cite; here they are:
  - The call is `(*st25r3916.Device).Close()`, which writes `regOpCtrl` (0x02) = 0 (`driver/st25r3916/st25r3916.go:297-301,791`). That clears `en`, `rx_en`, `tx_en` and `wu`.
  - `New` touches no register (`:88-95`). `reset()` is reached only from `Detect` (`:221`), which only the poller calls. So in the Refugium build this write is the firmware's only contact with the chip.
  - The write is still worth making: a warm reboot (`machine.CPUReset`) does not reset the ST25R3916.

  "Boot shows a non-secret screen and stops" has no mechanism today. An `Init` error ends in `fmt.Fprintf(os.Stderr…); os.Exit(2)` (`cmd/controller/main.go:18-21`) with no screen. Specify the mechanism: a platform flag that `gui.Run` turns into a fatal screen, or a direct LCD draw in `Init`. Or accept halting before `gui.Run`, which is fail-closed but blank, and say so.

- **M-8. The CI side of the tagged emulator is missing.** See I-1's last fix: §4.5 should list `GOOS=js GOARCH=wasm go vet -tags refugium ./cmd/emu/` and the host `nfc_test.go` coverage of the new delivered counter. Without these, the walk's instrument can break unnoticed between manual runs.

## Nits

- **N-1.** The Refugium `NFCReader` must return an *untyped* nil (`return nil`). `startScanner` compares the interface to nil (`nfc_scan.go:56`), and a typed-nil `*poller.Poller` would start a goroutine on a nil device.
- **N-2.** `cmd/emu` cannot see gui's unexported `refugiumProfile`. It needs its own tag pair or an exported constant. Say which, so the emulator twin and gui cannot disagree.
- **N-3.** "`pass:` record refused at load": say whether the whole payload is refused or only the record is dropped, with the operator text for each. The systemwide container is the Refugium build's only data channel, so refusing the whole payload over one record is a behaviour worth stating.

---

## D3 check (the big change): flows with no reader

On a platform without `FeatureNFC` and with a nil reader:
- `startScanner(nil)` returns a channel that never delivers (`nfc_scan.go:56-59`). Nothing blocks on it, because every site selects on it alongside UI events.
- The seed picker drops SCAN (`derive_xpub.go:287-289`).
- Transaction shows its no-payload message and returns (`transaction.go:316-330`).
- Bundle gathering switches to the no-reader wording (`bundle_flow.go:195`, `:331`).

No flow hangs. The only misleading screens are the M-3 list, which offer a scan route that leads to a Back-only screen. Those are UX defects, not stuck states.

The D3 decision itself agrees with the spec. Spec lines 572-573 and 574-578 say seeds and read-back are typed, and nothing in UI spec §4.4/§4.6.2 or the parent plan's lane F uses NFC in the Refugium build. Putting it to Brian on a decision card is right.

## Process

- No code blocks in the plan, so `plan-build-gate` does not apply.
- Rust-primary: F5 is a fork-native layout. The only codec touchpoint is the vector oracle, now Rust-generated (M-5 is about wording only).
- Two gates have never executed: the emulator walk (I-1) and the TinyGo ELF (I-2). Under CLAUDE.md the plan cannot close until both have run at least once, on the default build if nothing else.
