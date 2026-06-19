# T6a-2 (GUI) Implementation Plan — single-sig flagship on-device flow

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development or executing-plans. `- [ ]` checkboxes; strict TDD (fail → run-fail → impl → run-pass → commit per task).

**Goal:** A new `engraveSingleSig` device program: typed seed → pick wallet type → derive ms1+mk1+md1 (using the shipped T6a-1 headless core) → engrave (full or watch-only) → verify-bundle → watch-only restore doc.

**Architecture:** Reuses the shipped headless core (`md.EncodeSingleSig`, `codex32.EncodeMS1`, `md.WalletPolicyIDStubChunks`, `bundle.Verify`) + T4's `seedEntryFlow`/`deriveAccountXpub` + T5's `bundleEngrave`. Net-new = the program + lockstep, the single-sig picker (BIP-84 default + Advanced), the orchestrator flow, the ms1 engrave card, the watch-only verify path, and a direct-`bip380.Descriptor` restore-doc screen.

**Tech stack:** Go/TinyGo (host tests via `/home/bcg/.local/go/bin/go`). Module `seedhammer.com`.

**Spec:** `design/SPEC_seedhammer_T6a_singlesig_flagship.md` (Phase B; GREEN @ R1 + picker refinement). **Recon (the authoritative surface map @ `bfff857`):** `design/agent-reports/seedhammer-T6a2-gui-recon.md`. **T6a-1 (shipped headless core):** fork `main` `bfff857`.

## Locked decisions (spec + recon)
- New program **between `engraveBundle` and `qaProgram`** (`qaProgram` stays debug-only/non-navigable); all navigable bounds currently on `engraveBundle` move to the new top.
- Typed seed entry ONLY (`seedEntryFlow`); NEVER `act.scan` for the seed (the `scan.go` bip39/codex32 footgun). Scan IS reused for read-back mk1/md1 in verify.
- Picker: **BIP-84 default** (one-tap) + the other 3 (pkh/sh-wpkh/tr) behind "Advanced"; 4 single-sig types only; **mainnet-only**.
- mk1 stub = `md.WalletPolicyIDStubChunks(md1)` (POLICY-BOUND, non-zero); DROP T4's `stubZeroWarning`. `EncodeSingleSig.fp` = the account `masterFP` (not `xpub.ParentFingerprint()`).
- **T6 ENGRAVES the derived ms1** (full mode) — the device's purpose; append a `cardMS1` kind. Watch-only = mk1+md1 + the ms1 reminder.
- **Watch-only verify** skips the ms1 leg (extend `bundle.Verify` for empty-MS1-on-both).
- Restore doc: build `*bip380.Descriptor` DIRECTLY (BIP-49→`P2SH_P2WPKH`); BYPASS the classifier (it drops sh-wpkh). Display via `address.Receive/Change` + a plain screen (NOT `DescriptorScreen` — the 0-alloc gate).
- Per-leg scrub (D11); `TestAllocs` stays green.

---

## Task 0: Worktree + baseline
- [ ] **Step 1:** Ensure `bfff857` is reachable first — `git -C /scratch/code/shibboleth/seedhammer fetch origin` (the `third_party/seedhammer` submodule tracks UPSTREAM `713aee2`, which lacks the fork's T4/T5/T6a-1 surface; the work happens in the `bg002h` fork checkout, where `bfff857` is HEAD). Then `git worktree add ../seedhammer-wt-t6a2 -b feat/t6a2-gui bfff857` (sibling-dir; sandbox-fallback `git checkout -b` in place + say so).
- [ ] **Step 2:** Baseline `/home/bcg/.local/go/bin/go test ./gui/... ./md/... ./codex32/... ./bundle/... ./mk/... ./bip380/... ./address/...` → all pass; else BLOCKED.

---

## Task 1: `engraveSingleSig` program + 8-site lockstep

**Files:** Modify `gui/gui.go`; Test `gui/singlesig_program_test.go` (+ update `gui/bundle_program_test.go`/`gui/derive_xpub_program_test.go` wrap bounds).

- [ ] **Step 1: Failing test** (mirror `bundle_program_test.go`): nav reaches `engraveSingleSig`, asserts navigable + wrap-correct + a NON-BLANK title; `qaProgram` stays out of the carousel; `TestAllocs` still green.
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** — insert `engraveSingleSig` in the enum BETWEEN `engraveBundle` and `qaProgram` (`gui/gui.go:147-152` → `…engraveBundle=2, engraveSingleSig=3, qaProgram=4`). Update ALL bounds keyed on `engraveBundle`: left-wrap (`:1634`), right-wrap (`:1641`), `npage` (`:1840`), `npages` (`:1859`) → `engraveSingleSig`. Add the dispatch case (`:1490-1500`) → `engraveSingleSigFlow` (Task 7), the title case (`:1659-1666`, non-blank e.g. "Engrave Single-Sig"), the `layoutMainPlates` arm (`:1848-1856`). Update BOTH existing nav-tests' wrap upper bound from `engraveBundle` to `engraveSingleSig`.
- [ ] **Step 4: Run → PASS** + `TestAllocs` green (re-run after the enum change).
- [ ] **Step 5: Commit** — `gui: engraveSingleSig program + 8-site lockstep (between engraveBundle and qaProgram) (T6a-2)`.

---

## Task 2: single-sig wallet-type picker (BIP-84 default + Advanced)

**Files:** Create `gui/singlesig_pick.go`; Test `gui/singlesig_pick_test.go`.

`singleSigPickFlow(ctx, th) (purpose int, script md.ScriptKind, ok bool)` — first screen offers **BIP-84 (native segwit)** + an **"Advanced…"** entry; "Advanced" → a second `ChoiceScreen` of {BIP-44 legacy (pkh), BIP-49 nested (sh-wpkh), BIP-86 taproot (tr)}. Map purpose→`md.ScriptKind`: 44→`ScriptPkh`, 49→`ScriptShWpkh`, 84→`ScriptWpkh`, 86→`ScriptTr`. Build the `bip32.Path` `{purpose|hardened, 0|hardened, 0|hardened}` (mainnet coin-type 0', `hardened=0x80000000`). NO network axis (mainnet-only).

- [ ] **Step 1: Failing tests.** Default screen → selecting the first entry yields BIP-84/`ScriptWpkh`/`m/84'/0'/0'`; "Advanced" → each of the 3 yields the correct purpose+ScriptKind+path; Back from Advanced returns to the default screen; Back from default → `ok=false`.
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `gui/singlesig_pick.go` — **R0-m3:** define a NEW local/unexported single-sig table (BIP-84 first, then 44/49/86) in this file; do NOT mutate or index-couple to the shared package-level `var scriptTypePurpose` (`gui/derive_xpub.go:32-42`, order load-bearing for the 6-entry picker). Build the Advanced submenu via chained `ChoiceScreen.Choose`.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: single-sig wallet-type picker (BIP-84 default + Advanced) (T6a-2)`.

---

## Task 3: derive-3-legs + bound-stub core

**Files:** Create `gui/singlesig_derive.go`; Test `gui/singlesig_derive_test.go`.

`deriveSingleSigBundle(m bip39.Mnemonic, passphrase string, net *chaincfg.Params, path bip32.Path, script md.ScriptKind) (b bundle.Bundle, masterFP uint32, parentFP uint32, xpub string, err error)` (a small result struct is fine): (1) `xpub, masterFP := deriveAccountXpub(...)`; (2) decode xpub via `hdkeychain.NewKeyFromString` → `(chainCode [32]byte, compressedPubkey [33]byte)` AND **`parentFP := key.ParentFingerprint()`** (R0-I1 — capture it in this SAME decode for Task 6; pattern `mk/encode.go:117,161-164`); `fp4 := masterFP→[4]byte` (`binary.BigEndian.PutUint32`); (3) `md1 := md.EncodeSingleSig(chainCode, compressedPubkey, fp4, originComponents(path), script)`; (4) `stub := md.WalletPolicyIDStubChunks(md1)`; (5) `mk1 := mk.Encode(mk.Card{Network, Path: path.String(), Fingerprint: hex(masterFP), Stubs: [][4]byte{stub}, Xpub: xpub})` — **bound stub, NO `stubZeroWarning`**; (6) `ms1 := codex32.EncodeMS1(m.Entropy())` (gate validity first). Return `bundle.Bundle{MS1, MK1: mk1, MD1: md1}` + masterFP + **parentFP + xpub** (the latter two for Task 6's canonical restore-doc descriptor). **Scrub** entropy + any secret copies on all exit paths.

- [ ] **Step 1: Failing tests.** For the abandon-test seed at m/84'/0'/0': **R0-m1 — decode mk1 via `mk.Decode` and assert the DECODED FIELDS (network/path/fingerprint/xpub) match T4's known card AND the stub == `WalletPolicyIDStubChunks(md1)` (NON-zero, NOT `[0,0,0,0]`)** — do NOT assert raw-string-equality vs T4's golden (the bound stub changes the bytes). md1 round-trips to the wpkh wallet-policy; ms1 decodes to the seed entropy; `bundle.Verify(b, b) == nil` (self-consistent, incl. the stub binding). Repeat for sh-wpkh (stub bound; `EncodeSingleSig` sh-wpkh shape). `EncodeSingleSig.fp` uses masterFP (assert the md1's embedded fp == masterFP, not the xpub parent fp). Also assert the returned `parentFP` is the real non-zero parent fingerprint (for Task 6).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `gui/singlesig_derive.go` (+ the xpub→bytes + originComponents helpers).
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: derive single-sig ms1+mk1+md1 with policy-bound mk1 stub (T6a-2)`.

---

## Task 4: engrave (full = ms1+mk1+md1; watch-only = mk1+md1) — the `cardMS1` addition

**Files:** Modify `gui/bundle.go` (+`cardMS1`), `gui/bundle_flow.go`; Test `gui/singlesig_engrave_test.go`.

- [ ] **Step 1: Failing tests.** Full mode → `bundleEngrave` sequences 3 cards (ms1, mk1, md1) "Card X of 3", each engraving its verbatim strings via `validateMdmk`; NO ms1-reminder (the device engraved it). Watch-only mode → 2 cards (mk1, md1) + the ms1 reminder shown. Assert engraved strings == derived strings (verbatim). **R0-m2: include a 24-word seed** (ms1 = 75 chars) to prove the LONGEST ms1 engraves (fits a plate via `validateMdmk`) and does NOT trip the whole-bundle abort (`bundle_flow.go:331-337`). T5's existing `bundleFlow`/gather (which never produces `cardMS1`) is unaffected (assert T5's tests pass verbatim).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** — append `cardMS1` to `bundleCardKind` (`gui/bundle.go:21-27`, after `cardMD1`; T5 gather never produces it, so its classify/foreign logic is untouched). `bundleEngrave` engraves any card's `strings` via `validateMdmk` (already format-agnostic — confirm `bundlePlatePlan` + the per-card label handle `cardMS1`). **R0-I2: gate the end-of-engrave `bundleMs1ReminderText` on a `cards`-DERIVED signal — `any(card.kind == cardMS1)` over the `cards` slice INSIDE `bundleEngrave` (suppress iff an ms1 card was engraved). Do NOT add a parameter / change the `bundleEngrave` signature** (T5's call site `bundle_flow.go:36` must stay byte-unchanged; T5 gather never produces `cardMS1` → its reminder still shows). Keep `bundleMs1ReminderText()` defined (`TestBundleEngraveMs1Reminder` calls it directly). (Per-leg secret note: the ms1 string is secret; engraved onto owner-held steel only — never NFC; accept the immutable-string residual.)
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: engrave the derived ms1 (cardMS1) — full + watch-only modes (T6a-2)`.

---

## Task 5: verify-bundle flow (+ watch-only `bundle.Verify` extension)

**Files:** Modify `bundle/verify.go` (watch-only skip); Create `gui/singlesig_verify.go`; Test `bundle/verify_test.go` + `gui/singlesig_verify_test.go`.

- [ ] **Step 1: Failing tests.** **Headless (bundle):** `bundle.Verify` with `MS1==""` on BOTH bundles SKIPS the ms1 leg and verifies mk1+md1+stub-binding only (watch-only); with MS1 present on both → full comparison (unchanged); MS1 present on one side only → error ("ms1 presence mismatch"). **GUI:** re-type seed → re-derive (`deriveSingleSigBundle`) → **R0-C1: read back mk1/md1 over NFC via the T5 `bundleGatherer`** (`gui/bundle.go`, csid-keyed, yields `bundleCard.strings` for both kinds, handles the first/next-chunk priming, refuses ms1) — pull `cardMK1.strings`/`cardMD1.strings`; ms1 HAND-TYPED (`inputCodex32Flow`→`codex32.DecodeMS1`) → assemble the readback `bundle.Bundle{MS1, MK1, MD1}` → `bundle.Verify` → PASS/FAIL (`showError`). **Do NOT use `mk1GatherFlow`/`md1GatherFlow`/`.collected()`** — `mk1GatherFlow` returns a decoded `mk.Card` (not `[]string`), `md1GatherFlow` returns only `bool` (discards the strings), and `.collected()` is private to the gatherer (`gui/mk1_inspect.go:156`, `gui/md1_gather.go:72`). Watch-only verify omits the ms1 read (both MS1 empty).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** — extend `bundle.Verify` (empty-MS1-both → skip ms1 leg; one-sided → error; mk1+md1+stub-binding legs run regardless). Build `gui/singlesig_verify.go` (re-type seed → re-derive → read-back via the `bundleGatherer` → type ms1 → assemble `bundle.Bundle` → `bundle.Verify` → result screen). NFCReader nil in tests → drive the comparator + flow directly.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `bundle/gui: verify-bundle flow + watch-only (ms1-less) Verify path (T6a-2)`.

---

## Task 6: restore doc (direct descriptor; sh-wpkh-safe)

**Files:** Create `gui/singlesig_restore.go`; Test `gui/singlesig_restore_test.go`.

`restoreDocFlow(ctx, th, xpub string, masterFP, parentFP uint32, script md.ScriptKind, path bip32.Path)` — build `*bip380.Descriptor` DIRECTLY (Option Y, bypass the classifier): `Type=Singlesig`, one `Key{Network: &chaincfg.MainNetParams, MasterFingerprint: masterFP, ParentFingerprint: parentFP (R0-I1 — the REAL non-zero parent fp from Task 3's decode, NOT 0; else Key.String()/desc.Encode() re-serializes a non-canonical xpub that doesn't byte-match the engraved mk1), DerivationPath: path, Children: []bip380.Derivation{{Type: RangeDerivation, Index: 0, End: 1}, {Type: WildcardDerivation}} (R0-m4 — set <0;1>/* EXPLICITLY, matching useSiteToChildren md1_expand.go:144-147; don't rely on the empty-default), KeyData: xpub[32:65], ChainCode: xpub[0:32]}`, `Script`: 44→`P2PKH`, 49→`P2SH_P2WPKH`, 84→`P2WPKH`, 86→`P2TR`. Display (NOT `DescriptorScreen` — alloc gate): master fp + the descriptor + first receive (`address.Receive(desc,0)`) + first change (`address.Change(desc,0)`). Display-only, NO secret; optional NFC export.

- [ ] **Step 1: Failing tests.** For each of the 4 scripts (esp. **sh-wpkh**), the built descriptor's `address.Receive(desc,0)`/`Change(desc,0)` match the expected addresses for the abandon-test xpub (BIP-84 receive #0 = the known `bc1q…` vector); **R0-I1: assert `desc.Encode()` (the `Key.String()` xpub) BYTE-MATCHES the engraved mk1 card's xpub** for the abandon-test seed (a real golden — proves the parentFP is threaded; address-match alone would hide a dropped parentFP); the screen shows fp + descriptor + addrs; greps clean of any xprv. **sh-wpkh works** (the classifier would have dropped it).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `gui/singlesig_restore.go` (direct descriptor build + a plain display screen).
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: watch-only restore doc — direct *bip380.Descriptor (sh-wpkh-safe), display-only (T6a-2)`.

---

## Task 7: orchestrator `engraveSingleSigFlow` (stitch + full/watch-only + scrub + typed-only)

**Files:** Create/extend `gui/singlesig.go`; Test `gui/singlesig_flow_test.go`.

`engraveSingleSigFlow(ctx, th)`: `seedEntryFlow` (TYPED only) → `singleSigPickFlow` → optional passphrase → **full vs watch-only** ChoiceScreen → `deriveSingleSigBundle` (returns the bundle + masterFP + **parentFP + xpub**) → engrave (Task 4; full=3 cards, watch-only=2+reminder) → offer verify-bundle (Task 5) → `restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path)` (Task 6 — thread the captured parentFP+xpub so the restore-doc xpub is canonical, R0-I1). Per-leg scrub (D11): gate `m.Entropy()` validity; `defer wipeBytes` the entropy; scrub the mnemonic `[]Word` after the last derivation; the seed/master/intermediates are scrubbed inside `deriveAccountXpub`; restore-doc is public (masterFP/parentFP/xpub carry no secret).

- [ ] **Step 1: Failing tests** (drive via `runUI`+`click`/`runes`; NFCReader nil): the flow reaches engrave with 3 cards (full) or 2 (watch-only); **D12: a structural test that `engraveSingleSigFlow` uses `seedEntryFlow` and NEVER routes a scanned object to derivation** + a behavioral test that a scanned bip39/codex32 can't reach the derive entrypoint; the mnemonic/entropy buffers are zeroed on all exit paths (incl. abort).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `engraveSingleSigFlow` + wire it into the Task-1 dispatch case.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: engraveSingleSigFlow orchestrator (typed-only seed, full/watch-only, per-leg scrub) (T6a-2)`.

---

## Task 8: No-regression + fuzz
- [ ] **Step 1:** `/home/bcg/.local/go/bin/go test -count=1 ./...` + `TestAllocs` green; `go vet ./gui/... ./bundle/...` clean (vs baseline); `gofmt -l` empty. T4 `deriveXpubFlow`/`backupWalletFlow`, T5 `bundleFlow`/`bundleEngrave` (public gather path), single-card flows + codecs byte-unchanged (their tests pass verbatim).
- [ ] **Step 2: Fuzz** the watch-only `bundle.Verify` path + the restore-doc descriptor build (no panic). ≥1M execs.
- [ ] **Step 3: Run → 0 panics.**
- [ ] **Step 4: Commit** — `gui/bundle: no-regression + fuzz for the T6a-2 single-sig flow (T6a-2)`.

---

## Acceptance (GREEN bar for the exec review)
- `engraveSingleSig` program reachable (between engraveBundle/qaProgram, non-blank title, no panic, nav-tests updated), `TestAllocs` green (Task 1).
- BIP-84-default + Advanced picker → correct path/ScriptKind for all 4 (Task 2).
- Derive → mk1 with the POLICY-BOUND stub (not stub-0), md1 wallet-policy, ms1; `bundle.Verify` self-consistent (Task 3).
- Full mode engraves ms1+mk1+md1 (verbatim); watch-only mk1+md1 + ms1 reminder (Task 4).
- verify-bundle PASS/FAIL incl. watch-only (ms1-less) (Task 5).
- restore doc addresses correct for all 4 incl. sh-wpkh; display-only, no secret (Task 6).
- Typed-only seed (D12) + per-leg scrub (D11) proven (Task 7).
- Full suite + `TestAllocs` green; T4/T5/single-card flows byte-unchanged; fuzz 0 panics (Task 8).

## Self-review (author, post-plan-R0-fold)
- Spec Phase-B coverage: program→T1; picker→T2; derive+bound-stub→T3; engrave (+ms1 card)→T4; verify→T5; restore→T6; orchestrator+scrub+typed-only→T7; no-regression→T8. ✓
- Recon risk-locks: enum-insertion (T1), bound-stub-wiring (T3), ms1-card gap (T4), watch-only-verify (T5), sh-wpkh-direct-descriptor (T6), 0-alloc/no-DescriptorScreen (T6/T1). ✓
- **Plan-R0 folds (1C/2I/5m):** **C1** verify read-back uses the T5 `bundleGatherer`→`bundleCard.strings` (NOT `mk1GatherFlow`/`md1GatherFlow`/`.collected()`, which don't yield `[]string`) → T5 (+ recon Topic 6 corrected); **I1** capture the real `parentFP` in Task 3's xpub decode + thread to Task 6 + assert the restore-doc xpub byte-matches the engraved mk1 → T3/T6/T7; **I2** the ms1-reminder gate is `cards`-derived inside `bundleEngrave` (no signature change, T5 byte-unchanged) → T4; **m1** mk1 assertion compares decoded fields + bound stub (not raw-string vs T4) → T3; **m2** include a 24-word seed (longest ms1 fits) → T4; **m3** local single-sig table (not the shared `var`) → T2; **m4** explicit `<0;1>/*` Children → T6; Task-0 fork-fetch prerequisite added. ✓
- The only headless touch is the small `bundle.Verify` watch-only extension (Task 5) — integration, tested headlessly. Everything else is GUI. ✓

## Gate
This plan MUST pass opus R0 to 0C/0I before code; fold → persist → re-dispatch until GREEN. Then single-implementer TDD in the worktree → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h. Then T6b (multisig/miniscript via supplied md1).
