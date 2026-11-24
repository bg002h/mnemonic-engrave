# Codex review of the Gemini SH2 architecture submission

Date: 2026-10-30. Review scope: the four firmware architecture documents and the SeedHammer `GEMINI.md`, checked against their recorded source revisions. This report preserves the submission unchanged. Findings concern documentation and initialization guidance; they are not findings from a full firmware execution audit.

## Reviewed revisions and disposition

| Repository | Submitted commit | Source baseline |
|---|---|---|
| `bg002h/mnemonic-engrave` | `03776d704d2bf85a32f89c900b0a7368d06e0b5c` | `a75719996304086e5f293aaf26812ff242a25244` |
| `bg002h/seedhammer` | `ec4fffddcd10b36409ae6d3dba9b8354f6367af2` | `abdcb1825ed3bde5e01ff0e4da33f76f08f604c2` |

**Disposition: corrections required before adopting the initialization file or treating the guide as verified.** This architecture/initialization pass found **0 Critical, 7 Important, 5 Minor** items. The separately persisted [interface review](gemini-firmware-host-interfaces-codex-review.md) adds **12 findings: 2 P1 and 10 P2**. Its shared shell-placeholder observation is excluded from that count and belongs to I1 below.

The separation into architecture, source map, interface contracts, and open questions is useful. The two submitted commits exist on their advertised remote branches, and both SSH signatures verify against the local public key for `bg002h`. Several substantive descriptions also match the source, including the early Refugium NFC shutdown, separate flash regions, vector provenance files inspected, and the five-file formatting baseline. These positives do not resolve the defects below.

Locations below refer to the submitted files. Firmware evidence refers to `seedhammer@abdcb182`; companion evidence refers to `mnemonic-engrave@a7571999`, unless another revision is stated.

## Highest-priority interface findings

The independent interface report contains the most consequential errors in the submission:

- **H01–H02 (P1), `HOST_INTERFACES.md:199–203`:** wrong OTP register addresses, redundancy, key-validity bit meanings, page-lock details, and public-key coordinate byte order for the boot-key hash. These can lead an operator deriving a procedure from the guide to program incorrect irreversible state. The actual driver and host scripts contradict the table.
- **H03–H04, `HOST_INTERFACES.md:191–206`:** the described SRAM-range patch is not the patch in the repository; the actual signing gate requires the very endpoint the guide says was removed. The `picosign` invocation and its role in producing the signed digest are also wrong.
- **H05, `HOST_INTERFACES.md:68–71`:** `ct_len` excludes the 16-byte authentication tag; the guide's ciphertext-plus-tag slice omits it. A producer/reader implemented from that layout fails interoperability.
- **H06–H12:** incorrect NFC decoding/admission routes, ms1 profile details, preimage/phrase record grammars, host/device admission equivalence, mt1 confirmation semantics, and the host preview invocation. The full report provides exact source citations and correction directions for each.

The controller separately checked the cited implementation for H01–H05 after receiving the independent report. The remaining interface findings and targeted test results are attributed to that reviewer. The report's facts concern this source snapshot; it does not authorize a replacement OTP procedure or claim a physical hardware rehearsal.

## Important findings

### I1. The command block labeled “Verified” contains commands that do not execute as documented

**Locations:** `seedhammer/GEMINI.md:44`, `:53`; `design/firmware-architecture/HOST_INTERFACES.md:197` repeats the signing defect.

The signing example ends in `<key.pem> --no-overwrite` (or `<dummy.pem>` in the interface document). Bash parses this as input redirection from `key.pem` and output redirection to a file literally named `--no-overwrite`. The signing-key pathname and the `--no-overwrite` option are both absent from the arguments received by `picotool`. With the input file absent it fails before launching; with that file present it creates or truncates the misleading output filename. A syntax-only `bash -n` check passes, so it does not catch this defect.

The plate preview example uses `-out`. `cmd/plateview/main.go:35–46` defines `-o`, not `-out`. Executing the documented invocation with the installed Go toolchain and network dependency resolution disabled failed with `flag provided but not defined: -out` and program exit status 2.

**Correction:** use a quoted signing-key path or variable and only flags supported by the pinned tool. Verify `--no-overwrite` before retaining it: the interface reviewer found no entry for it in the installed tool's seal help, but did not execute sealing to test parser acceptance. Use the implemented preview `-o` option with an SVG or PNG filename. Validate argument construction using a stub, then validate the preview command. The signing example was checked here with a shell-function stub and synthetic input, without signing firmware or accessing a device.

### I2. The “strict safety sequence” omits automatic physical homing during startup

**Location:** `design/firmware-architecture/ARCHITECTURE.md:83–86`.

The documented sequence ends with initializing steppers, touch, and LCD. In the source, after successful power negotiation and engraver setup, `Init` schedules `go home()` at `cmd/controller/platform_sh2.go:273–281`, before LCD initialization at line 294. The closure obtains `p.Engraver(true)` and closes it; `homingEngraver.Close` calls `d.home()` at line 575. `cmd/controller/engraver.go:186–216` implements axis homing and movement to the plate origin.

Consequently, a sufficiently powered boot can initiate physical motion without a GUI confirmation. This matters when an agent follows the guide to restart or bench-test firmware. It is a missing description of existing behavior, not a proposal to change that behavior.

**Correction:** explicitly include automatic homing, its power/setup preconditions, and its concurrency with display startup in both the initialization sequence and the relevant hardware guidance. Distinguish peripheral configuration from motion.

### I3. The reconciled workflow drops parts of the standing risk set and mandatory gates

**Locations:** `seedhammer/GEMINI.md:70–72`; `design/firmware-architecture/OPEN_QUESTIONS.md:22–26`.

The newer risk-scoped policy really does supersede blanket orchestration, but the replacement summary is narrower than that policy. `mnemonic-engrave/CLAUDE.md:11–19` explicitly includes anything touching funds, seeds, keys, or addresses; normative wire, identity, validation, and admission changes; irreversible actions; and cross-repository work beyond a single context. It also specifies R0 before implementation, independent execution review, persistence, and the 0C/0I gate. `GEMINI.md` instead tells agents to implement mechanical ports and localized UI changes directly, and lists a smaller set of exceptions. An address or seed UI change, or a single-repository admission change, can fall through that summary despite being in the actual risk set.

OQ-3 additionally presents paraphrases as quotations and attributes a quota rationale absent from the cited text; the actual text says cost is not the constraint. The initialization file does not explicitly require loading the existing `CLAUDE.md` guidance.

**Correction:** instruct Gemini to read the applicable existing guidance, accurately preserve the full risk set and gate obligations, and remove invented quotations. Do not reinstate blanket multi-agent work for ordinary low-risk tasks.

### I4. The toolchain discrepancy conflates the host compiler with TinyGo’s module-directive constraint

**Location:** `design/firmware-architecture/OPEN_QUESTIONS.md:17–20`.

OQ-2 says the active toolchain is Go 1.25.9/1.25.10 because TinyGo 0.39.0 has a ceiling, and cites F-238. At the recorded firmware revision, `flake.nix:50–63` pins **TinyGo 0.41.1**, `.github/workflows/test.yml:12` selects host **Go 1.26**, and host tests use `testing.ArtifactDir` (`gui/op/draw_test.go:176`). The companion `CLAUDE.md:42–46` records host Go 1.26.7 and flake Go 1.26.3.

The relevant explanation is **F-494**, at `design/FOLLOWUPS.md:16616–16638`: retain the `go 1.25.10` module directive because the pinned TinyGo checks it, while running host tests with Go 1.26. F-238, at line 10155, concerns a transaction season worked example and is unrelated. The recommendation to preserve the directive is reasonable; the explanation and compiler guidance are not.

**Correction:** document the host compiler and module-directive requirement separately, name the actual TinyGo pin, and cite the closing explanation in F-494. Do not downgrade host tests or raise the module directive on the basis of OQ-2.

### I5. The host application map names nonexistent binaries and assigns mr-gui to the wrong repository

**Locations:** `design/firmware-architecture/SOURCE_MAP.md:22–24`; `ARCHITECTURE.md:24`; related host references in `HOST_INTERFACES.md:12`, `:54` and `seedhammer/GEMINI.md:8`.

The source map describes toolkit as `mtk`, the utility GUI as a Tauri application named `mg-gui`, and `mnemonic-refugium` as the repository containing `mr-gui`. The manifests show:

| Repository / manifest | Actual boundary |
|---|---|
| `mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml:16,32–72` | Binary `mnemonic`; linked ms/mk/md codecs. The claimed general ms/mk/md/mt/me orchestration needs separate evidence. |
| `mnemonic-gui/Cargo.toml:30,39,47–48` | `mnemonic-gui` and `gui-render`; eframe/egui, not Tauri. |
| `mnemonic-refugium/crates/mr-cli/Cargo.toml:17` | Binary `mr`, alongside the mr codec/library work. |
| `refugium-wallet/crates/mr-gui/Cargo.toml:2,21,25` | Actual `mr-gui` application; Slint. This repository is missing from the submitted inventory. |

The first three manifest checks use the exact revisions recorded in `SOURCE_MAP.md`; the omitted `refugium-wallet` manifest was checked at `f081c7bea12c9328c118575b8fe0ecdfcf054997`.

**Correction:** fix ownership and executable/framework names together across the suite. This is a repository-boundary check, not an expansion into reviewing the Refugium implementation.

### I6. The initialization file cannot reach its companion documentation from the delivered worktree

**Locations:** `seedhammer/GEMINI.md:12–16`; `design/firmware-architecture/ARCHITECTURE.md:204`.

From `/scratch/code/shibboleth/sh-worktrees/gemini-init`, every `../mnemonic-engrave/...` link resolves under `sh-worktrees/mnemonic-engrave`, which does not exist. This breaks all four primary guide links, the follow-up ledger, and two companion runbooks. The architecture guide's `../../../seedhammer/docs/custom-firmware.md` similarly resolves under `me-worktrees/seedhammer`, which does not exist. An inline Markdown link check found **eight missing local targets** across the five files. These repository-crossing relative links also provide no usable standalone GitHub fallback.

**Correction:** identify companion repository roots explicitly for the supported sibling/worktree layouts and provide repository-qualified remote links, preferably pinned to the reviewed documentation revision. Check navigation from both delivered worktrees and from standalone repository views.

### I7. OQ-9 confuses speed-dependent timing samples with physical motor steps per millimeter

**Location:** `design/firmware-architecture/OPEN_QUESTIONS.md:66–70`.

The claim that physical cuts emit twice as many motor steps per millimeter at `4 * mm` as at `8 * mm` is incorrect. Spatial calibration is independently fixed at **6,400 microsteps/mm** in `internal/sh2/params.go:20–29` and `cmd/controller/platform_sh2.go:195–223`. `stepper/stepper.go:41–77` emits timing words on each tick but asserts a step pin only when the coordinate changes. More timing words do not imply a different distance-per-step calibration.

The source comment at `platform_sh2.go:211–221` explains the intended change: the needle's fixed 25 ms period gives denser dots at lower feed speed; the four test-local speeds deliberately remain at 8 mm/s to preserve goldens. The preview/submodule lag is a real, separate concern.

**Correction:** distinguish physical microsteps, sampled timing/PIO stream length, and needle dot spacing. Do not describe the speed split as a doubled spatial calibration or prescribe mechanical calibration changes from it.

## Minor findings

| ID | Location | Evidence and correction |
|---|---|---|
| M1 | `SOURCE_MAP.md:42,95` | `cmd/picosign`, `picobin`, and `uf2` are labeled fork-only. All three exist in the stated upstream baseline `ea4b65be7137f268fc5e97e06d0fda4996233fbb`, and `git diff` for those directories against `abdcb182` is empty. Mark them upstream; separate `oracle` from that shared provenance row. |
| M2 | `OPEN_QUESTIONS.md:84` | F-76 is the payload-card Inspect/engrave-routing issue (`FOLLOWUPS.md:1219`), not the NFKD issue. Correct or remove the attribution. This does not refute the observed direct passphrase input to `bip39.MnemonicSeed`. |
| M3 | `ARCHITECTURE.md:95` | The WASM output is `emu.wasm`, produced by `cmd/emu/build.sh:13`, not `sh2.wasm`. |
| M4 | `SOURCE_MAP.md:116`; `GEMINI.md:22`; `ARCHITECTURE.md:199` | “Every change must land in Rust first” omits the established exceptions. `mnemonic-engrave/CLAUDE.md:28` permits Go-only convergence fixes after checking the Rust counterpart, and firmware-native work with no Rust counterpart. Preserve those exceptions while retaining Rust-first normative changes. |
| M5 | `ARCHITECTURE.md:85` | “Within 500 ms” is not the negotiation deadline. `adjustSupplyVoltage`, `platform_sh2.go:459–479`, has three attempts with 100 ms settling and 500 ms retry sleeps, plus I/O time; errors can also terminate early. Describe the retry behavior rather than a nonexistent fixed deadline. |

## Verification and limits

- Confirmed exact worktree commits, clean submitted worktrees, remote branch tips via `git ls-remote`, and both SSH commit signatures using a temporary allowed-signers file and the public key. No signing configuration was changed.
- Read the affected implementation paths and manifests; compared upstream provenance with Git; checked local document link targets; checked the quoted ledger references and process rules.
- Executed the documented preview invocation with the installed Go toolchain, `GOTOOLCHAIN=local` and `GOPROXY=off`; recorded its flag failure. Reproduced signing-example argument loss and output redirection using only a stub and synthetic file. No firmware signing, flashing, OTP operation, or physical motion was performed.
- The independent interface pass is preserved separately, including its own verification limits. It overlaps I1's signing example; count that defect once when folding reports.
- Full GUI, TinyGo, hardware, and cross-language regression suites were not run for this documentation review. The external Gemini cache-desynchronization notes in global `GEMINI.md` and `papercuts.md` were outside the reviewed five-file submission.

## Suggested correction order

1. Correct the OTP/signing facts and interface wire contracts, then the executable examples, startup-motion description, and initialization workflow/toolchain guidance.
2. Repair repository ownership and navigation, and fold the remaining independently reviewed interface findings.
3. Correct speed/provenance/ledger descriptions and verify the final commands, local/remote navigation, source citations, and interface field tables against the same explicit source revisions.

Keep the original submission and review reports intact; put corrections in a subsequent commit so the response to each finding remains reviewable.
