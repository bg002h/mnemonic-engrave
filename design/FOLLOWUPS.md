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

### F-78 — "·" has no glyph in the display font, and four shipped screens use it (owning phase: ownerless residue; a font cycle, not a feature cycle)

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

### F-85 — §2.2 does not name the during-engrave residency (owning phase: before the release tag)

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
*is* the geometry being cut; it must be resident while the needle moves. No
ordering of wipes changes that, and a plate pipeline over `[]byte` would
**relocate** the secret rather than remove it, because the spline still encodes
it. Filing it as work-to-be-done would be dishonest bookkeeping: a register whose
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
- **F-68** — closed by `scripts/plan-cite-gate.sh` (`7cdcbfc`), which resolves
  every `file:line` and `pkg.Symbol` in a plan against real source and prints the
  line. It was written for Plan B, whose defects are citations rather than code.
  It caught three of the author's own mis-cited lines on its second run.
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
