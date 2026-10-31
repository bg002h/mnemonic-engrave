# Open Questions, Code-vs-Spec Discrepancies & Follow-Ups

This document records concrete discrepancies between current firmware code (`seedhammer` at `abdcb1825ed3bde5e01ff0e4da33f76f08f604c2`), companion specifications/tooling (`mnemonic-engrave` at `a75719996304086e5f293aaf26812ff242a25244`), and upstream Rust constellation crates, separating **observed implementation behavior**, **normative specifications**, and **tracked follow-ups** ([FOLLOWUPS.md](../FOLLOWUPS.md)).

---

## 1. Repository & Toolchain Baseline Discrepancies

### OQ-1: Local `seedhammer` Checkout Behind `origin/main`
- **Observed state:** On 2026-10-30, `/scratch/code/shibboleth/seedhammer` (`main`) is at `be00ef8857ba6f093682e4ed89acd7c1848be59d`, which is **3 commits behind** cached `origin/main` (`abdcb1825ed3bde5e01ff0e4da33f76f08f604c2`):
  1. `d156a3e2f500`: `Refugium F5: ms1 plate with a Standard SeedQR of the words (#2)`
  2. `c0f93797fe97`: `Refugium F7: the Refugium build profile (NFC off, no OTP writer, no passphrases) (#3)`
  3. `abdcb1825ed3`: `flake: seal with the pinned patched picotool 2.3.1 (#4)`
- **Consequence:** Running builds or tests in `/scratch/code/shibboleth/seedhammer` without fast-forwarding `main` omits the entire `-tags refugium` build profile, `backup.EngraveSeedStringSeedQR`, and the `picotool seal --clear` Nix flake integration.
- **Action:** Fast-forward `/scratch/code/shibboleth/seedhammer` `main` to `origin/main` (`git merge --ff-only origin/main`) when no local work is in flight.

### OQ-2: Go Toolchain Floor (`go1.26` in `CLAUDE.md` vs `go 1.25.10` in `go.mod`)
- **Normative instruction (`mnemonic-engrave/CLAUDE.md`):** States `"Minimum Go toolchain floor is go1.26 across all repos with Go code (`seedhammer`, `mnemonic-engrave/preview`). Never downgrade below go1.26"`.
- **Observed code (`seedhammer/go.mod`, `mnemonic-engrave/preview/go.mod`):** Both files declare `go 1.25.10`, and the active workspace toolchain (`mise`) provides `go1.25.9` / `go1.25.10` because TinyGo 0.39.0's compiler pipeline caps supported Go standard library versions at `go1.25.x`.
- **Status:** Tracked in [FOLLOWUPS.md](../FOLLOWUPS.md) (`F-238`). Maintain `go 1.25.10` in `go.mod` until TinyGo supports `go1.26`.

### OQ-3: Conflicting Orchestration Policies Between `seedhammer/CLAUDE.md` and `mnemonic-engrave/CLAUDE.md`
- **Observed discrepancy:**
  - `seedhammer/CLAUDE.md` (dated `2026-06-17`) instructs: *"Default to ultracode (`uc`) for code changes"*.
  - `mnemonic-engrave/CLAUDE.md` (dated `2026-07-26`) instructs: *"Default to direct implementation... Reserve `/ultracode` for genuinely high-risk work"* and explicitly notes that `uc` burned disproportionate quota on low-risk work.
- **Reconciliation:** The 2026-07-26 risk-scoped policy in `mnemonic-engrave/CLAUDE.md` supersedes the 2026-06-17 blanket `ultracode` rule. `GEMINI.md` in `seedhammer` codifies the reconciled 2026-07-26 policy.

---

## 2. Codec & Cross-Language Parity Discrepancies

### OQ-4: Unported `md-codec` Rules in `seedhammer/md`
- **Normative rule:** `descriptor-mnemonic/crates/md-codec` (`v0.48.4` at `6cf6599a`) is the primary specification for `md1`.
- **Observed Go port gaps (`seedhammer/md`):**
  1. **Pathless shared origin (`sh_wpkh`):** `md/testdata/vectors/sh_wpkh.descriptor.json` is vendored in `md/testdata/vectors/` but excluded from `md/md_test.go` (`md/testdata/README.md`) because pathless shared-origin decoding (`[deadbeef]02...`) is not yet implemented in `seedhammer/md` ([FOLLOWUPS.md](../FOLLOWUPS.md) `F-166`).
  2. **Rendered-xpub header/origin agreement (`md-codec 0.44.0`):** `md/testdata/compose_refusal_vectors.provenance.json` notes that `md-codec 0.44.0` added a refusal check when a rendered xpub's 4-byte parent fingerprint/depth header disagrees with its `[fp/path]` origin depth (`depth == 0` with non-empty origin). Go `seedhammer/md` has not yet ported that check, so `md/testdata/compose_refusal_vectors.provenance.json` pins `b2c5d693` (`0.45.0`) while `md/testdata/compose_vectors.provenance.json` remains pinned at `430ea478`. (Note: an earlier elided-origin `WalletPolicyId` divergence, `F-212`, was resolved on 2026-08-20 via `canonicalOrigin` in `md/walletpolicyid.go:53–71`.)

### OQ-5: Stale Version Comment in `seedhammer/mk/mk.go`
- **Observed discrepancy:** `mk/mk.go:5` states `// Implements the mk-codec v0.2 wire format (family_token "mk-codec 0.2")`, whereas `mk/testdata/csid_ext_v0.1.json` and `crates/me-cli/Cargo.toml` use `"family_token": "mk-codec 0.5"`.

### OQ-6: `oracle/pins.json` Binary Pins Lag Current Constellation Repos
- **Observed state:** `oracle/pins.json` pins live CLI binaries from August 2026:
  - `md`: `5a0a4f41` (`v0.13.0`) vs `descriptor-mnemonic` `origin/main` `5dfb1fde` (`md-codec` `v0.48.4` `6cf6599a`).
  - `mk`: `a38a908e` (`v0.13.0`) vs `mnemonic-key` `origin/main` `bdb1df7f`.
  - `ms`: `d49d5c09` (`v0.16.0`) vs `mnemonic-secret` `origin/master` `85efbd42` (`ms-codec` `v0.10.0` `e5dff6f9`).
- **Consequence:** `scripts/oracle-live.sh` (`-tags oraclelive`) validates parity against the August 2026 binary pins (`F-211`) rather than current constellation `HEAD` binaries.

---

## 3. Code Comments, Documentation & Preview Drift

### OQ-7: Flash Address Typo in `seedhammer/gui/sysw_load.go` Comment
- **Observed code comment (`gui/sysw_load.go:20`):**
  ```go
  // It is completely separate from the Sealed Payload flow (0xE1000000).
  ```
- **Actual implementation (`sysw/wire.go:27` & `seal/read_tinygo.go:28`):**
  - `sysw.RegionAddr = 0x10D00000`
  - `seal.PayloadAddr = 0x10E00000` (not `0xE1000000`).

### OQ-8: Stale Documentation in `seedhammer/README.md`, `FIRMWARE-QUICKSTART.txt`, and `design/PICOTOOL_PIN.md`
- **`README.md`:** Still describes the fork as adding only 4 early features (`SLIP-39`, `Codex32`, `SEED XOR`, `BIP-85`), omitting `Wallet Policy` / composer, `Engrave Single-Sig`, `Engrave Multisig`, `Engrave Bundle` (`md1`/`mk1`), `Engrave Transaction` (`mt1`/`tx:`), `Engrave Text`, `BIP-39 Password`, `Account Xpub`, `Load Payload` (`sysw`), `Sealed Payload` (`seal`), hashlock preimage plates, and `-tags refugium` (`F-213`).
- **`FIRMWARE-QUICKSTART.txt` (line 122):** Documents building with `-ldflags="-X main.Version=seedhammerii-$(git rev-parse HEAD)"`, whereas `flake.nix` now stamps `v0.0.0-bg<short-sha>[-dirty]` (`gui/gui_test.go` / `gui/gui.go:parseVersion`).
- **`design/PICOTOOL_PIN.md` (line 121):** States *"The fork's devshell change is its own PR in the fork thread"*, which has already landed on `seedhammer` `origin/main` in commit `abdcb1825ed3bde5e01ff0e4da33f76f08f604c2`.

### OQ-9: `me-preview` Submodule Lag & Engraving Speed Mismatch
- **Observed state:** `preview/` builds against `third_party/seedhammer` (`7b6f2fbbdb909ef812ccc8c90768280c5fa7a4d4`), which points to the `bg002h/seedhammer` fork but lags `seedhammer` `origin/main` (`abdcb182`) by 24 commits. Furthermore:
  - Production `SH2` hardware (`cmd/controller/platform_sh2.go:223` and `internal/sh2/params.go`) uses `engravingSpeed = 4 * mm` (halved for deep steel cuts).
  - `preview/params.go:18` and four test files inside `seedhammer` (`engrave/engrave_test.go`, `backup/backup_test.go`, `gui/gui_test.go`, `stepper/stepper_test.go`) still define `engravingSpeed = 8 * mm` copies (`F-127`, `F-213`).
- **Consequence:** Golden `.bin` step counts in tests and `me-preview` are calibrated to `8 * mm`, while physical `SH2` cuts emit twice as many steps per millimeter (`4 * mm`), and `third_party/seedhammer` lacks the latest 24 fork commits (including Refugium F5/F7).

---

## 4. Hardware UX, Formatting & Security Follow-Ups

### OQ-10: `fadeClip` No-Op Stub & Warning Scroll Clipping (`R-E`)
- **Observed code (`gui/gui.go:465, 995`):** S6b (`F-208 / R-I`) wired touchable `arrowUp` and `arrowDown` `Clickable` buttons into `Warning.Layout`, resolving the earlier button-only scroll issue on buttonless `SH2` hardware. However, `fadeClip` (`gui/gui.go:995`) remains a no-op stub (`return o.Offset(image.Pt(0, 0))`) whose `scrollArrowDownVisible` side effect controls the down-arrow indicator (`R-E`, `F-95`, `F-207`).
- **Consequence:** Because `fadeClip` does not clip overflowing lines to `bodyClip`, any warning text that exceeds the viewport renders unclipped past the bottom edge until a real clip mask is restored in `fadeClip`.

### OQ-11: Pre-Existing `gofmt -l` Unformatted Baseline (`F-499`)
- **Observed state:** Five files in `seedhammer` (`gui/transaction.go`, `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`) carry pre-existing unformatted lines (`F-499`).
- **Guidance:** Do not run repo-wide `gofmt -w .` inside feature PRs; format only touched hunks until `F-499` lands as a dedicated formatting commit.

### OQ-12: `bip39.MnemonicSeed` Lacks NFKD Normalization (`F-76`)
- **Observed code (`bip39/bip39.go:261`):** `MnemonicSeed` passes the passphrase string directly to `pbkdf2.Key` without Unicode NFKD normalization.
- **Current safety boundary:** `passphrase.ValidatePassphrase` (`passphrase/passphrase.go`) and the on-device touch keyboard strictly enforce printable ASCII (`0x20..0x7E`), where NFKD is an identity transform. If non-ASCII passphrases are ever admitted via `sysw` or NFC, NFKD normalization will be required for BIP-39 compliance.
