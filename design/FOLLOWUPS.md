# FOLLOWUPS — mnemonic-engrave

Low/nit items deferred from architect reviews (per the iterative-architect-review standard: Critical/Important fixed inline; low/nit recorded here). Promote to a cycle when convenient.

## Open

> These are **cycle-sized** items (bigger than architect-review nits) — each warrants its own brainstorm → spec → plan → R0 → implement pass when picked up.

- **`seedhammer-fontproof-test-pattern`** — **Owning phase: immediately after Phase D merges, BEFORE O1. User design 2026-08-03.** O1 needs a plate carrying every engraved glyph, but typing 95 printable-ASCII characters in codepoint order across four keyboard pages is several hundred taps, and a mistype yields both a wrong plate and a false legibility reading. The NFC `debugCommand` route (`FOREVERLAURA!`, `gui/scan.go:59`, `gui/gui.go:1627`) is the existing precedent for a hidden test path, but **NFC is not yet available to this user**, so: **typing the literal constant `FONTPROOF!` into ANY of the three fields of the passphrase program offers to populate ALL THREE at once** (user design, refined 2026-08-03):

| Field | Populated with |
|---|---|
| Passphrase | all 95 printable ASCII, `0x20`–`0x7E`, codepoint order |
| Seed FP | `DEADBEEF` → renders `DEAD BEEF` |
| Expected Comb FP | `CAFEBABE` → renders `CAFE BABE` |

**Design constraints, all load-bearing:** (a) **it must ASK, never populate silently**, and the prompt must state that it replaces **all three fields** — triggering from a fingerprint field clobbers a passphrase already typed, so "load the test pattern?" is not honest wording; (b) **the collision risk exists only in the passphrase field.** `FONTPROOF!` is not valid hex, so `ValidateFingerprint` already rejects it and no legitimate fingerprint value can be displaced — but any string can be a real *passphrase*, including this one, so the prompt's "no" branch must proceed with `FONTPROOF!` as a genuine passphrase. The prompt is therefore **required** in the passphrase field and **consistent** in the other two; (c) checked **on OK/advance, not per keystroke**, and only when the constant is the **entire** field; (d) after accepting, stay on the current screen so the user sees what was loaded before advancing; (e) **scoped to these three fields in this program only** — the fingerprint fields use `NewAddressKeyboard`, which is shared with BIP-85 index entry and address verification, so the check lives in the passphrase flow's own field handlers and **never** in `PassphraseKeyboard`; (f) all three values are **fixed firmware constants**, never content from an external source. **Note the flow order** — entry → seed FP → comb FP — so triggering from the last field sets a passphrase the user has already stepped past; that is fine, since the confirm screen reveals everything before anything is cut. **Arithmetic, stated so the two counts reconcile:**  | | | |---|---| | Alphabet | **96** runes = `0x1F` (mark) + 95 printable (`0x20`–`0x7E`) | | Typed | all **95** printable, one each | | Engraved *as themselves* | **94** — `0x21`–`0x7E`; the 95th, `0x20`, is substituted | | Substituted | `0x20` → `0x1F`, **1** mark | | **Text block total** | 94 + 1 = **95 distinct glyphs** = 96 − 1 | | Missing from the text block | `0x20` itself — never cut, because it was substituted away | | Supplied by the bands | a **real `0x20`**, in `DEAD BEEF` and the footer's word gaps | | **Plate total** | **96** — every alphabet rune represented |  So "94" counts printable characters engraved as themselves, and "95" counts distinct glyphs in the text block. They are consistent: 94 + mark = 95 = 96 − 1. The fingerprints are what close the last one. Also add a line to the public guide, since this is a device behaviour with a footgun: if your passphrase is literally `FONTPROOF!`, answer **no**. Alternative considered and available if production firmware should carry nothing: a `-tags fontproof` build flashed only for the O1 run. Cross-ref spec §9.1 (the O1 procedure) and [[seedhammer-engraving-font-swap]], which would need this repeated.

- **`seedhammer-plate-carries-only-secret-prefix`** — **RESOLVED 2026-08-04** (fork `main`) — needed a call-site seam (`passphrasePlateHook`, nil in production, mirroring `freetextPlateHook`), because no unit test of `ppBuildPlate` can catch a defect in its caller. Mutating `secret[:n]` → `secret` now fails with the leaked residue printed. — **Owning phase: the next engraving cycle (test-only; no product defect).** Found by the FONTPROOF! execution review 2026-08-04 (M2). `ppFontProofLoader` sets `n = 95` into a 100-byte buffer, so when a longer passphrase was typed first, `secret[95:100]` retains a **printable** tail of it. **The shipped code is correct at every site** (`ppBuildPlate(…, secret[:n], …)`), and the hazard is not new — backspacing 100 characters down to 10 leaves the same residual; FONTPROOF merely adds a second route and is the only one that shrinks `n` without a keystroke. What is missing is a guard: mutating the call to pass the whole buffer left `go test -run TestFontProof` **green**, and the full-suite kill came only from an unrelated test panicking on `\x00` — an accident that would NOT fire in the case FONTPROOF uniquely creates (a printable residual). The confirm *screen* is guarded semantically by its counts line; the *plate* path is not. **Fix:** a flow-level assertion — build the plate from the flow's own buffer after loading the pattern over a longer passphrase, and require it to equal the plate built from the 95-byte reference. A unit test of `ppBuildPlate` cannot catch this, because the defect would be in the CALLER.

- **`seedhammer-bootkey-24word-backup`** — **Owning phase: before any further OTP work, and worth doing while both the key and the machine are known-good.** From the Gangleri fork map (`design/RECON_gangleri_fork_feature_map.md`): their `sh2key` can encode the secp256k1 boot key as **24 BIP-39 words** for a steel plate and restore it, with a single/double-word repair search against the key fingerprint, plus a second plate explaining the words are **not** a wallet seed. **We have no backup path at all.** Measured 2026-08-04: exactly one copy of `~/.sh2/sh2-boot-key.pem` exists (223 bytes, sha256 `cd3f86b3214d1b43…`), nowhere else on disk. Losing it is **not** catastrophic — the device keeps booting, and slot 0 still recovers to official v1.4.3 — but the key in slot 1 becomes permanently unusable and re-keying spends **one of the two remaining OTP slots**. The encoding is a plain secp256k1 scalar ↔ BIP-39 mnemonic with no BIP-32 derivation, and **our existing typed-word entry flow can already engrave the result**, so this needs no NFC and no new dependency. Take the idea and a small auditable encode/decode; do **not** port the 253KB TUI. Cross-ref [[sh2-boot-key-burned]].

- **`seedhammer-fontproof-guide-line`** — **Owning phase: the doc pass, before any public release of firmware carrying this.** The FONTPROOF! design called for "a line in the public guide" naming the footgun (if your passphrase is literally `FONTPROOF!`, answer no). **There is nowhere to put it:** neither `README.md` nor `docs/custom-firmware.md` mentions the passphrase program at all, and the fork has no user guide. The implementer correctly declined to invent a doc and put the warning on the prompt instead — *"Back = no: continue with FONTPROOF! exactly as typed. Any text can be a real passphrase, including this one."* — which is the only moment it actually matters. So this is **not** a safety gap; it is a documentation gap that opens only when a user-facing guide for the passphrase program exists. Write it then, or decide the on-screen warning is sufficient and close this.

- **`seedhammer-passphrase-space-underscore-compensating-swap`** — **Owning phase: O1 (hardware legibility), UX only. Found by the whole-feature fable review 2026-08-03 (M1).** The confirm screen renders a space as `_`, so intending `a_b c` but typing `a b_c` renders identically (`a_b_c`), counts identically ("5 chars, 1 space"), and shows the same legend — two different wallets behind one screen. **Every SINGLE substitution error IS caught** (it changes the space count), so this is only the compensating double-error case, and the plate itself disambiguates after the fact because the space mark and `_` are distinct glyphs. Within the spec's accepted bitmap-font limitation, hence Minor. Cheap hardening if O1's plates read badly: name the space POSITIONS on the confirm screen ("1 space at 4") rather than only the count.

- **`seedhammer-passphrase-qr-quiet-zone`** — **Owning phase: O1 (must be judged on the real plate). Found by the whole-feature fable review 2026-08-03 (M2).** In the worst case (dim 37 QR, legend + footer present) the quiet zone is ~2 mm above the envelope and ~1.75 mm below — under the ISO 4-module (4 mm at this pitch) convention. Precedent cuts both ways: the existing seed plates are equally tight and do scan. **O1 Plate A is exactly this worst case and already mandates a scan**, so the data arrives for free — but if that scan fails, **attribute it to the quiet zone before blaming module size**, since shrinking modules would make it worse. No code change unless O1 says so.

- **`seedhammer-fuzzconstantqr-never-reaches-ecc-l-dim37`** — **RESOLVED 2026-08-04** (fork `main`) — the control-flow half was already fixed in `2c16a6f`; the seeds half was not, so a plain `go test` never drove ECC-L past dim 33. Three `f.Add` seeds at 79/90/106 bytes now reach dim 37 on the L path. — **Owning phase: any later engraving cycle. Found by the Phase A execution review 2026-08-03 (M2).** `FuzzConstantQR` (`engrave/engrave_test.go:434,443-445`) raised its entropy cap to 120 bytes specifically so the fuzzer would exercise dim 37 — but with entropy ≥ 61 bytes the **ECC-Q** encode lands at dim ≥ 41, `ConstantQR` errors, and `if qrcq.Size > 37 { return }` bails **before the ECC-L half of the target runs**. ECC-L only reaches dim 37 at n ≥ 79. So the 61–120 byte band contributes one `qr.Encode` and an early return, and **ECC-L — which this feature pins — is never fuzzed above dim 33**. Measured capacity table: `n=47 → dimQ 37/dimL 29`, `n=60 → 37/33`, `n=61 → 41/33` (fuzzer returns from here on), `n=79 → 45/37`, `n=106 → 49/37`. **The 664 figure still stands** — dim 37 *was* measured, via ECC-Q at n ∈ [47,60] — and the reviewer closed the gap manually: 200 000 random printable-ASCII strings of 79–106 chars at ECC-L, all dim 37, **zero** over-budget; a 20 000-sample max gives **652** against the 684 budget. **Fix:** restructure so a Q failure skips only the Q half (`if err == nil { …Engrave… }`) rather than returning, and add printable-ASCII 79–106-byte seeds via `f.Add` so the L path reaches dim 37. No product defect; the fuzzer just does not test what its cap increase implies.

- **`seedhammer-constantqr-failclosed-test-always-skips`** — **Owning phase: same cycle as the above. Phase A review M4.** `TestConstantQRLargeVersionsFailClosed` (`engrave/engrave_test.go:466-483`) can never assert anything: dim 41 is unconditionally rejected by the `dim > 37` guard, so `ConstantQR` always errors and the test always takes `t.Skipf`. Its doc comment claims it checks the code "never silently truncates" — it tests neither truncation nor fail-closure. **The property does hold** (`ConstantQR:494-497` returns `too many … QR modules` when `len(modules) > nmod`, and `modules` is `append`-grown from a capacity hint so it cannot truncate) — the test simply does not say so. **Fix:** assert the error explicitly (`if err == nil { t.Fatal("v6 unexpectedly accepted") }`) instead of skipping, and move the no-truncation claim to a test that actually exercises the `len(modules) > nmod` branch. Same false-pass class as the four already found in this feature.

- **`seedhammer-phaseA-test-cite-drift`** — **Owning phase: opportunistic. Phase A review N1/N2, both Nit.** (a) Every `engrave.go` line citation in the three new Phase A test files is stale: `font/constant/coverage_test.go:9` cites `:1365` (a blank line); `glyph_rules_test.go:22` cites `:1216-1218` for `panic("variable width font")` (actually `:1279`) and `:67` cites `:1294-1296` for the sentinel; `engrave/passphrase_alphabet_test.go:16-17,44-45` cite `:1208-1210`/`:1215`/`:1218` (actually `:1271`/`:1276`/`:1279`). Only `cmd/vectorfont/main.go:414-428` is correct. Same cite-drift class the spec's own review history corrected three times — worth a lint or a convention rather than another manual sweep. (b) `TestNoGlyphStartsAtOrigin` (`glyph_rules_test.go:67-85`) guards a requirement the implementation **no longer has**: `2796b2b` deleted the `inf.Start != (bezier.Point{})` sentinel and `runSplitter` now skips structurally, so a glyph starting at the origin is harmless. Harmless and passing, but its stated rationale is obsolete — and if the requirement *did* still apply, spec §3.5.0(iii) extends it to **every run's** start while the test checks only the glyph's first knot.

- **`seedhammer-address-keyboard-inherits-4th-page`** — **Owning phase: Phase D or any later GUI cycle. Found by the Phase B post-implementation review 2026-08-03.** `NewAddressKeyboard` (`gui/passphrase_keyboard.go:133`) wraps `NewPassphraseKeyboard`, so extending the passphrase keyboard to four pages silently gave the **BIP-85 child-index** flow (`gui/bip85.go:183,199`) and **typed-address verification** (`gui/verify_address.go:45`) a fourth page of `% * < > [ ] { } \ ^ \` | ~` — characters meaningless in both contexts, and one extra press to cycle back to lowercase. **No correctness impact**: both validate downstream (`parseBip85Index`, `address.Find`), and no test covers the change either way. **Fix shape:** give `NewPassphraseKeyboard` a page-set parameter so the address keyboard keeps three pages, rather than every consumer inheriting the passphrase charset. Low priority, pure UX.

- **`seedhammer-keyboard-page3-never-rendered`** — **Owning phase: Phase D. Found by the Phase B review 2026-08-03.** `TestPassphrasePageCycleRender` renders page 2 only, and `TestKeyboardCoversPrintableASCII` inspects key structs without drawing — so **page 3 is never rendered by any test**, despite being the only page whose glyphs (`\` `` ` `` `|` `~` `^`) have never appeared on a keyboard before. The reviewer rendered it manually and it is fine (all 13 draw; `poppins.Bold25` has non-zero advances for all 95 printable ASCII; page 3's extent is 272×136 vs 340×182 for pages 0–2, so it introduces no new overflow). Add a render smoke test for page 3 so a future glyph or layout change cannot break it silently. **Note the pre-existing issue found alongside:** rightmost-key clipping on the 240 px test display affects pages 0–2 more than page 3 — unrelated to this work, worth its own look.

- **`seedhammer-hash-glyph-still-k4`** — ✅ **RESOLVED 2026-08-03 (fork `5f667dd`).** `#` is now a single stroke and **max k = 2** holds. Closing it required a deliberate exception to the no-golden-movement rule: `gui/slip39_polish.go:492` engraves share titles as `"7945 #1 1/1"`, so `#` IS on existing plates — an assumption D5 never checked, since `#` was in the original 52 rather than among the 17 added symbols. Accepted on two grounds: the **artwork is identical** (same four lines, different tool path — the golden moves because the engraving *plan* moves), and **no SLIP-39 plate has ever been engraved** (unshipped fork feature, user-confirmed). Exactly three `slip39-*` goldens moved; everything else untouched. Original text follows. ~~**Owning phase: BEFORE the Phase A post-implementation review closes. Found 2026-08-03 (fork `381364a`).**~~ The multi-run amendment (spec §3.5.0) requires the four reducible glyphs be redrawn as single strokes so **max k = 2** and the accepted disclosure bound `T_row = rowLen + n_row` holds. `*` (4→1) and `x` (2→1) redrew cleanly; **`#` remains at k = 4** and `$` at k = 2, because naive retracing corrupts them. **Root cause worth remembering:** `font/constant` is a **B-spline**, not a polyline — control points are not interpolated like polyline vertices, so inserting a collinear midpoint (the `$` attempt routed through `(477,5)` to reach the crossbar) or stacking 180° reversals changes the rendered curve. `#` grew a spurious diagonal across its lower left; `$`'s lower bowl collapsed to a triangle. **Every mechanical test passed on the broken glyphs** — advance, decodability and origin are all unaffected — so only the render loop caught it. **Consequence:** with `#` at k=4 a single `#` contributes **3** extra units, not 1, so §3.5.0's `T_row = rowLen + n_row` and its `2L` worst case are **currently false**. **Two ways to close, pick one:** (a) iterate on `#`'s stroke path with the `vectorfont -dump` → `rsvg-convert` → look loop until it renders correctly at k=1 — the parts genuinely do intersect, so a valid path exists; or (b) generalise the disclosure to `T_row = rowLen + Σ(kᵢ − 1)` and re-derive the bound, which needs the user's re-acceptance since the coefficient grows. **(a) is preferred** — it keeps the number already accepted true. Cross-ref [[seedhammer-engraving-font-swap]]: a font replacement would moot this entirely.

- **`seedhammer-font-svg-ncname-ids`** — **Owning phase: Phase A cleanup or any later font work. Surfaced by the Task 3 implementer 2026-08-03 (fork `71d143b`).** The 17 new symbol glyphs are addressed via `mapChar`'s single-character branch, so their SVG ids are the literal characters — `id="!"`, `id="&amp;"`, `id="""`, `id="\"` and so on. Several are **not valid XML NCNames**. Nothing in the toolchain validates them and Go's `encoding/xml` parses them happily, so the build is correct today — but `constant.svg` was originally authored in Illustrator, and a round-trip through a real SVG editor may not preserve or may mangle them. **Fix:** add proper names to `mapChar` (`cmd/vectorfont/main.go:704-771`, which already has the `lt`/`gt` precedent for `<`/`>`) and rename those ids to match. Low risk, cheap, but it silently constrains which tools can ever edit the font source. Cross-ref [[seedhammer-engraving-font-swap]] — a font replacement would moot this.

- **`seedhammer-titlestring-filter-widened`** — **Recorded, not a defect. Phase A, fork `71d143b`.** `backup.TitleString` keeps any rune the engraving face can decode, so extending the face to full printable ASCII **widened what it retains**: `! " $ % & + ; < = > ? \ ^ _ \` | ~` are now kept in plate titles where they were previously stripped. `backup.TestTitleString` had hard-coded the old behaviour (`{"$€#,", "#,"}`), asserting an *artifact* of missing glyphs rather than the function's actual contract; the fixture was corrected to `{"$€#,", "$#,"}` with `€` still dropped, which is the contract ("keep what the face can engrave"). **No production impact: `TitleString` has zero non-test callers** (verified — the only other hit in the tree is a comment), and GUI titles are program-generated. Worth recording because Phase A's "existing output provably unaffected" claim is precise about plate *output* but this is a real behaviour change one layer up; if a caller is ever added, titles will accept a wider charset than they did historically. This was also a **second, unidentified coverage baseline** that the plan's Task 1 never spotted — the plan listed `$` among the 43 missing runes without noticing a test depended on its absence.

- **`seedhammer-engraving-font-swap`** — **Owning phase: none (idea, unscheduled). Raised by the user 2026-08-03 while scoping the BIP-39 password font work.** The engraving face `font/constant` is a hand-authored single-stroke font covering only 52 of 95 printable ASCII (26 uppercase, 10 digits, 15 symbols, blank space; **zero lowercase**). Rather than hand-authoring forever, consider **replacing** it with — or offering a choice of — an existing public-domain single-line font. Best candidate family: the **Hershey fonts** (Dr. A. V. Hershey, US Naval Weapons Laboratory, c. 1967), purpose-built for plotters/engravers, full ASCII with lowercase, available as SVG via the Inkscape *Hershey Text* extension and several SVG/JSON repackagings. **Hard constraints any replacement must satisfy:** (a) **uniform advance** — `NewConstantStringer` panics `"variable width font"` (`engrave/engrave.go:1216-1218`) and the passphrase plate's "position implies index" property depends on monospace, whereas Hershey is proportional, so any import needs re-spacing into fixed cells; (b) **metrics** — em 9 / baseline 8 / cap 6, or else every existing glyph rescales and every plate's geometry changes (see spec D5); (c) **licence** — this repo is **Unlicense (public domain)**, so SIL OFL fonts are **incompatible** (OFL requires derivatives stay OFL) and even public-domain Hershey redistributions sometimes attach credit/no-resale notes worth checking; (d) **goldens** — swapping the face changes the engraved output of *every* plate type, so this is a normative change requiring its own gated cycle, not a drop-in. **Why it might still be worth it:** it would retire the hand-authoring burden permanently, and a font designed for engraving may beat a bespoke one on legibility — which is the open question in `seedhammer-engrave-33word-font-legibility` and spec O1. Cross-ref: `design/IMPLEMENTATION_PLAN_seedhammer_bip39_password_phaseA_substrate.md` Task 3, which uses Hershey as a *visual reference only* and deliberately redraws.

- **`bip39-passphrase-nfkd-normalization`** — **Owning phase: before any non-ASCII passphrase support ships. Found 2026-08-03 while scoping the Engrave-BIP39-Password feature.** `bip39.MnemonicSeed` (`bip39/bip39.go:217-226`) feeds the passphrase into PBKDF2 as raw bytes — `pbkdf2.Key(sentence, []byte("mnemonic"+password), 2048, 64, sha512.New)` — with **no NFKD normalization**, and `unicode/norm` appears nowhere in the tree. BIP-39 requires both the mnemonic sentence and the passphrase to be NFKD-normalized before derivation. **For ASCII this is a no-op, so all current behavior is conformant**; the defect is latent and bites only non-ASCII passphrases, where e.g. `é` as U+00E9 vs `e`+U+0301 derives a **different seed** than a conformant wallet (Trezor et al.) produces from the same typed characters — i.e. a silent interop failure that sends funds to a different wallet. Reachable today via `deriveAccountXpub` (the derive-xpub-with-passphrase flow). **This is why the Engrave-BIP39-Password feature restricts to printable ASCII and refuses non-ASCII loudly** — that restriction is the boundary of provable conformance, not merely a font limitation, and the spec states it as such. **MANDATORY per the Rust-primary rule:** `bip39` is one of the listed Go ports, and NFKD normalization is *normative* behavior (it changes the derived seed), so before fixing anything in Go we MUST check whether the primary Rust constellation implementation has the same defect; if it does, it is fixed **in Rust first with test vectors** and the Go change becomes a convergence port. Only a genuinely Go-only porting error may be fixed in Go directly. See [[rust-primary-go-port-rule]]. Suggested vectors: the BIP-39 Japanese test vectors, which exist precisely to exercise NFKD.

- **`seedhammer-warning-scroll-untouchable`** — **Owning phase: next fork GUI cycle. Found 2026-08-03 while fixing the StartScreen pager (fork `86e0da9`).** `Warning.Layout` (`gui/gui.go:281`) scrolls its body text **only** on `ButtonFilter(Up)`/`ButtonFilter(Down)`, and registers no `op.Input` hit area for scrolling. SeedHammer II has no directional buttons — its only production input is the `ft6x36` capacitive panel emitting `PointerEvent`s (the sole non-test source of directional `ButtonEvent`s in the tree is `cmd/controller/debug_sh2.go`, a UART debug harness) — so **any warning whose text exceeds `bodyClip` has unreachable content below the fold**. `maxScroll` is computed and clamped, confirming the overflow case is expected, not hypothetical. **Not a fork regression:** `git diff upstream/main` shows the scroll logic is byte-identical to upstream, so this is an upstream SH2 limitation the fork inherits — but the fork *raises* the stakes by adding warning-bearing flows (the loud template-engrave "experimental" warning for unclassifiable policy shapes, per `seedhammer-template-engrave-policy-summary-display`), and an unread warning on a device that engraves seed backups is a safety issue, not a cosmetic one. **Fix shape:** same as the pager fix — bind scroll to `Clickable`s (which route both `ButtonFilter` and `PointerFilter`) with `op.Input` hit areas, or add drag-scroll. **First: measure** whether any warning string actually overflows at current font/box sizes; if none do today, this is latent and drops to LOW. **Sibling class:** `ChoiceScreen` (`gui.go:1476`) and `SeedScreen` (`gui.go:2345`) were checked and are FINE — both register per-item hit areas — as are both keyboards. Warning is the only remaining button-only path. Cross-ref: the touch-axis test harness added in `gui/start_screen_touch_test.go` (`runUITouch` + `tap`) is the tool to reproduce this without hardware.

- **`sh2-custom-firmware-public-guide`** — **Owning phase: post-SH2-bringup (open until the first self-signed boot is confirmed on machine power).** User-requested public-facing `.md` teaching others to run their own firmware on a retail SeedHammer II. **Drafted 2026-08-03 → `CUSTOM_FIRMWARE_GUIDE.md`** (repo root), written immediately after this machine's OTP work completed so the hard-won details survive: the `(UNLOCKED)` permanence and what is/isn't given up; the phase-3-before-phase-5 A/B proof structure (a board never really sealed would blink in phase 5 and be recorded green); `0x040404` page locks are the RP2350 **factory default**, not a fault (our own tooling once demanded all-zero and would have condemned a good engraver); udev rule must be numbered **below 73** or `uaccess` never applies; the `error -71` enumeration quirk is the SH2's own USB front end (5/5 across three ports vs Pico 2 clean 7/7 on the same port) — **not** the cable; SHA-256 is over the **uncompressed 64-byte X‖Y** pubkey; `otp set -s` (OR-in) and never `otp load` for `KEY_VALID`; the ~1 s no-recovery window between hash-burn and readback; degraded 2-of-3 majority writes read as plain `3` under `picotool otp get`, hence `--sh2-verify-valid`; and judge the boot on **machine power**, since `monitorPowerSupply` runs before LCD init and reboots to BOOTSEL below 20 V. **✅ (a) RESOLVED 2026-08-03** — the full procedure ran to completion on the real machine and the guide now documents the confirmed result rather than a projection. Build target pinned to the fork's actual `nix run .#build-firmware` (there is no Makefile); `seedhammerii-v1.4.3-242-g98d0229.uf2` built, signed via `sign-firmware.sh` (all 7 assertion groups PASS), flashed with `picotool load --verify` → `OK`, and **the machine booted to the home screen displaying `(UNLOCKED)`** — the two-valid-keys confirmation. Two gaps found and folded while executing: **(i)** no official recovery image existed on disk at all (the only `.uf2` present was the fork build), so the "just reflash official" recovery path was theoretical — v1.4.3 is now downloaded to `~/.sh2/recovery/` **and proven** to carry a pubkey hashing to slot 0's `c8314536…` with `picotool` reporting `signature: verified`; the guide now makes fetching *and verifying* it a prerequisite, since an unverified download is not a safety net; **(ii)** `sign-firmware.sh` is device-agnostic and cannot check that the signing key is in *your* slot, so the guide adds an explicit image-key-hash-vs-burned-fingerprint comparison before flashing. **✅ (b) RESOLVED 2026-08-03 (user accepted the recommendation)** — the guide is published in the **firmware fork**, at `bg002h/seedhammer` → `docs/custom-firmware.md`, on the reasoning that someone who wants this searches for SeedHammer firmware, not for a constellation NDEF converter. The fork README's existing "Flashing a fork on retail hardware" note — which previously deferred to nonexistent "SeedHammer documentation" — now links to it and states plainly that it is not an official or supported procedure. The tooling stays here; this repo's README gained a **Custom firmware tooling** section pointing at the fork guide, and the draft `CUSTOM_FIRMWARE_GUIDE.md` was removed from this repo so there is one canonical copy and no drift. **This item is now closed.**

- **`seedhammer-upstream-prs-tracking`** — ❌ **BOTH UPSTREAM PRs CLOSED UNMERGED (confirmed 2026-07-26):** `seedhammer/seedhammer` **#34** (re-enable on-device CODEX32 entry, closed 2026-06-17) and **#35** (BCH-validated md1/mk1 engraving, closed 2026-06-17). Upstream did land two *other* fork-reported fixes on its own — the engrave fixes in v1.4.3 credit `@bg002h` via [issue #37](https://github.com/seedhammer/seedhammer/issues/37) — so the relationship is not adversarial, but our features are not coming back upstream. **The fork-fallback is now the live plan, not a contingency.** Owning phase: hardware-bringup. Status: **(a) upstream sync DONE** — fork `main` merged to upstream **v1.4.3** at `66d3121` (2026-07-26); resulting `bip380/bip380.go` is byte-identical to upstream and `engrave/engrave.go` differs by comments only. **(b) Device confirmed LOCKED** (2026-07-26, user read the home screen — no `(UNLOCKED)` suffix), so self-signed firmware requires burning our own boot key into a spare RP2350 OTP slot. **(c) Runbook drafted** → `design/RUNBOOK_custom_boot_key.md` (**not yet executed**). Key code-verified facts: 4 boot-key slots (`driver/otp/otp.go:15`), SeedHammer holds 1 → 3 free; `driver/otp/` never writes page locks / `DEBUG_DISABLE` / `KEY_INVALID`, so PICOBOOT-over-USB stays open on a sealed unit (no debug probe needed). **Remaining blockers before execution:** no `nix`/`go`/`tinygo`/`picotool` on the workstation; rehearsal on a Pico 2 / Pico Plus 2 (same `pico-plus2` target) is mandatory before touching the SH2's OTP; and the procedure's source is a *third-party* community guide ([Gangleri42](https://github.com/Gangleri42/seedhammer/blob/main/docs/howto-bootkey-and-signing.md)), not official SeedHammer docs — re-verify against the RP2350 datasheet and the installed picotool version. **Note the permanent cost:** adding a second valid key makes `isSecureBootEnabled()` (`platform_sh2.go:712-741`, requires `nvalid == 1`) return false forever, so the device will display `(UNLOCKED)` from then on — expected, not a fault, but the on-screen attestation is lost.

- **`bootkey-rehearsal-fidelity-residue`** — **Owning phase: hardware-bringup (before the SH2 OTP write).** The F11 remainder from the round-1 review (`design/agent-reports/pico2-bootkey-rehearsal-opus-review-round1.md`); F11(d) was folded as rehearsal **phase 5b** (sign + flash the real 2.4 MB fork UF2 to the Pico, acceptance = does NOT fall back to BOOTSEL), leaving two documentation/verification gaps that the Pico rehearsal cannot fully close: **(b) host-vs-on-device sealing** — phase 1 seals via picotool JSON, whereas the retail SH2 was sealed by on-device bootrom calls that also populated white-label strings and `USB_BOOT_FLAGS`; the rehearsal never writes OTP on a board with a populated user area, so "the Pico accepted phase 4" does not fully entail "the SH2 will". **(c) redundant-row readback** — `CRIT1` is 8-way and `BOOT_FLAGS1` 3-way redundant (`driver/otp/otp.go:93-95`); after phases 1/4 the raw rows `0x040-0x047` and `0x04b-0x04d` should be read individually to confirm all copies agree, since `otp set -s` writes `reg->redundancy` copies and a partial write would be invisible to the field-level read. Also **(e)** phase 0 cannot detect a Pico 2 W, so on the spare board "no blink" would be misdiagnosed as signature rejection (documented in the script header; the plain Pico 2 is the primary). **✅ (c) RESOLVED 2026-07-26** — the round-3 gate (`design/agent-reports/pico2-bootkey-rehearsal-fable-final-gate-round3.md`, M5) correctly flagged that this item's own text committed (c) to SH2 step 1 with owning phase hardware-bringup, i.e. it was **due now, not deferrable**, while the runbook contained no such commands. The raw redundant-row readback of `CRIT1` (0x040-0x047, ×8) and `BOOT_FLAGS1` (0x04b-0x04d, ×3) is now performed by `--sh2-precheck` via `read_row_raw24`, which dies on any unreadable row and surfaces picotool's disagreement warnings. **(b) and (e) remain open** — (b) host-vs-on-device sealing is unfalsifiable without a second retail unit and is bounded by the `--sh2-precheck` reads; (e) is a documentation mitigation only. Priority LOW; neither blocks the rehearsal or the SH2 write.

- **`seedhammer-nfc-secret-refusal`** — ❌ **WON'T FIX (user decision 2026-06-19):** by the time a user has written secret material onto an NFC tag in order to beam it, the secret is already exposed at tag-write time — refusing it at the engraver is closing the barn door after the horse has left (marginal added protection; NFC is a physical-tap, user-initiated vector, not remote). Upstream `bip39`-over-NFC behavior is also retained. Original proposal kept below for the record. — ~~Harden the device's NFC scan path to **refuse SECRET material**~~, enforcing the constellation spine ("`ms1` SECRET → NEVER over RF/NFC; hand-typed on the air-gapped touchscreen; `md1`/`mk1` are public → fine over NFC"). **Finding (2026-06-18):** `gui/scan.go`'s `Scan()` accepts secrets over NFC — `codex32.New(buf)` (`scan.go:68`) parses a codex32 string, and **`ms1` IS a standard BIP-93 codex32 secret (HRP `ms`)**, so an ms1 secret can currently be received + engraved over NFC; and `bip39.Parse(buf)` (`scan.go:59`) accepts a raw BIP-39 seed over NFC. (SLIP-39 over NFC is already disabled, `scan.go:64`.) **Fix:** make `Scan()` reject an `ms`-HRP codex32 string (→ `errScanUnknownFormat`), keeping the NFC path to the *public* artifacts only (descriptors via `nonstandard.OutputDescriptor`, `md1`/`mk1` via `codex32.ValidMD/ValidMK`); ms1/codex32 stay **hand-typed-only** (the existing CODEX32 entry flow). **Also decide, deliberately:** whether to keep `bip39.Parse`-over-NFC — it's **upstream SeedHammer behavior** (the companion app sends the generated seed to the engraver), but it's arguably against the same air-gap principle; the constellation may want it gated/removed on the fork. Caveat: NFC is short-range + user-initiated (a tap), not a remote vector — so this is design-consistency hardening, not an acute hole. **Size: S, security-positive; seed/secret-bearing → full gated pipeline** (carry the mandatory adversarial review). Surfaced while answering "can SH engrave ms1 / md1/mk1 over NFC?".

- **`seedhammer-slip39-hwsha`** — **PENDED 2026-06-18 (user direction — "pend the cipher accelerator for now").** Do not pick up until the user revives it; when resumed, the entry point is the Phase-0 hardware-benchmark spike below (cheapest unlock: a ~$5 RP2350 board — Pico 2 / Pico Plus 2 — NOT a SeedHammer II; can be prepped as a flash-and-read harness). — Add an RP2350 **hardware-SHA-256** path for SLIP-39's PBKDF2-HMAC-SHA256 Feistel round function (TinyGo uses pure-Go `crypto/sha256` today; high-iteration-exponent recovery is slow — e=15 ≈ 5–8.5 h; e=0/1 ≈ 0.5–1.9 s is fine). **Cycle-prep recon DONE 2026-06-18** (`design/cycle-prep-recon-slip39-hwsha.md`, 4-agent fan-out vs the RP2350 datasheet + pinned TinyGo SVD): **feasibility = GO** (firmware runs Secure state; SHA default ACCESSCTRL = Secure-Privileged-allowed; `LOCK_SHA_256` is a cooperative bootrom mutex off by default — NOT a lock-out; TinyGo pre-declares `rp.SHA256`). **BUT the performance win is UNPROVEN** — the hardware can't load an arbitrary IV, so a drop-in `hash.Hash` forfeits HMAC's marshal fast-path (~2× block count) and per-block CPU polling is slow; a naive drop-in may negate/invert the win. The real path is a **bespoke hw HMAC-PBKDF2 loop** that **must be benchmarked on real RP2350 first** (Phase-0 spike gates whether to build the cycle), and the value is bounded to **rare high-e backups**. **Priority LOW** — parked behind the benchmark spike. (See the cycle-prep doc for the full register/throughput facts + must-handles: no watchdog → bound polling; single shared block + two-distinct-HMAC-hashes → mutex/confine; register secret-scrub; build-tag host oracle + on-device golden.)

- **`seedhammer-wdt-id-override-tlv-golden`** — **Test-coverage gap (Minor) from the template-engrave exec review (2026-06-20, M2).** `md.WalletDescriptorTemplateId` (`md/template_id.go`) appends a `UseSitePathOverrides` TLV to its preimage when per-cosigner use-site overrides are present, but the committed golden (`b02b44037119e6b6fd1d82f61aa17e21`, keyless `wsh(sortedmulti)`) exercises only the NO-override path. The override branch mirrors the same `writeTLVSection` use-site path the policy-id already covers, so risk is low — but the WDT-Id is security-load-bearing (mk1↔md1 binding), so close the gap: add a golden for a template WITH per-cosigner use-site overrides, byte-pinned to `md inspect` at `descriptor-mnemonic@54dd765`. Size: XS (one fixture + one assert). Cross-ref: exec review `design/agent-reports/seedhammer-template-engrave-exec-review.md`.

- **`seedhammer-template-engrave-policy-summary-display`** — **Richer pre-engrave POLICY SUMMARY for complex template shapes (user decision 2026-06-20, R0 C3 resolution: "keep breadth but file followup to enable summary display of policy before engraving").** In the template-engrave cycle, shapes the device can't classify (general miniscript, depth-≥2 taptrees, `tr(NUMS,multi_a)`) reach the engrave-confirm screen with only `{script family, key-slot count N (=d.n), template-id}` + the loud warning — `classifyPolicy` (`md/md.go:1266`) returns `PolicyComplex,0,0` for them, so no k-of-N / cosigner breakdown is shown. This follow-up adds a **structural policy summary** derived by walking the already-decoded tree (threshold structure, per-branch k-of-N, timelock/hashlock presence, leaf count, taptree depth) so the user sees a meaningful policy description before committing a plate — WITHOUT the full `to_miniscript`-text render. **Intermediate tier** between the minimal {family,N,id} consent shipped in-cycle and the full on-screen script render of `seedhammer-broad-miniscript-renderer` (the summary is the cheaper, higher-value first step). Own gated cycle when picked up; Rust-primary (the summary semantics should mirror a Rust reference if one is added). Companion of [[seedhammer-broad-miniscript-renderer]].

- **`seedhammer-broad-miniscript-renderer`** — **Deferred display-breadth work for the fork template-engrave cycle (user decision 2026-06-20, option 3 "minimal now, render later").** The template-engrave cycle ships engrave + form-aware verify for ANY admissible md1 (the codec is shape-complete there — decode is a faithful port of all 36 tags/8 bodies, and the tree serialization is byte-faithful to Rust `tree::write_node`, so `WalletDescriptorTemplateId` verify works for every shape). But on-device **DISPLAY/EXPAND** is narrow: `classifyPolicy` (`md/md.go:1266-1315`) + `scriptForTemplate` (`gui/md1_expand.go:82-121`) hard-refuse any `tr`-with-tree and all combinators → `PolicyComplex`, and unsorted `multi` has no `scriptForTemplate` arm. So in the cycle, unrenderable shapes (general miniscript, depth-≥2 taptrees, tr(NUMS,multi_a)) show a generic **safe summary + template-id under the loud experimental warning** rather than a full on-screen script expansion. **This follow-up = port a `to_miniscript.rs`-equivalent renderer** (the Rust shape universe — combinators, multi-leaf taptrees, taproot multisig) so the device can fully render any admissible template on-screen. **L-sized** (a Go reimplementation of the rendering semantics — NOT a literal port, since the Go port omits `rust-miniscript` for TinyGo; per the Rust-primary rule the rendering SEMANTICS are defined/tested in Rust first). Smallest high-value first step: tr(NUMS,multi_a) + unsorted `multi` display + a `scriptForTemplate` PolicyMulti arm. Own full gated cycle when picked up. Companion of [[constellation-template-only-engraving]] and `seedhammer-template-engrave-key-search-time-estimate`.

- **`seedhammer-engrave-33word-font-legibility`** — **Residual (non-code, hardware/visual) from the engrave-bugfixes cycle** (BUG-3, shipped fork `main` `3a23dbb` 2026-06-19; bug-issue bg002h/seedhammer#1; spec/plan/exec-review all R0 GREEN — `design/agent-reports/seedhammer-engrave-bugfixes-{spec-R0-round0,plan-R0-round0,exec-review}.md`). The 33-word (256-bit) SLIP-39 verbatim layout (option-a rework) shrinks the plate font to **3.859 mm** (vs the 4.1 mm baseline) to fit 17 rows in column 1; column geometry and engraveability are **proven** (stroke fixed at 0.3 mm → glyph ≈12.9× stroke; 3.86 mm inter-column gap; all rows ⊂ [0,85] mm), so this is NOT a code defect. The only open question is **physical legibility** of the 3.859 mm font on a real engraved plate — a subjective/hardware judgement. SeedHammer already engraves a 24-word plate + QR at finer pitch in the same area, so no blocker is expected. **Action: visually confirm a 33-word plate on hardware before relying on it; if too dense, revisit (a 3rd column was analyzed and rejected — overruns the 85 mm plate at any legible font).** Priority LOW; only affects 33-word SLIP-39 (the other reachable counts {27,30} stay at full 4.1 mm).


### `sizeproof-qr-step-must-not-offer-what-it-drops` — ✅ RESOLVED 2026-08-06

**RESOLVED 2026-08-06 — the branch landed.** Implemented as seedhammer `f466b11`, merged to fork `main` in `1945251` and released as `fork-v0.0.0-g1945251`.


**Operator directive 2026-08-05: "Sizeproof must always be without a QR code."**
Recorded as a hard invariant in `SPEC_sizeproof.md` §3.0.

Level 1 (the plate cannot carry a code) is structural and holds: `FitSized` has
no QR parameter. Level 2 (the operator's flag cannot diverge) does NOT hold:
`ftQRChoiceFlow` (`gui/freetext_flow.go:457`) is a bare two-choice screen with no
knowledge of what is loaded, and it deliberately preserves a prior opt-in across
Back. So an operator can load a ladder — whose loader clears the flag with a
prompt — then press Back and set it again.

That divergence has now produced two defects, one per level of the stack:
- P5: the confirm screen printed `QR: yes` for a plate with no code.
- The whole-diff review: admission reserved a QR band, refusing a ladder that
  fits, with a remedy naming a code the plate cannot carry.

Both were fixed by reading the FIT rather than the flag, which is right and is
now the rule. What is still open is the flag itself: the QR step should not offer
a choice it will not honour. When the loaded composition needs the whole plate,
state that the QR is unavailable for this pattern rather than presenting
"Add QR" and discarding it — the same honesty `ftRefuse` already applies to
never dropping a QR automatically.

Deliberately NOT folded into the whole-diff fix, which is narrow by design and
must not grow at the final gate.

### `sizeproof-admission-count-at-its-own-rungs` — ✅ RESOLVED 2026-08-06

**RESOLVED 2026-08-06 — the branch landed.** Implemented as seedhammer `b2f40b4`, merged to fork `main` in `1945251` and released as `fork-v0.0.0-g1945251`.


The "proper" fix the whole-diff review recommended and the controller declined at
the final gate, in favour of the narrow `admitQR := useQR && !ftSizedBlocks(blocks)`
landed in `c9cc4db`.

`AdmissibleBlocks` lays a composition out **uniformly at `FontSizes`' smallest
rung** — the 3.0 mm anchor spec `SPEC_sizeproof.md` §6 pins deliberately, because
reserving unconditionally is what makes admission monotone. For a size ladder
that anchor describes a different plate from the one `FitSized` cuts, so the
readout's line count and the refusal's figures are about neither the plate nor a
bound on it:

- front: admission reports **12 of 24** used while `FitSized` lays out **16 rows**;
- back: admission reports **18 of 24** while `FitSized` lays out **20**;
- an edited ladder that genuinely overflows its own rungs is refused by
  `FitSized` while admission still says `ok` with room to spare — the refusal's
  numbers contradict the refusal (whole-diff review, Minor #3).

Nothing wrong is engraved: the verdict is correct in both directions after
`c9cc4db`, only the figures mislead. Fixing it means counting a sized
composition at its own rungs, which changes `AdmissibleBlocks`' contract —
guarded by `TestAdmissibleBlocksVerdictDoesNotMove`'s three measured cases, which
**must not move for uniform plates**. That is why it was not done at the gate.

Do it in the same change that reworks admission, not before.


### `seedhammer-diagonal-ripple-on-stainless` — OPEN, owning phase: machine-side, not font work

**Measured on engraved steel, 2026-08-06, and the diagnosis was CORRECTED once
already — read the correction before theorising.**

Observation 1, from SIZEPROOF!BACK: diagonal strokes ripple, axis-aligned ones do
not. `< > v ^ / \` wiggle; `|`, `-` and `_` do not. The discriminator is how many
axes move, not the glyph.

Observation 2, which arrived later and **overturns the first explanation**: there
are TWO plates, one soft steel and one stainless, cut from the same firmware and
the same toolpath. **Only the stainless shows it.**

**The first explanation was microstep positional error, and it is now doubtful on
its own.** That error is a property of the motor and driver; it would be present
in both cuts. Same path, same steppers, different result means the difference is
not in the motion planning. Recorded here rather than deleted because it fit the
axis evidence perfectly and will look attractive again to the next reader.

**Also ruled out, with measurements:** not the tripled control points (`|` has
them and is clean — the tripling is what makes a cubic B-spline turn a sharp
corner rather than rounding it); not the planned geometry (0.0004 mm from
straight, a tenth of a step); not speed (on `<` the LOWER half is measurably
slower, 0.2001 against 0.2362 mean, and it is the upper that wiggles); and not
the start of the run (`<` and `>` wiggle at their start, but `^` starts at the
bottom-left and wiggles at the END, on the right).

**SHARPENED 2026-08-06 by two more readings, and this is the strongest evidence
yet.** Every reported defect sits at a stroke ENTRY or EXIT — none is distributed
along the middle of a stroke:

| glyph | run starts | run ends | reported | lands at the |
|---|---|---|---|---|
| `` ` `` | top-left | bottom-right | wiggle near top left | **START** |
| `<` | top-right | bottom-right | wiggle in upper half | **START** |
| `>` | top-left | bottom-left | wiggle in upper half | **START** |
| `^` | bottom-left | bottom-right | **truncated** on the right | **END** |

`^` is not wiggly at all — it is cut SHORT. That is a second, distinct defect:
the cut does not reach the end of the stroke. Its arms are also the shortest in
the set at 0.94 mm against `<`'s 1.67, so its exit region is the largest fraction
of a stroke and is where a shortfall would show first.

**This nearly settles it against hypothesis (a).** Microstep error is present at
every microstep, so it would ripple the whole diagonal evenly; it cannot produce
a defect confined to the first third of a stroke, nor a truncated end. What fits
is the tool skating before it bites on hard material, and the cut not fully
establishing before the stroke ends — both cutting-force effects, both worse in
stainless, both invisible on an axis-aligned stroke because the deviation would
lie along the direction of travel rather than across it.

**RESULT OF THE HALF-SPEED TEST, 2026-08-06: the wiggle is NOT fixed by feed
rate.** Firmware built at 4 mm/s instead of 8 (`v0.0.0-gd7155b9-dirty`), flashed,
`SIZEPROOF!BACK` cut on stainless. The wiggle is "still present and not obviously
improved". So slowing the cut is not the treatment for this defect.

**THE MACHINE IS A HAMMER — it makes discrete DOTS, not a continuous cut.** This
was not written down anywhere and it reframes everything below. What looks like a
line is a row of overlapping hammer blows, and their spacing is feed ÷ strike
rate.

**`~` is by far the worst, and the operator's description is precise:** near the
BEGINNING (left) of the middle segment it bends upward, shallowing the slope;
near the END of that segment it dips slightly; and as a result the final upward
segment appears truncated because it starts too high. That is the shape of
FOLLOWING ERROR at a direction change — the tool carrying momentum out of one
corner and settling onto the next line late.

**Ruled out by measurement, so do not re-investigate:**

- the commanded geometry is exact — 0.0004 mm from straight, a tenth of a step;
- the corners are genuinely sharp in the command — every vertex of `~`, `<` and
  `^` is TRIPLED in the compiled font, which is what makes a cubic B-spline turn
  a corner rather than round it. B-spline rounding is not the cause;
- feed rate, now tested directly.

**So the deviation is in EXECUTION, not in the path.** The next lever is
ACCELERATION and JERK, not speed: these glyphs have 0.75–1.67 mm segments, and a
stroke that short never reaches the top feed at all, so lowering the ceiling
changes almost nothing about how the tool behaves at a corner. Halving the feed
lengthened the whole plate by only 33% for exactly this reason.

A tempting correlation that does NOT hold: sharp corners alone do not predict it.
`W` turns through 153° and was not called out; `~` turns 108° twice and is worst.
`~` also has the shortest segments in the set, so the tool has least room to
settle — but a turn-per-run-up ratio ranks `v` worst, and `v` only "wiggles a
little". Recorded so the next reader does not over-fit to a rule that is not
there.

**Two hypotheses survive, and they predict different ripple PERIODS:**

| | mechanism | period | ripples on a 1.33 mm arm | changes with feed? |
|---|---|---|---|---|
| **a** | ripple is real in the motion; soft steel burnishes it away, stainless records it | one full step, **0.040 mm** | ~33 | no |
| **b** | ripple is made by the cut — chatter or stick-slip at the higher force stainless demands | feed ÷ tool-gantry resonance | a handful, ~4–9 | **yes** |

**THE DISCRIMINATING TEST IS CHEAP: count the ripples on one arm under a loupe.**
About thirty means (a); a handful means (b). Confirm by re-cutting one plate at a
different engraving feed — under (b) the period moves, under (a) it does not.

**Do NOT redraw glyphs for this either way.** Cross-ref §4.1's slant, which was
also measured to be the machine rather than the drawing. Nobody has measured the
ripple's amplitude or period yet, only seen it.

### `seedhammer-soft-hard-material-setting` — OPEN, owning phase: its own gated cycle, AFTER the feed test

**Operator's idea, 2026-08-06, prompted by the two plates.** One engraving speed
does not suit both materials: the same firmware and toolpath cut clean on soft
steel and shows entry wiggle plus `^`-style end truncation on stainless. See
[[seedhammer-diagonal-ripple-on-stainless]] for the evidence.

**Name them `soft` / `hard`, not `carbon steel` / `stainless`** (operator). That
names the property that actually drives the behaviour — cutting force against
hardness — so brass, titanium or a mystery alloy still fall into a bucket the
operator can pick. Naming two alloys implies the machine knows which one is on
the bed, and it does not.

**THE FEED TEST IS DONE, 2026-08-06, AND IT REFRAMES THIS FEATURE.** Slower feed
does NOT fix the diagonal wiggle. What it does, and dramatically, is improve
ENGRAVING QUALITY on hard steel — the operator: "much much much nicer, because
the dots of the hammer blows are more closely spaced... slow speed on hard steel
looks closer to the continuous smooth line effect of the faster speed on soft
steel." It ALSO much improves the line widening where a path retraces itself.

So the feature is justified, but for a different reason than it was proposed:
**it is about dot density and finish, not about the wiggle.** The wiggle needs
acceleration/jerk, which is a separate lever and a separate follow-up.

The measured cost is smaller than expected: `SIZEPROOF!BACK` goes from ~14m28s to
~19m15s, **+33%, not +100%**, because travel moves are unchanged at 30 mm/s and
short strokes never reach the top feed anyway.

**THE MATRIX SO FAR, and it does not order simply.** Three of four cells cut:

| | dot texture | retrace widening |
|---|---|---|
| soft @ 8 mm/s | **best** — least evident | bad |
| hard @ 8 mm/s | worst | bad — **equally bad**, per the operator |
| hard @ 4 mm/s | middling | **best** |
| **soft @ 4 mm/s** | **NOT YET CUT** | **NOT YET CUT** |

**The two defects have DIFFERENT causes, and this is the useful part:**

- **Retrace widening is a SPEED effect, not a material one.** It is equally bad
  on soft and hard at 8 mm/s and good on hard at 4. Material does not enter it.
- **Dot texture depends on BOTH.** Soft at 8 is best, hard at 8 worst, hard at 4
  in between.

**So slow feed is very likely better everywhere, and the setting may not be
soft/hard at all** — it may be a single slower default, or a `fine`/`fast`
quality choice orthogonal to material. Cutting soft @ 4 mm/s decides it: if that
cell is best on both counts, there is no material axis, only a speed one. The
half-speed firmware (`v0.0.0-gd7155b9-dirty`) is already flashed, so it costs one
plate and no reflash.

**This also bears on the glyph work already done.** `z` and `*` were redrawn this
session specifically to stop them cutting parts of themselves twice, and `f` was
left still doing it. If slow feed fixes retrace widening universally, those
redraws were treating a symptom that a machine setting also treats — they remain
worth having (fewer passes is less time and less to go wrong) but the urgency of
chasing the remaining retraces drops sharply.

**Nothing else blocks it.** Nobody has yet cut a plate at a
slower engraving feed to confirm that slowing down is even the fix. If a slower
feed does not clean up the entry wiggle and the truncation, the answer is a
different tool or a lead-in, not a speed setting, and the whole feature is aimed
at the wrong thing.

**Why this is a gated cycle rather than an inline change:**

- `EngravingSpeed` is a compile-time constant in TWO places —
  `cmd/controller/platform_sh2.go` (the original, `tinygo && rp`) and
  `internal/sh2/params.go` (the host copy). `TestParamsMatchTheMachine` parses
  the former and fails on divergence, so the pair is already guarded; making the
  value runtime-selectable has to keep that guard meaningful.
- **Goldens pin timing, but at a TEST-LOCAL speed — CORRECTED 2026-08-06.** An
  earlier draft of this entry claimed that changing the engraving speed moves
  every golden. It does not, and the claim was checked by halving the machine
  speed and running the suite: **45 packages, zero failures.** There are FOUR
  independent copies of `engravingSpeed = 8 * mm`:

  | copy | who uses it |
  |---|---|
  | `cmd/controller/platform_sh2.go:191` | the machine — the ORIGINAL |
  | `internal/sh2/params.go:27` | host tools: plateview, emu, glyphtrace |
  | `backup/backup_test.go:35` | the plate goldens |
  | `engrave/`, `gui/`, `stepper/` `_test.go` | their own suites |

  `knotsCloseEnough` does compare `k1.T == k2.T` exactly, but against the
  TEST copy, which is independent of the machine's. So the blast radius of a
  material setting is far smaller than feared: **no golden moves at all.**

- **AND THAT IS ITSELF A GAP.** Nothing pins the machine's actual engraving
  speed. `TestParamsMatchTheMachine` only checks that the machine's copy and the
  host copy agree with EACH OTHER, not that either is any particular value, so a
  change to the machine's speed ships with a green suite.
  `TestPassphraseRuneDurationPin` looks like it would catch it and does not — it
  is computed at the test speed too. Worth a pin of its own, whether or not the
  material setting is built.
- **It touches the constant-time model.** `runeDuration`, `advDur`, `padDur` and
  `centerDur` are all derived from the stepper config. Within one plate they
  scale together, so the `T_row <= 2L` disclosure bound survives — but
  `TestPassphraseRuneDurationPin` pins an absolute tick count and would need one
  per setting. Anything touching that model is risk-set work.

**Design question left open:** what should actually differ. Slower feed is the
obvious lever and changes timing only. But the defects are at stroke ENTRY and
EXIT, not distributed, so a dwell at the start or a slight overrun at the end may
fix them at far less cost in plate time — and an overrun changes the toolpath,
hence the plate, hence the goldens, in a way a speed change does not.

## Phases used in this file

`B2a-i` · `B2a-ii` · `B2b` · `B2c` · **`post-merge polish and hardening`** ·
`before the release tag` · `the fable whole-diff review of ALL of Phase 2` ·
`post-release feature`

**`post-merge polish and hardening`** was created by operator ruling 2026-08-10
to hold work that binds but does not gate the merge. It now holds two groups:

- **seed residue** — **F-88**, **F-90**, **F-104**, **F-94**, **F-87**
- **the wipe's own reliability** — **F-103** (spurious touch input stops
  §10.2.4 firing *at all*, silently) and **F-109** (~35 K in ~81 reachable
  objects, unidentified, possibly seed-bearing)
- **motion** — **F-114**, whose severity is undetermined: efficiency if the
  resume traverse is motion-profiled, plate integrity if it is not
- **font and rendering** — **F-78**, **F-86**, **F-95**, **F-119**
  (assigned 2026-08-10, formerly "the font cycle")

Note within the second group: **F-78** (`·` has no glyph, on four shipped
screens) and **F-86** (`%` renders as zero pixels in the KDF progress screen)
are **visible on shipped screens today** — the `%` is absent for the whole of a
~31-second derivation, which is the machine's longest wait. They do not gate
anything, but they are the two items here most likely to be reported by a user
on day one.

**ANSWERED 2026-08-10 (operator ruling): this phase runs AFTER the tag, and the
tag remains `v0.0.0-g<sha>`.**

That version string is doing real work in the decision. `v0.0.0` is not a claim
of readiness — it marks a build, not a product — so tagging with this phase open
is not the same as shipping a finished machine with known gaps.

**What still follows.** §2.2 is explicitly *"normative and belongs in operator
documentation, not only here"*, and the items it already carries are exactly this
shape: item 9 (an open session's plaintext is SWD-readable), item 12 (other
programs do not wipe), item 13 (the plate under the needle). A version number is
not operator documentation, and an operator engraving a real seed does not read
semver. So §2.2 should say, plainly, what the tagged build's wipe does and does not do.
With F-103 and F-109 now in this phase too, that is more than the single sentence
first proposed — it is **two**:

1. the wipe does not reach every copy of a secret inside the payload flow
   (F-88/F-90/F-94/F-104), and ~35 K in ~81 reachable objects survives it
   unidentified (F-109); and
2. the wipe **can be prevented from firing at all**, silently, by a touch panel
   reporting spurious events — no warning, no wipe (F-103).

The second is the one that matters most to an operator, because it is not
"some residue survives" but "the control you are relying on may never run".

**The fable whole-diff review is POST-RELEASE, and that is deliberate**
(operator ruling 2026-08-10, superseding the earlier "do not tag with F-109
open"). The order is: **merge → tag → release → fable review.**

Its scope grows rather than shrinks. It receives:

- the **whole Phase 2 diff** — B2a-i + B2a-ii + B2b, read by one reviewer at
  once, which is the thing no single phase's context could do; and
- **every deferred follow-up**, with a mandate to **suggest closures**.

*Suggest*, not perform. The reviewer proposes which deferred items its reading
of the whole diff shows to be already satisfied, subsumed, or not defects; the
operator decides. That distinction matters because several items in this file
turned out to be unrecorded-rather-than-undone, and a whole-diff reading is
exactly the vantage point that can tell those apart — but a reviewer closing its
own findings would be marking its own homework.


## Reconciliation — 2026-08-10, on B2b closing

B2b merged and pushed (fork `b2b` `75233b8`). Per the standard, an item whose
owning phase has passed is **overdue, not deferred**, so this sweep ran against
the whole file rather than the phase's own list.

**Closed by this sweep:** F-79, F-105, F-107, F-108, F-111 — each with the
evidence in its entry.

**WAS OVERDUE, now re-assigned by operator ruling:** **F-109**. Its owning phase
was B2b, and B2b was merged and pushed with it open — a real gate slip, recorded
rather than hidden. Ruling 2026-08-10 moves it to **the fable whole-diff review
of all of Phase 2**, a gate this ruling creates, to run before the release tag.
That is a venue change rather than a deferral: the item spans three phases, which
is why three phases of review each left it standing.

**THE BURNDOWN LIST — six items, open, owning phase already passed.** Whoever
opens B2c starts here:

| item | owning phase | mechanism visible in tree |
| --- | --- | --- |
| F-77 | B2a-i Task 1 (GATING) | `seal/label_encrypted.go` names it explicitly |
| F-80 | B2a-ii (two of three bullets) | — |
| F-84 | B2a-ii Task 6 | `SeedScreen.NoEdit`, `gui/gui.go:2335` |
| F-87 | B2b | `gui/unlock_session_test.go`, `gui/wipe_inventory_audit_test.go` |
| F-89 | B2b (design constraint) | `wiping = true` unwind, `gui/run_flow.go:265,285` |
| F-93 | B2b, with F-89's unwind | `ctx.KeepAwake()` bracket, `gui/unlock_kdf.go:318-326` |

Five of the six look unrecorded rather than undone. They stay OPEN anyway: a
symbol existing is a presence check, not a behavioural one, and closing on that
is exactly the evidence this cycle has repeatedly found worthless.

**This file does not satisfy its own reconciliation contract, and that is a
finding.** The standard says to record the owning phase in each entry "so
reconciliation is a grep". It is not, because closure is marked in three
different places and three different words:

- in the **heading** (F-99: `### F-99 — CLOSED 2026-08-09 — …`),
- in the **body** (F-106, F-110),
- or as prose that never says CLOSED at all — F-59 is `WITHDRAWN`, F-83 is an
  `ACCEPTED` limitation.

Two automated sweeps over this file during the 2026-08-10 audit disagreed with
each other on F-93 and F-99 for exactly that reason, and a naive `grep -c CLOSED`
miscounts in both directions. Until closure has one canonical marker in one
canonical place, every reconciliation here is hand-verified — which is what the
owning-phase rule exists to avoid. Worth fixing before the release tag.


### F-58 — total input wedge on the Footer entry screen, before engraving (owning phase: GUI)

**Observed 2026-08-06**, on the `test-e4-a125-j1300` build, from a cold boot while
walking the `SIZEPROOF!BACK` workflow. On the **Footer** screen
(`ftLineEntryFlow`, `gui/freetext_flow.go:705`) the checkmark did not advance to
the next screen, **and then no button responded at all**. Power cycle was the only
way out. The workflow ran normally on the retry.

**No engraving took place, so there is no plate and no acceleration/jerk data.**
The wedge happens several screens before any motion.

Three wrong diagnoses preceded this, each corrected by the operator. Kept because
each is a trap, and because the pattern itself is the lesson — every one came from
reasoning ahead of the evidence:

- ❌ *"Driver state carried across the flash."* Impossible. The SH2 has **one**
  USB-C port, shared by BOOTSEL and power. Flashing means it is on the computer;
  engraving means unplugging it for the high-wattage supply. That disconnection
  kills power, so **every flash→engrave transition is already a cold boot**. No
  hypothesis may depend on state surviving it.
- ❌ *"It hung starting the engrave."* No motion was ever commanded.
- ❌ *"It hung on the post-engrave Accept, so the plate was cut."* Also wrong; the
  "footer" in question is the **Footer text-entry step of the workflow**, not the
  footer navigation bar of the engrave screen.

**The motion parameters remain untested.** Acceleration 125 / jerk 1300 has still
never driven a plate, and the experiment is outstanding.

**The decisive symptom is that *every* button died, not just the checkmark.** A
single unresponsive widget would be a widget bug; total input death on a screen
that still redraws points at the shared input queue.

**Suspected mechanism, NOT proven — a structural hazard that matches the symptom.**
`EventRouter.Next` (`gui/event.go:266`) is strict head-of-queue: it examines only
`r.events[0]`, and if the head does not match the filters the *calling widget*
passed, it returns nothing and leaves the head in place. `EventRouter.Reset`
(`:281`) discards head events matching **no** filter registered that frame — but an
event matching a filter that was registered and then *not consumed* is neither
delivered nor discarded. `Context.Reset` (`gui.go:97`) responds by scheduling an
immediate wakeup, so the frame loop spins at full rate, redrawing forever, while no
input is ever processed. **A live screen with dead buttons is exactly this
signature**, and it is nondeterministic because it turns on press/release timing
against an asynchronous state change.

This hazard is already known in this codebase: `gui/codex32_polish.go:316` carries
the note *"Button2 is drained every frame so it cannot block the queue head."*
Someone hit this class before and fixed that one screen locally.

**A second structural fact makes it worse, and it is the sharper of the two: a
filter is only registered as a SIDE EFFECT of calling `Router.Next`.**
`EventRouter.Next` begins `r.filters = append(r.filters, filters...)`, so a
consumer that returns *before* reaching that call registers nothing — and `Reset`,
which decides what to discard from the filters registered that frame, then judges
the head against an incomplete picture. `InputTracker.Next` (`gui.go:107`) has
exactly such an early return: when an arrow key is held it synthesises a repeat
event and returns **without touching the router**. `PassphraseKeyboard.Update`
(`passphrase_keyboard.go:248`) has two more, both `return true` mid-drain, and its
per-key loop is guarded by `k.Valid(*key) && key.clk.Clicked(ctx)` — Go
short-circuits, so an invalid key never registers its filter at all.

The Footer screen puts all of that on one screen at once: a keyboard filtering
arrows, Center and runes, plus two footer `Clickable`s on Button1 and Button3.

**Why it matters beyond the annoyance.** The failure is a *hang*, not an error: no
report, no timeout, no way to tell it from a machine thinking. The operator's only
exit is to cut power. On a workflow that will one day carry a real seed, an input
path that can wedge with no diagnostic is the wrong failure mode — and the wedge
arrives with no indication of *what* was lost, so text already entered is suspect.

**Work, when taken up.** Reproduce in a GUI test first. `ftLineEntryFlow` already
carries `hookPPWidget("kbd"/"back"/"ok", …)` test hooks, so the screen is drivable
without hardware. Hunt adversarial orderings against the head-of-queue rule: an
arrow held across a frame (exercising the synthetic-repeat early return), a
keystroke landing in the same frame as a checkmark press, a press whose release
crosses a screen transition. The test is the deliverable — this is too intermittent
to confirm fixed by hand, and a fix without a reproduction cannot be shown to have
worked. Then consider whether `Next` should scan the queue rather than only its
head, and whether filter registration should be explicit rather than a side effect
of consumption; either removes the whole class rather than this instance.

### F-59 — WITHDRAWN 2026-08-06: the artefact was Y-axis play, not cusps

**The cause was a loose screw in the Y axis**, found and fixed by the operator;
both forward and reversed tildes now cut perfectly. Nothing in this entry's
causal story survives. The *face-wide fact* it records is still true and still
interesting — `font/constant` is 94 polygons and no curves — but it is not the
reason for any artefact, and it is not a defect. See the resolution banner in
`design/RECON_cusp_dot_pileup.md`.

Original entry follows, kept for the record:

### F-59 (withdrawn) — `font/constant` has no curves, and its cusps pile dots (owning phase: the glyph pass, BEFORE `O`/`o`/`8` are drawn)

**Diagnosed 2026-08-06.** Full workings, with every measurement, in
`design/RECON_cusp_dot_pileup.md`. Summary:

`font/constant` is **104 polylines, 7 lines, 0 paths** — measured over the whole
face, **94 of 94 glyphs are all-tripled control points**, i.e. polygons with a
cusp at every vertex and not one curve anywhere. `font/sh` has 31 curved glyphs.
`O` in the constant face is a 9-sided polygon; `o` is a pentagon.

The needle fires on a **fixed 25 ms period** (`platform_sh2.go:154`), so dot
spacing is feed rate x 25 ms and nothing else. The planner spends **equal time per
knot**, so at a cusp the tool crawls: measured on `~` at 3.0 mm, **1.2 mm/s with
dots 0.031 mm apart at the corners against 7.8 mm/s and 0.195 mm on the
straights — a 6.3x swing**, two or three 0.3 mm strikes inside 0.06 mm.

**This explains the null accel/jerk plate.** Halving acceleration and jerk leaves
the ratio at **6.3x, unchanged**; it only makes everything 26% slower. A motion
parameter cannot fix a ratio. **The pile is absolute** (~2 dots, set by the needle
period) while the glyph scales, so it is half the glyph at 3.0 mm and a quarter at
6.0 mm — the size dependence the operator reported. `|` has the same speed ratio
and is clean, which sharpens it: **the defect is uneven dot spacing AT A DIRECTION
CHANGE**, not uneven dot spacing.

**Still unmeasured:** that the piled dots are what the eye reads as the wiggle on
steel. Geometry and timing say the ink must pile; only a plate says the pile is
the artefact. Cut the single-character plates (§7 of the recon) before acting.

**The fix this points at** is curves in `constant.svg` — mechanically available
today, since `cmd/vectorfont`'s `<path>` parsing is face-agnostic and
`glyph_rules_test.go` does not constrain the primitive. **The gate is the
constant-time property**: curving changes path length and therefore duration, and
duration equalisation across `constantAlphabet` is the face's whole purpose.
`k` is unaffected (it counts pen-lifts) and **max k = 2 stays a security
property**. Decide before `O`, `o` and `8` are drawn — they are the worst possible
polygons and the only three left.

### F-60 — single-character test plates, top-left and uncentred (owning phase: every engraving investigation from now on)

**Operator directive, 2026-08-06.** Engraving tests cut **one character at a
time**, at the **top-left-most position**, **not centred**.

No code change needed: in `backup/freetext.go` the *title* is centred
(`centerInset`, `:125`) but a *body row* is left-aligned at `margin + offx`
(`:153`), and with no title the first body row sits at the top margin.

```
go run ./cmd/plateview -plate freetext -face const -text '~' -size 3.0 -o /tmp/p.png
```

One glyph, top-left, left-aligned, **~2 s to engrave** against ~21 minutes for a
full plate. Pass no `-title` and no `-footer`; `-size` pins the rung instead of
letting auto-fit pick 6.0 mm. A centred title would move with the string's width,
so successive cuts would not be positionally comparable — that is why "don't
centre it" is load-bearing and not a preference.

### F-61 — `preview/params.go` is a fourth, stale copy of the machine's motion params (owning phase: the next `me` preview cycle)

**Found 2026-08-06** by the synthesis pass of the motion-params recon; all five
recon agents missed it because it lives outside the fork.

```
preview/params.go:5   // "Replicated VERBATIM from seedhammer v1.4.2"
preview/params.go:18  EngravingSpeed: 8 * mm     // the device is now 4 * mm
```

**Nothing binds it to anything.** The fork has `internal/sh2/params_test.go`'s
`TestParamsMatchTheMachine` holding its host copy to the device constants; this
sibling copy has no such test, so it drifted silently the moment `343fb05`
halved the engraving feed. **`me bundle --preview` therefore understates
engraving time by roughly 2x today.**

The one-line fix needs a decision first, and it is not derivable from the code:
should this model **our fork** or **upstream v1.4.2**? Its own comment claims
verbatim replication from v1.4.2, but the thing it previews is what the
operator's own machine will cut, and the two have now diverged. Whichever is
chosen, **it needs a binding test** — the defect is the absence of one, not the
number.

Note the fork itself carries **three** further copies that are desynchronised by
construction: `engraverParams` embeds a *copy* of `engraverConf`; homing reads
`engraverConf` directly, bypassing `EngraverParams()`
(`cmd/controller/engraver.go:194-196`); and `mjolnir2` latches `TicksPerSecond`
once at boot. Harmless while the values are immutable, and the single most
likely place for a subtle bug the moment they are not. Cross-ref
`design/SPEC_seedhammer_proof_speed_picker.md` §8.

### F-62 — STILL OPEN, but no longer motivated by the artefact: curving a glyph panics the constant-time passphrase engraver

**2026-08-06:** the artefact this was meant to fix turned out to be Y-axis play,
so there is no longer a reason to curve the face. **The panic itself is real and
stays filed** — anyone who curves a `font/constant` glyph for any reason will hit
it, and it is a firmware crash mid-plate with the needle down. Demoted from
"blocks the fix" to "a trap for a future editor".

Original entry follows:

### F-62 (context withdrawn) — curving a `font/constant` glyph panics the constant-time passphrase engraver (owning phase: BEFORE any curve lands, and before `O`/`o`/`8` are drawn)

**Found 2026-08-06** by trying it. `~` was redrawn as three cubics; the geometry
result was excellent — worst lateral dot pile-up fell from 0.0750mm to 0.0073mm,
a **10x** reduction landing exactly on the `font/sh` reference — and then the
suite refused it:

```
panic: unaligned delay
  engrave.(*timeScaler).Scale   engrave/engrave.go:1126
  TestPassphraseEngraveAlphabet, TestPassphraseNoPanicOverCharset,
  TestPassProofBuildsAPlate
```

`~` is in the **passphrase alphabet**, engraved in CONSTANT TIME so the plate's
duration cannot leak the passphrase. `timeScaler` stretches each rune to a fixed
budget; the curved glyph's **19 knots against the polyline's 12** make more
`Scale` calls, the fractional accumulator rounds differently, and the scaled
total overruns the budget. **On the device this is a firmware panic mid-plate,
needle down, for any passphrase containing `~`.**

**CORRECTED 2026-08-06, later the same day — curving does NOT fix the `~`
plateaus, and this entry originally implied it would.** Three operator nulls
(halved accel/jerk, lowest feed, soft steel instead of hard) exhausted every
machine variable, and then the commanded path was measured with timing removed:

```
const ~ 6.0mm   longest near-horizontal run in the PATH:  0.0000mm
const ~ 3.0mm   longest near-horizontal run in the PATH:  0.0000mm
```

**The flats are not in the toolpath at all.** What remains is the **0.30mm
needle footprint at a direction reversal** — the ink at a vertex is the union of
overlapping dots, whose top is a flat cap about one tool-width across. That is
feed-, material- and path-independent, which is exactly why all three
experiments came back null. A smooth path through the same vertex produces the
same cap, so **curving `~` buys essentially nothing here.** The earlier
"0.375mm -> 0.307mm" estimate was computed from the dot-cluster span and does
not survive the finding that the span is not what makes the flat.

**The lever for `~` is AMPLITUDE, not curvature.** The wave is 0.67mm
peak-to-peak at 3.0mm against a 0.30mm tool — the caps eat ~45% of it, against
~22% at 6.0mm. That ratio *is* the size dependence the operator reported, and it
is the same "two stroke widths minimum feature" wall the font rules already
name.

**But `O` is a different case, and curving genuinely does help it.** Measured on
the same pass, `O` carries a **0.667mm** near-horizontal run in its commanded
geometry — a real flat, because a 9-gon has a horizontal top edge. That one is
geometry and a curve removes it. So this entry stays open FOR THE ROUND GLYPHS,
and is withdrawn as a fix for `~`.

**So curving the face is not a per-glyph change.** It needs work inside the
constant-time scaler, and that machinery exists specifically to prevent a timing
leak — so it is security-sensitive and must not be rushed. Any curve landing
before it is fixed ships a crash.

Two things worth carrying forward: the fix direction is now **measured rather
than argued** (see the plate results appended to
`design/RECON_cusp_dot_pileup.md`), and **`O` is the worst glyph in the face** at
a 74.9x dot-pitch ratio as a 9-gon — so the constant-time question should be
settled BEFORE `O`, `o` and `8` are drawn, not after. Cross-ref
`SPEC_seedhammer_proof_speed_picker.md` section 8, which named this gate as the
cost of curving the face; it was reached by the glyph rather than by the feature.

### F-63 — the hammer's strike CURRENT is a lever the firmware cannot reach on this board (owning phase: any future depth investigation)

**Recorded 2026-08-06** while asking what the firmware knows about the engraving
head. It is the third lever on depth, after feed and passes, and it is currently
fixed in hardware.

**The head is a solenoid driving a needle, gated by a TEXAS INSTRUMENTS DRV8701**
brushed-DC gate driver — cited by datasheet URL in
`cmd/controller/engraver.go`'s current-limit block. That is the only vendor part
named anywhere near the engraver: the package is `driver/mjolnir2`, whose doc
comment says only *"a driver for the particular engraving hardware in the
Seedhammer II"*. Every other driver in the tree is a datasheet part number
(`tmc2209`, `ap33772s`, `clrc663`, `ft6x36`, `ili9488`, `st25r3916`); the
engraving head gets a myth, because it is SeedHammer's own design.

**The firmware CAN set a pulsed current limit, and on this board it does not.**
`engraver.go` computes DRV8701 formula (1) — `Vref = Ichop*Av*Rsense + Voff`
with `Av = 20 V/V`, `Voff = 50 mV`, `Rsense = 5 mΩ`, against `Vmax = 3300 mV` —
and drives it as a PWM duty on `S_VREF` (GPIO30, PWM7). But:

```go
// cmd/controller/platform_sh2.go
Ichop  = 0              // "Disable it by setting it to 0 on production boards."
P_ADC  = machine.NoPin  // pulse-length ADC input: absent
S_SENSE = machine.NoPin
```

`if Ichop > 0` therefore never fires. **The strike current comes from on-board
resistors**, and the pulse-length ADC (`mjolnir2.Device.PulseADC`) is nil, so
nothing measures the pulse either. These are development-board hooks that
production hardware fixes in silicon — which is what this machine is.

**Why it matters.** Strike energy has three inputs and the firmware only reaches
two of them:

| lever | reachable in firmware? |
| --- | --- |
| dot spacing (feed) | yes — `engravingSpeed`, and now the Speed picker |
| strikes per glyph | yes — the new Passes setting |
| **energy per strike** | **no** — `needleAct` 4–5 ms is voltage-interpolated and not exposed, and the current limit is resistor-fixed |

So if a depth problem is ever traced past feed and passes, the next lever is a
**hardware** change, not a firmware one. Worth knowing before anyone spends a
session looking for it in software — which is exactly the mistake the Y-axis play
investigation already made once (see the resolution banner in
`design/RECON_cusp_dot_pileup.md`).

**Related, unfiled:** `needleAct` interpolates 5 ms at minimum PD voltage down to
4 ms at maximum, a 25% swing in strike dwell decided by what the supply
negotiates. Nothing displays the negotiated voltage, so it cannot currently be
read off the machine. That would be a cheap diagnostic if depth is ever in
question again.

### F-64 — `VOLTPROOF!`: engrave the machine's own strike conditions onto the plate (owning phase: the next depth investigation, or whenever engraving settings go system-wide)

**Operator idea, 2026-08-06.** A proof trigger that cuts the negotiated USB-PD
voltage and the resulting `needleAct` dwell onto the plate itself.

**What it solves.** Strike energy has three inputs and firmware reaches only two
(F-63). The third — energy per strike — moves with the supply: `needleAct`
interpolates **5 ms at minimum PD voltage down to 4 ms at maximum**, a 25% swing
in dwell, and **nothing displays the negotiated voltage anywhere**. So two plates
cut on two machines, or on one machine across a supply change, are not comparable
and there is no way to tell. The operator's own machine shipped from a 240 V
country and runs on 120 V; whether that changes the negotiated contract is
currently unknowable from the device.

**Why on the PLATE rather than on a screen.** A screen readout is cheaper and
worth having too, but it is not attached to the evidence. A depth plate that
carries its own conditions is self-documenting forever — the marks and the
parameters that produced them stay together, which is exactly what every depth
comparison this session has lacked. Pair it with the `passes:`/`speed:` values
and a plate becomes a complete record.

**Grammar fits.** `design/LEXICON_proof_triggers.md`: the root names the AXIS the
plate proves, and the parameter slot means one kind of thing. `VOLTPROOF!` proves
the supply axis and needs no parameter.

**What it needs, and this is the real cost.** The GUI cannot currently see the
voltage. `gui.Platform` (`gui/gui.go`) exposes `LockBoot`, `AppendEvents`,
`Wakeup`, `Engraver`, `NFCReader`, `EngraverParams`, `DisplaySize`, `Dirty`,
`NextChunk`, `Features`, `HardwareVersion` — and nothing about power. The
negotiated voltage lives in `cmd/controller/platform_sh2.go` (`monitorPowerSupply`
/ `adjustSupplyVoltage`, around :257 and :466), and `needleAct` is computed from
it inside `cmd/controller/engraver.go`'s `Engraver()`, at engrave time.

So this needs the Platform interface widened by one accessor, plus
implementations in `cmd/emu/platform.go` and the test platform in
`gui/gui_test.go`. That is small, but it is an interface change on the surface
the whole GUI talks to, so it wants a moment's thought rather than being bolted
on.

**Suggested content**, kept to one line so it fits any rung:

```
    24.0V  4.4ms  4.0mm/s  x3
```

voltage, dwell, feed, passes — the four numbers that decide what a mark looks
like. Deriving dwell in the GUI means duplicating `engraver.go`'s interpolation,
so prefer exposing the computed `needleAct` rather than recomputing it, or the
plate will eventually disagree with the machine.

**Cheaper first step if this is ever wanted in a hurry:** put the voltage on the
START SCREEN beside the version string. No trigger, no plate, no interface
question beyond the same accessor — and it answers "is my supply
under-delivering" on every boot. The plate version is the better artefact; the
start-screen version is the faster diagnostic.

### F-65 — back up the SH2 boot signing key (owning phase: after the encrypted-payload cycle ships; NOT during it)

**Operator question, 2026-08-07.** Can the encrypted-payload path carry the
firmware boot signing key (`~/.sh2/sh2-boot-key.pem`)?

**As specified, no.** `SPEC_encrypted_payload_delivery.md` §6.3 admits exactly
four record types — `md1`, `mk1`, `ms1`, BIP-39 mnemonic — and §10.2.1's
allow-list rejects everything else by design. A raw secp256k1 scalar is none of
them.

**But it works today with zero spec change, and that is the recommendation.**
The key is a 256-bit secp256k1 scalar; BIP-39 at 256 bits of entropy is
256 + 8 checksum = 264 bits = **exactly 24 words**, so the scalar encodes
losslessly as a mnemonic — already an admitted secret record. It then gets the
correct handling for free: encrypted section, classified secret, offered first
under §10.2.2, wiped after its plate, covered by §10.2.4's timer.

**Calibrate the effort — this is not seed-class.** Per the operator's own
`~/.sh2/SEED_HAMMER_OTP_SLOT_USAGE.txt`: slot 0 holds SeedHammer AB's production
key, slot 1 holds theirs, and **slots 2 and 3 are FREE**. Losing the key costs
one spare OTP slot, not funds — burn a replacement into slot 2. Worth backing
up (the slots are one-time and a botched burn wastes one); not worth widening a
funds-critical spec for.

**The only real gap is labelling.** The plate comes out looking exactly like a
seed plate, and the sealed-payload path does not expose the Title/Footer fields
the free-text flow has. Future-you restores an empty wallet and loses an
afternoon. **Cheapest fix: a companion label plate via the existing free-text
flow** ("SH2 BOOT KEY / NOT A SEED", ≤18 chars per `ftMaxLineLen`). No spec
change, no new record kind, no new attack surface.

**Do NOT add a "labelled secret blob" record kind to close this.** That widens
§10.2.1's allow-list — the surface four review rounds were spent *narrowing*,
because the classifier's `command: ` branch reaches `Platform.LockBoot()` and
irreversible OTP writes. Paying that for a key recoverable from a spare slot is
a bad trade. Revisit only if the label convention proves unworkable in practice.

Superseded by [[F-66]] if that lands, since a plain-text record would carry the
label and the key on one plate.

### F-66 — carry arbitrary plain text over the sealed payload path (owning phase: its own gated cycle, AFTER the encrypted-payload cycle is GREEN and shipped)

**Operator request, 2026-08-07.** Deliver arbitrary text to the engraver through
the encrypted-payload path, not just constellation records.

**The device can already engrave free text** — `ftTextEntryFlow`, the whole
engraving-settings feature (font/size/title/footer/speed/passes). What is
missing is a way to *deliver* it: `SPEC_encrypted_payload_delivery.md` §6.3
admits four constellation record types and §10.2.1 fail-closes on everything
else. Today free text must be typed on the touchscreen.

**The hazard, and why this needs its own gate.** §10.2.1's allow-list exists
because `gui/scan.go`'s classifier accepts a `command: ` prefix that dispatches
to `debugCommand`, and `gui/gui.go:1668`'s `lock-boot` case reaches
`Platform.LockBoot()` → `writeOTPValues()` + `otp.EnableSecureBoot()` +
`machine.CPUReset()`. **A naive "raw text" record kind reopens that hole
exactly**, because `command: lock-boot` *is* arbitrary text. This was R0 round 1's
first Critical and it must not be undone.

**The clean shape, if it is built.** The danger is the classifier's dispatch,
not the text. A text record should **bypass `scan.go` entirely** — the payload
kind says "literal text for the free-text flow", the device never classifies it,
and `debugCommand` is therefore unreachable by construction rather than by
allow-listing. Route straight to `ftBuildPlate`.

**What that cycle must also settle:**
- **Constant-time engraving does not apply.** `ftWarnTiming`
  (`gui/freetext_flow.go:1212`) is explicit: "How long the machine runs depends
  on what it is cutting, so anyone watching or timing it learns about the text."
  A *secret* delivered as free text leaks to an observer with a stopwatch. If
  text records may be secret, that warning must reach the operator; if they may
  not, that must be enforced.
- **Where the layout parameters come from** — title, footer, font, size. In the
  header (authenticated, bound-checked pre-KDF) or typed on the device? Note
  §6.5's rule: the header is parsed before authentication, so anything there is
  hostile input.
- **Plate fit.** The free-text field is uncapped and backstopped only by
  `backup.ErrTooLarge` / `ftRefuse`; §6.2's `pub_len`/`ct_len` ceilings are 8191
  while `gui/scan.go`'s buffer overflows at exactly 8192.
- **Whether text may ride in the PUBLIC section.** §6.3's decode requirement for
  public records exists because `ValidMD` is a pure BCH verifier that never
  decodes — arbitrary bytes wrap into a checksum-valid `md1`. Plain text has no
  decode to require, so the public section would lose its only content check.

Subsumes [[F-65]]: a text record could carry "SH2 BOOT KEY" and the 24 words on
one plate. Related: [[F-58]] (input wedge on the Footer entry screen).

### F-71 — Nits from the Plan A whole-diff review (owning phase: ownerless residue; batch whenever `seal` is next touched)

Neither gates anything. Recorded so nobody rediscovers them or "simplifies" one
away believing a test covers it.

- **`WireError::TooLarge` / the `REGION_LEN` check is unreachable.** With
  `MAX_SECTION_LEN = 8191` checked first, max `total` is
  `52 + 8191 + 8191 + 16 = 16450 < 65536`. The code comment says it is deliberate
  defence-in-depth against a future implementation that drops the section caps.
  It is correct and it is dead; leave it, but know that no test reaches it.
- **§11.4's "seal D twice with different salts, assert the hash is unchanged"
  has no test.** Structurally satisfied — `public_data_hash` takes no salt
  parameter, so a salt dependency is unrepresentable. The *record-count* half of
  this Nit was closed by `mixed_payload_prints_the_sealed_hash_not_the_unsealed_one`,
  which pins the whole banner line.

### F-72 — md-codec 0.40 → 0.42 rode into the Task 1 commit (owning phase: none — historical note, do NOT rewrite)

The review established that **every** md-codec API `seal` uses already exists in
0.40 (`reassemble`, `decode_md1_string`, `ChunkHeader` + `chunk_set_id` +
`ChunkHeader::read` with the same signature, `pub mod bitstream`), so the bump
was never required — the plan itself says so. It nonetheless landed inside
`84c4591` ("52-byte wire header"), moving 929 source lines across
`bch/decode/validate/canonicalize` under the pre-existing converter and bundle
paths in a commit labelled for something else.

No defect: the only visible behavioural change is a tightening (new
`Error::EmptyOriginOverride`), the safe direction, and `golden`/`cross_lang` are
green on MSRV. Recorded because it is the exact bundling the standard workflow
forbids, and — as with `b946399` — **the history stays as it is.** The rule binds
future commits.


### F-79 — the payload buffer retains 64 KB for the GUI's whole lifetime (owning phase: **B2a-i, Task 2** — fix BEFORE the feature reaches an operator)

**CLOSED 2026-08-10 by fix C** (`b2b` `3de8aa1`, "bound the payload read"). `seal/read_tinygo.go` now allocates `out := make([]byte, n)` with `n` bounded by what the header declares — ~1.4 KB typical, against the 65,536 this entry measured. The 64 KiB `unsafe.Slice` that remains is a view of memory-mapped XIP flash and allocates nothing.

`uiFlow` probes once at startup and holds the result (`gui/gui.go:1541-1546`).
`XIPReader.Read` allocates `clampRegion(RegionLen)` = **65,536 bytes**
(`seal/read_tinygo.go:41-52`), and that slice lives until the GUI exits.

**At most 16,450 bytes of it can ever be meaningful.** §6.2's own caps make the
largest legal blob `52 + 8191 + 8191 + 16`, so ~49 KB of what is retained is
provably erased flash. Measured on the B1 branch: `tinygo build -target
pico-plus2 -size short ./cmd/controller` reports `ram 69300` — about 451 KB free
of the RP2350B's 520 KB — so this is **~14% of the free heap, held permanently**,
whenever a payload is present.

§6.4 already treats a **transient** ~98 KB as a design hazard ("a fifth of the
free heap"). This one is neither transient nor measured anywhere in the plan.

**Why it is not academic:** payload-present *plus* an engrave actually running is
the one configuration the 2026-08-07 hardware pass did not drive to completion —
the recorded checks stop at the §10.2.3 warning. `validateMdmk` builds three full
plate plans at once. If that combination exhausts the heap, the failure is an
out-of-memory **during an engrave**, not at boot.

**Fix:** after `Inspect` succeeds, reslice to `HeaderLen + PubLen + CtLen
(+ TagLen)`. Or hold the `seal.Reader` and re-read on selection instead of
retaining bytes at all.

Found by the B1 whole-diff review (M-3). Not folded there because it is a
production change and that fold was deliberately kept test-only, so B1's
hardware-verified behaviour stayed untouched.

### F-80 — residue from the B1 whole-diff review (owning phase: **B2a-ii** for the two the 2026-08-08 decision assigned it; **B2b/ownerless** for the rest)

**PARTIALLY CLOSED 2026-08-10.** Two of the three bullets are DONE, each killed by an applied mutation:
- Back-is-Lock: **CLOSED 2026-08-10 — assets.IconDiscard at gui/unlock_platelist.go:179; pixel-pinned by TestPlateListBackIconIsDiscardNotBack, killed by reverting to assets.IconBack.**
- already-cut marks: **CLOSED 2026-08-10 — " (cut)" mark at gui/unlock_plates.go:59, set on completion only at gui/unlock_platelist.go:116; killed by dropping the cut branch, which fails TestPlateListMarksCutAfterACompletedEngraveAndNotAfterACancelledOne/completed and TestUnlockPlateLabelWrapsPlateLabel.**

The third — the `layoutMainPager` pixel pin — is **correctly still open** and stays with **F-78**'s font cycle, exactly as the 2026-08-08 operator split assigned it: a real pin needs a rasterising check, which is F-78's work. This entry therefore holds one bullet, not three.

*(Amended 2026-08-08. CONTINUITY_2026-08-08 §9 said "F-80's two B2 items"; there
are **three** bullets below carrying an explicit `owning phase: B2`. The operator
split them that day: the "already cut" marks and the Back-is-Lock affordance go
to **B2a-ii**; the `layoutMainPager` pixel pin does **not**, because a real pin
needs a rasterising check, which is F-78's work.)*

None of these gate. Recorded so they are a grep rather than a recollection.
Source: `design/agent-reports/encrypted-payload-planB-phaseB1-whole-diff-round0.md`.

- **`layoutMainPager`'s `lastNav` wiring is unpinned** (M-2). `pagerDots`
  (`gui/unlock_program_test.go:103`) calls `layoutMainPager` directly with a
  constant, so it measures the function, not the screen. Reverting the draw site
  (`gui/gui.go:1801`) to a hardcoded `bip85Derive` leaves the suite green —
  measured. Failure mode is cosmetic: nine programs, eight dots, and on the
  ninth page no dot is filled at all. Not folded because a real pin needs pixel
  comparison of drawn frames — dots are not text, and `uiContains` only sees
  text. **Owning phase: B2**, which touches this screen anyway.
- **`"Sealed Payload"` is duplicated** as a literal at `gui/gui.go:1792` while
  `gui/unlock_flow.go:21` declares `const unlockTitle` as "the same string the
  menu entry carries". One stated invariant, two literals, no compiler link.
- **Back is drawn with `assets.IconBack`**, which §10.3 says it should *not* read
  as ("should read as leaving the session, not stepping back one screen"). There
  is no lock/exit glyph in `gui/assets`, so it needs an asset. Harmless in B1 —
  nothing is resident — but **B2 relies on this affordance to make "every exit
  wipes" legible**. Owning phase: B2, with F-78's font work.
- **`unlockWarnUnauthenticated` formats the digest without checking
  `p.HasHash`** (`gui/unlock_flow.go:116`), unlike the notice screen. Unreachable
  today only because `ParseHeader` rejects `pub_len == 0 && ct_len == 0` with
  `ErrEmpty`. If that bound is relaxed it prints the empty-set constant under a
  "compare this" instruction. A guard costs nothing.
- **`groupCards` has no production caller** (`seal/record.go:341`) — it survives
  as a wrapper for two tests. Either fold them onto `groupRecords` or say in the
  doc comment that it is test-facing.
- **`PlateIndex` is positional**, counted in record order by `labelCards`
  (`seal/record.go:257`), not read from the record's own
  `ChunkHeader.ChunkIndex`. A chunk-permuted public section is admitted —
  measured: reversing vector D's five records yields `err=nil, 5 records`.
  §6.6's "record order is plate order" makes the positional reading defensible,
  so this is a documentation gap; one line saying which the label means keeps B2
  from re-deriving it.
- **§10.2.2's "records already cut this session are marked" is unimplemented and
  unlisted.** The B1 plan's *What B1 does NOT cover* defers §10.2.2's lifecycle,
  wiping and §10.2.4 — but not this bullet, which is a property of the plate
  list, and the plate list is B1. Intended as B2; **owning phase: B2**, recorded
  so it is not lost between the two.

### F-78 — CLOSED 2026-08-11 (post-merge polish: the display font's index caps at ASCII; substituted '|' at 5 sites) — "·" has no glyph in the display font, and four shipped screens use it (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10; re-assigned from the font cycle)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling). Was: ownerless residue; a font cycle, not a feature cycle. Still open; scheduled, not excused.

Measured 2026-08-07 in `gui`, pinned by `TestPlateLabelSeparatorRenders`:

```
width("ab") = 22    width("a·b") = 22    width("a|b") = 27
```

The middot contributes **zero pixels**. Four shipped files render it today:

- `gui/bundle_flow.go:339` — `"Card %d of %d · Plate %d of %d"` → `Card 1 of 3  Plate 1 of 2`
- `gui/codex32_polish.go:49,182,286` — `id NAME · thr 2 · share C` → double spaces
- `gui/slip39_polish.go:237`
- `gui/bundle.go:306`

**Why it has gone unnoticed, and why it still matters.** In all four the
surrounding *words* carry the meaning, so an invisible separator degrades to a
double space — sloppy, not wrong. B1's plate list was the first place it would
have been load-bearing (`mk1 2/3 · 1/2` → `mk1 2/3  1/2`, two fractions with
nothing saying which is the card), which is why it surfaced there and why B1 uses
`|` instead (operator decision, 2026-08-07).

**The real fix is the font, not the call sites.** Adding `·` to the display font
repairs all five places at once and lets B1 return to the separator §10.2.2's
examples actually use. Note this is the **display** font, not the engraving
alphabet — the 2-stroke-width minimum-feature rules do not apply.

Not done in B1: it is a font change, and substituting a different character at
each call site is treating the symptom in four places instead of the cause in
one.

**Why no test caught it, which is the more general gap.** `uiContains`
(`gui/gui_test.go:516`) asserts against **extracted text, not pixels** — it
lowercases the op tree's strings and does a substring match. A glyph that is
missing, blank, or drawn as the wrong shape is invisible to every screen test in
this package, because the text was *submitted* correctly regardless of what
reached the panel. The middot was found by **measuring width**, not by rendering,
and only because B1 needed to know whether a separator survived to the screen.

A rasterising check would close it: draw each glyph alone and hash the pixels,
so two characters that draw identically collide even though they compare unequal
as text. Worth having before any future font edit — a font change is exactly the
kind of edit whose defects this suite cannot see.

### F-77 — the encrypted section's md1/mk1 cards have no grouping (owning phase: **B2a-i, Task 1** — GATING, it blocks §10.2.2's secret plate labels)

**CLOSED 2026-08-10 — labelEncryptedCards at seal/label_encrypted.go:28, wired into AdmitSection at seal/record.go:266-268 and reached from production via seal/unlock_key.go:102; killed by deleting the wiring, which fails TestEncryptedSectionCardsAreLabelled and TestEncryptedMultisigCardsAreDistinguishable.**

B1's Task 4a surfaces `HRP`/`CardIndex`/`CardTotal`/`PlateIndex`/`PlateTotal` on
`AdmittedRecord` so the plate list can render §10.2.2's `mk1 1/2` /
`mk1 2/3 · 1/2` labels. **Those fields are populated for `SectionPublic` only**,
because pass 3 — `decodePublicSet` → `groupCards`, the sole place grouping is
computed — runs only for the public section (`seal/record.go:186`).

**And the encrypted section is full of cards.** SPEC §6.3: "The encrypted
section may carry anything — `ms1`, `mk1`, `md1`, a BIP-39 mnemonic."
`permitted()` (`seal/record.go:147`) codes it as `if c == ClassMDMK { return
true }` — unconditional, not gated on section. In `seal/testdata/vectors.json`,
vector C's secret set is `ms1`×1 / `mk1`×2 / `md1`×3 and vector F's is `ms1`×3 /
`mk1`×6 / `md1`×6 — **twelve of vector F's fifteen secret records are cards.**

So B2, which must label secret plates, will reach for grouping that was never
computed for its records. **Extend pass 3's grouping over the encrypted
section's `ClassMDMK` subset, reusing `groupCards`/`cardKey`.** Do NOT re-derive
classification in `gui` — that is the two-code-paths divergence
`Opener.Inspect`'s doc comment exists to prevent, and Task 4a rejects it for the
public section on exactly the same grounds.

Found by the B1 plan's R0 round 2, against a paragraph a previous fold had
introduced. Gating for B2 rather than optional: without it §10.2.2's labels are
unimplementable for any multisig payload.

### F-76 — inspecting a payload-sourced card (owning phase: **after B2b**; NOT B2a)

`mk1GatherFlow` (`gui/mk1_inspect.go:156`) and `md1GatherFlow`
(`gui/md1_gather.go:79`) prime a fresh gatherer with the single string handed to
them, and when that alone is not a complete card they open
`ctx.Platform.NFCReader()` and wait for the operator to tap the remaining
**physical tags**.

A payload-derived record has no tags. Every chunk is already sitting in
`p.Public` — the gatherer simply has no way to reach it. So Inspect on a chunked
payload record strands the operator on a scan-waiting screen, and chunked is the
ordinary case (single-sig's `md1` alone is 3 records; vector G's is a 6-chunk
card).

Found by the B1 plan's R0 round 0 (finding 3), which is why B1 composes
`validateMdmk` + `ChoiceScreen` + `NewEngraveScreen` directly instead of reusing
`mdmkFlow`. **B1 engraves; it does not inspect.**

**What would close it:** a gatherer that can be primed from an in-memory record
set rather than only from NFC. The data and the decode both already exist; this
is plumbing, not new codec behaviour, so the Rust-primary rule does not bind it.

### F-93 — the screensaver still PARKS a spec-legal derivation, and Run has to be the one to stop it (owning phase: B2b, with F-89's unwind)

**CLOSED 2026-08-10 — ctx.KeepAwake() at gui/unlock_kdf.go:334, reconciled with F-89 via the "&& !armed" term at gui/run_flow.go:251; killed independently by (a) removing KeepAwake (TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver fails, derivation parks under the screensaver) and (b) removing "&& !armed" (TestRunKeepAwakeCannotPostponeAnArmedWipe fails, armed wipe never fires).**

Found by the B2a-ii whole-diff review, lens 7 M1, and confirmed by measurement
during the Minor/Nit fold. **This is the residue the Critical's fix does NOT
close, and it is filed rather than folded because closing it needs a `Run`-side
change that must be reconciled with §10.2.4's residency timer — F-89's
territory.**

The C1 fold (`ctx.WakeupAt` before `ctx.Frame`, commit `051d423`) closes the
frame-1 park, where the derivation inherited the previous screen's
`idleWakeup` deadline and stalled for the full three minutes at ~0 %. That half
is fixed and tested.

**What remains.** `Run` refreshes `a.idle.start` only on `len(evts) > 0`, and a
derivation produces no events. So any derivation longer than `idleTimeout` trips
the saver, and the saver branch `continue`s **without breaking**, so `ctx.Frame`
never returns and `d.Step` is never called again. The derivation does not lose
its progress screen — it **stops**, until a touch.

Computed, not quoted:

| quantity | value |
| --- | --- |
| §7.1's measured rate | 9,715 it/s on RP2350 |
| `idleTimeout` | 180 s |
| iterations that reach it | 180 × 9,715 = **1,748,700** |
| §6.2's ceiling | 2,000,000 = **205.9 s** |
| §7.1's default | 300,000 = 30.9 s |
| at-risk share of the legal range | 251,301 of 1,900,001 = **13.2 %** |

So it is reachable with a **conforming** blob, not merely a hostile one, and
`me seal --iterations N` exposes the knob. It also breaks §7.1's own argument
that 205.9 s is *"long, but bounded, which is what the no-watchdog argument
requires"* — parked, it is unbounded without operator interaction.

**Severity, stated honestly:** Minor. Bounded, self-healing on a touch, no wrong
plate, no seed disclosure, and the tamper signal still fires afterwards. But it
is an operator-facing hang on the one screen §10.2 step 7 exists to keep
legible, so it is not cosmetic either.

**What is already done:** the §7.1 log line now reports **derivation** time
separately from wall time, so Task 9.3's number survives any park (it accumulates
`time.Since` around `d.Step` only). That was the half that could be fixed inside
`gui`. The code carries a comment at the `WakeupAt` naming this follow-up so the
fixed half is not read as the whole.

**What closes it,** either:

1. treat an in-progress derivation as activity — a `Run`-side change, and the
   one that must not be conflated with §10.2.4's residency timer: this is
   `Run`'s `idleTimeout`, which exists to blank the panel, not to protect a
   secret; or
2. bound the ACCEPTED iteration count below what `idleTimeout` allows. That is a
   normative change to §6.2's range and therefore lands in the Rust primary
   first, with vectors.

Option 1 is preferred and belongs with F-89, which already has to reconcile a
residency timer with a saver that does not unwind.

### F-94 — CLOSED 2026-08-11 (post-merge polish: the 64-byte seed and BIP-32 master key now pinned) — the 64-byte BIP-39 seed and the BIP-32 master key are unpinned, and the seam is cheap (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling). Was: B2c, with F-88; re-assigned from B2b 2026-08-09. Scheduled, not excused — same class as F-88/F-90/F-104: seed-equivalent copies inside the payload flow.

> **Re-assigned B2b → B2c, 2026-08-09.** The B2b plan deferred this to "own
> cycle", which is **not a later phase — it is no phase**, and
> `/scratch/code/CLAUDE.md` forbids parking an item on nothing: "an item that
> binds the current phase, or is scheduled *to* a phase, is not deferrable past
> its owning phase." Found by the B2b residue sweep (I4). The work is real and
> is not B2b-sized — F-88's only actionable copy is a `bip39.MnemonicSeed`
> change that five other flows call and that wants its own review — so it gets a
> **named successor phase** rather than a silent deferral or a silent scope
> increase. **B2c is secret-residency cleanup: F-88, F-90 items 1 and 3, F-94.**

Found by the B2a-ii whole-diff review, lens 2 M2 (mutants G28/G29/G30).

`3c477b9` added `defer wipeBytes(seed)` to `deriveMasterKey`, `defer mk.Zero()`
to `masterFingerprintFor`, and zeroing to the `SeedScreen` validity probe's
discarded key. **All three fixes are right, and every one can be deleted with the
suite green.**

The reason this is filed rather than folded is scheduling, not difficulty — and
the record has been corrected accordingly. `gui/unlock_session.go`'s inventory
used to say these "cannot be [pinned] without unsafe — they are internal to
functions that return neither". That is **false by the same file's own
precedent**: `unlockMnemonicHook` pins `m`, also a local, in a function that does
not return it, with an ordinary package var. A `var deriveSeedHook func([]byte)`
fired beside `seed := bip39.MnemonicSeed(...)` does the same, with no `unsafe`.

What makes it B2b's rather than B2a-ii's: `deriveMasterKey` and
`masterFingerprintFor` are **shared funds-path code** that this phase only
scrubbed in passing. Adding test seams to them widens the diff into
`bundleWalletFlow` and the seed-entry path, which this phase does not otherwise
touch.

**Also unpinned, and it stays that way with a reason:** `seal.Classify`'s
`clear(m)` on the SUCCESS path (mutant G27). `m` is a local in `seal` and the
allocation seam (`bip39.parseWordsHook`) is unexported in `bip39`, so pinning it
needs an **exported** seam in a Rust-primary ported package for a
defence-in-depth wipe. `Parse`'s three ERROR exits — the reachable half, where a
full near-seed was being orphaned — are pinned in `bip39` itself.

### F-95 — CLOSED 2026-08-11 (post-merge polish: warning copy shortened; maxScroll +19 -> -17) — §10.2.3's warning clears the panel by 3 pixels, and its scroll affordance does not exist on this hardware (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10; re-assigned from the font cycle)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling). Was: the GUI/font cycle, with F-78 and `seedhammer-warning-scroll-untouchable`. Still open; scheduled, not excused.

Found by the B2a-ii whole-diff review, lens 4 MINOR 4. **Pre-existing from B1** —
the B2a-ii diff does not touch `unlockWarnUnauthenticated`.

Measured twice independently (the reviewer's numbers and the fold's, identical),
at 480×320 with the real styles:

```
bodyClip=(6,44)-(423,314)  body=413x257  top=60  bottom=317  panel=320  maxScroll=19
```

Three facts stack:

1. `Warning.Layout` computes `maxScroll = 19 > 0` — the widget itself believes
   the last line is scrolled out of view.
2. Its **only** scroll input is `ButtonFilter(Up)` / `ButtonFilter(Down)`. The
   SeedHammer II has no directional buttons; `processTouch` emits
   `PointerEvent` exclusively. So the body is **unscrollable on the machine**.
3. Nothing is actually cut off today **only because `fadeClip` is a no-op stub**,
   with the real mask commented out three lines below it. Because it does not
   clip, the body renders past `bodyClip.Max.Y = 314` to y = 317, inside a
   320-pixel panel.

The paragraph in that 19-px window is §2.2 item 10's downgrade instruction —
*"the encrypted part has been REMOVED. Do not continue."* — the single sentence
that tells the operator to stop.

**What B2a-ii closed:** nothing pinned the fit. `TestUnauthenticatedWarningFitsThePanel`
now asserts `bodyClip.Min.Y + scrollFadeDist + bodysz.Y <= DisplaySize().Y` at
every legal record count, and is killed by one extra sentence of copy
(measured: y=353, 33 px over).

**What is still owed, and the order matters.** Restoring `fadeClip` *without*
shortening the copy makes this **worse**: it would begin enforcing the 19-px
overflow the stub currently hides, silently removing the instruction. So either
shorten the copy to fit `bodyClip.Dy() - 2*scrollFadeDist` **first**, or give
`Warning` a touch scroll (bind it to `Clickable`s with `op.Input` hit areas, the
same fix the StartScreen pager took) — then restore the clip.

### F-96 — the §11.3 mutation runner is uncommitted, so the 30-mutant run is reproducible by nobody — **CLOSED 2026-08-10** (owning phase: B2b)

> **The phase-report half is DONE:** `design/PHASE_REPORT_encrypted_payload_deviceB_phaseB2a_ii.md`,
> written 2026-08-09 at the operator's instruction. It consolidates B2a-ii's 11
> lens verdicts, the §11.3 row table, the whole-phase 30-mutant total (29 killed,
> 1 predicted survivor), the three rows that first reported a wrong verdict, and
> the measured green at merge.
>
> **It also records what could NOT be recovered.** `3db3bfe` said the runner "is
> reproduced in the phase report"; the report was never written, so the script is
> gone — written inline, used, discarded with the tool call. The results survived
> because they were written into a commit message; the procedure did not.
> B2a-ii's rows are therefore **not re-runnable as they were**. B2b Task 7's
> `scripts/mutation-run.py` supersedes it and derives its rows from the plan's own
> tables, so the check is a command rather than a discipline.
>
> The runner half stays open until Task 7 commits.

Found by the B2a-ii whole-diff review, lens 5 M3, and it is a standing-rule
violation rather than a defect: `CLAUDE.md` says *"when an artifact will be
folded repeatedly, commit the extractor as a script so the check is a command,
not a thing to remember."*

Commit `3db3bfe`'s message says *"The runner is NOT committed here … It is a
single self-contained Python file and is reproduced in the phase report."*
Measured: `ls design/agent-reports/ | grep b2a-ii` returns the lens files and no
B2a-ii **phase report**, and `scripts/` holds `plan-build-gate.sh`,
`plan-build-gate-go.sh` and `plan-cite-gate.sh` and no mutation runner. The only
other mention in the repo, `design/agent-reports/MUTATION_planB_phaseA.md:15`,
records that the Phase A runner also lived in a scratchpad.

**Why it bites next, specifically.** B2b owns §10.2.4 plus the F-89 unwind — the
phase most in need of re-running these exact rows — and will re-derive the
runner with a different notion of "the substitution matched" than the one this
phase had to fix twice mid-run (rows 6.1/6.5, 6.7).

**Fix:** commit it as `scripts/mutation-run.py` with the row table as data, and
have it print what it does **not** cover, the same shape as
`plan-build-gate-go.sh`. Land it with the phase report if that is still owed.

*(The Minor/Nit fold that followed ran its ~20 mutants by hand for this reason,
each substitution asserted to match exactly once and each file restored from a
file copy. That is the discipline the script exists to make cheap; doing it by
hand is what the rule is trying to stop.)*

**CLOSED 2026-08-10 by the B2b follow-up reconciliation sweep.** The entry's own
text said the runner half "stays open until Task 7 commits". Task 7 committed it:
`scripts/mutation-run.py` at **`dd3d4b3`** ("tooling: mutation-run.py -- run the
plan's own §11.3 rows (F-96's runner)"), 26,827 bytes, present in the tree today.
Both halves are therefore done and the sentence claiming otherwise was stale.

Recorded as its own lesson: this item read OPEN for a week purely because nobody
re-read it after the commit that satisfied it. Records have been the wronger half
of this project throughout — see the reconciliation report,
`design/agent-reports/2026-08-10-b2b-followup-reconciliation.md`.


### F-97 — CLOSED 2026-08-09 — plan and record corrections owed to the B2a-ii artefacts

**Closed the same day it was filed.** It was filed as "NOT foldable from the
firmware worktree", which was true of the agent that filed it — every item lives
in `mnemonic-engrave/design/`, which was read-only to it. It was never true of
the controller. **A follow-up filed because of a sandbox boundary is not a
deferral, and the register should not carry one.**

The line counts are now `wc -l` output (304 / 237 / 83, measured by the build
gate) rather than a hand-count; §7c's "each frame" clause and mutation row 7.6
now say what `relabel()` actually does. The third item was already discharged
when filed.

Small, real, and NOT foldable from the firmware worktree — every item is in a
`mnemonic-engrave/design/` artefact rather than in code.

- **Two of three gui file line-counts in the plan are wrong** (lens 5 N2a). The
  plan's "Beyond the gate" section at line 1696 claims *"`gui/unlock_kdf.go`
  (247 lines), `gui/unlock_session.go` (184) and `gui/unlock_plates.go` (83)"*.
  Measured with `wc -l` on the plan's own blocks: **291, 237, 83**. Harmless
  downstream — the shipped files were byte-identical to the blocks the gate
  compiled — but it is a hand-count where a tool was available, in a GREEN plan,
  which is the one thing `CLAUDE.md` names by name. Correct them with `wc -l`
  output or drop the numbers.
- **Plan §7c's "each frame" clause is wrong, and so is mutation row 7.6**
  (lens 5 M2's other half). §7c mandates that the plate list *"builds its labels
  with `unlockPlateLabel(…)` **each frame**"*. It does not, and never did:
  `relabel()` is called on entry and after each engrave. The in-code comment has
  been corrected in the firmware; the plan clause and the 7.6 row still say it.
  The mutant 7.6 actually kills is *"the post-engrave `relabel()` deleted"*.
- **The Rust-primary check for the `bip39` fold was not recorded** (lens 5 N2b).
  `d0baf13` fixed a defect found in a ported Go package and the commit does not
  say the Rust check was made. **This is now discharged, not owed:** the
  Minor/Nit fold's `bip39` commit records it in full — the same shape exists in
  `mnemonic-toolkit/vendor/bip39/src/lib.rs`'s `parse_in_normalized`, but as a
  stack array in a vendored third-party crate, and the change is memory hygiene
  rather than normative behaviour, so the rule's "land in Rust first" does not
  bind. Recorded here only so the gap in `d0baf13`'s own message has a pointer.

### F-98 — two citations in the GREEN spec do not resolve (owning phase: with F-85, before the release tag)

**CLOSED 2026-08-10 (`3be5fc8`).** The repaired gate separated the two cases the old one conflated: `checksum.go:132` was AMBIGUOUS, not wrong — the claim is right and it needed `codex32/` prefixing. `main.rs:375` had genuinely decayed and is repointed to `crates/me-cli/src/main.rs:590` (`fn write_private`) with `:597` for the `0o600`. Both verified by the gate printing the resolved line.

Found while cite-gating the §10.2.4 amendment; **pre-existing, and unchanged by
it** — the gate reports the same two before and after, verified against
`git show HEAD:`.

```
FAIL  checksum.go:132   file has only 89 lines
FAIL  main.rs:375       file has only 146 lines
```

Ordinary citation decay, and exactly what `plan-cite-gate.sh` exists to surface.
It matters more here than in a plan because **`SPEC_encrypted_payload_delivery.md`
is GREEN and normative**: a reader resolving either citation to check a claim
lands nowhere, and the spec's own history includes a stale cite
(`driver/otp/otp_rp2350.go:13`, a `#define`) that survived nine review rounds
because every reviewer read it as authoritative.

Fix is to re-resolve both against the current source and correct or drop them.
Bundle with F-85's §2.2 amendment so the GREEN spec is opened once, not twice.

### F-99 — CLOSED 2026-08-09 — §10.2.4 row 1 did not fix WHEN the warning starts

**Closed by `7c3a625`**, operator-approved: §10.2.4 now states that the 30 s is
**additive** — the warning appears at 3:00 and the wipe fires at 3:30 — and names
the rejected alternative (warn@2:30/wipe@3:00) so it is not silently re-opened.
The same commit fixed a markdown defect the 2026-08-09 amendment had introduced:
the table's third row ("no secret record resident → none") had been orphaned
below the amendment paragraph, outside the table it belongs to.

B2b Task 8 is unblocked. Original entry follows.

### F-99 (original) — §10.2.4 row 1 does not fix WHEN the warning starts (owning phase: B2b Task 8 — blocking, needed operator sign-off BEFORE the hardware run)

Found by the B2b R0 design lens (opus, round 0). §10.2.4 row 1 reads
"**3 min**, 30 s warning", which is genuinely ambiguous between:

- **warn @ 3:00, wipe @ 3:30** — the idle timeout starts the warning, and the
  30 s runs on top. This is what `IMPLEMENTATION_PLAN_..._phaseB2b.md` builds
  (`wipeAt := idleWakeup.Add(wipeWarningDelay)`), and it reuses `idleTimeout`
  unchanged, which is the amendment's "the timer VALUE and time source are
  reused" point.
- **warn @ 2:30, wipe @ 3:00** — 3 min is the deadline and the warning is the
  last 30 s of it.

The plan commits explicitly, which is the right thing for a plan to do, but the
**spec** does not — and Task 8.1 has the operator confirm "the warning at 3:00,
the wipe at 3:30" on real hardware. A hardware pass that blesses a reading the
normative text never chose converts an ambiguity into a fait accompli, which is
precisely how the stale `driver/otp/otp_rp2350.go:13` cite survived nine rounds.

Fix: one amending sentence in §10.2.4 row 1 naming the chosen reading. **Not to
be folded by the plan author** — it is a normative change to a GREEN spec and
needs the operator's sign-off, like the 2026-08-09 amendment it sits beside.
Bundle with F-85 and F-98 only if that happens before Task 8; otherwise it goes
first, alone, because Task 8 is gated on it.

### F-100 — CLOSED 2026-08-09 — SPEC §11.5's "confirm firmware reflash preserves the blob"

**Closed on real hardware the same day it was filed.** Vector F's sealed payload
was loaded FIRST, the B2b firmware flashed second, and the start screen then
showed Sealed Payload present with **9 pager dots** (B1 baseline: 8 absent, 9
present). Recorded in `design/HARDWARE_RESULT_2026-08-09_phaseB2b.md`.

It cost nothing but the ordering — which is why the preflight's fix was to put
"payload first, firmware second" into Task 8's setup rather than to add a step.
Original entry follows.

### F-100 (original) — SPEC §11.5's "confirm firmware reflash preserves the blob" has never been run and is owned by nobody

Found by the B2b residue sweep, which asked the completeness question "what is
required before a tag, is named somewhere in the corpus, and appears in NEITHER
the plan's task list NOR its explicit 'does NOT cover' list" — silence being the
defect, because an item in neither list has no owner.

SPEC §11.5 requires confirming that **reflashing the firmware preserves the
payload blob**. Nothing has run it:

- `HARDWARE_RESULT_2026-08-07_phaseB1.md` covered exactly four things — write and
  read-back at the normative address, §10.1 negative, §10.1 positive + §6.6, and
  present→absent. Reflash-preservation is not among them. Its closest statement
  is the **converse**: "Only the 64 KB payload region was cleared; B1's firmware
  was untouched."
- B2a-ii's Task 9 does not cover it either — 9.1–9.2 load the payload *after* the
  firmware.
- `grep -rn "11\.5\|reflash preserves" design/` finds no plan step and no
  follow-up.

It matters because the blob's whole value proposition is that it outlives a
firmware update; if it does not, the feature's storage model is wrong in a way
no host-side test can reach.

**Secondary, same section:** §11.5 also specifies booting on **PD power**, which
neither B2b Task 8 nor B2a-ii Task 9 names. `cmd/controller/platform_sh2.go`
sets `minVoltage = 20_000` and calls `monitorPowerSupply` before display init,
so a non-PD supply is a different boot path.

Recorded in the B2b plan's "release tag's precondition set", which is now the
single place that list lives.

### F-101 — `mutation-run.py` is not crash-safe: killed mid-row it leaves a MUTANT in the worktree (owning phase: before the release tag, with F-96's runner)

**CLOSED 2026-08-10 (`ba31e0c`).** An on-disk sentinel written before each mutation and cleared only after the restore verifies, plus handlers for SIGINT/SIGTERM/SIGHUP; `recover_sentinel()` runs BEFORE `preflight_clean()`, because the mutant a kill leaves is exactly what makes preflight refuse. Verified by SENDING the signals (`scripts/test/mutation-run-crashtest.py`): SIGKILL leaves the mutant and the next run restores it; SIGTERM restores in-handler and keeps an honest exit status.

Hit three times in ten minutes on 2026-08-09 while verifying Task 7. A
backgrounded run was killed mid-row and left `armed := true` applied in
`gui/run_flow.go` — the mutant that makes §10.2.4's wipe timer **permanently
armed**, which would erase secrets during the *public* plate list.

**Why it is more than an annoyance:** the file it leaves behind **compiles**, and
passes most of the suite. A `git add gui/run_flow.go` at the wrong moment commits
a funds-safety guard that has been disabled, and the diff is one plausible-looking
line inside a function nobody re-reads.

**What already works, and is the only reason this surfaced as a refusal rather
than a false green:** the runner's pre-flight cleanliness check. Its own words —
*"a mutation's restore would be indistinguishable from a pre-existing change"* —
so it refused to start on the dirty tree instead of running 16 mutations on top of
a live mutant and reporting a plausible all-green table.

**Fix:** trap `SIGINT`/`SIGTERM` and restore before exiting; and write a sentinel
next to the backup recording the in-flight file, so a later invocation can *offer
to restore it* rather than only refusing. Until then, wrap any invocation in a
guard that restores unconditionally:

```sh
trap 'cd $WT && git checkout -- .' EXIT INT TERM
```

Note the wrapper is only safe because the worktree carries no legitimate
uncommitted work — the general rule (restore from a file copy, never
`git checkout`) still stands for anything else.

### F-102 — `me seal` takes SEED MATERIAL on argv, while every other subcommand reads stdin (owning phase: before the release tag)

**CLOSED 2026-08-10 (`2ed6ac1`).** `me seal` takes `--in <file>` or stdin into a Zeroizing buffer; argv survives for fixtures and warns when it carries seed material. Verified across every channel including CRLF-refused-not-normalised and warn-then-still-refuse without `--seal-secret`. SPEC §2.2 item 14 names the host-side exposure, which the spec had never mentioned.

Raised by the operator 2026-08-09 while reading Task 8's setup, and measured
rather than assumed.

**The inconsistency:** the subcommands that *refuse* secrets read from a private
channel; the one that *requires* `--seal-secret` does not.

| subcommand | input |
| --- | --- |
| `me convert` | stdin or `--in` — its doc comment even says *"Refuses secret ms1"* |
| `me bundle` | file or stdin |
| **`me seal`** | **`payload: Vec<String>` — argv only** (`crates/me-cli/src/main.rs:71`) |

**Measured exposure on the author's own workstation:**

- `/proc` is mounted with **no `hidepid`**, so `/proc/<pid>/cmdline` is
  world-readable — any local process can read a running `me seal`'s arguments.
- `~/.local/share/fish/fish_history` is `-rw-r--r--` and 517 KB. A record typed
  at the prompt persists there, world-readable, indefinitely.

Vector F is 1354 bytes of `ms1`/`mk1`/`md1` across 15 records — all of it seed
material by construction, which is precisely what `--seal-secret` exists to gate.

**The irony worth recording:** `--out` is `required = true` and documented
*"never stdout, because the passphrase shares that stream"* — so the tool guards
its OUTPUT channel scrupulously and leaves the INPUT channel wide open to `ps`,
`/proc` and shell history.

**Not a Task 8 problem.** Vector F is a public test fixture; sealing it on argv
leaks nothing. This binds before anyone seals a REAL seed, which is what a
tagged release invites.

**Fix:** give `me seal` the same `--in`/stdin path the other subcommands already
have (newline-separated records, read into a `Zeroizing` buffer, which
`main.rs:158` already does for `convert`). Keep argv working for fixtures and
tests. Consider warning to stderr when a secret-classified record arrives via
argv. The spec is silent — `grep -niE "argv|command.?line|/proc"` over
`SPEC_encrypted_payload_delivery.md` returns nothing — so §12's threat model
should gain a line either way.

### F-103 — CLOSED 2026-08-11 (post-merge polish: idle clock now keyed on EFFECTIVE input; narrowed, see F-122) — the PROTECTIVE SCREEN FILM silently disables §10.2.4's wipe, the screensaver, and every idle behaviour (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling; the concerns below were raised and the operator decided). Was: B2c, and it belongs in the operator docs before any release.

**MECHANISM CONFIRMED 2026-08-10, and the entry is both under- and
over-stated.** `design/agent-reports/2026-08-10-f103-screen-film-mechanism.md`.

**The mechanism:** `gui/run_flow.go:251` refreshes `a.idle.start` on raw
`len(evts) > 0`, with no requirement that an event resolve to *effective* input.
Touch readings that `processTouch`'s exact-equality dedupe fails to suppress keep
the machine perpetually not-idle; §10.2.4's warning branch is nested inside
`if a.idle.active`, so a machine that never goes idle never warns. Silent, as the
entry says. `ctx.keepAwake` is NOT the cause while armed — its term is ANDed with
`!armed`.

**Host-testable, and tested:** 100,000 distinct spurious touch polls over ~1000 s
of fake time under `synctest` produced zero warnings and zero wipes, while an
identical control platform without spurious events warned at ~3:00.

**Under-stated:** this is not about film. ANY source of spurious touch events
does it — moisture, debris, driver noise.

**Over-stated:** the 2026-08-09 incident's attribution to the film was never
confirmed with an event counter, the way F-106's phantom-input hypothesis was
*refuted* by one (`e` stayed 162). The hazard is confirmed live in code; that
particular incident's cause is plausible and unproven — and F-106 turned out to
be a different, event-free bug behind an identical-looking symptom.

**Smallest fix:** refresh the idle clock only on input that resolves to a
state change, or cap how often raw events may refresh it.

**Observed on real hardware 2026-08-09**, during B2b's Task 8.1, and diagnosed by
the operator. This is the first thing the hardware pass found, and it is not a
code defect — the code did exactly what it was told.

**What happened.** Sitting on the *Cut this plate / Skip* screen — which IS armed
(guard installed, no engrave job registered, so `armed()` is true) — **nothing
fired at 3:00 or 3:30. At 4:05 the screen was unchanged.**

**The diagnostic that located it in one step:** the **screensaver had not appeared
either**, then or at any earlier point in the session. Both run off the same
`a.idle.start` and the same 3:00 `idleTimeout`. So the machine did not believe it
was idle — which points at the clock being *refreshed*, not at the arming logic.
While armed, the `ctx.keepAwake` term is gated `&& !armed`, so the only remaining
refresh source is `len(evts) > 0`.

**The cause: the factory protective screen film was resting on the panel**,
generating a continuous stream of touch events.

**Why it is worse than it looks.** The idle clock keys on **any event, not on
effective input**. Panel noise that never resolves to a click — too brief, too
weak, below the press threshold — still satisfies `len(evts) > 0` and refreshes
the clock. So the UI remains perfectly usable (the operator unlocked, typed
twelve words and navigated normally) while **every idle-driven safety behaviour is
silently and permanently disabled**. There is no indication on screen. Nothing
logs it. The machine simply never goes idle, forever.

**And the film ships on the device.** An operator who never peels it gets a
machine on which §10.2.4's wipe — a funds-safety control — does not exist, and
who has no way to discover that short of timing it.

**PREDICTED.** The pre-hardware preflight listed exactly this in its accepted-risk
set: *"an object resting on the panel may refresh the clock forever (unmeasured on
real hardware — free bench check)."* It was recorded as a risk to know about
rather than a blocker, and the bench check was never run. **The very first
hardware step hit it.** Worth remembering the next time an accepted risk is cheap
to actually test.

**Candidate fixes, none of them decided:**

1. **Operator documentation** — "remove the screen film" in the setup runbook.
   Necessary, and on its own insufficient: it makes a silent funds-safety failure
   depend on someone reading a document.
2. **Key the idle clock on EFFECTIVE input rather than any event** — a click, a
   completed press/release, a router-consumed event — rather than
   `len(evts) > 0`. This is the real fix and it is a normative change to
   §10.2.4's "any touch refreshes it", so it needs the R0 loop, not a patch.
3. **Detect the pathological case** — a panel asserting continuously for far
   longer than a human touch could plausibly last is itself a signal, and could
   refuse to count.

Option 2 is the one that makes the guarantee real; note it also affects the
screensaver, which is upstream code this phase only borrowed.

### F-104 — four MORE members of the unreachable-seed-residue class, two of them on paths nobody had enumerated (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10; re-assigned from B2c)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling). Was B2c. This entry **still binds** — see the traced call graph above; §2.2 item 12 does not accept it — it is now scheduled later, not excused.

**STILL BINDS — checked against SPEC §2.2 item 12 on 2026-08-10 and NOT accepted.** all four: pbkdf2 state via F-88's chain; `splitMnemonic` residue via the classifier on `Inspect` and `UnlockWithKey` and via `unlockPassphraseFlow`'s `LastWordCandidates`; the `ms1` `ToUpper`/QR copies via `unlockEngraveCodex32` → `backup.EngraveSeedString`; keyboard fragments via `unlockPassphraseFlow` → `inputWordsFlow` → `Keyboard.Fragment`. Verified by tracing the call graph whole-tree, not by reading this entry: an earlier pass classified this as legacy-only from entry prose and was wrong. See `design/agent-reports/2026-08-10-b2c-program-boundary-verification.md`.

Found by the wipe-inventory audit (fable, 2026-08-09), which was dispatched to
answer the operator's question *"are we wiping an incorrect portion of memory, or
have we checked?"*. It found no second `Reset()`-class defect — that part is
**measured**, see the audit test at `a73191a` — but it did find four residues in
the accepted F-83/F-88 "unreachable heap garbage" class that **no prior inventory
recorded**:

1. **`x/crypto/pbkdf2`'s HMAC state**, holding the plaintext mnemonic
   **XOR-recoverable**. This is inside a dependency, on the funds path, and no
   wipe reaches it.
2. **`splitMnemonic`'s `math/big` and `entBytes` residue** — created by the
   **classifier**, on **every unlock**, and roughly **2,048×** over the passphrase
   prefix during last-word entry. `entBytes` is the one **zeroable `[]byte`** in
   this group, so it is the one with a cheap fix.
3. **The `ms1` arm's per-ranging `ToUpper` copies and its QR** — **missing from
   F-90's own enumeration**, on the arm F-90 itself calls *"the DEFAULT arm"* that
   six of seven vectors take.
4. **Keyboard fragment strings.**

Item 3 is the one that should sting: F-90 exists *because* the `ms1` arm was
under-examined, and its enumeration was still incomplete. That is F-88's lesson
recurring — *"a complete inventory that wasn't"* — and it is the reason B2c must
not simply re-read the existing lists.

**Condition on B2c, from the audit:** B2c must absorb these four rows **and** land
F-94's seam **first**, because those wipes remain silently deletable until it
does. Without both, B2c's inventory repeats the same failure a third time.

### F-105 — a typed passphrase is wiped by NOTHING until it is submitted (owning phase: **B2b Task 9** — operator ruling 2026-08-09)

**CLOSED 2026-08-10 on hardware** — reading 3 of the B2b gate: two words typed at the passphrase keyboard, warning at 3:00, wipe at 3:30. Task 9.5 was the only part still owed. `design/HARDWARE_RESULT_2026-08-10c_b2b_gate.md`.

> **OPERATOR RULING 2026-08-09: an in-flight passphrase IS seed-equivalent** — it
> derives the key that opens everything. That makes this a **defect**, not the
> design boundary the entry below called it, and it moves from B2c into **B2b as
> Task 9**. §10.2.4's scope is wrong, not merely narrow, and needs amending.

Found by the wipe-inventory audit, 2026-08-09, and the most consequential thing it
found.

**§10.2.4's bracket opens only AFTER decryption.** `unlockSecretSession` installs
the residency guard once there are secrets to protect. So on the passphrase entry
keyboard — *before* unlock — there is no armed timer, and the partial-exit
`clear(m)` needs a **Back tap that a parked flow never delivers**.

**Consequence.** An operator who types twelve words, is interrupted, and walks
away leaves the passphrase resident indefinitely, with **the sealed blob sitting
in flash beside it**. That is exactly the walk-away threat §10.2.4 was built for,
**one screen too early** — and the screen where the operator is most likely to be
interrupted, because entering twelve words on a touch keyboard is the longest
manual step in the flow.

It is a **design boundary**, not a defect: nothing promises this today, and
§10.2.4's own text scopes the timer to resident *records*. That is why it is filed
rather than treated as a regression. But the boundary is in the wrong place, and
the fix is not obviously small — arming before there is anything decrypted means
the guard's whole "resident secret" predicate needs rethinking.

Worth deciding deliberately: **is the passphrase in-flight seed-equivalent?** It
derives the key that opens everything, so a reasonable reading is yes.

**FIX LANDED — b2b `749fce7` (Task 9). Hardware validation (Task 9.5) is the only
part still owed.** Re-verified by reading the shipped code 2026-08-10:

- The bracket is `unlockPassphraseFlow`'s **own lifetime**
  (`unlock_kdf.go:135-137`), covering both the per-attempt retry and the
  checksum-retry loop, and it **closes on every return path via `defer` — before
  `unlockAttemptOnce`, therefore before `unlockDerive`**. That closure is row 5.
  It is not a flag on `wipeGuard`: arming across the KDF is unsurvivable, since
  Run's warning branch draws and `continue`s without returning control, so a
  derivation reaching 3:00 freezes for the full 30 s window and the wipe becomes
  certain — ~1,343,284 iterations, **34.6% of §6.2's legal range, permanently
  un-openable on the device**.
- Every copy is accounted for: `m` is cleared on each exit path inside the flow
  and again at `unlock_kdf.go:409` after the attempt; `pass` and `key` are both
  under `defer clear` in `unlockAttemptOnce`. `unlockAttemptOnce` returns before
  the secret session starts, so the typed words are gone before any record is
  displayed.
- The remaining unprotected window is **the derivation itself**, which is row 5
  as specified rather than a gap.

**Interaction with F-106, which decides how to test this.** F-106 barely affects
row 4: reaching the passphrase keyboard *requires* touching the screen, so that
window is always started by a real event. Row 1's post-unlock walk-away has no
guaranteed touch near it, which is why the defect surfaced there and not here.
That makes "type two words, stop, wait 3:30" a **discriminating experiment for
F-106** as well as Task 9.5's own check — see the bench card in
`design/DESIGN_f106_idle_timer_never_starts.md`. If that warning does **not**
appear, it is an F-105 defect in its own right and not merely F-106 spilling over.

### F-106 — §10.2.4's window runs 2x (6:00, not 3:00): a LATE ARM EDGE lands on the deadline (owning phase: **B2b — CRITICAL, gates the phase**)

**FIX WRITTEN AND R0-FOLDED 2026-08-10.** Design:
`design/DESIGN_f106_late_arm_edge.md`. Implementation: worktree
`seedhammer-f106`, branch `b2b-f106` @ `4b452d3` (off `b2b` @ `3de8aa1`).
Round 0 report `design/agent-reports/2026-08-10-r0-f106-round0.md` (RED, 1C/2I)
persisted at `e29d61b`; fold at `62b7917`. Round 1 re-review dispatched.

*The fix:* process the arm edge BEFORE the loop blocks as well as after it. The
pre-block call is worth a wakeup and is the fix; the post-block call is worth one
loop turnaround, is not load-bearing, and the code now says so. Engrave-side
edges are covered by neither call — `pl.Wakeup()` covers those, which the design
now states and a mutation row now pins.

**CLOSED 2026-08-10 on hardware**, build `v0.0.0-g747cf48`. Both required
readings green — `design/HARDWARE_RESULT_2026-08-10c_b2b_gate.md`:

1. Cut/Skip untouched → warning **3:00**, wipe **3:30** (was 6:00/6:30). The 2x
   is gone.
2. Back mid-cut → warning **3:00 after the head stops**, wipe +30 s. This is the
   `engraveStopping` park R0 round 0 raised as I2, whose only exit is
   `pl.Wakeup()`, and the device is the only place it could be settled.

No host test could close this, because the defect is a function of when
`platform_sh2.go`'s event source actually returns.


**ROOT CAUSE FOUND on hardware 2026-08-10**, build `b2b-idleprobe3` =
`256b38c`. Full readings and reasoning:
`design/HARDWARE_RESULT_2026-08-10b_f106_ROOT_CAUSE.md`. The original title
("never starts unless touched") is **wrong** and is kept above only as history.

**The measurement.** On Cut/Skip, untouched: `idle 0s w151 t770 e162 A!` — armed
reads TRUE live while the tracked value still disagrees, i.e. **the edge is
pending**. Three minutes later, at exactly 3:00: `idle 0s w170 t771 e162 A` —
site **170** (row 2's armed edge) wrote the clock and the `!` cleared. Warning
then animated at 6:00 and the wipe fired at 6:30.

**`t` advanced by exactly ONE across those three minutes** (770 → 771):
`AppendEvents` blocks, so the loop was parked and woke once — on the 3:00 idle
deadline — and that wakeup was consumed processing the pending edge.

**`e` never moved (162 throughout, t770 → t771 → t803).** Zero events arrived.
So `Pu0,0` is a STALE record of the last real touch, and the phantom-input
hypothesis — the probe's own first decision row, which requires `e` to climb —
is **refuted**. (`processTouch`'s dedup at
`cmd/controller/platform_sh2.go:398-402` compares `tp` even when `touching` is
false, which is a real latent fragility worth its own item, but it is NOT
F-106: nothing was generating events at all.)

**Mechanism.** `gui/run_flow.go`'s inner loop samples `armed := ctx.wipe.armed()`
only AFTER the blocking `AppendEvents` returns. The guard is installed during
the flow's own execution, so the edge goes pending; nothing wakes the loop on an
arming change; the next wakeup IS the idle deadline; row 2 then stamps
`a.idle.start = now` at the exact instant the wipe should have fired, and a full
fresh window runs. Deterministic 2x.

**The edge is also spurious.** `wipeGuard.armed()` is true as soon as `g != nil`
with no job running, and at Cut/Skip `g.job` is not set until the `Engrave`
call — so the FIRST transition is guard installation, not a finished cut, yet
row 2's "a finished cut starts a FRESH window" is applied to it. Processed on
arrival it would have been a harmless reset at t~0; **the damage is entirely
that it lands 3:00 late.**

**Fix is NOT yet written** and needs its own R0 pass — it changes §10.2.4's
timing on a secrets-residency control. Directions: process the arm edge BEFORE
the blocking read as well as after (arming can also change during the block when
a job finishes on the engrave goroutine, so both are needed); and/or seed
`a.armed` at guard installation so installation is not an edge.

**Trap for the test.** `gui/idle_realclock_diag_test.go` on `b2b` reproduces
`platform_sh2.go`'s timer structure line for line and the warning lands at
**3m0s, ticks=2, evtTicks=0** — the host harness does NOT reproduce this bug,
because its loop wakes often enough that the edge is never pending across the
deadline. A passing host test proves nothing unless it reproduces the PARKED
LOOP: a single `AppendEvents` call spanning the whole window.

<details><summary>Original filing (title superseded above)</summary>


Measured on hardware 2026-08-10. **Pre-existing, not a regression** — the operator
confirms every earlier successful test involved an inadvertent touch after
unlocking, which is why it was invisible until the two were deliberately
separated.

| sequence | result |
| --- | --- |
| unlock → **touch** → wait | warning at **exactly 3:00**, wipe at **exactly 3:30** |
| unlock → **touch nothing** → wait | **nothing at 4:15** |

**This defeats the feature's entire purpose.** §10.2.4 exists for one scenario —
unlock, be interrupted, walk away — and that is exactly the scenario with no
timer. It is strictly worse than the post-wipe hang: the hang is loud and the
operator knows something is wrong, whereas this is **silent**, and the machine
looks like it is protecting secrets it is not protecting.

**What is sound:** the schedule. Once started the window is exact — 3:00 and 3:30
to the second, repeatedly. The arithmetic, the warning, the countdown and the
unwind all work. The defect is entirely in **when the window begins**.

**Where to look.** `a.idle.start` has three refresh sources:
`len(evts) > 0`, the `armed` false→true edge, and `ctx.keepAwake && !armed`. The
armed edge is *supposed* to set `a.idle.start = now` when the guard installs, so
the window should begin at session start. Either that edge is not firing as
believed, or something refreshes the clock continuously until a real touch
arrives.

Worth checking first: `ctx.keepAwake` is set every slice during the KDF, when the
guard is not yet installed (`armed == false`), so the gate `&& !armed` permits it.
What clears it, and when, relative to the guard installing.

**This is the third finding in a row that only hardware could produce**, after
F-103 (the screen film) and the post-wipe hang — and the second whose earlier
"pass" was an artifact of how the operator happened to interact with the machine.

**UPDATED 2026-08-10 — the host is exonerated, and two of the leads above are
dead.** Full analysis in `design/DESIGN_f106_idle_timer_never_starts.md`.

*The `keepAwake` lead is closed.* `ctx.KeepAwake()` has exactly one caller in the
tree and it is the derivation, which is not running on Cut/Skip; the `&& !armed`
gate excludes it there besides:

```
$ grep -rn "KeepAwake()" --include="*.go" . | grep -v _test.go
gui/unlock_kdf.go:327:		ctx.KeepAwake()
```

*The "write the failing host test first" lead is closed too* — that test already
exists and passes. `TestRunSealedPayloadReentryAfterWipe/F_idle-wipe_nfc` drives
the **real** `uiFlow` through a real unlock, parks on Cut/Skip, delivers **no**
further events, and sees the warning at 3:00 and the wipe at 3:30.

A new opt-in diagnostic then removed the last two host substitutions at once —
real wall-clock time, and an `AppendEvents` structured like
`platform_sh2.go:369` (reused `*time.Timer`, select over timer/wakeup/touch):

```
    idle_realclock_diag_test.go:142: warning drawn at 3m0s (ticks=2 evtTicks=0)
    idle_realclock_diag_test.go:165: elapsed=3m30s sessions=2 ticks=32 evtTicks=0 longestDeadline=3m0s
--- PASS: TestIdleTimerUnderSH2ShapedEventLoop (210.09s)
```

Zero events across the whole run, and the window opened on time anyway.

*What remains.* Since the post-touch run proves the mechanism works end to end,
the only state that can differ is `a.idle.start`, assigned at exactly three sites
(`run_flow.go:48`, `:151`, `:170`). So it was either **continuously refreshed**
(A1: phantom input, which this panel has a history of — F-103; or A2: `armed()`
oscillating) or **set into the future** (B: a bad `time.Now()` read). One signed
number on the panel separates them, which is what branch `b2b-idleprobe` draws.

**UPDATED 2026-08-10 (second measurement) — it is not "never starts", it is a
DOUBLED WINDOW, and it is deterministic.** Three consecutive cycles on
`b2b-heapprobe2` (fixes C and D), timed from video:

| | measured | expected |
| --- | --- | --- |
| Cut/Skip → warning | **6:00** | 3:00 |
| warning → wipe | **29–30 s** | 30 s |

The second half is exact, so `wipeWarningDelay` and the arithmetic are intact.
The first half is consistently **2 × `idleTimeout`**. That is what an armed edge
landing at +3:00 produces: `run_flow.go:170` sets `a.idle.start = now` on the
false→true transition, putting the warning at +6:00 and the wipe at +6:30 —
6:29 and 6:30 were measured.

**This probably UNIFIES the original observation.** Yesterday's "4:15 and
nothing" is exactly what a 6:00 window looks like if you stop waiting at 4:15.
So the earlier reading — *the timer never starts* — may always have been *the
window is twice as long*, and this morning's clean 3:00 the outlier rather than
the norm. **No screensaver at +3:00 is expected and is not a clue**: while armed
the warning takes the screen the saver would have had; they are one branch and
can never both run.

**Open:** why `armed()` flips false→true at +3:00 rather than at unlock.
`b2b-idleprobe3` (the overlay rebased onto fixes C and D, built and green) prints
the field that decides it — `w170` means the armed edge rewrote the clock,
`w151` with `e` climbing means an event did.

*Cheapest next step, no flash required:* leave the device on the main screen,
untouched, for 3:30 and see whether the **screensaver** appears. The refresh
condition is upstream's own (`a01b666` has `if len(evts) > 0` and nothing else),
so that question is about the base firmware, not this phase — and it halves the
search either way.

</details>

### F-107 — the RENDERED seed is scrubbed ONLY on the wipe path; a normal exit leaves the twelve words in `ctx.B` (owning phase: **B2b — CRITICAL**)

**CLOSED 2026-08-10** — implemented on `b2b-residency` (`ctx.B.Scrub()` in both unlock brackets, pinned by `gui/unlock_session_scrub_test.go` and `gui/residency_wiring_test.go`), GREEN through three R0 rounds and a whole-diff review, and confirmed on hardware by reading 4c (abort→resume inside the secret session).

Found 2026-08-10 by an operator question — *"a normal exit reuses the Context, but
does a normal exit zero secrets?"* — which is a better question than the answer I
first gave it. My reply said a normal exit "means `runWithFlow` returns outright,
so the Context, its buffer and the drawer all become garbage together." **That is
wrong on the device**, and the correction is the finding.

**`ctx.B.Scrub()` has exactly ONE caller**, measured:

```
$ grep -rn "\.Scrub()" --include="*.go" . | grep -v _test.go
gui/run_flow.go:245:			ctx.B.Scrub()
```

and it sits **inside `if !wiping { return }`** — i.e. on the §10.2.4 wipe path
only.

**The other branch is unreachable in production.** `uiFlow` loops `for !ctx.Done`
(`gui/gui.go:1612`), and on the device `ctx.Done` is set by exactly one thing: the
wipe. The `!yield()` route needs the consumer to stop ranging, and
`cmd/controller/main.go:34` is `for range gui.Run(p, ver) {}`, which never stops.
So `runWithFlow` never returns on hardware, a UI-level "normal exit" is just the
flow walking back to the start screen, and **the same `Context` and the same
`op.Buffer` carry straight on**.

**What that leaves resident.** `Buffer.Reset()` runs per frame and is a
*truncation*:

```go
func (b *Buffer) Reset() {
	b.args = b.args[:0]     // TRUNCATE -- no zeroing
	clear(b.refs)
	b.refs = b.refs[:0]
}
```

`op.Glyph` encodes every rendered rune into `args`, so — in `Scrub`'s own words,
written for exactly this hazard — *"on the SeedScreen path the twelve words come
back VERBATIM AND IN ORDER from the backing array."* On the wipe path `Scrub`
zeroes them. **On a normal exit nothing does**, and they persist until later
frames happen to overwrite those indices. A start screen draws far fewer args
than a seed screen, so the tail — the later words — survives longest.

**Why this is Critical and not Minor.** §10.2.2's guarantee is wipe-by-**any**-
route. The data structures honour it: `rec`, `m`, the passphrase, the key and the
blob are each cleared on every exit path. The **rendered** copy is not, and it is
the one copy the operator's own eyes just confirmed contains their seed. The
exposed path is also the **common** one — an operator who reads their words and
presses back, rather than walking away for 3:30 — so the protected case is the
rare one and the unprotected case is the default.

**Smallest fix, provisionally:** `Scrub` on leaving the secret session, not only
on the wipe. The bracket already exists — `unlockSecretSession`'s defer — and it
is where §10.2.4's guard is installed and removed. This needs its own R0 pass
rather than an inline patch: `Scrub` zeroes to capacity and the buffer is live,
so the ordering against the next frame's build matters.

**Related, and the same shape:** the `Scrub`-only-on-wipe asymmetry was defended
in review by an argument about process teardown that this entry shows does not
hold. Cross-ref R0 round 0's M4 on `frameOp.op.src`/`inputOp.tag` — also an
enumerated argument rather than a structural one, and also about a copy `Scrub`
cannot reach. Two findings in one day where residency rested on enumeration.

### F-108 — `plate.Spline` is never zeroed AFTER the cut: F-83 buys the mid-cut window and nothing ends it (owning phase: **B2b — CRITICAL**)

**CLOSED 2026-08-10** — the zeroing landed on `b2b-residency` (`planEngraving`'s defer, the `SafePointer.Resume` trim, `splineResumer.Knot`'s `defer clear(c)`, `ClearHistory`, `releaseResumeState`), 11 mutation rows, toolpath byte-identical across 5 plates, and hardware readings 4a/4b/4c showed a resumed cut tracking its interrupted letter exactly.

Raised by the operator 2026-08-10, correcting a misreading of F-83 in this
session: *"after engraving a seed … the corresponding splines/plates may be wiped
and probably should be wiped immediately if possible."* That is right, and the
code confirms the gap.

**F-83's exemption is TIME-BOXED and the code says so.** `unlock_session.go:239`:

> `LIVE    plate.Spline, for the duration of the cut. It IS the seed rendered as
> geometry and must exist while the needle moves. F-83, accepted.`

*"For the duration of the cut"* is the **justification**. **No code ends that
lifetime when the cut ends.** Measured — there is no `clear` of a plate or a
spline anywhere in `gui/`:

```
$ grep -rn "clear(plate\|plate.Spline\|clear(.*Spline" --include="*.go" gui/ | grep -v _test
gui/unlock_session.go:196:	// carries the geometry: newEngraverJob holds plate.Spline
gui/unlock_session.go:239://	LIVE    plate.Spline, for the duration of the cut ...
gui/gui.go:2703:		job: newEngraverJob(ctx.Platform, plate.Spline, plate.Conf, 0),
```

Three mentions, all of them comments or a constructor. Nothing zeroes it.

**What happens today.** `unlockSecretPlate` builds the plate, runs
`clear(rec)` — correctly, and *before* `Engrave`, so the record is not resident
for the ~21-minute cut — then calls `scr.Engrave(...)`. When that returns,
`plate` goes out of scope. It becomes garbage; it is never zeroed, and TinyGo
does not zero on free. So **the seed, rendered as geometry, sits in the heap
after the plate is finished**, for as long as the allocation goes unreused.

The §10.2.4 idle wipe does not help: it unwinds the flow, which drops the
reference without zeroing it, and `ctx.B.Scrub()` covers the op buffer, not this.

**Why the contrast with `rec` matters.** The same function already demonstrates
the correct pattern, with a comment explaining exactly why the timing is
load-bearing: *"it must be HERE rather than after Engrave returns … Waiting for
Engrave would leave the seed resident for the whole ~21-minute cut."* The
reasoning was applied to the record and not carried to the geometry, which is the
other copy of the same secret.

**RE-SCOPED 2026-08-10, before review — the spline CANNOT be zeroed.** Measured
rather than assumed:

```
$ grep -rn "type Curve" bspline/bspline.go
bspline/bspline.go:22:type Curve = iter.Seq[Knot]
```

`Plate.Spline` is a **closure**, not a buffer. There is no `clear` to add, and
the "smallest fix" below **cannot be written as stated**. What survives of this
finding: the *reference* is already dropped promptly (`plate` is a local and
`unlockSecretPlate` returns straight after `Engrave`), but the geometry's bytes
are **unzeroable by construction** and linger as garbage until the allocation is
reused — TinyGo does not zero on free. That is exactly F-83's original point.

**RE-SCOPED AGAIN 2026-08-10, and the RE-SCOPING ABOVE IS WITHDRAWN — the buffer
IS clearable.** R0 round 0's I2 measured it: `Curve = iter.Seq[Knot]` is true and
**irrelevant**, because a closure over an already-materialised slice is still a
clearable slice, and `engrave.PlanEngraving` builds exactly that. **9 non-zero
knots survive a full cut in `knotBuf`; `clear(buf[:cap(buf)])` drives it to 0.**
Three buffers are ownable — `knotBuf`, `SafePointer.history`,
`splineResumer.catchup` — and only `appendLine`'s per-segment `make`
(`engrave/engrave.go:1146`) is genuinely unreachable.

So F-108 is **a real defect with a real patch**, not a spec amendment. I was
wrong twice here: first "zero the spline after the cut", then "impossible,
because it is a closure". The second was as wrong as the first, and worse — it
would have written an impossibility into the spec on a funds path. Design and
resolution in `design/DESIGN_b2b_residency_zeroing.md`; the ordering is split by
LIFETIME (cut state vs resume state) after R0 round 1 found that zeroing all
three together cuts a wrong plate on the operator's hold-to-resume.

~~**Smallest fix, provisionally:** zero the spline immediately after `Engrave`~~
returns — the point at which F-83's exemption expires by its own terms. Needs an
R0 pass rather than an inline patch, because the abort-mid-plate path
(`gui/gui.go:2726` calls `Stop()` and keeps rendering) means "Engrave returned"
and "the needle stopped" are not the same instant, and zeroing while the engrave
loop still iterates `e.spline` would corrupt a live cut. **The `:2651-2656`
anchor this entry used is WRONG** — that is `DescriptorScreen.Confirm`'s tail;
the real sites are `gui/gui.go:2715`, `:2726` and, the one that matters,
`:2747` `s.job.Start()`. The shipped comment at `gui/unlock_session.go:200`
carries the same drift.

**Not implicated in the 35 K residue** measured the same day: plates are built
*after* the Cut/Skip choice (`cs.Choose` precedes `toPlate`), and all three
measurement cycles walked away at that screen, so no plate geometry ever existed
in them. This is a separate defect on a path those cycles never took.

**Third residency finding in one day that rested on an enumerated or time-boxed
argument nobody re-checked** — with F-107 and R0 round 0's M4. The pattern is
worth naming in the phase report: each was individually defensible when written,
and each stopped being true without anyone editing the line that claimed it.

### F-120 — the device engraves `ms1` strings `me seal` will not seal: two different accept sets (owning phase: **post-merge polish and hardening**)

Surfaced 2026-08-10 by F-113's implementation, which could not write the test it
was asked for and said so rather than faking it.

`§10.2.1a`'s Rust test cannot demonstrate **"90 characters is admitted"**, because
`ms-codec` — the *constellation's* `ms1` codec, which `me` uses — has a discrete
accept set topping out at **77**:

```
[50, 56, 62, 69, 75]  ∪  [51, 58, 64, 70, 77]
```

The device's `codex32.New` admits two *ranges*: **48–93** and **125–127**.

So a 90-character BIP-93 codex32 secret is **engraveable by the device and
unsealable by `me`**. The Rust boundary test asserts *not-`MsTooLong`* rather
than `Ok`, and says so in its own text rather than implying a pass it did not
get.

**This is pre-existing and neither side is obviously wrong.** §10.2.1 states the
device MUST NOT assume a conforming `me` produced the blob, so being more
permissive than the host tool is deliberate there. And `ms-codec`'s discrete set
is the constellation's own m-format domain, not BIP-93's.

**But it means §10.2.1a changes nothing for payloads `me` produced** — `me`
cannot emit an `ms1` over 77 characters, so the 91–93 band it refuses is
reachable only from third-party tooling. That is consistent with the rule's
stated purpose and worth writing down, because a future reader will otherwise
assume the two accept sets agree.

**The design call nobody has made:** narrow the device to the constellation's
set, widen `me` to BIP-93's ranges, or document the divergence as intended.
Requires the Rust-primary rule if either codec's admission moves.

### F-121 — CLOSED 2026-08-11 (journeys/simulator: the emulator now homes, because the plate overlay cannot register without it) — the emulator does not HOME, so a resumed cut renders differently there than on the machine (owning phase: **post-merge polish and hardening**)

Filed 2026-08-11, out of the F-114 closure.

`cmd/controller` wraps its engraver in a `homingEngraver`
(`cmd/controller/platform_sh2.go:589`) which homes on the first write, so every
run — resumes included — begins with the head at the plate origin. **The
emulator has no such wrapper:** `platform.Engraver` returns a bare
`&emuEngraver{rec: p.toolpath}` (`cmd/emu/platform.go:185`) whose `Write` only
records and sleeps (`cmd/emu/engraver.go:35`). It never homes.

So in the simulator the recorded head keeps its position across an abort, while
`stepper.NewDriver` starts at `(0,0)` — the state the device is never in. Any
aborted-and-resumed plate therefore records a toolpath the machine would not
produce.

**Why this matters more than a fidelity nit.** `cmd/emu/platform.go:185`'s own
comment says the single recorder exists "so a plate that is aborted and resumed
records as ONE motion — which is the thing being compared". That is precisely
the comparison the missing homing corrupts. And `CONTINUITY_2026-08-11.md`
recommended `window.shToolpath` as the instrument for settling F-114's severity:
had that been used, it would have shown the head failing to arrive and confirmed
a defect that does not exist on hardware. **The measurement was settled from the
call graph and a host-side stepper test instead** (fork `d55c06b`), which is why
the wrong answer was not reached.

Generalises: an emulator that omits a step the device always performs does not
merely lose detail, it **manufactures evidence for the wrong conclusion**. Worth
an inventory of what else `cmd/emu` does not model.

**Fix shape:** give the emulator a homing wrapper equivalent to
`homingEngraver` — reset the recorder's position and emit the same
origin-seeking move — or make the recorder assert that a run begins at the
origin so the divergence fails loudly instead of rendering plausibly.

**✅ CLOSED 2026-08-11** — the first fix shape, taken while building the
simulator's plate overlay, which cannot align recorded motion onto planned
geometry until this is true. `toolpathRecorder.Home()` records the needle-UP
travel to the origin; `jobRecorder` performs it on the first `Write` of a job
and again on `Close`, mirroring `homingEngraver`'s two homing points. Both live
in `cmd/emu/toolpath.go`, which carries no build tag, so the once-per-job state
machine is host-testable — the wiring previously would have lived in the
js-only `engraver.go` and been unreachable by `go test`.

**The needle is UP, deliberately.** A needle-DOWN pass through the origin is the
F-108 signature `Summary.CutsThroughOrigin` exists to detect, so homing that
recorded the needle's last state would have forged that signature on every
healthy interrupted plate and made the flag worthless exactly where it is read.
`TestHomeTravelsWithTheNeedleUp` pins it.

**Measured, before and after.** The seed plate ends at (435840, 220160) =
**(68.1 mm, 34.4 mm)** on an 85 mm plate; that was the offset a resumed cut
recorded at. 6/6 mutants killed across `TestHomeReturnsTheHeadToTheOriginBetweenJobs`,
`TestHomeTravelsWithTheNeedleUp`, `TestHomeAtTheOriginRecordsNothing` and
`TestJobRecorderHomesOncePerJob`. Confirmed in the browser end to end: cut
"hello", abort mid-plate, hold to resume — the resumed strokes land on the plan,
and both the abort and the completion leave the head at (0,0).

One defect was found by these tests and not by reading: the first `Home()`
cleared `started` in the already-at-origin branch without emitting the pending
vertex, which **deleted the arrival** of a cut that closes at the origin —
`path()` appends the live position as a trailing vertex only while `started` is
set.

### F-119 — CLOSED 2026-08-11 (post-merge polish: comment corrected against a MEASURED fallback order) — `backup.go:368`'s comment describes a plate fallback order the code does not implement (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10; re-assigned from the font cycle)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling). Was: with F-78's font cycle, or whenever the descriptor plate is next touched. Still open; scheduled, not excused.

Found 2026-08-10 by R0 round 1 on §10.2.1a, which measured the behaviour after I
had quoted this comment into a normative spec.

The comment says the descriptor and mdmk callers keep a
**TEXT+QR → TEXT-ONLY → QR-ONLY** fallback. Measured: **QR-ONLY fails BEFORE
TEXT-ONLY**, so the order is wrong.

Not a correctness defect in the plate — the variants all exist and the caller
picks one that fits (`gui/gui.go:2106-2108`). It is a defect in the RECORD, and
this one propagated: I read it, believed it, and wrote it into
`SPEC_encrypted_payload_delivery.md` as justification for an admission rule. It
took an independent reviewer executing the code to catch it.

Textbook [[comments-outlive-their-conditions]]: the comment was likely true when
written and the ordering changed under it. Fix is to re-derive the order from the
code and say what it now does, or delete the ordering claim and keep only "these
variants exist".

### F-116 — `biptool seed -seedlen` emits codex32 strings this machine cannot engrave, silently (owning phase: **before the release tag**, with F-113)

**CLOSED 2026-08-10 — `seedhammer` `c0c958d`.** `warnUnengraveable` in
`cmd/biptool/main.go` warns on stderr, leaving stdout pipeable. Limits are
measured, not hardcoded: validity via `codex32.New`, engraveability by the same
`qr.Encode(ToUpper(s), qr.M)` / `Size > 33` call `backup.EngraveSeedString`
makes.

Exercising the full `-seedlen` range found **two** failures where I expected
one — 41–62 bytes produces 94–124 characters, which is codex32's **dead zone**
and is rejected by `codex32.New` outright. That is unparseable, not merely
uncuttable, and now gets its own message.

Found 2026-08-10 by R0 on §10.2.1a, which refuted the claim — mine, stated to
the operator as measured — that nothing in this constellation generates a long
codex32 code.

```
$ head -c 64 /dev/zero | biptool seed -seedlen 64 -id entr
ms10entrsqqqqq…mk6rc3gq4c88nvp        127 chars
codex32.New: ADMITTED (long band)
qr size 41 -> REFUSED (Size > 33)
```

`cmd/biptool/main.go:312` calls `codex32.NewSeed` directly, bypassing
`EncodeMS1`'s BIP-39 cap, and `-seedlen` advertises **16–64 bytes** in its own
flag help. So the tool is *designed* to reach the range the engraver refuses.

**Warn, do NOT cap.** The obvious fix is to reject `-seedlen` above the
engraveable range, and it is wrong: F-113's own test vectors need a generator for
125/127-character strings, and this is it. Capping would remove the only way to
produce the fixtures for the refusal being added. Print to stderr when the output
exceeds 90 characters, naming the length and that the machine cannot engrave it.

**Why it matters beyond tidiness:** we ship a tool whose output our own engraver
rejects. That is an inconsistency someone eventually hits and reasonably files as
a bug against the engraver rather than the generator.

### F-117 — the seed plate cannot engrave a QR above 33 modules, and could reach 37 today (owning phase: **post-release feature**, with F-118)

`backup.EngraveSeedString` refuses `qrc.Size > 33` (v4), while
`backup.EngraveText` — the md/mk path — already runs at **37** (v5) in
production, on the same `bitmapForQRStatic` marker table. The difference is
scale: text plates engrave modules at **2** stroke widths, the seed plate at
**3** (`backup/backup.go:166`, `qrScale = 3`), so 37 modules is 22.2 mm there
against 33.3 mm here, on an 85 mm plate.

Raising the seed plate to 37 would close the **91–93** band by engraving it
rather than refusing it (§10.2.1a). Deliberately NOT a pre-tag change: it alters
plate geometry, so it re-opens toolpath equivalence and needs a hardware read.

### F-118 — engraving a LONG codex32 share needs QR version 6 support (owning phase: **post-release feature**)

125–127 characters encode to **41 modules (v6)**, past every current limit.
`bitmapForQRStatic` tabulates position and alignment markers for **21/25/29/33/37
only**; anything else reaches its `default:` and panics, which is why
`engrave.ConstantQR` rejects `dim > 37` and says:

> *bitmapForQRStatic tabulates 21/25/29/33/37 only, so rejecting here is what
> keeps a larger version from reaching its default case and panicking. **Raise
> both together or not at all.***

So this is not "split the share across plates" — it is extending the QR
engraver's version support, plus 36.9 mm of QR on an 85 mm plate at the seed
scale (or 24.6 mm at the text scale, if F-117 lands first). A feature with its
own spec and hardware validation. §10.2.1a refuses these meanwhile, which is the
honest interim: a clear message beats a dead end.

### F-115 — `plan-cite-gate.sh` resolves citations by BASENAME and takes the first match, including build artefacts (owning phase: **before the release tag**, with F-101's runner work)

**CLOSED 2026-08-10 (`51ff889`).** The resolver prunes `target/`, `.git/` and `node_modules/` alongside `third_party/`, and FAILS as AMBIGUOUS naming every candidate rather than taking the first match. Verified: it now reports `bip380/checksum.go` and `codex32/checksum.go` by name instead of silently choosing the 89-line one.

Found 2026-08-10 while gating the §2.2 item 12 amendment. The gate reported two
unresolvable citations in `SPEC_encrypted_payload_delivery.md`; both citations
are **correct** and the gate resolved the wrong file:

| citation | gate resolved | should be |
| --- | --- | --- |
| `main.rs:375` | `target/package/mnemonic-engrave-0.1.0/src/main.rs` (146 lines) | `crates/me-cli/src/main.rs` (647 lines) |
| `checksum.go:132` | `bip380/checksum.go` (89 lines) | `codex32/checksum.go` (170 lines) |

Pre-existing: 2 failures before the amendment and 2 after.

**The false NEGATIVE is the dangerous half.** A wrong-file FAIL is merely noisy —
someone checks and moves on. But the same basename resolution will report **ok**
whenever the wrong file happens to be long enough, printing a line from a stale
`target/` artefact as though it were the cited source. That is the whole failure
mode the gate exists to prevent, running inside the gate itself: a check that
looks in the wrong place and returns a clean answer. See the same shape in
`grep -c CLOSED` over this file, and in `go list -deps ./cmd/controller`
returning empty because the build constraints excluded everything.

**Smallest fix:** skip `target/`, `third_party/` and any VCS-ignored path; and
when a basename matches more than one file, **fail as AMBIGUOUS** rather than
picking one. Prefer a repo-relative citation (`codex32/checksum.go:132`) and
teach the gate to require one where the basename is not unique.

### F-114 — CLOSED 2026-08-11 — NOT A DEFECT: the machine homes before every run, so the head really is at the origin

**CLOSED 2026-08-11, post-merge polish and hardening.** The premise is false.
The entry assumed the head is "wherever it actually is" when the synthesised
approach line executes. It is not: the machine has just **homed to the plate
origin**, so a line drawn from `bezier.Point{}` starts exactly where the head
is. Nothing crosses the work area twice, and there is no traverse-wear cost.

The chain, each link read rather than inferred:

| step | evidence |
| --- | --- |
| a FRESH `*homingEngraver` is returned per run, `homed=false` | `cmd/controller/platform_sh2.go:589` |
| `runEngraving` calls `pl.Engraver` once per run, resumes included | `gui/engraver.go:186` |
| `homingEngraver.Write` homes on the FIRST write, so homing precedes any engraved step reaching the device | `cmd/controller/platform_sh2.go:599` |
| `home()` drives to the limit switches, resets the driver, then moves to `(originX, originY)` = `(5.0mm, 3.2mm)` — the PLATE origin, i.e. engraving coordinate `(0,0)`; that offset is machine-zero→plate-origin and is consumed inside `home`'s own driver | `cmd/controller/engraver.go:186`, `platform_sh2.go:208` |
| `runEngraving` then builds a fresh `stepper.NewDriver`, whose `d.pos` is `(0,0)` | `gui/engraver.go:198` |

So `d.pos` and the physical head agree. **The "short distance towards top left"
in hardware reading 4a was the homing move**, which is intended — not the
defect this entry was opened for.

**Pinned, because the coupling is invisible.** Nothing in `engrave/` or
`stepper/` mentions homing, yet `Resume`'s correctness rests entirely on it.
`stepper/resume_homing_invariant_test.go` (fork `d55c06b`) asserts the homed
case lands exactly on the safe point, and *reports* the un-homed case rather
than asserting it, since that state does not occur today:

```
un-homed at 20,15 mm: cut starts     0,0 steps off the safe point
un-homed at 60,40 mm: cut starts     0,0 steps off the safe point
un-homed at 80,60 mm: cut starts 14932,0 steps (2.33,0.00 mm) off
```

The error appears only past the point where `Driver.fill`'s one-step-per-tick
catch-up can no longer close the gap within the approach line's own duration.
If homing is ever removed, made conditional, or moved after the first write, a
resumed cut silently starts in the wrong place — now a red test rather than a
ruined plate.

**Correction to the citation below:** the `appendLine` call is at
`engrave/engrave.go:1667`, not `:1664`.

**Spawned F-121** — the emulator does **not** home, so the simulator was the
wrong instrument for this question.

---

*Original entry follows, retained because its reasoning is what the closure
answers.*

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling; the concerns below were raised and the operator decided). Was: post-B2b, before the release tag.

Found on hardware 2026-08-10 (reading 4a) while validating F-107/F-108, and it
is **not** a residency defect — it is pre-existing upstream behaviour that the
reading happened to expose.

`engrave/engrave.go:1664`:

```go
move = appendLine(move, conf, false, bezier.Point{}, s.safePoint)
```

`appendLine` takes `dist := ManhattanDist(s, e)` and interpolates between `s`
and `e` in **absolute** coordinates. With `s = bezier.Point{}` the synthesised
catch-up is a line from the machine origin to the safe point, so on resume the
head tracks toward (0,0) before running out to where it stopped. Observed:
*"went a short distance towards top left and then directly to where it left
off"* — short only because that plate's work sat near the origin.

**Not a correctness bug.** The needle is up (`engrave` is `false`), the move
ends at the right place, and the resumed cut tracked the interrupted letter
exactly. What it costs is time and traverse wear, scaling with the distance
from the origin: a plate cut at the far corner resumes by crossing the whole
work area twice.

**Why it is filed rather than fixed now.** The obvious fix — start the line at
the head's current position — needs a position the planner does not currently
have; `SafePointer` tracks geometry, not where the driver is. `stepper.Driver`
knows (`d.pos`), so the seam exists, but threading it through is a change to
normative motion behaviour and belongs in its own cycle with test vectors.

**It also corrected a detector.** `cmd/emu`'s toolpath recorder flagged "returns
to the origin" as the F-108 signature; this is what every healthy resume does.
Fixed in `seedhammer` `c38cb6b` to require a needle-DOWN pass.

### F-113 — codex32 LONG CODES are admitted, decrypted and offered, then can never be engraved (owning phase: **post-B2b, before the release tag**)

**CLOSED 2026-08-10.** Implemented both sides, Rust first per the Rust-primary
rule. Device: `seedhammer` `b2b` (merge of `f113-ms1-engraveable` @ `f3f866f`) —
`AdmitSection`'s per-record pass refuses an `ms1` over 90 characters and wipes
the records already copied; `unlockSealedFlow` gains a case so the operator is
told the reason rather than *"Payload unreadable."* Host: `mnemonic-engrave`
`master` (merge of `f113-ms1-engraveable` @ `d8258e9`) — `record::validate_record`,
with `check_public` reporting an over-length `ms1` in `--plaintext` as a SECRET
rather than as a length problem.

**The 90 is bound, not written down twice.** `backup.go` gained
`seedQRLevel`/`seedQRMaxSize` so the derivation test READS `EngraveSeedString`'s
own constants. Measured: with `qr.M` duplicated in the test, switching the
function to `qr.Q` left the test green at 90 while the machine would refuse at
67; after binding, that same change turns it red.

**Gates:** R0 rounds 0–2 on the design (5I → 1I → GREEN), a whole-diff execution
review on the code (GREEN, 0C/0I, seven mutations re-run by hand), and a focused
fold review. 16 mutations, 15 killed; the survivor (`wipe(out)` removed) is
pre-existing and unobservable without `unsafe`, and a test was added that
discriminates the per-record pass from the post-loop block instead.

**What it does NOT do**, so nobody re-derives it: it changes nothing for
payloads `me` produced, because `me` cannot emit an `ms1` over 77 characters
(**F-120**). The band it refuses is reachable only from third-party BIP-93
tooling. Engraving long codes remains **F-117**/**F-118**.

Found 2026-08-10 while answering an operator question about QR codes on sealed
plates. Measured, not reasoned about.

`codex32.New` (`codex32/codex32.go:41-44`) admits two length bands:

```
shortCodeMinLength = 48    shortCodeMaxLength = 93
longCodeMinLength  = 125   longCodeMaxLength  = 127
```

`backup.EngraveSeedString` builds a QR from the uppercased share
**unconditionally** and refuses `qrc.Size > 33`. Measured against the real
encoder with the uppercased bech32 charset:

| share length | admitted by `codex32.New` | QR size | engrave |
| --- | --- | --- | --- |
| 48–**90** | yes (short) | 29–33 | **cuts** |
| **91–93** | yes (short) | 37 | **refused** |
| **125–127** | yes (**long**) | 41 | **refused** |

So **every codex32 long code, and the top three lengths of the short-code range,
are unengraveable** — while being perfectly decryptable. A 256-bit seed lands at
~74 chars → QR size **33**, exactly at the ceiling with zero headroom, so nothing
above it can ever fit.

**Why this is worth a follow-up rather than a shrug.** The operator learns only
*after* unlocking and decrypting — the record is admitted by §10.2.1's
allow-list, offered by the secret session, and then dead-ends. And the message
they get is **"This record does not fit any plate size."**, which reads as
*choose a bigger plate*: no plate size helps, because the ceiling is the QR
encoder, not the plate.

**Two fixes, and the first is the real one:**

1. **Refuse it earlier.** Best on the HOST, in `me seal`, which can reject a
   record the device could never engrave before the payload is ever written.
   Failing that, at §10.2.1's allow-list, so the record is never offered.
2. **Reword the message** to say the record is too large to ENCODE, not to fit a
   plate.

**Do not fix inline in the B2b residency work** — it touches admission on a funds
path and deserves its own scope. Note also the Rust-primary rule: if admission
changes, the normative behaviour lands in the primary Rust crate with test
vectors first, and the Go port follows.

**Not a residency problem.** The refusal path is one where `toPlate` has already
filled the knot buffer at build time and no cut ever happens — that geometry IS
zeroed, by the `defer` inside `planEngraving`'s closure (F-108 item 1), which
fires on the iterator's exit rather than on a cut.

### F-110 — an ABANDONED engrave job's resume state is never zeroed (owning phase: **B2b** — OVERDUE, re-assigned 2026-08-11 to **post-merge polish and hardening**)

**STATUS CORRECTION 2026-08-11.** `CONTINUITY_2026-08-11.md` and the brief given
to the 2026-08-11 triage agent both listed F-110 among the items "closed during
the cycle". **It is not closed**, and this entry never said it was — the error
was in the summary, not the ledger. The triage refuted the anchor it was handed,
which is what an independent reviewer is for. Both halves are still named as
open F-110 holes *by the shipped code itself*:

- `gui/engraver.go:126-132` — "TWO non-terminal returns skip this, not one …
  That hole is F-110, not a covered case."
- `engrave/engrave.go:1722-1730` — "4 orphaned arrays holding 15 knots, rising
  to 23 arrays / 119,891 knots if the driver reports no progress … That residue
  is F-110, not something this function covers."

Owning phase B2b has passed, so this is **overdue, not deferred**. Re-assigned
to post-merge polish and hardening.

**REFINEMENT from the fable whole-Phase-2 review (M3), 2026-08-11 — a positive
finding.** The §10.2.4 wipe **provably cannot fire mid-cut**: `armed()` is false
while a job is running or stopping, and `ctx.Done` is wipe-only. So the wipe can
never strand resume geometry, and the bullet of this entry that describes that
path is describing something **unreachable**. That narrows the entry; it does
not close it — the two sites above are reached by the *double-Back* and
*ctx.Done* returns, not by a mid-cut wipe. Reword rather than close.

Filed by the R0 round-1 fold of `DESIGN_b2b_residency_zeroing.md`.

**REVISED after R0 rounds 2 and 3 — the `catchup` half is CLOSED and this entry
described a placement that no longer exists.**

`SafePointer.history` is **resume state**: its lifetime is the job, not the
goroutine, because `e.catchup()` re-reads it on the operator's hold-to-resume
(`gui/gui.go:2747`). It is zeroed at `EngraveScreen.Engrave`'s return via
`releaseResumeState` → `SafePointer.ClearHistory`, and **only when the job is
terminal** — a terminal state is the receive on `e.errs`, so `runEngraving` has
provably returned and there is no live writer.

**`splineResumer.catchup` is no longer part of this item.** Round 2's I-A showed
the job-level placement could never have reached it (`res :=` is a local in
`runEngraving`, and `s.catchup` is nil by the first resumed knot). It is now
zeroed by `defer clear(c)` inside `splineResumer.Knot`, where the array is still
named — and round 3 verified by execution that `SafePointer.Resume` returns a
NON-ALIASING array, so that clear cannot corrupt `history`.

What remains open:

1. **Two non-terminal returns skip the zeroing**, not one: `Engrave` returning on
   `ctx.Done` (§10.2.4 firing mid-cut) and the double-Back return in
   `engraveStopping`, where the goroutine is still winding down. Neither is
   covered elsewhere — the wipe unwind is `ctx.B.Scrub()` + `Drawer.Release()`
   and reaches no engrave state. Skipping is still the right call: zeroing under
   a live goroutine races it, and a wrecked plate is worse than the residue.
2. **`SafePointer.history` grows by `append`** (`engrave/engrave.go:1683`), so it
   carries the same outgrown-array class as `op.Buffer` did: the tail-clear at
   `:1675-1676` reaches only the CURRENT array, and every reallocation before it
   left a full copy of the knots behind. `ClearHistory` zeroes the current array
   to cap, so this is bounded to arrays outgrown *during* a single job.

Both remaining halves are seed-derived geometry, and neither is covered by the
design that files them.

### F-111 — `knotBuf` unzeroed wherever a plate is built and no cut happens — SUBSUMED by the F-108 design (owning phase: **B2b**)

**CLOSED 2026-08-10 — SUBSUMED by F-108's design and implemented with it.** `planEngraving` zeroes the caller's knot buffer on every exit path, which covers the plate-built-but-never-cut route this entry was filed for.

Filed by the R0 round-1 fold of `DESIGN_b2b_residency_zeroing.md`; sharpens round
0's M3.

`toPlate` → `bspline.Measure` fills the knot buffer at **build** time
(`gui/gui.go:2988-2989`), so "for the duration of the cut" was never the right
lifetime bound. On the too-large path (`gui/unlock_session.go:191-193`:
`showError`, `return`) the buffer is **full** and no cut, no goroutine and no
send on `e.errs` ever happen — so the design's cut-end zeroing, which hooks the
goroutine's exit, cannot fire at all. **The failure case leaks geometry the
success case scrubs.**

**WIDENED, then SUBSUMED — R0 round 2 (M-a).** Filing this as an
`ErrTooLarge`-only item was itself the defect. `toPlate` fills the buffer at
build time for **every** plate, so the same hole is open on an ordinary operator
path — *"insert a blank plate… hold button to start"*, then **Back** before the
cut starts (`gui/gui.go:2721-2725`: `st.State != engraveRunning` →
`break frames`) — and on `bspline.Measure`'s build-time range, which never cuts
at all. Implemented as filed, the patch would have closed the error path and left
the ordinary one open.

**Now subsumed.** `design/DESIGN_b2b_residency_zeroing.md` item (1) puts the
zeroing in a `defer` inside `planEngraving`'s own closure, so it fires when the
ITERATOR finishes: all three paths covered by one line, and no fourth path can be
added that misses it. **Close this with that design; do not implement it
separately.** Applied in the gate worktree and building.

### F-112 — six LEGACY seed-rendering flows sit inside no `Scrub` bracket at all (owning phase: post-B2b, before the release tag)

**CLOSED 2026-08-10 as ACCEPTED under SPEC §2.2 item 12** (operator ruling
2026-08-10): §10.2.4's residency wipe and the `Scrub` brackets are scoped to the
**Sealed Payload program's session**, and all six flows listed here are other
programs. Their residue is accepted, not a defect.

**Verified, because I twice said the opposite.** I claimed one of the six sat
inside the payload path and that the entry "needs splitting, not closing". Wrong
on both counts: `gui/unlock_session.go:276` appears above as the entry's
*contrast* — the one BRACKETED construction site — not as one of the six. And
none of the six is reachable from the Sealed Payload program: searched whole-tree
for each of `backupWalletFlow`, `seedEntryFlow`, `bip85DeriveFlow`,
`recoverSLIP39Flow`, `combineSeedXORFlow`, `passphraseFlow` and found **0** calls
from `unlock_session.go`, `unlock_kdf.go` or `unlock_flow.go`.

Note `passphraseFlow` (`gui/gui.go:584`) is the LEGACY BIP-39 passphrase
keyboard, distinct from `unlockPassphraseFlow` (`gui/unlock_kdf.go:109`), which
is the sealed-payload one and IS bracketed — §10.2.4 row 4. Two similar names,
opposite sides of the boundary; worth stating so the next reader does not merge
them.

**Still true and still worth saying:** "the machine wipes the rendered seed" is
not true in general. §2.2 item 12 is where that now lives, and §2.3 carries the
operator-facing half.

Filed by the R0 round-1 fold of `DESIGN_b2b_residency_zeroing.md`; round 0's M2.

F-107's fix brackets the B2b secret session and the passphrase flow. Every
pre-existing flow that renders seed material has **no bracket whatsoever**:

| flow | where |
| --- | --- |
| `backupWalletFlow` | `gui/gui.go:2194` |
| `seedEntryFlow` | `gui/derive_xpub.go:82` |
| `bip85DeriveFlow` | `gui/bip85.go:269` |
| `recoverSLIP39Flow` | `gui/slip39_polish.go:229` |
| `combineSeedXORFlow` | `gui/seedxor_polish.go:40` |
| `passphraseFlow` | `gui/gui.go:584` |

Measured: `SeedScreen` has exactly one bracketed construction site,
`gui/unlock_session.go:276`. The outgrown-array zeroing helps these flows for
free — it is in `op.Buffer` itself — but the CURRENT array is `Scrub`'s job and
nothing calls `Scrub` on any of these paths. **This is the pre-existing product,
not B2b**, which is why it is scheduled after the phase rather than inside it —
but it means "the machine wipes the rendered seed" is not true in general.

### F-109 — DOWNGRADED to Minor 2026-08-11 (measured: no secret in the residue; ~12 K of ~74 objects still unnamed) — ~35 K in ~81 REACHABLE objects survives every wipe, unidentified (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling; the concerns below were raised and the operator decided). Was: the fable whole-diff review of ALL of Phase 2 — operator ruling 2026-08-10.

**OPERATOR RULING 2026-08-10 — re-assigned to the fable whole-diff review of all
of Phase 2.** This item was OVERDUE: its owning phase was B2b, and B2b was
merged and pushed with it open.

**Why the re-assignment is the right venue and not a re-park.** F-109 is a
question about the *whole* secret lifecycle — where a decrypted record, its
rendered form and its plate geometry come to rest across B2a-i's KDF engine,
B2a-ii's session and B2b's wipe. No single phase's context holds that, which is
exactly why three phases of review each left it standing. It needs one reviewer
reading the entire Phase 2 diff at once.

**This ruling CREATES that gate.** Scope: the whole-diff of B2a-i + B2a-ii + B2b
against the phase's base, at **fable** tier — the single highest-stakes review
this project reserves for the last gate before an irreversible action, here the
release tag. It runs **before the tag**, and F-109 is a named must-answer
question in its brief, not a line item it may note in passing.

**What the review must answer, not merely observe:** identify the ~81 objects.
Name them. For each, say whether it can hold seed material, ciphertext, a
rendered secret or plate geometry — and if none can, say what they are instead
and why the count plateaus at exactly that number. "Probably harmless" is not an
answer; the entry is open *because* they are unidentified, so a review that
leaves them unidentified has not closed it.

~~**Do not tag the release with this open.**~~ **SUPERSEDED 2026-08-10 by a
later operator ruling:** the merge, tag and release happen **before** the fable
review, intentionally. See the phase banner. The tag is `v0.0.0-g<sha>`, which
marks a build rather than a product, and the fable review moves to
post-release.


Measured on hardware 2026-08-10 with `b2b-heapprobe2`, which forces
`runtime.GC()` before every readout — so these are reachable objects, not
garbage awaiting a sweep.

| | baseline | wipe 1 | wipe 2 | wipe 3 |
| --- | --- | --- | --- | --- |
| in use | 127 K | 162 K | 162 K | 162 K |
| free | 318 K | 283 K | 283 K | 283 K |
| live allocs | 193 | 274 | 276 | 276 |

**It PLATEAUS.** Cycles 2 and 3 are byte-identical and differ by two objects, so
this is a one-time ~35 K cost, not a per-wipe leak. No exhaustion, no cliff.

**Fix D is not the answer, and the same run proves it.** `buf 2048/512` — the
abandoned Buffer's capacities at the moment of abandonment — is
`2048×4 + 512×8` ≈ **12 KB**, the entire ceiling on what `Drawer.Release` could
ever recover, and D is already in the measured build so those bytes are already
back. The 35 K sits on top. R0 round 0's C1 predicted exactly this and was
conservative: it estimated ~24 KB.

**Why this is not merely a memory-hygiene item** (operator, 2026-08-10: *"for all
we know that missing 35 K is unwiped secret data, right?"* — correct, and it
overturns the "bounded, therefore a follow-up" framing I first gave it).
§10.2.4's guarantee is about **secrets**, not bytes. Bounded ≠ safe. Until the 81
objects are **named**, this is an open residency question on a funds path.

**Ruled out so far:** plate geometry. Plates are built *after* the Cut/Skip choice
(`cs.Choose` precedes `toPlate`), and all three cycles walked away at that
screen, so no spline ever existed in them. See F-108 for the separate defect on
the path they did not take.

**The measurement that closes it, and it needs no hardware.** `MemStats` can count
objects but never identify them, so stop asking it. Instead do at the `gui` level
what `gui/op/release_test.go` now does at the `op` level: drive `runWithFlow`
through a real unlock and a real wipe, attach `runtime.SetFinalizer` to each
secret-bearing object — the blob, the decrypted records, the passphrase buffer,
the parsed words — force collection, and assert **every one is gone**. Anything
that survives is *named*, not guessed. Note the three traps that test found the
hard way: `runtime.KeepAlive` the holder or the collector reclaims it and the
test passes vacuously; two `GC()` calls plus a timeout, not one; and choose the
canary so it actually enters the structure under test.

### F-92 — `tinygo test` cannot build `seal` at all: the TinyGo wipe caveat has never run on the target toolchain (owning phase: before the release tag)

**DECLINED 2026-08-10 — operator ruling: "what we have is good enough."**
Accepted limitation, not a defect to fix before the tag. The evidence that makes
that reasonable is recorded above and in
`design/agent-reports/2026-08-10-f92-tinygo-seal-investigation.md`: the DEVICE
build is clean, `bip39.Parse`'s append-orphan guarantee is already verified under
`tinygo test -gc precise` today, and every `clear()` in `seal` is GC-independent
by construction. What remains untested is a host-toolchain gap, not a firmware
one. Re-open only if the wipe story changes to depend on GC behaviour.

**NARROWED 2026-08-10 — the premise is partly false.** Investigated in
`design/agent-reports/2026-08-10-f92-tinygo-seal-investigation.md`.

- `bip39.Parse`'s append-orphan guarantee — which `seal.Classify`'s `clear(m)`
  depends on — is **already verified under the shipping toolchain**:
  `tinygo test -gc precise ./bip39/` exits 0 today, with no code changes. And
  every `clear()` in `seal` is GC-independent by construction. The untested
  surface is smaller than "has never been tested" implies.
- The failure is **two** failures. `tinygo test ./seal/` fails to COMPILE
  (`undefined: FileReader`, 8 sites) — a build-tag split, free. Past that it
  fails at LINK on `golang.org/x/sys/cpu.cpuid/xgetbv`, an external
  TinyGo/amd64 limitation reached via `record.go`'s `btcd/address` import.
- **The device build is unaffected** — `pico-plus2` builds clean, exit 0. This
  is a host-test-toolchain problem, not a firmware one, which lowers its
  severity considerably.

Recommended route: take the free build-tag split, then run `seal` under
`tinygo test -target cortex-m-qemu -gc precise` (needs qemu in `flake.nix`).

Measured by the completeness critic:

```
$ tinygo test ./seal/
seal/open_test.go:508:7: undefined: FileReader
seal/read_test.go:75:8:  undefined: FileReader   (+7 more)
```

`seal.FileReader` is behind `//go:build !tinygo` and eight `_test.go` sites
reference it unguarded, so the suite is **structurally unrunnable** under the
firmware compiler.

**Why this one matters more than a coverage gap.** The caveat repeated throughout
this feature is *specifically a TinyGo claim* — §10.2 step 10's "TinyGo's GC may
copy or retain, so this is defence in depth". Every `clear()`, every
escape-analysis assumption, `passphraseBytes`' fixed capacity and
`bip39.Parse`'s `make(Mnemonic, 0, 24)` are validated **only under gc Go on
linux/amd64**, whose allocator behaves differently from the one the caveat names.
`tinygo build` proves it compiles; it does not prove a byte is where the test
says it is.

**AMENDED 2026-08-09 — the fix recorded above is WRONG, measured end to end.**
The build-tag split is necessary and not sufficient, and it was worth doing the
experiment rather than the reasoning.

Performed in a throwaway copy: moved the seven `FileReader` tests and
`TestOpenFromAReader` behind `//go:build !tinygo`, keeping the two `clampRegion`
tests untagged (they must stay reachable — `read.go`'s own comment says the bound
lives untagged precisely so a host test can kill the unbounded-read mutant).
Result: **host `go test ./seal/` still passes (12.1 s)**, so the split is safe —
and `tinygo test` then fails one layer down, at LINK:

```
ld.lld: error: undefined symbol: golang.org/x/sys/cpu.cpuid
ld.lld: error: undefined symbol: golang.org/x/sys/cpu.xgetbv
```

Those are **amd64 assembly stubs**, reached by a chain that has nothing to do
with this feature (`go mod why`):

```
seedhammer.com/address → btcd/chaincfg/v2 → btcd/wire/v2
                       → golang.org/x/crypto/sha3 → golang.org/x/sys/cpu
```

TinyGo defaults `tinygo test` to the **host** target, and on amd64 that package's
CPU-feature detection is assembly TinyGo does not provide. **Note the device
build is unaffected** — `-target pico-plus2` selects the ARM path and has always
worked.

**Two further obstacles, both measured, so nobody re-derives the easy answer:**

1. `tinygo targets` does offer emulated targets (`cortex-m-qemu`, `riscv-qemu`)
   which would dodge the amd64 assembly and be closer to the real part — but
   **no qemu binary is on `PATH` in the dev shell**, so that route needs a
   `flake.nix` change first.
2. Even with an emulator, **three filesystem call sites** in `seal`'s tests read
   `testdata/vectors.json` from disk. A bare-metal target has no filesystem, so
   the normative fixtures would have to be `//go:embed`-ed.

**So the realistic closure is narrower than "make the suite run under TinyGo".**
It is a small TinyGo-buildable target that exercises the claims the caveat
actually makes — `clear()` reaching the buffer it names, `passphraseBytes`' fixed
capacity not regrowing, `bip39.Parse`'s `make(…, 0, 24)` — over **embedded**
fixtures, run under `cortex-m-qemu`. That is a real piece of work with a
`flake.nix` dependency, not a build-tag edit.

Owed before the tag, with F-85. **Do the split anyway when this is taken up** —
it is safe, it is a prerequisite, and it costs nothing.

### F-91 — CLOSED 2026-08-09 — the normative `vectors.json` digest is now asserted

One test, `TestVectorFileMatchesTheDigestTheREADMERecords` (`seal/vectors_test.go`),
mutation-checked with a byte no other test can see: one space appended to the
file's `note` field — touching no vector field — fails it while
`TestVectorsAreLoadable` stays green. Filed and closed the same day; it was one
test, not a deferral.

`seal/testdata/README.md:17` records
`sha256 = 333ac47e7f61d031c995b85510565bfffd86cd1992f09b0230c1484fffd4d4bc` and
declares the file generated by the Rust implementation and never hand-edited.
Under the Rust-primary rule **that digest is the binding**. `grep -rn 333ac47e`
over the repo returns **exactly one hit: the README line itself.**

`blob_sha256` is an *internal* consistency check — it catches editing `blob_hex`
alone and catches nothing about `passphrase`, `iterations`, `public`, `secret`,
`pubhash_sealed` or `pubhash_unsealed`, which are the six fields B2a-ii newly
leans on. One test asserting the file digest closes it. *(Measured today: the
file still matches.)*

### F-90 — the `ms1` engrave arm is the under-examined one, and it is the DEFAULT arm (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10; re-assigned from B2c)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling). Was B2c. This entry **still binds** — see the traced call graph above; §2.2 item 12 does not accept it — it is now scheduled later, not excused.

**STILL BINDS — checked against SPEC §2.2 item 12 on 2026-08-10 and NOT accepted.** `unlockEngraveCodex32` has exactly ONE caller in the whole tree — `unlockSecretPlate`, inside the same bracket. Not mixed. Verified by tracing the call graph whole-tree, not by reading this entry: an earlier pass classified this as legacy-only from entry prose and was wrong. See `design/agent-reports/2026-08-10-b2c-program-boundary-verification.md`.

> **Re-assigned B2b → B2c, 2026-08-09.** The B2b plan deferred this to "own
> cycle", which is **not a later phase — it is no phase**, and
> `/scratch/code/CLAUDE.md` forbids parking an item on nothing: "an item that
> binds the current phase, or is scheduled *to* a phase, is not deferrable past
> its owning phase." Found by the B2b residue sweep (I4). The work is real and
> is not B2b-sized — F-88's only actionable copy is a `bip39.MnemonicSeed`
> change that five other flows call and that wants its own review — so it gets a
> **named successor phase** rather than a silent deferral or a silent scope
> increase. **B2c is secret-residency cleanup: F-88, F-90 items 1 and 3, F-94.**

The completeness critic's headline, and it is a distribution-of-attention
finding rather than a defect.

`unlockEngraveMnemonic` accumulated: a Critical (early `clear(m)`), an Important
(the BIP-39 seed and BIP-32 master key), a caveat rewritten after being wrong
**twice**, a pinned-and-mutation-checked `clear(rec)`, a dedicated hook, a
dedicated test, and four follow-ups (F-83, F-87, F-88, F-89).

`unlockEngraveCodex32` has: a five-line comment naming no copy individually,
pointing at *another file's* comment instead of a follow-up. No hook. No
inventory. No follow-up. It appears in no row of any of them.

**And it is the arm that actually runs.** Six of the seven canonical vectors
carry `ms1` secrets; exactly one (A) carries a bare mnemonic. Both wallet shapes
the design cites — single-sig `bip84` (1×`ms1`) and 2-of-3 `wsh-sortedmulti`
(3×`ms1`) — route entirely through it. The scrutiny went to the arm one vector
reaches.

Its copies are enumerable in thirty seconds, and every one is the codex32 secret
share, i.e. spendable key material: `string(rec)`; whatever `codex32.String`
retains; `id` from `Split()`; `s.String()`; `plan`, then `plate.Spline`.
`clear(rec)` zeroes exactly one of six.

**Three things close it, all B2b-owned:**

1. Write the F-88-equivalent inventory for this arm. D1's finding was that an
   inventory claiming completeness while incomplete is worse than none — and
   right now the mnemonic arm's inventory reads as *the* inventory.
2. **Correct `p.SecretsResident()`'s contract** — this is the one with a funds
   consequence, and it belongs with **F-89**. The predicate scans `p.Secret`
   only, so it reads **false** the instant `clear(rec)` runs, while four string
   copies of the share are still live. B2b keys its timer on that predicate.
3. Add an `unlockCodex32Hook` mirroring `unlockMnemonicHook`. That hook exists
   *because* a local was unreachable and a seed sat live through a whole cut with
   the suite green (C1). This arm has the identical structure and no hook.

### F-89 — B2b's idle wipe MUST unwind the flow, not just call `p.Wipe()` (owning phase: B2b — a DESIGN CONSTRAINT, not a defect)

**CLOSED 2026-08-10 — unwind via ctx.Done at gui/run_flow.go:282-288, RecordsResident's narrowed contract at seal/session.go:20-51; killed by removing ctx.Done=true from the armed-wipe branch, which fails TestWipeZeroesEveryPinnedBufferAtRunLevel (both subtests, "the wipe never restarted the session").**

Found by lens 1 pass 3 (M3). Nothing is wrong today; this is a trap laid for the
phase that has not been written yet, and it is the C1 Critical arriving through a
different door.

**The shape.** `unlockEngraveMnemonic` holds two copies of the seed: `rec`, which
lives in `p.Secret[i].Record`, and `m`, which is a **local**. Both are cleared
before `Engrave` — that is C1's fix. But the function also keeps `defer clear(m)`
for its three early returns, and a defer only fires when the function **returns**.

So if B2b's §10.2.4 timer fires while a secret flow is on screen and wipes **in
place** — `p.Wipe()`, or `WipeSecretAt` over the set — then:

```
rec                 zeroed
SecretsResident()   false        <- the timer believes it is done
m                   STILL THE SEED
```

That is verbatim the state C1 described: a live seed that no wipe in the system
can reach, with the residency predicate reporting clean. The timer would have
*created* it rather than closed it.

**The constraint, and it is one line to satisfy:** B2b's idle wipe must cause the
flow to **unwind** — set `ctx.Done`, or otherwise make the secret flow return —
so the deferred `clear(m)` runs. Calling `p.Wipe()` without unwinding is
insufficient and *looks* sufficient, which is what makes it worth a follow-up
rather than a comment.

**Generalises past `m`:** any local copy in any flow has this property. The rule
for B2b is that the timer's job is to make flows RETURN, and the wipe is what
their defers then do — not the other way round.

**AMENDED 2026-08-09, and this is the half with a funds consequence (F-90 item
2).** The same trap is worse on the `ms1` arm, where it needs no timer at all:
`p.SecretsResident()` scans `p.Secret` only, so it goes **false the instant
`clear(rec)` runs** — while `string(rec)`, `codex32.String`'s internals, `id`,
and `s.String()` are all still live for the whole ~21-minute cut. B2b will key
its timer on a predicate that already reports "nothing to protect" on the arm six
of seven vectors take. **Fix the predicate's contract before building the timer
on it**, or B2b inherits a control that is correct only for the arm one vector
reaches.

### F-88 — three more seed-equivalent copies on the mnemonic engrave path, two of them unreachable from `gui` (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10; re-assigned from B2c)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling). Was B2c. This entry **still binds** — see the traced call graph above; §2.2 item 12 does not accept it — it is now scheduled later, not excused.

**STILL BINDS — checked against SPEC §2.2 item 12 on 2026-08-10 and NOT accepted.** `unlockEngraveMnemonic` (`gui/unlock_session.go:270`) is inside the `:88-89` wipeGuard bracket and reaches all three copies, directly or via `SeedScreen.Confirm` / `masterFingerprintFor`. Verified by tracing the call graph whole-tree, not by reading this entry: an earlier pass classified this as legacy-only from entry prose and was wrong. See `design/agent-reports/2026-08-10-b2c-program-boundary-verification.md`.

> **Re-assigned B2b → B2c, 2026-08-09.** The B2b plan deferred this to "own
> cycle", which is **not a later phase — it is no phase**, and
> `/scratch/code/CLAUDE.md` forbids parking an item on nothing: "an item that
> binds the current phase, or is scheduled *to* a phase, is not deferrable past
> its owning phase." Found by the B2b residue sweep (I4). The work is real and
> is not B2b-sized — F-88's only actionable copy is a `bip39.MnemonicSeed`
> change that five other flows call and that wants its own review — so it gets a
> **named successor phase** rather than a silent deferral or a silent scope
> increase. **B2c is secret-residency cleanup: F-88, F-90 items 1 and 3, F-94.**

Found by the I1/M1 fold re-review (D1), after two earlier attempts at this
inventory were both incomplete.

| copy | where | why it is not fixed here |
| --- | --- | --- |
| `sentence []byte` — **the plaintext mnemonic** | `bip39/bip39.go:218-224`, inside `MnemonicSeed` | a local of another package; `gui` cannot reach it. Also orphans `append` reallocations. |
| the `[]byte` behind `string(seedqr.QR(m))`, and `qr.Code.Bitmap` | `gui/gui.go`'s `engraveSeed` → `kortschak/qr` | the bitmap is a third-party struct field |
| `engraveSeed`'s `words []string` | `gui/gui.go:516-538` | **`clear(words)` is NOT free — see the correction below. Do not do it.** |

**The third one carries a correction worth keeping.** An earlier caveat listed
`words []string` as unwipeable "immutable Go strings". The *strings* are not the
secret: `bip39.LabelFor` returns substrings of the **public** wordlist. What is
secret is their **selection and order**.

> **THE REMEDY THIS ENTRY ORIGINALLY GAVE — "`clear(words)` destroys that at no
> cost" — IS FALSE, AND ACTING ON IT WOULD CUT A CORRUPT PLATE.** Measured
> 2026-08-09: `words` is **captured by `frontSideSeed`'s closure**
> (`backup/backup.go:214-230`) and read *during* the cut, exactly as
> `plate.Spline` is (F-83). Clearing it after the plate is built does not scrub a
> spent buffer — it empties a live one, and the engraver then cuts the wrong
> thing onto steel. The correct fix, if any, is the same shape as F-83's: there
> isn't one short of a pipeline that materialises before cutting. **Reasoning
> from "it's a `[]string`, so `clear` is free" produced a remedy that would have
> destroyed an operator's backup.** Reasoning from "it's a string, so it can't be wiped" got the conclusion
right for the wrong reason, and the wrong reason is what stopped anyone fixing
the one case that is fixable.

**Owning phase B2b**, with F-87 — both are residency work needing the same
scrutiny, and `MnemonicSeed`'s scrub is a `bip39` change that wants its own
review because five other flows call it.

**Not urgent, and the reason is on the record:** every one of these is resident
only while a plate is being cut, and `plate.Spline` — which **is** the seed
rendered as geometry — is resident then too and cannot be removed at all (F-83,
accepted 2026-08-08). Scrubbing these shortens no window that F-83 does not
already hold open. They are worth doing for the same reason the others were: the
package's own convention scrubs what it can.

### F-87 — CLOSED 2026-08-11 (post-merge polish: unlockEngraveMnemonic's third early return pinned) — nothing pins `unlockEngraveMnemonic`'s deferred wipe (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling). Was: B2b, with the §10.2.4 timer tests. Scheduled, not excused — the missing test is on unlockEngraveMnemonic, the same path F-88 covers — doing them together reads that path once instead of twice.

**NARROWED 2026-08-10, still OPEN.** Verified: 2 of the 3 early returns ARE pinned by applied-mutation testing. The residue is exactly one leg — the `masterFingerprintFor`-error path — which no test covers. That is the whole of what remains; the entry's original framing ("nothing pins it") is no longer accurate and would overstate the work.

Measured during the C1 fold re-review: **deleting `defer clear(m)` leaves the
whole `gui` package green.** No test drives the three early returns —
Confirm-cancel, the fingerprint error, the `engraveSeed` error — to a point where
the words are observable, so the defer is unpinned.

**Why this is not urgent and not nothing.** The wipe that carries §10.2.2's
weight is `clear(m)` beside `clear(rec)`, and that one IS pinned, five ways
(`TestMnemonicWordsAreZeroWhenThePlateReachesEngrave` — reverting the whole fold
leaves exactly that one test failing). The defer only covers paths where **no
plate was ever built**, so the residency window it guards is the few hundred ms
of a `showError` screen, not a ~21-minute cut.

But the fold's own comment now *justifies keeping the defer*, and an unpinned
justification is how a later "simplification" removes it — which is the exact
shape of the Critical this whole thread began with.

**What would close it:** drive each early return with `unlockMnemonicHook` set and
assert the words are zero after the flow returns. Scheduled to B2b rather than
now because B2b writes the residency-timer tests anyway, and they need the same
observability.

### F-86 — CLOSED 2026-08-11 (post-merge polish: '%' added to Boldprogress45's alphabet; 3 tests had relied on it being invisible) — `%` renders as zero pixels in the KDF progress screen (owning phase: **post-merge polish and hardening** — operator ruling 2026-08-10; re-assigned from the font cycle)

**RE-ASSIGNED 2026-08-10 to the post-merge polish and hardening phase** (operator ruling). Was: with F-78's font cycle. Still open; scheduled, not excused.

`unlockDerive` (`gui/unlock_kdf.go`) formats its percentage as `"%d%%"` in
`ctx.Styles.progress`, which is `poppins.Boldprogress45` — the engrave timer's
face, which has no `%` glyph. **Measured during B2a-ii implementation:**

```
width("50")  = 57
width("50%") = 56      # the % contributes NOTHING, and costs a pixel
Styles.lead  renders the same string 12px wider — it HAS the glyph
```

So the operator sees a bare number where the code says a percentage. It is
**legible rather than wrong** — the lead line beneath reads "About N seconds
left." and carries the meaning — which is exactly the shape F-78 describes for
the missing `·`: a glyph whose absence degrades to something merely sloppy, so
nobody notices.

Pinned by `TestProgressStyleRendersNoPercentSign` so it cannot silently change,
and the plan's code was kept verbatim rather than edited around it.

**Fix with F-78, not before it.** The real repair is the font — adding the glyph
to `poppins.Boldprogress45` fixes this and any future numeric readout in that
face, where substituting a different string at this one call site treats the
symptom. Note this is the **display** font, not the engraving alphabet, so the
2-stroke-width minimum-feature rules do not apply.

**The general form, which is the reason to keep filing these:** `uiContains`
compares extracted **text, not pixels**, so no screen test in `gui` can see a
missing glyph. Both this and F-78 were found by MEASURING WIDTH. Until F-78's
rasterising check exists, every new screen in a new face is unverified in exactly
this way.

### F-85 — §2.2 does not name the during-engrave residency (owning phase: before the release tag)

**CLOSED 2026-08-10 (`b6bbae1`).** SPEC §2.2 item 13 names the during-engrave residency, in the same register as item 9, with physical custody as the control — and states its own narrowness explicitly, since the broad reading of F-83 is what cost F-108. §2.3 gains the operator half: do not start a secret cut you will walk away from.

SPEC §2.2 lists what this design does **not** defend against. It does not say
that a secret plate's geometry is resident in SRAM for the whole of its cut, and
cannot be wiped until the cut ends. See F-83, which records why that is
unavoidable rather than a defect.

One paragraph, in the same register as items 9 and 11, saying that during a
secret engrave the seed is recoverable from SRAM by someone with physical access
and an SWD probe, and that physical custody is the control. It changes no
behaviour and no test.

**Why it is owed before the tag and not sooner:** the SPEC is GREEN, so this is
an amendment with its own gate, and amending a normative document as a side
effect of an implementation commit is exactly the bundling the standard workflow
separates out. **Why it is owed at all:** §10.2.2's wipe is described to the
operator as removing the secret, and it removes the *record*. The gap between
what the machine does and what the operator has been told is the whole subject
of §2.2.

### F-84 — `SeedScreen` gains `NoEdit` (owning phase: B2a-ii, Task 6 — implemented there, not deferred)

**CLOSED 2026-08-10 (already recorded as implemented, not deferred, in the entry's own header) — SeedScreen.NoEdit at gui/gui.go:2341,2388,2464, wired at gui/unlock_session.go:291; killed by reverting the production call site to &SeedScreen{}, which fails TestPayloadSeedScreenRefusesEditing.**

Recorded rather than deferred, because it changes a screen the NFC scan path also
uses. `SeedScreen.Confirm`'s edit affordance writes `mnemonic[selected]` in
place; for a **typed** seed that is a typo fix, for a **payload-sourced** one it
is corruption, and the flow then derives a matching fingerprint and engraves it —
so the plate is internally self-consistent and does not restore the payload's
wallet.

Zero value stays editable, so every existing caller is unaffected by
construction. Found by the B2a plan's R0 round 0 (M5); round 1 (I1) then found
that the guard must sit on the **click handler**, not the nav layout, because
`Filter.matches` gates a button event on identity with no bounds check.

### F-83 — the plate cannot be wiped until the engrave finishes — ACCEPTED LIMITATION, not a follow-up (operator, 2026-08-08)

`validateMdmk`, `backup.SeedString`, `engraveSeed` and `toPlate` copy a record
into Go strings and into `Plate.Spline`, none of which can be zeroed.
`gui/ms1_decode.go:19-20` already carries the same caveat for the display path.

**Operator decision 2026-08-08: "the one honest gap is unavoidable."** The plate
must be resident while the needle moves. No ordering of wipes changes that, and a
plate pipeline over `[]byte` would **relocate** the secret rather than remove it.

> **CORRECTED 2026-08-09 — the MECHANISM recorded here was wrong, and the truth
> is worse.** This said the plate "*is* the geometry being cut", which implies a
> materialised buffer that the plaintext was consumed to produce. It is not.
> `bspline.Curve = iter.Seq[Knot]` (`bspline/bspline.go:22`) is a **lazy
> iterator**, so `Plate.Spline` holds a **closure over the plaintext**, re-read
> on every knot for the whole ~21-minute cut. The acceptance stands and the
> conclusion stands; the reason given for it did not. Found by B2b's recon
> (`RECON_b2b_idle_timer_surface.md`), and it is the same mechanism that makes
> F-88's `clear(words)` remedy destructive. Filing it as work-to-be-done would be dishonest bookkeeping: a register whose
value is that every open item is real cannot carry one that will never be
actioned.

**What is therefore true, stated once so nothing downstream overclaims:** during
a secret engrave the seed is recoverable from SRAM by an attacker with physical
access and an SWD probe (§2.2 item 9, live because `debug enable: 1` is measured
in §3). §10.2.2's wipe removes the **record** — the only copy that outlives the
plate — and that is the whole of what it claims. B2a-ii's `clear(rec)` at plate
construction is what makes "the record" and "the plate" distinct lifetimes rather
than one.

The SPEC amendment this owes is **F-85**.

### F-82 — `seal.Deriver` and the folded `DeriveKey` have no Rust counterpart (owning phase: ownerless residue)

The chunked derivation is device-only: the host has no progress bar to draw.
B2a-i §3d then folds `DeriveKey` onto it, so `seal` no longer calls
`crypto/pbkdf2` at all while the Rust side still uses its own PBKDF2.

**The Rust-primary rule does not bind it**, and the reason is worth stating
rather than assuming: the rule binds *normative behaviour* — wire format,
identity and stub algorithms, validation, admission. A PBKDF2 that produces
byte-identical output changes none of them. The contract is the six
`derived_key_hex` literals in `seal/testdata/vectors.json`, produced by Rust,
which the Go tests assert directly; measured, the fold also agrees with the
old `crypto/pbkdf2` body byte-for-byte at iterations −1, 0, 1, 2, 3, 999,
100000 and 100001. Recorded so a future reader does not mistake the asymmetry
for drift.

### F-75 — stale `gui/bundle_flow.go:224` citations outside the SPEC (owning phase: ownerless residue)

`bundleReviewFlow` is at `gui/bundle_flow.go:227`; `:224` lands on a comment
line — ordinary citation decay, and exactly what `plan-cite-gate.sh` exists to
surface (it resolves `:224` as "ok" and prints the comment, which is the gate's
stated blind spot working as designed).

Corrected in `SPEC_encrypted_payload_delivery.md` (three occurrences) by the B1
cycle. Two copies remain in shipped records:

- `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseA.md:638`
- `design/CONTINUITY_2026-08-07b.md:148`

Both are historical artefacts of merged work. Per F-72's precedent they are
recorded here rather than rewritten — the citation is wrong, the record of what
was believed at the time is not.

## Resolved

### F-81 — WITHDRAWN 2026-08-08 before it was ever open

Filed by the B2a plan's first draft: "a FAILED secret plate stays resident while
its retry prompt is up." It described a residency window created by wiping the
record *after* `EngraveScreen.Engrave` returned.

R0 round 0 (C1) showed that design was both **unnecessary and non-conforming** —
`newEngraverJob` holds `plate.Spline` and nothing reads the record after the
plate is built, so the record can be zeroed *before* the engrave is entered; and
waiting for `Engrave` to return did not satisfy §10.2.2 anyway, because Back
while running does not return, it pauses. The plan now wipes at plate
construction, so the window this described does not exist.

Recorded as withdrawn rather than deleted, so a reader of that R0 report finds
its disposition.


*(Moved below the marker 2026-08-08. Both were marked CLOSED in their
headings but still sat in the open list, unlike F-67–F-70 which were moved
when they closed — the record defect CONTINUITY_2026-08-08 §9 asked to fold
into the next commit touching this file.)*

### F-73 — CLOSED 2026-08-07 — the XIP read at the NORMATIVE 0x10E00000 is verified on hardware

**Operator decision 2026-08-07: leave it, do not buy a board for this.** Filed so
it is tracked rather than remembered.

Task 7 Step 4 was run on a Pico 2 and proved the read MECHANISM: a fixed-address
`unsafe.Slice` compiles under TinyGo 0.41.1, executes on RP2350 silicon, and
returns byte-exact flash contents, with `ParseHeader` parsing them correctly.

    probe @0x10300000 first 16: 4d 4e 45 4d 42 4c 4f 42 01 00 00 00 00 00 00 00
    probe header OK — pub_len=203 ct_len=0 sealed=false

`pub_len=203` is independently right (3 records x 67 chars + 2 LF).

**What is NOT covered.** That test ran at `0x10300000`, not at `PayloadAddr`.
The Pico 2 has **4 MB** of flash (`flash size: 4096K`, measured) and
`0x10E00000` is **14 MB** in, so the region does not exist on that board:

    ERROR: File size 0x100 starting at 0xe00000 is too big to fit in
           flash size 0x400000

and an XIP read there silently **aliases** to `0x10200000`. `cmd/sealread`'s
"no payload at 0x10e00000 — CLEAN state" line is therefore a correct-LOOKING
answer from the wrong address, and must not be cited as evidence about the
normative region. The doc comment in `cmd/sealread/main.go` says so at the
point of use.

**What would close it:** a 16 MB RP2350 part — a **Pico Plus 2**
(`__flash_size=16M`, which is why the fork's own build target is `pico-plus2`;
`pico2-w` does NOT qualify, it `inherits: ["pico2"]` at 4 MB) or the SH2 itself.

**Board chipids now live in one place: `design/HARDWARE_INVENTORY.md`.** Check
the chipid before every flash — on 2026-08-07 two RP2350s were in BOOTSEL at
once, and `tinygo flash` / `picotool load` take whichever they find. The bench
also holds a **Pico 2 W** (`0xb3d19289d3ec3f0e`) that is easy to mistake for the
rehearsal Pico: same form factor, same 4 MB, but `secure boot: 0` — it runs
unsigned images — and its LED is not where a Pico 2's is. It does **not**
qualify for this follow-up.

**Why the residual risk is small but real:** §5's arithmetic fixes the address,
and the 2026-08-06 hardware validation already showed the SH2 accepts a
data-family UF2 there byte-exact with the firmware region's sha256 unchanged.
What is untested is only that a 14 MB offset behaves like a 3 MB one on a part
where both are in range. Small — but this cycle has twice produced
plausible-for-the-wrong-reason results, so it is recorded rather than waved off.

**Also still open, same hardware:** the PBKDF2 rate on an RP2350**B**. §7.1's
measured 9,715 it/s is from an RP2350**A** (Pico 2); the SH2 is a **B**. Tracked
in SPEC §12 residual; grab it whenever a B is flashed.

**CLOSED 2026-08-07.** Verified on the SeedHammer II (RP2350B, 16 MB) with
Phase B1 firmware. Full record: `design/HARDWARE_RESULT_2026-08-07_phaseB1.md`.

- Wrote the payload and read it straight back off the device at `0x10E00000`:
  magic `MNEMBLOB`, `pub_len=1125` — independently right (12 records = 1114
  chars + 11 LF), `ct_len=0`, and kdf/aead/iterations all zero per §6.2's
  unsealed rule. This step is IMPOSSIBLE on the 4 MB Pico, so it is new
  information rather than a repeat.
- §10.1 negative path (region erased): entry absent, 8 dots, every other
  program still reachable, Engrave Bundle unmoved in slot 5.
- §10.1 positive path: entry present, 9 dots, and the §6.6 hash on screen
  `fc10 4898 39dc 6da3 8f56 575d 45f7 655b` byte-identical to the host's —
  which an aliased or erased read cannot produce. First end-to-end proof that
  §6.6 agrees across host and RP2350 silicon.

- Present → absent: a payload known present on the previous boot was erased
  (host readback confirmed `MNEMBLOB` → all `0xFF`), and the entry disappeared,
  8 dots, Engrave Bundle still slot 5. So the menu reflects the region at each
  start rather than any cached state.

**The RP2350B PBKDF2 rate is NOT part of this closure, and is no longer tracked
as a benchmark run.** `cmd/kdfbench` says to run on a Pico 2 / Pico Plus 2 and
NOT the SH2, and argues the rate transfers because PBKDF2 here is compute-bound
and cache-resident while A/B differ only in package, pins and flash banking. Both
existing figures already come from a Pico 2, so a re-run adds nothing and an SH2
run costs a firmware overwrite plus reflash. Operator decision 2026-08-07: skip
it. SPEC §7.1 is amended to confirm the rate **in situ during B2** by timing the
real unlock KDF — still owed before release, stronger measurement, different
method.

**UPDATE 2026-08-07 (B1 planning): the SH2 is available, so this closes in B1.**
The "leave it, do not buy a board" decision above was about not *buying*
hardware; it never applied to the SH2 itself, and the owning phase already said
"or the first SH2 session — whichever comes first". `IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB1.md`
Task 7 carries the procedure, including the RP2350**B** PBKDF2 rate on the same
trip. **Not deferrable past B1.**


### F-74 — CLOSED 2026-08-08 — a build gate now covers a Go plan's code

`scripts/plan-build-gate.sh` extracts ```rust blocks into a scratch crate and
builds them. Plan B's documents are **Go**, so every Go fragment in a plan
reaches its reviewer uncompiled — the B1 plan says so in its own gate-coverage
section rather than letting the brief imply coverage it does not have.

This is not hypothetical. Continuity §4 measured **folds** as the dominant defect
source this cycle (three of four introduced a defect nobody had looked at), and
compile errors as a recurring class among them — round 3 of the Plan A review
burned an entire opus round on five of them, every one a `cargo build` away.
Plan B has the same exposure with none of the coverage.

**What would close it:** a Go equivalent that extracts fragments into a scratch
package beside the real `gui`/`seal` and runs `go build`. It must state its own
blind spot the way the Rust one does — fragments that are *modifications* to
existing files (a new `case` in a switch, a field added to a struct) cannot be
assembled mechanically and still need a reviewer's execution pass.

**CLOSED 2026-08-08** by `scripts/plan-build-gate-go.sh`, before B2's plan
review as scheduled. Two tiers, because Go plans are mostly MODIFICATIONS that
cannot be assembled mechanically:

- **TIER 1 gates.** Blocks anchored to a NEW file are assembled into a scratch
  copy of the fork and run through `go build` + `go vet`. Real type checking.
- **TIER 2 is informational.** Every other block is offered to `gofmt -e` under
  four wrappers (file, func body, struct, interface). Proves SYNTAX, never
  semantics — and the script says so in its own output.

Verified in BOTH directions, which is the standard this cycle kept finding
tests failing: a valid whole file exits 0; the same file with `p.NoSuchField`
exits 1 reporting `p.NoSuchField undefined (type *seal.Payload has no field or
method NoSuchField)`.

Five things it learned the hard way, all now encoded:
1. Plans write anchors PATH-FIRST (`` `x.go`, new file ``), not verb-first. The
   first version matched only verb-first and type-checked NOTHING while
   reporting success.
2. It extracted a file and then never compiled it — the exact bug
   `plan-build-gate.sh` had with `tests/seal_cli.rs`. Now step 3 tests what
   step 2 actually wrote instead of guessing from mtimes.
3. Plans omit the `package` clause; it is synthesised from the path.
4. A block anchored as a new file but containing `...` is a fragment wearing a
   whole-file label. It is DEMOTED to tier 2 with a message, not failed on.
5. An anchor must not leak past a heading, or a later unrelated block is
   assembled into the previous task's file.

It also baselines `go vet` against the unmodified fork, so pre-existing
findings (`testing.ArtifactDir requires go1.26`) cannot fail a correct plan.
Failing on correct input gets a gate ignored as fast as crying wolf does.

**On the B1 plan it reports honestly: no whole-file blocks, nothing
type-checked.** That is a finding about how plans are written — B2 should carry
COMPLETE files where it can, so tier 1 has something to check.

**Why not in B1:** it is tooling, and bundling tooling into a feature commit is
the third-commit case the standard workflow separates out. B1 states the gap;
B2 should not have to.

### Closed 2026-08-07 — F-67 through F-70 (the encrypted-payload prerequisites)

All four were owed *before* Plan B could let Go bind to either artefact, and all
four landed that day. Bodies retained verbatim below.

- **F-67** — fixed in the fork, `4192458`: `codex32.ValidMD` now caps md1
  codewords at 93 symbols, matching md-codec's `REGULAR_CODE_SYMBOLS_MAX`.
  Convergence port, Go-only (Rust was already correct). Mutation-verified: the
  cap removed → `TestValidMDRejectsOverLongCodeword` FAILs at n=81 and n=496.
- **F-68** — ~~closed by `scripts/plan-cite-gate.sh` (`7cdcbfc`)~~ **NOT CLOSED.
  Mis-attributed; corrected 2026-08-11 by the deferred-follow-up triage.**
  `plan-cite-gate.sh` resolves every `file:line` and `pkg.Symbol` in a plan
  against real source — a different tool for a different problem. F-68 is that
  **`plan-build-gate.sh` compile-checks the CLI tests but never runs them**, and
  `scripts/plan-build-gate.sh:163` still passes `--no-run` today, with the
  script's own header (line 30) still declaring the gap. The cite gate is
  genuinely useful — it caught three of the author's own mis-cited lines on its
  second run — but it closes nothing here. Low severity, since the real suite
  now exists and runs; the *record* was wrong, which is the class this cycle
  has repeatedly found to be wronger than the code.
- **F-69 / F-70** — closed in `0ca972a`: `--seal-secret` now covers a bare BIP-39
  mnemonic as well as `ms1` (`classify` needs a bech32 `1`, so a 24-word phrase
  returned `Err(NoSeparator)` and sealed with no ceremony), and §9 + §12 item 6
  document the flag. Framed per operator decision as a **best-effort anti-footgun,
  not a security boundary**.

### F-67 — the Go `MDDataSymbols` lacks Rust's 93-symbol codeword cap (owning phase: Plan B, before the public-section decode ships)

**Found by the §6.3 scoped re-review, 2026-08-07.** Rust's
`md_codec::codex32::unwrap_string` rejects an over-93-symbol codeword
(`REGULAR_CODE_SYMBOLS_MAX` — "cycle-4 I1: β has order 93, degrees d and d+93
alias"). The fork's `codex32/mddata.go:15` `MDDataSymbols` has **no such cap**:

```
data-syms= 80  total-syms= 93  len= 96   MDDataSymbols err=<nil>
data-syms=496  total-syms=509  len=512   MDDataSymbols err=<nil>
                                          Rust: StringSymbolCountOutOfRange
```

So host and device disagree about which records are admissible at §6.4's
512-byte bound — on the exact function §6.3's decode requirement just made
normative. Fail-closed in both directions, and **not introduced** by that
amendment.

**Per the Rust-primary rule this is a CONVERGENCE port, not a leading change:**
Rust is already correct, so the fix lands in Go only, and the mandatory
"does the same defect exist in Rust" check is answered — it does not.

Also consider tightening §6.4's 512-byte per-record bound, which is roughly 5×
wider than any valid md1 codeword.

### F-68 — `plan-build-gate.sh` compiles the CLI tests but never runs them (owning phase: before Plan B's plan review; NOT gating Plan A)

The gate extracts `tests/seal_cli.rs` and, since 2026-08-07, compile-checks it
with `--no-run`. It cannot run it: the scratch crate's binary is built from the
UNMODIFIED `main.rs`, which has no `seal` subcommand, so every case would fail
for a reason that says nothing about the plan.

**This blind spot has cost one Critical already.** Round 5 found that
`refuses_a_record_carrying_a_cr` — added by the round-4 fold, in the same commit
that added the compile check — passed a single md1 chunk where the card-set
decode demands three. It compiled, and failed identically mutated and unmutated.
An expensive review round found what one `cargo test` would have. The header
names the gap honestly; naming a gap is not closing it.

**It is mechanically closable.** Rounds 4 and 5 both assembled the full crate by
inserting Task 9's three `main.rs` fragments verbatim from the plan's fences
(enum variants, the two early-return blocks, `run_seal_cli`/`run_hash_cli`), then
built the real `me` binary and ran the suite. If a reviewer can do it from the
document, so can the script.

**Why this is not gating Plan A:** the plan's correctness was re-verified by hand
after the fix — 11/11 unmutated, and under the CR-trim mutation exactly one test
fails, the new one. And once implementation starts the real crate exists, so
`cargo test` runs everything and the gap closes on its own. It binds again at
**Plan B's** plan review, where the same extract-and-check pattern recurs against
Go. Do it before that review, not after.

Watch for the stale-binary trap when mutating: restoring a file with `mv` from a
backup can leave an mtime older than the built artifact, so cargo skips the
rebuild and the "restored" run is still the mutant. `touch` the file. Round 4 hit
this, and so did the fix verification for this entry.

### F-69 — amend §9 and §12 item 6 for `--seal-secret` (owning phase: before Plan B's plan review; the two artefacts must agree before Go binds to either)

Plan A Task 9 added a `--seal-secret` opt-in that the spec does not have. §9's
synopsis omits it and §12 item 6 records `ms1` as ADMITTED with no opt-in, so the
**spec's own documented invocation** — `me seal <ms1> --out x.uf2` — exits
`EXIT_REFUSED` in the shipped binary. The plan flags this as a deliberate
divergence (safer than the spec, not looser) and says to file the amendment
rather than leave the artefacts disagreeing. This is that entry.

Amend the spec to match the implementation; do **not** remove the flag.

### F-70 — the `--seal-secret` guard covers `ms1` only, not a raw BIP-39 mnemonic (owning phase: with F-69, same spec amendment)

Found by the whole-diff review (`design/agent-reports/REVIEW_plan_a_whole_diff_2026-08-07.md`,
Minor). `main.rs`'s guard is
`!seal_secret && secret.iter().any(|r| matches!(classify(r), Ok(Format::Ms)))`.
`classify` on a 24-word mnemonic returns `Err(NoSeparator)` — there is no `1`
separator — so the guard never fires, while `record_or_mnemonic` explicitly
admits the mnemonic. Measured:

```
me seal "bacon … bacon" (×24) --out a.uf2     [no --seal-secret]  → exit 0, 512 bytes written
me seal <ms1>                 --out a.uf2     [no --seal-secret]  → exit 3, demands the flag
```

Both inputs are the same seed material. The flag's own doc comment says
"Required to encrypt an ms1 (a seed). Sealing a seed must never be accidental."

**Not a defect against Plan A** — the plan specifies the `ms1`-only form and the
code is plan-conformant; the resulting blob is correct and §10.2.1's allow-list
admits a mnemonic in the encrypted section. It is an inconsistency in what the
opt-in *means*, so it belongs with F-69's amendment rather than to a code fix
made in isolation.



### Funds-safety audit follow-ups (`me-*`) — SHIPPED in v0.4.0 (PRs #1–#4, 2026-07-09)
Six cycle-sized items descoped from the funds-safety audit (F8–F18 subset), each run through SPEC → opus R0 gate (0C/0I) → single-implementer TDD → mandatory post-impl adversarial review, merged as PRs #1–#4 and released in `mnemonic-engrave` v0.4.0. Full detail retained per entry.

- **`me-preview-stale-plates-and-sidecar-output-validation`** — ✅ **CLOSED by PR #1** (Cycle A, 2026-07-09): dirty-dir refused fail-closed (exit 2, never deletes) + sidecar output signature-validated before a preview is recorded (0-byte/garbage → exit 4). — (funds-audit 2026-07-06, F8+F9, confirmed low; descoped from `SPEC_me_testing_hardening.md` at R0 round 0.) (a) `me bundle --preview` never cleans the output dir: stale higher-index `plate-N.{svg,png}` from a prior (different-wallet) run persist and can be engraved as part of the new wallet's set (proven; `main.rs` `wire_previews` only checks `dir.is_dir()`). Fix fail-closed: refuse a dir containing unreferenced `plate-*` files, or clean exactly that namespace first. (b) `render_plate` (`preview.rs`) treats sidecar exit 0 as sole success — a 0-byte/garbage `--out` file is recorded as a valid preview (proven). Validate: exists, non-empty, SVG root/PNG magic; else non-zero exit, no preview path. Evidence: `agent-reports/funds-audit-D3-bundle-round0.md`, `funds-audit-D5-hygiene-round0.md`.

- **`me-output-file-permissions`** — ✅ **CLOSED by PR #1** (Cycle A, 2026-07-09): NDEF/manifest/preview artifacts now written 0o600 via `write_private` (+ truncate fix). — (funds-audit F10, confirmed low.) NDEF/manifest/preview artifacts written 0o644 world-readable; manifest embeds raw md1/mk1 strings, previews depict scannable QR. Write 0o600 (Rust `OpenOptions`, Go `os.WriteFile` mode); assert `mode & 0o077 == 0` in tests. Evidence: `funds-audit-D5-hygiene-round0.md` (D5-2) + verdict.

- **`me-preview-render-goldens`** — ✅ **CLOSED by PR #2** (Cycle B, 2026-07-09): whole-SVG SHA-256 + exact M/C command counts + decoded-`img.Pix` SHA-256 + black-pixel-mass goldens with `-update`, pinned over the disc-brush-corrected output. — (funds-audit F15, low; descoped from SPEC B4.) No SVG path-content / PNG pixel golden: a pen-state swap in `render_svg.go`/`render_png.go` renders a wrong preview with the Go suite green. Add `render_golden_test.go`: pinned `d`-attribute hash per mode + deterministic PNG byte hash + M-vs-C token-count assertion, `-update` regeneration. Evidence: `funds-audit-D6-tests-round0.md` (D6-5) + verdict. **ORDERING (fable decision 2026-07-06): perform the [[me-preview-png-stroke-width]] fix FIRST, within this same cycle, BEFORE pinning the PNG byte-hash golden — pinning first would force an immediate re-baseline of a just-created golden when the stroke fix lands.**

- **`me-fuzz-proptest-targets`** — ✅ **CLOSED by PR #3** (Cycle C, 2026-07-09): proptest P1–P6 (never-panic, ms-always-refused, plate-strings-trace-to-input, ndef byte-length round-trip) in the CI-covered stable suite + workspace-detached cargo-fuzz targets sharing `#[path]`-included invariants (no new pub API; no libfuzzer leak into the stable lock; MSRV 1.85.0 proven). Invariants confirmed already-true = insurance. — (funds-audit F18, low; descoped from SPEC B7.) No fuzz/property targets. Add cargo-fuzz `fuzz_convert`/`fuzz_run_bundle` (never panics; any ms-HRP → RefusedSecret; manifest strings ⊆ input) + proptest round-trips; CI smoke-run only. Evidence: `funds-audit-D6-tests-round0.md` (D6-7) + refuting verdict (panic path not currently reachable — insurance, not a live bug).

- **`me-sidecar-discovery-integrity`** — ✅ **CLOSED by PR #4** (Cycle D, 2026-07-09): dropped the `$PATH` fallback (co-located-only discovery via a pure `locate_in`) + `ME_PREVIEW_BIN` explicit opt-in that fails loud (exit 2) when set-but-missing; version gate still applies to an explicit binary. A planted `$PATH` `me-preview` is no longer auto-reached. — (funds-audit F11, low.) `locate_sidecar` falls back to `$PATH` and the version gate is a spoofable string match; a planted `me-preview` receives the (public-only) payload and writes attacker-controlled files into the preview dir. Consider hash-pinned or co-located-only discovery. Evidence: `funds-audit-D5-hygiene-round0.md` (D5-4).

- **`me-preview-png-stroke-width`** — ✅ **CLOSED by PR #2** (Cycle B, 2026-07-09): 1px hairline replaced with the decided deterministic integer disc-brush (radius 2 at default → ~4.85× the black-pixel mass), no AA; pixel-mass + discRadius + 1px-floor tests. — (funds-audit F13, low.) PNG preview draws 1px hairlines vs SVG's physical 0.3mm strokes — legibility mis-assessment only (centerlines identical); at the default ~1000px render the honest stroke is ~3–4px, so the PNG under-draws ~3–4×. Evidence: `funds-audit-D4-sidecar-round0.md` (D4-3). **DECIDED (fable, 2026-07-06) — fix, don't document away; folded INTO [[me-preview-render-goldens]] (one cycle, stroke fix first):** replace the 1px `drawLine` with a **deterministic disc-brush** — stamp a precomputed integer disc of radius `max(1, round(strokeWidth*scale/2))` at each Bresenham step (round caps/joins for free, matching the SVG's `stroke-linecap/linejoin: round`; fully integer, NO anti-aliasing — the PNG golden pins byte hashes, so determinism is load-bearing; 1px floor so downscaled renders never lose strokes; no new dependency — rejected `x/image/vector` AA as golden-fragile overkill and centerline-only documentation as leaving the artifact misleading). Add a pixel-mass regression test (black-pixel count for the reference md1 at default scale must reflect the multi-px stroke; flips red on hairline regression) + a `strokeWidth*scale → px` mapping unit test. PNG renderer is fork-native (upstream `golden.go` is SVG-only) — no upstream-binding constraint; `render_svg.go` untouched.

### SeedHammer per-tier review nits & cycles — RESOLVED (2026-06-18 → 2026-06-21)

- **`seedhammer-t4-review-nits`** — ✅ **RESOLVED 2026-06-19** (fork `8eb51d7`; see Resolved §FOLLOWUPS-nits-burndown). **3 non-blocking Minors** from the T4 whole-diff exec review (`design/agent-reports/seedhammer-T4-seed-xpub-mk1-exec-review.md`, GREEN 0C/0I; shipped fork `main` `e4ca173` 2026-06-19). Sub-cycle nits, sweepable opportunistically (NOT cycle-sized): **(M1, cosmetic)** the xpub verify screen renders the path in `h`-form (`m/84h/0h/0h`) while the decode/inspect flow renders `'`-form (`m/84'/0'/0'`) — same device, two notations; *not* an interop defect (the path string is never serialized — only path components go on-card; bytecode round-trip is exact). Pick one notation for display consistency. **(M2, defensive)** `byte(total-1)`/`byte(i)` in the chunked-header build (`mk/encode.go`) are unmasked; harmless today (worst-case chunk count under encoder limits is 22 < 32) but a future stub/path-limit increase could silently wrap — add a `& 0x1f` mask + a guard. **(M3, robustness)** `codex32.MKChecksumSymbols` returns `nil` on an unreachable `inputData` error instead of surfacing it; gated by `ValidMK` in tests so currently unreachable — consider returning an error for defense-in-depth.

- **`seedhammer-10a-encoder-nits`** — ✅ **RESOLVED 2026-06-20** (M-1 n-mismatch guard done `8eb51d7`; M-3 skipped/optional; **M-2 = the pre-tag `tinygo build ./cmd/controller` gate** — static TinyGo-compat PROXY audit done 2026-06-19 (`design/agent-reports/seedhammer-tinygo-compat-static-audit.md`): LIKELY-clean, no likely-break (only generic = trivial `cloneSlice[T]`; bit-packing fixed-width-uint correct on 32-bit; new concurrency clones an already-shipping scanner; no banned stdlib). **AUTHORITATIVE gate ✅ GREEN 2026-06-20** — fork GitHub Actions enabled; `test.yml` `tinygo-device-build` (`nix develop --command tinygo build -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`) ran on `65a1018` = **success** (~6 min; the `tests` job also green). The proxy audit was correct — the full firmware compiles clean on the RP2350 device target. M-2 RESOLVED. **3 non-blocking Minors** from the #10a (md1 encoder) whole-diff exec review (`design/agent-reports/seedhammer-10a-md-encoder-exec-review.md`, GREEN 0C/0I; shipped fork `main` `3a55ae5` 2026-06-19). Sub-cycle nits: **(M-1)** the Go encoder computes `kiw` off `pathDecl.n` while Rust uses `Descriptor.n` — byte-identical for all well-formed/decoded descriptors (the two `n`s agree post-canonicalize), but a defensive assert that they match would harden against a future author-built AST. **(M-2, pre-tag gate)** `md/canonicalize.go` introduces the first firmware-PRODUCTION Go generic (`cloneSlice[T]`); it is TinyGo-safe by inspection but **a one-time `tinygo build ./cmd/controller` (tags `tinygo,rp`) should be run on the hardware target before any firmware TAG/release** to confirm the generic + the new `md/encode` paths compile under TinyGo (host build + import-safety only proven so far; same standing CI gate noted for T4). **(M-3)** `sort.SliceStable` is used on the encoder/TLV path — already firmware precedent, optionally replaceable with the hand-rolled insertion sorts used elsewhere in the diff for allocation determinism.

- **`seedhammer-10b-walletpolicy-nits`** — ✅ **RESOLVED 2026-06-20** (M-1+M-2 comment fixes `8eb51d7`; **M-3 `sh(wpkh)` projection ✅ SHIPPED fork `main` `4bcef4b` 2026-06-20** — the `InnerWpkh` discriminant + `classifyPolicy sh(wpkh)→PolicySingle` decoder arm + `ScriptSh`+`PolicySingle`+`InnerWpkh`→`P2SH_P2WPKH` projection; BIP-49 golden byte-exact (`m/49'/0'/0'`→`37Vuc…`/`34K56k…`), no `P2SH-P2WPKH`↔`P2SH-P2WSH` collision (disjoint switch), A6 fuzz both-halves coupled; spec/plan/exec-review all R0 GREEN, independently re-derived — `design/agent-reports/seedhammer-sh-wpkh-{spec-R0-round0,plan-R0-round0,exec-review-round0}.md`. One cosmetic Minor ✅ RESOLVED fork `01b62a9` 2026-06-20: the fuzz `isBip380ExpressibleShape` doc-comment now accurately states its `(root, policy, renderable)` inputs + the one-directional `expandOK⇒expressible` rationale.). **3 non-blocking Minors** from the #10b (md1 wallet-policy display/verify) whole-diff exec review (`design/agent-reports/seedhammer-10b-md-walletpolicy-exec-review.md`, GREEN 0C/0I; shipped fork `main` `bb0e506` 2026-06-19). **(M-1)** stale comment at `md/md.go:1093-1094` says `canonicalOrigin` is "never used to substitute a renderable key's OriginPath" — now contradicted by the intentional R0-I1 fallback in `gui`/`md` expand (behavior correct; comment misleading — update it). **(M-2)** the `tamperedCSIDChunks` test constant's comment says its csid is "real+1"; the actual value isn't literally real+1 (test is still sound — it just needs a consistent-wrong csid; fix the comment). **(M-3, small scope/capability gap)** `sh(wpkh)` (P2SH-P2WPKH singlesig) is silently **display-only**, not projected to a `*bip380.Descriptor` for address-verify — a reduction vs the plan's D2 list and vs Rust (which can express it). SAFE (never address-verified → never a wrong-address), but if on-device verify of legacy nested-segwit *singlesig* is wanted, add the `ScriptSh`+`PolicySingle`+`InnerWpkh`→`P2SH_P2WPKH` projection branch (needs an InnerWpkh discriminant analogous to InnerWsh). Decide deliberately whether it's in scope.

- **`seedhammer-t5-bundle-nits`** — ✅ **RESOLVED 2026-06-19** (fork `8eb51d7`; M-1 done, M-2 no-action). **2 non-blocking Minors** from the T5 (guided bundle sequencing) whole-diff exec review (`design/agent-reports/seedhammer-T5-bundle-sequencing-exec-review.md`, GREEN 0C/0I; shipped fork `main` `e4013a8` 2026-06-19). **(M-1, divergence-hardening)** the program-lockstep consts (`npage`/`npages`/wrap bounds/`layoutMainPlates`) are hand-keyed off `engraveBundle` with no compile-time guard — correct + TDD-covered today, but a FUTURE program insertion repeats the T4 lockstep failure surface. Optional: a `const _ = uint(qaProgram - (engraveBundle + 1))`-style static assertion (or a `//go:` doc comment) pinning `engraveBundle` as the last navigable program. **(M-2, defense-in-depth note, no action)** `bundleEngrave`'s "verified card won't fit a plate" branch (`gui/bundle_flow.go:331-337`) aborts the WHOLE set rather than skipping one card — unreachable in practice (real fixtures always fit; `TestBundlePlanValidatesEachPlate`) and the correct direction for a wallet backup (a bundle missing a card is unusable anyway). Noted only.

- **`seedhammer-t6c-ondevice-policy-picker`** — ✅ **BUILT (both phases) — SHIPPED fork `main` `f323dd2` (A) / `76ffcdf` (B) / `8459654` (nits) 2026-06-20.** (Deferred 2026-06-19; user reversed after the recon de-risked it.) The "choose" half of T6's choose-or-supply: an ON-DEVICE multisig/miniscript wallet-policy PICKER (template menu — e.g. `wsh(sortedmulti)` k-of-n — + cosigner mk1 NFC gather + threshold/slot UI), where the device ASSEMBLES the policy md1 itself. Requires the big NET-NEW **`md.EncodeMultisig`** (multi-key `multiKeysBody` + sortedmulti node + threshold + N pubkey TLVs + canonical key-sort permutation; several× the size/risk of T6a's `EncodeSingleSig`; golden-locked vs Rust md-codec) — the most golden-locked-risky piece in the whole T6 program. **Superseded for now by T6b's SUPPLY path:** the user supplies multisig/miniscript policies as md1 strings (the device cross-matches the user's slot + engraves verbatim, NO encoder), which satisfies the choose-or-supply requirement without this. Pick up T6c only if on-device policy AUTHORING is later demanded; it's its own full gated cycle (own R0 focused on the multisig md1 wire format vs Rust). Architect analysis: `design/agent-reports/seedhammer-T6-architect-scope-multisig.md` (mechanism i). **RECON (no build) 2026-06-19 — `design/cycle-prep-recon-T6c-encode-multisig.md` (vs Rust md-codec `descriptor-mnemonic@c85cd49` v0.36.0) MATERIALLY DE-RISKED the original "biggest-risk" framing:** the bit-level multisig encoder ALREADY EXISTS + is byte-cost-tested (`writeNode case multiKeysBody:` `md/encode.go:188-203` ≡ Rust `tree.rs:115-139`); the `split→encodePayload→canonicalize→identity` pipeline is descriptor-shape-agnostic (identity wiring needs ZERO change). So `EncodeMultisig` is a ~90-140 LOC assembler mirroring the 107-LOC `EncodeSingleSig`. **The alleged #1 risk (canonical key-sort permutation) does NOT exist** — Rust md-codec does not lexicographically sort cosigner keys at encode time (`sortedmulti` is spend-time semantics); the only permutation is placeholder first-occurrence canonicalization, already ported/tested in `canonicalize.go`. Residual = a deterministic-cosigner-ordering CONTRACT, not a byte-sort. Goldens solved/non-circular (T6b `gui/testdata/t6b_multisig_full.md1.txt` + vendored `md/testdata/vectors/wsh_*` + a working `md encode 'wsh(sortedmulti(...))' --force-chunked` generator). Sizing ~600-950 LOC: the ENCODER is the small/LOW-risk slice; the real cost is the picker/cosigner-gather UX (~250-400, MED-HIGH). Clean headless-first split (Phase A `EncodeMultisig` / Phase B GUI picker); extend the existing `engraveMultisig` program with a choose-or-supply front-door ChoiceScreen (avoids touching the t5-M1 guard `gui/gui.go:164`). **✅ BUILD APPROVED 2026-06-19 (user)** — with a MANDATORY loud on-device **EXPERIMENTAL / not-proven-end-to-end** warning (unskippable; the device-authored policy has no hardware/coordinator round-trip validation — user must verify the assembled descriptor against their coordinator/wallet before funding). Headless-first split: **Phase A** headless `md.EncodeMultisig` (byte-exact vs Rust md-codec `@c85cd49`, own R0/exec-review centered on wire-format fidelity) → **Phase B** GUI choose-or-supply front-door + template picker + cosigner-mk1 NFC gather + threshold/slot UI + the loud warning + engrave. **Phase A (headless `md.EncodeMultisig`) ✅ SHIPPED fork `main` `f323dd2` 2026-06-20** — byte-exact vs the T6b fixture (`WalletPolicyId 7b716421…`) + Rust `@c85cd49` bit-layout, order-preserving (no key-sort), additive-only; spec/plan(R0 r0→r1)/exec-review all GREEN (`design/agent-reports/seedhammer-T6c-phaseA-*`). 1 cosmetic Minor → **`seedhammer-t6c-phaseA-originmode-errmsg`**: `md/encode_multisig.go:110` `OriginMode` `default:` returns `errMultisigBadScript` (wrong message for a bad origin-mode; unreachable from the public 2-value enum) → ✅ RESOLVED fork `8459654` 2026-06-20 (added `errMultisigBadOriginMode`). **Phase B ✅ SHIPPED fork `main` `76ffcdf` 2026-06-20 — T6c COMPLETE (both phases).** Choose-or-supply front-door on the `engraveMultisig` program (Supply = unchanged T6b; Build = new on-device authoring) + bounded pickers (template / n∈2..5 / k∈1..n / **user-picked self-slot @S** / **build-time fingerprint-presence, homogeneous**) + cosigner-mk1 NFC gather + the pure `assembleBuildPolicy` (sole `md.EncodeMultisig` caller) + a MANDATORY unskippable EXPERIMENTAL warning, reusing the T6b derive/engrave/verify/restore machinery verbatim. Byte-exact vs the T6b fixture (`WalletPolicyId 7b716421` on the fp-absent path, A3 re-derived through the production wrapper); zero lockstep/`md` edits; Supply body byte-identical. USER DECISIONS (2026-06-20): self-slot user-picks @S; fp-presence user-chooses at build (homogeneous, no mixed-presence card). Spec/plan/exec-review all R0 GREEN (`design/agent-reports/seedhammer-T6c-phaseB-*`). **EXPERIMENTAL** — device-authored policy not hardware/coordinator round-trip validated (hence the loud warning). 2 non-blocking Minors → **`seedhammer-t6c-phaseB-nits`**: (M-c) n-picker Back-from-n abandons the Build flow (re-showing the template is optional polish); (M-2) the in-tree A3 test drives `md.EncodeMultisig` directly for the foreign slots (fixture foreign keys lack base58 xpubs) — the wrapper path is covered by `TestAssembleBuildPolicy_Wrapper` + the exec-review probe; a future fixture with base58 foreign xpubs could drive the headline byte-match through the wrapper directly. ✅ BOTH RESOLVED fork `8459654` 2026-06-20 — M-c: Back now steps back one stage (template re-shown), only Back-from-template abandons; M-2: `TestAssembleBuildPolicy_T6bWrapperByteMatch` drives the production `assembleBuildPolicy` wrapper for all slots (foreign keys re-serialized to base58) → `7b716421`. Exec review GREEN `design/agent-reports/seedhammer-T6c-nits-exec-review.md`.

- **`constellation-template-only-engraving`** — **CONSTELLATION-LEVEL design item (user-owned, 2026-06-19).** Allow engraving a wallet-policy md1 as a TEMPLATE-ONLY descriptor (script + origin + use-site, `pubkeys:null`) instead of the full wallet policy — **motivation: fewer costly engraving plates** (a single-sig template ≈ 1 chunk/1 plate vs the full policy's ~2-3). The watch-only wallet is recomposable from template + the key card (mk1, public) or template + ms1 (secret). **Why constellation-level, not a SeedHammer-fork hack** (per recon `design/agent-reports/seedhammer-T6-recon-bundle-composition-stub.md`): the constellation TODAY emits ONLY full-policy md1 in bundles (template is unused/refused), and the mk1↔md1 stub = top-4 of the KEY-DEPENDENT `WalletPolicyId` — a template md1 and the full-policy md1 of the same wallet hash to DIFFERENT ids, so template engraving would BREAK the stub binding unless the stub formula (and toolkit `synthesize`/`restore`/`verify-bundle`) are changed constellation-wide to a template-stable id (e.g. `WalletDescriptorTemplateId`, `identity.rs:71-104`). So this needs a coordinated change across md-codec + mk-codec + the toolkit + SH, decided at the constellation level. **T6 (SeedHammer flagship) is full-policy-only** (user-confirmed); revisit template engraving here only after the constellation adopts it. **Companion update (2026-06-20):** the toolkit + codec layers are now COMPLETE — **phase 1** single-sig template emit/restore/verify shipped at toolkit **v0.59.0**, **phase 2** multisig / general policies + `tr(NUMS,multi_a)` (keyless template `md1` + N keyless cosigner `mk1` stubs + the parallel permutation-search completion in `restore` / `verify-bundle`) shipped at toolkit **v0.60.0** (all R0-GREEN; md-codec / mk-codec needed no change). **The SeedHammer on-device leg is now the only remaining work** — the constellation has adopted template engraving, so this entry is UNBLOCKED for SH-side pickup. **✅ SH LEG SHIPPED 2026-06-21** (fork `main` merge `f924556`; CI Test [incl. `tinygo-device-build`] + Build image GREEN). The full gated cycle delivered opt-in keyless-template engraving for ANY admissible md1 (engrave+verify; default unchanged) + form-aware binding + recovery estimate + depth-≥2 EXPERIMENTAL gate. Spec/plan/exec-review all R0 GREEN — `design/agent-reports/seedhammer-template-engrave-{spec-R0-round0..2,plan-R0-round0..1,exec-review,exec-review-round1}.md`. This constellation item is now COMPLETE end-to-end (toolkit v0.59/v0.60 + SH). Residual deferred display tiers: `seedhammer-template-engrave-policy-summary-display`, `seedhammer-broad-miniscript-renderer`; + the Minor `seedhammer-wdt-id-override-tlv-golden`.

- **`seedhammer-template-engrave-key-search-time-estimate`** — **SeedHammer-UI feature (user-filed 2026-06-20); UNBLOCKED 2026-06-20** — the HOLD prereq LANDED: toolkit **v0.60.0** (`6de53879`) shipped multisig/general template completion (#28 phase 2) **with** the parallel permutation-search engine in `restore`/`verify-bundle` (single-sig was v0.59.0). No longer on HOLD; build it alongside the fork template-engrave cycle (current recon: `design/cycle-prep-recon-seedhammer-template-engrave.md`; the old single-sig-only `design/cycle-prep-recon-constellation-template-only-engraving.md` is SUPERSEDED). Companion of [[constellation-template-only-engraving]] and the fork template-engrave cycle. **What:** when engraving a wallet-policy **TEMPLATE** md1 (keyless), show in the SeedHammer UI an estimate of the **time to recover the wallet by searching key permutations** (assigning the N cosigner keys to the template's `@N` slots) against a known target — **policyID** or **first receive address** — as a function of the **number of keys N**, plus a **link to the GitHub toolkit repo** that performs the search. **Why:** a template md1 omits the keys (and their slot assignment/order), so recomposition can require a permutation search if the slot mapping isn't otherwise recorded; surfacing the search-cost-vs-N tradeoff *at engrave time* lets the user judge recoverability before committing a template plate. **Benchmark inputs (record verbatim — the search engine SHIPPED in toolkit v0.60.0 as the parallel permutation-search completion in `restore`/`verify-bundle`):** ~**6.9 µs/permutation** to test against a known **policyID**, ~**7.4 µs/permutation** to test against the **first address**, measured on a **24-core Intel i7-13700 @ 5.3 GHz** using the **Rust mnemonic toolkit**. Estimate ≈ (permutation count for N) × (per-permutation time). *Illustrative, single-thread, N! key→slot model @6.9 µs:* N=5 ≈ 0.8 ms, N=9 ≈ 2.5 s, N=11 ≈ 4.6 min, N=13 ≈ 12 h, N=15 ≈ 104 d (÷ ~24 with full parallelism). **The authoritative permutation model + parallel/throughput model are owned by the toolkit search feature** (note: `sortedmulti` is key-order-invariant, so the permutation search applies to ordered `multi` / origin-distinct-slot cases — confirm the exact space against the toolkit feature when picked up). **Link:** `https://github.com/bg002h/mnemonic-toolkit` . **Scope:** SeedHammer fork UI only (display the estimate + the repo link); the search itself runs off-device in the toolkit. Surface only on the template-engrave path (full-policy md1 carries the keys → no search needed). Mirror a toolkit companion entry if the search feature wants the SH-side cross-reference. **✅ SHIPPED 2026-06-21** — folded into the template-engrave cycle as slice S6 (fork `main` `f924556`, CI green). The template opt-in displays the honest recovery model (sortedmulti→none; ordered/distinct→N! @6.9/7.4µs) + the toolkit repo link.

- **`seedhammer-own-code-findings-rust-cross-check`** — **Diligence item under the Rust-primary rule (filed 2026-06-20).** The Rust-primary rule's mandatory-check clause (CLAUDE.md Conventions): *whenever a defect is found in a Go port, ALWAYS check whether the same defect exists in the primary Rust implementation.* That clause postdates the **8 own-code findings** already fixed in the fork (cycle `seedhammer-own-code-fix-followups`, merge `39cb5cf`; `design/agent-reports/seedhammer-fork-own-code-bughunt.md`). **Action: retroactively check each of the 8 against the primary Rust constellation** to confirm none is a latent Rust defect we left unpatched. Per-finding triage: **H1** (multisig mk1 self-compare false-PASS), **H2** (md1 `collected()` map-order false-FAIL), **M1** (ms1 entropy-only false-PASS), **L2** (tautological verify), **L1** (DecodeMS1 probe scrub) are largely SeedHammer-GUI/verify-flow logic with no obvious Rust counterpart (likely Go-only) — but the **scrub findings M2 (slip39 group-share), M3 (seedxor per-part), M4 (bip85 pkey.Zero)** mirror toolkit code paths (`mnemonic_toolkit::{slip39,seed_xor,bip85}`) and **must** be checked against the Rust source's zeroize discipline; if Rust under-scrubs the same secret, fix Rust first (with a test) then treat the Go fix as convergence. Size: S (audit + any Rust-side zeroize fixes). LOW-MED priority; do before the next fork bug hunt so the rule is satisfied going forward. **✅ CHECK DONE 2026-06-20** (2 agents; reports `design/agent-reports/seedhammer-own-code-rust-crosscheck-{scrub,verify}.md`). **Result: 7 of 8 GO-ONLY; exactly 1 RUST-ALSO-AFFECTED (LOW).**
  - **GO-ONLY (no Rust action):** H1/H2/M1/L2 (verify-correctness — Rust verify-bundle is strictly more conservative: derived-vs-independently-supplied compare, ordered `Vec`+`sort_by_key(index)` reassembly with no unordered-map iteration, full byte-identical codex32 ms1 compare incl. HRP/language); M2/M3/M4 (slip39/seedxor/bip85 — Rust scrubs via idiomatic `Zeroizing`/`ZeroizeOnDrop` RAII on all paths).
  - **RUST-ALSO-AFFECTED — `seedhammer-rust-l1-selfcheck-ms1-zeroize` (LOW, the Rust-first fix the rule obligates):** `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs:2473` (`self_check_bundle`) decodes ms1 into a bare non-`Zeroizing` `ms_codec::Payload` carrying master-seed entropy, uses it only as an equality oracle (`:2477`), then drops the husk un-scrubbed — violating ms-codec's own documented caller-wrap contract (`ms_codec::Payload` is `#[non_exhaustive]`, intentionally NOT zeroize-wrapped; widening it = deferred breaking change). The OTHER 4 toolkit ms1-decode sites correctly move bytes into `Zeroizing`. **Fix:** mirror the in-file sibling idiom (`bundle.rs:2028-2039`) — move the entropy OUT of `Payload` into `Zeroizing<Vec<u8>>` (Entr→`Zeroizing::new(bytes)`, Mnem→`Zeroizing::new(entropy)`, non-exhaustive `_`→copy) and compare that; existing tests `bundle.rs:2687/2694` are the behavior regression guard (pure refactor, scrub is a type-level guarantee). **✅ HANDED OFF 2026-06-20 (user decision: defer to the other instance).** The toolkit is actively worked by another Claude instance (a `PLAN_constellation_bughunt_fix_program.md` + in-flight source edits on branch `fix/fuzz-build-unrestorable-advisory-cmd-ref`); rather than commit unilaterally, the Rust-first fix is filed as toolkit FOLLOWUP slug **`self-check-ms1-decode-not-zeroizing`** (`mnemonic-toolkit/design/FOLLOWUPS.md`, docs-only commit `3e35afc0`, their working changes untouched) for that instance to pick up. The convergence for fork-L1 closes constellation-wide once the toolkit fix lands.
  - **2 optional NON-defect quality notes (verify report):** (1) `verify_bundle.rs:645` single-sig template `md1_match` is a positional compare with deterministically-ordered inputs — could sort for symmetry with the multisig sibling; (2) some verify-bundle UNIT tests use `supplied = expected.clone()` (`:3409,3462,3605,3739`) — a fixture-level tautology (production + integration tests use independent fixtures, so coverage is real; a genuinely-independent-supplied unit test would harden vs drift). Fold-in candidates if/when the L1 fix is done.

- **`seedhammer-t6a1-walletpolicyid-origin-guard`** — ✅ **RESOLVED 2026-06-19** (fork `8eb51d7`). **1 non-blocking Minor** from the T6a-1 whole-diff exec review (`design/agent-reports/seedhammer-T6a1-headless-exec-review.md`, GREEN 0C/0I; shipped fork `main` `bfff857` 2026-06-19). **M1 (defensive):** `md.WalletPolicyId`'s `resolveOriginRaw` returns an empty origin path SILENTLY in the case (empty origin + no override-TLV + a None-canonical tree like `sh(wpkh)`) where Rust `expand_per_at_n` raises `MissingExplicitOrigin`. UNREACHABLE via the public API — `WalletPolicyId` takes an unexported `*descriptor` and every public entry reaches it only through a decoded descriptor that `validateExplicitOriginRequired` (`md/md.go:1033`) already rejects with the identical rule. Zero impact on the verified byte-exact surface. Optional hardening: make `resolveOriginRaw` return the `MissingExplicitOrigin` error directly (defense-in-depth for any future direct caller).

- **`seedhammer-t6a2-gui-nits`** — ✅ **RESOLVED 2026-06-19** (fork `8eb51d7`). **3 non-blocking Minors** from the T6a-2 whole-diff exec review (`design/agent-reports/seedhammer-T6a2-gui-exec-review.md`, GREEN 0C/0I; shipped fork `main` `072461a` 2026-06-19). **M-1:** regenerate the stale-csid vendored `wpkhMK1` golden in `bundle/verify_test.go` — it carries `csid 0x1c017` (mis-derived from the stub bytes by an older/hand-crafted encoder) while the shipped `mk.Encode` deterministically derives `top20(SHA-256(bytecode))=0xbaa99`; the bytecode is byte-identical and the csid is a non-load-bearing chunk-grouping nonce (the decoder only checks csid CONSISTENCY across chunks; canonical Rust mk-codec draws it from CSPRNG, `key_card.rs:97-98`), so `bundle.Verify` (binding via the integrity hash, not csid) is unaffected — harmless, decode-only, but regenerate/annotate for cleanliness. **M-2:** drop the dead `derived` parameter on `singleSigVerifyFlow` (it correctly re-derives internally instead). **M-3 (cosmetic):** verify-PASS uses a function named `showError` for a success screen — rename/wrap for clarity. (Also notable for the record: a deliberate Go↔Rust divergence — Go `mk.Encode` uses a DETERMINISTIC csid (`top20(SHA-256(bytecode))`) while canonical Rust uses a CSPRNG nonce; both valid since the decoder ignores the csid value.)

- **`seedhammer-t6b-nits`** — ✅ **RESOLVED 2026-06-19** (fork `8eb51d7`; M-1/M-2 done, M-3 no-action). **3 non-blocking Minors** from the T6b (multisig via supplied md1) whole-diff exec review (`design/agent-reports/seedhammer-T6b-exec-review-round0.md`, GREEN 0C/0I; shipped fork `main` 2026-06-19). **M-1 (cosmetic UX):** the reused-key notice in `gui/multisig.go:91-92` names only `reused[0]`/`reused[1]` — with ≥3 matched slots (same seed at 3+ cosigner positions) the 3rd+ index isn't shown; the FIRST-by-index slot is still engraved deterministically and correctly, so this is display-only. **M-2 (polish):** `gui/multisig.go:57` decodes `tpl` then discards it (`_ = tpl`) and `multisigRestoreDocFlow` re-decodes it (`gui/multisig_restore.go:58`) — a harmless redundant decode of public, display-only data; could thread the already-decoded `tpl/keys` through to save the second `ExpandWalletPolicyChunks`. **M-3 (test posture):** `TestMultisigSeedHookSeamExists` (`gui/multisig_fuzz_test.go:59`) only asserts the scrub-hook seam compiles; the behavioral scrub-on-exit is structurally guaranteed by the `defer` (`gui/multisig.go:69-73`, mirrors the proven single-sig pattern) but isn't driven headlessly — matches the single-sig test posture, acceptable. (Also: the Task-6 verify deviation that REMOVED the plan's `d.MS1=""` mask is the CORRECT behavior — it closes a silent-pass hole; documented in the exec review, no action.)

- **`seedhammer-t7b-bip85-followups`** — from the T7b (on-device BIP-85 derive-child → engrave) cycle (shipped fork `main` 2026-06-19; spec/plan/exec-review all R0 GREEN; reviews `design/agent-reports/seedhammer-T7b-{spec-R0-round0..2,plan-R0-round0..1,exec-review-round0}.md`). **`seedhammer-t7b-bip85-index-entry` — ✅ SHIPPED fork `main` `f907eea` 2026-06-20** (user-requested 2026-06-20). The bounded `0..9` ChoiceScreen is replaced with a TYPED numeric child index over the full hardened range `0..2^31-1`. The recon corrected the original "no reusable numeric-entry widget" claim — the passphrase/address keyboard already types digits — so `bip85IndexEntryFlow` clones `typeAddressFlow`/`NewAddressKeyboard` (cleartext re-prompt on parse error), backed by a width-safe `parseBip85Index` (`strconv.ParseUint(…,10,64)`, rejects empty/non-digit/sign/`>2^31-1`, accepts leading zeros) AND an independent upper-bound guard inside `deriveBip85Child` (`index > 2^31-1` errors BEFORE the `uint32(index)+hardened` cast) — closing a silent uint32-truncation bug (an unguarded `≥2^31` derived an UNHARDENED off-spec child with no error on a 64-bit host). m\*-free, firmware-only, no lockstep; app stays BIP-39, output unchanged (words+SeedQR); high-index golden byte-exact at `2^31-1`, fuzz negative-control-proven. Spec/plan/exec-review all R0 GREEN — `design/agent-reports/seedhammer-bip85-custom-index-{spec-R0-round0,plan-R0-round0,exec-review}.md`. **(deferred scope) other BIP-85 applications:** only the BIP-39-words app (`39'`) is wired (engrave-as-words faithful); `32'` XPRV / WIF / hex / `128169'` seed apps need a different artifact — defer unless wanted. **(M-1, exec-review, functionally inert)** the dispatch `case bip85Derive:` sits after `case engraveMultisig:` (`gui/gui.go:1509-1511`) rather than the plan's literal "just before `case backupWallet:`" — Go switch cases are unordered + each `continue`s, so behavior is identical; cosmetic, no action.

- **`seedhammer-own-code-fix-followups`** — ✅ **CYCLE SHIPPED 2026-06-20** (fork `main` merge `39cb5cf`; CI Test + `tinygo-device-build` + Build-image all GREEN). All **8 confirmed findings** from the own-code adversarial bug hunt (`design/agent-reports/seedhammer-fork-own-code-bughunt.md`) fixed via the 2-track gated remediation (orchestration: `design/seedhammer-own-code-fix-orchestration-plan.md`; specs/plans/4×R0 + 2 per-track exec reviews + 1 combined exec review, all R0 GREEN 0C/0I — artifacts `design/agent-reports/seedhammer-{verify-cluster,scrub-batch}-{spec,plan}-R0-round0.md` + `…-exec-review.md` + `…-combined-exec-review.md`). **Track A (verify-correctness, merge `39cb5cf`):** H1 multisig verify now reads back the operator mk1 plate (was self-comparing → silent false-PASS; fix is compiler-enforced); H2 `md1Gatherer.collected()` index-ordered (was map-random → false-FAIL of a multi-chunk md1); M1 `Verify` compares ms1 codex32 language (was entropy-only → false-PASS of a non-English readback); L2 honest multisig verify copy; L1 scrub 2 verify-flow `DecodeMS1` probes. **Track B (secret-scrub, merge `2ea7754`):** M2 slip39 group-share scrub all paths + `wipe(d)`; M3 seedxor per-part entropy wipe; M4 `gui/bip85` `defer pkey.Zero()`; L1 codex32_polish probe scrub. The test-masking that hid these (synthetic-readback unit tests) is defeated by production-routed flow-level tests, each probe-proven fail-before/pass-after at every gate. **Residual (1, optional polish, LOW):** `extractSuppliedMd1AndMk1` (the new H1 helper, `gui/multisig_supply.go`) has unit coverage for all 6 accept/reject cases but no fuzz target, whereas its sibling `extractSuppliedMd1` is fuzzed (`gui/multisig_fuzz_test.go:49`) — add a fuzz target if/when convenient (non-blocking; the helper is a non-panicking switch). The L2-wording and combined-review brief-phrasing Minors were no-action.

### FOLLOWUPS nits burndown — RESOLVED 2026-06-19 (fork `main` `8eb51d7`)
Swept the accumulated non-blocking per-tier review Minors in one gated cleanup (branch `chore/followups-burndown`, 8 signed+DCO commits; consolidated whole-diff exec review GREEN 0C/0I — `design/agent-reports/seedhammer-followups-burndown-exec-review.md` — which RAN base-vs-branch byte-equality probes proving WalletPolicyId [incl. the pinned `6650b980…`], md1/mk1 encode output, and the mk1↔md1 stub binding are byte-identical; 9M+ fuzz execs clean). Resolved: **`seedhammer-t4-review-nits`** (all 3: path-notation display unified to `h`-form; `& 0x1f` chunk-count guard; `MKChecksumSymbols`→error); **`seedhammer-t6a1-walletpolicyid-origin-guard`** (`resolveOriginRaw` surfaces `errMissingExplicitOrigin`; identity bytes unchanged); **`seedhammer-t6a2-gui-nits`** (all 3: regenerated stale-csid `wpkhMK1` golden, dropped dead `derived` param, `showNotice`/`showError` split); **`seedhammer-t5-bundle-nits`** (M-1 compile-time last-navigable-program static assertion; M-2 was no-action); **`seedhammer-t6b-nits`** (M-1 reused-slot notice lists all slots, M-2 threaded decoded `tpl`/`keys`; M-3 was no-action). PARTIAL (see Open): `seedhammer-10a-encoder-nits` M-1 done (n-mismatch guard) / M-3 skipped (optional sort) / **M-2 pending = the pre-tag `tinygo build` gate**; `seedhammer-10b-walletpolicy-nits` M-1+M-2 comment fixes done / **M-3 `sh(wpkh)` projection = surfaced scope-decision**.

### `seedhammer-seedxor` — DONE 2026-06-18 (fork `main` `04a1e95`)
On-device **Coldcard Seed XOR combine**: a `SEED XOR` input-menu entry → pick N (2–5) → pick
part length (12/18/24) → enter N parts → XOR their BIP-39 entropy → recovered seed →
`backupWalletFlow`. New pure `seedxor` package (`Combine`, port of
`mnemonic_toolkit::seed_xor_combine`; no `math/big`), strictly N-of-N, Coldcard-interop lengths
only (load-bearing `{16,24,32}`-byte guard). Safety: a **mandatory "no built-in check"
fingerprint gate** (Seed XOR has no auth tag) + a per-part `isMnemonicComplete && Valid()` panic
guard; no interpretation fork (result is unambiguously a BIP-39 seed). `inputWordsFlow` gained an
additive `title` param (empty = unchanged). **Completes the on-device recovery suite: BIP-39 /
codex32 / SLIP-39 / Seed XOR.** Recon `cycle-prep-recon-seedxor.md`; architect consult +
spec R0→R1 + plan R0→R1 + whole-diff execution review all GREEN (vectors authenticated vs
Coldcard `docs/seed-xor.md` + `testing/test_seed_xor.py`); reviews in
`design/agent-reports/seedhammer-seedxor-*`. SPLIT remains out of scope (no on-device CSPRNG).

### `seedhammer-slip39-recovery-trezor-routing` — DONE 2026-06-18 (fork `main` `bc63caa`)
Shipped the two-way post-recovery fork (the rescope of `-verbatim-hex`; architect consult
`design/agent-reports/seedhammer-slip39-verbatim-hex-design-consult.md` found verbatim-hex is a
non-restorable artifact — won't-build). `engraveRecoveredSLIP39`'s one-way acknowledgement is now
a `ChoiceScreen`: **"BIP-39 seed"** (this toolkit / from a phrase → fingerprint check →
`backupWalletFlow`) vs **"Engrave shares"** (Trezor / other SLIP-39 wallet → `engraveSLIP39Verbatim`
on the share, **no** BIP-39 fingerprint). Removes the dead-end for non-constellation backups +
a README doc line. Gated: consult → plan R0→R1 GREEN → single-implementer TDD → whole-diff
execution review GREEN (0C/0I); reviews `design/agent-reports/seedhammer-slip39-trezor-routing-*`.

### `seedhammer-slip39-cycleC-all-lengths` — RESOLVED-BY-D2 2026-06-18
Cycle D Phase D1 widened `slip39.ParseShare` to accept all valid SLIP-39 share lengths
({20,23,27,30,33} words → {16,20,24,28,32} B; dropped `errUnsupportedSize`/`wordsShort`/
`wordsLong`), and Phase D2 added a **word-count picker** to the menu `case 3:` single-share
entry (`inputSLIP39Flow` gained a variable length). So the single-share verbatim entry+engrave
path now accepts all lengths, not just 20-word/128-bit — exactly this followup's ask. Shipped
on fork `main` `9db3fd2`.

### `seedhammer-slip39-recovery` (Cycle D) — DONE 2026-06-18 (fork `main` `9db3fd2`)
On-device SLIP-0039 secret recovery. **D1** (`f0092d5`): in-tree Go port of
`mnemonic_toolkit::slip39` — GF(256) field, Lagrange, 4-round Feistel decrypt, two-level
`Combine`, share-value extraction; no `math/big`; TDD vs official vectors + Rust-`split`-
generated intermediate-length fixtures. **D2** (`9db3fd2`): GUI recover flow — Recover button,
all-length entry, two-level roster + `selectForCombine`, optional SLIP-39 passphrase, the
entropy-interpretation hold-to-confirm + always-on fingerprint display, engrave via
`backupWalletFlow`. Full gated pipeline (spec R0→R1 + 4-lens architect panel; D1 plan R0→R2,
D2 plan R0→R1; both impl + whole-diff execution review GREEN 0C/0I). Reviews:
`design/agent-reports/seedhammer-slip39-recovery-*`. Two follow-ons filed above
(`-verbatim-hex`, `-hwsha`).

### `me-bundle-preview-sidecar` — Phase B DONE 2026-06-16 (v0.3.0)
Shipped the faithful host-side **plate preview** + the signed cross-platform release-CI. The `me-preview` (Go) sidecar (`preview/`) pins **UPSTREAM seedhammer v1.4.2** via a git submodule (`third_party/seedhammer` @ `713aee2`) and renders ONLY a validated public string → `engrave.Engraving` → SVG (optional `--png`):
- **B1 (sidecar/trust split) — DONE.** `preview/go.mod` imports `backup`+`engrave` directly (not `gui`); `seedhammer.com v0.0.0` sentinel + local `replace` (the `firmware/ndef-roundtrip/` pattern); not blocked on PR #35. The sidecar has no secrets and no network; `me` excludes ms1 from rendering.
- **B2 (faithfulness) — DONE.** Replicated `validateMdmk` layout: `backup.EngraveText`, QR via `qr.Encode(s, qr.L)`, `qrScale = 3`, modes TEXT+QR / TEXT / QR-only; replicated SH2 `engrave.Params` with a geometry-golden drift-guard; **exact cubic-Bézier SVG** (mirrors seedhammer's own `internal/golden` renderer — single `<path>`, B-spline G1 continuity preserved). Fidelity target = exact (not approximate).
- **B3 (delivery/version binding) — DONE.** `me bundle --preview <dir>` locates `me-preview` beside itself / on `$PATH`, checks `me-preview --version` against `CARGO_PKG_VERSION` (mismatch → exit 2, never a silent stale render), and degrades gracefully when absent (manifest + checklist still emitted, exit 0). `.github/workflows/release.yml` builds all targets (windows/arm64 omitted), assembles per-platform archives (`me` + `me-preview` + `minisign.pub` + `THIRD_PARTY_LICENSES` + verify note), and minisign-signs `SHA256SUMS`. A Rust↔Go cross-lang round-trip test (`crates/me-cli/tests/preview_cross_lang.rs`) builds the real sidecar and asserts one SVG per public plate, none for ms1.

`me` → **v0.3.0**. Spec `design/SPEC_me_bundle_phaseB_preview.md`; plan `design/IMPLEMENTATION_PLAN_me_bundle_phaseB_preview.md` (both R0/R1 GREEN). **Maintainer prerequisite — DONE:** the minisign keypair was generated (`minisign -G`); the public key is committed (`minisign.pub`, in README); the secret key + password are set as GitHub Secrets `MINISIGN_SECRET_KEY` / `MINISIGN_SECRET_KEY_PASSWORD` (never committed).

### `me-bundle-preview-layer` — Phase A DONE 2026-06-16
Shipped the pure-Rust **bundle orchestration core** (`me bundle`): reads newline-separated public md1/mk1 strings (stdin/`--in`), classifies + ms1-early-refuses, per-string pristine-validates, groups by `chunk_set_id`, and proves each chunk set complete/consistent (catches dropped/reordered/duplicate/foreign chunks via `mk_codec::decode` / `md_codec::chunk::reassemble`). Emits a JSON manifest (stdout/`--manifest`) + a guided per-plate checklist (stderr); refuses ms1 (exit 3). `me` → **v0.2.0**. Spec `design/SPEC_me_bundle_phaseA.md` (R0/R1 GREEN); plan `design/IMPLEMENTATION_PLAN_me_bundle_phaseA.md`. The faithful **preview sidecar** is split out as the new Phase-B `me-bundle-preview-sidecar` item (see Open) carrying `DESIGN_me_bundle_preview.md` §B (R0 findings I-3/I-4/m-5 + the upstream-v1.4.2 pin).

### Deferred formal subagent reviews — RESOLVED 2026-06-16
Both formal opus-architect **subagent** reviews deferred during the 2026-06-16 Agent-API outage (which had forced inline self-reviews) were run after agents recovered:
- **(a) PR2 (#35) final whole-diff review — DONE.** Caught 1 Important (md1/mk1 lowercase-only) + 3 Minor the inline self-review missed; folded in seedhammer `6ab12c0` (PR #35 updated), R1 **GREEN** (`design/agent-reports/firmware-pr2-mdmk-final-review-R{0,1}.md`).
- **(b) converter-polish diff (`5086119`) review — DONE.** R0 caught 1 Important (I-1: with `--echo`, the input was copied into an un-zeroized heap `String` *before* `convert()`, so `--echo --in <ms1-file>` left the secret un-scrubbed on the ms1-refusal path — defeating nit 4's defense-in-depth) + 1 Nit (N-1: echo test lacked a stdout-purity assertion). Folded: `echo_line` now built only when `cli.echo && result.is_ok()` and wrapped in `Zeroizing<String>`; echo test now asserts stdout stays binary-only. R1 **GREEN** (`design/agent-reports/me-converter-polish-final-review-R{0,1}.md`).

### Converter (`me`) polish cycle — RESOLVED 2026-06-16 (commit `5086119`)
All five nits from the converter execution review (`design/agent-reports/me-converter-execution-review.md`) were cleared in one PATCH cycle (spec `design/SPEC_me_converter_polish.md`, plan `design/IMPLEMENTATION_PLAN_me_converter_polish.md`):

- **`me-in-stdin-intermediate-zeroize`** — input now read into a `Zeroizing<String>`, scrubbed on drop (`main.rs`).
- **`me-validate-ms-unreachable`** — `panic!` → `unreachable!("ms1 is refused before validation")` (`validate.rs`).
- **`me-decode-text-tlv-comment`** — `decode_text_tlv` now documents its intentional 1-byte-TLV / no-terminator-check scope (`ndef.rs`).
- **`me-canonical-string-stderr`** — reconciled via an opt-in `--echo` flag (prints the validated string to stderr on success); spec §5 amended to match (`main.rs`, `cli.rs`, `SPEC_seedhammer_engrave.md`).
- **`me-go-harness-shortread-loop`** — the harness now reads the NDEF record in a short-read loop (`firmware/ndef-roundtrip/main.go`).

### crates.io publish — RESOLVED 2026-06-16
- **`me-crates-io-publish`** — **`mnemonic-engrave` v0.1.0 published** to crates.io (<https://crates.io/crates/mnemonic-engrave>; `cargo install mnemonic-engrave` → the `me` binary). Added publish metadata (`repository`/`homepage`/`keywords`/`categories`) + a crate-local `README.md` (`9ad758c`); dry-run green; uploaded with a `publish-new`-scoped token. Future versions: bump `version` and `cargo publish` (needs `publish-update` scope).

## Reconciliation — 2026-08-11, the post-release Phase 2 review round

Six independent reviews ran over the released tree (`me` `v0.5.0` / fork
`93ee004`), each persisting its own report to `design/agent-reports/` and
committed verbatim before any fold. Reviewers were briefed to **suggest**
closures, not perform them. This section records what the round found and what
was done about it; per-entry edits are noted against their F-numbers.

**Reports:** `phase2-whole-diff-fable.md` · `f109-residue-identification.md` ·
`followup-triage-suggestions.md` · `phase2-claims-audit.md` ·
`phase2-rust-side-review.md` · `phase2-spec-conformance.md`

### What the round changed

- **C1 — `me seal` was emitting backups the device cannot open.** FIXED
  (`ad8f95f`). A BIP-39 mnemonic record was *accepted* on its whitespace-
  normalised form and *emitted* as supplied; the device splits on a single ASCII
  space. A double space, TAB, NBSP, VTAB, ideographic space or newline all
  sealed with exit 0 and were refused on the machine after the ~31 s KDF, shown
  as §6.4 "payload unreadable" — which §2.2 item 4 teaches the operator to read
  as **tampering**. New `RecordError::NonCanonicalSpace`, mutation-killed.
- **F-114 — CLOSED, not a defect.** The machine homes to the plate origin before
  every run, so the approach line from `bezier.Point{}` is correct. See its
  entry; pinned by fork `d55c06b`.
- **F-121 — FILED** out of that closure: the emulator does not home, so it
  renders a resumed cut the machine would never produce.
- **F-110 — status corrected.** It was never closed; the error was in
  `CONTINUITY_2026-08-11.md` and in the brief handed to the triage reviewer,
  which refuted it from the shipped code's own comments. Now **overdue** and
  re-assigned to post-merge polish and hardening. Narrowed by fable's M3: the
  wipe provably cannot fire mid-cut, so one of its bullets describes an
  unreachable path.
- **F-68 — closure de-attributed.** Credited to `plan-cite-gate.sh`, which
  resolves citations; F-68 is that `plan-build-gate.sh` never *runs* the CLI
  tests, and `:163` still passes `--no-run`.
- **Released binaries misreported their own version.** `v0.5.0` was tagged with
  `Cargo.toml` at `0.4.0`, so every published archive prints `me 0.4.0`. Bumped
  (`00adcf3`). **The published v0.5.0 assets are not retroactively fixed** —
  whether to re-tag is an operator call.
- **`bip39`'s `zeroize` feature was off**, so `Mnemonic` was not
  `ZeroizeOnDrop` — a parsed passphrase dropped in the clear, undercutting the
  `Zeroizing` wrappers around the same words. FIXED (`24dff51`).
- **Spec citations now name symbols where lines keep decaying.** `idleTimeout`
  had moved 2801 → 2879 → 2932 → 2955.

### F-109 — DOWNGRADE recommended, and the security question is answered

The residue was measured for the first time. **No secret was found in any of
it**, against controls proving the search detects a secret when present: each
canary scored at its own live instant and **zero at every post-wipe dump**,
including the `[]bip39.Word` passphrase buffer the harness still referenced.

23,024 B is now named and benign — a write-only display mask (12,480 B), the
§10.2.4 warning frame's arg/ref buffers (9,472 B, decoded: compile-time
constant text and package-level singletons over `//go:embed` flash, **zero**
references to any session object), and the drawer's stacks (1,040 B). A further
~13.5 K was never residue: `heapLine()` rides `StartScreen.Version`, built
*before* the first frame, so the baseline and the later readings were not
comparable. **~12 K across ~74 small objects remains unnamed.**

**Caveat that keeps this open rather than closed:** measured on host Go 1.26.3,
not the device's TinyGo build, whose `-gc precise` scans stacks conservatively
and can retain what host Go frees. That can add bytes; it cannot un-zero a
cleared buffer.

**Suggested:** downgrade to Minor, fix the probe placement rather than the
memory, and file the genuinely open question this surfaced — whether TinyGo's
non-releasing `sync.Pool` retains a `fmt`-formatted copy of seed material on the
**cut** path. That is F-88 territory, not F-109's.

### F-120 — the ledger UNDERSTATES it, with a measured table

It is not a boundary case at 90. The device admits **27** codex32 lengths in
48–90; `me` admits **10**; **22 diverge**. The reverse set is **empty**, so
unlike C1 this cannot produce an unopenable backup — every `ms1` that `me`
emits, the device accepts.

The entry's `[50,56,62,69,75] ∪ [51,58,64,70,77]` is misleading: those are two
**disjoint tag families** (`entr` v0.1 vs `mnem` v0.2), and an `entr`-tagged
77-character string is refused while a `mnem`-tagged one at the same length is
admitted. Also, the "widen `me`" option is not actionable from this repo — the
narrowing lives in `ms-codec`.

### Still open from this round, not yet folded

- **Rust, Important** — the secret record is freed **unscrubbed**:
  `Payload.secret` is a plain `Vec<String>`, which undoes part of F-102's fix
  one line later. Proven with a probing allocator and both controls.
- **Rust, Important** — `normalise` leaks 3 blocks per call despite returning
  `Zeroizing`.
- **Rust, Important** — the **mk1 pristine-BCH guard has no test**; the mutation
  survived. It is load-bearing for cross-implementation agreement, and the
  device was confirmed to refuse a BCH-correctable `mk1`.
- **F-115** — measured blind spot: **68 of 175** file:line-shaped citations in
  this file use range/comma/slash forms `plan-cite-gate.sh`'s regex never
  attempts. The gate's silence over this file is therefore not coverage.
- **F-83** — two copies named by no prior inventory: `stepper.Driver.buf` plus
  the bezier/bspline motion state (the seed as PIO step words during a cut), and
  the SH2 LCD DMA chunk buffers (rendered-seed pixels). Both F-83-class,
  confined to the cut window. Record them here rather than re-deriving later.
- **F-105** — fable's M4: F-103 defeats **row 4** (the passphrase wipe) exactly
  as it defeats row 1, so F-105's "CLOSED — hardware" holds only on a machine
  whose touch panel is quiet. Cross-reference, not a re-opening.

### Triage's closure candidates — evidence recorded, operator decides

Whole entries: **F-75, F-60, F-63, F-72, F-82, F-71**. Bullets: **F-80's
`HasHash`** (guard now exists, `gui/unlock_flow.go:91`), **F-80's `groupCards`**
(doc states it is test-facing, `seal/record.go:429-431`, still 0 production
callers), **F-90 item 2** (subsumed by F-89's rename to `RecordsResident`).

Now **due** rather than closeable, both gating conditions met: **F-65**, **F-66**,
**F-76** — each waited on "after the cycle ships", and it has.

### Handed back by the seed-residue implementer, 2026-08-11

- **F-104 item 2b (new sub-item)** — `LastWordCandidates`'s
  `m := make(Mnemonic, len(prefix))` is a one-line zeroable copy of the
  operator's 11-word prefix, left live. F-104 item 2 names `entBytes`, not this,
  so the implementer correctly declined to widen a Rust-primary package's diff
  on its own initiative. Same class, same fix shape, not yet done.
- **`gofmt -l` is not empty at HEAD** — six files were already unformatted
  before this phase: `gui/bip85_test.go`, `gui/md1_expand_fuzz_test.go`,
  `gui/multisig_build_test.go`, `gui/multisig_match.go`,
  `gui/multisig_testhelpers_test.go`, `md/template_guard_test.go`. Verified by
  the controller against the tree (the implementer reported five; the sixth is
  `md/`, outside the directories it was scoped to). None was introduced by this
  phase. Not fixed immediately only because two implementers were still holding
  `gui/` worktrees; do it once they merge. **Consequence worth naming:** a
  dirty `gofmt` baseline means "gofmt is clean" cannot be used as a gate by any
  future agent, so every one of them has to special-case it.

### Post-merge polish and hardening — closures, 2026-08-11

Fork `6fb8442`. Verified by the controller on the MERGED result each time, not
on the implementer's report: `CGO_ENABLED=0 go test ./...` exit 0, 49 ok, 0 FAIL.

| item | outcome |
| --- | --- |
| **F-78** | CLOSED — but **not by the remedy the entry assumed**. `font/bitmap.go`'s binary index caps lookup at ASCII 127, so adding a U+00B7 glyph would mean widening a shared format and regenerating all 8 `.bin` faces. Substituted `|` (B1's precedent) at **five** occurrences; the entry named three. |
| **F-86** | CLOSED — `%` *is* inside the ASCII index range, so this was the small change F-78 ruled out. Only `boldprogress45.bin` regenerated, +629 B. |
| **F-95** | CLOSED — by shortening the copy, not by adding touch-scroll. Two blank lines removed, zero words changed; `Warning.Layout`'s own `maxScroll` moves +19 → −17. |
| **F-119** | CLOSED — comment-only, as anticipated. Fallback order **measured** (TEXT+QR fails at 269 chars, QR-ONLY at 642, TEXT-ONLY last at 646); the code was right. |
| **F-94** | CLOSED — the 64-byte BIP-39 seed and the BIP-32 master key are pinned via `deriveSeedHook` / `deriveMasterKeyHook`. |
| **F-87** | CLOSED — `unlockEngraveMnemonic`'s third early return pinned. |
| **F-104 item 2** | CLOSED — the discarded entropy copies in `bip39` are zeroed. **Item 2b remains open** (see the handback above). |

**The finding worth keeping from this batch:** fixing F-86 **broke three
unrelated tests** whose regexes had been implicitly relying on `%` rendering as
zero pixels. They passed for the wrong reason for exactly as long as the bug
existed, and no amount of reading would have shown it — only the repair could.
That is the false-PASS class this project otherwise finds by mutation, surfaced
here by a real fix. It is an argument for fixing cosmetic defects rather than
carrying them: a bug in the render is also a load-bearing assumption in the
tests around it.

### F-122 — a flickering touch panel still produces GENUINE edges, so the wipe can still be delayed (owning phase: **post-merge polish and hardening**)

Filed 2026-08-11, out of F-103's fix. **This is the part of F-103 that was
narrowed rather than closed, and it is filed rather than folded into that entry
so it cannot be lost behind a CLOSED heading.**

F-103's fix keys the §10.2.4 idle clock on *effective* input — a contact-state
change on the pointer, a rune, or a button — instead of raw event presence. That
defeats the measured failure: 100,000 position-only spurious polls now leave the
clock alone, and the regression test is committed.

**What it does not defeat.** A panel whose contact repeatedly crosses the
detection threshold — film, moisture, or debris causing the driver to assert and
de-assert contact — produces real down/up **edges**. Those are indistinguishable
at this layer from an operator tapping, so they legitimately reset the clock. A
seed can still be held resident indefinitely by a panel that flickers rather
than one that merely reports positions.

**Why it was not fixed in the same change.** The remedy is a plausibility bound
— rate-limiting or debouncing edges that no human could produce. That is a
**tunable constant inside a funds-safety control**, and picking it blind trades a
known failure for an unknown one: too tight and a slow deliberate operator gets
wiped mid-task, which is worse than the defect. The missing input is a **bench
capture of the real `ft6x36` stream under a protective film** — the same free
bench check the B2b preflight recorded and never ran. Get the capture, then pick
the bound from it.

**Operator guidance stands regardless, and is the real mitigation:** with the
film on, the panel is unusable as *input* anyway. Take it off. The fix only
ensures the machine wipes rather than holding a seed forever.

### F-123 — the documentation implies the wiping class is meaningfully safer than it is (owning phase: **systemwide payloads**)

Filed 2026-08-11, out of the systemwide-payloads brainstorm. **Operator ruling:
Sealed Payload is frozen; its documentation is not.**

§2.2 item 12 already draws the line correctly — "the machine offers two classes
of program, and only one of them wipes" — and tells the operator to "use Sealed
Payload for anything you intend to protect." That advice was written when the
wipe was believed to work. The operator's own assessment 2026-08-11 is that the
sealed-payload program **tries to be secure and fails**: the wipe is incomplete
by explicit prior decision (§2.2 item 16), F-110 names both halves as open holes
in the shipped code's own comments, and F-88/F-90/F-104 item 2b are the
remaining unwipeable-garbage class.

**What is wrong is not the ruling but the inference an operator draws from it.**
Someone who reads "only one of them wipes" concludes the wiping one is the safe
one. What actually protects them is **physical custody** — the device is
deliberately debuggable, SWD readable, BOOTSEL enabled. A wipe that runs and
leaves residue is not a weaker version of protection; it is a different thing
from what the sentence implies.

**Fix:** README's security section, §2.2 item 12's operator-facing wording, and
`SPEC_systemwide_payloads.md` §11 must all say the same thing — the two classes
differ in *behaviour*, not in whether your funds are safe if the machine is
taken. Do not soften §2.2 item 12; qualify what it means.

Blocks nothing. Must land before any operator journey documents this, because a
journey that repeats the current wording propagates the inference.

### F-124 — remedy Sealed Payload's security failures (owning phase: **deferred, a future cycle — operator ruling 2026-08-11**)

Filed 2026-08-11 alongside F-123. **Deliberately deferred, not forgotten.**

The operator's ruling for the systemwide-payloads cycle was "keep it, file a doc
fix, we will attempt to remedy the security failures at a future date." This
entry exists so that date has something to be scheduled against, and so the
decision reads as a deferral rather than as an oversight to a future reviewer.

**The known open items, all already filed:**

- **F-110** — overdue; both halves named as open holes by the shipped code
  (`gui/engraver.go:126-132`, `engrave/engrave.go:1722-1730`, the latter with a
  measurement: 4 orphaned arrays → 23 arrays / 119,891 knots).
- **F-88 / F-90 / F-104 item 2b** — the remaining unwipeable-garbage class, plus
  the un-inventoried sibling copy in `LastWordCandidates`.
- **F-109** — downgraded to Minor, but on **host Go, not TinyGo**, which is why
  it was downgraded rather than closed.
- **§2.2 item 16** — the wipe is incomplete by explicit prior decision. Revisiting
  that decision is part of this item, not a precondition for it.

**What this entry must NOT become:** a reason to hold the systemwide-payloads
cycle. Those programs were always in the non-wiping class (§2.2 item 12); they
inherit no debt from this. The two are independent.

### F-125 — the restored user-supplied passphrase mode requires amendments to EPD and passphrase.rs, and they are unscheduled (owning phase: **systemwide payloads, before implementation**)

Filed 2026-08-11 out of R0 rounds 3 and 4 on `SPEC_systemwide_payloads.md`,
which flagged it twice as NOT FIXED. Recording it so the collision is scheduled
rather than argued about later.

`SPEC_systemwide_payloads.md` decision 8 restores the **user-supplied**
passphrase mode. Two normative statements elsewhere say it must not exist:

- **`SPEC_encrypted_payload_delivery.md` §2.2 item 1 and §8** forbid it.
- **`crates/me-cli/src/seal/passphrase.rs`**'s module doc: *"GENERATED, never
  user-supplied… a human-chosen passphrase is worth 25–35 bits — one rented
  GPU, minutes. `age` reached the same conclusion and generates 10 words rather
  than letting the user pick."*

The new spec marks the overrule where it occurs, which §1 requires. **What it
does not do is amend the documents being overruled**, and a normative MUST NOT
left standing unqualified is exactly the kind of stale record this project keeps
being bitten by — the next reader has two documents that disagree and no marker
saying which is current.

**Not a design question.** The operator ruled; this is bookkeeping with an
owning phase. Two edits:

1. EPD §2.2 item 1 / §8 gain a qualification: the prohibition holds for the
   **Sealed Payload** program, and `SPEC_systemwide_payloads.md` §8 governs the
   systemwide container.
2. `passphrase.rs`'s module doc gains the same, without weakening its argument —
   the 25–35 bit figure is still correct and is precisely why `[cliff]` places
   every user-supplied passphrase below the threshold.

**Must land before implementation**, not after: the first person to read
`passphrase.rs` while building this will otherwise find a module doc telling
them the mode they are implementing does not exist.

### F-126 — CLOSED 2026-08-12 by plan stage 10 — presenting an NFC tag to a gathering flow FREEZES the emulator, so the path stage 6 exists to open cannot be walked (owning phase: **systemwide payloads**) `#mnemonic`

**CLOSED 2026-08-12** by plan stage 10, and by fix 2 as recommended — the loop
shape, not the reader. The five duplicated scan loops are now one
`startScanner` (`gui/nfc_scan.go:45`) with a backoff keyed on **idle**, which is
the EOF case the old guard missed: `scanFailed` was never the condition that
mattered. 253 lines of duplication went with it. Measured by the implementer:
4 reads per 150 ms against ~198,000 iterations before.

Verified here rather than taken on report — `startScanner` has six non-test
callers, covering every original site. Two `scan.Status == scanFailed` lines
survive: one INSIDE `startScanner`, which is the point of centralising it, and
one at `derive_xpub.go:230` which is CONSUMER-side message selection inside a
`select` with a `default`. Neither can spin.

Original analysis follows.


Filed 2026-08-11 while building the operator-journey document, which tried to
deliver the 25-card bundle over NFC and hung the browser instead.

**Mechanism, in two halves that are individually reasonable.**

`gui/bundle_flow.go` samples `ctx.Platform.NFCReader()` **once**, on flow entry.
A tag presented afterwards is invisible: the scanner goroutine was never started.
That much is only awkward — present the tag first and it reads.

The defect is what happens when a tag **is** pending. `cmd/emu/nfc.go`'s source is
deliberately one-shot ("a real tag crosses the reader once"), so the moment its
single record is consumed the reader sits permanently at EOF. The scan loop:

```go
obj, err := s.Scan(r)
...
case err == nil || err == io.EOF:      // Status stays 0
...
scans <- scan
wakeup()
if scan.Status == scanFailed {
    time.Sleep(1 * time.Second)        // the ONLY yield in the loop
}
```

`io.EOF` does not take the `scanFailed` branch, so nothing sleeps and nothing
blocks. Under Go/wasm — cooperatively scheduled on the browser's single thread —
a goroutine that never blocks starves the JS event loop outright.

**Observed:** after `shNFC.present(<md1>)` and entering Engrave Bundle, the page
stopped responding entirely; even navigating the tab away timed out at 60 s and
the tab had to be destroyed. Entering the same flow with **no** tag pending is
fine, because `NFCReader()` returns nil and no goroutine runs — which is exactly
the discriminator that identifies the loop as the cause.

**Why it matters beyond the emulator.** SPEC_systemwide_payloads §8.2 added the
emulator NFC source specifically so the eight programs' new secret-delivery path
would be walkable. Today that path is the one path the tool cannot walk. Any
qualification of NFC screens done in the emulator has been done blind.

**Two separable fixes; the second is the real one.**

1. `nfcSource.reader()` should return a reader that **blocks** rather than
   reporting EOF once drained — a real reader waiting on a tag does not spin.
2. The scan loop should yield on **every** iteration, not only on `scanFailed`.
   A driver that returns instantly is not a hypothetical, and the loop is
   currently only safe because the hardware reader happens to be slow.

Fix 2 is device code and is where the latent assumption lives; fix 1 alone would
paper over it.

**All five scan loops share the defect — measured, not assumed.** Every one of
`bundle_flow.go`, `md1_gather.go`, `mk1_inspect.go`, `verify_address.go` and
`gui.go:1814` guards exactly one `time.Sleep` behind `scan.Status == scanFailed`
and has no other yield:

```
$ for f in gui/{bundle_flow,md1_gather,mk1_inspect,verify_address,gui}.go; do
    grep -A2 'scan.Status == scanFailed' $f | grep -c time.Sleep; done
1 1 1 1 1
```

So the fix belongs in the loop shape, in all five, rather than in whichever one
the emulator happened to expose first.

**Also worth noting for the journey work:** even once unfrozen, one flow entry
consumes exactly one tag, so a 25-card bundle cannot be delivered over the
emulator's NFC source as it stands.

### F-127 — `mk encode --from-md1` cannot read a CHUNKED md1 (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-11 building the pathological-wallet journey
(`design/journeys/SeedHammer-II-pathological-wallet-journey.pdf`).

Every mk1 key card must carry a 4-byte policy-id stub; `mk encode` refuses
without one. The only automatic way to obtain it is `--from-md1`. Given a chunk
of a chunked md1 set it fails:

```
$ mk encode --xpub xpub6CatWdiZiodmU… --origin-fingerprint 73c5da0a \
    --origin-path "m/84'/0'/0'" --from-md1 md1fqgpcpqpz3m6jzz… --group-size 0
error: md1 input rejected: wire-format version mismatch: got 9, expected 4
[exit 2]
```

**Cause is a stale vendored copy.** `mnemonic-key/vendor/md-codec` is at
**0.34.0**; `descriptor-mnemonic/crates/md-codec` is at **0.42.0**. Version 9 is
the chunked wire form the vendored copy predates. This is precisely what the
provenance pin exists to catch, and it did not.

**Scope of the blast radius.** Any policy over the single-string cap. The
journey's wallet — 11 keys, four timelock kinds, two 32-byte hash literals —
comes to 182 data symbols against a cap of 80, so it is not an exotic corner.

**Workaround, used in the journey:** read the identity out of `md inspect` and
pass `--policy-id-stub` by hand. That requires knowing F-128, and nothing tells
an operator either thing.

**DOWNGRADED 2026-08-11 by the adversarial pass.** As filed this said a chunked
policy has "no documented route to a key card". That clause is false and the
refuter broke it: `--from-md1` accepts only a *single, unchunked* md1, but the
stub remains derivable — which is exactly what this journey did, successfully.
So this is **ergonomics, not a binding failure**: severity Important → **Minor**.
What stays true, and is the part worth fixing, is that the tool gives the
operator a hard parse error and no hint that `--policy-id-stub` plus
`md inspect` is the way through.

### F-128 — the stub's spec sentence and `mk`'s behaviour name different identities (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-11, same run as F-127.

`SPEC_mk_v0_1.md` §3.3: *"Each stub is 4 bytes = the top 4 bytes of the
MD-encoded policy's **WalletPolicyId** — `md_codec::compute_wallet_policy_id`"*,
and today's `md inspect` prints a field by exactly that name. **`mk` does not use
it.** Measured on a single-string wallet where `--from-md1` works, so `mk`
derived the stub itself:

```
wallet-descriptor-template-id: 726a666305756435b7c52c5b3fc69c41
wallet-policy-id:              f05e8a1c282f7740bbfd902a759b5577
policy_id_stubs (mk decode):   726a6663
```

The stub tracks the **template-id**. Most likely a rename that landed in md-codec
after 0.34.0 — which would make this F-127's twin rather than an independent bug
— but as it stands the spec sentence and the binary disagree about which identity
a key card indexes, and the stub is what a recovering operator uses to tell one
wallet's cards from another's.

**Resolve in this order:** decide which identity is correct, fix whichever of the
two is wrong, then bump the vendored pin. Do not bump the pin first: if the
rename is real, bumping silently changes the stub every existing key card carries.

### F-129 — `--path` is mandatory for a non-canonical wrapper and flattens divergent origins; which source wins on restore is unpinned (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-11, same run. **A design question, not a defect.**

A `wsh(or_i(…))` wrapper has no canonical default derivation path. Without
`--path`, `md encode` warns, `md decode` PARTIAL-decodes (exit 4 VERIFY-ME) and
`me bundle` refuses the set — all correct and all clearly signposted. With
`--path`, everything validates.

But `--path` takes a **single shared** path and its own help says it "flattens
Divergent mode to Shared". The journey's eleven keys sit at four account indices
(`84'/0'/0..3'`) across three masters, so the shared value is true for @0–@2 and
false for @3–@10. The true per-key origin survives on each mk1 card
(`origin_path` decodes correctly), so the bundle as a whole is not lossy — but
the descriptor card alone would restore eight of eleven keys wrongly.

**What is missing is a test pinning precedence:** when the md1's shared origin and
an mk1 card's `origin_path` disagree, the card must win. Add a restore vector for
exactly that disagreement before this shape is recommended to anyone.

#### F-129 — ANSWERED 2026-08-11 by running the round trip `#mnemonic`

The precedence question is settled, and in the safe direction: **the mk1 cards
win; the md1's flattened `--path` never overrides them.** Proven twice by
refusal, not by inspection —

- supplying a slot as a bare xpub instead of its card: *"non-canonical wrapper
  requires explicit origin for @2, but none provided"* (exit 1);
- supplying no cards at all: *"cannot infer the own origin family (no canonical
  origin, no cosigner mk1, and no --origin)"* (exit 1). `--origin` is single-sig
  only, so for a multisig the card is the ONLY origin source.

A full restore (3 md1 chunks + 10 cards + the seed for the own slot) exits 0 and
reproduces all 11 origins exactly, including the divergent account indices the
descriptor card flattened to one value.

**The residual risk moved to the OWN slot**, the one `--from` fills rather than a
card. That key IS derived at the flattened shared path. With the own slot at @0
(true path `84'/0'/0'`) it is right. Asking for the own slot at @3 (true path
`84'/0'/3'`) derived @0's key instead — and was caught:

```
error: restore: multisig-template-floor mismatch — derived duplicate cosigner
keys: supplied keys at positions 0 and 3 are identical
```

**The guard is real but incidental.** It fires because the mis-derived key
collides with a slot that was supplied. In this wallet every master happens to
have an account-0 slot among the eleven, so every possible mis-derivation
collides and is caught. A wallet where some master's account-0 key is NOT one of
the slots would mis-derive the own slot into a key nobody supplied — no
duplicate, no error, **a silently wrong wallet**.

**So the remaining work is narrow:** the own slot should take its path from its
own mk1 card (or an explicit per-slot origin) rather than from the template's
shared path, and the duplicate-key check should not be the thing standing between
an operator and a wrong wallet. Keep the check; stop depending on it.

**Separately, the descriptor text does NOT round-trip byte-for-byte** — see
F-130. That is a different problem and does not affect the above.

### F-130 — restored xpubs lose their BIP-32 depth/parent/child, so the descriptor and its checksum change (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-11 from the same round trip.

The recovered wallet is the right wallet: for all 11 slots the chain code and
public key are **byte-identical** to the original, and the policy template
decodes byte-identical from the three md1 chunks. Child derivation uses only
those, so addresses are unaffected.

What changes is the xpub *serialization*. Decoded headers, original vs restored:

| | depth | parent fp | child |
| --- | --- | --- | --- |
| original @0…@10 | 3 | `7ef32bdb` / `ea517ee5` / `d061c20c` | `0h`…`3h` |
| restored @0…@10 | **0** | **`00000000`** | **`0`** |

An mk1 card stores the account key's chain code and public key plus the origin
(fingerprint + path); it does not carry depth/parent/child, so the reconstructed
xpub is serialized with them zeroed while still annotated `[fp/84'/0'/N']`. The
origin says depth 3; the key says depth 0.

**Two concrete consequences, both measured:**

- The descriptor checksum changes — `#4ld0crxa` → `#jgulue7j`. An operator who
  recorded the checksum (the obvious thing to record) sees a mismatch on a
  correct restore.
- Tools in this constellation enforce depth. `md address` refuses the restored
  key outright: *"--key @0: expected depth 4 for this script context, got 0"* —
  the same check that rejects the ORIGINAL depth-3 keys for this wallet shape,
  which is worth noting on its own.

**Decide which is true** before changing anything: either the mk1 wire format
should carry depth/parent/child (costing bytes on every key card), or a restored
descriptor is defined as equivalent-not-identical and the checksum is documented
as not comparable across a round trip. Today neither is written down, so the
first operator to check a checksum after recovery will think the backup failed.

### F-131 — the engraving checklist tells the operator a recovery rule that is false in BOTH directions (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-11 from the miniscript-nesting review of the pathological wallet.
Verified by running it, not by reading the report that raised it.

```
$ mnemonic bundle --network mainnet --descriptor-file .examples-build/degrade2.desc
# Threshold: 3 of 11
# Recovery: any 3 of 11 signing keys + md1 (template card).
```

The wallet is not a 3-of-11. It is a four-tier degrading policy with **eight**
distinct minimal key-sets, each carrying its own timelock, and two of them also
requiring a hash preimage:

| tier | key-sets | also needs |
| --- | --- | --- |
| 1 | `{@0,@1,@2}` | preimage + absolute HEIGHT ≥ 1000000 |
| 2 | `{@3,@4}` `{@3,@5}` `{@4,@5}` | preimage + absolute TIME ≥ 1893456000 |
| 3 | `{@6,@7}` | relative 65535 BLOCKS |
| 4 | `{@8}` `{@9}` `{@10}` | relative TIME (~365 d) |

The printed line is wrong **both ways**, which is what makes it dangerous rather
than merely imprecise:

- it OVERSTATES — `{@8,@9,@10}` is three of the eleven keys and cannot spend
  together at all before the tier-4 timelock, and no 3-key set spends tier 1
  without the preimage;
- it UNDERSTATES — `{@8}` alone spends after ~365 days, so the wallet is a
  1-of-3 to an attacker who waits, not a 3-of-11.

An operator sizing their key custody off that line gets the threat model
backwards. **This is engraved-adjacent output**: it is the checklist a person
follows while cutting permanent plates.

Fix is not "reword": the summariser is computing a threshold for a shape that
does not have one. It must either enumerate the key-sets (compare-cost already
does) or refuse to print a threshold for a non-threshold policy.

### F-132 — the hashlock preimage is required to spend, absent from the backup, and unmentioned by it (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-11, same review.

Tiers 1 and 2 are `and_v(v:sha256(H), …)`. Spending either requires revealing
the 32-byte preimage `X` where `H = sha256(X)`. In the worked example `X =
sha256("opensessame")`. Measured against the engraved set:

```
preimage X present in any backup string : 0
the word "opensessame" present anywhere : 0
```

Correct — the descriptor commits to `H`, and `H` is what the md1 carries. But
nothing in the bundle, the checklist, or the plate set records that a secret the
operator must supply from memory stands between them and **five of the eight**
key-sets. Lose the word and tiers 1 and 2 are gone; what remains is tier 3
(`{@6,@7}`, 455 days) and tier 4 (any one of `{@8,@9,@10}`, 365 days).

This is not the codec's bug — a preimage is deliberately not key material and
arguably should not be engraved next to the policy. The defect is **silence**.
The bundle should state that the policy contains a hashlock, name which branches
it gates, and say that the preimage is not in the backup. A backup that omits a
required factor without saying so is the failure mode this whole project exists
to prevent.

### F-133 — the relative tiers are INVERTED: the weakest key-set matures ~90 days before the stronger one (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-11, same review. Arithmetic verified independently from BIP-68's
field layout.

| tier | key-set | lock | decoded | matures |
| --- | --- | --- | --- | --- |
| 3 | `{@6,@7}` — 2-of-2 | `older(65535)` | bit22 clear → blocks | 65535 blocks ≈ **455 days** |
| 4 | `{@8}`/`{@9}`/`{@10}` — **1**-of-3 | `older(4255898)` | `0x40F09A`, bit22 set → 61594 × 512 s | **365.00 days** |

(The ~90 days is nominal at the 600 s block target: 455.10 − 365.00 = 90.10 d.
The block-count side drifts with real hashrate; the time side does not. The
ordering does not depend on that drift — tier 3 would have to run ~20% fast to
catch up.)

A degrading vault is supposed to degrade *monotonically*: each tier that
activates should be weaker than the last, and later. Here the **1-of-3** tier
opens ~90 days **before** the **2-of-2** tier. From day 365 the wallet is a
1-of-3; tier 3 never becomes the operative security floor, because by the time
it activates a strictly weaker path has been open for three months.

Not a consensus or standardness problem — the script is valid and
rust-miniscript is silent, correctly, because ordering between disjoint branches
is not something it models. It is a **policy design defect in the example**, and
the example is the one the documentation calls "the pathological example" and
that this project has now engraved into a journey document.

**It is upstream, and the upstream document states both numbers without noticing.**
`mnemonic-toolkit` Examples §5 lists the tiers in ascending order as a degrading
vault, and then, a few lines below, spells out the two durations adjacently:

> - `older(65535)` -- relative **blocks**: 65,535 blocks (~455 days). …
> - `older(4255898)` -- relative **time**: … 61,594 units x 512 s ~= 365 days.

455 then 365, printed one after the other under a table that presents tier 3
before tier 4. So the defect is in the source example, not in our transcription
of it. Either swap the two locks or say the inversion is deliberate — and fix it
where it is authored, in `mnemonic-toolkit`, so the Rust-primary direction holds.

### F-134 — plate count for one wallet ranges 26 → 58 depending on an md1-form flag nobody is told about (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-11, same review. All three counts measured.

| route | md1 | mk1 | plates |
| --- | --- | --- | --- |
| `md encode --force-chunked --path bip84` (keyless template) + `me bundle` | 3 | 22 | **26** |
| `mnemonic bundle --descriptor-file --md1-form=template` | 4 | 33 | 38 |
| `mnemonic bundle --descriptor-file` (default, `--md1-form=policy`) | 24 | 33 | **58** |

Same wallet, same keys, a 2.2× spread in permanent physical plates. The default
is the most expensive one, because policy-form md1 embeds the keys rather than
referencing them — defensible, since it makes the descriptor card
self-sufficient, but it is a large cost chosen silently.

Nothing in the tool tells the operator the trade: self-sufficient descriptor card
vs less than half the steel. **Print the comparison before engraving**, the way
`me bundle` already prints the plate checklist.

(Note the first row differs from what the review reported. It measured that path
as REFUSED, which was true of the state it inspected; `--path bip84` — the fix
recorded in F-129 — makes it validate and produce 26. The 38 and 58 figures are
unaffected.)

**SCOPED 2026-08-11 by the adversarial pass.** A refuter showed 25/26 also
reproduces at exit 0 on the *default keyed* path, so "only reachable via a
refused route" would have been wrong — the three counts above are each real and
each reachable. The finding is therefore about **an unadvertised 2.2× cost
spread**, not about one route being broken. A further variant was measured at 23
md1 → 24 plates under a different origin choice (`m/48'/0'/0'/2'`), which widens
the spread rather than changing its shape.

### F-135 — CLOSED on filing: miniscript nesting depth is not a risk for this wallet, with the numbers so nobody re-derives them `#mnemonic`

Recorded 2026-08-11 so the question stops being re-asked. Measured with
`miniscript` v13.0.0 from `rust-miniscript-fork`, the crate actually depended on,
against the real 11-key descriptor:

| property | measured | limit |
| --- | --- | --- |
| witnessScript | **498 bytes** | 3600 standardness / 10000 consensus |
| max_weight_to_satisfy | **756 WU** | — |
| parse in Segwitv0 context | OK | itself enforces ops ≤ 201 and stack items ≤ 100 |
| `or_i` nesting depth | ~3 | `MAX_RECURSION_DEPTH = 402` (`src/lib.rs:503`) |

3102 bytes of standardness headroom. `Descriptor::sanity_check()` returns `Ok`,
and that is stronger than it looks: `segwitv0.rs:57` delegates to
`Miniscript::sanity_check` (`analyzable.rs:225`), which checks five properties in
sequence — requires_sig, non-malleable, within_resource_limits, no repeated keys,
and **no mixed timelocks** (`HeightTimelockCombination`). All five pass, so the
classic deep-nesting bug — one spend path needing both a height lock and a time
lock, satisfiable by no single transaction — does not arise here.

BIP-68 encodings verified from the field layout rather than from the docs that
assert them: `older(65535)` is exactly at the 16-bit ceiling; `older(4255898)` =
`0x40F09A`, bit 22 set, 61594 × 512 s = 365.00 days. **The repo's warning that
`older(65536)` masks to zero is correct** — `65536 & 0xFFFF == 0` and bit 22 stays
clear, so the lock silently becomes *no lock*. One increment from a real wallet.

**Taproot depth, asked separately and answered the same way.** Two unrelated
axes, both boundaries located by probe rather than cited — I built `tr()`
descriptors of increasing depth until they flipped:

| axis | deepest accepted | first rejected | error |
| --- | --- | --- | --- |
| TapTree Merkle depth (nested `{}`) | **128** | 129 | `maximum Taproot tree depth (128) exceeded` |
| fragment recursion inside ONE leaf | **400** | 401 | `maximum recursion depth exceeded (max 402, got 403)` |

`TAPROOT_CONTROL_MAX_NODE_COUNT = 128` comes from the `bitcoin` crate and is
enforced in `descriptor/tr/taptree.rs`; `MAX_RECURSION_DEPTH = 402`
(`src/lib.rs:503`) is context-agnostic — verified there is no Tap relaxation at
either enforcement site (`expression/mod.rs:592`, `miniscript/mod.rs:333`).

Practically, **128 is the wall you hit**: adding script paths the normal way
branches the tree, so the Merkle limit arrives long before a single leaf's own
fragment could approach 402. The 402 cap only binds a pathologically nested
fragment inside one leaf. (First probe of this measured nothing, because reusing
one pubkey trips the repeated-key check before any depth check — worth knowing if
anyone re-runs it.)

The real costs of this shape are downstream of Bitcoin entirely: F-127, F-130,
F-131, F-132, F-134, and `md address` refusing the keys on depth. Depth itself is
two orders of magnitude from anything that bites.

### F-136 — `md encode` does not auto-chunk, though two places say it does (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-11 from the codec lens; **confirmed first-hand** — this is the
error that stopped the journey build before the review raised it:

```
$ md encode --group-size 0 '<the 11-key policy>'
md: codec error: payload is 182 data symbols; the codex32 regular code caps
    single strings at 80 (use chunked encoding / --force-chunked)
[exit 1]
```

The operator has to know to retry with `--force-chunked`. The flag's own help
calls the behaviour automatic ("Reserved for v0.2; mk-codec auto-dispatches
today" on the mk side), and the codec docs describe dispatch as automatic. Either
auto-chunk on overflow or stop describing it as automatic; today the first
encounter with a large policy is a hard error that reads like the policy is
unsupported.

### F-137 — the md encoder has no depth guard but the decoder does, so an unrestorable card is expressible (owning phase: **operator journeys**) `#mnemonic`

Raised by the codec lens; **carried on that report's authority — I have not
re-measured it.** See `design/agent-reports/miniscript-nesting/codec.md` §F5.

The claim: the encode path applies no recursion/depth bound while the decode path
does. If exact, the failure mode is the worst one a backup tool has — a policy
that encodes cleanly, engraves onto steel, and then refuses to decode on the way
back.

**Confirm before acting**, and confirm in this order: (1) does an encodable-but-
undecodable depth actually exist, or does some earlier bound (payload symbols,
chunk count) always bite first? (2) if it exists, the guard belongs on the
ENCODER, since that is the side that can still say no while the plate is blank.
A decoder-only bound protects the reader and abandons the writer.

### F-138 — WITHDRAWN: the Go port does NOT enforce a `Renderable` bound Rust lacks `#mnemonic`

**WITHDRAWN 2026-08-11.** The pre-flash conformance review refuted it by
measurement: zero hits for `Renderable` in Go `sysw/` and in every Rust crate.
It exists only in fork-native GUI md-template code, which the Rust-primary rule
explicitly exempts, and it has nothing to do with this seam. Filed on a report's
authority without re-measuring, and it was wrong — the entry stays, withdrawn,
rather than being deleted.

Original text follows.

Raised by the codec lens; **not independently re-measured.** See
`codec.md` §F6–F7. That report also measured Rust↔Go bounds as otherwise in exact
lockstep, which is the good news here.

The asymmetry matters under the **Rust-primary rule**: if the fork's Go port
refuses a policy the Rust codec accepts, then the machine is the one saying no,
and the constellation's normative behaviour is being set downstream. That is the
direction the rule exists to forbid.

Two legitimate resolutions, and the choice is a real one: either `Renderable` is
a genuine constraint of the engraving surface (a plate that cannot be drawn is
not a policy problem, it is a physics problem) and belongs in Rust with a test
vector so both sides agree — or it is fork-native GUI logic and should not be
able to reject a valid card. Decide which; do not leave it implicit.

### F-139 — CORPUS.md §C6 has an answer now (owning phase: **operator journeys**) `#mnemonic`

`descriptor-mnemonic/design/CORPUS.md` §C6 "Pathological deeply-nested
miniscript (chunking forced)" has stood as an explicit placeholder — its own text
says the 8-nested-`or_d` form "actually fits single string (45 B)" and that a
genuine chunking-forcing example would be defined "once the spec is closer to
fixed". The summary table still reads `C6 | Chunking-forced | TBD`.

The four-tier degrading wallet is that example, measured: **182 data symbols
against a single-string cap of 80**, forced to 3 chunks. Two 32-byte hash
literals and four timelock arguments are what get it there — not key count, which
a keyless BIP-388 template does not pay for.

Fill C6 in with it, including the measured symbol count and the bytecode figure,
so the next person does not re-discover that the 12-key `multi(5,…)` alternative
encodes to 13 bytes and one string.

### F-140 — `compare-cost` omits the witnessScript from its wsh column but not the tapleaf from its tr column, inverting the comparison it exists to inform (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-11. Raised by the limits lens, survived adversarial refutation with
its numbers independently reproduced, and the **mechanism re-read at source by me
before filing**.

`mnemonic compare-cost` is the tool an operator uses to choose between a
`wsh(...)` and a taproot form of the same policy. For this wallet it reports
taproot as **+127..+131 vB per input** more expensive. The true delta is
**+1..+6 vB**. The comparison is not close to right, and it points the wrong way
for a decision that gets engraved.

**Mechanism — one side counts its script, the other does not.** Verified in the
fork at `/scratch/code/shibboleth/rust-miniscript-fork`:

- `Wsh::plan_satisfaction` (`src/descriptor/segwitv0.rs:164`) is exactly
  `self.ms.build_template(provider)` — **no script placeholder is pushed**.
- the Tr path (`src/descriptor/tr/mod.rs:500-501`) pushes
  `Placeholder::TapScript(script)` **and** `Placeholder::TapControlBlock(..)`.
- `Plan::witness_size()` (`src/plan.rs:258`) sums `self.template`, so it silently
  includes the tapleaf script for tr and silently excludes the witnessScript
  for wsh.
- `mnemonic-toolkit` consumes it raw:
  `crates/mnemonic-toolkit/src/cost/enumerate.rs:267` → `plan.witness_size()`.

For this descriptor the omission is **501 bytes per input** — the 498-byte
witnessScript plus its 3-byte varint — on every row. Measured plan sizes
256/184/152/78 against real satisfactions of 757/685/653/579.

**Two things make this worse than an off-by-one.**

1. The tool prints, in its own notes: *"absolute numbers may differ by ±1 from
   real-tx accounting, **Δ values are correct**"* (`src/cost/mod.rs:173`). The Δ
   is the one number that is not correct, and the note is what stops a careful
   operator from checking.
2. `design/SPEC_compare_cost_v0_26_0.md:213` carries the comment
   `// includes scriptCode (the witnessScript)` against exactly the call that
   does not. The spec asserts the property the code lacks — the same
   record-is-wronger-than-the-code shape this project keeps finding.

**Scope is every wsh descriptor**, not just this one; the error equals the
witnessScript size, so it is largest for exactly the complex policies the tool is
most useful for. Fix in `mnemonic-toolkit` (add the script + varint to the wsh
side), fix the spec comment, and drop or qualify the "Δ values are correct" note
until it is true. A regression test should compare `witness_size()` against a
real `get_satisfaction()` for one wsh and one tr descriptor — that single
assertion would have caught this.

### F-141 — CLOSED on filing: `me sysw pack --region`, because the plan's stage-2 green line was never actually met `#mnemonic`

Found 2026-08-11 by trying to do the thing the feature exists for — put a payload
on the machine — and discovering there was no command that produces the artifact.

`design/IMPLEMENTATION_PLAN_systemwide_payloads.md:147` states the stage-2 green
criterion as:

```
me sysw pack --no-passphrase 'text:...' | wc -c     # 65536
```

Measured, it was **79**. `pack` emits the container; only `wipe` emitted a full
`REGION_LEN` image, and `wipe`'s whole purpose is to destroy a payload rather
than to write one. So the one artifact that can be written to `0x10D00000` had no
command behind it, and the first person to need it — me, this session — padded it
by hand in Python.

**Why this got through.** The criterion was a `wc -c` one-liner in a plan nobody
re-ran after stage 2 landed, and no test asserted it. The stage's own tests
checked that `wipe` emits 65536 and that `pack` separates blob from digest;
neither would notice `pack`'s size. A green criterion that is never executed is
decoration.

**Fixed here, TDD:** three tests written red first
(`crates/me-cli/tests/sysw_cli.rs`), then `--region`:

- `region_pads_the_container_to_a_flashable_image` — exactly 65536, magic at
  offset 0, tail all `0xFF`. **`0xFF`, not zeros**: that is the erased state of
  NOR flash, so the image is byte-for-byte what the sector looks like with only
  the container written. Zero padding would be a 65 KiB write for nothing.
- `region_and_container_have_the_same_digest_and_identity` — padding must not
  move the number the operator compares on screen. It does not, because
  `identity` bounds itself by the header's declared total.
- `region_works_for_a_sealed_payload_too` — `--region` says where the bytes go,
  not what is in them; refusing to seal would make the flashable form the one
  form a secret cannot take.

Verified the flag reproduces the hand-padded image **byte-identically**, and the
digest is unchanged from the container form:
`616f 88f5 bb98 2e84 eb3d 0b5a f3d3 8777`.

Gate: `cargo test -p mnemonic-engrave` 190+ pass, `cargo clippy --all-targets
-D warnings` clean.

**Left open deliberately:** the plan line at :147 is now correct only because the
code moved to meet it. Sweep the other plan/spec `# expected` one-liners for the
same class — a criterion nobody runs is a claim, and this project has been bitten
by claims more often than by code.

### F-142 — CLOSED 2026-08-12 — the Go suite never runs at the device's word size, so a whole class of defect is invisible to CI (owning phase: **systemwide payloads**) `#mnemonic`

Filed 2026-08-11 out of the pre-flash conformance Critical, fixed in the fork at
`74871d3`.

`ParseHeader` widened the wire's `uint32` lengths to `int` before comparing them
against `MaxSectionLen`. On the builder that is 64 bits and harmless; on the
device (Cortex-M33 via tinygo) it is 32 bits, and `pub_len = 0xFFFFFFFF` becomes
`-1`, slips the cap, and yields `TotalLen() == 67` — a small **positive** length
the device would have accepted for a payload the host rejects as malformed.

**The bug is fixed. The blind spot is not.** Every test, including the shared
conformance vectors that exist precisely to stop host and device disagreeing,
runs at the *builder's* word size. No vector can see a 32-bit wrap, because no
vector is ever evaluated at 32 bits. The two tests added with the fix say so in
their own comments rather than implying coverage they do not have: on a 64-bit
builder they pass before the fix too.

**CLOSED 2026-08-12.** `seedhammer/scripts/test-32bit.sh`, wired into the
existing CI test job (`3b42405`). It runs `./sysw/` under `GOARCH=386` — which
both builds AND runs on the host, so the assertion is real — and builds under
`GOARCH=arm`, the device's actual architecture.

**The blocker was cgo, not the package.** The earlier attempt concluded the
package "would not cross-build" and stopped; the real error was `runtime/cgo`
wanting 32-bit glibc headers (`gnu/stubs-32.h`) that a 64-bit devshell does not
ship. `CGO_ENABLED=0` fixes it outright, and these packages are pure Go. A
diagnosis abandoned one layer too early cost this a day.

**Proven to bite before it was committed:** with the original `int(...)`
comparison restored, the amd64 run exits 0 and the script exits 1.

Original analysis follows.

**What to do:** run `go test ./sysw/` for a 32-bit `GOARCH` in CI. `GOARCH=386`
was tried during the review and the package would not cross-build (dependency,
not `sysw` itself) — so the work is making that build, or finding another 32-bit
target that does. Until then, treat every width-dependent conversion in `sysw/`
as unreviewed by machine.

**Grep the port for the same shape while you are there:** `int(` applied to any
`uint32` read off the wire. This one was found by a reviewer looking at a
different question, which is not a repeatable process.

### F-143 — sh2-flash compares the key against a RECORDED fingerprint, not the device's live OTP (owning phase: **post-merge polish and hardening**) `#mnemonic`

Filed 2026-08-12 alongside the fix for the pre-flash flashpath review's I1.

`sh2-flash` now refuses to sign with a key whose fingerprint is not the burned
one — the check the runbook always described and the script never made. But the
expected value is a **constant in the script** (`SH2_BOOTKEY_FP`, overridable),
sourced from `design/HARDWARE_INVENTORY.md`. It answers "is this the key we wrote
down" and not "is this the key THIS device will boot".

The stronger form is available: `picotool otp get BOOTKEY1_0 … BOOTKEY1_15`
reads the 32 bytes straight out of the attached unit's OTP, and comparing
against that makes the check about the hardware in front of you rather than
about a note. It also fails correctly for a second machine with a different key,
where the constant needs a manual override nobody will remember.

**Why it was not done in the same change, plainly:** the device had already
rebooted out of BOOTSEL, so `picotool otp get` could not be run and the exact
output format could not be confirmed. Writing unverified parsing into a safety
check is how a check silently starts passing everything — which is precisely the
failure mode the same change already had to fix twice (an empty `openssl` piped
into `sha256sum` yields sixty-four valid hex characters, and under `set -e` the
pipeline's failure killed the script with no message at all).

**Do it with the device in BOOTSEL**, confirm the row format by running it, and
keep the recorded constant as the fallback for when OTP cannot be read — falling
back LOUDLY, never silently.

### F-144 — the plan has no stage for the LOAD FLOW, so all six stages are done and the feature is inert (owning phase: **systemwide payloads**) `#mnemonic`

Filed 2026-08-12, from the operator's question after the firmware booted: *why
does the machine never look?*

**Measured, in `seedhammer` at `b14662a`:**

| symbol | non-test references |
| --- | --- |
| `ctx.sysw` — **read** | 7 |
| `ctx.sysw` — **assigned** | **0** |
| `sysw.Open` | **0** |
| `ctx.Platform.SyswReader()` — **called** | **0** |

Both ends of the feature exist and are correct. The region reader is real on the
controller (`cmd/controller/platform_sh2.go:581`); the session store, admission
table and flags exist (`gui/sysw_session.go`, `gui/sysw_admit.go`); and the
consumers are wired and asking (`derive_xpub.go:127,135`, `bundle_flow.go:30`,
`sysw_session.go:104,112`). **The pipe between them was never laid.** Nothing
reads the region, opens the container, shows the digest, or fills the session, so
every consumer takes its `ctx.sysw == nil` branch forever.

**This is a PLAN gap, not an execution gap, and that is why it went unnoticed.**
All six stages are complete as written. The plan's only mention of `SyswReader`
is the interface declaration in stage 4's table (`:184`); no stage says *call*
it. Stage 4 built the reader and the store, stage 5 wired the eight programs to
consume from the store, stage 6 gave the emulator an NFC source. The step that
puts something IN the store belongs to no stage.

The spec is not the thing that failed — it specifies the behaviour repeatedly:
*"the device displays it at load, and the operator compares"* (§:438), *"a
plaintext container carrying a secret class is flagged on screen at load"*
(§:545), the `[compared]` gate (§:356), and `seedEntryFlow` offering
Typed / Scanned / **Payload** (§:209). The plan simply never sequenced it, and
the plan passed its R0 review to 0C/0I in that state.

**No operator-visible harm today**, which is the one piece of luck here:
`syswOffer` guards on `ctx.sysw == nil` before drawing anything, so the
"FROM PAYLOAD" choice is never shown and a machine with no payload behaves
exactly as it did before. The feature is inert, not broken.

**What the missing stage owes**, from the spec rather than invented: read
`REGION_ADDR` via `SyswReader()`; `sysw.Open` it; display `[digest-shown]` and
hold for the operator's `[compared]` confirmation; evaluate flags F1–F4 and show
them; populate the one-entry session. Plus the decision the plan never had to
make because it never got here: **when does this run** — at boot, or from a menu
entry the operator chooses?

**The transferable lesson, which is the reason this entry is long.** Six stages,
a green R0 gate, every stage's tests passing, and the feature does nothing. A
plan that enumerates COMPONENTS will pass review while omitting the CALL that
joins them, because reviewers check the stages against each other and not against
"can a user do the thing". A plan for a user-visible feature should state the
end-to-end journey first and derive stages from it, so a missing stage shows up
as a broken sentence rather than as an absent row.

### F-145 — `syswLoadFlow` has no test of its own; the gui harness has no Platform fake with a SyswReader (owning phase: **systemwide payloads**) `#mnemonic`

Filed 2026-08-12 with the load flow itself (`seedhammer` `b1fb067`).

The flow that closes F-144 is exercised by nothing. `go test ./gui/` passes and
would pass just as well if `syswLoadFlow` returned immediately — which is
uncomfortably close to the failure F-144 was about, arrived at from the other
direction.

**Why it was not written rather than faked:** the gui harness's `testPlatform`
returns `nil` from `SyswReader()`, and every existing sysw test drives the
session by constructing `syswSession` directly and calling `load()`. There is no
fixture that hands the GUI a region to read, so a test written today would
either exercise the parts below the flow (already covered) or assert that a nil
reader is handled (the one branch that needs no help).

**What it owes**, and the order matters — the second is the one that would have
caught a real defect:

1. a `testPlatform` `SyswReader` returning a fixture region, so the flow can be
   driven at all;
2. cases for: **no reader**, **probe false**, **malformed header**, **truncated
   region** (header declares more than is present), **unsealed with a digest**
   (operator confirms → compared; operator declines → loaded-but-refusing),
   **sealed with the right passphrase** (compared via AEAD, no digest prompt
   when `pub_len == 0`), and **sealed with the wrong one**;
3. the boot path specifically: **SKIP must leave `ctx.sysw` nil**, and a machine
   with no payload must see no prompt at all — that is what keeps the feature
   additive, and it is asserted nowhere.

Use `crates/me-cli/testdata/sysw_vectors.json`, padded to a region the way
`me sysw pack --region` does, so the fixture is the artifact that actually gets
flashed rather than a hand-built blob.

#### F-145 — PARTIALLY DONE 2026-08-12, and its stated reason was wrong `#mnemonic`

Three tests landed in `seedhammer` `9134ca0`, each mutation-checked. **Correction
first:** F-145 claimed the gui harness had no Platform fake with a `SyswReader`
and no region fixture. Both already existed — `testPlatform.sysw` with
`SyswReader()` (`gui_test.go:343,447`) and `sysw.FileReader`
(`sysw/read_host.go:9`). The gap was real; the reason given for it was written
from assumption rather than a grep, which is the same failure the entry above it
is about.

Covered and proven to fail when the code is broken: the additive property (nil
reader and probe-false return false, create no session, never call `Read()`),
boot **SKIP** leaving the machine untouched, and §5.2's "never say unreadable".

Still uncovered, and now blocked on F-146 rather than on fixtures: malformed and
truncated regions producing no session, unsealed-with-digest in both operator
directions, and sealed with the right and wrong passphrase.

### F-146 — gui flow outcomes cannot be asserted: `runUITouch` gives the test goroutine no synchronised view (owning phase: **systemwide payloads**) `#mnemonic`

Filed 2026-08-12 from writing F-145's tests, and it is why three of them are
missing rather than merely unwritten.

`runUITouch` drives a flow as an `iter.Pull` coroutine on another goroutine. A
test can observe what was DRAWN — `pumpUntil` on frame text works, and is what
the surviving assertions use — but it cannot reliably observe what the flow
RETURNED or what it wrote to `ctx`. Reads of a captured variable or of
`ctx.sysw` from the test goroutine are unsynchronised and can be stale.

**Measured, not deduced.** A mutant that made the malformed-region path build a
session and return `true` left the test PASSING. The mutation was verified to
have landed (the first attempt matched nothing and silently "passed" — a
false-survivor that nearly got recorded as evidence of coverage). Those
assertions were then deleted rather than kept as decoration.

**What it needs:** a way to run a flow to completion and hand its result back
with a happens-before edge — a done channel closed by the coroutine and waited
on by the test, or a harness variant that returns the flow's value. Every gui
flow test today asserts only on drawn text, so this is not specific to `sysw`:
**no gui flow's return value or context mutation is under test anywhere.** That
is a large blind spot on a firmware whose flows decide whether a secret is
handed to a program.

Do this before F-145's remaining cases; without it they would be written, pass,
and prove nothing.

#### F-146 — MISFILED. Corrected 2026-08-12 by the load-flow fable review `#mnemonic`

**The diagnosis was wrong.** I claimed `runUITouch` gives the test goroutine no
synchronised view because `iter.Pull` runs the flow on another goroutine, and
concluded that **no gui flow's return value is under test anywhere**. That
conclusion rested on a wrong premise and is withdrawn.

`iter.Pull`'s `stop()` runs the body to completion and returns only after it —
verified by the reviewer against the go1.26.3 source and with `-race` clean. The
happens-before edge I said was missing is there. Assertions placed **after**
`quit()` do observe the flow's writes, and do kill a reachable mutant.

**The observation was real; both actual causes are mine.** (1) My assertions ran
*before* `quit()`, so they read values the flow had not yet written — proven with
a poisoned sentinel. (2) The mutant I used sat in the malformed-region branch,
which **never executes**: `FileReader.Read()` rejects junk first via
`boundBlob`, so `ParseHeader` is never reached with that input.

**The lesson, and it is sharper than the one I filed.** I asserted the mutation
had LANDED — that the text was in the file — and treated that as evidence it had
RUN. Presence is not execution. A mutation harness must prove the mutated line
executed, not merely that it was written; otherwise every unreachable mutant
reads as a surviving one, and unreachable code is exactly where mutants are
easiest to place by accident. See [[mutation-testing-finds-false-passes]], which
this extends.

No harness work is needed. The assertions belong after `quit()`.

#### F-145 — NOT BLOCKED, and writing the tests found a Critical `#mnemonic`

The "blocked on F-146" status was wrong, because F-146 was wrong. All five
remaining cases are writable today with the existing harness, using
post-`quit()` assertions and the direct-call pre-queued-events idiom already in
the tree. The reviewer wrote and ran all five.

**Three pass at HEAD. The two sealed ones failed — because they catch C1**, the
zero-filled passphrase buffer that made every sealed payload unopenable. The
tests I deferred as unwritable are the tests that would have caught the worst
defect in the change.

Remaining work is now ordinary: port those five cases in (malformed and
truncated regions producing no session, unsealed-with-digest in both operator
directions, sealed with the right and the wrong passphrase). The buffer-contract
test committed with the C1 fix covers the initialisation defect itself; these
cover the flow around it.

### F-147 — I claimed `clippy clean` in three commit messages while it was RED, because `cmd && echo OK` prints nothing when cmd fails (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-12. Found by the stage 7/8 implementer, not by me, and confirmed
by checking out the commit into a worktree and running clippy there:

```
$ git worktree add /tmp/wt c49199b && cd /tmp/wt
$ cargo clippy -p mnemonic-engrave --all-targets -- -D warnings
error: this `repeat().take()` can be written more concisely
error: could not compile `mnemonic-engrave` (test "sysw_cli")
exit 101
```

The offending line is mine — `std::iter::repeat("abandon").take(40)` in the
passphrase-bounds test added at `b34944d`. So **`b34944d`, `4692e40` and
`470c43f` all carry a gate claim that was false when written.**

**The mechanism, which is the point.** I verified with

```sh
cargo clippy … >/dev/null 2>&1 && echo "  clippy clean"
```

When clippy PASSES this prints a line. When it FAILS it prints **nothing at
all** — and nothing is what I then failed to notice, because I was looking for
the presence of a problem rather than the absence of a confirmation. The commit
was never gated on the exit status; the `&&` only decorated the transcript.

This is [[empty-output-is-not-absence]] pointed at my own tooling, and the exact
inverse of the `gofmt -l` trap already recorded in
[[mutation-testing-finds-false-passes]]: there, a command reported by PRINTING
and exited 0, so `&& echo OK` fired falsely. Here the command reported by
EXITING, and `&& echo OK` stayed silent. Both end with a false claim in a commit
message; the tell in each case was output I did not read.

**Fix the habit, not the instance.** Print the exit status unconditionally —
`cargo clippy …; echo "clippy exit: $?"` — so a failure produces a LINE rather
than a silence. A verification whose failure mode is "no output" cannot be
distinguished from one that never ran, which is the same reason
`SYSW_REQUIRE_VECTORS=1` exists in this repo.

Also note the near-miss it rode in with: at `4692e40` I read `8` from
`grep -c "test result: ok"` and committed, while a suite was failing further down
the same run. Counting successes is not checking for failures. Grep for `FAILED`
across the whole run FIRST, then count passes.

Nothing to fix in the tree — the implementer corrected the lint inside
`2b570fc`, and HEAD is clean: clippy exit 0, 0 `FAILED`, 10 suites ok, verified
here.

### F-148 — flashing is remote-safe; VERIFYING a flash is not — FIRST VERIFICATION LANDED 2026-08-12 (owning phase: **systemwide payloads, stage 11**) `#mnemonic`

Recorded 2026-08-12 when the operator noted they are remote. Two halves of the
flash operation have opposite answers, and conflating them is how a remote
session ends with a machine nobody can judge.

**The rule was exercised end to end on 2026-08-12 and held.** `ga039c2b` — the
whole systemwide-payload feature, stages 7–13 — was signed and flashed remotely
while the operator was away, recorded as FLASHED/UNVERIFIED, and then confirmed
booting by the operator once physically present. Remote flash, in-room verdict,
exactly as this entry prescribes. The split is not theoretical.

**Flashing is recoverable without hands, and here is why.** `Init()` requires a
20–28 V USB-PD contract before it configures the LCD, and reboots into BOOTSEL
when it does not find one. A computer's USB port cannot supply that. So a device
left plugged into the workstation **returns to BOOTSEL by itself** after every
boot attempt — observed: it is enumerated as `2e8a:000f` right now, having been
flashed and booted earlier today. A bad image, a wrong signature and a good image
all land in the same reachable place. No button press is required, and
`sh2-flash`'s own notes confirm neither script contains an OTP write, which is
the only unrecoverable class.

**Verifying is NOT remote-able.** "It boots" can only be judged on the machine's
normal supply, because on workstation power a correctly signed image is
indistinguishable from a rejected one — both give a dark screen and a device back
in BOOTSEL. So a remote flash can be *performed* and cannot be *confirmed*.

**Consequence for stage 11**, the tree's first flash write: its hardware gate has
a precondition the plan does not yet state — **someone with physical access must
judge the boot before the result is called good.** Until then the honest status
is "flashed, unverified", and no follow-on work may assume the image runs.

Practical rule: remote sessions may flash freely as long as the device stays on
workstation USB, and must record the outcome as UNVERIFIED. Moving it to machine
power is the verification step and belongs to whoever is in the room.

### F-149 — stage 12's integration is pinned by AST only; nothing drives a completed engrave into the verify flow (owning phase: **operator journeys / simulator**) `#mnemonic`

Filed 2026-08-12 on the whole-cycle review's recommendation (item 2.3.3), which
judged it worth a follow-up rather than a log paragraph.

`backupWalletFlow` reaching `plateVerifyFlow` after a COMPLETED engrave is
asserted structurally — an AST check that the call exists — and by no test that
runs it. The behavioural pin is feasible today: the harness already has
`testEngraver`, so a test can drive an engrave to completion and assert the
verify flow follows.

**Why it matters more than a normal coverage gap.** §7's verify is the last
thing standing between a mis-cut plate and an operator who believes their backup
is good. An AST check proves the call is written; it cannot prove the flow
arrives there with the state it needs, which is exactly the class of defect this
feature has produced twice — `ctx.sysw` read everywhere and assigned nowhere
(F-144), and §8c's `done` button built but never drawn.

**ATTEMPTED 2026-08-12, and the premise is wrong — measured.** The review said a
behavioural pin was "feasible, the harness has `testEngraver`". It is feasible in
principle and unaffordable in practice, because `backupWalletFlow` builds a REAL
seed plate: the engrave screen reports an **11:14** job, and pumping **200,000
frames took 71 s without completing it**. One test would have more than doubled
the `./gui/` suite. The attempt was reverted rather than shipped slow or shipped
failing.

What the attempt DID establish, and it is most of the value: every step up to the
engrave is drivable and was driven — the seed review screen, the BIP-39
passphrase offer, and the hold-to-start, all reached in ~2.7 s. **The journey
arrives at the engrave; only the completion is out of reach.**

**So the work is not "write the test", it is "make a completed engrave cheap".**
Options, in the order I would try them:
1. a test seam that lets a plate be substituted — `residency_wiring_test.go`
   already drives `NewEngraveScreen` with an 8-knot synthetic spline in
   milliseconds, so the machinery exists; what is missing is a way for
   `backupWalletFlow` to accept one;
2. a `testEngraver` that can report the job complete on demand, rather than
   simulating its duration;
3. failing both, keep the AST pin and accept the gap KNOWINGLY, which is what
   this entry now records.

Not urgent: the walk-through review drove the surrounding journeys by execution
and they closed. But "the call exists" and "the journey arrives" are different
claims, and only one of them is currently tested.

### F-150 — the on-device wallet-descriptor builder needs major attention: it dead-ends, assumes one key, and offers none of miniscript (owning phase: **a future cycle — needs its own brainstorm**) `#mnemonic`

Filed 2026-08-12 from the operator's own use of the feature on the machine.
**These are field observations, reported as given; only the code pointers below
are mine and only they are verified.**

**1. It dead-ends. `buildMultisigPolicyFlow` fails to deliver a descriptor after
configuration — a BLANK SCREEN after pressing next.** This is the severe one: the
operator completes the configuration and gets nothing, with no error to act on. A
blank screen is the failure mode that teaches an operator the machine is broken
rather than that an input was wrong, and it is the same shape as this cycle's
recurring defect — a flow that is built and does not arrive.

**2. It assumes the operator at the console holds only ONE key.** Real
multisig setups routinely have one person holding several cosigner keys — the
pathological wallet in `design/journeys/` is exactly that: 11 keys from 3
masters, all reachable from one seat. A builder that cannot express "I hold @0,
@1 and @2" cannot build the wallets this project already ships journeys for.

**3. Script types are limited to three, and taproot is absent.**
`multisigScriptChoices()` (`gui/multisig_build.go:276`) offers exactly
`wsh (native segwit)`, `sh(wsh) (nested segwit)`, `sh (legacy)`. There is no
`tr()`, so no taproot wallet can be built on the device at all. Verified by
reading the function.

**4. No miniscript operators.** `after()`, `older()`, hash locks
(`sha256`/`hash160`) and the composition operators are unavailable, so every
timelocked, degrading or hashlocked policy is un-buildable on the machine. Note
the constellation already handles these end to end on the HOST — the
pathological wallet uses all four timelock kinds plus a `sha256` hashlock, and
`md` encodes it — so this is a device-side gap, not a codec one.

**Scope note, and the reason this is filed rather than fixed.** Items 3 and 4
are not bug fixes; they are a feature. Miniscript on a 480×320 touch panel is a
design problem (how does an operator compose `and_v(v:older(65535),multi(2,…))`
by tapping?) and it deserves its own brainstorm before any code. Item 1 is a
defect and could be fixed on its own; item 2 sits between the two. **Do not fold
these together** — the dead-end should not wait on the design work.

Related: the host side already does all of this (`md compile`, `md encode
--from-policy`), so a design that lets the device CONSUME a host-built policy —
which is what the systemwide-payload feature now delivers — may be cheaper than
teaching the panel to author one. Worth weighing before building an editor.
