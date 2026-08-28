# FOLLOWUPS — mnemonic-engrave

Low/nit items deferred from architect reviews (per the iterative-architect-review standard: Critical/Important fixed inline; low/nit recorded here). Promote to a cycle when convenient.

## Convention — a follow-up's STATUS lives in its heading

**If an item is closed, answered, withdrawn or partially done, the heading says
so.** Not the body, not a nested `####` entry, not a paragraph three screens
down.

The reason is that status gets *counted* far more often than it gets read. Asked
how many `#mnemonic` items were open, a grep over headings said 24; the real
number was about 16, because four items recorded their closure somewhere a grep
could not see. Reconciled 2026-08-12: F-129, F-145 and F-146 had the answer in a
nested entry, and F-144's fix had shipped without anything being written down at
all.

Keep the nested entries — they carry the reasoning, and F-145's records that its
stated reason was wrong, which is worth more than the closure. Just put the
verdict in the heading too, so the index and the detail agree.

This is the same failure that sent a session rebuilding the finished simulator
overlay: `PREP_journeys.md`'s heading named the task while its body, forty lines
below, recorded it built. A heading that describes the work rather than its state
is a trap for whoever reads only headings — which is everyone, most of the time.

## Phase policy — test infrastructure is POLISH, not functionality (operator ruling, 2026-08-12)

> *"All these checks on the behavior of the code are fantastic but if second tier
> priority more appropriate for Polish phase rather than functionality phase. If
> such things are easy, do them… but if they require extensive coding and testing
> let's save them for v0.0.1."*

**The default owning phase for a test-infrastructure item is `polish / v0.0.1`, not the phase that discovered it.** Discovery and ownership are different things, and I have been conflating them: an item found while doing journeys work is not thereby journeys work.

The split is by **cost**, not by importance:

- **Do it now** — a handful of lines, no new harness, no new abstraction. The F-151 ink floor was this: one file, one helper, one test.
- **Defer to v0.0.1** — anything needing a new fake, a new build step, a font-metrics model, or its own spec. Extensive coding *about* the code is precisely the second-tier work this ruling names.

Applies to a real bug found *by* such a check too: **the bug is functionality and gets fixed now**; the generalised guard against its whole class is polish. F-151 is exactly that shape — the blank screen was fixed in `ae9fefa` (functionality, shipped, hardware-confirmed), while items (2) and (3) generalise it and are now parked.

Context that makes this the right call: the machine currently does not draw a plate layout during an engrave, and there is no completed-engrave path in the simulator. Functionality gaps that visible outrank hardening the tests that guard what already works.

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
`post-release feature` · **`key & password custody refinement`**

**`key & password custody refinement`** was created by operator ruling
2026-08-17, in the operator's own framing:

> "let's make further refinement of verifying user has keys and passwords for
> any or all keys a separate polish phase"

**What it holds:** the general problem of confirming the operator actually
possesses every key and every password a set depends on — as opposed to S6b's
narrower job, which is stopping a *single-sig* artifact from vouching for a
wallet it cannot restore.

- **F-205** — `backupWalletFlow` and `deriveXpubFlow` engrave passphrase-bound
  artifacts and say nothing about the missing factor (moved here 2026-08-17 from
  "none yet — needs a scoping decision")
- **the multisig marking** — §3 Q5 of `REQUIREMENTS_s6b_pre_flash_cycle.md`.
  Same defect as the single-sig marking, more plates, and it multiplies across
  cosigners, which is what makes it a phase rather than a follow-on.

**It does NOT gate the hardware flash.** S6b closes the single-sig path; this
phase generalises it afterwards.

**Consequence for S6b's design, recorded here because it is easy to violate
accidentally:** `validateMdmk` is a four-call-site chokepoint and one of those
callers is `gui/derive_xpub.go:494` — **F-205's own flow**. So marking placed
unconditionally inside `validateMdmk` would close part of F-205 as a side
effect, crossing this phase boundary without anyone deciding to. The marking
must therefore be **conditioned**, not merely located.

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

### F-59 — ~~the artefact was Y-axis play, not cusps~~ **CLOSED 2026-08-21** (withdrawn — the artefact was Y-axis play, not cusps)

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

### F-72 — ~~md-codec 0.40 → 0.42 rode into the Task 1 commit~~ **CLOSED 2026-08-21** (historical note, not a task)

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

### F-106 — ~~§10.2.4's window runs 2x (6:00, not 3:00): a LATE ARM EDGE lands on the deadline~~ **CLOSED 2026-08-10** (heading corrected 2026-08-20)

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

### F-107 — ~~the RENDERED seed is scrubbed ONLY on the wipe path; a normal exit leaves the twelve words in `ctx.B`~~ **CLOSED 2026-08-10** (heading corrected 2026-08-20)

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

### F-108 — ~~`plate.Spline` is never zeroed AFTER the cut: F-83 buys the mid-cut window and nothing ends it~~ **CLOSED 2026-08-10** (heading corrected 2026-08-20)

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

#### Addendum, 2026-08-10 — the ledger UNDERSTATES it, with a measured table

*Filed as a second `F-120` heading; folded here 2026-08-21 so the number is not used twice. A follow-up number is cited in commits and reports, so reusing one makes those citations ambiguous forever.*

It is not a boundary case at 90. The device admits **27** codex32 lengths in
48–90; `me` admits **10**; **22 diverge**. The reverse set is **empty**, so
unlike C1 this cannot produce an unopenable backup — every `ms1` that `me`
emits, the device accepts.

The entry's `[50,56,62,69,75] ∪ [51,58,64,70,77]` is misleading: those are two
**disjoint tag families** (`entr` v0.1 vs `mnem` v0.2), and an `entr`-tagged
77-character string is refused while a `mnem`-tagged one at the same length is
admitted. Also, the "widen `me`" option is not actionable from this repo — the
narrowing lives in `ms-codec`.

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

### F-111 — ~~`knotBuf` unzeroed wherever a plate is built and no cut happens — SUBSUMED by the F-108 design~~ **CLOSED 2026-08-21** (subsumed by the F-108 design)

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

#### Addendum, 2026-08-11 — DOWNGRADE recommended, and the security question is answered

*Filed as a second `F-109` heading; folded here 2026-08-21 so the number is not used twice. A follow-up number is cited in commits and reports, so reusing one makes those citations ambiguous forever.*

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

### F-83 — ~~the plate cannot be wiped until the engrave finishes — ACCEPTED LIMITATION, not a follow-up (operator, 2026-08-08)~~ **CLOSED 2026-08-21** (accepted limitation — operator ruling, not a follow-up)

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

### F-81 — ~~WITHDRAWN 2026-08-08 before it was ever open~~ **CLOSED 2026-08-21** (withdrawn before it was ever open)

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

### F-127 — ~~`mk encode --from-md1` cannot read a CHUNKED md1~~ **CLOSED 2026-08-21** `#mnemonic`

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

#### Re-measured 2026-08-21 — it is worse than "cannot read a CHUNKED md1"

`mk encode --from-md1` now cannot read **any** current md1, chunked or not:

```
error: md1 input rejected: wire-format version mismatch: got 9, expected 4
```

The vendored md-codec has fallen five wire versions behind. The consequence is
concrete and it bit while building F-216's tests: **the one command that derives
a key card's `policy_id_stub` from the card it belongs to cannot be used with any
card this constellation produces today.** Every mk1 must be minted with an
explicit `--policy-id-stub`, which means computing the stub by hand — from
`md inspect`'s `wallet-descriptor-template-id`, first 4 bytes, for a keyless
template.

That is exactly the manual step `--from-md1` exists to remove, and getting it
wrong produces a card that is refused at membership with no indication that the
stub was the problem.

#### RESOLVED 2026-08-21 — `mnemonic-key` 6ac5f99 + 85cf6c7 (this repo)

Two defects, stacked, and the second was invisible until the first was fixed.

1. The vendored md-codec was 0.34.0, five wire versions behind, so every chunk
   was refused with `wire-format version mismatch: got 9, expected 4`. Bumped
   to 0.42.0.
2. With that fixed the error CHANGED rather than cleared, to `chunk set
   incomplete: got 1 chunks, expected 4` -- because `mk` decoded each
   `--from-md1` value INDEPENDENTLY. A four-chunk card was four incomplete
   sets. Values are now grouped by the 20-bit chunk-set id in their wire
   header, and each GROUP yields one stub.

**The entry understated the blast radius, and the correction is the useful
part.** It read "any policy over the single-string cap", framed as a large-
wallet problem. Measured: a keyed wallet policy is **246 data symbols** against
a single-string cap of **80**, so EVERY keyed wallet-policy card the
constellation can produce is chunked. `--from-md1` was not degraded for big
wallets; it was **absent for all of them**, and only ever worked on keyless
templates small enough to fit one string. The severity downgrade to Minor was
made on the belief that the stub "remains derivable" by hand -- true, but it
was the *only* route for every keyed card, not a fallback.

**Why it hid for so long.** The one test covering keyed wallet policies used a
hand-minted **138-symbol single md1 string** -- a card no encoder emits and no
engraver could cut, since it exceeds the 93-symbol regular-code cap. The test
was standing in for a card that cannot exist. 0.42.0 enforces the cap, refused
the fixture, and that is how both defects surfaced together. Replaced with two
REAL chunk sets minted by `md encode --force-chunked`.

`--from-md1` still means one card per POLICY: grouping keys on the set id, not
on adjacency or on the whole argument list, so a key card belonging to two
wallets still gets two stubs in first-appearance order.

**Equivalence measured, not assumed.** Ran `transcript_pathological.sh` before
and after and diffed the outputs: the 30 mk1 strings across 11 cards, and
`card-index.txt`, are BYTE-IDENTICAL. Switching the binding route changed no
engraved plate. The journey's section 5, headed "OBSTACLE 1", is now a
demonstration that it works, and its hand-derivation survives as a CROSS-CHECK
(85cf6c7) -- the only thing in that journey that would notice mk silently
switching identities.

**A second defect the bump exposed, in CI rather than code.** The repo commits
a `vendor/` tree that the `--offline --locked` release build resolves against.
Bumping `Cargo.lock` to 0.42.0 left `vendor/md-codec` at 0.34.0, so a release
build would have silently reintroduced this exact bug. `ci/repro/vendor-
freshness.sh` REDed on it locally, which is precisely the PR-time failure it
was ported in to catch. Re-vendored and compiled through the vendored tree.

**Not the whole labour win it was billed as.** The working estimate was that
this collapses ~33 hand-built commands to ~3. Measured, it does not: `mk
encode --xpub` takes ONE key, so an 11-cosigner wallet is 11 invocations
before and after. What the fix removes is the hand-copied hex between a policy
and the cards claiming membership in it, plus the `md inspect` step that
produced it -- and it makes keyed cards possible at all. Reaching ~3 commands
needs a batch input mode (`--xpub` repeatable, or a key file), which is a
separate feature; see F-223.

### F-128 — ~~the stub's spec sentence and `mk`'s behaviour name different identities~~ **CLOSED 2026-08-21** `#mnemonic`
#### RESOLVED 2026-08-21 — `mnemonic-key` (SPEC 3.3 + 5)

Fixed in the TEXT, since the refutation below establishes the code is right.
3.3 now carries the form-aware dispatch as a normative table, with the
measurement that shows why an unconditional WalletPolicyId is wrong: the same
wallet in both forms shares a key-stable template-id (`a235ee75`) but has
DIFFERENT policy-ids (`38bd7cec` keyed, `16ba6a79` keyless), so a keyless
template hashes to a value binding nothing about its cosigners.

5 (Linkage to MD) was the load-bearing half and is the reason to keep reading
past 3.3. Its recovery flow said to compute the WalletPolicyId unconditionally,
so a tool built to the letter of this spec would reject EVERY card minted from
a template and present it as "none of my cards belong to this wallet". Fixed,
with that failure mode named inline.

The funds-relevant consequence is now stated rather than implied: **one wallet
has two stubs**, and membership compares them verbatim. Pinned by a new test,
`one_wallet_two_forms_two_stubs`, which asserts AGAINST `16ba6a79` so a
regression to the old wording is caught by value; mutating the dispatch to
unconditional kills 3 tests.

#### The stale-pin hypothesis is REFUTED, 2026-08-21

This entry speculated the divergence might be a consequence of md-codec drift
"after 0.34.0 — which would make this F-127's twin rather than an independent
bug". F-127 is now fixed, md-codec went 0.34.0 -> 0.42.0, and **the behaviour
did not change**: `mk` still stamps a keyless template with the
`WalletDescriptorTemplateId` prefix. Measured on the pathological card, whose
`wallet-policy-mode` is `false`:

```
wallet-descriptor-template-id: 5b48af35d4321a3ac18b43045e2523cc
wallet-policy-id:              bd6ba7e6bd7a86038b3963f977e727a6
policy_id_stubs (mk decode):   5b48af35     <- the TEMPLATE id
```

So this is NOT F-127's twin and not drift. The dispatch is deliberate and
form-aware -- `derive_stub_from_md1_card` picks the WalletPolicyId for a keyed
policy and the WalletDescriptorTemplateId for a keyless template, citing audit
I1 (2026-06-10) and toolkit #28 -- and the tests now pin BOTH arms with
cross-language goldens. What is stale is the **spec sentence**, which names
`WalletPolicyId` unconditionally and describes only one of the two arms.

That relocates the fix: `SPEC_mk_v0_1.md` 3.3 needs the form-aware rule
written down, and no code change is warranted. Still open on those terms.



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

### F-129 — ANSWERED 2026-08-11 (see the nested entry) — `--path` is mandatory for a non-canonical wrapper and flattens divergent origins; which source wins on restore is unpinned (owning phase: **operator journeys**) `#mnemonic`

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

### F-136 — ~~`md encode` does not auto-chunk, though two places say it does~~ **CLOSED 2026-08-21** `#mnemonic`

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

#### RESOLVED 2026-08-21 — `descriptor-mnemonic` `9af0975`

`md encode` now chunks automatically when the payload exceeds the codex32
regular code's 80-data-symbol cap. **Neither help string needed editing** — the
code caught up to them. That is the shape of this fix: the documentation was
right about the intent and the encoder was the thing that disagreed.

**The scale, re-measured:** this entry recorded 182 data symbols; a plain 2-of-2
measures **246** against a cap of 80. Combined with what F-127 established —
that *every* keyed wallet-policy card is chunked — this was not a large-wallet
corner but the default path for any card carrying keys.

**No wire change.** Auto-chunked output is byte-identical to what
`--force-chunked` produced, pinned by a test. What changed is which inputs are
accepted. `--force-chunked` keeps its documented meaning (chunk even a short
policy), the fallback matches only `PayloadTooLongForSingleString` so unrelated
failures still propagate, and md-codec's fail-closed guarantee is untouched —
`wrap_payload` still refuses to emit an over-length single string.

**One pre-existing test changed, and the distinction is the point.**
`md_encode_default_rejects_oversize` asserted the ERROR, which was the
mechanism. Cycle-4 H6's actual property is *never emit an un-decodable single
string*. Restated to assert that: an over-cap policy emits a chunk SET, every
chunk fits the regular-code envelope, and the set decodes back — which holds
across this change and still catches a regression that emitted one long string.

**Rust-primary check:** the Go port never had this. `md/encode_singlesig.go`
and `encode_multisig.go` route through `split` unconditionally, with no
single-string-or-error mode. Second time in this cycle the strictly-downstream
port was already correct where the primary was not (the first was the mk1
encoder path cap, R2/C2).

**Falsified by this fix, and corrected in the same pass:**
`transcript_walletpolicy.sh`'s comment explaining why the flag is passed, and
`build_pdf_payload.py`'s findings-table row. Both said auto-chunking does not
happen. The journeys keep passing `--force-chunked` — it is now redundant, but
the output is byte-identical and these are recorded walks, so removing it would
change a printed command without changing a card.

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

### F-138 — ~~the Go port does NOT enforce a `Renderable` bound Rust lacks `#mnemonic`~~ **CLOSED 2026-08-21** (withdrawn — the claim was wrong) `#mnemonic`

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

### F-144 — CLOSED 2026-08-12: the load flow was built, shipped in me v0.6.0 / firmware g753f729, and confirmed on the machine — the plan has no stage for the LOAD FLOW, so all six stages are done and the feature is inert (owning phase: **systemwide payloads**) `#mnemonic`

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

### F-145 — PARTIALLY DONE 2026-08-12 (see the nested entry) — `syswLoadFlow` has no test of its own; the gui harness has no Platform fake with a SyswReader (owning phase: **systemwide payloads**) `#mnemonic`

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

### F-146 — ~~MISFILED, withdrawn 2026-08-12 (see the nested entry) — gui flow outcomes cannot be asserted: `runUITouch` gives the test goroutine no synchronised view`#mnemonic`~~ **CLOSED 2026-08-21** (misfiled and withdrawn) `#mnemonic`

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

### F-149 — stage 12's integration is pinned by AST only; nothing drives a completed engrave into the verify flow (owning phase: **polish / v0.0.1**) `#mnemonic`

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

### F-151 — the frame extractor sees text the DEVICE cannot draw, so every wording assertion in `gui/` shares a blind spot (owning phase: **(1) DONE; (2)+(3) polish / v0.0.1**) `#mnemonic`

Filed 2026-08-12. Found by the operator looking at the panel, and by nothing
else in this project.

Unloading a payload produced an almost entirely blank white screen carrying only
the checkmark. Its body was one ~110-character sentence with an em dash and
backticks. **`TestSyswUnloadFlow` asserted three separate phrases from that body
and passed**, because `runUITouch`'s extractor reads the text OPS a frame
contains, not the pixels the device would light. A string the panel renders as
nothing still "appears" in the harness.

**This is not one bad test.** Every `uiContains` assertion in `gui/` — and there
are many — proves that a string was *submitted for drawing*, never that it was
drawn. So the whole class is invisible: over-long bodies that clip, glyphs
missing from the font, text laid out off-canvas. The suite is green and the
screen is blank, which is the exact shape of F-144 and of §8c's undrawn `done`
button, arriving a third time through a different door.

**What would close it**, cheapest first:
1. assert against the RASTER rather than the op list — `op.Drawer` already
   renders to a bitmap for the touch tests, so a "this frame is not blank"
   check (non-background pixel count above a floor) is nearly free and would
   have caught this exact defect;
2. a font-coverage check: fail if a string handed to a screen contains a rune
   the UI face has no glyph for — turns a silent blank into a build error;
3. a width/height budget on modal bodies, so an over-long string is refused at
   the call site rather than clipped at draw time.

(1) is the one to do: it is a handful of lines, needs no font work, and converts
this entire class from invisible to noisy. Until then, **treat every wording
assertion in `gui/` as evidence about intent and not about the screen.**

**(1) DONE 2026-08-12** — `gui/raster_test.go` (fork `c4f50fe`): `runUITouchRaster`
rasterises each frame through the `op.Drawer` the touch tests already build and
counts pixels differing from the frame's own corner, and
`TestUnloadNoticeIsActuallyDrawn` pins the screen that started this.

**With a correction to the recommendation above, which the measurement earned.**
"Nearly free and would have caught this exact defect" was true only of the
*second* attempt. The first floor — 2000, reasoned from *"the broken screen drew
just the checkmark"* — **passed with the original defective body restored.** The
premise was wrong: the frame was never near-empty. Measured:

| body | ink |
| --- | --- |
| original, blank on the device | **2652 px** |
| the fix | **6688 px** |

So the defect is a body that lays out to *almost* nothing, not to nothing, and
an ink floor only separates them if it is set from the two real numbers. The
committed floor is 4000, and it is verified in both directions: exit 1 with the
original `showNotice` body restored, exit 0 with the fix.

Worth keeping as the general lesson, because this project keeps re-finding it: a
rasterising test that then asserts a threshold nothing can cross is **worse than
no test**, since it reads as coverage of exactly the defect it cannot see —
[[mutation-testing-finds-false-passes]] again, arriving through the assertion's
constant rather than its subject. Any future `assertFrameHasBody` floor gets
calibrated the same way, against a restored defect.

**(2) and (3) RE-FILED to `polish / v0.0.1`, operator ruling 2026-08-12** (see
the Phase policy at the top of this file). Both need real machinery — (2) a font
face's coverage set threaded to every call site, (3) a text-metrics model of the
modal box — and neither guards a defect that is live: the screen they were filed
against is fixed and hardware-confirmed. An ink floor catches a body that
vanishes wholesale, which is the failure that actually happened.

**One cheap experiment survives, for whoever picks (2) or (3) up.** `ae9fefa`
changed *two* variables at once — it shortened the body AND dropped the em dash
and backticks — so **which one blanked the panel was never isolated**, and the
ink count cannot say (2652 px either way). That decides which item is even worth
building: flash a build with the LONG body restored but ASCII-only. Blank →
length, so (3). Readable → glyphs, so (2). One flash, one look, and it should
ride along on some other flash rather than earning one.

### F-152 — selecting "from payload" when one is PRESENT BUT NOT LOADED should launch the loader (owning phase: **a future cycle — needs a spec §3.1 state and one plan stage**) `#mnemonic`

Filed 2026-08-13 by operator ruling. Agreed as a feature, deliberately not
implemented freehand.

**Today** the pickers gate on `ctx.sysw != nil`, so a payload sitting in flash
that the operator skipped at boot is invisible inside every program. Their route
back is the `Load Payload` carousel entry, then re-entering the program.

**Wanted:** the picker offers the payload row when one is PRESENT, and choosing
it runs `syswLoadFlow` inline, then hands over the record.

**The mechanics are cheap.** `SyswReader().Probe()` reads eight bytes, so
"present but unloaded" is already distinguishable from "absent", and calling the
load flow from a picker is on the order of thirty lines.

**The design is not, and this is why it is filed rather than written.** Four
questions no implementer should answer alone:

1. **The loaded payload may not hold the wanted class.** Select "from payload"
   in BIP-39 Password, the load succeeds, and there is no `pass:` record — the
   operator has paid a digest comparison, or a passphrase and a ~31 s KDF, for
   nothing. What does the screen say, and where do they land?
2. **Declining the digest now UNLOADS** (§13, 2026-08-13). So a decline mid-app
   drops back to a picker whose payload row has just disappeared — a menu that
   changes under the operator between one press and the next.
3. **Nesting.** The load flow draws a digest screen, possibly a passphrase
   keyboard, and a warnings screen, all INSIDE another program's flow. No other
   source does that, and the Back semantics through two nested flows are
   unspecified.
4. **§3.1's source table does not model this state.** It enumerates what a
   program OFFERS; "present but unloaded" is a fourth state, and adding it is a
   spec change before it is a code change.

**Precedent for filing rather than building: F-144.** That was components that
each worked, joined by a step nobody had specified, and it passed a green R0
gate while doing nothing. This has the same shape — a picker, a loader, and an
unwritten seam between them.

**Cost of not having it is low**, which is what makes deferring reasonable: the
route back is one carousel tap, and LOAD is now the boot default, so the common
path already ends with the payload loaded.

**What it needs:** a §3.1 state for present-but-unloaded, a §13 ruling for
questions 1 and 2, and one plan stage carrying the journey — written as a
journey first, per the plan's own map, so a missing step reads as a broken
sentence rather than an absent row.

---

### F-153 — `me sysw pack`'s record index is 0-based and unlabelled, and `--in` filters blank lines so it is not a line number either (owning phase: **polish / v0.0.1**) `#mnemonic`

Filed 2026-08-12 writing the Load Payload journey
(`design/journeys/SeedHammer-II-load-payload-journey.pdf`). Hit on the FIRST
`me sysw pack` invocation of it.

**The half that is already fixed** (`1538ef0`, this phase): the refusal named a
cause that had not occurred. `sysw::classify` returns `Class::Unknown` for two
unrelated situations, and the message described only one of them:

```
$ me sysw pack --no-passphrase --in records-as-first-written.txt
me: record 1 is not a form this container can place. Descriptors and
    addresses are not yet classifiable here — see sysw::classify
[exit 4]
```

The record that failed was `pass:correct horse battery staple` — a RESERVED
prefix whose body is not lowercase hex (§5.3.1). Descriptors and addresses had
nothing to do with it, and the remedy is one `xxd`. `SyswError::Unclassifiable`
now carries an `UnknownReason`, and the message names the prefix and the fix.

It deliberately does **not** carry the record. The record most likely to land
here is a `pass:` one, whose body IS the passphrase, and stderr is scrolled
back, logged, and pasted into bug reports. Only the prefix is named, and that is
a `&'static str`. Pinned by
`a_plain_text_pass_body_is_refused_by_body_and_never_echoed`, mutation-checked:
with the passphrase spliced into the message the assertion fails with `THE
PASSPHRASE MUST NOT REACH STDERR`, so it is not passing vacuously.

**The half still open — the index base.** Every operator-facing record number in
`me sysw` is 0-based and says so nowhere:

| site | prints |
| --- | --- |
| `sysw_error` (`Unclassifiable`) | `record {i}` |
| `report_unconfirmed` | `record {i}, as given` |
| `print_mdmk_confirmation` | `public record {i}` |

Every text editor, `sed -n 'Np'` and human counts from 1, so on a three-line
file `record 1` sends the reader to the wrong line — it sent me there.
`main.rs`'s own comment calls these "the OPERATOR'S" indices, "the list they can
act on", which is exactly the reading that makes 0-based wrong.

The cheap fix is applied — the messages now say `(records count from 0)`. The
real fix is not `+1`, and that is why this stays open: **`read_records` filters
empty lines**, so with `--in` the record index is not the file's line number
either. A file with a blank line between records makes `record 2` point at line
4. Making the number a LINE number means carrying line provenance through
`read_records`, and renumbering touches three print sites plus their tests.

Not deferrable on correctness grounds — the current text is honest — but the
right answer is a small design decision, not a patch. **v0.0.1**, with the
question stated: number the records from 1, or report the line they came from?

---

### F-154 — ~~the tenth program's carousel dot is drawn underneath the firmware version line~~ **CLOSED 2026-08-20** `#mnemonic`

Filed 2026-08-12, same run. **Measured on the framebuffer, not eyeballed** —
scanning row y=297 of `shots/p09-load-payload-program.png` for near-white runs:

```
circles (hollow, 12 px):  157–169  174–186  191–203  208–220
                          225–237  242–254  259–271
current (filled):         276–288
next circle would be:     293–305        <- one edge visible at 293
tenth circle would be:    310–322        <- entirely inside the text
"Firmware: emu" begins:   305
```

Eight programs' dots have clear room; the ninth is half-covered and the tenth is
gone. It was fine at NINE programs, which is why nobody saw it: `loadPayload`
made ten. The same frame is in every journey document's menu screenshot.

Cosmetic, and it does not mislead — the filled dot is always visible, so the
operator can still see where they are. But the carousel's whole job is to say
how many programs there are, and it now under-reports by two.

Fix is a layout question, not a bug: either move the version lines down, shrink
the dot pitch, or drop the dots past N. Owning phase **polish / v0.0.1** per the
test-infra/cosmetics rule — this is UI polish found during functionality work.

---

**CLOSED 2026-08-20** (`seedhammer` `1cf9dfc`). **The prediction above came
true on schedule**: Stage 4's Wallet Policy program was the tenth, and the
Wallet Policy journey's capture caught it on the emulator's own framebuffer —
two dots drawn *around* "Fi" and "rm", enclosing the letters, with dot ink
running to x=327 against a label starting at x=306.

**The spacing was never the problem; the centring was.** The label is anchored
to the right edge and the pager was centred on the FULL width, so at 480px the
pager could be at most ~132px — and ten 13px dots do not fit that even edge to
edge. The pager is now centred in the room it actually has: the label is
measured first and its width sets the region. Re-running the capture moved the
dots to x=59..241.

**The gate took two tries, and both failures are the interesting part.**

- *The difference method is invalid here.* Render with and without the version
  string, diff the columns — obvious, and broken by this very fix, because the
  pager's position now depends on the label's width. It reported a 195-column
  "collision" on a screen with none.
- *It was aimed at the wrong label.* `uiFlow`'s parameter is named `version`, so
  the test passed `"emu"`: 24px wide, left edge x=452, and a comfortable pass.
  But `run_flow.go:231` calls it with `versionText` — `"Firmware: <v>\nHardware:
  <hw>"` — a two-line label ~171px wide starting at x=305. Against the real
  string the gate fails on the old centring at x=301, agreeing with the
  framebuffer to a pixel.

Also worth carrying: `go test` served a **cached pass** for the first mutation
run, so a mutant looked caught without having run. `-count=1` on every mutation.

### F-155 — the home screen cannot tell you whether a payload is loaded (owning phase: **systemwide payloads — spec question first**) `#mnemonic`

Filed 2026-08-12, same run. Not cosmetic, and not test infrastructure: the
thing whose presence is unreported is a **seed in RAM**.

Three different histories, one identical frame — sha256 of the emulator's own
framebuffer:

```
f773d610f050ad9983c427635f25df6d80893a8c60ec2cef7efaedc2ab134204  loaded at boot
f773d610f050ad9983c427635f25df6d80893a8c60ec2cef7efaedc2ab134204  loaded, then unloaded
f773d610f050ad9983c427635f25df6d80893a8c60ec2cef7efaedc2ab134204  boot offer skipped
```

So an operator who walks away and comes back — or who hands the machine to
somebody else — cannot tell from the home screen whether a mnemonic and a
passphrase are currently live in it. The only way to find out is to enter Load
Payload and read which two options it offers (`LOAD` / `SKIP` vs `LOAD AGAIN` /
`UNLOAD`), which is a state query disguised as an action.

**Why this is a spec question before it is a UI one.** The machine already
refuses to state loaded-ness anywhere else, and there is an argument for that:
an indicator says "there is a seed here" to anyone who picks the device up.
That argument is real, and it is the opposite of the argument for the indicator.
§3.1 should rule it rather than leaving it to whoever draws the home screen.

Note it interacts with **F-152** (selecting "from payload" when one is present
but unloaded should launch the loader): both are about the operator's model of a
state the device tracks and does not show.

---

### F-156 — neither published journey can be regenerated by the commands its own README gives (owning phase: **operator journeys**) `#mnemonic`

Filed 2026-08-12, found while writing the third journey against the convention
the first two set — the convention turned out not to run.

Three independent breaks, all in the pathological document's pair:

| what | says | is |
| --- | --- | --- |
| `transcript_pathological.sh` ×6 | `$W/inputs/…` | the pathological files are in `inputs-pathological/` |
| `build_pdf_pathological.py` ×3 | `inputs/seeds`, `inputs/keys` | same |
| `build_pdf_pathological.py:14,66` | `design/journey/build_pdf.py` | there is no `design/journey/`; it is `design/journeys/` |
| `build_pdf_pathological.py:65` | `keys.json` beside the script | never committed |

`inputs/` today holds the OTHER document's twelve cosigners, so
`transcript_pathological.sh` does not fail cleanly — it reads
`inputs/keys/key-00.xpub`, which does not exist, while `inputs/wallet-policy.txt`
DOES, so it gets partway. The rename happened when the second document was added
and neither script was run again afterwards.

A fourth, smaller one: the README's *Reproducing* section says
`python3 build_pdf_pathological.py` produces the PDF. It produces
`out/journey.html`. The HTML→PDF step is nowhere in the repo; the published PDFs
were made by a manual pass. `build_pdf.py`'s own last page does document
`go run ./cmd/journeykeys > keys.json`, so `keys.json`'s provenance is recorded —
just not where someone following the README would find it.

**Why it matters beyond tidiness.** These documents are the artifact that says
"nothing here is illustrative". A document that cannot be regenerated cannot be
re-verified, so the claim degrades to a promise the moment the first correction
needs folding — which is exactly the situation the twelve corrections above
created.

`build_pdf_payload.py` is the corrected pattern: it resolves every path it uses,
and it runs the Chrome headless print itself so the reproduction instructions
are complete.

**Owned by THIS phase**, not deferred: it is the phase's own toolchain.

---

### F-157 — a nested-segwit multisig is labelled identically to a legacy P2SH one, in the RESTORE DOCUMENT (owning phase: **`SPEC_multisig_build_repair.md` P2**) `#mnemonic`

Filed 2026-08-13 while brainstorming the on-device policy builder. **Measured by
running the three summary surfaces**, not read:

```
wsh(sortedmulti)       -> "P2WSH 2-of-3 multisig (sorted)"
sh(wsh(sortedmulti))   -> "P2SH 2-of-3 multisig (sorted)"
sh(sortedmulti) legacy -> "P2SH 2-of-3 multisig (sorted)"
sh(wpkh) BIP-49       -> "P2SH single-key"
```

The middle two are **byte-identical strings for two wallets that hash to
different addresses**. `gui/md1_inspect.go:20` `scriptName(k md.ScriptKind)`
takes only the root kind, so it cannot see `Template.InnerWsh`/`InnerWpkh`.

The codec is not at fault and says so at `md/md.go:1212-1219`: the discriminant
exists precisely because "they hash to DIFFERENT addresses, so a consumer
building a `*bip380.Descriptor` MUST use this to pick P2SH_P2WSH vs P2SH and
never verify one against the other."

Three callers: `gui/md1_inspect.go:58`, **`gui/multisig_restore.go:51` — the
restore document** — and `gui/bundle.go:315`.

**The engraved steel is correct.** `gui/md1_expand.go:112` honours `InnerWsh`,
and `deriveMultisigLeg` passes the md1 through verbatim and derives no
addresses. So this is F-131's shape exactly: a correct backup with a document
that tells the operator something false about it — and the restore doc is what
they will read years later, alone.

Fix is small (§4.4 of the spec): `scriptName` takes the whole `Template`, three
call sites updated together, and a test asserting the three names are pairwise
distinct — the defect is that two of them are equal.

---

### F-158 — no NFC gather flow can be executed by any test or in the emulator, so half of Build-policy has never run outside the operator's hands (owning phase: **`SPEC_multisig_build_repair.md` P0**) `#mnemonic`

Filed 2026-08-13. This is the cause behind F-150 item 1 reaching hardware.

Three verified facts:

1. **`cmd/emu` cannot deliver a card to any gather.** Walked in a browser: a
   valid mk1 presented via `window.shNFC` both BEFORE and AFTER entering the
   gather left the tally at `md1 descriptors: 0 / mk1 keys: 0`, and **no
   `nfc scan:` log line ever appeared** — so `gui/nfc_scan.go:45`
   `startScanner` received `nil` and never polled. Not specific to Build
   policy; plain Engrave Bundle behaves the same.
2. **`gui`'s test platform has no reader either** — `gui/gui_test.go:445`
   returns nil, with the consequence recorded at `gui/bundle_flow.go:96` and
   `gui/mk1_inspect_test.go:104`.
3. **The only end-to-end test of the build flow stops at the gather.**
   `gui/multisig_build_flow_test.go:199`, whose own comment says "with no NFC
   reader the gather yields zero cards, so a Build flow at n=2 returns on
   gather Back WITHOUT typing a seed".

A prior session recorded the mechanism at `gui/nfc_scan.go:25-27` — "a screen
fetches `Platform.NFCReader()` once at entry and `cmd/emu`'s source returns nil
until a record is pending" — and I rediscovered it empirically before finding
that note.

**Consequence:** cosigner decode, seed entry, key derivation,
`assembleBuildPolicy`, the review screen, template consent, the experimental
warning, engrave, the verify offer and the restore doc have never executed
anywhere except on the machine, by hand. Every mk1 carrying an xpub is ≥2
chunks, so a single-shot tag source cannot complete one even in principle.

**Cause not yet isolated** — two candidates, neither confirmed: the one-shot
`reader()` lifetime versus the once-at-entry acquisition. Isolate before fixing;
a guess here produces a harness that lies, which is worse than none.

> **RESOLVED IN PART — 2026-08-14, fork `5374255`.** Isolated, and it was BOTH
> candidates compounding, not either alone. `bundleGatherFlow` takes
> `Platform.NFCReader()` once at entry (`gui/bundle_flow.go:110`);
> `cmd/emu`'s source returned `nil` unless a record was already pending; and
> `startScanner(ctx, nil)` returns a channel that never delivers
> (`gui/nfc_scan.go:47`). So a tag presented after entry was unreachable
> forever, and one presented before arrived exactly once — one card per flow
> entry, when Trace A needs two and Trace B several.
>
> `nfcSource` is now a queue behind a reader that outlives any single tag, with
> `shNFC.detach()`/`attach()` keeping the genuinely-no-reader state reachable on
> purpose. Verified in a browser with a discriminator rather than a bare
> positive: a card presented mid-gather gives "Descriptor added", and
> re-presenting the same card gives **"Already captured that card."** — the
> gatherer's own dedupe replying, so the second tag really did cross the reader.
>
> **Item 1 of the three above is closed. Items 2 and 3 are NOT** — `gui`'s test
> platform still returns a nil reader (`gui/gui_test.go:445`), so
> `gui/multisig_build_flow_test.go:199` still stops at the gather and the build
> flow still has no end-to-end test on the host. That half stays open under this
> entry, and it is the half that would catch a regression without a browser.
>
> Still true and still blocking a full Trace A walk: a standalone non-chunked
> mk1 is refused by design (`clsSingleMK1Refuse`, host parity), so the walk needs
> properly CHUNKED cosigner cards — most likely fed from
> `sysw_cards_payload.bin` via `shSysw("cards")` rather than hand-presented.

Same family as [[can-a-user-do-the-thing]]: components that each work, joined by
a step nothing exercises.

---

### F-159 — the Build-policy cosigner gather is titled "Engrave Bundle" (owning phase: **`SPEC_multisig_build_repair.md` P1**) `#mnemonic`

Filed 2026-08-13, observed in the emulator. Inside Engrave Multisig → "Build
policy", after the five parameter pickers, the cosigner-gather screen's title
reads **"Engrave Bundle"** — a different program. It is the shared
`bundleGatherFlow` screen used verbatim, title included.

Cosmetic, but on a device where the operator is being asked to scan keys for a
wallet they are about to cut into steel, a screen claiming to be a different
program is exactly the moment to not be ambiguous. Fixed alongside P1's dead-end
work since both are in that flow.

---

### F-160 — the engraved census cannot see an ms1 cut through the standalone codex32 flows (owning phase: **`SPEC_multisig_build_repair.md` P0**) `#mnemonic`

Filed 2026-08-14, from the independent false-PASS review of the walk harness
(`design/agent-reports/walk-harness-false-pass-review.md`, its one Critical).

`shToolpath.strings()` announces a plate only if the string passed through
`validateMdmk`. Two ms1 paths do not:

- `engraveCodex32` (`gui/codex32_polish.go:218`) → `backupSeedStringFlow`
- `unlockEngraveCodex32` (`gui/unlock_session.go:186`) → `toPlate` directly

Both carry `Plate.id == 0`, so a cut ms1 lands in `unattributed`,
indistinguishable from an ordinary seed plate. The census's docs claimed
"md1/mk1/ms1" in five places; **that overclaim is corrected** in the same fold as
this entry, and a gate must now treat `unattributed > 0` as "something was cut
that this census cannot name".

**Does not block S1–S5.** The traces §4.5 gates cut their ms1 through
`bundleEngrave`, which does pass through `validateMdmk` and IS covered. This
binds any later gate that engraves an ms1 outside that path.

The fix is not mechanical, which is why it is filed rather than done blind:
`backupSeedStringFlow` also serves ordinary BIP-39 seed-string backups, which
must NOT be announced, so extending coverage needs a source-tagged variant. Also
owed here: an end-to-end test driving `engraveCodex32` to acceptance and
asserting where it lands — the reviewer wrote one in a scratch worktree and it
is the thing that would keep this boundary honest.

---

### F-161 — ~~the GUI *does* redraw during a cut, and the refresh degrades with `shPace``#mnemonic` `#seedhammer`~~ **CLOSED 2026-08-21** (withdrawn — the claim was wrong) `#mnemonic` `#seedhammer`

Filed 2026-08-14 and **corrected the same day.** The original entry claimed the
GUI stops redrawing during a cut and that *"nothing in the engrave path calls
`ctx.WakeupAt`"*. **Both are false.** The retraction is kept in full rather than
deleted, because how the wrong claim was produced is the useful part.

**What is actually there.** `EngraveScreen.Engrave` (`gui/gui.go`) contains:

```go
if s.job.Status().State == engraveRunning {
	// Update progress twice a second.
	ctx.WakeupAt(time.Now().Add(time.Second / 2))
}
```

**And it works.** Measured over 25 s of continuous cutting, the screen refreshed
at **1.8–2.0 frames/s with the countdown advancing every window** (15:24 →
15:07). `reportProgress`'s "twice a second" comment, which the original entry
called false, is exactly right.

**How the wrong claim was produced**, since that is the transferable part:

1. **Absence judged through a truncated pipe.** The `WakeupAt` search was
   `grep … | head`, and the `Engrave` call sits below the cutoff. The grep
   *did* find widget.go, unlock_kdf.go and run_flow.go — enough to look like a
   complete answer. Exactly [[empty-output-is-not-absence]].
2. **An instrument that caused what it measured.** The load-bearing evidence was
   "0 frames with **every page timer cleared**, so the browser cannot be the
   cause" — done by looping `clearInterval(i)` over every id up to a scanned
   maximum. In a browser `setTimeout` and `setInterval` **share an id space**,
   and Go's wasm runtime schedules its own goroutine wakeups through
   `setTimeout`. That sweep therefore cancels the Go scheduler's timer. The step
   that was supposed to *eliminate* the browser as a cause was the one most
   likely to have *been* it.

The original 8-minute freeze at `16:25` has **not** been reproduced under clean
conditions, and no mechanism for it is known. It is not being re-filed on the
strength of one unrepeatable observation taken with a broken instrument.

**What IS real, and is new — the refresh degrades as `shPace` rises**, measured
on fresh plates, with and without page pollers:

| pace | frames/s, no pollers | with pollers |
| --- | --- | --- |
| 1 | **2.00** | 2.00 |
| 2048 | 1.38 | 1.31 |
| 4096 | 0.87 | 1.13 |

Page polling has no measurable effect — the second column is there to retire
that hypothesis, which was the other half of the original claim. The pace does:
the engrave goroutine yields less often, so the GUI gets fewer chances to
service its 500 ms timer. **Emulator-only by construction** — `shPace` does not
exist on the device — and harmless to an operator, who runs at pace 1 and gets
the full 2 Hz.

**The walk's design survives, for a weaker reason than originally given.**
Keying plate progress off `shToolpath` and consulting the screen only after
motion stalls is still correct: at walk paces the screen refreshes about once a
second, so a read can be up to ~1 s stale, and `walk_trace_a.js` taps to force a
redraw before reading. But it is a *staleness* margin, not a frozen screen, and
`shWaitFor` would in fact have worked given a long enough timeout.

**No hardware question remains.** The original entry asked S6 to check whether
the device shares the defect. There is no defect to share; the device's frame
loop is fed by the same `WakeupAt` that measurably works here.

---

### F-162 — ~~`mk1Gatherer.collected()` returned chunks in RANDOM order`#mnemonic` `#seedhammer`~~ **CLOSED 2026-08-21** (fixed `88c028e`) `#mnemonic` `#seedhammer`

**Closed.** The index walk now mirrors `md1Gatherer`, with the doc comment and
the regression tests md1 got at `3a23dbb`. All four assumptions below were
verified by reading before the change, and all four held — nothing compares mk1
positionally (`equalStrings` has one call site and it is MD1), `mk.Decode`
reassembles by index, `collected()` is only reachable after `complete()`, and
`ParseHeader` rejects an out-of-range index so the walk cannot read a `""` gap.
So it was an ordering and labelling defect, never a funds one, exactly as filed.

Tests were written first and confirmed red, mutation-checked by restoring the
map range, and both assert the *contract* (slot `i` declares `ChunkIndex i`)
rather than a canonical slice. Acceptance: two consecutive six-plate walks now
produce identical census order **and** identical per-plate toolpath digests,
against four different orders in four runs before.

**Firmware-visible**, unlike the rest of that day's emulator work: `gui` is
compiled into the controller, so the image moved +160 bytes flash
(1,342,308 → 1,342,468), RAM unchanged.

The original entry follows.

Filed 2026-08-14, found by running S0's Trace A walk three times and noticing the
census came back differently each time.

**The bug, in four lines.** `gui/mk1_inspect.go`:

```go
g.set[h.ChunkIndex] = s            // offer(): keyed BY INDEX

func (g *mk1Gatherer) collected() []string {
	out := make([]string, 0, len(g.set))
	for _, s := range g.set {      // <-- map iteration; Go randomises it
		out = append(out, s)
	}
	return out
}
```

The chunk index is the map key and is then thrown away at exactly the point the
contract says to use it. `bundleCard.strings` documents itself as *"verbatim
chunk strings **in index order**"* (`gui/bundle.go`).

**Its own sibling shows the intended pattern.** `md1Gatherer.collected()`, same
struct shape, same purpose, does it correctly:

```go
for i := 0; i < g.total; i++ { out = append(out, g.set[i]) }
```

So this is a porting slip, not a design choice.

**Measured, three runs of the identical walk:**

| run | plate order, cards A and B | digests 1–2 |
| --- | --- | --- |
| 1 | A c1, A c2, B c1, B c2 | — |
| 2 | **A c2, A c1**, B c1, B c2 | `ce88ff48…`, `6ec13029…` |
| 3 | A c1, A c2, **B c2, B c1** | `6ec13029…`, `ce88ff48…` |

Run 3's first two digests are run 2's, swapped — the same physical plates cut in
a different sequence. Randomisation is per card, as one `mk1Gatherer` per card
predicts.

**NOT a funds-safety defect, and the Rust check that the standing rule mandates
is DONE and clean.** `mk-codec`'s `reassemble_from_chunks`
(`crates/mk-codec/src/string_layer/chunk.rs:109`) states *"Chunks may arrive in
any order; this function sorts internally"*, concatenates in `chunk_index` order,
and is pinned by `reassemble_accepts_out_of_order_chunks`. The Go decoder is
order-tolerant for the same reason, which is why `mk.Decode(collected)` succeeds
on a shuffled slice and why a restore still works — every plate carries its own
index. **Go-only porting error; no Rust fix is owed**, so the exemption applies
and the fix may land in Go directly.

**What it does break:**

1. The screen says "Card 1 of 3 | **Plate 1 of 2**" while engraving an arbitrary
   chunk. The label is a claim about which chunk this is, and it is wrong half
   the time.
2. `bundleCard.strings` violates its documented contract, which anything
   downstream is entitled to rely on.
3. **It makes §4.5's byte comparison order-flaky** — the gate S0 exists to
   build. `engravedRecorder.Strings()` documents order as load-bearing: *"a set
   that arrives in the wrong order is a different restore than the one the walk
   asked for."* A later gate comparing ordered output would flake roughly half
   the time on a 2-chunk card, and more often as chunk counts rise.
4. `mk.Decode` is fed an unordered slice today. It tolerates that by design; if
   it ever gained an order assumption this becomes silent corruption.

**Fix**, matching the sibling exactly, plus the test that would have caught it —
a gather offering chunks out of order must still collect them in index order.
Deliberately NOT done in the commit that found it: it is funds-adjacent and
belongs to a gated cycle. Worth checking `collected()`'s other consumers in the
same pass, since it feeds `mk1DisplayFlow` too.

---

### F-163 — ~~S3's gate is a whole-tree `grep` and S0 already broke it`#seedhammer`~~ **CLOSED 2026-08-21** (fixed `2b7fc96`) `#seedhammer`

Filed 2026-08-14 by the parallel-implementation review
(`design/agent-reports/parallel-implementation-feasibility.md`), controller-verified.

S3's gate line is `grep -rn TYPED-ONLY --include='*.go'` returns **0**. The plan
measured **9** occurrences on 2026-08-13. On 2026-08-14 S0's own
`cmd/emu/embed_confinement_test.go` (`3009f22`) added a **10th** — in a comment
citing `TYPED-ONLY` as the archetype of a hand-maintained list that goes stale.
Measured now: **10**, in `gui/multisig.go` ×4, `gui/bip85.go` ×2,
`gui/singlesig.go` ×2, `gui/multisig_build.go` ×1, `cmd/emu/embed_confinement_test.go` ×1.

So S3 can no longer satisfy its own gate without editing a file S0 owns, and the
two stages never ran concurrently — one agent, one day apart, and they still
collided. **A whole-tree `grep` is a shared resource**: it makes every stage's
acceptance depend on every other stage's text.

Fix, when S3 opens: scope the gate to the code it governs —
`grep -rn TYPED-ONLY --include='*.go' gui/` returns 0 — and have S3 retire the
`cmd/emu` citation in the same change, since that comment is *about* the phrase
and reads wrong once the phrase is gone. Do not widen the count and re-measure;
that repeats the defect one stage later.

Not urgent: S3 has not opened. It is filed rather than fixed because the plan
text is the thing that needs editing, and the plan is a gated artifact.

### F-164 — ~~S0's gate names eight tests; three of them exist under different names, two do not exist`#seedhammer`~~ **CLOSED 2026-08-21** (fixed `2b7fc96`) `#seedhammer`

Filed 2026-08-14, found by checking S0's gate against the tree rather than
against the plan's own prose. Sibling of [F-163]: a gate written in terms that
have drifted from the code.

The plan's S0 gate is "All eight tests pass." Resolved with
`git grep -l "func <name>" -- '*_test.go'`:

| plan's name | in the tree |
| --- | --- |
| `TestEmbeddedPayloadsAreStructurallyConfined` | **renamed** — `TestEveryEmbeddedPayloadIsStructurallyConfined` (`cmd/emu/embed_confinement_test.go`) |
| `TestCosignerPayloadCarriesTheTracesCards` | **renamed, and split in two** — `TestSyswCardsPayloadCoversEveryStagesWalk` + `TestSyswCardsPayloadMatchesItsDigest` |
| `TestWalkHarnessDrivesAndExtracts` | **no test of that name.** The harness itself is done and wired — `shTap`/`shPress`/`shRelease`/`shPace`/`shSysw` (`cmd/emu/walk_js.go:48,63,70,102,112`), `shScreen` (`screen_js.go:48`), `shToolpath` incl. `strings()` (`toolpath_js.go:84,80`) — and the proof is the walk running, not a Go test |
| `TestBip383SortedMultiScriptMatchesPublishedVectors` | EXISTS (`address/bip_vectors_test.go`) |
| `TestBip67SortedMultiKeyOrderScriptAndAddress` | EXISTS |
| `TestBip141NestedSegwitScriptDiffersFromLegacy` | **superseded** by `TestBip143NestedP2wshScriptPubKeyMatchesPublishedVector` — BIP-141 publishes no vectors (`RECON_bip_vectors_S0.md`) |
| `TestOracleHarnessRefusesVendoredTestdata` | ABSENT — D5 undone |
| `TestOracleHarnessPinsBySourceCommit` | ABSENT — D5 undone |

So the gate is **6 of 8 satisfied, and only 3 of the 8 names resolve as
written**. The risk is not that the work is missing — it is that anyone
checking this gate by grepping the plan's names concludes S0 never happened,
or writes a duplicate test alongside the one that already covers it.

Fix when S0 closes: rewrite the gate list against the tree, and prefer naming
the PROPERTY plus the file over a bare test identifier, so a rename does not
silently invalidate a gate. Do not rename the tests to match the plan — the
tree's names are better, and the plan is the thing that drifted.

### F-165 — D4 rescoped: it constrained a receiver no walk reaches (owning phase: **`SPEC_multisig_build_repair.md` S0**) `#seedhammer`

Filed and rescoped in the same change, 2026-08-14 (`2b7fc96`). Third gate this
day that could not fail, after [F-163] and [F-164].

S0 D4 required "the frame receiver keeps its existing security properties" —
one pinned origin, flat filenames only, resolved-path re-check. Measured
(`design/agent-reports/s0-tail-file-sets.md`, controller-verified): the walk
harness D3 built **posts no frames at all**. `cmd/emu/walk_trace_a.js` makes no
network call of any kind; the only match for `fetch|XMLHttpRequest|toDataURL|POST`
in it is the word "fetched" inside a prose comment. Screenshots come from the
Playwright driver, not from the emulator pushing to a receiver. The only frame
receiver in either repo is `design/journeys/shot_server.py` — in
**mnemonic-engrave**, not the fork — used by the manual PDF-journey builder.

So no code change could satisfy or violate D4. Split in the plan into:

- **(a) a standing constraint**, inherited by whichever stage ever adds a frame
  receiver — that stage owns proving the three properties. Costs S0 nothing.
- **(b) the real S0 item:** verify `shot_server.py`'s three properties hold
  *today* rather than trusting its docstring. A read plus a test in
  mnemonic-engrave; touches no fork file. **Closing D4 means doing (b).**

Pattern worth naming across F-163/164/165: all three were gates whose subject
had drifted out from under them, and none was found by reading the plan — each
took running the check. A gate that has never executed is a hypothesis.

### F-166 — the fork's md decoder cannot read a PATHLESS origin; the Rust primary can (owning phase: **`SPEC_multisig_build_repair.md` post-S0 / its own cycle**) `#seedhammer` `#mnemonic`

Filed 2026-08-14, found by S0 D8's coverage catch-up — and *only* by it. The
re-pin's provenance half was green; this surfaced the moment the primary's new
vectors were actually exercised.

`md/testdata/vectors/sh_wpkh` fails in this fork:

    decodePayloadValidated("md1yqpqqxpsq258xsks3kh0ye")
      -> md: missing explicit origin        (md/md.go:893)

Its descriptor declares a pathless shared origin —
`"path_decl":{"tag":"Shared","data":"m"}` — i.e. depth-0 `m`. The Rust primary
gained exactly this in the release D8 re-pins to: `5a0a4f41`, *"release:
md-codec 0.42.0 + md-cli 0.13.0 — **pathless**/dead-card partial-decode"*.

**Rust-primary status: CONVERGENCE, not a lead.** The primary is already
correct and ships the vector; the Go port is behind. Per the standing rule that
makes this exempt from "land it in Rust first" — but the rule's second half
still binds, and it was checked: this is not a Go-only porting slip masking a
Rust defect, because the Rust side is the thing that *has* the feature.

Scope: teaching the Go decoder pathless origins is a feature with its own
tests, not a re-pin, so it was deliberately NOT patched inside D8. The vector
data is vendored and `md/testdata_test.go` names it in a comment, so the gap
reproduces in one line — add `"sh_wpkh"` to `singleStringVectorNames` and run
`go test ./md/`.

Not urgent for this plan: no stage of the multisig-build-repair cycle engraves
a pathless descriptor. It matters for anyone feeding the device an md1 produced
by a current primary.

**Method note worth keeping.** This is the second time today that vendoring a
published/primary corpus and *running* it found something no amount of reading
would. The first was BIP-141 publishing no vectors at all. Coverage catch-ups
are not bookkeeping.

### F-167 — D5's gate record carries a seed DIGEST, not seed words: a departure from the plan's text (owning phase: **`SPEC_multisig_build_repair.md` S0, folded at close**) `#seedhammer`

Filed 2026-08-14 alongside the D5 implementation (`1333cc4`), so the departure
is visible rather than discovered later by someone diffing the plan against the
code.

The plan's D5 says the harness must record "the full input tuple (template, n,
k, slot order, fp choice, per-slot origins, **seeds**)" so that "same inputs" is
reproducible rather than remembered.

**Implemented as `SeedRef{Label, Digest}` instead** — a label naming the seed's
source (`"payload:card0"`, `"typed:trace-a"`) plus the first 16 hex chars of
`sha256(words)`. `oracle.NewSeedRef` does not retain the words, and
`TestGateRecordCarriesCommitsAndTheInputTuple` asserts that no seed word
reaches the marshalled record.

**Why.** A gate record is written to disk, committed, and pasted into CI logs
and commit messages. One containing seed words is key material with none of the
handling that implies. The reproducibility the plan actually asks for —
proving two runs used the same seed, and being able to re-select a known test
seed — is fully served by a label plus a digest.

**Not a silent substitution.** It is weaker in exactly one way: a record alone
can no longer *reconstruct* a run's seed, only identify it. For the walk's
known test seeds that is no loss, since the label names them. If a future gate
genuinely needs reconstruction, that is a decision to take deliberately and
with the handling rules written down, not a default.

**Action at S0 close:** fold this wording into the plan's D5 text so the plan
and the code agree, or overrule it explicitly. Do not leave the plan saying
"seeds" while the code records digests.

### F-168 — the only automated walk is a BUNDLE-ENGRAVE walk, and S0's evidence line calls it Trace A (owning phase: **`SPEC_multisig_build_repair.md` S0, folded at close**) `#seedhammer`

Filed 2026-08-14 from `design/RECON_S1_S6_walk_gates.md`.

`cmd/emu/walk_trace_a.js` selects exactly two programs — `goTo("LoadPayload")`
at `:169` and `goTo("EngraveBundle")` at `:180`. `engraveBundle` dispatches to
`bundleFlow` (`gui/gui.go:1816-1817`). The plan defines Trace A at `:173-177` as
`Engrave Multisig → Build policy → template → n → k → self-slot → fp → cosigner
review → seed entry → policy review → form → EXPERIMENTAL → mode → engrave →
restore doc`. **None of those eleven screens appear in the walk.**

So the file name, its header ("Trace A as a script"), and S0's gate row 3 ("a
six-plate Trace A run in ~165 s", plan `:355`) all certify a journey that did not
run. This is F-164 one level up: not a stale identifier, a stale claim about
*which journey* the evidence came from.

**S0 D3 is not in question.** The shapes it delivered do drive the build flow —
see §5 of the recon. What is wrong is the label, and the inference every later
gate draws from it.

**Action at S0 close:** say in one line what journey the script actually walks,
and correct row 3 so it does not certify Trace A. Carry the recon's I4 across
too — the census is cumulative for the page session (`cmd/emu/engraved.go`), so
**one walk per page load** is a standing constraint. It currently fails closed
twice by accident (`strings.length === plates` is an equality, and
`oracle.ParseWalk` requires `len(digests) == len(strings)` while `digests` is
per-run); write it down so a future `>=` does not convert that into a fail-open.

### F-169 — S1–S5 each need their own walk, and none exists; the shared script cannot tell which flow it is in (owning phase: **`SPEC_multisig_build_repair.md` S1**, gating) `#seedhammer`

Filed 2026-08-14 from `design/RECON_S1_S6_walk_gates.md` (C1, I1).

`buildMultisigPolicyFlow` has one production caller, `gui/multisig.go:55`,
behind the "Build policy" choice at `gui/multisig.go:45`, dispatched at
`gui/gui.go:1822-1823`. The existing walk takes the sibling case. Per
`design/agent-reports/plan-wide-file-touch-matrix.md`, **all five of S1–S5 edit
`gui/multisig_build.go:39-193`** — so every one of their "by emulator walk"
clauses is a gate that cannot execute a line of what its stage changed.

**And a walk written by editing this script's `goTo` target would look
identical.** Every needle it uses is ambiguous: `"First card from where?"` is in
three production flows (`gui/bundle_flow.go:25`, `gui/multisig.go:76`,
`gui/multisig_build.go:54`); `"Choose engraving"` has six sites; and the gather
title is `layoutTitle(…, "Engrave Bundle")` at `gui/bundle_flow.go:155` — inside
the *shared* gatherer, so it reads "Engrave Bundle" even from Build policy. The
only program identification in the whole script is the carousel match at entry.

A **flow-identifying needle that exists in one flow only** is therefore
mandatory. **Three exist today** — corrected after review (I-4), because the
original text here said one arrives only after S2 fixes D-4, which argued for
deferring the scaffolding past S2. Each is a single production site
(`git grep -F … -- 'gui/*.go' | grep -v _test`):

    gui/multisig_build.go:300   Lead: "Choose policy type"
    gui/multisig_build.go:376   Lead: "How many keys (n)?"
    gui/multisig_build.go:394   Lead: "Which slot is your key?"

plus `gui/multisig.go:44` `Lead: "Supply or build a policy?"`, unique to
`engraveMultisigFlow`. **Decoy, named so nobody reaches for it:** `Title:
"Engrave wallet policy"` / `Lead: "Which md1?"` is two sites —
`gui/multisig_build.go:121` and `gui/singlesig.go:94`.

Owning phase is **S0b if that stage is created** (see
`design/agent-reports/s1-walk-gate-judgement-review.md`), otherwise S1 as the
first stage that needs it. Pairs with **F-174** — zero `shNFC.present`.


**✅ RESOLVED 2026-08-14 (S0b, fork `8345b0e`).** `cmd/emu/walk_build_policy.js`
reaches the Build-policy cosigner gather via `Engrave Multisig`, and
`cmd/emu/needle_test.go` machine-checks that the needles it anchors on have
exactly ONE production site each — the counts in this entry are now a gate, not
a comment, and the two decoys are pinned at 2 and 3 so a drift toward uniqueness
is a deliberate promotion rather than an accident.

**The ambiguity was worse than this entry claimed, and it was measured rather
than argued.** Both flows were driven in the emulator and the gather screen is
CHARACTER FOR CHARACTER identical:

    via Build policy    EngraveBundlemd1descriptors:0mk1keys:0Scanacard,orDone.
    via Engrave Bundle  EngraveBundlemd1descriptors:0mk1keys:0Scanacard,orDone.

So the title reads "Engrave Bundle" while the operator is inside Build policy,
and a driver checking "did I reach a card gather" passes in the WRONG flow.
Driving the sibling program produced `needlesSeen: []`, the positive proof that
the needle discriminates.

**Mutation-proved:** a second real `ChoiceScreen{Lead: "Choose policy type"}`
added to `gui/singlesig.go` — the mutant COMPILES (`go build` exit 0, so not a
build-failure false proof) — turned the gate red naming both sites; green on
restore. Minor citation drift folded: `"Which md1?"` is now
`gui/multisig_build.go:122` and `gui/singlesig.go:95`.

### F-170 — the walk asserts a plate COUNT where the plan requires a census derived from the input tuple (owning phase: **`SPEC_multisig_build_repair.md` S1**, gating) `#seedhammer`

Filed 2026-08-14 from `design/RECON_S1_S6_walk_gates.md` (C3, I3).

    cmd/emu/walk_trace_a.js:274
    ok: census.strings.length === plates && census.unattributed === 0,

`CARDS` (`:66-76`) holds the exact six strings expected and is used **once**, to
present chunks at `:198`. It is never compared to the census. A walk that
engraved six *wrong* strings is green; the header defers the comparison to a
human ("compare run()'s census against `go run ./cmd/buildpayloadcards`").

The plan's own §3 preamble (`:193-201`) forbids this: *"A walk's expected
artifact census MUST derive from the recorded input tuple, never from what the
walk produced… The script computes how many md1 chunks, mk1s and ms1s the inputs
REQUIRE and fails when fewer arrive."* Measured: no such computation exists, and
`plates` is a parameter defaulting to `6` (`:151`).

Each stage's plate count differs — Trace A on the build path cuts md1 chunks
plus the self mk1 plus (in full mode) an ms1, not six mk1 chunks; Trace B is 6–9
plates (`:757`). So the derivation is not a nicety, it is the only way a
per-stage walk can have a count at all.

**S3 owns this too, added after review (I-2).** The plan's preamble exempted S1
*and S3* from the derived census on the premise that both "end at a screen, not
an engrave". Measured: `bundleEngrave` is `gui/multisig_build.go:168` and the
restore doc S3's gate reads is `gui/multisig_build.go:191` — **after** it. Any
walk satisfying S3 has cut plates, so S3 could have engraved wrong artifacts and
passed on a screen string. Preamble corrected; S1 alone is artifact-free.


**✅ RESOLVED 2026-08-14 (S0b, fork `c94c135`).** `oracle.DeriveExpected` computes
the artifact set from the recorded input tuple by invoking the primary
toolchain, and `oracle.CompareCensus` compares it to the census byte for byte
and IN ORDER. `plates = 6` is gone: six now falls out of three seeds × two
chunks, computed by `mk`. Exercised against S0's committed record —
*"derived 6 artifact(s) from the recorded inputs; all matched the engraved
census"*.

**Seen to go red**, all four: a ONE-CHARACTER flip in plate 2 (the refusal names
the plate and prints both strings in full); a census one plate short; reordered
plates; and `CompareCensus(nil, nil)`, which FAILS rather than passing
vacuously — as does an unknown expectation kind, so a typo cannot derive an
empty set that then "matches".

**Bonus the derivation forced:** the recorded ORIGIN is now checked rather than
merely recorded. The deriver computes the path its own template implies and
refuses on disagreement, so a record whose origins drifted from its key material
cannot pass.

### F-171 — nothing invokes the pinned `md`/`mk`/`ms`, so S2's and S5's byte-comparison gates are unimplemented (owning phase: **`SPEC_multisig_build_repair.md` S2**, gating) `#seedhammer`

Filed 2026-08-14 from `design/RECON_S1_S6_walk_gates.md` (C4).

S2's gate: *"the current primary BUILDS an md1 from the same inputs and the
strings are equal"*. S5's: *"each engraved ms1 must equal `ms encode --hex <that
master's entropy>`"*.

    $ git grep -nE 'exec\.Command\("(md|mk|ms)"|cargo/bin/(md|mk|ms)' -- '*.go'
    (no production hit)

`oracle` resolves all three to source commits and stops there; as of `88d43c7`
its only importers are `cmd/gaterecord` and `cmd/emu`'s anchor test, neither of
which runs an oracle. Both gates are **unimplemented, not merely unrun** — the
distinction matters because "we'll do it at the gate" assumes a mechanism that
is not there.

Note `oracle.CheckDataSource` refuses any `testdata` path by design, so the two
`gui` tests reading `md/testdata/vectors` (`bundle_testdata_test.go:43`,
`md1_gather_test.go:30`) are fixtures and cannot stand in for this.


**✅ RESOLVED 2026-08-14 (S0b, fork `c94c135`).** `oracle/expect.go` invokes the
pinned binaries; `TestS0CensusMatchesTheDerivedExpectation` is the gate. The
chain is entirely primary-toolchain — `ms derive` for seed → fingerprint +
account xpub, `mk encode` for xpub → mk1 chunks — so nothing re-implements a
derivation.

**It could not be closed as filed, and the blocker was in the PRIMARY.**
`mk encode` drew `chunk_set_id` from the OS CSPRNG per call, so three runs on
identical inputs emitted three different cards and byte-identity against chunked
mk1 was permanently unsatisfiable. Per the Rust-primary rule that was fixed
upstream first, as a conformance fix rather than a wire-format change —
mnemonic-key `a38a908` (mk-codec 0.5.0), since SPEC §2.5 already required
encoders to "reuse the same value for all subsequent re-encodings of the same
card" and a stateless encoder cannot do that from entropy.

**A second gap, and a real product defect rather than gate scaffolding.**
`ms derive --template` offered single-sig templates only and took no literal
path, so seed → MULTISIG account xpub had no oracle at all: the one tool that
turns a seed into an account xpub could not serve the format the constellation
exists to back up. Closed upstream as ms-cli 0.15.0 (`ddfa497`) with BIP-48
`bip48-p2wsh` / `bip48-p2sh-p2wsh`, so an operator names their SCRIPT TYPE
instead of knowing that native segwit multisig lives at `m/48'/0'/0'/2'`.

Both oracles re-pinned (`04f2716`, `c94c135`). The old `ms` pin was additionally
found to be an inaccurate attestation — it named commit `bf77f89` with version
`ms 0.14.0`, but that commit's source declares `0.14.1`, so the binary was never
built from the commit it named.

### F-172 — ~~S3's restore-doc gate has nothing to read on the template branch~~ **RESOLVED 2026-08-15 by S3 — but the filed cause was only HALF of it** (owning phase: **`SPEC_multisig_build_repair.md` S3**) `#seedhammer`

> **RESOLUTION 2026-08-15. Read the second cause before citing this entry as
> closed — closing it on the filed cause alone would put a false record on
> disk.**
>
> The filed cause is real: picking **"Full policy md1"** is *necessary*, because
> the template-only form skips the restore doc. It is **not sufficient**, and
> nothing in this entry or in the plan said so.
>
> **The second cause, measured while running S3's gate for the first time.**
> `gui/multisig_restore.go`'s `desc4Display` — the site SPEC §2.2 D-3 and §4.4
> both name as the one that matters — sits on the **display-only** branch. Every
> md1 the Build flow authors is bip380-expressible, so a full-policy build lands
> on the **`expandOK`** branch instead, **where the script type was named nowhere
> at all**. So the `scriptName` fix the plan describes would have left S3's walk
> reading a restore doc with no type line — indistinguishable from a broken
> screen, and a gate that fails for a reason nobody would have diagnosed.
>
> Fixed in `7bdd1f3` by putting the type line on **both** branches. The walk then
> read `Type: P2SH-P2WSH 2-of-3 multisig (sorted)` and passed, with a negative
> control proving it discriminates.
>
> **The lesson, and it is the one this cycle keeps paying for:** the spec named a
> call site it had *analysed*, not the site the flow *reaches*. Identical shape
> to "the four `TYPED-ONLY` comments" turning out to be nine. **A gate that has
> never been run is a hypothesis** — this cost about an hour to find by trying
> it, and was invisible to every reading before that.

Filed 2026-08-14 from `design/RECON_S1_S6_walk_gates.md` (I2).

S3's gate is an emulator walk of an `sh(wsh)` build *"showing `P2SH-P2WSH` on the
restore doc"*. `multisigRestoreDocFlow` does render to a screen
(`gui/multisig_restore.go:54-58`), so `shScreen()` can read it — **but
`gui/multisig_build.go:185` skips the restore doc entirely when the operator
picked the template-only form at `:120-142`.**

So the gate is satisfiable only if that stage's walk picks **"Full policy md1"**,
and nothing says so. A walk down the other branch reaches the end with the gate's
subject never drawn, which reads exactly like "the screen did not say it".

### F-173 — RULED 2026-08-14 (`0..n`): Trace A could not complete on the payload S0 delivered, once S1's unconditional payload feed landed (owning phase: **`SPEC_multisig_build_repair.md` S1**) `#seedhammer`

Filed 2026-08-14 from `design/agent-reports/s1-walk-gate-judgement-review.md`
(C-1), independently re-measured by the controller before filing. **Independent
of the walk-gate findings (F-168–F-172) and survives every option proposed for
them** — it is unsatisfiable with or without a walk.

Measured, not inferred:

- The delivered cosigner payload holds **nine `ClassMDMK` records forming FOUR
  cards** — A@0, A@1, B@0, C@0 (`cmd/buildpayloadcards/main.go:53-58`; the
  record count is `TestSyswCardsPayloadCoversEveryStagesWalk`'s own log line).
- S1's implementation note is to replace the single `syswOffer` seeding with
  **every** `ClassMDMK` record fed through `bundleGatherFlow`'s `offer()`, and
  the gather has no per-card decline — only `dropPending` for an incomplete
  chunk set (`gui/bundle_flow.go:127`), never removal of an added card.
- `gui/multisig_build.go:61` calls `buildCosignerCards(cards, p.N-1)`, whose last
  check is `if len(out) != want` at `gui/multisig_build.go:268`, and the flow
  then shows *"Gather exactly %d cosigner key cards (and no md1)."*
- `multisigNChoices()` is `{"2","3","4","5"}` with `multisigNFor(idx) = idx+2`
  (`gui/multisig_build.go:310`), so `want = n-1` ranges over **1..4**.

So four cards arrive into `want` open slots and the build **refuses for every n
except 5**. Consequences:

- **S2's gate is unsatisfiable.** "Trace A completes end to end: engrave" cannot
  happen at n=3 on this payload.
- **S1's gate has a third outcome nobody assigned.** Its disjunction is "either
  the flow completes an engrave, or D-1 reproduces and is captured as a failing
  test"; the walk instead ends on a legitimate **over-supply refusal**, which
  S1's own test 6 (`TestBuildRefusesMoreCardsThanOpenSlots`) makes the specified
  behaviour.
- The only n this payload admits under an unconditional feed is **5**, which is
  neither Trace A (2-of-3) nor Trace B (3-of-4).

**Why S0's gate did not catch it.** `TestSyswCardsPayloadCoversEveryStagesWalk`
asserts the payload carries *at least* the cards each stage needs — it has a
`len(mdmk) < 8` floor and no ceiling. "Enough cards" and "a usable number of
cards" are different properties, and only the first was ever gated.

**RULED by the operator, 2026-08-14: *"Available key count could be 0 to n."***

Wider than either option put to them (per-card accept/skip; run the walks at
n=5), and wider on purpose — it is a **property of the design**, not a
workaround for this payload. **The payload may carry anywhere from zero to `n`
cosigner cards, and no stage may assume it carries exactly `n-1`.** Folded into
the plan at §1 as a standing ruling.

What it changes, each recorded where the old assumption lived:

- **The exact-count check moves from the FEED to the ASSEMBLED set.** Over-supply
  is *normal* — the delivered payload carries four cards for a 2-of-3 — so it is
  resolved by **selection**, and only a selection that still does not fit
  refuses. `buildCosignerCards`'s `if len(out) != want` stays; what reaches it
  changes.
- **S1's test 6 `TestBuildRefusesMoreCardsThanOpenSlots` is re-scoped.** Written
  against the payload feed it pins the very behaviour that makes Trace A
  unreachable.
- **S1's test 7 extends to ZERO cards.** An empty payload is now a legitimate
  input, and a build that dead-ends on it with no named route is the same defect
  at the other end of the range.
- **New S1 test 8, `TestPayloadCardCountIsIndependentOfN`** — the ruling as a
  test, over the product of `n ∈ 2..5` and `0..n` cards, every cell assembling or
  refusing **by name**. Mutation: restore the feed-side exact-count refusal and
  the n=3 rows go red.
- **Upper bound is `n`, not `n-1`, and that is not a typo to correct.** A payload
  carrying a card for *every* slot includes one that may be the operator's own —
  which is precisely S4's `both` case (a card whose key derives from a payload
  seed). The delivered payload already contains that pair: card `A@0` and the
  single `ClassMnemonic`, both master A. S4 should read this ruling as
  confirming its model rather than as a new case.

No stage may close having assumed `n-1`. **Everything below is the record of how
it was found; the ruling above is what binds.**

### F-174 — a stage-gate build walk must assert ZERO `shNFC.present` calls (owning phase: **`SPEC_multisig_build_repair.md` S0b/S1**, gating) `#seedhammer`

Filed 2026-08-14 from the same review (I-1). The recon's own remedy table
prescribed `shSysw` **+ `shNFC.present`** for the build-flow gather — the harness
substitution that makes S1's gate pass without S1's feature.

S1 delivers *"the payload supplies the whole cosigner set"* and its test 3 says
**zero scans**. A build walk that completes its gather by presenting chunks over
the emulated reader is green whether or not `takeAll` exists, and phase-1
hardware has no reader at all, so the affordance is the harness's alone.

**Fix:** count `shNFC.present` calls in the harness, assert zero for any
stage-gate build run, and make that assertion one of the seen-to-fail mutations.
An NFC-fed build run is a driver smoke test and must be labelled as one — and it
is separately the cheapest route anyone has to **reproducing D-1**, which S2's
test 1 requires to fail on unfixed code and which `SPEC §2.2:95` still records as
NOT YET REPRODUCED.


**✅ RESOLVED 2026-08-14 (S0b, fork `8345b0e`).** `nfcSource.presented()` counts
records across the reader for the session and is exposed as
`window.shNFC.presented()`; `walk_build_policy.js` asserts ZERO at entry and
again at the gather.

**No reset is exposed, deliberately** — a counter a driver can zero just before
asserting is a gate that always passes.

**Mutation-proved live:** control green at 0; presenting one record throws; and
`shNFC.clear()` does NOT launder it back to green, which is the assertion's
whole integrity. Also unit-covered in `cmd/emu/nfc_presented_test.go`.


### F-175 — an artifact-free stage cannot produce a gate record at all (owning phase: **`SPEC_multisig_build_repair.md` S1**, gating) `#seedhammer`

Filed 2026-08-14 from S0b, and **measured rather than reasoned about**:

    $ go run ./cmd/gaterecord -stage S0b -walk <a walk with an empty census> …
    gaterecord: oracle: the walk did not finish green, so it cannot anchor a
    gate record: the census is empty, so nothing was engraved to anchor to

`ParseWalk` refuses an empty census (`oracle/record.go`), by design — a record
with nothing in it is bound to nothing. But the plan's own §3 preamble says
**S1 "ends at a screen, not an engrave"**, and S0b's driver engraves nothing
either. So the stage whose gate is a SCREEN assertion has no way to emit the
artifact the S0 D5 machinery makes mandatory.

Not a defect in S0b: S0b's mechanisms are exercised against S0's record, which
is what its gate asks for, and its own evidence is committed as tests. It is S1
that first has to answer this.

**✅ RULED 2026-08-15 (operator, accepting the fable recommendation): option
(b) — recordless, with the substitute NAMED, and scoped to the D-1 arm only.**
Folded into the plan at S1's gate. The key scoping point, which this entry
missed when filed: S1's gate has **two arms** ("either the flow completes an
engrave, or D-1 reproduces and is captured as a failing test"), and F-175 bites
only the second. On the engrave arm S1 produces artifacts and takes a record
like any other stage.

On the D-1 arm the named substitute is the walk script, the single-site needle,
`shNFC.presented() === 0`, and the captured failing test — and the plan now says
a stage may pass with **neither a record nor that substitute** never. Option (a)
(a schema variant anchoring a screen assertion) waits for a SECOND artifact-free
stage: one instance does not justify a schema bump plus a new definition of
"green". Option (c) was rejected outright — an engrave tail re-couples S1 to S2
and undoes the plan's own staging.

Original options preserved below.

**Options, as filed:** (a) a record variant that anchors a screen
assertion instead of a census — needs a schema bump and a clear rule for what
makes such a walk "green"; (b) leave artifact-free stages recordless and say so
explicitly in the plan, accepting that their evidence is the committed test plus
the walk script; (c) give S1's walk an engrave tail so it produces artifacts,
which changes what S1 is. **(b) is the cheapest and (a) the most honest**; the
one thing that must not happen is a stage quietly passing with neither a record
nor a named substitute.

Cross-ref: `TestS0GateHasARecord` demands a record for **S0 only**, so nothing
is red today — which is exactly why this would go unnoticed until S1's gate.


### F-176 — ~~`md` cannot author per-key origins, so S5's divergent md1 byte-comparison has no producer~~ **WITHDRAWN 2026-08-15: the premise is FALSE, measured** (owning phase: **none — nothing blocks S5**) `#seedhammer` `#cross-repo`

**✅ WITHDRAWN 2026-08-15, before any upstream change was made.** The implementer's
first act was to run the mechanism rather than the three failing invocations
below, and **`md encode` authors per-key origins today** — not through a flag,
through the **template placeholder syntax**, which the audit never tried. The
three measurements in this entry each probed a *different* mechanism and all
three failures are correct behaviour for what they asked.

Measured against the pinned oracle `md 0.13.0` (commit `5a0a4f41`, the exact
`oracle/pins.json` pin), reproduction verbatim:

    $ md encode "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))" \
        --key "@0=xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf" \
        --key "@1=xpub6EAMBJLn1jiquajTsNRkZXU1oKnA4WJMNvcz4FRR4QmFKdfHxJVvfRLoysWfcc16AMTR4CoMD8UNjvs9JtbsLeuLwpTczgq8zuuERnp8YZF" \
        --group-size 0 --force-chunked --json

emits 4 chunks; `md decode --json` on them returns
`path_decl {tag: "Divergent", data: ["m/48'/0'/0'/2'", "m/48'/0'/1'/2'"]}`,
each path on the right `@i`; and `md verify … --template … --key @0= --key @1=`
over the same four strings prints `OK`, exit 0.

**The gate is not merely satisfiable — it is already PROVEN byte-identical.**
The fork's `md.EncodeMultisig` in `OriginDivergent` mode over the same two
xpubs and the same two origins emits the **same four strings, character for
character** (`stub=4cb7f1a8`, 4/4 chunks), so S5's "the primary BUILDS an md1
from the same inputs and the strings are equal" relation holds today with the
pinned binary and needs no upstream change, no release, and no re-pin.

Why the entry got it wrong — the mechanism is in `md-cli`'s own source and
`grep` would have found it: `make_path_decl` (`crates/md-cli/src/parse/template.rs:495-510`)
emits `PathDeclPaths::Divergent` whenever the per-`@i` inline origins are not
all equal, and `emit_pathless_advisory`'s doc-comment
(`crates/md-cli/src/cmd/encode.rs:180-183`) *names the feature in prose*:
"an inline per-`@N` explicit origin (e.g. `sh(sortedmulti(2,@0/48'/0'/0'/1'/<0;1>/*,…))`)
with no `--path` FULL-decodes (exit 0)". Three probes of the wrong surface
(`--key` with a bracketed origin; a concrete-key descriptor; `--help` for a
flag) plus one probe of a template that *carried no inline origins*
(`--path` omitted → `Shared "m"`, which is right for that input) read as
consensus. **Absence of a flag was mistaken for absence of the capability.**

Behaviour of the real mechanism, all measured, so the questions the upstream
change was going to answer are already answered:

- **`--path` + inline divergent origins:** `--path` wins and flattens to
  `Shared` — documented on the flag itself
  (`crates/md-cli/src/main.rs:93-95`: "Override the inferred origin path with a
  single shared path (flattens Divergent mode to Shared)").
- **Partial specification** (an origin on `@0`, none on `@1`): accepted; the
  unspecified `@i` gets a depth-0 empty origin and the encoder emits the
  pathless advisory on stderr.

**Residual, and it is sugar, not a blocker:** `md encode --origin @i=<path>`
would be a nicer surface than string-splicing origins into the template, and
`md`'s FOLLOWUPS may want it on ergonomic grounds. It is **not** gating, has no
S5 lead time, and costs a cross-repo manual lockstep
(`mnemonic-toolkit docs/manual/src/40-cli-reference/42-md.md`, gated by
`tests/lint.sh flag-coverage`) — so it is not worth a release chain on this
cycle's schedule. **Do not re-file it as gating.**

The recorded fallback (decode-equivalence as a named gate deviation) is
**moot** and must not be used: full string equality is available.

Original entry preserved below, as the record of how a false premise formed.

---

Filed 2026-08-15 from `design/agent-reports/fable-s1-s6-assumption-audit-2026-08-15.md`. **This is the S0b failure shape caught before it cost a stage** — a gate written against a mechanism that does not exist — and it is the only S5 dependency with external lead time.

Measured against `md 0.13.0`, three ways, all failing:

- `--key "@0=[fp/48'/0'/0'/2']xpub…"` → `base58check decode` error;
- a concrete-key descriptor → `template contains no @i placeholders`;
- `md encode --help` exposes no per-key origin flag at all; omitting `--path` yields `path_decl {tag: Shared, data: "m"}` (via `md decode --json`).

**The codec is fine; the CLI is the gap.** md-codec decodes Divergent correctly — the fork's divergent encode round-trips through `md decode --json` with `tag: "Divergent"` and both paths present. So S5's gate ("the current primary BUILDS an md1 from the same inputs and the strings are equal") is unsatisfiable for Trace B, which is the plan's flagship wallet.

**Decision: file the upstream flag now, Rust-first.** `md encode --origin @i=<path>` (repeatable, pairing with the existing `--key @i=`), landed in `descriptor-mnemonic` with test vectors, then released, then re-pinned in `oracle/pins.json`. Filing costs about an hour; discovering it at S5 costs the stage — and the release + re-pin chain is exactly the lead time that cannot be compressed later.

**Named fallback, if the flag has not landed when S5 arrives:** compare Trace B's md1 by DECODE equivalence (`md decode --json` field-by-field against the input tuple) rather than by string equality, and record that in the gate record as an explicit deviation from §1a's full-string-equality rule. A deviation that is named and recorded is a different thing from a gate quietly downgraded, and only the first is acceptable.

Cross-ref: §1a now rules **full string equality for all three artifact classes**; this is the one place that rule currently cannot be honoured.

### F-177 — the `ms` oracle pin lags the settled ms-cli 0.16.0 (owning phase: **before S2's oracle extension**) `#seedhammer`

Filed 2026-08-15. `oracle/pins.json` pins `ms` at commit `ddfa497` / `ms 0.15.0`, and the installed binary is that build — so the pin is HONEST and internally consistent, and no gate is affected. But `mnemonic-secret` HEAD is now `de593ca` with ms-cli **0.16.0**, whose bare-`bip48` permissiveness is the settled behaviour.

Not urgent and deliberately not done in-session: re-pinning is a chain — rebuild, install, re-record `pins.json`, then re-anchor S0's gate record. Per the D5 doctrine that chain needs **no new emulator walk**, because an oracle re-pin cannot reach the device path; `gaterecord -force` over the saved walk is the sanctioned rebuild. Do it when S2 extends the oracle, so the re-anchor happens once rather than twice.

### F-178 — S1's gate has a THIRD outcome: D-1 did not reproduce, and the flow ran to the engrave screen (owning phase: **`SPEC_multisig_build_repair.md` S6** — hardware; reassigned 2026-08-15) `#seedhammer`

Filed 2026-08-15 from S1's implementation. **This entry exists because the spec
demands it**: SPEC P1 says *"If P0 found no D-1 on the payload path, this stage
records that as its result and names the source or shape that was not
exercised, rather than closing silently."* This is that record.

S1's gate as the plan writes it has TWO arms — *"either the flow completes an
engrave, or D-1 reproduces and is captured as a failing test"* — and **neither
fired**. S1 does not engrave (plan §3 preamble: it ends at a screen, and F-175
ruled it recordless on that arm), and D-1 did not appear.

**Measured, by driving the emulator, not by reading the flow.** After
`walk_build_policy.js` closed green at the seed picker, the remaining screens
were driven by hand from the same live session. Every screen drew; none was
blank:

    Input Seed (Where from? -> FROM PAYLOAD)
    Input Seed: Source: the systemwide payload
    Add a BIP-39 passphrase? (Skip)
    Policy Review — Slots @1 and @2 filled from the payload (cards 1 and 2
      of 4, in payload order). Policy stub: 4c3c96f1 Slots: @0 (no fp) @1 …
    Which md1? Full policy md1 / Template-only md1
    EXPERIMENTAL (hold to confirm)
    Engrave Mode — What to engrave? Full (seed + keys) / Watch-only (keys)
    Choose engraving  TEXT+QR / TEXT ONLY / QR ONLY   Card 1 of 3 | Plate 1 of 1

So the Build-policy flow, **fed entirely by the payload with zero records across
the NFC reader**, is drivable from the template picker to the first engrave
screen. D-1 ("the flow dead-ends: a blank screen after configuration",
SPEC §2.2) **does not live on the payload path in the emulator.**

**What was NOT exercised, named as the spec requires** — D-1 belongs to one of
these, and S2 owns finding out which:

1. **The engrave itself.** The drive stops at the engraving-style picker; no
   plate was cut, `deriveMultisigLeg`/`bundleEngrave` ran only as far as
   drawing that screen. S2 holds the completed-engrave gate.
2. **HARDWARE.** D-1 was field-observed on a physical SH2. This was the wasm
   emulator, whose display, engraver and NFC are all stand-ins. S6 owns
   hardware validation, and this is the single most likely home for it.
3. **The NFC-scanned card source.** Deliberately excluded: F-174 makes
   `shNFC.presented() === 0` mandatory on a stage-gate run, so a scanned-card
   build is a driver smoke test and was not run here.
4. **The typed-seed source.** This run took the self seed FROM THE PAYLOAD.
   The field report says "after configuration", and a keyboard-entered seed is
   a different amount of state and a different code path
   (`seedEntryFlowTypedOnly` + `inputWordsFlow`).
5. **Every shape but one.** The walk drove n=3, k=2, self slot @0,
   fingerprints omitted, template `wsh`. `sh(wsh)`, `sh`, n ∈ {2,4,5},
   includeFp, and a non-zero self slot were covered by unit tests but by no
   walk.

**Consequence for S2, and it is gating.** S2's test 1 is *"the D-1 reproduction
from S1, promoted to a regression test — it MUST fail on the unfixed code"*.
There is no reproduction to promote. S2 must therefore either reproduce D-1 in
one of the five unexercised shapes above (2 and 4 are the cheapest), or record
that it could not and carry the completed-engrave gate alone — which is exactly
what SPEC P1's own sentence anticipates. **What S2 may not do is treat test 1
as discharged.**


**⚠ RE-READ 2026-08-15, and it inverts this entry's reassurance.** The session
recorded here as "the flow ran to the engrave screen, every screen drew, no dead
end" was **assembling a degraded policy**. Machine-checked in
`design/agent-reports/fable-s2-inheritance-rulings-2026-08-15.md`:

- the S1 walk's selection loop takes the first two payload cards by DEFAULT —
  A@0 and A@1 — and this hand-drive continued from that session, taking the self
  seed FROM PAYLOAD, i.e. masterA;
- `deriveAccountXpub(masterA, multisigSharedOrigin())` yields the byte-identical
  65-byte cc‖pk as card A@0, and `assembleBuildPolicy` accepts it, returning stub
  **`4c3c96f1`** — **the stub on this entry's own Policy Review screen**.

So the wallet on those screens is a "2-of-3" in which masterA's acct-0 key sits
at BOTH @0 (self) and @1, with masterA acct-1 at @2: one master holds the whole
wallet, and the duplicated key alone satisfies k=2. Every slot rendered
`(no fp)`, so it is invisible in every artifact.

**Consequences.** S2 may NOT pin these screens as a good-state regression guard —
they are screens of the defect. D-1's owning phase moves to **S6** (hardware;
unfalsifiable in the emulator), which is why this entry's header changed. And the
duplicate-key refusal is not merely "S2-owned": it is **S2's FIRST landing**,
before any S2 work that completes an engrave, with no hardware engrave of the
Build path until it is in.

The record of non-reproduction below stands, and is still what SPEC P1 required.
What was wrong was reading it as evidence that the flow was healthy.

---

### F-179 — ~~an em-dash BLANKS THE WHOLE BODY~~ **RESOLVED 2026-08-15 by S3b — and it blanks a SIBLING LINE, which is worse** (owning phase: **`SPEC_multisig_build_repair.md` S3b** — re-owned 2026-08-15, was S3) `#seedhammer`

> **RESOLVED by S3b (`0290459` the 27 fixes, `db6486c` the guard). The mechanism
> was worse than every description of it, including this entry's.**
>
> **An undrawable rune does not blank its own sentence — it blanks a CLEAN
> SIBLING LINE.** Measured on the Payload Warnings body, which holds two F1
> lines: the pure-ASCII `"A SECRET is stored unencrypted in flash."` and a
> separate em-dash sentence.
>
>     clean line only     7442 px
>     broken line only    5004 px
>     BOTH together       5004 px   <- identical to broken-only
>
> The clean line contributed **zero ink**. A correct, ASCII-only, funds-critical
> warning was invisible because a *different* line in the same body carried a
> rune the face lacks. So the blast radius was never limited to the strings that
> contained the bad rune — which is why per-string review could never have found
> it and only ink could. Fixed: 11014 px for that body, 10870 for the NFC
> no-integrity refusal.
>
> **What landed.** 27 sites fixed (U+2014 ×21, U+2026 ×4, U+2713 ×1, U+2192 ×1);
> `⌫ U+232B` correctly **exempted, not "fixed"**, since it is a keyboard sentinel
> blitted as an image. The guard is a **face-coverage lookup via `go/parser`**,
> not a blocklist: it reports `scanned 1790 production string literal(s) across
> gui/*.go; 0 undrawable rune site(s)`, names file/line/code-point on failure,
> and structurally cannot see comments because they are not in the AST. S2's
> 7-rune `blankingGlyphs` blocklist was **deleted as subsumed** — a list of runes
> that already bit you cannot catch the next one, which is F-163's construct.
> The sentinel exemption is **proven, not asserted**: a test requires the source
> to compare the rune as a char literal *and* reference the image asset, both
> read from the AST, plus a raster check — and a rubber-stamped exemption was
> mutation-proven to fail.
>
> **F-183 fixed here too** (below): the shared floor is now derived from a
> measured blank rather than a constant.

> **ADDENDUM 2026-08-15 — the site list below is stale in BOTH directions, and
> the class is wider than "em-dash". Read this before working the entry.**
>
> Owning phase moved S3 → **S3b**, a stage ruled in on 2026-08-15
> (`design/agent-reports/operator-rulings-2026-08-15.md` §A; S0b is the
> precedent). This is re-ownership into an adjacent gated slot, not a deferral:
> **S4 may not start until S3b closes green.** S3 was kept to its briefed naming
> fix because F-179 sites sit *inside* the function S3 edits
> (`gui/md1_inspect.go:58` is S3's caller edit; `:60` and `:65` are F-179 sites
> in the same `md1Summary`), so the two must be sequential commits by one agent,
> never parallel writers on one file.
>
> **The mechanism, machine-checked — this is the ground truth the entry lacked.**
> `font/bitmap/bitmap.go:33` sets `indexLen = unicode.MaxASCII`, and `glyphFor`
> rejects `int(r) >= indexLen` at `:62`. So **every non-ASCII rune is
> unrenderable on every bitmap face** — not just `—` and `·`. Face choice is
> therefore immaterial (all six faces `gui` uses share one boundary), and a
> hand-written blocklist of offending runes is the wrong instrument: the guard
> must be a **coverage lookup**, which is also why it will not go stale.
>
> **Both prior enumerations were em-dash-shaped.** A rune-agnostic scan over
> `gui/*.go` non-test string literals (comments stripped) finds **28 raw hits**
> against the entry's 27 and a 2026-08-15 re-derivation's 21. The delta is not
> line drift — it is four rune classes nobody was looking for:
>
>     ✓ U+2713  bundle_flow.go   "%d. %s ✓"        (drawn into the review body)
>     … U+2026  singlesig_pick, slip39_polish, verify_address, codex32_polish
>     → U+2192  codex32_polish   "pos %d: %c → %c"
>     ⌫ U+232B  gui.go           see the false positive below
>
> **One confirmed FALSE POSITIVE, and it is load-bearing.** `gui/gui.go`'s
> `alphabet += "⌫\n"` is a **sentinel rune for the keyboard's backspace key, not
> drawn text** — `gui.go:1572-1574` special-cases `key.r == '⌫'` and blits
> `assets.KeyBackspace` as an image. A guard that merely refuses non-ASCII
> literals will flag it and force a "fix" that breaks the keyboard. **The guard
> needs a documented exemption for runes that never reach the text path**, and
> that exemption is itself the sort of hand-maintained list F-163 indicts — so
> pin it with a test that proves the rune is image-drawn, not with a comment.
>
> Net: **27 candidate live sites** at `4b8488e` (28 raw minus the keyboard
> sentinel), of which 6 are the non-em-dash classes above. **Re-derive at
> execution time anyway** — S3 lands before S3b and moves line numbers.

Found 2026-08-15 by S2's whole-walk raster floor, on its first run, on the one
screen that is the operator's last chance to stop.

**F-78 recorded that `·` is "a zero-pixel glyph" in `poppins.Regular16`. That
understates it, and the understatement is what let this survive.** A glyph the
face lacks does not merely fail to draw itself.

**CORRECTED 2026-08-15 by the S2 execution review, and the correction is the
point of this entry.** This item first said the glyph blanks *its line*. It
blanks **the entire body of the frame**. The reviewer measured five bodies of
different lengths through `showError` and every one of them rastered at
**exactly 2652 px — the title-only value** — regardless of how much text
followed. One glyph anywhere in a body and the operator sees a title and nothing
else.

**And the second correction is worse than the first: `uiContains` still returns
TRUE on the blank frame.** The text ops are submitted; only the drawing fails.
So **every content-based assertion in the `gui` package is blind to this class**
— including S2's own D-4 guard, which asserted the gather's new title on a frame
whose body was gone. Ink is the only instrument that sees it. Any fix for the
remaining sites must be checked by raster or by a source/glyph lookup, never by
asserting the text is present.

Measured through `showError` with the repo's own `runUITouchRaster`:

    "Dropped an incomplete card - scan all its chunks to include it."   ink 7419
    "Dropped an incomplete card — scan all its chunks to include it."   ink 2652

2652 is the **exact** figure `gui/raster_test.go` records for F-151's
shipped-blank body ("the body that shipped blank drew 2652 px, the fixed one
6688"). So F-151's defect and this one are the same defect, and the raster floor
that was written to catch F-151 has been catching this class all along without
anyone naming the cause.

On the Build path the EXPERIMENTAL warning measured **4973 ink pixels against a
5482 px title-only frame** — i.e. below blank. Removing the em-dash took it to
**18563**. S2 fixed that one and the review screen's fp line, because both are on
its own walk.

**FIXED AT S2 (6 sites), because they are on the flow S2 edited:** the
EXPERIMENTAL warning body and the Policy Review's fingerprint line
(`gui/multisig_build.go`), and all four gather strings in `gui/bundle_flow.go` —
the two "Done" refusals plus the three `feedback()` messages, which the review's
list did not include and which are drawn into the GATHER's own body, so one of
them blanks the card tally rather than a modal. The pending-card refusal is now
driven from Build and rastered
(`TestGatherPendingRefusalIsReadableFromBuild`): **2652 -> 9855 px**.

**The rest are not fixed.** Enumerated by a script over `gui/*.go` non-test
string literals with whole-line comments excluded, re-run 2026-08-15 AFTER S2's
two fixes — **31 sites**, of which 4 (`bundle_flow.go:383`,
`codex32_polish.go:185,289`, `slip39_polish.go:237`) are trailing `// F-78:`
comments quoting the glyph deliberately, leaving **27 live strings**:

    gui/bip85.go:228
    gui/bundle_flow.go:62,65,67,184,200,202,430,438
    gui/codex32_polish.go:28
    gui/derive_xpub.go:254,487
    gui/gui.go:1020
    gui/md1_gather.go:105,155,168
    gui/md1_inspect.go:60,65
    gui/mk1_inspect.go:202
    gui/seedxor_polish.go:85
    gui/sysw_load.go:128,274,275,279,280
    gui/sysw_source.go:114
    gui/verify_address.go:95

Re-derive rather than trust this list; line numbers decay every merge.

Several are refusals — `sysw_load.go`'s "A SECRET is stored unencrypted in
flash", `sysw_source.go`'s "NO integrity check at all" — where a blank body is
the worst possible outcome.

**What to build, not just what to fix.** A per-string fix is a fix; the class
needs a GUARD. The cheap one is a test over `gui/*.go` production string literals
refusing any rune the body and title faces lack, which is a lookup, not a raster.
Scope it to the faces actually used (`poppins.Regular16`, `poppins.Bold25`).

S2 built the shape of it for one file — `stringLiterals` +
`TestGatherScreenTextCarriesNoBlankingGlyph` in
`gui/bundle_gather_refusal_test.go`, with its own scanner-can-see mutation proof.
It scans LITERALS rather than raw source, because the first version fired on a
COMMENT containing an em-dash, which is not a defect and would have taught the
next author to delete the guard instead of the glyph. Widening it to the whole
package is S3's job and is mostly a matter of choosing which functions draw.

---


**COUNT CORRECTED 2026-08-15, and the correction is a COMMAND rather than a
number.** This entry said 27, the S2 fold report said 24, the fold review
re-enumerated 21, and a fourth method gives 40 — four numbers for one fact,
which is the failure this repo has recorded before: the code was right and the
record went stale, three times over, because each pass hand-counted a slightly
different set.

The number is whatever this prints on the day you ask:

    grep -rn '—' gui/*.go cmd/emu/*.go | grep -v _test.go | grep -E '"[^"]*—' | wc -l

**40 as of 2026-08-15.** It over-counts deliberately — it catches every
production string literal carrying the glyph, including ones that never reach a
screen — because a guard that misses a real site is worse than one that lists an
irrelevant one. Narrow it by reading the hits, never by trusting a remembered
total.

**What is GUARDED is much narrower, and that distinction is the whole reason the
numbers disagreed.** `TestGatherScreenTextCarriesNoBlankingGlyph` scanned exactly
three functions — `bundleGatherFlow`, `feedback`, `tally` in `bundle_flow.go` —
and those were clean. It scanned LITERALS, not raw source, because a v1 that
scanned source fired on a comment, and a guard that cries wolf gets deleted
instead of the glyph. `TestStringLiteralScannerCanSee` was the scanner's own
mutation proof.

> **SUPERSEDED 2026-08-15 by S3b, and the line number above was DELIBERATELY
> DROPPED rather than re-pinned.** That guard and its seven-rune `blankingGlyphs`
> blocklist are **gone**, subsumed by `TestProductionStringsAreDrawable`, which
> asks the faces themselves about **every** production string literal in
> `gui/*.go` — 1790 of them, 0 undrawable — so those three functions are covered
> as a consequence rather than by being named. What survives in
> `gui/bundle_gather_refusal_test.go` is the drive/ink test, which is the only
> thing there that proves a REACHED screen draws ink; a source lookup cannot.
>
> The old line-143 citation into that file decayed the moment S3b shrank it from
> ~210 lines to 104, and `scripts/plan-cite-gate.sh` caught it as *"file has only
> 104 lines"* — which is the gate working. It is not re-pinned to a new line,
> because a line number for a deleted test is a second decay waiting to happen;
> the test **name** is the durable reference.
>
> (Nor is the dead citation quoted in this note. Writing it out in backticks —
> even to explain that it is dead — puts it straight back in front of the
> parser, which is exactly what happened on the first attempt at this repair and
> kept the count at 21.)

**So S3 inherits: everything the command lists that is not inside those three
guarded functions.** Two of them are secret-exposure warnings the S2 reviewer
measured at 2652 px — i.e. currently invisible — which makes them the first
ones to fix, not the last.
### F-180 — the Go cosigner-card roster is in a DIFFERENT order from the emulator payload (owning phase: **`SPEC_multisig_build_repair.md` S4**) `#seedhammer`

Found 2026-08-15 while writing S2's typed-seed walk, by running it:

    gui/multisig_build_payload_testdata_test.go  cosignerCardRoster
        A@0, B@0, C@0, A@1, B@1
    cmd/buildpayloadcards/main.go                wanted
        A@0, A@1, B@0, C@0

Both are deliberate and neither is wrong, but they are not the same payload, so
**a tap sequence measured against one is wrong against the other.** S2's Go walk
taps SKIP, USE, USE to reach Trace A's B@0 + C@0; the emulator walk taps SKIP,
SKIP for the same result. The plan and the fable ruling both describe the
emulator order as though the unit fixtures shared it.

The first draft of `TestBuildWalkTypedSeed` tapped SKIP, SKIP and selected A@1 —
caught immediately by S2's own foreign-origin refusal, which is the system
working, but a test asserting only "the flow completed" would have passed on the
wrong cosigner set.

**Do not reorder the roster casually**: S1's tests take `cosignerCardRecords(t,
n)` PREFIXES and assert which card fills which slot, so the order is load-bearing
in both directions. S4 owns the slot-assignment model and is the right place to
either align them or to state, in both files, that they deliberately differ.

---

### F-181 — ~~the typed-seed EMULATOR leg is not delivered: `shTap` cannot find a keyboard key~~ **WITHDRAWN 2026-08-15: the leg needed no keyboard, and S2's gate is now driven** (owning phase: **none for the gate; a keyboard driver remains OPTIONAL for S4**) `#seedhammer`

**WITHDRAWN, the way F-176 was, and for the same reason: the premise was false.**

This entry said S2's emulator gate was blocked on driving the on-device
keyboard. It was not blocked on anything. **The emulator payload has always
carried a `ClassMnemonic`** (master A, `cmd/emu/sysw_cards_payload.go`), so the
self seed arrives FROM THE PAYLOAD with confirm-taps only — and the plan's own
wording said so all along: *"default taps + **payload seed**"*. The keyboard was
never on the path this gate needed.

The mistake was not the measurement; the measurements below are real and still
useful. The mistake was **treating "the route I picked is hard" as "the gate is
blocked"**, and then writing the gate off in the same breath. The entry's worst
sentence was the justification, not the deferral: *"S2's own gate is satisfied by
the Go walk plus the payload-leg emulator walk that RAN green"* — the payload-leg
walk stopped at `waitFor("Input Seed")` by design and had never reached an
engrave. A closed gate was reported from a walk that could not have closed it.

**Both arms are now driven, tap-only, and recorded** (S2 fold, 2026-08-15):

    refusal arm  picks=[use,use],   seedFrom=payload -> "Duplicate key",
                 naming slot @0 (your key, from your seed) and slot @1 (payload card 1)
    clean arm    picks=[skip,skip], seedFrom=payload -> 9 plates, unattributed 0,
                 census 1 ms1 + 2 mk1 + 6 md1, policy stub 06215ac0, presented 0

**What remains, and it is now genuinely optional.** A `typeWord` over `shTap`
would let a walk drive the KEYBOARD, which no walk does. That is worth having for
S4's per-slot multi-seed entry, but it gates nothing today. The measurements that
make it non-trivial stand:

S2 delivered `TestBuildWalkTypedSeed` as a Go test and a `typeWords` driver over
the event router, both run. The **emulator** half — a `typeWord` over `shTap`, as
the plan's test 1(b) describes — was attempted 2026-08-15, driven live in the
browser, and STOPPED rather than shipped unrun.

**Why, measured rather than assumed.** The BIP-39 keyboard's key rectangles are
computed at layout time from font metrics (`NewKeyboard`: `ctx.Styles.keyboard.
Measure(MaxInt, "W")` plus per-row centring), so no coordinate formula exists in
the walk's own terms. Probing empirically in the live emulator did not converge:

  * `shTap(80, 180)` typed `Q`; `shTap(300, 200)` typed `U`  — same row, and the
    hit regions do not fall on the grid a naive `rowY`-style formula predicts;
  * **the valid-key mask makes blind probing self-defeating.** After `Q` only
    `U` is enabled, so a sweep reads "nothing happened" for every other key and
    learns nothing about where they are;
  * a sweep looking for backspace instead COMPLETED word 1 and advanced the flow
    to word 2, because an auto-completing fragment needs no confirm.

**What would make it cheap, and it is not a shortcut into the GUI.** `op.Drawer`
already resolves an input's screen rectangle — `tapNavSlot` in `gui/raster_test.go`
uses exactly that to tap a nav slot by button rather than by coordinate. Exposing
the same resolution to the walk (e.g. `shKeyRect(rune)` returning the CURRENT
key's rect, which the walk then taps) keeps `walk_js.go`'s rule intact: it
computes a coordinate a finger could have found by looking, and bypasses no
screen, no validation and no flow.

~~Not built at S2 because it is a walk-API change and S2's own gate is satisfied
by the Go walk plus the payload-leg emulator walk that RAN green.~~ **That
sentence was the defect** — see the withdrawal at the top. Not built at S2
because it gates nothing; S4 may build it if per-slot multi-seed entry wants
driving in the emulator, and the Go-side `typeWords` driver
(`gui/multisig_build_walk_test.go`) already covers the same ground in tests.

---

### F-182 — the end-of-bundle ms1 reminder is titled "Engrave Bundle" on the Build path (owning phase: **`SPEC_multisig_build_repair.md` S5** — with the engrave tail) `#seedhammer`

D-4-adjacent, found 2026-08-15 while fixing D-4 and deliberately left out of it.

S2 made `bundleGatherFlow`'s title the caller's, which fixes the gather for all
five callers. `bundleEngrave` still hard-codes one:

    gui/bundle_flow.go:396   showError(ctx, th, "Engrave Bundle", bundleMs1ReminderText())

That modal is shown at the end of a Build-policy engrave too — measured: S2's
walk cuts 9 plates and reaches it. It is a DIFFERENT screen from D-4's (which
names the gather specifically), and `bundleEngrave` is shared by T5, single-sig
and the supplied-md1 path as well, so the same parameter-not-rename judgement
applies and is worth doing once, in the stage that owns the engrave tail.

### F-183 — ~~`assertFrameHasBody`'s floor is calibrated for ONE screen shape but named and worded as general~~ **FIXED 2026-08-15 by S3b (`db6486c`)** (owning phase: **`SPEC_multisig_build_repair.md` S3b** — with the F-179 raster class) `#seedhammer`

> **FIXED in the stage that owned it, the same day it was filed.** `const floor =
> 4000` is now `titleOnlyInk(t) + margin`, where `titleOnlyInk` **searches 1..3
> nav buttons and returns the worst** — so the floor is derived from a measured
> blank instead of a constant calibrated on one screen. The sibling constant
> `buildWalkRasterFloor` is pinned the same way rather than converted, and is now
> bracketed on both sides:
>
>     worst blank 5482 px  <  floor 6000 px  <  thinnest real screen 6566 px
>
> That bracket is the property the old constant lacked: 4000 sat *below* the 5482
> px blank of any three-nav screen, so it would have passed a completely blank
> body — the exact defect it existed to catch.

Found 2026-08-15 by S3, which hit it, declined the helper, and rolled its own
floor rather than lowering the bar to fit.

`gui/raster_test.go:73` sets `const floor = 4000`, and its own comment says why:
*"CALIBRATED against the real defect rather than guessed. Measured on the unload
result screen: the body that shipped blank drew 2652 px, the fixed one 6688."*
For that screen the floor is correct and well-argued.

**The trap is that chrome contributes ink, and the floor does not know which
chrome.** A screen drawing three nav buttons plus a title renders **5482 px while
its body is entirely blank** — a figure this cycle has now measured twice
independently (S2's execution review recorded 4973 ink against the same 5482 px
blank frame). **5482 > 4000**, so on any such screen `assertFrameHasBody` passes
a completely blank body.

Currently **latent, not live**: the helper has exactly one caller
(`gui/raster_test.go:106`, the unload result screen — the very screen it was
calibrated on), which is correct. What makes it a trap is the generic name and
the generic failure text, *"a screen with a title AND a body draws far more"*,
which read as a general-purpose guard. The next author to reuse it on a
chrome-heavy screen gets a silent false PASS on exactly the F-179 defect the
helper exists to catch.

S3's own handling is the template for the fix: it measured the blank frame for
*its* screen shape (5482) and set its floor above it (`buildWalkRasterFloor`,
6000; measured ink 10762).

**Fix.** Make the floor a required argument derived from the screen's own
measured blank, or have the helper take a blank-frame baseline and assert
`ink > blank * k`. A single global constant cannot separate blank from drawn
across screens with different chrome, and one that silently cannot is worse than
no helper. Belongs with S3b because it is the same class and the same instrument.

### F-184 — a needle's uniqueness proof counts COMMENTS as production sites (owning phase: **none — cross-cutting Minor, batches to the end**) `#seedhammer`

Found 2026-08-15 by S3, which worked around it and left a comment explaining the
workaround so the next author does not undo it.

`cmd/emu/needle_test.go` proves a walk needle is single-site by substring match
over `gui`'s source. The match is blunt: **it counts occurrences in comments as
production sites.** So quoting a needle string in a comment — the ordinary way to
explain why a screen says what it says — makes that needle look two-sited and
costs it its uniqueness proof.

S3 hit this twice: writing `P2SH-P2WSH` into a `gui/` comment broke its own walk
anchor, and its sweep comment reintroduced the literal `TYPED-ONLY` and failed
gate arm (a). Both were caught by running the gates, not by reading. There is
existing precedent for the workaround — `buildCosignerGatherTitle`'s comment
deliberately does not quote its old title — but nothing states the rule, so it is
rediscovered by tripping over it.

**Direction is fail-SAFE** (over-counting makes a needle look less unique than it
is, never more), which is why this is Minor and not Important. The cost is
paid in author confusion and in comments that must be written around the check.

**Fix.** Strip comments before counting — a Go-aware scan, or the same
comment-stripping the F-179 scanner needs. Doing both with one helper is the
obvious economy. Until then, **do not quote a needle literal in `gui/` source
comments**, and say so where needles are defined.

### F-185 — a modal's body can scroll off the first frame with no affordance, so a required instruction is present in the string and absent from the screen (owning phase: **`SPEC_multisig_build_repair.md` S5** — with the engrave tail's screens) `#seedhammer`

Found 2026-08-15 by S4's emulator walk, on the screen whose whole purpose is to
tell an operator what to do about a seed↔key mismatch.

The gate's FAIL body carried all four elements the plan requires — likely causes,
the statement that reassigning the slot suppresses the check rather than fixing
it, the slot name, and the host route. The **rendered first frame** ended
mid-word:

    ...rewritethepayloadonthehostwith

`ErrorScreen`'s body scrolls, and **nothing on the frame says so**. So the host
route — the one safe action the screen exists to offer — sat below a fold the
operator has no reason to suspect. S4 trimmed both refusals (606→422 and
636→478 chars) and pinned the result with a first-frame test.

**Why no Go test caught it, and this is the transferable part.** Every content
assertion in the package checks the string that was *submitted*, not the pixels
that were *drawn*. This is the same seam as F-179 — where an undrawable rune
blanked a whole body while `uiContains` still returned true — arriving by a
different route: there the text was submitted and not drawn because a glyph was
missing; here because it was past the viewport. **A string assertion cannot see
either.** Ink and frame inspection can.

**Two things this does NOT fix, both recorded honestly rather than left implied:**

1. **The new test pins "it draws today", not a margin.** A +65-character mutation
   still fits on the first frame, so a future edit can re-break it without
   turning the test red. A margin test would assert the drawn body ends where the
   source string ends, not merely that a known-good string fits.
2. **Every other long modal in the firmware carries the same unmeasured
   exposure.** S4 measured the two screens it owns. Nothing has measured the
   rest, and no guard exists for the class.

**What to build, not just what to fix.** The class needs the F-179 treatment: a
check that compares the drawn frame against the source string — or that refuses a
body longer than the viewport can hold — rather than per-screen trimming done
each time somebody notices. Scoped to S5 because that is where the engrave tail's
screens land, and because S5 already owns the "every comparison the device asks
for must be one the operator can perform" constraint, of which this is the
rendering half.

### F-186 — ~~`md encode` cannot encode a DIVERGENT-origin multisig template~~ **HALF WITHDRAWN 2026-08-15: md CAN. The surviving half — an `internal:` error on wrong syntax — is FIXED in the primary.** (owning phase: **`SPEC_multisig_build_repair.md` S5**) `#seedhammer` `#cross-repo`

> **CORRECTION 2026-08-15, and the correction matters more than the entry.**
>
> **md encodes divergent origins today, and always could.** The claim below that
> it cannot was MINE and was wrong: I fed md **descriptor** syntax
> (`[fingerprint/path]@0/…`) where **template** syntax was required
> (`@0/48'/0'/0'/2'/<0;1>/*`). `lex_placeholders` has captured the inline
> origin all along, and `make_path_decl` emits `PathDeclPaths::Divergent`
> whenever the per-placeholder origins differ. Measured with the correct form:
>
>     path_decl: {"data": ["m/48'/0'/0'/2'", "m/48'/0'/1'/2'"],
>                 "tag": "Divergent"}
>
> **S5 is NOT blocked.** Its md1 *does* require divergent origins, so S5's oracle
> derivation must build the template in the inline per-`@N` form rather than the
> shared `--path` form S5.0 shipped — and **uniform inline origins are
> byte-identical to `--path`**, so no committed record or golden is staled.
>
> **What survived is defect 2, and it is FIXED** in the primary
> (`descriptor-mnemonic` `11b01a9e`): the wrong syntax produced
> `internal: synthetic key [73c5da0a not found in key map`, which reads as a tool
> bug rather than a rejected input — and is exactly how I reached the wrong
> conclusion. It now refuses at lex time, naming the correct syntax.
>
> **The trap recorded for whoever touches that encoder next:** the obvious fix —
> teaching `lookup_key` to strip the bracket — makes md encode with the origins
> **silently dropped** (`path_decl` empty, verified). That trades a loud
> wrong-syntax error for a quiet wrong-policy on an encoder that describes where
> money lives. A **round-trip** assertion catches it; an encode-succeeds
> assertion passes.
>
> **The generalisable lesson:** a confusing error message is not cosmetic. This
> one cost a wrong conclusion about a primary tool's capabilities, an
> almost-landed change that would have introduced a funds-relevant silent
> failure, and a fable consult. `internal:` in user-facing output should mean
> "the tool broke", never "your input was wrong".

Found 2026-08-15 while building S5.0's built-policy `ExpectKind`, independently
by that stage's implementer and by the controller reproducing it.

**Measured, with real BIP-48 keys** derived via `ms derive --template
bip48-p2wsh` from BIP-39's `abandon…about` vector (fingerprint `73c5da0a`), and
run against the pinned oracle `~/.cargo/bin/md`, version `md 0.13.0`:

    # DIVERGENT: two accounts of one master, bracketed per-key origins
    md encode "wsh(sortedmulti(2,[73c5da0a/48h/0h/0h/2h]@0/<0;1>/*,\
                                 [73c5da0a/48h/0h/1h/2h]@1/<0;1>/*))" \
      --key @0=<A@0 xpub> --key @1=<A@1 xpub> --network mainnet --group-size 0
    TRUE_EXIT=1
    md: template parse error: internal: synthetic key [73c5da0a not found in
        key map (rendered: [73c5da0a/48'/0'/0'/2']xpub6DXuQW1FgeHbfmex…)

    # SHARED via --path: reaches the CODEC, i.e. a real encode
    md encode "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))" --key … \
      --path "m/48'/0'/0'/2'" --fingerprint @0=73c5da0a --fingerprint @1=73c5da0a
    TRUE_EXIT=1
    md: codec error: payload is 246 data symbols; … (use --force-chunked)

So the shared path works and only the divergent one breaks.

**Two distinct defects, and the second is the worse one.**

1. **No working invocation for divergent origins.** `md encode --help`
   documents `--path` as *"Override the inferred origin path with a single
   shared path (**flattens Divergent mode to Shared**)"* — md plainly has a
   notion of Divergent mode, yet no invocation found encodes one.
2. **It fails INTERNALLY, not cleanly.** The message says `internal:`, and md
   **rendered the descriptor correctly** before failing to find `[73c5da0a` in
   its own key map — the parser appears to split on `/` and take the bracket
   prefix as a key name. An unsupported input should produce a clean refusal
   naming what is unsupported. A funds-relevant encoder that fails confusingly
   is a defect in its own right, independent of whether the feature is wanted.

**Why it may block S5.** S5 is *"multi-slot self, divergent origins, and the
engrave tail"*, and the payload inventory describes Trace B as **A@0 and A@1
(one master, two accounts) plus B@0 and C@0 — multi-slot, divergent origins,
multi-master**. S5's gate mints a record whose census includes the engraved md1.
If that md1 is divergent-origin, **the mint refuses and S5 cannot close.**

**Why it may NOT block S5, which is why this is a question and not yet a
Critical.** "Divergent origins" may describe the **cosigner cards'** own origins
— each mk1 carries its own — while the assembled **policy** md1 still uses one
shared origin. Evidence for that reading: S4 derives the self key *"at the
LOCKED shared origin (self-origin == policy-origin by construction)"*; S2
shipped an interim foreign-origin refusal; and §0.1a defers template-aware
defaults to S5. If the device emits a shared-origin md1, md's limitation never
bites.

**If it does block**, the fix is **Rust-primary first**: the md primary is
`descriptor-mnemonic` (pin `5a0a4f41`, which is also its HEAD and the
`md-cli-v0.13.0` tag), it lands there with test vectors, and only then does the
oracle pin move — noting the pin binds a maintainer-built binary's SHA-256, so a
new md tag implies a rebuild and re-record, and an S0 re-anchor exactly like the
`ms` bump just performed.

**Defect 2 is worth fixing regardless of the answer to defect 1.**

### F-187 — md's template origin syntax is undocumented for end users, and the only feedback for getting it wrong was an `internal:` error (owning phase: **none — cross-repo docs, batches to a manual cycle**) `#cross-repo` `#docs`

Filed 2026-08-15 out of F-186, which was itself a wrong conclusion caused by the
gap this entry describes.

**What is missing.** `md` templates carry a key's origin **after** the
placeholder — `@0/48'/0'/0'/2'/<0;1>/*` — and per-key origins may differ, which
is how a **Divergent** path declaration is expressed. Nothing user-facing says
so. `md encode --help` describes `--path` as *"Override the inferred origin path
with a single shared path (flattens Divergent mode to Shared)"*, which mentions
Divergent mode without ever showing how to write one, and the `[TEMPLATE]` help
line gives only `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))` — an origin-free example.

**Why it cost something.** A reader who knows Bitcoin descriptors reaches for the
descriptor form, `[fingerprint/48'/0'/0'/2']@0/…`, because that is what every
other tool in the ecosystem accepts. It is not md's template syntax. The feedback
was `template parse error: internal: synthetic key [73c5da0a not found in key
map` — which reads as a tool bug, not a rejected input. That produced a wrong
conclusion (that md could not encode divergent origins at all), an
almost-landed change to a funds-relevant encoder that would have **silently
dropped** the origins, and a design consult. The error message half is fixed in
`descriptor-mnemonic` `11b01a9e`; **the documentation half is this entry.**

**What to write, and where.** The end-user manual for the m-format star lives in
the sibling `bg002h/mnemonic-toolkit` repo at `docs/manual/`, and `md-cli` has
its own reference surface (`--help`, man pages). Both want:

1. **The template origin form, with a worked divergent example** — two accounts
   of one master is the motivating shape and the one a shared `--path` cannot
   express:
   `wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))`
2. **An explicit "this is NOT descriptor syntax" note.** The bracketed
   `[fp/path]KEY` form is the thing readers will try first; say plainly that md
   templates put the origin after `@i`, and that md models the fingerprint
   separately (`--fingerprint @i=HEX`) rather than inside a bracket.
3. **Shared vs Divergent as a user-visible concept** — when `--path` applies,
   what it flattens, and that uniform inline origins and `--path` produce
   identical bytes.

**No owning phase.** This does not gate the multisig-build-repair plan; S5 is
unblocked and derives the correct form directly. It batches to whenever the
manual is next worked. Companion entries belong in `mnemonic-toolkit` (the
manual) and `descriptor-mnemonic` (`--help`/man) when this is picked up, per the
cross-repo notification convention.

---

### F-188 — RULED 2026-08-15 (operator: "Build this"): the supply path engraves ONE plate where the seed fills SEVERAL slots, so the engrave rule and the verify rule disagree at the source (owning phase: **`SPEC_multisig_build_repair.md` S5**) `#seedhammer`

Filed and immediately ruled IN by the operator, having been raised as a
follow-up by the fable design review persisted at `5fc08c4`.

**The disagreement, measured.** `allUserSlots` (`gui/multisig_match.go`) derives
the operator's seed **at each policy slot's own `OriginPath`**, so it returns
every slot ONE SEED accounts for — at different origins, carrying **different
keys**. The supply path (`gui/multisig.go:141-149`) then engraves a plate for
the **first** match only and announces:

    This key is reused at slots @0 and @1; engraving the first (@0).

**That sentence is false**, and the falsity is the tell: the keys at @0 and @1
are not the same key. It is one seed filling two slots at two origins. The
message describes a shape the code does not produce.

**Why it had to be ruled rather than just fixed.** It changes what goes on
**steel** in a flow this plan does not own — the operator may now cut more
plates than the same inputs produced yesterday — so it is a normative output
change, not a refactor. The **Rust-primary rule does not bind**: this is
fork-native GUI/UX with no Rust counterpart (exempt clause b).

**What it buys.** The verify's rule is a plate per matched slot; the engrave's
is one plate per matched seed. Making the engrave agree removes the mismatch at
its source rather than teaching the checker to tolerate it — and a checker
taught to tolerate a disagreement is how a false GREEN gets in. Two attempts to
absorb it on the verify side were rejected: dropping the leg→plate rule makes a
LOST plate pass, and deduping legs by identical mk1 is inert for the real shape.

**It does NOT make the slot-set fix (F-8 / the `expectedSlots` work) redundant,
and reading it that way would reopen the false RED on the build path.** There,
`allUserSlots` can still exceed the operator's declared `SelfSlots`: a payload
cosigner card carrying a DIFFERENT key derived from the same seed at another
origin is not a duplicate — `duplicateSlotPair` refuses only IDENTICAL keys —
and is admitted. The two changes are complementary.

**Constraints on the build.** Reuse `multisigEngraveCardsMulti` (already built
in S5.B); do not invent a second emitter. One ms1 per distinct seed, and the
supply path has exactly one seed. Rewrite the message to announce what WILL be
cut instead of apologising for what is dropped. **State the plate count before
the tail** — the plan's S4 prose constraint, and it binds harder here than
anywhere, because this is the change that makes the count differ from what the
operator saw last time.

---

### F-189 — `multisigEngraveCards` and `findUserSlot`'s `reused` return have no production callers left (owning phase: **`SPEC_multisig_build_repair.md` S5** — with the block that retired them) `#seedhammer`

Filed 2026-08-16 by the F-188 implementer, which could not write this file from
its worktree.

Both were the single-leg world's API. `multisigEngraveCardsMulti` replaced the
first when the build tail went multi-slot (S5.B); `allUserSlots` replaced the
second when the supply tail did the same (F-188). Measured: neither has a
non-test caller.

**Why this is not merely tidiness.** A retired API left in place is an
invitation to reintroduce the rule it encoded — exactly what happened with
`errVerifyLegHasNoPlate`, where a *review* proposed relaxing a rule the deleted
symmetry made look optional. The same argument retired
`extractSuppliedMd1AndMk1` rather than leaving it as a trap. Delete them, or
state in each why it is kept.

### F-190 — `cmd/emu/needle_test.go`'s uniqueness counter reads SHARED-HELPER strings as if they were per-flow (owning phase: **`SPEC_multisig_build_repair.md` S5.D** — with the walk block) `#seedhammer`

Filed 2026-08-16, widened by F-188 and found by it.

A walk needle must identify ONE flow. The counter proves a string appears at one
production site, but a string emitted by a **shared helper** appears once in
source and on **every caller's screen** — so the count says "unique" while the
needle is anything but. F-188 hit this directly: reusing the build path's
`"Plate Count"` census title on the supply path made the build walk's anchor
two-site, and the implementer had to differ the title (`"Plates To Cut"`) purely
to keep the walk honest.

**That is the tail wagging the dog** — a test's needle mechanism chose an
operator-facing string. The body the operator reads is identical in both flows,
which is correct; only the title differs, and only because of this counter. S5.D
owns the walks, so it owns fixing the counter to attribute shared-helper strings
to their CALL SITES rather than their definition — after which the titles should
be reconciled on UX grounds alone.

This is the same class as **F-184** (a needle's uniqueness proof counts
comments as production sites): the counter measures source text where the claim
is about screens.
### F-191 — ~~a passphrase divergence between engrave and verify is reported as "That seed is not a cosigner"~~ **FIXED 2026-08-16 at the site filed, in `023505c`. THE CLASS IS NOT CLOSED — see the note at the end.** (owning phase: **`SPEC_multisig_build_repair.md` S5.D** — with the screens/prose block) `#seedhammer`

Filed 2026-08-16 by the S5 policy-identity fold implementer, which could not
write this file from its worktree. Found by the fable seam review (M1).

`gui/multisig_verify.go:479-481`. The engrave accepts a payload-borne
passphrase (`syswPassphraseFlow`, `gui/multisig.go:147`); the verify requires
it re-typed (`passphraseFlow`, deliberate per §7.4). A CORRECT seed with a
forgotten or mistyped passphrase makes `allUserSlots` return empty, so the
flow names the SEED and never mentions the passphrase — on plates that are
perfectly good.

**It is a false RED that teaches the operator the wrong lesson.** "My seed
isn't in my wallet" is the most alarming sentence this device can say, and
here it is caused by a keystroke. The device knows a passphrase was offered
and can say so; distinguishing "no slot matches with this passphrase" from
"no slot matches at all" costs one re-derivation with the empty passphrase.

Pre-existing in kind (the pre-S5 verify had the same shape through
`findUserSlot`) and unchanged by the S5 seam, so it did not gate the fold.

**Landed 2026-08-16 by the controller** on the implementer's behalf: an agent
confined to a worktree cannot write this file, so a filed defect would otherwise
survive only inside a report.

**FIXED 2026-08-16 in `023505c`** (S5, "the screens an operator reads before
putting a seed on steel"), verified by the controller before the whole-diff gate:
`multisigVerifySeedIsInnocent` (`gui/multisig_verify.go:74`) re-derives with the
EMPTY passphrase only when one was actually typed — a passphrase never offered is
not re-derived, because that is the derivation which just failed and re-running it
would route a genuinely foreign seed to the reassuring arm. It feeds a three-state
`multisigVerifyNoSlotBody(passphraseTyped, innocent)`: proved innocent, passphrase
typed but still nothing, and no passphrase typed. Two tests pin it, and the second
defeats a `return true` predicate explicitly.

**BUT THE CLASS RECURRED AT A NEW SITE, and this is the part worth carrying
forward.** The whole-diff gate's **I-14** found the same reasoning error one arm
over: the verify's "that seed is a cosigner, but none of its slots were engraved in
this run / that seed's slots have already been checked" arms assert a *foreign
seed* where a same-seed passphrase divergence is equally likely. Fixing F-191 where
it was filed did not fix the flow's habit of naming the frightening cause when it
cannot tell which cause it is. **A follow-up closed at its filed site is not a class
closed** — grep for the mechanism, not for the ticket. I-14 is burned down with the
gate's fold, not here.

---

### F-192 — the F-185 drawn-frame check gates only the screens S5.C touched; every other long modal is still unmeasured (owning phase: **S6b — operator ruling 2026-08-17, sweep before the hardware flash**) `#seedhammer`

Filed 2026-08-16 by the S5.C implementer, landed by the controller.

The class check F-185 asked for now exists and is **a one-line call**, which is
the point — the cost of gating a screen is now trivial. What has not happened is
the sweep. Only this block's screens are measured; every other long modal in the
firmware carries the same unmeasured exposure F-185 recorded.

**Do not re-derive the mechanism when picking this up.** It is not a character
budget: capacity depends on how words WRAP (588 normalised chars of short-word
filler fit, while F-185's real refusal was cut at ~500), so the check compares
the drawn frame to the source string and binary-searches the cut point. It
carries the margin F-185 says its own per-screen fix lacked.

**OWNING PHASE FIXED, AND THE REMEDY RULED ON — operator, 2026-08-17.** The
owning phase was the bare string `S6`, which S6a has now passed; by the burndown
rule that made this overdue rather than deferred. It is **S6b's**, to be swept
before the hardware flash.

The operator's opening instinct was an affordance:

> "I think we need the down arrow available on screen everywhere there is a need
> to scroll down"

**CORRECTED 2026-08-17, same day, before any work started.** The first version of
this entry claimed the SH2 *cannot* scroll and that an arrow would promise an
impossible action. **That was wrong, and the error was mine (the controller's),
not the operator's.** What is true is narrower and the distinction is the whole
point:

- `cmd/controller/platform_sh2.go` emits `gui.PointerEvent` **exclusively** (its
  only two event constructions, both inside `processTouch`) and carries **zero**
  references to directional buttons. There are no *physical* arrow keys.
- **But directional input is synthesised from touch.** `Clickable.Next`
  (`gui/widget.go:48`) routes `ctx.Router.Next(ButtonFilter(c.Button),
  ButtonFilter(c.AltButton), PointerFilter(c))` at `gui/widget.go:70`, and
  carries press-and-hold auto-repeat written specifically for
  `case Up, Down, Right, Left`. So a `Clickable` bound to a direction **is**
  reachable by touch.
- **It already ships.** The StartScreen pager is `prevBtn := &Clickable{Button:
  Left}` / `nextBtn := &Clickable{Button: Right}` (`gui/gui.go:1931-1932`), with
  the code stating it binds arrows this way to keep the button path working
  "while making them touchable".

**So the real defect is that `Warning.Layout` never wired itself up.** It reads
bare `w.inp.Next(ctx, ButtonFilter(Up), ButtonFilter(Down))` — filters with **no
`Clickable` and no drawn hit area** — so nothing can deliver those events on an
SH2. The handler is dead *as written*, not dead *in principle*.

**Why fit-first still leads.** A modal that fits needs no affordance, and the
sweep is the prerequisite for ever restoring `fadeClip` (see below). But it is a
**sequencing** call, not a capability limit — and F-208 is correspondingly
cheaper than first written.

**F-208 IS ALSO IN S6b** (operator directive, restated on the corrected facts:
*"We need arrows on screen for scrolling when necessary"*). The two are
complements: this entry guarantees the **authored** copy fits; F-208 covers the
residue a fit gate cannot reach — bodies interpolating a descriptor, a policy, a
plate count, an operator-chosen label, whose runtime expansion is not fixed at
authoring time.

**~~SEQUENCING CONSTRAINT~~ — DISSOLVED by operator ruling R-I, 2026-08-17.**
This entry previously recorded that F-208's arrow layout had to be *decided
before this sweep sets its budgets*, because any change to body width re-opens
the wrap calculation and wrap is what decides capacity.

**R-I chose a layout that costs no body width** — the arrows float over the
body's top and bottom edges rather than taking a column, so the body clip stays
**417 px wide, unchanged**. The dependency is therefore gone: **this sweep and
F-208 can proceed independently**, and the fit measurements taken here stay
valid whenever the arrows land.

The original constraint is kept above rather than deleted, because it was true
of three of the four layouts considered and would bind again if R-I were
revisited.

So the S6b remedy is **the fit-gate sweep as filed**: guarantee every long modal
fits, so nothing is ever below the fold and no affordance is needed. The
affordance itself — touch scrolling, and only then the arrow — is filed
separately as **F-208**, owned **post-flash**.

**Two facts that make the sweep more urgent than "unmeasured exposure" sounds.**
`fadeClip` (`gui/gui.go:763`) is a **no-op stub** — `return o.Offset(image.Pt(0,
0))`, with the real mask commented out at `gui/gui.go:764`, immediately above
that return — so there is today no fade *and* no
clip: an overrun is a hard cut with no gradient hint, which is how F-185's frame
ended mid-word. And per F-95's closed entry, **restoring that mask before the fit
is closed would make things worse**, silently enforcing an overflow the stub
currently hides. The sweep is therefore a prerequisite for ever fixing the
renderer, not merely a hardening pass.

### F-193 — the same key is spelled two ways on two device screens (owning phase: **none — cross-cutting Minor, batches to the end**) `#seedhammer`

Filed 2026-08-16 by the S5.C implementer, landed by the controller.

The review screen shows the operator's **real** base58 xpub (`xpub6DkFA…`); the
restore doc's descriptor shows the **parent-fingerprint-zero reconstruction** of
the same key (`xpub6DXuQ…`), because md1 carries no parent fingerprint.
Pre-existing, shared with the supply path (`expandedToDescriptor`,
`gui/md1_expand.go`), and harmless for import.

**It is filed rather than fixed because the measurement changed a design
decision and should not be quietly re-litigated.** S5.C originally intended to
display the reconstructed form on the review screen; that would have asked the
operator to perform a comparison **they cannot perform** — one of this stage's
four normative prose constraints. The review now maps slots via the assembled
md1's bytes but *displays the operator's own strings*. Reconciling the two
renderings is a real improvement; reverting the review to the md1 form is not.

### F-194 — the pre-engrave review's first page cannot show a key while the §0.1 clause-3 header holds page one (owning phase: **none — cross-cutting Minor, batches to the end**) `#seedhammer`

Filed 2026-08-16 by the S5.C implementer, landed by the controller.

Measured, not predicted: page 1 ends at `"@0, no fingerprint:"` — the first key
falls on page 2. §0.1 clause 3 requires the assumption to be announced **on the
confirmation surface itself**, so the header must hold page one, and that is a
genuine cost paid deliberately rather than a bug.

A page-break-aware pager — one that keeps a slot's label and its key chunks
together — would fix it properly. It touches `confirmReviewScreen`, which is
**shared code**, so it wants its own scope rather than a drive-by edit.

### F-195 — **CLOSED 2026-08-17** (S6a: the census page states it outright — `gui/multisig_build_census.go:208`, "Seed: this set contains NO seed. It is watch-only") — a watch-only set never states outright that it contains no seed (owning phase: **`SPEC_multisig_build_repair.md` S6**) `#seedhammer`

Filed 2026-08-16 by the S5.C implementer, landed by the controller.

S5.C made the passphrase lines mode-safe and the inventory lists what was cut,
but **no line says plainly that a watch-only set holds no seed**. It is the same
family as the plan's "the backup must say what is NOT in it" requirement, which
that block implemented for the passphrase and deliberately did not scope-creep
to the seed.

The asymmetry matters years later: an operator holding a watch-only set and a
"Full" set side by side has the mode label to tell them apart, and a mode label
is exactly the kind of thing that gets copied onto the wrong tin.

---

### F-196 — a MIXED held set is not expressible through the screens (owning phase: **the spec — it is a model change, and earns its own R0**) `#seedhammer`

Filed 2026-08-16 by the S5 picker/verify implementer, **landed by the controller
2026-08-16 out of the whole-diff gate's I-10**, which is the only reason this
entry exists: the implementer drafted it in
`design/agent-reports/s5-picker-and-verify-implementation.md` §4 and could not
write this file from its worktree, and — unlike its sibling F-191 from the same
round — nobody landed it. Meanwhile `gui/multisig_build_slots.go:510-518` told
every future reader the gap had been "filed rather than smuggled in".

**It had not been.** A grep over this file for `SelfFromCard`,
`buildSelfSourceFlow`, `per-slot`, `mixed build`, `not expressible` and
`genuinely mixed` returned zero hits until this entry landed. That is the defect
worth remembering, more than the picker limit itself: **a claim of the form
"filed rather than smuggled in" is exactly the class of assertion a reviewer
inherits as a given.** It must be a grep, not a promise — so the comment now
cites this ID, and the ID resolves.

The limitation, in the implementer's own words:

> The derived-vs-`both` question is asked once and applied to every held slot
> (`gui/multisig_build_slots.go:494`). An operator holding `@0` on a card and
> `@1` from a seed alone cannot say so; both wrong answers are loud (gate
> refusal, or a derived slot announced on the "Key sources" review) but neither
> is what they meant. Expressing it means `buildPolicyParams.SelfFromCard`
> becoming a per-slot set, which touches `buildSlotSources`,
> `buildCosignerOrigins`, `buildSlotProvenance` and the pre-gather supply
> arithmetic.

**Not a funds-loss path**, which is why the gate rated it Important and not
Critical: every reachable branch either refuses loudly or falls back to an
all-derived configuration. The limit is in the PICKER, not the model —
`slotSource` is already per-slot and `assembleBuildPolicy` already reads the
mixture off the held-key set, so the later change is additive rather than a
rework. It contradicts SPEC §4.3's per-slot language
(`design/SPEC_multisig_build_repair.md:383`, "Every slot @0..@{n-1} carries
exactly one source, **chosen by the operator**"), which is why the owning phase
is the spec and not a stage.

---

### F-197 — **CLOSED 2026-08-17** (S6a: the abort ends the program — `gui/singlesig.go:177` returns on any non-`bundleEngraveDone`, so nothing below vouches for a set that was not fully cut) — the SINGLE-SIG engrave does not stop on an aborted set (owning phase: **`SPEC_multisig_build_repair.md` S6** — before the hardware cycle) `#seedhammer`

Found 2026-08-16 by the S5 whole-diff fold while landing I-12, and **not folded**:
`gui/singlesig.go` is outside the review's scope, and scope creep in a fold is how
a review round gets spent on unreviewed text.

`gui/singlesig.go:127` calls `bundleEngrave(ctx, th, "Engrave Single-Sig", cards)`
and discards the result. It is **I-12's defect verbatim**, on the flow the review
did not scope: an operator who aborts mid-set reads "Bundle Incomplete … This set
is not a usable backup yet", is then offered "Verify the engraved plates?" over a
set whose last card was never cut, and is finally shown `restoreDocFlow(...)` —
the artifact that is read years later, alone, presented as the last word of a run
the device just said produced no usable backup.

The fix is now one line, because the machinery landed with I-12: `bundleEngrave`
returns `bundleEngraveResult` as of commit `9f93362`, so this is

    if bundleEngrave(ctx, th, "Engrave Single-Sig", cards) != bundleEngraveDone {
        return
    }

**It needs the flow-level test that goes with it.** See
`TestSupplyAbortIsTheLastScreenOfTheProgram` for the shape: drive to the first
engrave picker, press Back, assert the program ENDS with neither the verify offer
nor the restore document drawn afterwards. **A call-site assertion alone is not
enough — that is exactly what let the multisig instance ship.**

### F-198 — **CLOSED 2026-08-17** (S6a, the cycle built for it: the label now reads `Full (seed + keys, NOT passphrase)` at `gui/multisig_build_census.go:387`, and the restore document always renders with a plate inventory, a seed statement and a passphrase statement) — **CRITICAL** — the SINGLE-SIG flow takes a passphrase into derivation, labels the result "Full (seed + keys)", and its restore document cannot mention a passphrase (owning phase: **`SPEC_multisig_build_repair.md` S6** — MUST land before the hardware cycle) `#seedhammer`

Named by the S5 whole-diff review as "adjacent, out of scope, file it" (C-3), and
filed by the fold with the harm **explicitly unverified**. **The controller
verified it 2026-08-16, and the answer is the bad one**, so this is not a label
tidy-up — it is C-3's defect on a third path, and single-sig is the more travelled
path of the two.

Measured against `gui/singlesig.go` at `s5-multislot`:

1. **The passphrase is a live derivation input.** `:64-72` takes one via
   `syswPassphraseFlow`, and `:90` passes it to
   `deriveSingleSigBundle(mnemonic, passphrase, …)`.
2. **The mode label is hard-coded.** `:80` carries
   `Choices: []string{"Full (seed + keys)", "Watch-only (keys)"}` — the raw
   literal, not `buildFullModeLabel(passphrase != "")`.
3. **The restore document cannot say it.** `:136` calls
   `restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path)` and
   `gui/singlesig_restore.go:119` has **no passphrase parameter at all**; `grep -i
   passphrase gui/singlesig_restore.go` returns nothing.

So a "Full" single-sig engrave with a passphrase cuts ms1 — which encodes the
**words only** — and hands the operator a document asserting the set is complete.
The passphrase is a required spending factor and is on no plate and in no
sentence. That is a permanently unspendable wallet discovered years later, which
is the exact harm S5 built `buildFullModeLabel` and the passphrase inventory line
to prevent.

**PRE-EXISTING, not an S5 regression** — verified: the literal is already in `main`
at `gui/singlesig.go:80`, introduced by `b100425` (T6a-2), and S5's only edit to
that file added a title argument to `bundleEngrave`. It therefore does **not** gate
the S5 merge. It **does** gate the hardware cycle: S6 flashes firmware an operator
engraves real backups with, and this path is reachable from the front door.

The fix mirrors commit `5f54737` (which wired the multisig SUPPLY path): use
`buildFullModeLabel(passphrase != "")` at `:80`, and give `restoreDocFlow` the
passphrase-bearing inventory line so the document states what it does not contain.

**Why this one is worth remembering beyond the fix.** Both the review and the fold
declined to assert the harm because neither had checked; the fold said so plainly
and wrote "check that FIRST … because 'somebody assumed' is what made C-3 survive
a whole stage." It then survived a *second* stage on a third path for the same
reason. **An unverified claim in a follow-up is a defect with a countdown, not a
note** — the answer cost one grep.

---

### F-199 — `verifyRefused` dead-ends on a CORRECTABLE readback (owning phase: **S6b** — corrected 2026-08-17 from the bare string `S6`, which S6a has passed; the S6b assignment was asserted only in F-204's body and the continuity doc, never in this heading) `#seedhammer`

Found 2026-08-16 by the B1..B5 fold while implementing B3, and deliberately
**not folded**: it is outside B1..B5, and folding an unreviewed control-flow
change into a fold is exactly what produces the text nobody has read.

`gui/multisig_verify.go:698-702` shows *"Read back one wallet-policy md1 AND the
operator key card(s) (mk1)."* — a screen that names precisely what the operator
should do — and returns `verifyRefused`, a verdict neither engrave caller
re-offers on. So the next screen is the restore document, headed *"If any of them
is missing, this backup is incomplete."* It is round-1 **B3's class at a third
site B3 did not name**, and it is reachable *before any seed is typed*: present
the mk1s and forget the md1, or bring one plate short.

**It needs a decision, not a reflex.** B3's fix scoped a `correctable` local to
the seed-entry and ms1-entry breaks. The same local would cover this site, but
`verifyRefused` **also carries two programmer-error refusals** (an empty
`expectedSlots`, a missing engraved md1) which must NOT loop — so the fix is
**per-site, not per-verdict**. Widening the verdict is the obvious move and the
wrong one.

**Provenance, verified by the controller rather than assumed — this is why it
files rather than gates.** The dead-ending message is **pre-existing in `main`**
(`gui/multisig_verify.go:82` there, from `b2c3231`, H1). `verifyRefused` itself
is **new in this cycle's own fold** (`9f93362`). Before S5 *nothing* retried, on
any path — so S5 did not regress this. It built a retry mechanism and did not
extend it to this site, which is an **incomplete new feature, not a regression**.
That is the same test F-191 was measured against ("pre-existing in kind and
unchanged by the S5 seam, so it did not gate the fold").

### F-200 — `engraveOnePlate`'s frame budget is harness-dependent, and the failure looks like a broken flow (owning phase: **none — cross-cutting Minor, batches to the end**) `#seedhammer` `#test-infra`

`gui/multisig_build_walk_test.go:443` gives one plate 4096 frames. **Measured on
the same plate**: the engraver closed at frame **881** under `runUITouchRaster`
and at frame **10585** under plain `runUI`, because virtual time in the synctest
bubble advances per idle point rather than per frame. `s5EngraveOnePlate`
(`gui/multisig_supply_passphrase_test.go:110`) carries 32768 for the same reason
and says nothing about it.

**The cost is misattribution, which is what makes it worth a number.** A future
test pairing `runUI` with `engraveOnePlate` fails with *"the engrave never closed
the engraver, so no plate was cut"* while the engrave is running perfectly — it
cost an hour inside this fold. Either make the budget a function of the harness,
or have the helper state its precondition. Recorded meanwhile at
`s5EngraveEveryPlate` in `gui/multisig_engrave_tail_walk_test.go`.

### F-201 — `multisigVerifyRetryLeadFor(res)` now covers three distinct verdict shapes with one sentence (owning phase: **none — cross-cutting Minor, batches to the end**) `#seedhammer`

Upgraded from round 1's Minor 4 by the B1..B5 fold. The retry lead *"Not every
plate is verified. Try again?"* was filed for narrating a FAILED verify as an
incomplete one. B3 adds a third shape: a first-seed refusal with **zero** legs.

The lead stays literally TRUE in all three — zero verified plates is "not every
plate" — so this is not a defect, which is why it is Minor. But one sentence now
covers *"some plates checked"*, *"a comparison disagreed"* and *"the seed you
typed fills no slot"*, and those want different next actions from the operator.
`TestBothEngraveFlowsDriveTheRetryLoop` already drives the offer through a seam
that can return any verdict, **so a verdict-specific lead is now assertable at
flow level** — the mechanism to fix this properly landed with B4.

### F-202 — **CLOSED 2026-08-17** (S6a: `gui/singlesig.go:163` gates the engrave behind `confirmReviewScreen(…, "Plates To Cut", buildPlateCensusLines(cards))`) — the SINGLE-SIG engrave shows no pre-engrave plate census (owning phase: **S6a — `IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`, in scope, not deferred**) `#seedhammer`

Found 2026-08-16 by the controller during the S6a recon, and **not previously
filed by anyone**. It is recorded here for the record and is being fixed inside
S6a — it entered that plan *before* its R0 gate, so it is reviewed rather than
folded in around the gate.

Measured: `buildPlateCensusLines` is called at exactly two sites, both multisig —

    grep -rn "buildPlateCensusLines" --include="*.go" gui/ | grep -v _test
    # gui/multisig_build.go:394 and gui/multisig.go:271 only.

So the single-sig operator commits to a 2- or 3-plate cut — minutes of
irreversible machine time per plate — with no count on any screen. Both multisig
paths show one, behind `confirmReviewScreen`, where Back is still free.

It is the same family as F-198/F-195 (**the flow is silent about the set**), one
step earlier: those are silences on the document read years later, this one is a
silence at the machine. S4 built the census for exactly this and wired it to two
of the three engraving paths.

Rated **Minor** and not Important because nothing is lost: an operator who runs
out of blanks has cut real plates but the encoders are deterministic, so a re-run
mints byte-identical plates (`TestReRunMintsByteIdenticalPlates`). The cost is
wasted blanks and wasted hours, not funds.

### F-203 — the two multisig paths give the plate census two different titles (owning phase: **none — cross-cutting Nit, batches to the end**) `#seedhammer`

Found 2026-08-16 alongside F-202. The same census screen is titled **"Plate
Count"** on the BUILD path (`gui/multisig_build.go:394`) and **"Plates To Cut"**
on the SUPPLY path (`gui/multisig.go:279`).

Neither is wrong and no funds ride on it. S6a adds a third instance and picks
`"Plates To Cut"` for single-sig — matching the other front-door path rather than
the one behind the EXPERIMENTAL warning — so the split becomes 2:1 rather than
1:1. Deliberately **not** unified inside S6a: renaming a screen title on a path
S6a otherwise does not touch is scope creep into a reviewed diff, and the whole
value of the census work is that it went through the gate.

### F-204 — a FAILED single-sig verify sends the operator to doubt the PLATES, where the multisig sibling says suspect the passphrase first (owning phase: **S6b — with F-199, before the hardware flash**) `#seedhammer`

Filed 2026-08-16 by the S6a R0 adversarial review (M-5), FILE-not-fix.

`gui/singlesig_verify.go:145` tells a failed verify to "Check the engraved
plates". The multisig sibling explicitly rules the other way --
`multisigVerifyNoSlotBody` (`gui/multisig_verify.go:151-165`): *"Check the
passphrase before you doubt the plates"*.

The asymmetry costs steel. The verify requires the seed RE-TYPED (SPEC 7.4, so
the engrave source is never compared against itself) and one wrong character
derives an entirely different wallet, so a mistyped passphrase at verify is a
common cause of a FAILED comparison on **correct** plates. The screen then sends
the operator to destroy them.

Not folded into S6a: S6a's C-1 work makes the *document* honest about the verify
outcome, which is a different surface from the on-screen remedial instruction,
and F-191 already established that the "a keystroke must not be reported as a
wrong wallet" family is its own line of work. It rides with F-199 in S6b because
both are single-screen verify-flow copy decisions on the same tail.

### F-205 — `backupWalletFlow` and `deriveXpubFlow` engrave passphrase-bound artifacts and say nothing about the missing factor (owning phase: **`key & password custody refinement`** — operator ruling 2026-08-17; NOT gating the hardware flash) `#seedhammer`

Filed 2026-08-16 by the S6a R0 adversarial review (N-1), FILE-not-fix.

`backupWalletFlow` (`gui/gui.go:2419-2432`) lets the operator engrave a
**passphrase-derived master fingerprint** onto a seed plate that carries only the
words. `deriveXpubFlow` (`gui/derive_xpub.go:344-354`) mints a passphrase-bound
mk1 with the same silence.

This is the **F-198 class without the vouching half**: a required spending factor
is absent from the engraved artifact and unmentioned, but neither flow produces a
restore document, so nothing asserts the result is complete. That is why it is
not C-1's class and does not gate S6.

It is filed rather than scoped because the remedy is not obviously the same one:
these flows engrave single artifacts rather than sets, so `buildFullModeLabel`
and the plate inventory have no natural home in them, and deciding what they
should say is a design question rather than a wiring change.

### F-206 — the pass line's ms1 clause stays singular on a multi-seed multisig verify (owning phase: **S6b, with F-199's verify-screen copy pass**) `#seedhammer`

Filed 2026-08-17 by the S6a whole-diff adversarial review (M-1), FILE-not-fix.

§4.7c's clause **B** is the fixed string `The ms1 secret you typed matched this
seed.` On a **full multisig** verify where the operator typed **two** ms1 secrets
for two seeds, the document still says *"the ms1 secret"* and *"this seed"*,
singular. The device's own already-reviewed screen gets this right — it says
*"the ms1 you typed for each seed"*.

**It UNDER-claims, so it is G2-safe and does not gate**: the line names fewer
comparisons than actually ran, and omission weakening a claim is the direction
this design is built to fail in. It is a legibility defect, not a truth defect.

Owned by S6b because that cycle already opens the verify-flow copy on the same
tail (F-199), and because the fix is a plural rule over `passRecord.legs` /
seed count rather than a new recorded fact — **no new field, so it does not
reopen NG1.**

### F-207 — `singleSigReadbackCards` silently drops a card of an unexpected kind (owning phase: **none yet — pre-existing, NOT gating the hardware flash**) `#seedhammer`

Filed 2026-08-17 by the S6a whole-diff adversarial review (N-1), FILE-not-fix.

The readback accounting recognises the card kinds it expects and **silently
ignores any third kind** rather than refusing or counting it.

**Pre-existing, and it produces no false claim**: an unrecognised card cannot
satisfy an expected slot, so the accounting that follows either falls short and
takes an adverse exit or is unaffected. Nothing on the document asserts anything
about it. Filed so the behaviour is written down rather than rediscovered — the
question worth answering later is whether a card the device cannot classify
should be an adverse observation rather than a non-event.

### F-208 — a long modal has NO affordance saying more text exists, and `Warning` never wired its scroll to anything touchable (owning phase: **S6b** — operator directive 2026-08-17, REAFFIRMED on corrected facts) `#seedhammer`

Filed 2026-08-17 on an operator directive:

> "I think we need the down arrow available on screen everywhere there is a need
> to scroll down"

**The intent is right and the mechanism does not exist yet.** Three measured
facts, in the order that matters:

1. **The art is already compiled in and used nowhere.**
   `gui/assets/arrow-down.alpha.png` ships as `assets.ArrowDown`
   (`gui/assets/embed.go:13`) and has **zero** usages outside its own
   declaration. So do `arrow-up`, `arrow-left`, `arrow-right`.
2. **The machine has no PHYSICAL directional buttons, but it can still scroll.**
   `cmd/controller/platform_sh2.go` emits `gui.PointerEvent` exclusively and
   references no directional button — yet `Clickable.Next` (`gui/widget.go:48`)
   routes `PointerFilter(c)` alongside its button filters (`gui/widget.go:70`)
   and carries press-and-hold auto-repeat for `case Up, Down, Right, Left`.
   Directional input is **synthesised from touch by a drawn hit area**.
3. **The precedent already ships.** The StartScreen pager is
   `&Clickable{Button: Left}` / `&Clickable{Button: Right}`
   (`gui/gui.go:1931-1932`), touch-driven today.
4. **So the actual gap is wiring, not capability.** `Warning.Layout`
   (`gui/gui.go:380`) reads bare `ButtonFilter(Up)`/`ButtonFilter(Down)` with no
   `Clickable` and no drawn target, so nothing can deliver those events. The
   work is: give `Warning` two `Clickable`s bound to `Up`/`Down`, draw them with
   the `ArrowDown` asset, and the existing repeat logic supplies held-scroll.
   **No gesture handling is required.**

**A correction is recorded here deliberately.** This entry first claimed the SH2
*could not* scroll and that an arrow would promise an impossible action. That was
a controller error, corrected the same day before any work began, and it is left
visible rather than quietly rewritten because it changed the framing the operator
ruled against — the ruling was re-offered on the corrected facts.

**Ordering is the whole content of this follow-up.** Touch scrolling first, then
the arrow, and neither before the F-192 fit sweep has closed — because
`fadeClip` (`gui/gui.go:763`) is a no-op stub, and restoring the real clip mask
while any body still overflows would *silently* delete the sentence telling the
operator to stop (F-95's closed entry measures exactly that: `maxScroll = 19 > 0`
on §10.2.3's warning, the 19-px window holding *"the encrypted part has been
REMOVED. Do not continue."*).

**There is a precedent to copy, named in F-95:** bind `Warning`'s scroll to
`Clickable`s with `op.Input` hit areas, *"the same fix the StartScreen pager
took"*. So this is not novel input work.

**MOVED TO S6b — operator directive 2026-08-17, second statement, on corrected
facts:**

> "We need arrows on screen for scrolling when necessary"

The first ruling parked this post-flash, but it was made against the controller's
**wrong** claim that the device could not scroll at all. Re-offered on the
corrected facts, the operator restated the requirement. **It is in scope for
S6b.** Both prior statements are kept above so the record shows the requirement
survived the correction rather than being introduced by it.

**F-192 and F-208 are complements, not alternatives — and the word "necessary"
is what joins them.** F-192 guarantees the *authored* copy fits, so on those
screens no arrow should ever appear. F-208 covers the residue F-192 cannot
reach: bodies whose length is **not fully controllable at authoring time** —
anything interpolating a descriptor, a policy, a plate count, an operator-chosen
label. A fit gate proves a string fits; it cannot prove every runtime expansion
of it does.

### What "when necessary" requires, and the trap in it

The arrow must render **iff** content extends past the fold — an always-on arrow
on a screen with nothing below it is the same class of lie in the other
direction. The predicate is `maxScroll > 0`.

**That predicate is currently UNRELIABLE, and this is the gating design
problem.** `Warning.Layout` derives `maxScroll = bodysz.Y - (bodyClip.Dy() -
2*scrollFadeDist)` (`gui/gui.go:409`) — but because `fadeClip` is a no-op stub
(`gui/gui.go:763`), the body renders past `bodyClip.Max.Y` regardless. F-95
measured exactly this divergence: `maxScroll = 19 > 0`, i.e. the widget believed
a line was hidden, **while nothing was actually cut off**. Wiring an arrow to
that predicate today would show it on a screen with nothing below the fold.

**REVISED by operator ruling R-E, 2026-08-17 — the clip mask is NOT restored in
S6b.** The original three-step order below assumed it would be. It is kept
because the *dependency* it records is still true; what changed is that step 2
now happens **after** this cycle, which forces a compromise into step 3.

1. **F-192's fit sweep** — the authored copy fits. **In S6b.**
2. ~~**Make the geometry honest** — restore the real clip mask~~ — **DEFERRED
   past S6b (R-E).** Restoring it would start enforcing a clip nothing enforces
   today, silently deleting text that currently draws.
3. **The conditional arrow — in S6b, but it CANNOT use `maxScroll > 0`.**

**Why step 3 is harder than it looks now.** With the mask stubbed,
`maxScroll = bodysz.Y - (bodyClip.Dy() - 2*scrollFadeDist)` (`gui/gui.go:409`)
reserves 32 px of fade margin that is never drawn as fade, and the body is not
clipped to `bodyClip` at all (F-95 measured it drawing to y=317 against
`bodyClip.Max.Y = 314`, in a 320-px panel). **So content can satisfy
`maxScroll > 0` while being entirely visible**, and an arrow keyed to it would
appear with nothing below the fold — a false statement by the UI, in the
opposite direction, which R-D forbids just as firmly.

**So the predicate must be defined against what is ACTUALLY VISIBLE (the panel),
not against `bodyClip`, for as long as the mask stays stubbed** — and it is
therefore **coupled to R-E**: whatever S6b writes must be revisited when the
mask is restored. Say so in a comment that names R-E, or it becomes exactly the
kind of stale safety argument this project has been bitten by before.

**S6b owes a test that the two agree:** `maxScroll > 0` on a screen where
nothing is actually hidden is a **finding**, not a rounding error. That test is
the cheapest guard on the divergence R-E deliberately leaves in place.

### ~~Open question~~ — ANSWERED by operator ruling R-I, 2026-08-17

**The arrows float over the body's top and bottom edges**, centred, over the
16 px fade zone — each with a background chip and an enlarged invisible touch
target. Not in the nav gutter, not in a new column. Full argument and the four
measured alternatives: `REQUIREMENTS_s6b_pre_flash_cycle.md` §2bis R-I.

Measured geometry it rests on: panel 480×320; nav slots 53×53 at **y = 44 / 133
/ 223**, x = 427–480; body clip (6,44)–(423,314) = **417 wide**;
`assets.ArrowDown`/`ArrowUp` are **15×9**.

**Why not the nav gutter:** `ErrorScreen` leaves **two** slots free (top,
middle) but `ConfirmWarningScreen` only **one** (middle) — so the arrows could
not sit in the same place on both, and one screen could not host both.

**And it dissolves the F-192 coupling:** floating costs no body width, so
F-192's sweep no longer waits on this and its fit measurements stay valid.

**Three implementation constraints, carried into the spec:**

1. **Not through `layoutNavigation`.** It computes
   `idx := int(clk.Button - Button1)` into a `[3]int`, and `Up`/`Down` sort
   *before* `Button1`, so they index **negative**. This binds any layout.
2. **Hit area ≠ drawn icon.** The nav button already separates them —
   `op.Input(buf, t).Clip(...)` sets the touch region independently of the mask
   drawn — so a 15×9 icon can carry a finger-sized target.
3. **The chip is not optional.** The arrows sit where body text currently draws
   (`fadeClip` clips nothing), so without a background they can land on a glyph.

**It also rescues work orphaned inside a CLOSED entry.** F-95's *"What is still
owed, and the order matters"* paragraph is the origin of most of the above, and
F-95's heading reads **CLOSED 2026-08-11** — so by this file's own convention
(status lives in the heading) no sweep of open items would ever have surfaced it.
F-208 is now the open handle for it.

### F-209 — **CLOSED 2026-08-18, same day, inline** (S6b: `gui/s6b_modal_fit_sweep_test.go` gains the third arm — 177 raw chars, 146 drawn, headroom 418 — in fork commit `e3ac212`; the staleness it exposed swept in `1cec141`) — F3's new failure-copy arm is missing from the modal-fit sweep its two siblings are in (owning phase: **S6b — filed and closed within it**) `#seedhammer`

Found by the P9 fold verification (`design/agent-reports/s6b-p9-fold-verification.md`,
Minor N1), the sonnet pass over the failure-states fold. P9's F3 fix turned
`singleSigVerifyFlow`'s failure-copy `switch` from two arms into three; the fit
sweep's own stated gating rule covers the new arm and both siblings were already
in the table, so this was an omission, not a judgement. **Not a live defect** —
the verification proved the arm draws in full with 418 characters of headroom
before reporting it.

**Closed inline rather than deferred**, because its owning phase was closing that
hour and the fix is one table entry. Verified byte-identical to the production
call site programmatically (177 == 177) rather than by eye, and mutation-tested
RED before GREEN.

**What it actually bought was the defect underneath it.** Forcing the new entry
RED printed the class check's own failure message, which said the lost text is
unreachable *"because this modal's scroller is bound to buttons the SH2 does not
have."* **P5 had falsified that earlier in the same diff** — `Warning` now draws
touchable scroll arrows, and `ErrorScreen` embeds `Warning` by value
(`gui/gui.go:317`), so every `showError` modal inherited them. Six sites across
five files still asserted the old world, including a live `t.Errorf` in
`gui/unlock_flow_test.go` stating outright *"Warning has no touch scroll"*.

Swept in fork commit `1cec141`, classified per site rather than string-replaced:
three were **falsified** (they describe the `Warning` modal, which now scrolls),
four were **imprecise** (they describe screens that are not `Warning` — `ftProofBody`,
`ppConfirmBody`, `ppPassProofBody` each build their own body ops — so the
conclusion always held and only the reason was wrong; restated on the stronger
true ground that those screens have no scroller at all). **Every gate was kept**,
on the weaker but sufficient footing that a funds-critical warning must be on the
first frame, not one tap below the fold. Reachable-by-scrolling is not the same
as read.

New test `TestErrorScreenModalCarriesTheScrollArrows` (`gui/scroll_arrows_test.go`)
backs the corrected messages: P5's own `TestGate51ArrowActuallyScrolls` drives
`Warning` directly, and nothing drove the `ErrorScreen` production actually shows.
Mutation-tested two ways after passing first try in 0.00s.

**The lesson is the lens, not the finding.** Neither fable pass looked for *text
this diff made false elsewhere* — both read what the diff wrote, and the falsified
prose sat in files the diff never opened. A scoped sweep for that class was
dispatched on discovery; see `design/agent-reports/s6b-falsified-elsewhere.md`.

### F-210 — the operator journeys cannot be regenerated: all three transcripts read intermediates nothing writes, and the tool versions have moved under them (owning phase: **the arbitrary-`tr()`/`wsh()` cycle — before it leans on the pathological journey**) `#seedhammer` `#test-infra`

Found 2026-08-18 by running `design/journeys/transcript.sh` on the operator
(5-of-12 `wsh(multi(…))`) journey, at the operator's request, to check whether it
still reproduced before the next cycle relied on it. It does not.

**Measured, not inferred:**

| | committed `transcript.txt` | fresh run |
| --- | --- | --- |
| non-zero exits | **1** | **9** |
| `mk` | 0.12.1 | **0.13.0** |
| `ms` | 0.14.1 | **0.16.0** |
| `me` | 0.5.1 | **0.6.0** |

**Defect 1 — the scripts consume files they never produce.** Six intermediates
are read across the three transcripts and **none exists on disk**:
`md-encode-raw.txt`, `mk-encode-raw.txt`, `ms-encode.txt`, `md1.txt`,
`manifest.json`, `sysw-public.bin`. Read-vs-write counts per script:
`transcript.sh` 9/2, `transcript_pathological.sh` 5/1, `transcript_payload.sh`
1/0. So `grep '^md1' out/md-encode-raw.txt` fails, every downstream command
receives an empty argument, and the run cascades:

```
md: codec error: codex32 decode error: string does not start with HRP md1
[exit 1]
```

`out/` is untracked, so nothing carried those files across sessions.

**Defect 2 — the committed transcript was made somewhere that no longer
exists.** Line 23 of `transcript.txt` reads:

```
$ cat /tmp/claude-1000/…/22fd28a4-…/scratchpad/journey/inputs/wallet-policy.txt
```

a scratchpad path from a dead session. The script now uses `$W/inputs`, so the
artifact of record was produced by a script that no longer exists in this form.

**Why it matters more than a stale doc.** `README.md` opens *"Nothing in these
documents is illustrative"* — every CLI block real stdout, every screenshot the
emulator's own framebuffer. That was true when written and the PDFs still assert
it, while the generator behind them cannot produce them. **A reproduction path
nobody re-runs rots while its artifact keeps vouching for it.** The PDFs and two
of three `transcript*.txt` are tracked; `out/` is not — so the vouching half is
in git and the proving half never was.

**Owning phase assigned deliberately.** This does not gate the hardware flash and
did not gate S6b. It DOES gate the arbitrary-`tr()`/`wsh()` cycle, because the
pathological journey — 11 keys, all four timelock kinds, a sha256 hashlock — is
the only journey exercising exactly the complex shapes that cycle is about. Fix
it before relying on it, not after.

**Two repairs, and the choice is a scoping decision, not a detail:** either make
each script produce its own intermediates (self-contained, slower, regenerates
anywhere) or commit the intermediates as fixtures (fast, but re-creates the same
decay one version bump later). The version drift above argues for the first.

**STATUS 2026-08-20 — the transcripts are FIXED; the gap is now screenshots and
a missing print step.** Re-measured by running everything, against freshly built
binaries (`md` 0.13.0, `mk` 0.13.0, `ms` 0.16.0, `me` 0.7.0 — all four rebuilt
first, so this measures the tree and not a stale binary):

| script | non-zero exits, fresh vs committed | diff vs committed transcript |
| --- | --- | --- |
| `transcript.sh` | **1 vs 1** | **0 lines — byte-identical** |
| `transcript_pathological.sh` | **3 vs 3** | **0 lines — byte-identical** |
| `transcript_payload.sh` | 2 (both deliberate refusals) | no committed transcript |

The remaining non-zero exits are the refusals each journey exists to show — a
`pass:` record whose body is not hex, `sysw show` on a wiped region, and the
three the pathological header names. Defect 1 was repaired by `runcap()` (the
capture mechanism these scripts never had); Defect 2 is gone with it — no
`claude-1000`/scratchpad path survives in either tracked transcript.

`derive-pathological-keys.sh`, recorded elsewhere as orphaned, also RUNS: it
rewrote all 11 `inputs-pathological/keys/*.xpub` **byte-identically** to the
committed ones.

**What is actually left, and it is not the transcripts:**

1. **Screenshots.** `shots/` is gitignored on purpose (size), so the proving
   half was never in git — the structural point above, still true. Missing
   today: **13** for the pathological journey (7 `a*` seed-entry screens, 6
   `b*` engraving screens and plate overlays) and **19** for the operator
   journey. Both builders REFUSE rather than emit a draft, which is the right
   behaviour and is why neither produced a stale PDF.
2. **No committed capture path.** `README.md` documents `shot_server.py` as the
   receiver the emulator POSTs frames to — but **nothing in `cmd/emu/` posts
   them.** The capture was ad-hoc console code in a session that no longer
   exists. That is the same rot as Defect 1, one layer out: the driver was never
   committed, so it decayed to nothing.
3. **The print step (half of F-156).** `build_pdf.py` and
   `build_pdf_pathological.py` write HTML and stop; the published PDFs came from
   a manual headless-Chrome print that lives nowhere in this repo.
   `build_pdf_payload.py` DOES print its own PDF — and that journey consequently
   regenerates **end to end today** (transcript + 0 missing shots + PDF written,
   exit 0). It is the worked example the other two should follow.

**DONE 2026-08-20 for the PATHOLOGICAL journey — the one this entry gates the
`tr()`/`wsh()` cycle on.** It regenerates end to end from committed inputs:

```sh
bash transcript_pathological.sh > transcript_pathological.txt 2>&1
python3 capture_pathological.py     # rebuilds emu.wasm, drives it, writes shots/
python3 build_pdf_pathological.py   # writes the PDF
```

`cmd/emu/shots_pathological.js` (seedhammer `f763067`) is the capture driver that
never existed, and `capture_pathological.py` runs it. Rebuilt output: 15 pages, 4
plate captions, and `pdftotext | grep -ci missing` = 0.

Three defects the capture found on the way, each of which had REPORTED SUCCESS —
worth recording because they are the shape of this whole follow-up:

- the seed picker has **five** rows, not two, so a computed row typed the seed
  into `Input m*1 string` three screens away;
- the plate `viewBox` is in **device units** (544000²), and using it as a canvas
  size makes `toDataURL` return `"data:,"` — which the shot server writes as a
  **zero-byte PNG with a 200 OK**, four times, with the driver reporting `ok`;
- sampling by wall clock made `b6-plate` and `b8-plate` **byte-identical**, which
  would have put one picture in the document twice under two captions. Sampling
  is on `shToolpath.summary().steps` now, with a >2% drift check against the
  measured plan.

And the captions are captured data rather than prose — they had **already
drifted**: the sample labelled "three words down" shows all twelve words cut.

**CLOSED 2026-08-20 — all three journeys regenerate end to end.** The operator
journey got `cmd/emu/shots_operator.js` (19 shots) and `capture_operator.py`, and
`build_pdf.py` now prints its own PDF, which closes the other half of F-156 too.

Its capture had one finding the pathological one did not: **the emulator must be
slowed while it cuts.** `steps` only advances when the engraver yields, and at
the default walk pace this short text plate yields so rarely that four samples
landed at 51/61/91/100% — the start of the cut was unobservable at any threshold.
`shPace(128)` yields often enough for 12/35/71/100%, and changes when control
returns rather than what is cut.

Two more claims turned out false and are now rendered from the walk: the carousel
has **ten** programs, not the eight the document counted and named, and the font
caption had `font/sh` and `font/constant` the wrong way round.

Verified from a wiped `shots/`: transcript diff 0, 19/19 shots, 14 pages,
`pdftotext | grep -ci missing` = 0 — and the pathological journey still rebuilds
from the same clean state, so the two captures do not tread on each other. The plate captions must come FROM the capture — `#plate-caption`
already emits `head X,Ymm`, which is exactly what the PDF currently hardcodes as
prose beside the image.

### F-212 — ~~Go and Rust compute DIFFERENT `WalletPolicyId` when the origin is elided~~ **CLOSED 2026-08-20** `#seedhammer` `#security` `#codec`

**Found 2026-08-20 by the R3 keyed conformance vectors, on their first run.** This
is precisely the class those vectors were built to find, and no keyless corpus
could have found it: the divergence is in how KEYS enter a hash.

**Measured on ONE wallet encoded two ways** — same template, same xpub, same
fingerprint:

| origin | Rust | Go |
| --- | --- | --- |
| explicit `84'/0'/0'` | `c79039c5…` | `c79039c5…` — **agree** |
| **elided** | `c79039c5…` | `260f334a…` — **DIVERGE** |

Rust's policy id is **stable across origin-elision**; Go's is not. Reproduced on
four of six keyed vectors — every one whose origin is elided — while both
explicit-origin vectors (`keyed_tr_with_leaf`, `keyed_tr_depth2`) agree exactly.

**Both sides call their behaviour deliberate, and they contradict each other.**

- Rust `crates/md-codec/src/identity.rs`: canonical-fills an empty origin before
  hashing so the id "honors its documented *stable across origin-elision*
  invariant" (comment L14).
- Go `md/walletpolicyid.go:138-145`: resolves the origin "**AS-IS — NO
  canonicalOrigin fallback (the deliberate divergence from the display accessor,
  R0-I2)**", and calls an elided shared path "a legitimate empty origin".

Under the **Rust-primary rule** Rust is normative, so the port is the side that
must converge — but R0-I2 chose this deliberately, so the ruling is not mine to
make silently. It is also not obviously free: `WalletPolicyId` binds mk1 cards to
a wallet, so changing it on-device changes what previously engraved cards verify
against.

**Why it matters beyond conformance.** The device computes this id to bind key
cards. An elided-origin keyed card gets one id on the device and a different one
from the toolkit, so a cross-tool verify would report a mismatch on a wallet that
is actually correct — or, read the other way, agreement would stop meaning what
it appears to mean.

**Pinned, not skipped.** `md/conformance_keyed_test.go` asserts the gap's exact
shape: an elided-origin vector must still diverge and an explicit-origin one must
still agree, so the test fires when either changes — **including when the
divergence is fixed**, which is when the arm should be deleted.

**CLOSED 2026-08-20 — the fork converged** (`seedhammer` `90f4624`).

**Operator ruling:** *"We can ignore cards already engraved. They don't exist. We
can establish seeds and derive / replace keys for any journey if we desire."* That
removes the only real cost of changing an on-device identity, so the Rust-primary
rule decides it outright.

**And R0-I2 turned out not to be the argument on the other side.** R0-I2 is a
different ruling — it says `OriginPath` is a `bip32.Path` with in-band hardening
and drops an `OriginHardened []bool` field. The ruling that governs this fallback
is **R0-I1**, and it *requires* it: *"canonicalOrigin(d.tree) when both are
empty"*. So the divergence was justified by a mis-citation, not by a competing
position. Checked before overriding it.

`resolveOriginRaw` has exactly one caller, so the change touches the policy id
alone; the key-stable template id already agreed and is untouched.

**The part worth remembering:** nothing in the fork's own suite caught or depended
on the old value — 887/887 gui and 50 packages pass either way. The divergence was
invisible to every test the fork had, and the cross-language vectors saw it on
their first run. That is the argument for keyed conformance vectors in one line.

### F-213 — ~~`md encode` mints a card carrying a key for a placeholder the template never uses~~ **CLOSED 2026-08-20** `#codec` `#funds-safety`

**Found 2026-08-20** while adding a `wsh(or_b(...))` conformance vector: the
vector was accidentally given three keys for a two-key template, Rust encoded it
happily, and the Go port refused to read the result.

**Reproduced at the CLI, both directions:**

```sh
# @2 is not referenced by this template. Rust encodes it anyway:
md encode 'wsh(or_b(pk(@0/<0;1>/*),s:pk(@1/<0;1>/*)))' \
    --key @0=<xpub> --key @1=<xpub> --key @2=<xpub> --path "48'/0'/0'/2'" --force-chunked
# -> a valid-looking 4-chunk card

# the fork, reading that card:
md: override order violation
```

The card carries `n = 2` while its Pubkeys TLV holds three entries (`@0`, `@1`,
`@2`). The Go re-encode path walks the TLV against `n` and refuses; Rust never
checks that the supplied key indices are in range for the template.

**Why it matters beyond tidiness.** The operator-visible failure is
**engrave-then-discover**: the host tool accepts the command, prints a card, and
the card is unreadable by the device that is supposed to restore it. Nothing
between those two moments says anything is wrong, and the moment in between is
where steel gets cut. A stray `--key` is an easy typo — `@2` for `@1` on a
two-key policy — and it produces a *plausible* card rather than an error.

**Which side is wrong: Rust.** A key for a placeholder the template does not
reference is not meaningful data — there is nothing for it to bind to — so
minting it is the defect, and the port's refusal is correct. Under the
Rust-primary rule the fix lands in `md-cli` / `md-codec` first, with a vector:
reject `--key @N` where `N >= n` at encode time.

Note this is a NARROWING of admission, not a widening, and the operator has
ruled that no engraved cards exist to protect — so the usual compatibility
objection does not apply.

**CLOSED 2026-08-20** (`descriptor-mnemonic` `bf028ad0`). `md encode` refuses at
encode time, naming both the stray slot and the ones that exist:

```
md: --key @2 does not appear in this template (it uses @0, @1); a key bound to
no placeholder cannot be encoded, and a card carrying one is rejected on decode
```

**Fail-closed rather than dropped**, because a key bound to no placeholder is
meaningless rather than partial: an operator who typed `@2` meant something, and
guessing which slot is not the tool's job. **The check is the placeholder SET,
not a count** — `wsh(or_b(pk(@0),s:pk(@3)))` uses two placeholders that are not
`@0`/`@1`, so counting would accept two keys while binding one to nothing.
Fingerprints get the same treatment. 766/766 pass, so nothing relied on the old
laxity.

The conformance vector was corrected to two keys rather than left as an exercise
of the disagreement: a vector's job is to pin agreed behaviour, not to encode an
open dispute.

### F-211 — `bip39.RandomWord()` is an exported CSPRNG-backed word generator compiled into the firmware, on a device that is not supposed to generate seeds (owning phase: **next `#seedhammer` cycle**) `#seedhammer` `#security`

**Surfaced 2026-08-19** by an operator-directed audit of every RNG call site
across the constellation. Operator statement: *"sh2 seed generation is not
supposed to exist."*

**PROVENANCE — ANSWERED: we did not add it. It is original SeedHammer firmware.**

- `bip39/bip39.go:371 func RandomWord() Word` reads `crypto/rand` and returns a
  random BIP-39 word.
- Introduced upstream in `3398580`, **2023-03-22**, author `seedhammer` — about
  two years before this fork existed. Still present upstream today at
  `upstream/main:bip39/bip39.go:266`, byte-identical in behaviour.
- The only fork commit touching it is upstream's own `74b8d00` (2024-01-01,
  "bip39: store words more efficiently"), inherited via merge.

**So the fork neither introduced nor wired up seed generation.** That is the
question asked, and the answer is clean.

**WHAT IS ACTUALLY OPEN — the capability is compiled in but unreachable.**

Measured: **no production code calls it, in the fork OR upstream.** Every
reference is a test —

```
bip39/bip39_test.go:98      mnemonic[j] = RandomWord()
engrave/engrave_test.go:34  mnemonic[i] = bip39.RandomWord()
```

— both generating random mnemonics as test material. `git grep` for
non-`_test.go` callers returns nothing in either tree.

So it is **dead code in a production package**: an exported, CSPRNG-backed
generator sitting in the `bip39` package's public API, on a device whose design
intent forbids the feature. `bip39` is linked into the firmware (`gui/derive.go`,
`gui/multisig_derive.go`, `gui/ms1_decode.go` all import it), so the symbol is in
the build graph even though nothing reaches it.

**Why file it rather than shrug.** The risk is not that it runs — nothing calls
it. The risks are that (a) it is *exported*, so wiring it up is one line and
reviews as ordinary API use rather than as adding seed generation; and (b) a
device that must not generate seeds is stronger if it *cannot*, not merely if it
*does not*.

**OPEN QUESTION, not yet measured:** does the symbol survive TinyGo's
dead-code elimination into the shipped firmware binary? If DCE drops it the
concern is API-surface only; if it does not, the device ships an unused CSPRNG
path. **Check before choosing a remedy** — this is exactly the "verify the
mechanism, don't argue about it" rule.

**Remedy options, cheapest first:**
1. Move `RandomWord` into a `_test.go` file (or an internal test-only package).
   It is only ever test material, so this is a relocation, not a removal, and
   `_test.go` is never compiled into the firmware.
2. Gate it `//go:build !tinygo`, which keeps host tests working and provably
   removes it from device builds.
3. Leave it and document the deliberate choice — the weakest option, and only
   defensible if DCE is shown to drop it.

Option 1 or 2 also makes the guarantee **testable**: a build-tag or symbol check
can assert the device binary contains no seed-generation path, turning an
intention into a gate.

**Upstream-facing:** if this is fixed, it is a candidate small PR to
`seedhammer/seedhammer` — the same dead export exists there, and the fix is
strictly an improvement for a device with the same design intent.

---

### F-214 — ~~a card this constellation can ENGRAVE has addresses the DEVICE cannot derive~~ **CLOSED 2026-08-21** `#seedhammer` `#funds-safety`

`md encode` accepts `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(144)))` and
`md address` derives its addresses. The device cannot: `md.TapLeavesChunks`
describes `pk`, `multi_a` and `sortedmulti_a` leaves only, so anything else
returns `ErrTapLeafUnsupported` and the operator gets "Complex policy — display
only".

**Not a defect in what shipped.** The refusal is correct — an approximated tap
leaf is a valid-looking address for a script nobody can spend from, which is
strictly worse than showing nothing. The gap is that the constellation's own
tools disagree about a card it happily produces: the host can verify the backup
and the machine that engraved it cannot.

Vendored as `md/testdata/vectors/gap_tr_leaf_and_v.*` in the fork, with Rust's
addresses alongside as ground truth, and **pinned by shape**: the test asserts
*this must refuse*, so it fails with "THE GAP IS CLOSED" the moment the emitter
grows — rather than going quiet and letting a capability arrive unnoticed.

The same shape covers `wsh` fragments outside the emitted set. The wsh emitter
is much further along (`or_b`/`or_c`/`or_d`/`and_b`/`thresh` and the wrappers all
landed in `6585115`), so the tap-leaf side is the narrower and more urgent half.

Sized honestly: each new leaf kind needs a Script builder, a use-site-correct
derivation, a conformance vector **that discriminates** (see `keyed_tr_multi_a`
— the corpus had no order-sensitive tap leaf at all until 2026-08-20), and a
mutation pass. It is not a one-liner, and it is not urgent while the refusal
holds.

**CLOSED 2026-08-21** (`seedhammer` `e2f1ec3`, vector `descriptor-mnemonic`
`276df02a`). The device EMITS tap leaves now instead of describing them.

**The fix was to delete the vocabulary, not extend it.** A tap leaf is ordinary
miniscript and the segwit-v0 emitter already walked it; only two things differ,
and both now live in one `emitEnv.tap` flag — x-only 32-byte keys, and BIP-342's
CHECKSIGADD in place of the disabled `OP_CHECKMULTISIG`. `multi` is now refused
under taproot and `multi_a` outside it, rather than either being translated.

**The pathological wallet derives, and matches Rust on every address.** It is
vendored as `keyed_tr_pathological`: depth-3 tree, two hashlocks, both timelock
flavours, `multi_a` at 3/2/2/1, NUMS internal key, eleven distinct accounts.

**One defect found in the wiring, worth carrying forward.** The GUI first read
the internal-key facts from `TapLeavesChunks` while tolerating its error — but
that error path returns `0, false, nil, err` *before* reading them. Every shape
the describer could not name silently got `isNUMS=false` and index 0, derived
the internal key from `@0`, and produced a **well-formed wrong address**. Caught
only against Rust, and the pattern named it: `multi_a` alone matched, all three
`and_v(v:…, multi_a(…))` shapes did not — exactly the ones the describer could
not name. **Never consume values from a call that returned an error.**

**Still refused, and now the pinned gap:** `pkh()` in a tap leaf. The primary
derives it; emitting it needs a hash160 of the derived key, which would pull
RIPEMD-160 into a codec that does no key work at all. `gap_tr_leaf_pkh` carries
Rust's addresses so a future fix has ground truth.

### It blocks the constellation's own flagship wallet (measured 2026-08-20)

Asked whether the taproot pathological wallet round-trips, and the answer sharpens
this entry considerably. `design/journeys/inputs-pathological/wallet-policy-tr.txt`
is the taproot form of the four-tier degrading vault:

```
tr(NUMS,{and_v(v:after(1000000),and_v(v:sha256(a84d…),multi_a(3,@0,@1,@2))),
        {and_v(v:after(1893456000),and_v(v:sha256(a84d…),multi_a(2,@3,@4,@5))),
        {and_v(v:older(65535),multi_a(2,@6,@7)),
         and_v(v:older(4255898),multi_a(1,@8,@9,@10))}}})
```

| | |
| --- | --- |
| host round-trip | **works** — the 3 committed chunks decode back to the policy exactly, exit 0 |
| device address derivation | **refused** — `TapLeavesChunks` → `ErrTapLeafUnsupported`, 0 leaves |

Every one of its four leaves is `and_v(v:…)` wrapping a timelock or hashlock, so
**not one** is in the described set (`pk` / `multi_a` / `sortedmulti_a`). This is
not an invented edge case: it is the wallet this repo calls *the* pathological
example, the one the whole 182-symbol chunking story is about, and in taproot form
the device can say nothing about where it pays.

**Two adjacent facts found on the way:**
- Re-encoding it today does **not** reproduce the committed cards — it needs
  `--path`, and without one `md decode` exits **4** (partial decode, origin
  unspecified) rather than failing loudly.
- `backup-strings-tr.txt` and `wallet-policy-tr.txt` have **zero consumers** — no
  transcript writes them, no builder reads them. Confirmed by an earlier agent
  report with a positive control (4 hits for `backup-strings`, 0 for
  `backup-strings-tr`). They are inputs to a journey that was never written, the
  same class as F-156 / F-210.

---

### F-215 — ~~the template-engrave shape guard refuses two shapes that have both moved out from under it~~ **CLOSED 2026-08-21** `#seedhammer` `#codec`

`md.templateEngraveShapeGuard` refuses `tr(sortedmulti_a)` and `sortedmulti`
nested under a combinator, on the stated grounds that the shipped off-device
toolkit cannot reconstruct them — *"they would be silently engraved as an
UNRECOVERABLE backup"*. Both halves were true when written. Neither is now, and
the guard's own comment still says the fork does not port rust-miniscript, which
S0's pin lift changed.

Measured, not inferred:

| shape | guard's premise | today |
| --- | --- | --- |
| `tr(sortedmulti_a)` | unrecoverable | **round-trips**: `md decode` returns the template verbatim, exit 0 |
| `sortedmulti` in a combinator | unrecoverable | **unencodable**: `md encode` refuses it by BIP-383/388, so it cannot reach a card at all |

So the guard now blocks exactly one thing: a legitimate `tr(sortedmulti_a)`
template engrave, which D4 explicitly wants to allow.

**Conservative, not dangerous** — it refuses something safe rather than admitting
something unsafe, which is why this is a follow-up and not a defect. The new
Wallet Policy program calls the guard *deliberately* despite knowing it is stale:
a new program quietly admitting more than the shipped path is the worse of the
two errors, and loosening an admission rule is risk-set work that should be
applied to **both** paths in one cycle with vectors.

This is [[comments-outlive-their-conditions]] with a worked example: the guard
enumerates its shapes, and enumerated safety arguments go stale silently. Grep
for the mechanism, not the claim.

---

**CLOSED 2026-08-21** (`seedhammer` `008b1a3`). `tr(sortedmulti_a)` is admitted;
`sortedmulti` under a combinator stays refused.

**Re-measured on the current binaries before a line changed** — an enumerated
safety argument goes stale silently, and this one had:

```
md encode  tr(@0,sortedmulti_a(2,@0,@1))  -> 1 chunk
md decode  -> exit 0, the template verbatim
md verify  -> re-encodes to its own template
md address -> bc1p588jmtx4ptv76t9sclt6gt33eyydvsrea4njyayerqj2frw5m5aq5gzycw
```

Fully recoverable, which is the only thing the guard ever asked.

**Convergence, not leading.** The primary admits the shape at encode, decode,
verify and address, and has no `template_admissible` refusing it either —
despite the guard's own comment citing one. The fork was the only thing saying
no.

**The other arm stays**, and is now doubly defensive: our own encoder rejects
`sortedmulti` under a combinator by BIP-383/388, so that arm guards against a
card from some other producer. A guard narrowed to nothing is a guard deleted,
which the measurement does not support.

The pre-existing table test asserted the OLD behaviour; its row moved from
`refused` to `admitted` **with the reason recorded** rather than being deleted,
and three new tests pin the boundary directly.

### F-216 — a keyless template gathered *with* its mk1 key cards still shows no addresses (owning phase: **the tr/wsh cycle, Stage 5**) `#seedhammer`

Plan D3 has two halves. The Wallet Policy program ships the second — *"skipping
the gather proceeds to consent without address proof"* — and not the first:
*"a keyless template md1 … gates addresses on gathering N mk1 key cards."*

The gap is visible rather than hidden: `walletPolicyMd1` **accepts** mk1 cards in
the set (they are legitimate cargo — `bundleEngrave` cuts every card), so an
operator can gather a template plus its key plates and still be told *"Keyless
template - no addresses"* with the keys sitting in the same bundle.

**Not implemented as a mechanical extension, on purpose.** Combining them needs a
rule for mapping each mk1 to an `@N` slot, and that rule is not obvious: a
template carries no xpubs, so the mapping has to come from fingerprint + origin
path, from gather order, or from the policy-id stub — and they disagree when a
template elides fingerprints or seats one key at several slots. **A wrong slot
mapping derives a wrong address and presents it as proof**, which is worse than
showing none. It needs a decision and vectors, not a guess.

Until then the screen is honest about which case it is in, and that distinction
is tested.

### RULED 2026-08-21, and A0 measured

`design/agent-reports/RULING_f216_slot_mapping.md`: **build it.** A gathered mk1
card is seated by **declaration match** — its origin, and its fingerprint when
the template declares one, equal the slot's — under **mandatory
`policy_id_stub` membership**. Gather order and operator assignment are both
rejected as inputs. The argument is that this is not a heuristic: given three
invariants the device already enforces (stub = membership in *this* policy; one
origin binds one key; slot origins pairwise distinct since F-217), the
assignment is either fully determined or the gather is refused — there is no
state in which the device guesses and shows the guess as proof.

**A0 — its one precondition — is SATISFIED.** The stub is computable at gather
time from the keyless md1 alone:

```
FormAwareStubChunks(keyless_tr_with_leaf) = c8fe87cd
```

**And measuring it surfaced the consequence that will bite the implementation.**
`FormAwareStub` is form-aware by design: it returns the **template** id for a
keyless card and the **policy** id for a keyed one. The same wallet therefore has
two different stubs —

```
keyless template stub : c8fe87cd
keyed   policy   stub : b6713001
```

— so **an mk1 card minted for the FULL policy will not match a keyless
template's stub**, and every card would be refused at membership. Only cards
re-stubbed on the template id (what `templateizeBundle` / `reStubMk1` already do
when the device builds a template bundle) can pass layer 1.

That is not an objection to the ruling; it is the admissibility boundary the
implementation must state on screen, because "all your key cards were refused"
with no explanation is the worst possible version of a correct refusal.

### The CORE landed 2026-08-21 (`seedhammer` `2f3d140`)

`seatKeyCards` implements the ruling: stub membership, then declaration match,
with typed refusals for every undecidable state. **A1, A2, A3 and A5 are met** —
order-invariance, the seated template derives the vector's Rust-computed
address, a mutation (seat every card at slot 0) proves that cross-check can
fail, and a refusal returns nothing to derive from.

**Four things measured that would otherwise have shipped as bugs:**

1. **The two sides spell paths differently.** `bip32.Path.String()` renders
   `m/48h/0h/0h/2h`; `mk.Card.Path` carries `m/48'/0'/0'/2'`. A string compare
   matches **nothing**, and the symptom is every card refused — which reads as a
   corrupt card rather than a formatting mismatch. Compared structurally now.
2. **`StripToTemplate` drops FINGERPRINTS along with the keys**, deliberately: a
   fingerprint identifies a master, which is what a template-only engrave omits.
   So a **stripped** template whose slots share an origin cannot be seated at
   all, and is correctly refused. Only a template encoded with `--fingerprint`
   and no `--key` can seat that shape. **This is the admissibility boundary the
   screen must state.**
3. **`sortedmulti` hides a misseating** — it sorts before building the script, so
   slot assignment does not move the address. A swap test on a sortedmulti
   fixture passed while proving nothing. Any cross-check of seating must use an
   order-sensitive policy.
4. **One card may fill several slots**, and the same card scanned twice is not a
   contest — refusing a re-scan would be a false alarm on an ordinary mistake.

### WIRED 2026-08-21 (`seedhammer` `a18d19e`, `4d2fc2f`) — D3's first half ships

A keyless template gathered **with** its mk1 key cards now shows addresses on the
consent screen, and D3's second half survives: skipping the gather still reaches
consent without address proof, pinned by its own test.

**Seating only when the template is keyless, all-or-nothing.** A full-policy card
already carries its keys and seating over the top would let a stray card silently
replace a declared one; a partially-keyed card is a shape this device already
refuses to derive from, and filling its gaps would produce a wallet half engraved
and half scanned.

**One sentence per refusal**, and the first carries the finding that would
otherwise confuse everyone:

> "A key card doesn't belong to this policy. Note that key cards made for the
> full-policy card carry a different stub than a template-only card expects."

**Proved from real mk1 STRINGS**, not hand-built `Card` structs — scan →
chunk-assemble → `mk.Decode` → seat → the address Rust derived. That path is
where this kind of feature usually fails with every component green.

**A6 done** (`seedhammer` unit tests): one card fills TWO slots when the policy
seats one master at two multipath branches — a genuine two-key script that
F-218's `(xpub, use-site)` check exists to let through, and the shape a naive
"one card, one slot" rule would break. Pinned by mutation.

**A4 DONE 2026-08-21** (`seedhammer` `a3f89c6`) — `capture_seating.py`, both
arms, on the emulator:

- **happy:** a keyless template + 2 key cards presented in **reverse order** →
  all four host-derived addresses, with absence assertions rejecting "no
  addresses" / "Keyless template" / "can't derive", because those are exactly
  what the pre-F-216 device showed and what a regression looks like;
- **refusal** (`--prove-refusal`): a card stubbed on the **policy** id instead of
  the template id → the device refuses **in words**, naming the stub difference,
  and shows **no address**. Both are asserted.

**The refusal card had to be MINTED, not corrupted** — mk1 is BCH-checksummed,
so a mangled card is dropped at the scanner and never reaches seating at all
(measured: the key-card tally stayed at 0). The realistic failure is a card made
for the full-policy form of the same wallet.

**F-216 is functionally complete**: A0–A6 all met. What remains is optional
polish — the other two refusal arms (no-slot, contested) have unit coverage and
their own sentences but no emulator walk.

---

### F-217 — ~~a card can declare ONE key origin for SEVERAL DIFFERENT keys~~ **CLOSED 2026-08-20** `#codec` `#funds-safety`

Found by a reader's two questions about the Wallet Policy journey — first why all
four cosigners share a master fingerprint, then the sharper one: **how can one
seed and one path yield two different keys?**

They cannot. BIP-32 is deterministic: a `(master fingerprint, derivation path)`
pair identifies exactly **one** xpub. A descriptor that repeats one origin across
different keys is not merely mislabelled, it is **self-contradictory**, and
saying so needs no seed, no network and no derivation — it is a pure function of
the card.

**The journey's card, rendered:**

```
[73c5da0a/48'/0'/0'/2']  xpub661MyMwAqRbcGQnC…LzpvZG2s
[73c5da0a/48'/0'/0'/2']  xpub661MyMwAqRbcG516…HaHy6tFz
[73c5da0a/48'/0'/0'/2']  xpub661MyMwAqRbcGS5Q…Nm3cGkqg
[73c5da0a/48'/0'/0'/2']  xpub661MyMwAqRbcGeL2…Z7JECRrT
```

distinct origins: **1**; distinct xpubs: **4**.

**And it is not one card.** Scanning every vendored keyed vector's rendered
descriptor for an origin bound to more than one xpub:

| | |
| --- | --- |
| multi-key vectors that are **contradictory** | **9** |
| multi-key vectors that are **consistent** | **0** |

`keyed_tr_depth2`, `…_rightspine`, `keyed_tr_multi_a`, `keyed_tr_sortedmulti_a`,
`keyed_tr_with_leaf`, `keyed_wsh_or_b`, `keyed_wsh_or_d_degrading`,
`keyed_wsh_thresh`, `keyed_wsh_timelock_hashlock`. **The corpus that gates the Go
port against Rust pins an impossible wallet shape**, in every entry where the
question can even be asked.

**Cause.** The keys are genuinely distinct — accounts 0..3 of BIP-39's test seed,
which live at `48'/0'/N'/2'` — but `md encode --path` **flattens Divergent mode
to Shared**, overwriting all four with account 0's path. The CLI cannot express
the truth: `--key` rejects an inline origin
(`--key "@0=[73c5da0a/48'/0'/0'/2']xpub…"` → *"base58check decode: decode"*), and
`--path` is shared-only, while the codec's per-key `OriginPathOverrides` sits
unreachable behind both. For a `tr()` template, whose wrapper has no canonical
default, the only choices are **no origin** (partial decode) or **one shared
origin that is false for most keys**.

**Why nothing caught it.** Addresses come from the xpubs the card carries, not
from the declared origin — so every address check, including the journey's
device-vs-host comparison and the whole cross-language corpus, passes either way.
The origin is what a *signer* uses to find its key, so the failure surfaces at
signing time and nowhere earlier. This is the [[cross-language-vectors-see-what-no-repo-test-can]]
shape one level up: both languages agree, and both are wrong together.

**Three separable pieces of work:**
1. **Refuse it, Rust-first.** Same `(fingerprint, path)` bound to two different
   xpubs is a contradiction a validator can prove. `md encode` should refuse;
   `md inspect` should report it; the device should refuse such a card on the
   supplied-policy paths.
2. **A per-key origin surface**, so the correct card is expressible at all.
3. **Regenerate the corpus** with genuinely divergent origins once (2) exists —
   and add a gate asserting no vector declares one origin for two keys, so this
   cannot come back.

Ordered that way deliberately: the refusal is what stops new bad cards, and it is
worth having before the corpus is rewritten, because the rewrite is what proves
the refusal works.

---

**CLOSED 2026-08-20.** All three pieces, in the order fable ruled
(`design/agent-reports/RULING_f217_vs_stage6_ordering.md`):

| piece | where | outcome |
| --- | --- | --- |
| (1) refuse it | `descriptor-mnemonic` `fe4b1ec9`, `seedhammer` `ca3e7d9` | `validate_origin_key_consistency`, on the ENCODE path in both languages |
| (2) per-key origin surface | — | **already existed**; see the correction above |
| (3) regenerate + gate | `fe4b1ec9`, `seedhammer` `0e180f6` | 9 contradictory → **0**; 11 consistent; a corpus gate reads what is committed |

**Refused on encode, not decode**, so new impossible cards stop while
already-written ones stay readable — a card that cannot be read is a backup that
cannot be restored.

**It caught a second one on its first Go run**: the fork's own
`TestAssembleBuildPolicy_IncludeFpDiffers` synthesized the SAME fingerprint for
two DIFFERENT foreign keys at one shared origin. Nothing that test asserts
needed them to match.

**And the journey it was found in was itself carrying the defect.** Repointed at
per-key origins, re-run, re-captured: the wallet id changed to `4e67c6fd…`
(matching the regenerated corpus) and **all four addresses are byte-identical to
what the impossible card produced**. That is the finding, demonstrated: the
device-vs-host comparison passed just as happily against a wallet that cannot
exist.

---

### F-218 — ~~`md encode` accepts the SAME xpub in every slot~~ **MOSTLY CLOSED 2026-08-21** `#codec` `#funds-safety`

From the same question. `md encode` on a 4-key template with one xpub repeated
four times emits a card without complaint — a policy that reads as 4-of-4 and is
spendable by one key.

The fork **does** refuse this, with a named, well-written refusal naming the
slots (`gui/multisig_build.go:364`, `errBuildDuplicateKey`) — but that check sits
in `buildMultisigPolicyFlow`, the path where **the device assembles the policy
from cosigner cards**. A policy that arrives already built — Engrave Bundle, and
the new **Wallet Policy** program — never reaches it.

So the check exists and guards the one path where the operator could not have
made the mistake off-device, and is absent on the two paths where a host tool
just did. Same asymmetry class as F-213, in the other direction.

Two halves, and they are separable:
- **Rust-primary:** `md encode` should refuse (or at minimum loudly warn about)
  a duplicated key across slots, with a vector.
- **Device:** the supplied-policy paths should run the same check the build path
  does. `errBuildDuplicateKey`'s message is already the right one.

### Re-measured 2026-08-20, on the question "do we reject repeated keys as unsafe?"

| surface | refuses a repeated key? |
| --- | --- |
| `md encode` (host) | **No.** Emits the card. No error, and **nothing on stderr** either |
| device, **build** path (`buildMultisigPolicyFlow`) | **Yes** — `duplicateSlotPair` → `errBuildDuplicateKey`, naming both slots |
| device, **supplied-policy** paths (Engrave Bundle, Engrave Multisig's supply, **Wallet Policy**) | **No** |

The build-path check is well-placed and well-argued — it runs *before*
`buildReviewFlow` precisely because, with fingerprint-presence `Omit` (the
default), that screen renders every slot `(no fp)`, so **a wallet one master can
spend alone looks exactly like a wallet three masters share**. Its own comment
states the scope limit outright: *"mints passes here; not every md1 the device
engraves."*

**MOSTLY CLOSED 2026-08-21** (`descriptor-mnemonic` `38cc2fb5`, `seedhammer`
`9a7a3b9`). `md encode` refuses, and the Wallet Policy program refuses a supplied
card, both before anything is minted or consented to.

**The check is `(xpub, use_site_path)`, and the second half is what makes it
correct rather than merely strict.** The fork's existing `duplicateSlotPair`
compares chain-code‖pubkey alone, reasoning that "identical xpubs derive
identical child keys at every address index" — true only when the use-sites
match. Measured: `<0;1>` and `<2;3>` over one key give
`bc1qsa6qqvk…` and `bc1ql5j095g…`, two different wallets. The build flow builds a
uniform use-site so its check is correct there and is untouched; supplied
policies carry arbitrary use-sites, which is exactly where the imprecision would
start refusing legitimate wallets.

**It caught two more fixtures on its first run** — `h13_hardened_multipath_reject`
and `m5_multipath_not_last_reject` both bound `@0` and `@1` to one tpub, in tests
about something else entirely. That is the third fixture set this cycle's checks
have corrected.

**STILL OPEN:** **Engrave Bundle** and **Engrave Multisig's supply path** also
take an already-built card and still do not run this check. They are older shared
flows, and editing them is a separate and riskier change than adding a check to a
program written this cycle.

**Scope, so this is not confused with F-217:** the check refuses only
**IDENTICAL** keys. The same seed at a *different* origin is not a duplicate and
is admitted deliberately (`gui/multisig.go:306`). And F-217's origin-contradiction
check explicitly **exempts** the same key at one origin, so the two refusals stay
distinct — one origin bound to two different keys is impossible; one key in two
slots is merely unsafe.

---

## Ledger reconciliation, 2026-08-21

The mechanical pass from `design/agent-reports/BURNDOWN_followups_2026-08-20.md`
§2, run in full. **126 → 109 open**, and the count moved without a line of code
changing, which was the point: about a sixth of the "debt" was bookkeeping.

- **Closed 11** whose own titles already said withdrawn / fixed / subsumed /
  misfiled, plus the two record-only entries (F-72's historical note, F-83's
  accepted limitation). Each heading now says WHY it closed, not merely that it
  did.
- **Repaired both duplicate numbers.** F-109 and F-120 were each used for two
  entries; the second of each was an addendum on the same subject and is now
  nested under its original as `#### Addendum`. Numbers are never reused — they
  are cited in commits and reports, and a reused one makes those citations
  ambiguous forever.
- **Verified the two "B2b — CRITICAL, gates the phase" entries first**, before
  anything else acted on the plan. All three of that family (F-106, F-107,
  F-108) were fixed on 2026-08-10 with commits recorded in their bodies —
  F-106 confirmed on hardware at `v0.0.0-g747cf48`. Only the headings never said
  so. **Zero still-open Criticals.**

**Three entries were deliberately NOT closed**, because closing them needs a
judgement the mechanical pass is not allowed to make. They read as resolved and
are not marked so:

| | why it was left |
| --- | --- |
| **F-129** | titled ANSWERED; whether the answer *resolves* it or merely records it is unclear |
| **F-173** | titled RULED; the ruling constrains later work that may still be open |
| **F-188** | titled RULED (operator: "Build this") — a ruling to DO something, so the work may remain |

Two more are correctly labelled partial and stay open on purpose: **F-145**
(PARTIALLY DONE, "still uncovered, now blocked on F-146") and **F-151**
(`(1) DONE; (2)+(3) polish / v0.0.1`).

---

### F-219 — a card's per-key origins are shown by `inspect --json` but by no TEXT surface, and the decoded text re-encodes to a DIFFERENT card (owning phase: **the tr/wsh cycle, Stage 6**) `#codec`

Found building the taproot pathological journey, whose round-trip gate this
breaks.

**Measured, minimal case:**

```
source   wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))
decoded  wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))            <- origins gone
```

| check | result |
| --- | --- |
| `md verify --template <SOURCE>` | **OK** — the origins ARE in the card |
| `md verify --template <DECODED>` | **MISMATCH**: expected 54-bit payload, got 115-bit |
| re-encode the decoded text | **different cards** |
| `md inspect` | shows template, `n`, and three ids — **no origin anywhere** |

**CORRECTED 2026-08-21, before any fix was built — the original claim was too
strong.** `md inspect --json` **does** show them:

```json
"path_decl": {"tag": "Divergent", "data": ["m/48'/0'/0'/2'", "m/48'/0'/1'/2'"]}
```

So the origins are recoverable from a card, and the honest finding is narrower
and still worth fixing:

1. **`md inspect`'s TEXT output omits what its own `--json` carries.** Text is
   what an operator reads off a terminal; the two surfaces disagree about what
   the card contains.
2. **`md decode`'s output is lossy and not a fixpoint.** It renders the origins
   away, and re-encoding what it prints yields a *different card* — so a "did I
   transcribe this right?" check fails for a plate that is perfectly correct.

**This is the THIRD time in this cycle I asserted a capability was absent after
checking one surface** (the others: "concrete rendering does not exist in
md-codec", and "the CLI cannot express per-key origins"). All three were one
command from being disproved, and all three made the work look bigger than it
was. See [[departure-sections-need-a-run-check]].

**Why it still matters.** The origin is what a signer uses to find its key. An
operator restoring from an engraved plate reaches for `md decode` — the command
named for the job — gets a template that looks complete, and has silently lost
the field that says where the keys live. `--json` on a different subcommand is
not where they will look.

**It is the mirror of F-217**, and the pair is the whole story: F-217 was origins
declared *wrongly* and refused; this is origins carried *correctly* and never
shown.

**Consequence for the journey**, and the reason it surfaced: the plan's E1 asserts
`md decode` of the chunk set is byte-identical to the committed policy. That
cannot hold while decode is lossy. **`md verify` is the right instrument** — it
exists precisely to check that backup strings re-encode to a given template, it
returns exit 0/1, and it compares payloads rather than rendered text. The journey
uses it.

Two separable pieces:
1. ~~**Show the origins on read-back.**~~ **DONE 2026-08-21**
   (`descriptor-mnemonic` `5ca5ceec`) — `md inspect`'s text output now prints
   them per `@N`, in descriptor spelling when a fingerprint is present and as a
   bare path when it is not. Gated on **agreement with `--json`** rather than on
   the presence of a string, so the two surfaces cannot drift apart again, with
   `md verify` as arbiter that the reported origins are the ones the card
   re-encodes to (and a negative control, since a `verify` that accepts anything
   would make that worthless).
2. **PARTLY DONE 2026-08-21** (`descriptor-mnemonic` `4fe5c2db`) — `md decode`
   now NOTES the origins on **stderr**:

   ```
   note: key origins carried by this card (not shown in the template):
     @0: [73c5da0a/48'/0'/0'/2']
     @1: [73c5da0a/48'/0'/1'/2']
   ```

   stderr and not stdout, and that constraint chose the design: stdout is the
   template and is piped into `verify`, `encode` and diffs — the pathological
   journey's own round-trip gate does exactly that. A test asserts stdout is
   still one line and still a template. Another asserts the note **agrees with
   `md inspect`**, since two surfaces disagreeing about one card is the defect
   itself.

   **STILL OPEN, and it is the structural half:** decode's *stdout* does not
   round-trip. The rendered template omits the origins, so re-encoding what it
   prints yields a different card. Fixing that means threading the resolved
   origins through the whole normative renderer and rewriting every vector's
   `.template` — a design change with corpus-wide blast radius, and it should be
   scoped as one rather than bolted on. Once it lands, the decode → re-encode
   fixpoint becomes assertable; today it silently is not one.

---

### F-220 — a CANONICAL wrapper never demands an origin, so a keyless template can be engraved declaring `m` for every slot (owning phase: **the tr/wsh cycle, Stage 6**) `#codec` `#funds-safety`

Found adding the address step to the operator journey (5-of-12 `wsh(multi(...))`).

`md inspect` on that journey's engraved card:

```
origins:
  @0: m
  @1: m
  …
  @11: m
```

**Twelve cosigners, and the card says nothing about where any of their keys
live.** The origin is the field a signer uses to FIND its key, so a restorer
holding only these plates has twelve xpub slots and no way to derive a single
one of them.

**Why nothing objected.** `wsh(...)` is a *canonical* wrapper, so it has a
default derivation path and `md encode` never demands `--path` — that warning
fires only for non-canonical wrappers like `tr()` (F-129). A keyless template
records no origin of its own. So the card is well-formed, decodes cleanly,
re-encodes byte-identically, and is missing the one field that makes it
restorable.

**The contrast that makes it sharp:** the *taproot* pathological journey carries
all eleven origins, because `tr()` forced the issue and the origins went into the
template. The `wsh` journeys do not, because nothing forced it. **Whether a
backup records where its keys live currently depends on which script type the
wallet happens to use.**

Adjacent to F-219 but distinct: F-219 is origins *carried and not shown*; this is
origins *never captured at all*.

Two candidate fixes, and the choice is a ruling:
1. **Warn (or refuse) on a keyless multi-key template with empty origins** at
   encode. Cheap, and it puts the decision where the operator still has options.
2. **Emit the canonical default explicitly** rather than leaving it empty, so the
   card states the path it is relying on. Changes existing card bytes, so it
   needs the same corpus care F-217 did.

Recorded in the operator journey itself rather than only here — that document now
shows the `origins: m` block beside its addresses and says plainly that the
twelve paths came from the key files, not from the card.

---

### F-221 — F-217's contradiction check cannot see a KEYLESS template, which is where the pathological journey's card hides one (owning phase: **the tr/wsh cycle, Stage 6**) `#codec` `#funds-safety`

Found by a reader asking *"what is a round trip if not a restore test?"* about the
pathological journey's decode step.

F-217 refuses a card that binds one `(fingerprint, path)` to two different keys.
The check needs **both** a fingerprint and an xpub per slot — a keyless template
carries neither, so it passes silently. And a keyless template is exactly what
gets engraved on the descriptor plate.

**Measured on that journey's own card.** `md encode --path "48'/0'/0'/2'"` over a
policy whose eleven keys sit at four account indices across three masters. The
card declares one shared origin; the restore test
(`design/journeys/restore_test_pathological.py`) derives what a card-trusting
restorer would get:

| | |
| --- | --- |
| slots recovered correctly | **3** — @0, @4, @8, one account-0 key per master |
| slots recovering the WRONG key | **8** — @1, @2, @3, @5, @6, @7, @9, @10 |

Every tier of that vault needs signatures this restore cannot produce.

**The keyed form of the same wallet WOULD be refused** — that is the asymmetry.
Add the xpubs and fingerprints and F-217 fires; leave them off and the identical
mistake engraves cleanly. The check is on the form that is not engraved.

**Related but distinct from F-220.** F-220 is a canonical wrapper never demanding
an origin, so the card records `m`. This is a card that records an origin which
is *wrong for most of its slots*. Both end the same way — a restorer cannot find
the keys — and neither is visible to any address check, because addresses come
from the xpubs a card carries rather than the origins it declares.

Candidate fix: extend the contradiction check to keyless templates using
**declared origins alone** — if a template's slots all share one origin while its
`n` exceeds the number of keys one master could plausibly seat there, that is at
minimum a warning. The precise rule needs a ruling; the measurement does not.

---

### F-222 — ~~the example vault is single-master in THREE of its four tiers and neither journey says so~~ **CLOSED 2026-08-21** `#mnemonic` `#docs`

Found by the comprehension lens, then re-measured and found to be worse than
reported. The lens said master C alone drains the vault after ~365 days; the
slot-to-master mapping says **three of the four tiers need only one master**:

| tier | lock | threshold | slots | masters |
| --- | --- | --- | --- | --- |
| 1 | `after(1000000)` — a block HEIGHT, passed in 2016 | 3-of-3 | @0 @1 @2 | **A alone** (+ the sha256 preimage) |
| 2 | `after(1893456000)` | 2-of-3 | @3 @4 @5 | A and B |
| 3 | `older(65535)` ≈ 455 d | 2-of-2 | @6 @7 | **B alone** |
| 4 | `older(4255898)` ≈ 365 d | 1-of-3 | @8 @9 @10 | **C alone** |

So the wallet reads as an eleven-key, three-party vault and behaves as *any one
of three parties, on a timer* — A immediately if they hold the preimage, C after
a year, B after fifteen months. Tier 2 is the only one that needs two parties,
and it is the one furthest out.

**This changes what the example teaches.** It is presented as the constellation's
showcase of a degrading multi-tier vault; a reader takes the tier structure as a
model. The tiers do not degrade in strength, they degrade in *which single party
wins the race* — and F-133 already established the timers fire in the wrong order
relative to strength.

Not a code defect and not funds at risk (published test seeds). It is a
**teaching** defect in the example itself.

**CLOSED 2026-08-21** — BOTH journeys now carry a "What this backup will NOT
do" section stating it plainly: *"in master terms this is a 1-of-3 vault with
delays, not an 11-key multisig"*, with the per-tier breakdown, alongside F-132's
missing preimage and F-133's inverted timers.

Adjacent: F-131 (the checklist's false recovery rule), F-132 (the preimage is
absent from the backup), F-133 (the tiers are inverted). This is the fourth
member of that family and the one that most changes how the wallet reads.
### F-223 — ~~`mk encode` takes one key per invocation~~ **CLOSED 2026-08-21** `#mnemonic`

Filed 2026-08-21, closing F-127.

F-127 was expected to collapse the pathological wallet's key-card build from
~33 hand-built commands to ~3. It does not, and the measurement is worth
keeping so the estimate is not repeated: `mk encode --xpub <XPUB>` accepts
**one** xpub, so eleven cosigners are eleven invocations both before and after
the fix. What F-127 removed is the hand-copied stub hex and the `md inspect`
step that produced it -- real, but a constant, not a factor of N.

The remaining labour is genuinely per-key and genuinely mechanical: for each
cosigner, read an xpub, read its origin fingerprint and path, and invoke
`mk encode` with the same `--from-md1` chunk set every time. Every operator
journey scripts this as a shell loop, which is the tell that the tool is
missing the mode.

Two candidate shapes, unruled:

- make `--xpub` / `--origin-fingerprint` / `--origin-path` repeatable and
  positionally correlated (fragile -- three parallel lists that can desync,
  and a desync mints a card that names the wrong master);
- accept a key FILE, one `[fingerprint/path]xpub` descriptor-style record per
  line, which is the form the journey's `inputs-pathological/keys/*.xpub`
  files already carry and which cannot desync.

The second is the better shape for the same reason `card-index.txt` keys on
the mk1 string rather than on key order (see the note in
`transcript_pathological.sh` section 7b): an ordering assumption between two
lists is exactly what produced 30 wrongly-captioned plates once already.

**Not blocking anything.** The loop works and is committed. This is
ergonomics, and it is the ergonomics an operator meets first.

#### RESOLVED 2026-08-21 — `mnemonic-key` (`--keys`) + this repo `01697a1`

`mk encode --keys <FILE>` mints one card per record. Records are BIP-380 origin
notation, one per line (`[fingerprint/path]xpub`), `-` reads stdin, blank lines
and `#` comments ignored. The key-file shape recommended in this entry was the
one built, for the reason given: parallel repeatable flags can desync, and a
desync mints a card naming the wrong master.

**The number in this entry was wrong, and the correction is worth keeping.** It
recorded that F-127's "~33 commands to ~3" estimate "does not survive
measurement", on the grounds that eleven cosigners are eleven invocations
either way. That was true of the *cards* and missed the larger half: the
card-index step called `mk encode` TWICE PER KEY -- once to count chunks, once
to read strings. The original estimate was right in shape; the second
measurement counted only section 7.

**Both of this entry's numbers were still wrong, and the third measurement is
the one to trust.** Counted with an instrumented `mk` shim that logs every
actual process invocation, rather than by grepping the transcript LOG for
`mk encode` -- the log only shows ECHOED commands, and the `$(...)` captures are
silent, which is what produced both earlier errors:

| tree | `mk encode` invocations |
| --- | --- |
| `6e6753c` (before this cycle) | **34** |
| `85cf6c7` (parent of the commit that claimed "33") | **35** |
| current | **3** |

"33" omitted section 5's demonstration call; "2" omitted the section 6 stub
read-back added in the same commit. Two independent reviewers (R3, R4) caught
this and reported 34 and 35 respectively -- both correct, against different
baselines. **So: 34 -> 3 across the cycle**, with every artifact byte-identical
to the loop it replaces.

The lesson is the same one three times: measuring a PROXY (a log, a static
grep) instead of the thing. Count invocations by instrumenting the binary.

**The part that matters more than the count.** The old card-index loop
RE-DERIVED the cards it was indexing, so nothing guaranteed the strings it
captioned were the strings engraved. Both now come from one batch output.

**Equivalence is the pinned property, not the convenience.** `--keys` is an
input multiplexer; if it ever mints a different card than the loop it replaces
it is wrong however convenient it is. `batch_matches_per_key_loop` asserts
byte-identity against N single-key invocations, and both routes share one mint
path so they cannot drift.

**Deliberate refusals** (each tested): `--keys` is mutually exclusive with
`--xpub`, `--origin-path`, `--origin-fingerprint`, `--chunk-set-id` and
`--privacy-preserving`. A record carries its own origin, so a global one would
override it or be ignored. Privacy-preserving cards are minted one at a time on
purpose -- a record always declares a fingerprint, and dropping it silently is
how a card gets engraved wrong. An empty key file is refused rather than
minting nothing at exit 0.

**Cross-repo catch worth remembering.** Making `--xpub` optional flipped
`mk gui-schema` to `required: false`, caught by an existing test. `mnemonic-gui`
keeps a HAND-WRITTEN mirror (`src/schema/mk.rs`) with `required: true` and there
is no automated gate between the repos, so the flip would have desynced them
silently. Resolved by holding the GUI contract fixed rather than editing the
test: `--keys` is excluded from the schema, which describes the form the GUI
RENDERS (one card), and the emitted schema was verified BYTE-IDENTICAL against
a build of the previous tree.

### F-224 — residual Minors/Nits from the six-lens review — ~~24~~ **PARTIALLY BURNED DOWN 2026-08-21** `#mnemonic`

Filed 2026-08-21, closing the review round.

**All 3 Criticals and all 8 Importants are folded** (`mnemonic-key` 47d7f97 +
this repo e3da435). What remains is **16 Minor / 8 Nit**, none gating.

**They are NOT transcribed here on purpose.** The six reports are committed
verbatim in `design/agent-reports/R1..R6` and are the record; copying 24 items
into this file is exactly the transcription step that has introduced misquotes
in this repo before. Read them with `git show 89470c9`.

#### The valuable third is DONE (`mnemonic-key` 9b30566)

Triaged by asking what each item actually costs an operator, rather than
burning down by count. **8 done, ~5 already moot, ~4 declined, the rest are
message polish left open.**

**Two were worth more than their filed severity:**

- `mk verify --from-md1` reported a **CORRECT card as FAILING** — the stub
  comparison was order-sensitive, so the same card checked against the same
  policies in a different `--from-md1` order returned exit 4. Filed Minor;
  in effect Important, because a verification tool that calls a good card bad
  invites re-engraving a good plate. Now a multiset compare.
- The phantom `SPEC §3.5.4` cite was **one of nine** (§3.5.2, .3, .4, .5,
  .6 ×2, .7 ×2, and §1.1 — `error.rs:3` carried two on one line). Written as
  "eight" on first filing while the enumeration beside it listed nine;
  machine-counted and corrected 2026-08-21. Fourth wrong count in this cycle,
  and the third produced while correcting the second — every one of them from
  counting a rendering instead of the thing. §3.5 is "Origin path encoding" and has no
  subsections at all. A 2026-06-10 audit had already found four of this class
  and repointed them, leaving these — so fixing only the one a reviewer noticed
  would have repeated that exactly. `tests/spec_cites_resolve.rs` now makes the
  check a command instead of a discipline.

Also done: a batch mint failure names the record; the origin fingerprint is
checked at depth 0/1 where the xpub proves it; BOM and CR-only key files are
handled or refused by name (CR-only was a SILENT SHORT BUNDLE); `--from-md1`
accepts display-grouped md1; the SLIP-0132 note no longer names `--xpub`.

**Declined, with reasons**, so they are not relitigated: stub order being
argument order is real but changing it is a behaviour change to a documented
property — better documented than "fixed"; batch `--json` entries omitting
`schema_version` is consistent with the envelope-level house style, not a
contradiction; duplicate-policy form-dependence and the unanchored
`sed 's/key-0*//'` are theoretical against the fixtures in play.

**Moot**: all three R3 awk findings, its double-FATAL guard, and the wrong
invocation counts died with the rewrite to `build_card_index.py`.

**Still open**: the message-quality residue — a wrong bound in the `>255 stubs`
error, a `u8`-truncated depth in `XpubOriginPathMismatch`, a trailing-slash
origin that is refused for the right outcome by the wrong check, and the
non-JSON `--keys` output carrying no per-card identity (the JSON form now
does, which is what the journey consumes).

Triage, by report:

- **R1** (6 Min / 2 Nit) — mostly ergonomics on the new batch path: a batch
  encode error names no record; `--from-md1` refuses comma-grouped md1 that
  `mk`'s own display format produces; `verify --from-md1` is order-sensitive;
  batch JSON entries carry no `schema_version`. The "crossed record is
  accepted" Minor is **partly addressed** — the over-claiming doc comment it
  cited is corrected, and cards now report their origin in `--json`, which is
  what detects a crossed record.
- **R2** (6 Min / 2 Nit) — input-handling edges that all REFUSE correctly but
  with poor messages (a trailing `#`, a Windows CRLF file, duplicate
  `--from-md1`, SLIP-0132 handling), plus stub-order being argument-order
  dependent.
- **R3** (3 Min / 3 Nit) — journey-script hygiene. Two are now moot: the awk
  paragraph-mode parse and its escape-sequence trap are gone with the rewrite
  to `build_card_index.py`.
- **R6** (1 Min / 1 Nit) — a stale unmarked Q-2 answer in the 2026-04-29
  closure-design doc, and a phantom `SPEC §3.5.4` doc-cite in `verify.rs`
  (a fifth instance of the "phantom §3.5.x cite" class the 2026-06-10 audit
  fixed four of).

**One Important was CLOSED BY DESIGN rather than by refusal** and is recorded
here so the reasoning is not relitigated: R2 flagged that a `--keys` batch can
mint N cards for N+1 cosigners. Refusing an incomplete set would break the
ordinary workflow — a cosigner cards their own key without the others' xpubs in
hand — so `mk encode` now emits a **note** naming how many cosigners are not
carded, and still exits 0. Membership itself IS enforced, so every card in such
a batch is a genuine member; only the count is short.

### F-225 — **HIGH PRIORITY FEATURE**: one chunk per plate wastes ~60% of every plate, and nothing decided it (owning phase: **engraving throughput**) `#mnemonic` `#hardware`

Filed 2026-08-21 from an operator question — "if we chunk at 80 for the
mnemonic, does that force us to engrave each chunk separately, or can multiple
chunks be on one plate?" — which nothing in this repo answers.

**Measured today on the pathological wallet.** `me bundle` emits exactly one
plate per string: 34 public strings → 34 plates (+ 1 `ms1` secret plate, listed
and deliberately never rendered). Against `PLATE_TEXT_BUDGET = 300`
(`crates/me-cli/src/lib.rs:48`, "SeedHammer's 85×85mm text layout wraps ~35
chars/line over ~20 usable lines"):

| chunks | length | share of one plate |
| --- | --- | --- |
| 19 | 111 chars | 37% |
| 3 | 88 | 29% |
| 1 | 86 | 29% |
| 3 | 80 | 27% |
| 8 | 29 | 10% |

The longest chunk uses **37%** of a plate; the two longest together are 74% and
would fit. So the wallet engraves **34 plates to carry content that fits on
roughly 13**. At the measured ~21 minutes per plate that is about **12 hours
versus 4½** — the single largest throughput lever in the engraving path.

**Two limits are being conflated, and that is the root of it.** The 80-symbol
cap is a CODE constraint: the codex32 regular code is BCH(93, 80, 8) and cannot
protect a longer string. The 300-char budget is a PHYSICAL constraint: how much
text fits on an 85×85mm plate. Nothing requires them to line up, and they do
not — which is exactly where the slack is.

**No decision record exists.** Grepped `design/` for a ruling on plates-per-
chunk and found none; the one-per-plate shape appears to be inherited from the
single-string era rather than chosen. That is the first thing to establish.

**The real counter-argument, which must be answered rather than assumed away.**
Each plate is independently damageable, and a chunk set is ALL-OR-NOTHING —
`reassemble` refuses an incomplete set, so losing one plate loses the whole
descriptor either way. That cuts both directions and needs measuring, not
arguing:
  - AGAINST packing: two chunks on one corroded plate lose two chunks, so
    per-plate redundancy schemes (engrave a spare) get more expensive.
  - FOR packing: fewer plates is less metal, less time, less handling, and
    fewer opportunities for a mis-ordered or mislaid plate — and the
    all-or-nothing property means the set already has no partial-loss
    tolerance to protect.

**Open questions, in the order they should be settled:**
1. What is the plate's REAL capacity? The 300 is self-described as conservative
   and says "with a QR present, far less" — does the bundle render a QR, and
   what is the measured character ceiling with and without one?
2. Does the SeedHammer II firmware accept a multi-string plate at all, or does
   its plate model assume one payload? (`ErrTooLarge` is the current backstop.)
3. Is there a legibility floor — does packing shrink the glyphs, and does that
   collide with the 2-stroke-width minimum-feature rule already established for
   engraving fonts?
4. Chunk-set integrity: should chunks of ONE set be allowed to share a plate,
   or only chunks of DIFFERENT sets, so a lost plate never takes two chunks of
   the same descriptor?

**Not a codec change.** Nothing here touches md1/mk1 or the 80-symbol cap; it is
purely how `me bundle` lays strings onto plates. That keeps the blast radius
small and makes it independently gateable.

Related: F-136 (auto-chunking, closed 2026-08-21) is the *encoder* half of the
same operator confusion; this is the *engraving* half and the expensive one.

### F-226 — ~~`descriptor-mnemonic`'s vendor-freshness gate cannot pass~~ **CLOSED 2026-08-21** `#mnemonic` `#ci`

> **Title corrected.** It originally ended "…and is path-filtered so it never says so". The gate DID say so — it ran on the pin commit and failed, and the failure was ignored for two days. See the correction in the body.

Filed 2026-08-21, found incidentally while gating the F-136 fix.

#### RESOLVED 2026-08-21 — `descriptor-mnemonic` `d22aedd` (gate) + the re-vendor that followed

**It was masking a live defect, which is the part that matters.** Making the
gate runnable turned it red immediately: `cargo vendor` was last run 2026-06-24
(`e8474f48`), miniscript was pinned to the git fork 2026-08-20 (`5b4d20ad`),
and it was never re-run — so the committed tree held the **crates.io**
miniscript 13.0.0 while `Cargo.lock` demanded the fork. Proven rather than
inferred:

```
vendor/miniscript/.cargo-checksum.json   "package": "867b1f11e0545ad5…"
    -> a crates.io sha; a git-vendored crate has "package": null
git log -1 -- vendor/miniscript          e8474f48  (2026-06-24)
git log -1 -- Cargo.lock                 5b4d20ad  (2026-08-20)   ANCESTOR check: vendor predates the pin
diff vendor/miniscript/src/lib.rs <fork checkout>/src/lib.rs  -> DIFFERS
```

So the `--offline --locked` reproducible build — the entire reason a vendor
tree is committed — would have resolved the WRONG miniscript. That is the
toolkit v0.74.0 release-CI failure class, live here since 2026-08-20 and
invisible because the gate could not run.

**The fix, in two commits.** The script moved to the three-block source config
its own error message prescribed (crates-io + the miniscript git fork +
vendored-sources), with the rev derived from `Cargo.lock` so it auto-tracks the
pin and fails closed on an empty match. Then `cargo vendor vendor/`, whose
churn is confined to one crate.

**The original guard was kept, generalized.** The three-block form covers
exactly one fork, so a SECOND git dependency would still be unredirected — the
same false GREEN one dep further along. Any git source not matching the
configured rev now trips a loud error naming it. Both arms verified: an
injected `git+https://example.com/fake` fails closed and prints the source;
removing it returns green with `Cargo.lock` byte-exact.

**Verified past what the gate checks.** The gate resolves metadata only; a
release build was additionally compiled through the vendored tree under the
reproducible build's own flags and reported `Compiling miniscript v13.0.0
(https://github.com/rust-bitcoin/rust-miniscript?rev=ff4732e…)` — the fork's
rev, from `vendor/`, offline. That is the claim the gate exists to make.

**Two lessons, both already in the constellation's notes and both re-earned
here:** a gate that cannot pass and does not fire is indistinguishable from one
that passes; and the cheapest way to find out what a silent gate was hiding is
to make it run.



`ci/repro/vendor-freshness.sh` in `descriptor-mnemonic` fails immediately:

```
::error::vendor-freshness: Cargo.lock now has a git source — the codec
two-block config can't redirect it. Add a per-source git-fork [source] stanza
(see the toolkit ci/repro/vendor-freshness.sh three-block form).
```

**Pre-existing, and proven so** — the same script fails identically on `HEAD~1`,
and my commit's `Cargo.lock` diff is empty. The cause is the miniscript git pin
landed by the tr/wsh cycle (`5b4d20ad`, "pin miniscript at ff4732e"). The
script's two-block source-replacement config cannot redirect a `git+` source,
and it **fails closed on purpose** — the guard is correct and is telling the
truth.

**Why it is quiet.** ~~The workflow is path-filtered ... so it has not run.
`gh run list --branch main` shows no vendor-freshness runs at all.~~

**THAT WAS WRONG, and the correction changes what this entry is about.**
Queried properly with `gh run list --workflow vendor-freshness.yml`, the gate
**did** run — on the pin commit itself:

```
2026-08-22  fc7548ce  success     <- the fix
2026-08-20  5b4d20ad  failure     <- the pin; it fired and reported
2026-07-11  5a0a4f41  success
```

It fired, it failed, and the failure **sat unacted-on for two days**. The
original claim came from `gh run list --branch main --limit 6`, which truncates
across ALL workflows — the vendor run was older than those six, and absence
from a truncated list was read as never-having-run. Fifth wrong claim of this
cycle, same root cause every time: measuring a rendering instead of querying
the thing. The right query names the workflow.

So the lesson is NOT "a gate that cannot fire is invisible". It is worse and
more ordinary: **a red gate nobody is watching costs exactly as much as one
that cannot run.** The signal existed, was correct, and was ignored.

**Not currently blocking.** It is not one of `main`'s required contexts
(`cargo test (ubuntu-latest)`, `cargo clippy`), so nothing is bypassed today.
But the next change to `Cargo.lock` or `vendor/` turns it red for a reason
unrelated to that change — and the *release* build it stands in for is the
`--offline --locked` reproducible one, which is exactly the path a git pin
breaks.

**Fix:** port the toolkit's THREE-block form (crates-io + vendored-sources +
a per-source git-fork stanza with the miniscript rev), which the script's own
error message already prescribes. `mnemonic-key`'s copy is the two-block CODEC
form and is correct there — that crate is fork-free — so this is not a
copy-paste sync, it is the fork-carrying variant.

Related: this is the same class as the mk-side catch earlier in this cycle,
where `vendor/` had silently fallen out of sync with `Cargo.lock` and only a
PR-time gate caught it. There the gate worked; here the gate cannot run.

### F-227 — a keyless template with colliding origins and no fingerprints cannot be seated `#journeys` `#funds` `#md` `#firmware`

Filed 2026-08-21, found by building the hashlock-vault journey — not by review.
**Owning phase:** the hashvault journey part is DONE; the `md encode` warning
and the two pathological journeys are open.

**The class.** A keyless md1 template names its slots by origin. `seatKeyCards`
(`gui/key_card_seating.go`) matches a card to a slot on that origin, checking
the fingerprint **only when the template declares one**, and refuses every
undecidable state. So when two slots share a derivation path and the template
declares no fingerprints, the card set is **unseatable** — the engraved backup
does not determine the wallet.

This is not exotic. `48'/0'/0'/2'` is *the* standard multisig account path;
every cosigner using it is the normal case. Measured across the journeys'
own artifacts:

| journey | slots | distinct origins | fingerprints |
|---|---|---|---|
| hashlock vault (bare) | 6 | 1 | 0 |
| tr-pathological | 11 | 4 | 0 |
| pathological | 11 | 4 | 0 |

**Why it matters.** `multi_a` commits to key order, so a wrong assignment is a
different wallet, not a cosmetic difference. Measured on the hashvault: three
assignments, three different addresses; 6! = 720 available.

#### RESOLVED for the hashvault journey — `md encode --fingerprint`, one extra chunk

`--fingerprint @i=HEX` is repeatable and was simply never passed. The six
masters already have distinct fingerprints. Declaring them moves no path,
reveals no key, leaves the policy character-identical, and the transcript gates
on `wallet-descriptor-template-id` being unchanged (`68a1a888…` both ways).
Cost: **4 chunks → 5**.

Proved on the device, three arms (`capture_hashvault.py`), not argued:

- `--keyed` — the 15-chunk keyed card derives; rules out the vault's shape
  (4 tiers, 2 hashlocks, both timelock flavours, `multi_a`, NUMS) as the cause.
- default — bare template + 6 cards: **refused**, `errSeatSlotContested`,
  *"Two different key cards claim the same slot, and this template can't tell
  them apart. It declares no fingerprints."*, no address shown.
- `--fingerprinted` — the SAME six cards on the 5-chunk template: **seated**,
  and derived all four host addresses.

`--prove-it-can-fail` demands the refusal from a card that derives, and exits 0
only when the walk fails — so the refusal assertion is not vacuous.

#### STILL OPEN

1. ~~**`md encode` says nothing.**~~ **DONE 2026-08-21** — `descriptor-mnemonic`
   `65cd940a`. `md encode` now warns, naming the colliding slots, their path and
   the remedy. Warn-only (a bare template is legal; an operator may record slot
   order out of band), stderr, exit 0, `--json` branch too.

   The subtlety, recorded because the first implementation got it wrong:
   **ambiguity is not equality of declarations.** Slots collide iff one card can
   match both — same path, and *not* both declaring a distinct fingerprint — so
   an undeclared slot is ambiguous with every slot at its path, and declaring
   fingerprints on only *some* of a group does not help. Grouping by
   `(fingerprint, path)` reports nothing; the test written for that case is what
   caught it. 4 mutations, all killed. 805/805, clippy 0.

   No Go port due: CLI authoring UX, not normative codec behaviour, and the
   device does not author templates from a policy string. **But the device's own
   multisig BUILD path (`gui/multisig_build_*`) authors a policy and was not
   checked** — it has the cosigner xpubs so it probably declares fingerprints,
   but "probably" is not measured. Open, below.
2. ~~**Both pathological journeys carry the same latent gap.**~~ **DONE
   2026-08-21** — `93e2f6d`. Not latent, as it turned out: their engraved set is
   the keyless template plus 30 mk1 cards, so both were shipping a backup that
   could not be restored. Declaring the fingerprints fixed both outright (all 11
   `(fingerprint, path)` pairs are unique), 11 slots going from 4 distinct
   declarations to 11. Cost two chunks each, 4 → 6; template-id unchanged, so
   every address, wallet id and mk1 stub is the same and the tr device walk —
   which gathers the KEYED card — is untouched.

   The check that it actually worked is that `md encode`'s new advisory is now
   **silent** on both, rather than that the plates got bigger. Their seating
   path still has no device walk; that is what item 3 would cover.
3. ~~**No plates-to-restore walk anywhere.**~~ **PARTLY DONE 2026-08-21** —
   `restore_from_plates.py`, wired into all three journeys as a gate. It decodes
   the QR out of every rendered plate image with `zbarimg` — an **independent**
   decoder, not the library that drew them — and rebuilds the wallet from the
   decoded strings alone.

   Two layers, and the first is nearly worthless on its own: QR-matches-manifest
   is `me bundle` agreeing with itself. The load-bearing one is the rebuilt
   `wallet-descriptor-template-id` matching what the run derived by a separate
   route, plus a seatability check on the plate-borne slots.

   Results: both pathological journeys restore clean (36 of 37 images, 11/11
   distinct declarations, SEATABLE). **The hashvault plates do NOT** — 22 of 23
   decode byte-exact and the template-id is right, but 6 slots share 1
   declaration, so the driver exits 1. That journey's gate is therefore
   *inverted*: it requires the failure, and goes red if those plates ever
   restore cleanly without the narrative being rewritten.

   Two controls, exercising different layers, because the obvious one does not
   test what it looks like it tests: `--prove-it-can-fail` flips a character and
   is refused by the **codex32 checksum at decode**, never reaching the
   comparison; `--prove-layer2-can-fail <journey>` substitutes another wallet's
   perfectly valid template, which is the only control that exercises the id
   check.

   **STILL OPEN, and it is the harder half: this is not metal.** The path from a
   rendered PNG to scratched steel to a phone camera — legibility, depth, glare,
   engraving artefacts — is exercised nowhere. What is now closed is the
   software chain end to end.
4. **The device's multisig build path is unchecked for this.**
   `gui/multisig_build_*` authors a policy on-device and engraves it. It holds
   the cosigner xpubs, so it very likely declares fingerprints and is fine —
   but that is a guess, and the whole point of this follow-up is that an
   unseatable template looks correct until someone tries to restore it. Measure
   it the way the host side was measured: author one, inspect the slots.

### F-228 — you cannot get from the English spec to the policy with the shipped tools `#md` `#usability` `#experimental`

Filed 2026-08-22. The operator asked the obvious question a journey exists to
provoke: *how does a user go from the four-tier English description to
`policy-tr.txt`?* Measured answer: **they cannot.** I hand-wrote that string in
the hashlock-vault cycle, and everything since is copy plus `sed` — which is why
the question had no good answer.

`md encode --from-policy <EXPR> --context tap|segwitv0` is exactly the intended
route. Two things stop it.

#### G1 — `--from-policy` is not in the default build

```
$ md encode --from-policy "…" --context tap
md: --from-policy requires the cli-compiler feature
```

`crates/md-cli/Cargo.toml:25` — `cli-compiler = ["miniscript/compiler"]`, not in
`default`. A user following this journey hits a wall and has to rebuild with
`--features cli-compiler`.

**Decision needed, not just a fix:** enable it by default (cost: miniscript's
compiler feature in every build — larger binary, longer compile), or ship it as
a documented opt-in with the journey saying so. Either is defensible; silence is
not.

#### G2 — `--experimental` does not reach the compiler, so this wallet cannot be compiled at all

With the feature built, isolated on real invocations:

| policy | result |
| --- | --- |
| `or(pk(@0),and(older(144),pk(@1)))` | **compiles** — positive control |
| last branch WITH a key | **compiles** (`chunk-set-id 0x888c8`) |
| last branch KEYLESS, **plus `--experimental`** | **refuses** — *"compile: Top Level script is not safe on some spendpath"* |

`--experimental` relaxes the **parse** path (`sanity_check`'s `requires_sig` via
`ExtParams::top_unsafe`). The **compiler** applies its own safety rule and
nothing waives it. So the whole `English → Policy → template` route is closed
for any wallet with a keyless spend path — which is this wallet's defining
feature.

**Consequence:** the documented authoring workflow works for wallets that do not
need `--experimental`, and silently stops working exactly where the flag was
invented to help. A user is left hand-writing miniscript, which is what the
compiler exists to prevent.

**Rust-primary**, `descriptor-mnemonic`. Options, unruled: extend
`--experimental` to the compiler path (`compile_tr`/`compile` with relaxed
`ExtParams`, if the fork's compiler supports it); or refuse with a message that
names the limitation instead of a generic compile error; or document the route
as unavailable for keyless policies and say what to do instead. **The current
behaviour — a generic error that does not mention the flag the user already
passed — is the one option that should not survive.**

**Owning phase:** any phase that claims a user can author this wallet. The
journey currently does not claim it, because it prints a hand-written policy
without saying it was hand-written; the transcript's new §1b now says so.

### F-229 — ~~decide whether tier 4 gets a key~~ **RESOLVED 2026-08-22: IT GETS ONE** `#wallet-design` `#interop`

**Operator ruling: *"keyless path is not reasonable."*** Tier 4 of the RCW is now
`after(1383520) AND sha256(H3) AND pk(@6)` — a seventh seed. Applied to all three
fixture policies, to `design/journeys/derive-rcw-keys.sh` (the generator, not the
artifacts), and to the regenerated `inputs-rcw/`. The earlier keep-keyless ruling
(`design/agent-reports/RULING_export_deadlock.md`, delegated to a stand-in) is
**superseded**.

**One thing the trade table below did not know**, found while applying this:
stock `rust-miniscript 13.1` refused the keyless **tr** form outright — *"All
spend paths must require a signature"* — while **accepting** the keyless **wsh**
form, because `Descriptor::from_str` only sanity-checks `Tr`. The two wrappings
of one wallet disagreed about their own validity, and only one said so. Both are
accepted now, and `md encode` no longer needs `--experimental`. See F-233.

The original entry follows, kept because its measured trade is the reasoning.

**The trade, measured:**

| | keyless (today) | keyed tier 4 |
| --- | --- | --- |
| passphrase-only escape hatch | **yes** — the tier's whole point | gone: becomes "phrase AND key @6" |
| third-party descriptor import | **impossible**, every app, non-waivable in Core | Core v29+ accepts the wsh form |
| addresses | current | **all change** — funds migration |
| watch in Core | via `addr()` list (Phase 1b, works) | via descriptor |

Isolated on real Core binaries: `tier 4 as-is` refuses, `tier 4 with a key`
accepts, and nothing else about the wallet bites.

**Re-open only if** interop with third-party software becomes worth more than a
key-free recovery path. The stand-in's reasoning for keeping it: the keyless
tier looks deliberate, and interop was plausibly an accepted cost.

**Not to be actioned by an agent.** Changing a funds-bearing vault's spending
conditions is the operator's signature.

### F-230 — hot-wallet export: NOT NOW, with a two-part trigger `#export` `#secrets` `#LOW`

**Priority: LOW.** Operator-filed 2026-08-22. Ruled NOT NOW; **"never" was
explicitly rejected**, so this is deferred rather than closed.

`mnemonic export-wallet` is watch-only by definition (`validate_watch_only`
rejects phrase/entropy/xprv/wif at slot resolution). Hot export exists nowhere in
the constellation.

**Why not now, in ascending order of weight:**

1. It writes spendable key material to disk — the worst class of thing to get
   wrong — and arrived as one clause of a long request.
2. It is new attack surface, not parity: nothing like it exists today.
3. **Decisive: it has no consumer for this wallet.** Core refuses the descriptor
   for watching *and* for signing alike — the keyless-tier rule is checked
   before signing ability matters — so an `export-signer` artifact would import
   into nothing. Building it now yields the most dangerous file in the plan with
   zero function.

**Trigger — BOTH required:**

1. A named wallet whose descriptor-with-keys is **measured to import**
   (per-entry `success: true` on a pinned target version — an **import** test,
   not an emit test; this is C1's lesson and the founding error of the whole
   export cycle was accepting emission as evidence of import).
2. A renewed operator ask naming that wallet and the hot-load purpose.

**Contract, if built** — already ruled sound, only its trigger withheld: a
distinct `mnemonic export-signer` subcommand; **account-level xprvs, never
master** (master xprvs with hardened paths trip a Core duplicate-key false
positive, root-caused to `PubkeyProvider::operator<`, reproduced on v29 and
v31.1); `--output` required; `0600` + `create_new`; always-on advisory; no
interactive confirm. R0 still applies.

### F-231 — the OTHER two fixtures still carry the defects the RCW just fixed (owning phase: **journeys**) `#journeys` `#funds-safety` `#wallet-design`

**Filed 2026-08-22**, while applying the two RCW rulings (F-229 and the
double-hash fix). Those rulings were scoped to the reasonably-complex wallet on
purpose. Two sibling fixtures were left alone, and each still has one or both of
the problems:

| fixture | keyless tier? | hashlock satisfiable? |
| --- | --- | --- |
| `fixtures/reasonably-complex-wallet` | **no** — fixed | **yes** — fixed |
| `journeys/inputs-hashvault` | **yes**, tier 4 | **no** — single-hashed |
| `journeys/inputs-pathological` | no (all tiers keyed) | **no** — no preimage committed at all |

**The hashlock problem, stated once.** Miniscript's `sha256(H)` compiles to
`OP_SIZE <32> OP_EQUALVERIFY OP_SHA256 <H> OP_EQUAL` — read off the compiled leaf
script, `OP_SIZE OP_PUSHBYTES_1 20 …`, where `0x20` is 32. **The witness preimage
must be exactly 32 bytes**, consensus-enforced. The hashvault fixture commits to
`sha256(phrase)` where the phrases are 34–40 bytes, so those tiers cannot be
spent by anyone. The pathological fixture commits to `sha256(a84dce40…)` with no
preimage in the repo at all, so its hashlock tiers are unspendable *and*
unreproducible.

**The RCW's fix, for reuse:** commit to `sha256(sha256(phrase))` and use the
32-byte inner digest as the witness preimage. The passphrase stays human; the
recoverer hashes it once. `derive-rcw-keys.sh` writes both `preimage-N.txt` (the
phrase) and `preimage-N.hex` (the preimage) and refuses to emit a preimage that
is not 32 bytes or whose double-hash is not a literal in `tr.policy`.

**The decision is per-fixture and is NOT obviously "converge".** A case exists
for leaving hashvault exactly as it is: it is the *historical* keyless shape, its
journey PDF and transcript are evidence of a real device run against it, and
something should still pin the shape that F-229 ruled against — otherwise no test
covers the case the ruling exists to prevent. If that is the call, say so in the
fixture's README so the next reader does not "fix" it.

**Not to be actioned by an agent** for the same reason F-229 was not: changing a
vault's spending conditions is the operator's signature. Filing the options only.

### F-232 — the RCW journey artifacts describe a wallet that no longer exists (owning phase: **journeys**) `#journeys` `#docs`

**Filed 2026-08-22.** The two RCW rulings changed the wallet's identity —
seven keys instead of six, three new hash literals — so every id and address
moved:

| | before | after |
| --- | --- | --- |
| policy chars (tr / wsh) | 575 / 519 | 616 / 560 |
| tr template-id | `68a1a888385797337ce5debc90fcfb1e` | `a00772edbdbb41fb4acb450672c5e5cb` |
| wsh template-id | `daee67be4eacf85e8b832ae64fc06566` | `6c635eac0f5a772d80c2eb7a43872bc8` |
| tr policy-id | `a0b128ceaef3155a40af6f8e88765ecb` | `fa568be08b48847595bf536db6a1f74d` |
| wsh policy-id | `9c74e0d2e96dd80c605b5fea19d551a9` | `f095e31101e2c77139d77c98c5d6d9f6` |
| keyed md1 chunks | 15 | 16 |
| keyless md1 chunks | 4 | 4 (unchanged) |

Before-column values are `git show HEAD:design/fixtures/reasonably-complex-wallet/README.md`;
after-column values are `md 0.13.0` run against the updated policies.

**Already updated** (so this item is only about what remains):
`fixtures/reasonably-complex-wallet/*.policy` and its `README.md`,
`derive-rcw-keys.sh`, `inputs-rcw/`, `check_tiers.py`, `NewFeatureIdeas.md`.

**`check_tiers.py` earned its keep here.** It is the gate on the prose tier table
and it went red the moment the policy changed — four correct failures, exit 1.
It now derives the three literals from the preimage files instead of hard-coding
them, so it cannot drift from the generator, and its keyless assertion is
**inverted**: it used to require exactly one keyless branch, and now requires
zero. Mutation-checked both ways — stripping `pk(@6/…)` back out makes it fail
with exit 1 on both the tier-4 predicate and the keyless count.

**REPAIRED 2026-08-22, after running it.** `transcript_rcw.sh` was executed and
went red on five gates. Four were real and are fixed; the scripts are updated,
not their recorded output:

- three key loops iterated `0 1 2 3 4 5`, so `@6` was never supplied and
  `md address` returned nothing — *"expected 5 receive addresses, got 0"*;
- the seed-fingerprint gate demanded exactly 6;
- six `--experimental` flags, no longer needed now that every tier requires a
  signature;
- prose: "six seeds", "six masters", "@0 through @5", "Tier 4 has NO KEY",
  "EXACTLY ONE tier is keyless".

`build_pdf_rcw.py` had the same counts **and one that would have failed
silently**: `section(tx, "2. Six seeds")` is a live lookup keyed on the
transcript's heading, so once the transcript printed "Seven seeds" the PDF would
have dropped that whole section without erroring.

**THE NEGATIVE CONTROL HAD GONE VACUOUS — the worst of the five.** It mutated
tier 4 to ADD a key and required the gate to reject it. Post-ruling the gate
*wants* a key there, and the `sed` no longer matched anything, so it fed the gate
a byte-identical copy. That copy sat in a scratch directory with no sibling
`preimages/`, so the gate exited non-zero for a MISSING FILE, and the control
read that as success. It would have gone on printing *"AND THE GATE CAN FAIL"*
forever while proving nothing.

Fixed in three places, because one was not enough:

1. `check_tiers.py` now separates **exit 1 (the policy is wrong)** from **exit 2
   (could not evaluate)**, and takes `--preimages DIR` so a policy outside
   `inputs-rcw/` can still be checked. A gate whose failure modes are
   indistinguishable by exit code cannot be used by a control.
2. The control's mutation is **inverted** — it now REMOVES `@6`, the regression
   that would actually matter — and aborts if the mutation does not apply
   exactly once, so a future no-op cannot pass silently again.
3. The control demands **exit 1 specifically**, and its failure message names the
   exit-2 case explicitly.

Verified: real policy 0, copy without preimages 2, copy with `--preimages` 0,
unreadable policy 2, de-keyed policy 1.

**WHAT REMAINS — one gate, and it needs hardware, not editing.** With
`ME_PREVIEW_BIN` set, the transcript now has exactly one FATAL:

    the BSMS canary, md and the device do not agree on the first address

    BSMS canary (mnemonic-toolkit) : bc1qmm0vfnpxpsst2jv973tenr9e4hrfaxcjv3ck0fcgyf2ps24vcw6szr5a29
    md address  (descriptor-mnemonic) : bc1qmm0vfnpxpsst2jv973tenr9e4hrfaxcjv3ck0fcgyf2ps24vcw6szr5a29
    SeedHammer II (captured walk)     : bc1qr6h5gahcaqa8a35p3ts0d2w6qvhmsn7dhunu5xd9kyculcgz3dwqf266zj

The two host implementations **agree with each other** on the new wallet — that
cross-language check still passes.

**Settled while investigating, so the re-run does not re-open it:** the device
proves **two addresses per chain**, and that is enough. Operator ruling
2026-08-22, *"Two addresses is fine, we don't need 5."* It agrees with the
firmware, which pins `addrProofPerChain = 2` with a test
(`gui/wallet_policy_test.go:33`) and argues at the constant that a chain
mismatch silently loses funds, so proving both chains beats proving one chain
five times. The host keeps deriving five: the device proves a sample, the host
proves the range. Recorded at `DEVICE_PER_CHAIN` in `capture_rcw.py`. The device value is read from a captured JSON
of a walk against the OLD wallet, so it cannot match and no code change will make
it. **Re-run `capture_rcw.py --wrapper wsh --route seating`** against the
emulator, then rebuild the PDF. `transcript_rcw.txt` (old template-id at lines
196/234/235/287, old address at 537) is rewritten by that run — it is a record of
something that happened, so it is regenerated, never hand-edited.

**Do NOT hand-edit `capture_hashvault.py`.** Its `TEMPLATE_ID` constant is
`68a1a888…`, which is still correct: the hashvault wallet was not changed, and it
still hashes to that value. Verified, not assumed.

### F-233 — `rust-miniscript` sanity-checks `Tr` only, so one wallet's two wrappings disagree about their own validity (owning phase: **the tr/wsh cycle**) `#codec` `#funds-safety` `#upstream`

**Filed 2026-08-22**, found while keying the RCW's tier 4.

`Descriptor::from_str` runs `sanity_check()` **only** for the `Tr` variant —
`miniscript-13.1.0/src/descriptor/mod.rs:1052-1057`, guarded by
`if let Descriptor::Tr(ref inner) = ret`, with an upstream `FIXME` calling it
*"preserve weird/broken behavior from 12.x"*
([rust-miniscript#734](https://github.com/rust-bitcoin/rust-miniscript/issues/734)).

**The observable consequence**, measured on the RCW before the fix: the keyless
`tr` form was **REFUSED** — *"All spend paths must require a signature"* — while
the byte-equivalent keyless `wsh` form was **ACCEPTED**. Same four tiers, same
keylessness, opposite verdicts, no warning on the accepting path.

**Why this matters beyond one fixture.** `requires_sig` is the same check that
closes Bitcoin Core and Nunchuk (`NeedsSignature()`), so it is a genuine interop
predictor — but only the `tr` path reports it. A `wsh` policy that will be
refused by every third-party wallet sails through the constellation's own
tooling. Anything in `md`/`mnemonic` that treats "parsed OK" as "this will
import" inherits the asymmetry.

**Worth checking, not yet checked:** whether `md encode --experimental`'s
relaxation is likewise `tr`-only in effect, and whether the Go port
(`Rust-primary rule`) reproduces the asymmetry or diverges from it — a divergence
here would be a cross-language behaviour difference of exactly the class F-212
was.

### F-234 — every QR carries the STANDARD form, never a codex32 string: a constellation-independent recovery path (owning phase: **Goal 1 — Engrave a Transaction**; RE-OWNED 2026-08-24, **OVERDUE** — its original owning phase was *the mt cycle*, which closed with this open because QR was deferred out of v0.1 entirely and nothing re-scoped it) `#mt` `#qr` `#recovery` `#firmware` `#md`

**Operator directive, 2026-08-22:** *"convert all QR codes to remove all
codex32-style encoding … this way we have a constellation independent
information recovery format."*

**The principle.** A plate should carry two representations with two different
audiences and two different failure modes:

| | engraved TEXT | engraved QR |
| --- | --- | --- |
| format | codex32 (`md1`/`mk1`/`ms1`) | **the standard form only** — raw tx bytes, BIP-380 descriptor, BIP-39 words |
| audience | a human with eyes and a keyboard | anyone with a camera and standard Bitcoin tooling |
| error correction | BCH — fixes **transcription** slips | Reed-Solomon — fixes **physical damage** |
| needs constellation knowledge? | **yes** | **no** |
| survives a dead decoder? | yes | no |
| survives a scratched plate? | degrades gracefully | cliff — kill the finder patterns and nothing decodes |

Putting a codex32 string inside a QR pays for error correction **twice** — BCH
inside Reed-Solomon — on data a machine was going to read anyway, and it still
leaves the recoverer needing to know what `md1` means. **That second reason is
the load-bearing one.** The QR's whole value is being the escape hatch for
someone who has the plate and none of our tools; a codex32 payload throws that
away for nothing.

The *density* cost is real but small — **9%**, not the ~60% the 8/5 expansion
suggests, because uppercased bech32 lands in QR's alphanumeric mode. Measured
below in "Density by representation"; do not argue this follow-up on capacity
grounds.

**Current state — CORRECTED 2026-08-24 at the re-own.** The paragraph this
replaces read *"verified, and better than feared … nothing needs undoing there"*.
That conclusion was wrong, and its evidence had decayed. Re-measured against
`bg002h/seedhammer` at `a91df84` by enumerating **every** non-test `qr.Encode`
call site rather than by re-reading the three the original survey named:

| site | QR content | verdict |
| --- | --- | --- |
| `backup/backup.go:160` | BIP-39 words (`seedQRLevel`) | standard |
| `gui/gui.go:687` | `desc.EncodeNoChecksum()` — BIP-380 | standard |
| `gui/gui.go:789` | `seedqr.QR(m)` — SeedQR | standard |
| `backup/fit.go:154` | free text | n/a — no standard form exists |
| `backup/passphrase.go:102` | the passphrase | n/a — no standard form exists |
| `cmd/biptool/main.go:354` | host tool, not firmware | out of scope |
| **`gui/gui.go:2514`** (`validateMdmk`) | **the `md1`/`mk1` codex32 string itself** | **VIOLATION** |

**`validateMdmk` is the violation this entry existed to forbid, and the original
survey missed it.** It is not a corner: four production call sites reach it —
`gui/derive_xpub.go:575`, `gui/bundle_flow.go:459`, `gui/gui.go:2570`
(`mdmkFlow`), `gui/unlock_platelist.go:222` — so every md1/mk1 plate cut with a
QR today carries a payload no camera-and-standard-tooling recoverer can use.

**Both citations the original survey did give have decayed**, which is why
re-reading them would not have caught this: `backup/backup.go:76-77` is now
`:160`, and `gui/gui.go:401` (`EncodeCompact()`) is now `gui/gui.go:687`
(`EncodeNoChecksum()`). Both remain standards-based — the conclusion about
*those two* held — but a survey that names three sites and concludes about all
of them was never measuring what it claimed. **Enumerate the call sites; do not
re-read the ones you already know about.**

- three plate modes already exist — `TEXT + QR`, `TEXT ONLY`, `QR ONLY`
  (`gui/gui.go:411-423`), so the dual-representation layout is precedent.
- `me bundle` emits **no QR at all**: constellation plates are text-only today.

So the work is (a) write the principle down so it is not eroded, (b) extend it
to `me bundle`'s plates and to `mt`, (c) **never** let an `md1`/`mk1`/`ms1`
string become QR content, and (d) **remediate `validateMdmk`'s four callers** —
which is new scope this entry did not previously carry, and which needs its own
ruling, because for an `md1`/`mk1` card the "standard form" is not obvious the
way raw transaction bytes are.

**What it buys, measured** (`design/measurements/RESULTS_qr_physical_max_2026-08-22.txt`).
The plate is not the limit — at the 0.3 mm stroke floor it could hold 263
modules across, while the largest QR that exists is v40 at 177 modules (53.1 mm
of a 79 mm square, **49% linear headroom**):

| one plate, raw bytes | L (~7%) | H (~30%) |
| --- | --- | --- |
| one v40 @ 0.3 mm | 2,953 B | 1,273 B |
| 2x2 v26 @ 0.3 mm (Structured Append, 4 symbols) | **5,468 B** | **2,372 B** |
| one v26 @ 0.6 mm (2 strokes, conservative) | 1,367 B | 593 B |

Against **38 bytes** on a codex32 text plate today. Even at the *strongest*
error correction a single plate carries a whole signed transaction: 2-11 inputs
for the RCW depending on spend path, 21 for key-path.

**Prerequisite that gates the small-module numbers — NOT yet run.** 0.3 mm is one
stroke wide. It is the theoretical floor, and whether a camera reads 0.3 mm
engraved modules off brushed steel is a **hardware** question, not a
calculation. The font work established a two-stroke minimum for *glyph*
legibility; a QR module is a solid square, so the constraint genuinely differs —
but it has to be tried. Cut a test plate with QR blocks at 0.3 / 0.45 / 0.6 /
0.9 mm modules and scan them. Until that exists, **design against the 0.6 mm
column**. Cheap: the single-character test-plate technique cuts in ~2 s rather
than ~21 min.

**Open questions.**

1. **Raw bytes, base45, or uppercased text inside the QR?** Measured below —
   the density spread is smaller than assumed, and **base45 is the candidate to
   beat**. See "Density by representation".
2. **Raw bytes or UR?** The fork vendors `bc/ur`. UR is self-describing and
   multi-part, which is worth something for recovery, at a size cost — and it is
   arguably a *third* format to know, which cuts against the whole point.
3. **What states the QR's content type**, so a recoverer knows whether they are
   holding a transaction, a descriptor or a seed? A plate legend is the obvious
   answer and costs text budget.
4. **Does `QR ONLY` become dangerous under this rule?** A QR-only plate has no
   hand-transcribable fallback at all — exactly the property the constellation
   exists to provide. Likely: QR ONLY stays available but is never the default,
   and the guide says why.
5. Reed-Solomon's percentages are **per RS block**, not per symbol, so a single
   deep scratch can exceed one block's budget while total damage looks fine.
   Tiling helps — four independent symbols fail independently, and Structured
   Append reports *which* one is unreadable.

#### Density by representation — and a correction to this entry's own argument

Measured 2026-08-22, `design/measurements/RESULTS_qr_modes_2026-08-22.txt`.
Source bytes of a signed transaction that fit **one v40 symbol**:

| representation | ECC L | ECC H | vs binary | why |
| --- | --- | --- | --- | --- |
| **raw binary** | 2,953 B | 1,273 B | **100%** | 1 octet per byte |
| base45 (RFC 9285) | 2,864 B | 1,234 B | **97%** | 3 alphanumeric chars per 2 bytes; built for QR |
| **base32 / bech32 UPPERCASED** | 2,685 B | 1,157 B | **91%** | 5 data bits in a 5.5-bit char |
| base64 | 2,214 B | 954 B | 75% | mixed case -> byte mode |
| base58 | 2,162 B | 932 B | 73% | mixed case -> byte mode |
| hex uppercase | 2,148 B | 926 B | 73% | 2 chars/byte, only partly offset |
| decimal digits | 2,943 B | 1,269 B | 100% | densest mode, worst expansion — they cancel |

QR's alphanumeric mode is **UPPERCASE ONLY** (0-9 A-Z space `$%*+-./:`), at 5.5
bits/char against byte mode's 8. That single fact drives the whole table.

**This corrects the size argument made above.** Codex32 uppercased lands in
alphanumeric mode and costs only **9%** against raw binary — not the ~60% its
8/5 expansion suggests, because the alphanumeric discount very nearly cancels
the expansion. bech32 is case-insensitive and the fork already uppercases before
engraving (`strings.ToUpper(plate.Seed)`, `backup/backup.go:76`). So **the case
for raw bytes in the QR rests on CONSTELLATION-INDEPENDENCE, not on density.**
The directive stands; one of its stated reasons was overstated and should not be
repeated as a capacity claim.

Note also that **base58 is the worst practical choice** at 73% — it needs
lowercase, so it drops to byte mode and pays 8 bits for 5.86 bits of data. It is
the instinctive Bitcoin encoding and it is the wrong one here.

**Why base45 may beat raw binary in the field.** Raw binary is the most
efficient and the least robust: many QR scanners assume UTF-8 and mangle octets
>= 0x80, and this encoder emitted an **ECI header** for high bytes — costing
~0.5% and, more tellingly, marking binary payloads as a special case in real
toolchains. That is exactly why UR uses an alphanumeric-safe encoding. base45
gives essentially binary's density (97%) with none of the charset hazard, and it
is an RFC rather than a constellation invention, so it does not reintroduce the
problem this follow-up exists to remove. **Recommend measuring both against a
real scanner on a real engraved plate**, together with the 0.3/0.45/0.6/0.9 mm
module test above — same test plate, same cycle.

#### What codex32 wrapping actually costs — whole plates, and a hard wall

The 9% figure above is per CHARACTER and understates the practical cost, because
a payload has to fit a whole number of discrete symbols. Measured on real
multisig PSBTs, 1-in/1-out, into v26 ECC L QRs
(`RESULTS_psbt_qr_multisig_2026-08-22.txt`):

> **CORRECTED 2026-08-24 (at the F-234 re-own). EVERY CHUNK NUMBER BELOW WAS
> WRONG, AND SO WAS THE CONCLUSION DRAWN FROM THEM.** Two independent defects,
> both already fixed in `SPEC_mt_v0_1.md` on 2026-08-23 and neither propagated
> here — the classic incomplete-propagation shape, where the facts are corrected
> at the source and the duplicates are left standing.
>
> 1. **The counts were ~13% low.** They imply ~44.4 payload bytes per chunk. The
>    normative rule is `count = ceil(payload_len / 40)` (`SPEC_mt_v0_1.md:1527`).
>    The 44.4 comes from the probe helper `SPEC_mt_v0_1.md:1568` retracts, which
>    modelled a chunk as `(bytes*8).div_ceil(363)` — 363 bits being what a chunk
>    *could* carry if the chunker filled it rather than balancing.
> 2. **There is no 64-chunk cap.** `SPEC_mt_v0_1.md:1548-1549`: 64 was
>    `md-codec`'s 6-bit `count` field, **which `mt1` does not use**. `mt1` spends
>    15 bits each on `count` and `index` (`:1291`), so the ceiling is **32,768
>    chunks / 1,310,720 B** — above Bitcoin's own ~100 KB standardness limit.
>
> The `codex32 chars` and `codex32 -> QRs` columns came from the same broken
> probe and are **NOT re-measured here** — recomputing them from a rule-of-thumb
> would substitute arithmetic for a measurement, which is how the original error
> got in. They are struck and left for the next QR sizing run, which must carry
> the mode-segmentation gate anyway.

| wallet | PSBT | codex32 chars | **codex32 chunks** | raw -> QRs | codex32 -> QRs |
| --- | --- | --- | --- | --- | --- |
| 3-of-5 `wsh` | 647 B | ~~1,440~~ unmeasured | ~~15~~ → **17** | **1** | ~~1~~ unmeasured |
| 3-of-5 `tr` | 950 B | ~~2,016~~ unmeasured | ~~21~~ → **24** | **1** | ~~2~~ unmeasured |
| 9-of-11 `wsh` | 1,237 B | ~~2,688~~ unmeasured | ~~28~~ → **31** | **1** | ~~2~~ unmeasured |
| 9-of-11 `tr` | 1,732 B | ~~3,744~~ unmeasured | ~~39~~ → **44** | 2 | ~~2~~ unmeasured |

The chunk column is now exact rather than probed: it is `ceil(bytes / 40)`
applied to the PSBT sizes in column 2, which is normative arithmetic from §3.

**codex32 is EXPENSIVE, not walled.** The chunk counts are also the engraved-TEXT
cost, at one plate per string today: **17-44 plates, 6-15 hours**, for artifacts
that fit on ONE plate as a raw QR. A 9-of-11 `tr` with a change output added
(3,293 B) is **83 chunks — 0.25% of `mt1`'s ceiling, and ~83 plates.** The raw QR
takes 3 symbols on one plate.

> **The retracted sentence said that case was "unencodable at any plate count".**
> It is encodable. The design answer changes from *impossible* to *expensive*,
> and those call for different things: an impossibility earns a refusal, an
> expense earns a stated plate count and the operator's decision. **Any design
> that inherited "unencodable" from this entry is reasoning from a fact that was
> retracted the day before it was read.**

#### Signed transactions are the smaller artifact, and the gap is wrapper-shaped

Same wallets, 1-in/1-out sweep:

| wallet | unsigned PSBT | signed tx | signed as % of PSBT |
| --- | --- | --- | --- |
| 3-of-5 `wsh` | 647 B | 488 B | 75% |
| 3-of-5 `tr` | 950 B | 501 B | **53%** |
| 9-of-11 `wsh` | 1,237 B | 1,130 B | 91% |
| 9-of-11 `tr` | 1,732 B | 1,097 B | **63%** |

Taproot is the EXPENSIVE wrapper for a PSBT and the CHEAP one for a spend: the
PSBT carries `tap_key_origins` and leaf data for every key, while the spend
reveals one leaf. `wsh` is the reverse. So "which wrapper is bigger" has no
answer independent of which artifact is being stored — worth stating in the spec,
since `mt`'s scope is signed transactions only and the intuition from PSBT work
points the wrong way.

**Measurement caution, recorded because it bit three times.** The encoder does
optimal MODE SEGMENTATION and will silently re-encode parts of a payload in a
denser mode. An all-`0x41` payload measured *alphanumeric* capacity while
claiming byte; a high-byte payload paid an ECI header; a digits-and-letters
payload split into numeric and alphanumeric segments and read 6.6% low. Every
one produced a plausible-looking number. Only asserting measured v40 capacity
against the published limits (numeric 7089 / alnum 4296 / byte 2953 at L) caught
them. **Any future QR sizing work must carry that gate.**

### F-235 — CLOSED 2026-08-24 — `mt` rendered every address with MAINNET parameters, so a testnet or regtest transaction showed an address that does not exist (owning phase: **post-v0.1 UX**) `#mt` `#report` `#LOW`

**Found by running P5's fixtures, 2026-08-24.** The `OUT` row of a regtest
transaction reads

    OUT       1 output(s)
                bc1q07h88fcj0j86excq5m9k97e26su7j5tdvldytq   7.99900000 BTC

while the node calls that same output `bcrt1q07h88fcj0j86excq5m9k97e26su7j5tdys0686`.
Same witness program, different HRP and therefore a different checksum — so the
printed string is not an address on any network the transaction belongs to.

**Why it is LOW and not Important.** A transaction's `scriptPubKey` carries no
network, so `mt` genuinely cannot know; mainnet is the only defensible default
for a mainnet tool, and the spec is silent. The row is `stderr`-only and
disposable. Crucially **the legend's `TO` line does not come from here** — §5
takes it from `--to` / `--to-label`, which are the operator's own assertion
(§10.4), so nothing wrong reaches steel.

**What would close it.** Either a `--network` flag defaulting to mainnet, or
deriving the network from the node when one is reachable and saying `network
UNKNOWN — shown as mainnet` when not. The second is better: it needs no new
operator input, which is §6a's posture. **Not** silently guessing from address
prefixes in the inputs — there are none, the inputs are outpoints.

**Do not fix this by suppressing the address.** The address is the single most
useful row for a recoverer deciding whether to broadcast; a hex `scriptPubKey`
is not a substitute.

### F-236 — CLOSED 2026-08-24, by the adversarial review rather than by this entry — `--input-value` took BTC as an `f64` (owning phase: **post-v0.1 UX**) `#mt` `#funds-safety` `#LOW`

**Noticed while writing P5's tests, 2026-08-24.** `parse_input_values` does
`btc.parse::<f64>()` then `(btc * 100_000_000.0).round()`. For every value a
person will actually type this is exact — `f64` has 53 bits of mantissa and
21 M BTC is under 2^51 satoshis — so the rounding recovers the intended integer
and no realistic input is wrong.

**It is recorded anyway because the reasoning is load-bearing and invisible.**
The safety rests on a bound nobody has written down next to the code, and the
same expression in a context with larger numbers would be a defect. The
constellation's own lesson applies: an enumerated safety argument that lives
only in someone's head goes stale silently.

**What would close it.** Parse the decimal STRING into satoshis directly —
split on `.`, require at most 8 fractional digits, reject anything else — which
also gives a better refusal for `0:1.234567891` (nine decimals) than silently
rounding it. That refusal is the actual user-visible gain, not the arithmetic.

### F-237 — CLOSED 2026-08-24 — `md1`/`mk1` strings reached `mt decode`'s codec and were reported as bad bech32 rather than as a sibling's material (owning phase: **post-v0.1 UX**) `#mt` `#refusals` `#NIT`

**Noticed while implementing §8.9, 2026-08-24.** §8.9 refuses `ms1` *before*
§8.2e's byte-naming, because that refusal prints the first eight bytes and for a
secret those bytes are the secret. The sibling formats have no such hazard —
`md1`/`mk1` are watch-only public material — so nothing was added for them.

The consequence is only a message-quality one: hand `mt decode` a valid `md1`
string and it reports a codec error about symbols and checksums rather than
*"this is an `md1` descriptor string; you want `md`"*. The operator is holding
the right material for the wrong tool and the tool does not say so.

**Deliberately NOT done in v0.1.** It would put knowledge of the sibling HRPs
into `mt`, which currently knows only its own — and the constellation's
fork-per-codec ruling (2026-05-03) exists precisely to stop that coupling
spreading. A one-line hint keyed on the literal prefixes `md1`/`mk1` would be
enough and would not import anything; that is what to write if it is done.

### F-238 — CLOSED 2026-08-24 — §5 and §8.4's worked example `~FALL 2034` disagreed with §8.4's own algorithm, which gives SUMMER (owning phase: **the mt spec, next touch**) `#mt` `#spec` `#MINOR`

**Found by implementing it, 2026-08-24.** §8.4 rules the projection exactly:

    estimated unlock = MT_REF_TIME + (target_height − MT_REF_HEIGHT) × 600 s
    MT_REF_HEIGHT = 963_759
    MT_REF_TIME   = 1_787_507_701

For the worked example, block **1,383,520**: 419,761 blocks × 600 s =
251,856,600 s, so the projection is **2034-08-16T18:05Z**. §8.4 also rules the
seasons as *northern-hemisphere meteorological quarters*, under which August is
**SUMMER**. Both §5's legend table and §8.4's own example render it `~FALL 2034`.

**The spec anticipated this exact case and it is why the finding is Minor.**
§8.4: *"The exception is a projection landing near a season boundary, which can
tip"*, against a measured drift of *"+16 to −34 days"* over this very span — and
the projection lands **15 days** from the September boundary. So the `~` is doing
precisely the work it was put there for.

**What was done.** The algorithm is implemented as ruled and pinned by
`locktime::tests::the_worked_example_projects_to_summer_not_the_spec_s_fall`,
with the boundary table pinned separately in
`the_season_boundaries_are_the_meteorological_quarters`. **The code follows the
rule, not the example** — the reverse would have been the defect, since the
example is one datum and the rule governs every plate.

**What to change.** Either re-render the example as `~SUMMER 2034` in both
sites, or pick a worked height that lands mid-season so the example stops being
a boundary case. The second is better: an example that tips is an example that
will disagree with some future reference pair too.

### F-239 — CLOSED 2026-08-24 — §8.4 gave ONE state two normative spellings and never said they are different surfaces (owning phase: **the mt spec, next touch**) `#mt` `#spec` `#MINOR`

**Found by the post-implementation spec-conformance review, 2026-08-24 (S-2).**
For a transaction with a non-zero `nLockTime` and every input final:

- §8.4's list of report spellings gives
  `nLockTime 900000 present but NOT ENFORCED (all inputs final)`;
- forty lines below, §8.4 rules *"`NO TIMELOCK` is reserved for a transaction
  with `nLockTime = 0` **or with all inputs final**"*.

One reading resolves it — the `NO TIMELOCK` sentence sits in the paragraph about
the **engraved legend**, the `NOT ENFORCED` line in the list of **`stderr`
report** spellings — **but that split is never stated**, and §1.1 binds the
report's `LOCKTIME` row to *"§8.4's five normative spellings"* as a single set.

**This is the same two-spellings-one-input class §8.4 itself calls a real
defect** (R6 implementability I-8), landing on §8.4.

**What was implemented**, on the resolving reading: `Lock::report_row` emits
`nLockTime N present but NOT ENFORCED (all inputs final)` and `Lock::legend`
emits `NO TIMELOCK`, with the split stated in the code. The legend is 11
characters and cut into steel; the report is disposable and can afford the
value.

**What to change.** One sentence in §8.4 naming the two surfaces, so the next
implementer does not have to derive it.

### F-240 — CLOSED 2026-08-24 — §1.1's row-presence table named `verify` as a report caller; §1.1's own `verify` example does not (owning phase: **the mt spec, next touch**) `#mt` `#spec` `#NIT`

**Found by the post-implementation spec-conformance review, 2026-08-24 (S-1).**
§1.1's table lists `mt1 SET` as a row `verify` produces, which reads as `verify`
being a caller of the shared report. §1.1's own worked `verify` output is a
single `OK` line plus the margin report — no report rows at all — and §1.1
elsewhere describes `verify` as **structural only**, never consulting a node,
which is exactly what makes it runnable air-gapped.

**Implemented as the examples describe**: `verify` prints its OK line, the
duplicate/unreadable notices and the margin report, and calls no node. `decode`
and `inspect` are the report's two callers.

Low stakes — nobody is misled in a way that costs anything — but it is a table
and a worked example disagreeing inside one section, and the table is what an
implementer reads first.


## Burndown — the `mt` cycle's six, closed 2026-08-24

All six were owned by **post-v0.1 UX** or **the mt spec, next touch**, and this
is that moment: the cycle is closed and nothing later owns them.

| | closed by |
| --- | --- |
| **F-235** | `Node::chain()` reads `getblockchaininfo`. Addresses render for the operator's network, and **when no node is reachable the row SAYS `addresses shown as MAINNET — no node to ask`** rather than printing one silently. Read from the node, not asked of the operator — §6a's posture, and it adds no flag to set wrongly. |
| **F-236** | Already fixed, by the adversarial review, hours before this burndown. `parse_btc` parses a decimal STRING into satoshis; the only `f64` left in the amount path is a comment describing what was removed. **The entry was stale when written down** — worth noting, because a follow-up list that is not reconciled reports work that is already done. |
| **F-237** | `sibling_format` names `md1`/`mk1` by their literal prefix and points at the tool that reads them. **Keyed on three characters and nothing else**, so `mt` imports no knowledge of the siblings' codecs and the fork-per-codec ruling stands. It also asserts `mt` echoes none of the material back. |
| **F-238** | The worked example now reads `~SUMMER 2034` in both sites, with a note recording that **the example was wrong and the rule was right** — the only ordering that could be corrected safely, since the rule governs every plate and the example is one datum. Pinned in code, with the season boundaries pinned separately. |
| **F-239** | §8.4 now states the split it always relied on: the `stderr` REPORT says `nLockTime N present but NOT ENFORCED (all inputs final)`, the engraved LEGEND says `NO TIMELOCK`. Two surfaces, two spellings, one state — said out loud, in a table, where an implementer will hit it. |
| **F-240** | The row-presence table no longer names `verify` as a report caller, with a note pointing at §1.1's own worked `verify` output — a single `OK` line — and at the structural-only ruling that makes `verify` runnable air-gapped. |

**What the burndown itself found.** F-236 was already done and nobody had said
so. Reconciling *before* working is what the per-phase rule asks for, and it cost
one `grep` here — against a list of six. On a longer list the same omission is
how a closed item gets "fixed" twice, or how a real one hides behind a stale one.

### F-241 — CLOSED 2026-08-24 — `SPEC_mt_v0_1.md` §3's retraction note stated the `count` width as **12 bits**, while §3 itself states **15** (owning phase: **the mt spec, next touch**) `#mt` `#spec` `#MINOR`

**Found 2026-08-24**, reading the spec against itself while correcting F-234's
chunk arithmetic. `SPEC_mt_v0_1.md:86` reads:

> That rested on a 64-chunk ceiling `mt1` never had — `md-codec`'s 6-bit `count`
> field, which §3 corrected to **12 bits**.

§3 says otherwise, twice, and the rest of the document agrees with §3:

| site | says | implies a ceiling of |
| --- | --- | --- |
| `:86` (the retraction note) | `count` corrected to **12** bits | 4,096 chunks |
| `:157` | `version(5) + chunk_set_id(20) + count(15)` | 32,768 |
| `:1291` | "**`mt1` uses 15 bits each for `count` and `index`** — a **55-bit** header" | 32,768 |
| `:33`, `:79`, `:1545` | ceiling is **32,768 chunks / 1,310,720 B** | 32,768 |

So `:86` is the lone dissenter and **12 is the wrong number** — 2^15 = 32,768 is
what four other sites state. Nothing derives from `:86`, which is why it is
Minor rather than Important: it is a parenthetical inside a correction note.

**Why it is worth filing anyway.** It is the fourth defect of the F-238/F-239/
F-240 class — a *record* disagreeing with itself, found by reading rather than by
running — and it sits **inside the very sentence that retracts a wrong ceiling**.
A note whose job is to stop a stale number propagating, propagating a stale
number, is the sharpest available argument for the standing rule that records are
the weak half of any cycle.

**Fix:** change `12` to `15` at `:86`. One character pair, no other site moves.

**CLOSED same day, operator-confirmed** ("Header is 15 bit count/index"). `:86`
now reads *"which §3 **widened to 15 bits**"* — `widened` rather than
`corrected`, because `mt1` did not fix a defect in `md-codec`, it chose a wider
field for a different job. Re-grepped afterwards: **no `12 bits` / `12-bit`
claim remains anywhere in the spec.**

The copy at `mnemonic-transaction/design/SPEC_mt_v0_1.md` is pinned in that
repo's `PROVENANCE.md` at `aa232ca` and must be re-synced with the new SHA, or
`check-provenance.sh` goes red. That gate is not in CI (it needs both repos), so
it will not catch itself.

### F-242 — `SPEC_mt_v0_1.md` calls a chunk "~96 characters"; a full chunk is **91** (owning phase: **the mt spec, next touch**) `#mt` `#spec` `#sizing` `#MINOR`

**Found 2026-08-24**, sizing the `sysw` transaction payload for Goal 1 and
declining to hand-count what the shipped vectors could be measured for.

`:1562` reads *"one chunk is one hand-cut string of ~96 characters"*, and `:3456`
implies **88.7** for the 535 B / 14-chunk case (1,242 chars / 14). Two figures,
neither matching. **Measured** on `mnemonic-transaction/design/vectors/mt1_v1_vectors.md`
— 14 real `mt1` strings, whose chunk payload sizes the file states:

| chunk payload | measured chars | count |
| --- | --- | --- |
| 32 B (the short final chunk of the `uneven` vector) | **79** | 1 |
| 36 B | **85** | 7 |
| 37 B | **87** | 6 |

Those three points solve exactly, with no free parameters left over:

    chars = ceil((payload_bytes * 8 + 55) / 5) + 16

55 is the header (`version(5) + chunk_set_id(20) + count−1(15) + index(15)`,
`:1291`); the +16 is `mt1` (3) plus a 13-symbol checksum. It reproduces 79, 85
and 87 exactly. **A full 40-byte chunk is therefore 91 characters** — the vectors
top out at 37 B so none directly exhibits it, which is why the formula is given
rather than a fourth measured row.

**Neither published figure is right.** 96 is **5.5% high**, 88.7 is 2.5% low.
The `~96` is the one that matters, because `:1562`'s surrounding paragraph is
what a sizing calculation reaches for — it is the sentence that converts a chunk
count into an engraving cost.

**What it moves.** Nothing already shipped: `mt` chunks by bytes, never by
characters, so no code reads either number. It binds **future sizing work** —
the QR configuration search (`SPEC_mt_qr_DEFERRED.md` §4), plate legend budgets,
and any transport-capacity arithmetic. Goal 1's `sysw` payload ceiling for the
chunks form is **~3,561 B**, not the ~3,377 B that `~96` gives.

**Fix:** replace `~96` at `:1562` with the formula and the 91-character full-chunk
figure, and reconcile `:3456`'s 1,242 (14 chunks of 39 B → 14 × 89 = 1,246).

**Class.** Same as [F-241]: a *record* disagreeing with itself, found by reading
rather than running, in a document that has passed thirteen review lenses. Both
were found in one afternoon by checking numbers against the artifact instead of
against the prose — which is the standing rule, and the reason it exists.

**CORROBORATED FROM THE SPEC'S OWN TEXT, and a third site found (2026-08-24,
same day).** The 91 figure was not only derivable from the vectors — it is
**already written in the document**. `SPEC_mt_v0_1.md:1308`:

> chunk-string goes from **89 to 90 characters** at the 49-bit header this box
> was written under; at the ruled 55-bit header a 40-byte chunk is **91** and the

So the spec states **91** at `:1308` and **~96** at `:1562`, four hundred lines
apart, and the derivation from the shipped vectors agrees with `:1308`. That
settles it without needing a fourth measurement — and it downgrades the cause
from "nobody measured" to **"it was measured, written down, and then contradicted
elsewhere in the same file"**, which is the harder failure to catch and the one
worth naming.

**Third site, in a different document.** `SPEC_mt_qr_DEFERRED.md:98` lists, among
three unmodelled additive inputs making its plate table a lower bound, *"the
**49-bit** `mt1` chunk header per symbol"*. 49 is a superseded draft layout —
`:1281` and `:1291` rule the header at **55 bits** (`version(5) +
chunk_set_id(20) + count−1(15) + index(15)`), and `:1307` names 49 explicitly as
"the header this box was written under". So the deferred QR spec's own
lower-bound correction is computed against a header width that no longer exists,
which makes that table **more provisional than it claims to be** — a document
warning you its numbers are low, using a stale number to say by how much.

**Fix, revised — three sites, not two:**
1. `SPEC_mt_v0_1.md:1562` — replace `~96` with the formula and 91.
2. `SPEC_mt_v0_1.md:3456` — reconcile 1,242 (14 chunks of 39 B → 14 × 89 = 1,246).
3. `SPEC_mt_qr_DEFERRED.md:98` — 49-bit → 55-bit, and note the table must be
   regenerated against it (§10.14 already requires a regeneration for the
   font-metric correction; these are the same job).

### F-243 — F-234's case against raw octets in the QR rests on an UNTESTED scanner claim, stated in the same register as its measurements (owning phase: **Goal 1 — Engrave a Transaction**) `#qr` `#mt` `#measurement` `#IMPORTANT`

**Found 2026-08-24**, when the operator asked *"Raw octets doesn't work for some
scanners?"* and the claim was checked instead of repeated.

F-234 says, arguing base45 over raw binary:

> Raw binary is the most efficient and the least robust: **many QR scanners
> assume UTF-8 and mangle octets >= 0x80**, and this encoder emitted an **ECI
> header** for high bytes — costing ~0.5% and, more tellingly, marking binary
> payloads as a special case in real toolchains.

**Two claims in one sentence, and only one of them is ours.**

| claim | status |
| --- | --- |
| our encoder emits an ECI header for high bytes, ~0.5% | **MEASURED** — `design/measurements/mt-size-probe/src/bin/qrplate.rs:28-29` |
| many scanners assume UTF-8 and mangle octets >= 0x80 | **NOT MEASURED.** `grep -ri "scanner\|utf-8\|utf8\|mangl" design/measurements/` returns no test, no result, no apparatus — only two prose asides, one of which is `README.md:269` restating the same assertion |

**Zero scanner tests exist in this repository.** Not "inconclusive" — never run.

**The phenomenon is real in general**, which is exactly why it survived: QR byte
mode is nominally ISO-8859-1, many decoders guess UTF-8 and re-encode, and the
Bitcoin ecosystem moves PSBTs as base64 or UR rather than raw octets. But
*generally true* is not *measured here*, and the sentence's construction — a real
measurement and an untested assertion joined by "and" — makes them read as one
finding. It is the same shape as the retracted 64-chunk cap: a plausible fact,
never checked, load-bearing for a design decision.

**What it is load-bearing FOR.** It is the main argument against putting raw
transaction bytes in the QR — and raw octets are the *only* candidate that
delivers F-234's own stated promise, that a recoverer with a camera and standard
Bitcoin tooling needs no constellation knowledge. So an untested claim is
currently the reason F-234's headline goal is not being met.

**It costs almost nothing to settle, on a plate we already owe.** F-234 already
requires an optical test plate — QR blocks at 0.3 / 0.45 / 0.6 / 0.9 mm, scanned
off brushed steel — and that gate **has never been run** either. Adding one
raw-octet symbol and one base45 symbol to the same plate answers this with
evidence. The single-character technique cuts test plates in ~2 s rather than
~21 min.

**Until then the QR encoding is a PARAMETER, not a ruling** — operator decision
2026-08-24, so Goal 1's design does not stall on an unrun test and does not bake
an untested assertion into steel.

**Do not close this by re-reading the prose.** A negative inherits the scope of
the search that produced it; this one's scope was `design/measurements/`, and it
found nothing because nothing is there.

### F-244 — CLOSED 2026-08-24 — `me sysw pack` wrote the container with `std::fs::write`, so an UNSEALED payload holding a BIP-39 mnemonic landed mode 0644 (owning phase: **immediate — pre-existing defect, not Goal 1**) `#me` `#sysw` `#funds-safety` `#CRITICAL`

**Found 2026-08-24 by the Goal 1 journey walk**, at the step where the operator
said *"I didn't realize `>` creates a world readable file"*. The walk was about
transactions; it found a seed-exposure defect in shipped code.

**Reproduced, three ways, all mode 644:**

```
$ me sysw pack --no-passphrase --out p1.bin "text:6869"     -> 644   (fresh --out)
$ me sysw pack --no-passphrase --out p2.bin "text:6869"     -> 644   (pre-existing 0644)
$ me sysw pack --no-passphrase "text:6869" > p3.bin         -> 644   (shell redirect)

$ me sysw pack --no-passphrase --out s.bin "abandon ...x11... about"
$ stat -c '%a' s.bin   -> 644
$ strings s.bin        -> abandon abandon abandon ... about
```

The last one is the finding: **a BIP-39 mnemonic, cleartext, world-readable, no
warning.** (Standard BIP-39 test vector; nothing real was written.)

**Cause.** `SyswCmd::Pack` ends in `emit(&blob, out.as_ref())`
(`crates/me-cli/src/main.rs:924`), and `emit` (`:1131-1136`) uses
`std::fs::write`, which creates at `0o666 & ~umask` = 0644 under the default
umask.

**The fix already exists in the same file and is not called.**
`write_private` (`:751-762`) creates at `0o600` and its doc comment states this
exact threat model:

> F10 (D5-2): NDEF and manifest artifacts embed/depict md1/mk1 material, so **on
> a multi-user host their at-rest copies must not be world/group-readable**.

It has three callers — `:326`, `:391`, `:515` — for NDEF, manifest and UF2.
**None of them is the payload container.** So `me` protects the artifact that
*depicts* key material and leaves unprotected the container that *contains* it,
including `ClassMnemonic`, `ClassCodex32Secret` and `ClassPassphrase` records
whenever `--no-passphrase` is used. `Class::IsSecret()` already names those three
as secret; nothing consults it on the write path.

**Why CRITICAL rather than Important.** The repo's severity rule counts "security
/ an unmet guarantee". `me` states the guarantee in its own source, applies it to
a lesser artifact, and misses the greater one. A sealed payload's ciphertext is
protected by cryptography; an **unsealed** payload's records are protected by
nothing but the file mode, and `--no-passphrase` is a supported, documented mode.

**Two separate gaps, and fixing only the first leaves the one the operator hit.**

1. **`--out`** — route it through `write_private`. Straightforward.
2. **`> file` shell redirection** — `me` never sees that path, so no create-mode
   can help. **Mechanically verified during the walk that a process CAN detect
   this**: `os.fstat(1)` on a redirected stdout reports `S_ISREG` true and mode
   0644, and reports `S_ISFIFO` for a pipe — so a check can fire exactly on a
   world-readable regular file and leave `| picotool` and terminals alone.
   Operator asked for **a refusal with a command-line override**; that is
   implementable as specified.

**`write_private`'s own documented residual is also real and now measured**:
*"0o600 binds on CREATE. Overwriting a pre-existing world-readable file keeps its
old mode."* Case 2 above confirms it. A fix that only calls `write_private` still
leaves a pre-existing 0644 target at 0644 — so the write path must **also**
`fchmod` or refuse, not merely create carefully. **This is the near-miss shape:
the obvious fix closes the case the finding names and not the one beside it.**

**Prior art to match, not to duplicate.** `mt` already warns on redirected stdout
(`mt-cli/src/blocks.rs:268`, `redirected_output_warning`) — *"the strings just
left this terminal — and they are BEARER"* — with `shred -u` advice. It fires on
**any** redirection and does not consult the mode. Whatever `me` grows should be
consistent with it, and `mt`'s should probably become mode-aware in the same pass.

**Not Goal 1's scope, and it must not wait for Goal 1.** It is a defect in
shipped code affecting seeds today, independent of transactions.

**SCOPE RULED 2026-08-24 (operator): ALL of `me`, and `mt` too.** One rule across
both tools and every write path — refuse a world-readable destination unless
overridden; `--out` creates at 0600 **and** `fchmod`s an existing target.
`mt`'s `redirected_output_warning` becomes mode-aware in the same pass: today it
fires on *any* redirection, so it cries wolf on a 0600 file and warns no harder
on a 0644 one. A constellation-wide rule is the kind an operator can remember;
two tools treating one hazard differently teaches the wrong lesson from whichever
they meet first. Note `emit` is a SHARED helper — fixing `sysw pack` alone would
knowingly leave the same hole elsewhere in the same binary.

---

**CLOSED 2026-08-24, both halves, TDD throughout.**

| | commit |
| --- | --- |
| `me` — the Critical itself | `46f2fd4` |
| `mt` spec §8.2h | `f152aac` (engrave) |
| `mt` implementation | `542b391` (transaction) |
| provenance re-sync | `a76c1a9` (transaction) |

**`me`.** `emit`'s `--out` now routes through `write_private`, which additionally
`fchmod`s an existing target — the documented residual *"0o600 binds on CREATE"*,
which this entry **measured true**, so creating carefully was never enough. The
stdout side `fstat`s fd 1 and refuses a world-readable **regular file**, with
`--allow-world-readable`. The converter's `--stdout`/`--hex`/`--base64` are gated
too: all three carry the same bytes, and gating raw but not hex teaches the
operator to reach for hex.

**`mt`.** §8.2h, the other half of §8.2g. `validate.rs::world_readable_stdout_guard`,
called before a byte of stdout is written so a refusal leaves no artifact.
**Additive** to `redirected_output_warning`, not a replacement: that one is about
how long the file lasts, this one about who can read it.

**TWO NEAR MISSES CAUGHT, one of them mine.**

1. **`me sysw wipe`** — `emit` is a **shared** helper, so the new guard reached a
   command whose output is 65,536 bytes of `random`/`zeros`/`ones` with nothing
   in it, existing to **destroy** a payload. Refusing it buys no safety and costs
   a working command. Caught by asking what else the guard would now catch, then
   pinned with a test. **The fifth instance** of the pattern the `mt` cycle
   recorded.
2. **`mt encode` has no `--out`** — and the first draft of both the refusal and
   §8.2h advised one. Verified against `mt encode --help`; its absence is
   deliberate, since stdout **is** the strings by ruling (§3b). A refusal naming
   a flag that does not exist is worse than one naming none: it sends the
   operator to `--help` to look for it.

**Near misses are half of each test suite, and they PASSED before the fix** —
which is what makes them non-vacuous. They catch a guard that **over**-fires,
which is this codebase's demonstrated failure mode. Pinned: a pipe (`S_ISFIFO`)
in both tools, an owner-only `0600` redirect in both, a wipe image, and a piped
converter run.

**GATES, all green:**

```
me   cargo nextest run --locked           303 passed, 1 skipped
me   cargo clippy --all-targets           clean
mt   cargo nextest run --locked           210 passed, 0 skipped
mt   cargo clippy --all-targets           clean
mt   check-refusal-coverage.sh            31 refusal tests over 18 ruled refusals
mt   mutate-refusals.sh                   all 31 go RED without their check,
                                          INCLUDING §8.2h world_readable_stdout_guard
mt   journeys.sh                          A, B (both forms), C pass
mt   check-provenance.sh                  every copied file matches its source
```

`check-refusal-coverage.sh` **refused the new entry first**: §8.2h was not in its
seeded set. That is the typo guard working, and the seeded set now records that
this entry did not come from widening the gate — the refusal did not exist until
the walk found it.

### F-245 — `me sysw pack` packs a record's trailing whitespace VERBATIM into the public section (owning phase: **post-P1, `md1`/`mk1` path**) `#me` `#sysw` `#IMPORTANT`

**Found 2026-08-24** while machine-checking a claim in R0 round 3 of the P1 plan.
The plan cited `seal`'s `validate_record` as precedent for refusing padding. The
citation does not hold, and checking why turned up a live defect on a path P1
does not touch.

**`seal` trims before it checks** (`crates/me-cli/src/seal/record.rs:118`):

```rust
pub fn validate_record(s: &str) -> Result<RecordKind, RecordError> {
    let s = s.trim();                                   // <- TRIMS FIRST
    if let Some((pos, ch)) = first_noncanonical(s) {    // <- then checks
```

So padding never reaches the canonicality check. **And `sysw` does not merely
tolerate it — it preserves it.** Executed against the shipped binary:

```
me sysw pack --no-passphrase "<md1 string> "     -> exit 0
the packed record's last byte                    -> b' '
```

**The space rides into the public section intact**, and EPD §6.4 states the hazard
in its own words: records engrave **verbatim**, so a character outside the BCH
checksum's coverage *"turns a scratch on the operator's only copy into
silently-absorbed damage rather than a detected error."*

**Why Important rather than Critical.** It needs the operator to supply a padded
record, and the most likely source — a copy-paste with a trailing space — is
plausible but not automatic. Nothing is lost silently *today*; the damage is
deferred to a scratch on a plate cut from the padded string.

**Scope.** This is the **`md1`/`mk1`** path. P1's `tx:` records are refused for
padding by its own E13, which stands on its own reasoning now that the false
precedent is struck. **P1 neither introduces this nor is scoped to fix it.**

**What would close it.** Refuse rather than trim, or trim and refuse the
difference — but check the device side first: `gui/scan.go` may already tolerate
or reject padded records differently, and a host that refuses what the device
accepts is its own defect.


---

### F-246 — `me sysw pack` generates and PRINTS a passphrase before it validates the records, so an invalid input still emits secret material

**DONE 2026-08-25** — `mnemonic-engrave` `08c9c80` (both instances: admission hoisted out of `split` into `sysw::admit_check`, and the write gate hoisted above every report line) and `9952c7f` (the gate repositioned so R2 still outranks it).

**Severity:** Minor. **Owning phase:** post-P1 UX (not P1 — P1 neither introduces
this nor is scoped to fix it). **Found:** 2026-08-24, while verifying R0 round 4's
C2 against the shipped binary.

The C2 check packed six bare `mt1` records to confirm they are refused. They are —
`exit 4`, *"record 0 (records count from 0) is not a form this container can
place"*. But the refusal is not the first thing that happens:

```
$ me sysw pack --in <six bare mt1 records> --out /tmp/t.bin
passphrase — write this down and store it APART from the machine:

    parade accident toilet various cluster demand dress afraid around system crunch vapor

strength: 12 words — at or above the threshold
me: record 0 ... is not a form this container can place: ...
exit 4
```

**A twelve-word passphrase is generated and written to stdout, and then the pack
aborts having produced no container.** The passphrase is meant to be shown — that
is not the defect. The defect is that it is shown for a run that produces
**nothing**, so the operator is handed material to "write down and store apart"
that protects no artifact, immediately above an error telling them the run failed.

**Why it matters beyond tidiness.** The instruction is imperative and the operator
is told to record it off-machine. An operator who follows it now holds a
passphrase with no payload, and the natural next move — fix the record, re-run —
generates a **different** one. The first is now a written-down secret that opens
nothing, and the operator has no way to tell which of the two is live.

**Why Minor, not Important.** Nothing is lost and nothing is exposed that the
successful path would not also print; the run visibly fails at exit 4. The cost is
operator confusion and a stray written secret, not a wrong result.

**Related:** this is the shape of [`a guard downstream of the parser has lost`] —
work performed and output emitted before the validation that would have made it
unnecessary. Same class as clap echoing a bearer transaction before the refusal
ran (spec §1.5's *what runs before it*).

**What would close it.** Validate and classify every record **before** generating
or printing any passphrase. Cheap: the classification pass already exists and
already runs; it simply runs second.
---

**SECOND INSTANCE, found in the side-by-side walk 2026-08-25 — the DIGEST, on a
different trigger.** The operator typed the bare command and pasted one `tx:`
record:

```
$ me sysw pack                       (paste the record, Ctrl-D)
sealing:  NOT SEALED — no record in this payload is secret material ...
strength: no passphrase — BELOW the threshold
digest:   7981 04fa 8223 f3fc 8839 6701 2f0b 5a8e
          re-print it with: me sysw show <the file you just wrote>
me: stdout is a world-readable file, and this payload is BEARER.
    --out <FILE> / umask 077 / --allow-world-readable
exit 2, and the redirected file is 0 bytes.
```

**Same defect, and the artifact handed over is worse.** A passphrase protects
nothing when no container exists; a **digest is the value the operator verifies
the plate against on the device.** Recording this one means carrying a checksum
for a payload that was never written — and the line directly beneath it says
*"the file you just wrote"*, which is false at the moment it prints.

**This widens what would close it.** The original fix — classify records before
printing the passphrase — does not reach here, because the trigger is not record
validity but the **world-readable stdout guard**, which fires after a wholly valid
pack. The rule that covers both: **no report line describing a container may be
emitted until every gate that can abort the write has run.** Sealing, strength
and digest all describe an artifact, so all three belong after the guard.

**Not a defect:** the refusal itself names three concrete ways on (`--out`,
`umask 077`, `--allow-world-readable`). *(An earlier version of this paragraph
said `mt`'s equivalent offered none, citing F-249. That was wrong — `mt` names
three too, and F-249 has been WITHDRAWN as false-as-filed. Both binaries print
remedies here; neither needs to copy the other.)*
---

### F-247 — `mt encode --qr` does not say whether the record fits an NFC TAG (owning phase: **P2, and it needs an operator ruling FIRST**) `#mt` `#nfc`

**NOT DONE, and deliberately — operator, 2026-08-25: "skip nfc stuff for now."** It was in the burndown range but is the one item there that needs a ruling rather than an implementation, and its own text below forbids grafting the worked reference without one. Stays open at its stated owning phase.

**Filed 2026-08-25 during P3b, deliberately NOT implemented.**

`SPEC_engrave_transaction.md` §2.3 item 2 (line 450) and the §6 P2 scope row both
say the record must state whether it fits an **NFC tag** — `gui/scan.go`'s 8 KB
scan buffer, **8191**, and explicitly **not** `MaxSectionLen` (32,734). The two
caps belong to different transports: a `sysw` container has exactly one (picotool
to `0x10D00000`), while a bare record on a tag is bounded by the scan buffer, so
quoting the container's number on a record would promise a journey the artifact
cannot take.

**Why it is not built.** The operator has not ruled on it, and the graft's brief
scoped it out in those words. The parallel arm (`exp/tx-plan-driven` `fc7072a`)
did build it — a `RECORD    <n> characters — fits an NFC tag (8191 max…)` line on
stderr, with a `TOO LARGE … deliver it by me sysw pack and picotool instead`
branch — so there is a worked reference if the ruling goes that way. **Do not
graft it without the ruling**: that arm's line was attached to its own `MTX1`
framing, whose length is not this record's length.

**What the ruling has to settle**, because it is not obvious which way it should
go: `mt` has no `--out` (stdout *is* the artifact, §3b), so it cannot know the
record's destination. A line that says "fits an NFC tag" on a record the operator
is about to flash by picotool is noise on every run of the commoner journey.

**Done when:** the operator rules, and either the line exists with a test
asserting it names 8191 and never 32,734, or this item is closed as declined.

---

## RULING 2026-08-25c (G-P3.10) — two transactions sharing a txid: ENGRAVE BOTH

**Operator ruling, taken during the journey walk 2026-08-25:**

> *"If two transactions in a payload have same txid, we can just engrave both
> without much concern. The odds are low and we can't be responsible for every
> edge case."*

**Consistent with the standard** — reasonable effort funds safety, not
perfection. A deliberate 20-bit collision costs under a second to construct, but
a full 256-bit txid collision does not, and the operator is not the threat model.

**BUT THE CODE DOES NOT DO THIS, AND IS WORSE THAN THE RULING ASSUMES.**
`gui/transaction.go:449`:

```go
if c.confirmed && c.tx.TxidDisplay == tx.TxidDisplay {
    continue next   // merged: the set candidate already carries the bytes
}
```

A `tx:` record whose txid matches an existing chunk-set candidate is **skipped
entirely** — not merged, not flagged, **dropped**. So today two transactions
sharing a txid yield **one** engraveable candidate and the operator is never told
the other existed. That is silent data loss, which is a different and worse thing
than the duplicate-row confusion G-P3.10 was filed about.

**The merge itself is legitimate and must stay:** the SAME transaction delivered
both as `mt1` chunks and as a `tx:` record should appear once.

**WHAT CHANGES: NOTHING. The operator was told the code DROPS rather than
duplicates, and ruled again: leave it that way.**

So the behaviour is: two transactions sharing a full 256-bit txid yield **one**
engraveable candidate, the second is discarded, and nothing says so. My earlier
proposal to key the merge on the bytes instead is **retracted** — it was raised,
heard, and declined.

**The cost, recorded once so it is on file and not relitigated:** this is silent,
not loud. Every other lossy condition in this design reports (an incomplete set,
an orphan, a set that will not decode). This one does not. The operator's
judgement is that a full-txid collision is rare enough, and not the tool's
responsibility, that a silent drop is an acceptable trade at the
reasonable-effort bar. **A future reviewer finding this should read this
paragraph and move on, not re-open it.**

**G-P3.10's premise is retired.** It asked about two identical picker rows. The
rows were never the problem.

---

---

## RULING 2026-08-25d — S0 (P4) is DEFERRED UNTIL AFTER SHIP, and its purpose changes


**P5 M-7 IS LUMPED IN HERE — operator, 2026-08-26: "M7: lump with s0."**

The post-cut instruction tells the operator to *"Scan every QR with a phone,
join the hex, and run `mt inspect`"* (`gui/transaction.go`), and `mt`'s own
`verify_the_steel(RawRecord)` says the same. **The symbols carry raw transaction
BYTES.** Generic phone camera apps render byte-mode content as text, and a raw
transaction contains `0x00` and non-UTF-8 bytes that many apps truncate or
replace — so "join the hex" describes a presentation layer nothing in this cycle
has measured. The ZXing decode gate proved byte-level decodability; it did not
prove that a phone hands those bytes to a human as joinable hex.

**It belongs to S0 because only S0 can settle it**: it needs a cut plate and a
real phone. Until then the instruction is unverified rather than known-wrong —
and S0's first plate is exactly where it gets tested, at no extra cost.

**What S0 must record:** which scanner app, what it produced for a byte-mode
symbol, and whether the hex was joinable as written. If it was not, the
instruction changes and this becomes a defect rather than a gap.
**Operator ruling:**

> *"P4 will be deferred until after ship. The results are essentially already
> known from work outside the repo, but it's still worth doing as probing higher
> density / smaller features isn't well explored."*

**This inverts `FORWARD_PLAN` §4**, which read *"S0 gates the release and nothing
earlier."* It no longer gates it. **P5 may ship with P4 outstanding.**

**AND IT CHANGES WHAT S0 IS FOR.** The acceptance sheet frames G-P4.1–G-P4.6 as
*confirmatory* — "no engraved QR has ever been scanned", "the physics gate has
never run". That framing is true **of this repository** and false of the
operator's knowledge: the results are already known from work outside it. So S0's
remaining value is **exploratory** — probing higher density and smaller features,
which is genuinely unexplored — not proving the design works.

**WHAT SHIPS UNPROVEN-IN-REPO, stated plainly so nobody has to reconstruct it.**
Six gates rest on S0, and after this ruling the release carries all six as
out-of-repo evidence rather than committed artifacts:

| gate | what ships without in-repo proof |
| --- | --- |
| G-P4.1 | no engraved QR has been scanned **in this repo's record** |
| G-P4.2 | the Structured-Append physics gate — the `SA_FIXTURE` pair cut and reassembled off steel |
| G-P4.3 | the legend reservation stays hard-coded; no face below 3.0 mm tested here |
| G-P4.4 | 0.3 mm optically unvalidated (and correctly never emitted) |
| G-P4.5 | the byte encoding is proven only against a software decoder (ZXing) |
| G-P4.6 | the post-cut verify path unwalked |

**This is a legitimate call at the stated standard** — *reasonable effort funds
safety, not perfection* — because the evidence exists, it simply is not in these
files. **The risk is not that the design is unproven; it is that the REPOSITORY
cannot show its work**, which matters to a future maintainer rather than to a
present operator.

**What P5's whole-diff review must therefore be told:** these six are deferred by
ruling, not overlooked, and are **not** grounds for a blocking finding.

**When S0 does run, its first plate is already chosen** (P3a): the reference
transaction defaults to **ECC H at 0.6 mm** — the smaller face, never read off
steel. If that fails to scan, the QR objective's module-size tie-break is wrong.

---

---

## CLOSED 2026-08-25 — G-P3.14 and the NFC-fit line, both by operator ruling

**G-P3.14 — the device review screen shows the txid and nothing else** (no
outputs, amounts, fee, locktime, network). **DISMISSED.** Operator: *"I don't
care."*

Recorded once so it is not re-opened: the txid **commits to every output and
amount**, the wallet displayed those before signing, and fixing this is a
**parser** change on both sides of the language boundary plus on-device address
encoding — not a screen change. The operator's identity check is the txid
comparison, which §4.3 already builds. **Not a gap; a decision.**

**Spec line 450 — `mt encode --qr` stating whether its record fits an NFC
tag.** **NOT BUILT, and it was already settled.** Operator: *"we long ago decided
nfc comes later."*

**And the spec says so itself, at line 142:** *"There is no NFC reader for a
`sysw` container."* NFC arrives by a different path than the flash-XIP route
(`XIPReader`, `sysw/read_tinygo.go`), and §1's table splits the two. So line 450
describes a fit-check for a delivery route this container has no reader for.

**This one is on me:** I raised it twice as an open scope question. It was
answered in the document I had already read, eight lines from a passage I quoted
in a different context. *Look for the existing decision before asking for a new
one.*

---

---

## CONTEXT 2026-08-25 — no real funds have ever passed through this software

**Operator, stated during the pre-ship walk planning:**

> *"Nobody but me uses the software and it's never been used for real funds at
> all yet (across the entire repo)."*

**Recorded because it relocates the risk, and every funds-safety argument in
these documents reads differently once you know it.**

**What it makes CHEAPER.** Shipping. There are no users to break, no deployed
versions to stay compatible with, and no funds currently depending on any
guarantee here. A tag is a checkpoint, not a commitment. The `ci/staging` ritual
and the whole-diff review are worth doing for the codebase's sake, not because
something is at stake in the release itself.

**What it makes MORE serious, and this is the real gate.** **The first
real-funds engraving will be the first time any of this touches something that
matters.** Every guarantee in the acceptance sheet — the signature predicate, the
txid binding, the anti-smuggling decode, the QR round-trip — is currently
supported by synthetic evidence only: corpus vectors, a software decoder, and an
emulator. All of it is *good* evidence. None of it is *consequential* evidence.

**So the release is not the moment to be careful about; the first real spend is.**
That is the operator's, it is deliberately not scheduled here, and the honest
statement of readiness is: **the software is ready to be tried, not proven in
use.**

**The side-by-side walk is the closest rehearsal available** without real funds,
which is why it happens BEFORE the tag rather than after.

---

### F-248 — `mt encode` refuses its own output without recognising it (owning phase: **post-ship polish**) `#mt` `#ux`

**DONE 2026-08-25** — `mnemonic-transaction` `24b8cef`. Two forms: `mt1` strings (with an exact count) and the `tx:` record, neither echoed.

**Found in the side-by-side walk, 2026-08-25.** The operator ran `mt encode`,
saw 22 `mt1` strings, decided to use the SeedHammer instead, re-ran for the
record form and **pasted the strings back in** — a plausible mistake, because
they were the last thing on screen. (The walk typed `--record --chunks`. That
spelling was retired hours later — it was a no-op — and the defect is unchanged:
bare `mt encode` reproduces it.)

```
mt encode: REFUSED — §8.2e, input is not a PSBT or a raw transaction (1978 bytes)
```

**The refusal is correct and names the wrong thing.** `mt` has `ValidMT`: it can
recognise an `mt1` string on sight, and it is looking at 22 of them. It knows the
operator pasted its own output and reports a byte count.

**What it could say:** *"that input is 22 `mt1` strings — my own output. To check
them use `mt verify`; to turn them back into a transaction use `mt decode`; to
re-encode, paste the PSBT or raw transaction instead."*

**Same shape as R0 round 3's `Unrecognised` defect:** a refusal that is true and
names the wrong thing, on a tool that already holds the information needed to
name the right one.

---

### F-249 — WITHDRAWN 2026-08-25 — FALSE AS FILED; the message names three remedies (owning phase: **none — closed**) `#mt` `#ux` `#f-244`

**WITHDRAWN before implementation, during the overnight burndown.** The claim
was **false**, and the way it became false is the finding worth keeping.

**What the entry asserted:** that the refusal explains the permission and not the
remedy. **What `mt` actually prints**, captured unfiltered:

```
$ mt encode < tx.hex > records.txt
mt encode: REFUSED — §8.2h, stdout is a file of mode 0644 — readable by other
users on this machine.

  These strings ARE the engraving, and a finalized transaction is
  BEARER: anyone who can read that file can broadcast it.

  mt has no --out: stdout IS the strings, by design (§3b). So the
  remedies are the shell's:

  umask 077 then re-run; the shell creates it 0600
  chmod 600 <file> then re-run -- `>` truncates but keeps the mode
  --allow-world-readable proceed anyway
```

**All three remedies are there, and `validate.rs`'s `with_remedy(...)` has carried
them since F-244 closed.**

**HOW THE FALSE FINDING WAS MANUFACTURED.** The walk captured that refusal through
`grep -iE 'REFUSED|error|mt encode:'`. The grep matched the one line beginning
`mt encode: REFUSED` and discarded the indented remedy block beneath it — so the
transcript pasted into this entry was an artifact of the filter, not of the tool.
**The observation instrument produced the defect.** This is the fourth time in one
session that reading a stream through a pipe gave a wrong answer here (a `tail`
exit code instead of `mt`'s, a `grep` that made stdout a pipe so the world-readable
guard could not fire, an exit code from `tail` instead of `picotool`, and this).
**Capture whole, then filter the capture — never filter the capture.**

**Nothing to implement.** The guard is right, the message is right, and the
original text below is kept only for what it says about the author tripping it.

A pipe works; `>` does not. **The guard is right** — F-244 exists because a
finalized transaction is bearer material and `>` creates a world-readable file
under the default umask. Nothing here argues for weakening it.

**WHAT SURVIVES THE WITHDRAWAL.** The author tripped this refusal three times in
one session *despite* the remedies being printed every time — having fixed F-244,
written its follow-up, and quoted §8.2h earlier the same day. That is worth
keeping, because it is a different claim from the one filed: not *"the tool does
not say"* but *"the operator does not read a wall of correct text at the moment
they are mid-task."* Any future work here is about **prominence**, not content,
and it should start by measuring whether the remedy block is even seen — the
entry that follows the digest in F-246 is the same shape.

**One thing genuinely absent, and still not chosen:** `mt encode` has no `--out`,
so every remedy offered is the shell's rather than the tool's. `me sysw pack` has
`--out` and creates it 0600. Adding one to `mt` remains a candidate; it was not
adopted here because §3b rules that stdout IS the artifact.

---

### F-250 — `mt encode -` is rejected as an unexpected argument (owning phase: **post-ship polish**) `#mt` `#ux`

**DONE 2026-08-25** — `mnemonic-transaction` `5c7d827`. A hidden positional whose `value_parser` admits only the literal `-`.

**Found in the side-by-side walk, 2026-08-25.** The operator typed the ordinary
shell idiom:

```
$ cat file.psbt | mt encode -
error: unexpected argument '-' found
Usage: mt encode [OPTIONS]
```

`-` meaning *read stdin* is honoured by `cat`, `tar`, `curl`, `gpg`, `jq` and
most of the tools an operator has in muscle memory. `mt` reads stdin **by
default**, so the intent was already satisfied — the command failed for asking
politely.

The message is clap's generic one. The usage line implies the answer by showing
`[OPTIONS]` with no positional, which is a deduction rather than an answer. It
never says **"stdin is already the default — drop the `-`."**

**Trivially fixable** — accept an optional positional `-` and ignore it, or catch
it and print the one sentence. Not chosen here.

**THE PATTERN, and it is the real finding — CORRECTED 2026-08-25 when F-249 was
withdrawn.** F-248, F-250 and F-251 came from one walk, and they are one defect
wearing three faces: **every refusal on this path is correct, and none of them
says what to do instead.** Pasting your own output gets a byte count; using a
standard idiom gets a parser error; asking `me` for help gets a menu with no word
for the job. The tool knows the answer in each case — it holds `ValidMT`, it knows
stdin is its default, it holds `Class::Tx` — and says none of them.

**F-249 was originally counted as a fourth face and is NOT one.** Its refusal
names three remedies and always has; the entry was written from a `grep`-filtered
transcript. It is left in place as WITHDRAWN rather than deleted, because how a
filtered capture manufactured a finding is worth more than the finding was.

**Worth fixing as a class rather than three tickets**, and worth remembering that
a correctness lens finds none of them: every one of these commands does exactly
what the specification says.
---

### F-251 — `me`'s help tree never names the operator's goal, and the one sentence that does is unreachable (owning phase: **post-ship polish**) `#me` `#ux`

**DONE 2026-08-25** — `mnemonic-engrave` `6c3289b`. The one-liner lives in `Cargo.toml`; clap renders the first LINE, not the first paragraph.

**Found in the side-by-side walk, 2026-08-25**, at the step after the `--qr`
collapse landed. The operator had a `tx:` record in hand and typed `me -h`
"because I want to start engraving a QR coded tx".

**Measured across `me`'s help tree**, not read:

| what the operator types | says "transaction" | says "QR" |
| --- | --- | --- |
| `me -h` | 0 | 0 |
| `me sysw help` | 0 | 0 |
| `me sysw pack -h` | 1 | **0** |
| `me sysw pack --help` | 4 | **1** ← the answer |

The sentence that answers the question exists and is good:

> ``tx:<hex of the raw signed transaction>`` feeds the device's QR engraving
> path — produce it with ``mt encode --qr``, which checks the bytes parse AND
> that every input carries a signature.

It is in **paragraph 3** of `pack`'s doc comment, so clap renders it only under
`--help` (73 lines) and never under `-h` (19 lines). The operator typed `-h`
twice and never saw it.

**Silence is not the worst of it.** The top-level screen actively advertises the
NFC converter — `--in`, `--out`, `--stdout`, `--hex`, `--base64` — which is the
one path that does not apply: SPEC **§1.2** line 142 records that there is **no
NFC reader for a `sysw` container**, and NFC is ruled later work. An operator who
follows what is on screen feeds the record to the converter and gets:

```
$ me --in rec.txt --hex
me: unrecognized HRP 'tx:0' (expected md, mk, ms, or mt)     (exit 4)
```

Correct, and a bech32 parse complaint about a thing `me` can identify perfectly
well — `tx:` is `Class::Tx` in its own `sysw` code. **This is F-248/249/250's
defect wearing a fourth face:** the tool knows the answer and reports the
mechanism instead. Being told *nothing* at the top level would be better than
what the menu currently implies, which is the test a journey divergence has to
pass to earn a change.

**Two smaller facts from the same measurement:**

- `me`'s one-liner says it converts `(md1/mk1)`. It accepts `mt1` too —
  verified by feeding one string through the bare converter and getting NDEF
  bytes back. The description is stale with respect to a capability this cycle
  added.
- `sysw`'s summary — *"Build, inspect or overwrite a SYSTEMWIDE payload"* —
  never connects to records, plates, or engraving. `SYSTEMWIDE payload` is the
  container's name, not the operator's goal.

**The fix is three doc comments and no behaviour:** name transactions in `me`'s
one-liner, say what `sysw` is *for* in its summary, and move the `tx:`/QR
sentence into `pack`'s **first** paragraph so it survives `-h`. Clap's
short/long split is a fine convention and is not the defect — putting the
load-bearing sentence outside the form everyone types is.

**NOT a defect, and deliberately kept:** the HRP message lists `ms` among
expected prefixes although `me` refuses `ms1`. Parsing it is what earns the real
refusal — *"ms1 is secret seed entropy and must never be transmitted by radio"* —
instead of a generic one. Verified by feeding an `ms1` string.
---

### F-252 — the world-readable refusal asserts reachability it never checked, and is FALSE under any 0700 ancestor (owning phase: **post-ship polish**) `#me` `#mt` `#ux`

**DONE 2026-08-25** — `mnemonic-transaction` `54c6d54` and `mnemonic-engrave` `86854c6`. The sentence changed in both; the guard did not.

**Found in the side-by-side walk, 2026-08-25**, when the operator read the
refusal that had just blocked their `me sysw pack` and asked plainly: *"Is it
true that stdout is readable systemwide?"*

**It is not, in the commonest case there is.** Both binaries say:

> stdout is a world-readable file … **readable by other users on this machine**

The guard behind it (`stdout_is_world_readable`, `me-cli/src/main.rs:871`) does
`fstat(1)` and tests `mode & 0o044`. **It inspects the file's own mode and
nothing else.** POSIX requires search (`x`) permission on *every* directory in a
path, so a 0644 file beneath a 0700 directory cannot be opened by anyone else.
Measured on this box:

```
file mode:                       0644
first dir denying other-search:  /tmp/tmp.jM9DwOet55   (mktemp -d is 0700)
actually reachable by others?    False        <- and yet: refused, exit 2
```

`$HOME` here is **0700**, so *every* file the operator creates under their home
directory is in exactly this state. The refusal that stopped the walk described a
file no other user could open.

**A truthful check is implementable — mechanism verified, not assumed.**
`readlink /proc/self/fd/1` resolves stdout to a real path, and walking its
ancestors for `S_IXOTH` returns the right answer. Cost: it is Linux-only
(`/proc`); macOS needs `fcntl(F_GETPATH)`. That is a real portability question
and the reason this is filed rather than fixed in passing.

**DO NOT DROP THE BLOCK.** A 0644 file is a latent hazard even while currently
unreachable: move it to a 0755 directory (`/scratch` on this box), tar it, back
it up, or `chmod` the parent, and it becomes readable with no further warning.
For bearer material that defence is worth keeping, and this entry exists so a
future reader does not "fix" the false sentence by deleting the guard.

**What to change is the SENTENCE.** State what was measured — *"stdout is a file
with mode 0644 (group/other-readable)"* — instead of asserting reachability the
guard never established. Crying wolf in the majority case is precisely what
trains an operator to reach for `--allow-world-readable` reflexively, which is
the aliased-away failure mode the retired R3 rule was built around; see
`SPEC_engrave_transaction.md` §2.2.

**Both repos.** `mt`'s §8.2h refusal carries the same sentence, and per the
Rust-primary rule the wording should move together — the FALSE sentence is the
same in both, so this is one edit applied twice, not a port. *(An earlier version
of this paragraph leaned on F-249 to claim `mt`'s refusal offers no remedy. F-249
is WITHDRAWN as false-as-filed: both binaries already name three remedies.)*



---

## RULING 2026-08-25 — an INCOMPLETE chunk set REPORTS LOUDLY and PACKS; and P1's refuse posture diverged from its own container spec

**Operator ruling, taken 2026-08-25.** Asked whether an incomplete `mt1` chunk
set should be REFUSED (the P1 plan's E20) or reported-and-packed (the
`md1`/`mk1` sibling's posture), the operator ruled: **"Report loudly and pack."**

**The question arose from an operator correction.** A fable simplification report
justified P1's refuse posture on the grounds that *"a `tx:` payload is
regenerable, an `md1` card may be the only copy"*. **The operator corrected this:
a `tx:` payload is NOT necessarily regenerable.** The journey pipes
`tx.final.psbt` — a FINALIZED transaction, embodying a completed signing
ceremony. A multisig quorum collected across time and geography may take days to
re-collect or be impossible if a cosigner is unavailable; the source PSBT may be
gone; and **re-signing is not idempotent under this design** — a fresh nonce
yields a different signature and therefore a different **wtxid**, which this plan
carries (E17) and binds on (R15's top-20 of the txid). A regenerated payload is a
different artifact by the design's own identifiers.

**That correction is what makes report-and-pack right.** Every `mt1` chunk is
independently valid, BCH-protected, and carries its own index and count, so a
partial set is self-describing about what is missing: 201 engraved chunks plus
the 202nd recovered later reassembles. That is exactly how `md1`/`mk1` multi-card
backups already work. Refusing means the operator engraves NOTHING and may lose
the ceremony.

**AND THE PLAN WAS DIVERGING FROM ITS OWN CONTAINER'S SPEC, WHICH NOBODY NOTICED
IN EIGHT ROUNDS.** `design/SPEC_systemwide_payloads.md:587` —

> **5.3.2 The card-set DECODE check — now a FLAG, not a refusal (R0-I1; demoted
> 2026-08-12, §13 D6)**

EPD §6.3's per-card-set decode requirement *"reaches this container as a **flag
input**, not as the refusal EPD gives it"*. So report-and-pack is not a
divergence from the sibling at all — **it is what the container spec already
requires, and P1's refuse posture was the divergence**, from the normative
document it is building inside.

**"Loudly" is normative and means more than the sibling does.** `mdmk_unconfirmed`
reports quietly — it returns indices. P1's incomplete-set report MUST:
- emit a **stderr warning at pack time** naming the set and **every** missing
  index, not the first (r7-M1);
- be visible in `me sysw show`, marked INCOMPLETE with the missing indices;
- carry no format change — the chunks' own `count`/`index` let any reader
  recompute it, so P4's device display can too.

**Still OPEN and NOT ruled here:** whether `not_a_transaction` (W15/V28 — a
COMPLETE set that reassembles to bytes that are not a transaction, the §2.1 C3
smuggling channel) also demotes to a flag under §5.3.2, or stays a refusal. The
two cases are different: incomplete is *missing material*, non-decoding is
*wrong material*. **P1 may not close while this is open.**

---

## RULING 2026-08-25b — `not_a_transaction` is a LOUD FLAG ON THE DEVICE with MANDATORY LEGEND SUBSTITUTION. Nothing refuses.

**Operator ruling.** Asked whether `not_a_transaction` (a COMPLETE chunk set that
reassembles to bytes that are not a transaction — the §2.1 C3 smuggling channel)
stays a refusal or demotes to a flag like the incomplete case, the operator ruled:

> *"Becomes a loud flag on sh2, mandates legend substitution: drop user desired
> legend and replace with 'incomplete, re-encode payload' or something to that
> effect but don't refuse engraving."*

**So NOTHING in the chunk path refuses.** Incomplete sets and non-decoding sets
both pack, both reach the device, and both engrave — with the operator's chosen
legend **replaced** by a warning.

**Why this is a better control than a refusal, and it is not a weakening.** A
stderr line at pack time is gone in a week; `me sysw show` must be re-run to be
seen. **An engraved legend is permanent and travels with the artifact** — and the
device has NO CAMERA (`sh2-has-no-camera`), so it can never read a plate back to
warn anyone later. Substituting the legend converts an ephemeral warning into a
durable one, on the only surface that outlives the session. For a
fifteen-year-recovery product that is the right place to put it.

**THE DEVICE MUST COMPUTE THIS ITSELF — verified precedent, not inference.** The
fork already does exactly this for the siblings, on-device:

```
seedhammer/sysw/confirm.go:81       _, err := md.Reassemble(set)
seedhammer/seal/record.go:475       decodePublicSet enforces §6.3: every public
                                    record belongs to a card set that decodes
seedhammer/seal/record.go:231       public | md1/mk1 only, AND every card group
                                    must reassemble and decode
```

**A host-set flag byte would be worthless for this**, and the reason is in EPD's
own threat statement: the adversary is *"a defective or third-party sealer"*. A
sealer that smuggles will also set the flag to "fine". The check is only worth
anything where it cannot be forged, which is on the device.

**THE COST, and it lands on P3, not P1.** The fork has **no `mt` package and no
Bitcoin transaction deserialiser** (packages verified: `md`, `mk`, `codex32`,
`bip32/39/85/380`, `slip39`, `seedxor`, `address`, `seal`, `sysw`, …). To flag
`not_a_transaction` on-device, **P3 must port `mt` reassembly AND a transaction
deserialiser to TinyGo.** Incomplete is far cheaper — the chunk header carries
`count` and `index`, so only the header parser is needed.

**TWO CONSEQUENCES THE OPERATOR SHOULD HAVE IN WRITING.**

1. **The anti-smuggling gate stops being admission control and becomes LABELLING
   control.** Under this ruling the smuggled bytes still reach metal — 32 bytes of
   entropy engraved on a plate, under a legend that says the payload is bad. The
   flash exposure EPD measured is unchanged either way (the payload is in flash
   once loaded); the plate is *additional* exposure the refusal would have
   prevented. The operator is warned and chooses. **This is a deliberate trade,
   not an oversight** — recorded here so no future reviewer "fixes" it back.
2. **Substitution is sometimes INSERTION.** The legend field is OPTIONAL. When a
   payload carries no legend, the device must **add** one, which consumes plate
   space that was not budgeted. P4 owns the layout consequence; P1 owes only that
   the field's absence is representable and detectable, which it already is.

**Un-overridable.** The substituted legend MUST NOT be dismissible back to the
operator's own text — a warning the operator can turn off is not a control.
---

### F-253 — a bare `me sysw pack` writes the BEARER container to the terminal at exit 0, under an exemption justified by a false claim (owning phase: **post-ship polish**) `#me` `#security` `#ux`

**DONE 2026-08-25** — `mnemonic-engrave` `9ef69ee`. Both gates now live in one pure `write_block()`; a pipe is unaffected.

**Found in the side-by-side walk, 2026-08-25**, immediately after F-252 and from
the same question. The operator typed the plain command and pasted one `tx:`
record — no redirect, no `--out`:

```
$ me sysw pack                      (paste, Ctrl-D)
sealing:  NOT SEALED — ...
digest:   7981 04fa 8223 f3fc 8839 6701 2f0b 5a8e
          re-print it with: me sysw show <the file you just wrote>
MNEMSYSW^A^@^@^@ ... 1,743 bytes of raw container ...
exit 0
```

**Nothing failed, so nothing contradicts the false line.** F-246's second
instance records *"the file you just wrote"* printing when a guard aborts the
write; here it prints on a **successful** run that also wrote no file, so there
is no error beneath it to give the game away.

**THE EXEMPTION RESTS ON A FALSE PREMISE.** `stdout_is_world_readable`
(`me-cli/src/main.rs:871`) returns `false` for any character device, and its own
comment justifies that:

> *"A terminal and `/dev/null` persist nothing, so neither can leak."*

The `/dev/null` half is correct and the exemption is genuinely load-bearing for
it — `/dev/null` is mode 0666, so a mode-only test would refuse
`me … > /dev/null`, which the comment rightly calls one of the most ordinary
things anyone does with a CLI. **The terminal half is false.** A terminal
persists in scrollback, and terminal sessions are routinely logged — this very
finding was captured through `script`. For BEARER material that is exposure, not
absence of it.

**ONE CAUSE, TWO OPPOSITE FAILURES.** With F-252, the guard is wrong in both
directions because it inspects the file's own mode instead of where the bytes
actually come to rest:

| situation | reality | `me` does |
| --- | --- | --- |
| 0644 file under a 0700 ancestor | unreachable by others | **refuses**, exit 2 (F-252) |
| a terminal | scrollback, session logs | **writes it**, exit 0 (this entry) |

**WHAT WOULD CLOSE IT — the operator's proposal, 2026-08-25, and it has
precedent in this binary.** Rather than dumping the blob or bare-refusing, print
the command that writes it. **`me seal` already does this** (`run_seal_cli`,
`me-cli/src/main.rs:633`):

```
load:  picotool load --verify <file>   (machine in BOOTSEL)
wipe:  picotool erase -r 0x10E00000 0x10E10000
```

`me sysw pack` prints **zero** picotool hints — measured. So the fix is not a new
invention, it is making `pack` do what its sibling verb already does, with `sysw`'s
own region address (`0x10D00000`, `--region`) instead of `seal`'s `0x10E00000`.

Mechanically: when stdout is a TTY and no `--out` was given, emit the `--out` +
`picotool` sequence instead of the container. That closes the scrollback exposure
**and** supplies the route, which is the through-line of F-246/248/249/250/251.
`IsTerminal` is already used in this binary at `main.rs:1740` for the stdin
banner, so the mechanism is present today.

**PIPING STRAIGHT INTO `picotool` IS NOT THE ANSWER — SETTLED ON HARDWARE
2026-08-25**, with the operator's SH2 in BOOTSEL (`2e8a:000f RP2350 Boot`).
Tested with `picotool verify`, which is READ-ONLY on the device — `load` was
never run, because it writes and the machine carries the burned OTP key.

| stdout | `st_size` | picotool does |
| --- | --- | --- |
| regular file | 4096 | reads it, reports the flash mismatch |
| `/dev/stdin` ← a file | 4096 | **identical** to naming the file |
| `/dev/stdin` ← a pipe | **0** | *"No ranges to verify"* |

**picotool sizes the file with `fstat`, and a pipe reports `st_size` 0.** So a
pipe does not fail — it silently reads NOTHING. `picotool load -` is separately
rejected at argument parsing ("unexpected option: -").

**THE DANGER IS THE SILENCE — AND `load` WAS THEN TESTED DIRECTLY**, after the
operator authorised writes to `0x10D00000` (the payload region; firmware was
never a target). Piping 4,096 bytes of `0xAA` into
`picotool load /dev/stdin -t bin -o 0x10D00000`:

| stdout source | picotool exit | progress bar | flash |
| --- | --- | --- | --- |
| a pipe | **0** | frozen at `0%` | **byte-identical afterwards — nothing written** |
| a regular file, same bytes | 0 | runs to completion | written (`aa aa aa …`) |

Proved by reading the region back with `picotool save` before and after: after
the piped load the region still began `MNEMSYSW`; after the file load it began
`aa aa`. **A flashing command that reports success and writes nothing**, with the
only distinguishing signal a progress bar that does not move. An operator
running `me sysw pack | picotool load /dev/stdin` would have every reason to
believe the payload was on the device.

**A measurement trap worth keeping.** The first exit code read here was `tail`'s,
not picotool's, because the command was piped into `tail` — the same defect this
constellation has recorded before. The `0` above comes from
`sh -c 'picotool …; echo PICOTOOL_EXIT=$?'`, with the flash readback as the real
evidence rather than any exit code.

**Two controls were run, and the first one mattered.** With NO device attached,
a nonexistent path and a real path return the same "No accessible RP-series
devices" message — picotool checks for a device *before* opening the file, so
the device gate absorbs the whole test. The hardware run is what made file-level
errors observable at all ("Could not open '/nonexistent/nope.bin'").

It is moot regardless: a `0x10D00000` write wants `--region`, which pads to
`REGION_LEN` with `0xFF` and is file-shaped by design.

**Do not close it by refusing all char devices** — that is the `/dev/null` case
the existing comment protects (mode 0666), and there are tests for it.

**Check `mt` for the same shape.** Its §8.2h guard shares this design and the
Rust-primary rule applies; `mt encode` writes text rather than a binary blob, so
the terminal case is far less alarming there, but the exemption's reasoning is
the same and the comment may carry the same sentence.
---

## RULING 2026-08-26a (P5 I-2) — duplicate chunk indices: LAST WINS, and the device MUST SAY SO

**Operator ruling, taken while folding the P5 whole-diff review:**

> *"P5 I-2 last wins is fine but message to user that this is the rule is
> required so they can try again in different order. Message should say
> something like duplicate plate 13 of 20 found, last wins"*

**What the finding was.** `orderByIndex` (`gui/transaction.go`) builds TEXT-plate
content with `byIdx[h.ChunkIndex] = s`. When one `chunk_set_id` group holds two
DIFFERENT strings for the same index, all but the last are silently dropped from
the steel. `mt.Decode` correctly detects the ambiguity and the set is offered
unconfirmed under a substituted legend — but the review screen shows the deduped
count and nothing names what was dropped.

**No low-odds event is required.** Sign the same PSBT twice: a fresh nonce gives
different witness bytes but an **identical txid**, therefore an identical
`chunk_set_id` by construction. Pack strings from both runs interleaved and the
engraved last-wins mix can later decode and **CONFIRM** — the txid ignores
witness bytes — while its spliced signature is invalid and the transaction can
never be broadcast, under a permanent legend reading *"DOES NOT DECODE"*.

**The ruling keeps the behaviour and adds the disclosure.** Last-wins stays. The
device must name **which** plate duplicated and **out of how many**, so the
operator can re-order the payload and re-run. The operator's own example of the
shape: `duplicate plate 13 of 20 found, last wins`.

**Distinguished from 2026-08-25c**, which retired the txid-keyed *merge* drop on
"the odds are low" grounds. This site is not that: the collision is guaranteed
rather than unlikely, the ambiguity is *detected* and then discarded, and silence
here breaks the loudness ruling 2026-08-25b made normative.

**Implementation constraints carried into the brief**, because getting them wrong
is the whole failure mode: the number shown must be the **plate number the
operator sees** (1-indexed if the surrounding UI is), the total must be the set's
**declared** count from the header, **every** duplicated index must be named and
not just the first, and **no string body may be rendered** — `mt1` strings are
bearer.

---

## OPEN — awaiting an operator ruling: SECRET material on argv

**Not a ruling. A question raised 2026-08-26 while folding P5 I-1, recorded so it
is not lost.**

P5 I-1 was that `me sysw pack` refused a `tx:` record on argv while accepting the
same transaction as `mt1` strings. That is **fixed** (`90c1a04`): the gate is now
keyed on `Class::is_bearer()`.

**Fixing it surfaced a larger hole.** The first attempt refused
`is_secret() || is_bearer()` and broke 15 tests, because `me sysw pack` accepts,
**at exit 0 in silence, on argv**:

| what | result |
| --- | --- |
| a raw BIP-39 mnemonic | 145-byte payload |
| an `ms1` string (seed entropy) | 102-byte payload |
| a `pass:` record | 113-byte payload |

So `me` refuses a *transaction* on argv and accepts a *seed phrase* — the same
inverted gradient `SPEC_constellation_cli_uniformity.md` §1 found in `ms`, inside
`me` itself.

**Not fixed unilaterally**, because it is the same decision as that spec's D3
(refuse, with an override), it breaks 13 shipped invocations in this repo's own
tests, and it is a ruling rather than a fold. The gap is **pinned** by
`argv_still_accepts_secret_classes_which_is_the_open_gap`, which asserts the
current unsafe shape so the change is deliberate: when the ruling lands, that
test flips and its failure is the reminder. The one-line widening is named in a
comment at the gate.

**Answering D3 probably settles both.**


### F-254 — the installed `me` is four minor versions stale (0.3.0 vs 0.7.0), so a bare `me` in the operator's shell is not the `me` under review (owning phase: **post-ship polish**) `#me` `#repro`

**DONE 2026-08-26**, operator authorised ("You can update local binaries").
`cargo install --path crates/me-cli --force --locked` → *Replaced package
`mnemonic-engrave v0.3.0` with `v0.7.0`*. `mt` was not installed at all and is
now `mt-cli v0.1.0`. All six verified in sync against their repo builds:
md 0.13.0, mk 0.13.0, ms 0.16.0, mt 0.1.0, me 0.7.0, mnemonic 0.97.0.

**Found 2026-08-26** while building the tier-placement recon, by measuring rather
than assuming. `~/.cargo/bin/me --version` → **me 0.3.0**, dated **Jun 16**;
`mnemonic-engrave/target/debug/me --version` → **me 0.7.0**. The other four
binaries are in sync (`md` 0.13.0, `mk` 0.13.0, `ms` 0.16.0, `mnemonic` 0.97.0
all identical between installed and repo builds), so this is `me` alone.

**The walk was NOT contaminated, and that is provable rather than assumed.** Two
independent checks:

- `~/.cargo/bin/me sysw --help` → **exit 2**, `error: unrecognized subcommand
  'sysw'`. 0.3.0 has verbs `bundle help`; 0.7.0 has `bundle hash help seal sysw`.
  Every `me sysw` observation in the walk is therefore impossible on 0.3.0.
- The `-h` / `--help` divergence that the operator found exists **only** in
  0.7.0: 0.3.0 gives 17 lines to both and they are byte-identical; 0.7.0 gives
  **21 vs 42**.

**Fix:** `cargo install --path crates/me-cli --force`. **The reason to file it
rather than just run it** is that the drift is silent and re-accumulates — an
installed binary keeps answering, confidently, as a version nobody is reviewing.

### F-255 — `md` collides with a near-universal `mkdir -p` alias, and the collision fails as SILENT SUCCESS (owning phase: **constellation naming**, tier-placement cycle) `#md` `#constellation` `#ux`

**Found 2026-08-26**, same sweep. In the operator's own shell, `type md` →
**`md is an alias for mkdir -p`**, shadowing the real binary at
`~/.cargo/bin/md`. Demonstrated, not reasoned:

```
$ md repair md1yqpqqzqq8xtwhw4xwn4qh
  exit=0
  dirs created:  md1yqpqqzqq8xtwhw4xwn4qh/  repair/
```

**Exit 0, two directories, no output.** The verb and the argument each became a
directory name. A user checking `$?` sees success.

**Why this is a constellation finding and not a dotfile complaint.** `md` for
`mkdir -p` is one of the most common shell aliases in existence, so this is the
default environment for a large share of users, not an idiosyncrasy. Of the six
names, `md` is the only one that collides — `mk`, `ms`, `mt`, `me` and
`mnemonic` all resolve to their binaries here. And the failure mode is the worst
available: not "command not found", but **success with a side effect**.

**This is why measurements in this cycle were taken by path.** Re-verified by
path at filing time: `descriptor-mnemonic/target/debug/md repair <vector>` → **5**,
`mnemonic-toolkit/target/debug/mnemonic repair <vector>` → **4** — the D26 pair
the spec rests on, unchanged. Had those been taken by bare name they would both
have read **0** and the whole D26 analysis would have been built on `mkdir`.

**Ruling owed from the operator**, since renaming a shipped binary is not a
change to make unilaterally. The options are to leave it and document, to ship a
longer canonical name with `md` as an opt-in shim, or to detect the shadow at
install time.

### F-256 — the constellation's working set is **1.8T**, and ~1.83T of it is `target/`; ten linked worktrees are still registered (owning phase: **housekeeping**, operator ruling owed) `#housekeeping`

**Filed 2026-08-26** at the operator's direction ("We probably need to clean up
extraneous trees eventually too"). Measured rather than estimated, because the
answer was two orders of magnitude off what "extraneous trees" implies.

**The trees are the small half.** Unregistered scratch directories total
**~6.5G**:

| dir | size |
| --- | --- |
| `_experiment/` (A and B, three repos each) | 5.0G |
| `mt-size-probe/` | 1.4G |
| `me-review-scratch/` | 36M |
| `_work/` (**ACTIVE** — holds the running review worktree) | 26M |
| `wt-s5-skeptic-copy/` | 18M |
| `seedhammer-ref-v1.4.2/` | 3.1M |
| `mk-v010-cross-update/` | 868K |
| `me-impl-scratch/` | 32K |

**Ten linked worktrees are registered** across five repos — `mnemonic-engrave` 3,
`mnemonic-transaction` 2, `seedhammer` 2, `mnemonic-toolkit` 2, `mnemonic-key` 1.
One of the three on `mnemonic-engrave` is live and must not be touched.

**`target/` is the real number.**

| repo | `target/` | `.claude/worktrees/` |
| --- | --- | --- |
| `mnemonic-gui` | **1.1T** | — |
| `mnemonic-toolkit` | **623G** | 19G |
| `descriptor-mnemonic` | 41G | 4.0K |
| `mnemonic-secret` | 37G | 0 |
| `mnemonic-key` | 22G | 0 |
| `mnemonic-engrave` | 11G | 0 |

`/scratch` is 9.4T at **52% used**, 4.6T free — so this is not urgent, and that
is exactly why it has gone unnoticed. Rust `target/` directories never
garbage-collect; they accumulate one incremental artifact set per toolchain, per
profile, per feature combination, indefinitely.

**Nothing deleted, and deliberately so.** Two reasons this needs a ruling rather
than a `cargo clean`: `mnemonic-gui` is the 1.1T outlier and is explicitly out of
scope for the current cycle, and reclaiming it converts disk into rebuild time on
whatever is next touched. The safe subset — the `_experiment/` pair, the size
probe, and the finished scratch copies — is ~6.5G and needs its branches checked
for uncommitted work first.

### F-257 — `plan-glyph-check.sh` is red on a CLI spec it was never built for (owning phase: **tooling**, before the next spec fold) `#tooling` `#gates`

**Filed 2026-08-26** during the verification fold. The gate exits 1 on
`SPEC_constellation_cli_uniformity.md` at lines 699–700, and did so at `d31beed`
too — it is **not** introduced by any recent fold.

Both hits are inside a blockquote of **CLI exit-code help text**:

```
> ... 0 — every input was already valid (no corrections applied) 5 — at least
> one chunk had corrections applied (REPAIR_APPLIED) 2 — atomic-fail […]: ...
```

The em dash and the ellipsis are flagged because the **SeedHammer II display
font** does not carry them. That is the gate's stated purpose and it is correct
about the font — but this text is **terminal output**, where both render fine.
No string in this spec reaches the device.

**Do not "fix" this by rewriting the prose.** The characters are right for the
medium; the gate is out of domain. Correcting the text to satisfy it would make
the document worse to satisfy a check that does not apply.

**Why it matters enough to file.** A gate red for non-defects trains a reader to
ignore it exactly as fast as a gate green for everything — and this one is
currently red on every run, which means the next genuine undrawable string it
catches will be indistinguishable from the noise it already prints.

**The fix is scope, not suppression:** the gate should take the artifact class as
input, or skip documents that declare themselves CLI-only, so that a red result
means something again. Whatever the mechanism, it must keep printing what it does
**not** cover — a gate that hides its blind spot is worse than no gate.

**F-256 UPDATE 2026-08-26 — DONE, and the original figures were WRONG by ~3×.**

Operator authorised the cleanup ("Can you trim the chaff?"). Reclaimed:

| repo | logical removed | |
| --- | --- | --- |
| `mnemonic-gui` | 1.0 TiB | 485,444 files |
| `mnemonic-toolkit` | 621.1 GiB | 1,287,799 files |
| `descriptor-mnemonic` | 41.8 GiB | 83,607 files |
| `mnemonic-secret` | 36.8 GiB | 118,283 files |
| `mnemonic-key` | 22.5 GiB | 65,754 files |

**~1.72 TiB logical — but only 645 GB of real disk.** `/scratch` went from 4.8T
used (52%) to 4.2T used (45%).

**The correction matters more than the cleanup.** `/scratch` is **btrfs mounted
`compress=zstd:3`**. `du` reports *logical* bytes; `df` reports *allocated
blocks*. Every figure in the original F-256 — the 1.8T working set, the 1.1T and
648G headline repos — came from `du` and therefore overstated real consumption by
roughly **3×**. The true working set was closer to 600 GB.

The discrepancy was only visible because `cargo clean` printed its own total
(1.0TiB) next to a `df` delta (347 GB) that disagreed with it. **Two measurements
of the same action, disagreeing, is what exposed it** — one number alone would
have shipped the error, and did, for a full day.

Hardlinks were ruled out as the cause before compression was confirmed: `du` and
`du -l` agree to within 1 GiB on the same tree.

**Not done, and NOT a disk question:** `mnemonic-toolkit` holds **38 untracked
files** and `mnemonic-key` **1**, all design markdown (`cycle-prep-recon-*.md`,
`SPEC_chunk_set_id_verification.md`). `cargo clean` never touches source, so
these are unaffected — but they are uncommitted work sitting in a tree nobody is
watching, which is a different exposure from the one this item was filed about.

### F-258 — the `mnemonic-io-lib` extraction is **11 functions / 431 lines**, not the 3 the spec names (owning phase: **P0**, prep for the plan) `#P0` `#mnemonic-io-lib`

**Measured 2026-08-26** while preparing the P0 plan, before writing it. §5a names
three donated pieces — `write_private`, `is_argv_forbidden`,
`stdout_world_readable_mode`. The real closure is larger, and the plan needs the
inventory rather than the description.

**Where the pieces live today.** `me-cli` is the only crate in the constellation
with both `lib.rs` and `main.rs`, which reads like a head start. It is not:
**6 of the 9 named IO/safety functions are in `main.rs`, the binary half.**

| in `main.rs` (binary half) | in the lib half (`sysw/record.rs`) |
| --- | --- |
| `write_private` `stdout_world_readable_mode` `destination` `write_block` `refuse_write_block` `read_records` | `is_secret` `is_bearer` `is_argv_forbidden` |

**And the closure is nearly double.** Those six call **five more** `main.rs`-local
functions, each of which must move or the extraction does not compile:

| function | lines | | function | lines |
| --- | --- | --- | --- | --- |
| `read_records` | 132 | | `refuse_terminal_destination` | 31 |
| `emit` | 44 | | `split_record_stream` | 29 |
| `write_private` | 40 | | `stdout_world_readable_mode` | 25 |
| `refuse_write_block` | 34 | | `no_records_guard` | 25 |
| `destination` | 31 | | `write_block` | 21 |
| | | | `refuse_world_readable_stdout` | 19 |

**431 lines of `main.rs`'s 2,226 — 19% of the file — across 11 functions.**

**So P0's first move is inside `me` and is not small.** Nothing crosses a crate
boundary until those 11 are a library; treating this as "donate three functions"
would discover the other eight during implementation, one compile error at a time.

**How this was measured, because the first attempt was wrong.** The dependency
closure came from comparing the names the six *call* against `main.rs`'s
top-level `fn` list. The first run returned **empty** — no transitive deps — and
that was an artifact: the parse had left `fn` glued to each name
(`fnwrite_private`), so the comparison could never match. **A planted positive
control caught it**: searching for a name known to be in the file also returned
nothing, which is impossible for a working list. After fixing the parse the
control hit and the five extras appeared. **An empty result is only evidence when
a control proves the search could have found something.**

**F-258 ADDENDUM — the 11 are library-shaped, with exactly one design question.**

Measured across all 431 extracted lines (control-checked: the block really does
contain the functions named):

| binary-only thing | hits |
| --- | --- |
| `std::process::exit` | **0** |
| the `Cli` struct | **0** |
| `clap` anything | **0** |
| `std::env::args` | **0** |
| `eprintln!` | **4** |
| `println!` | **4** |

**The first four zeros are the good news** — nothing in the closure reaches for
the binary's argument parser, its `Cli` type, or process exit. The code is
already library-shaped and the move is mechanical.

**The eight prints are the design question, and P0 must answer it before writing
code.** A library shared by six binaries that writes to `stdout`/`stderr`
directly cannot be tested without capturing process stdio, and a caller cannot
redirect or suppress it. That matters more here than usually: **the whole purpose
of this crate is controlling what reaches stdout**, so a component that prints
unconditionally is at odds with its own reason for existing.

Two shapes, and the choice belongs in the plan rather than the implementation:
return a value the caller emits, or take a `&mut impl Write`. The second keeps
the call sites terse and makes the tests direct; the first is a larger diff at
every call site. **Either is fine; discovering the question mid-implementation is
not.**

### F-259 — `me sysw wipe` tells the operator a zeros image is BEARER, because "carries no secret" rides the `--allow-world-readable` parameter and the terminal arm ignores it (owning phase: **P0**, or sooner) `#me` `#ux` `#shipped`

**Found 2026-08-26** by the io-seam design review, in code **published as v0.7.0
the same day**. Reproduced on a real pty:

```
$ script -qec "me sysw wipe --fill zeros" /dev/null
me: stdout is a TERMINAL, and this payload is BEARER.
Writing it here would paint 65536 bytes of raw binary across your scrollback — ...
  exit 2
```

**The payload is a 65,536-byte zero fill image, and the code says so itself.**

**The mechanism is a parameter carrying two different meanings.** `main.rs:1385`
declares `const WIPE_IMAGE_CARRIES_NO_SECRET: bool = true` and passes it to
`emit()` in the `allow_world_readable` position. That argument reaches
`write_block(out_given, allow_world_readable, stdout_is_tty, world_readable_mode)`
— whose terminal arm is unconditional:

```rust
// `--allow-world-readable` does NOT override this. It says "this file's
// permissions are my problem"; it is not a request to paint a bearer
// container across a scrollback, and the message offers a file route.
Destination::Terminal => WriteBlock::Terminal,
```

So one `bool` means **"the operator accepts file-permission risk"** to the flag
and **"this payload is not secret"** to `wipe`. The terminal arm deliberately
discards the first — and in doing so silently discards the second, which it was
never told about.

**The refusal is arguably RIGHT; the stated reason is FALSE.** Painting 64 KB of
binary across a scrollback is worth refusing whatever the secrecy. But the
message asserts something untrue about the operator's data, and that costs twice:
someone who wipes may believe they exposed a secret, and everyone learns the
BEARER label is unreliable. **A guard that fires for a good reason while stating
a false one is worse than one that says nothing.**

**Why every test missed it.** `write_block`'s unit test at `main.rs:2212` asserts
`write_block(false, true, true, None) == W::Terminal` — correct for the *flag's*
meaning, and its comment argues exactly that case. It never contemplates the
second meaning, so it locks the defect in while reading as deliberate. And all 12
tests in `world_readable_output.rs` redirect to files, so **none reaches the
terminal arm at all**.

**The fix is a separate channel, not a changed rule.** `wipe`'s "carries no
secret" needs its own parameter or a payload-kind enum; the terminal refusal
should stay and its message should say *what is actually true* — a large binary
image, not a bearer secret. **A `bool` that two callers read differently is what
a type prevents**, and this is the argument for the record-vocabulary half of
`mnemonic-io-lib` rather than an argument against the terminal gate.

**CLOSED 2026-08-27 by P0 row 4** (`crates/mnemonic-io-lib/src/observation.rs`,
`PayloadKind`, threaded through `write_block` and `emit` as its own parameter).
The refusal STAYS and the digit stays 2; only the false claim goes.
**Killed by mutation:** matching `WriteBlock::Terminal(_)` and hard-coding
`PayloadKind::Bearer` — the exact edit the probe used to re-create it — turns
`tests/terminal_destination.rs::a_wipe_image_is_never_called_bearer` RED with
the other 393 tests green. The reverse mutation (deleting the BEARER label
everywhere) turns `a_real_container_is_still_called_bearer` RED, so the label
must also survive where it is TRUE.

### F-260 — `mt encode` refuses mode 0620 saying it "grants read to group or others", when no read bit is set (repo: **mnemonic-transaction**; owning phase: **P1**, reassigned from P0 2026-08-26) `#mt` `#ux` `#shipped`

**Found 2026-08-26** while machine-checking the io-seam review's counterexample.
Reproduced with a valid transaction and a 0600 control that passes:

```
stdout mode 0600 -> mt encode exit 0, 796 bytes
stdout mode 0620 -> mt encode exit 1:
  "REFUSED — §8.2h, stdout is a file of mode 0620 — its permissions grant
   read to group or others"
```

**Mode 0620 grants group WRITE. It grants read to nobody but the owner:**

| | owner | group | other |
| --- | --- | --- | --- |
| 0620 | `rw-` | `-w-` | `---` |
| group read bit (4) | | **not set** | |
| `0620 & 0o044` (any read outside owner) | | **`0`** | |

**The refusal is defensible; the reason is false.** `mt`'s gate masks `0o077`
(`validate.rs:585,653`) — *every* group and other bit, including write — and a
group-writable destination **is** a real hazard: someone else can alter the
strings before they are cut. That is worth refusing. But the message names the
one hazard that is **not** present.

**This is F-259's exact shape in a second repo, found the same day** — a guard
firing for a good reason while asserting something untrue about the operator's
situation. F-259: `me` calls a zeros image BEARER. Here: `mt` calls a
write-only mode readable. **The pattern is a message hard-coded to the rule's
*name* rather than derived from what was actually measured**, and it costs the
same twice over: the operator fixes the wrong thing, and learns the diagnostics
are unreliable.

**`me` is NOT affected on this axis** — checked, not assumed. Its mask is
`0o044` (`main.rs:912`), read bits only, so its "world-readable" wording matches
what it tests. The two tools disagree about the *rule*, and only `mt` misstates
it.

**Why this belongs to P0 rather than a `mt` patch.** The divergence is the
io-seam review's load-bearing counterexample: `mt` and `me` ship near-identical
*mechanism* (`fstat`, mask, extract mode) and deliberately different *policy*
(which bits are disqualifying). The shared crate should carry the mechanism and
the **vocabulary for describing what was measured** — a message derived from the
observed mode cannot say "read" about a mode with no read bits, whereas a
hard-coded string can and does.


**REASSIGNED P0 → P1, 2026-08-26**, during the P0 plan's R0 round 0 (I4). The
reasoning that filed it against P0 still holds — a message *derived* from the
observed mode cannot make this error — but **P0 does not touch `mt`**: §7 places
`mt`'s adoption in P1, and a P0 that edited `mt`'s message would contradict its
own scope. Under the per-phase burndown rule an item whose owning phase has
passed is **overdue, not deferred**, so this moves rather than drifting.
**P1 owns it.** F-259 stays with P0 — that one is `me`'s.

### F-261 — `plan-table-check.sh` silently skips INDENTED tables, and does not list that among its blind spots (owning phase: **tooling**) — **DONE 2026-08-27** `#tooling` `#gates`

**Found 2026-08-26** by watching a number fail to move. A fold added a
five-row table nested under a list item; the gate reported **82 rows, 0
malformed** both before and after. The file contains **7 indented table lines**
the gate never counted.

The table was fine — verified by hand, 4 pipes on every row including the
header. **The problem is that "0 malformed" was reported over a set that
excluded it**, so the gate's clean result said nothing about the thing just
added.

**Its stated blind spots are cell CONTENT, intentionally-empty versus lost
cells, tables with no separator row, and pipes inside code spans. Indentation is
not among them** — so a reader has no way to know coverage was partial.

**Fix, in preference order:** count indented tables (they are valid Markdown and
this document uses them deliberately, to keep a table inside the bullet it
belongs to); or, if that is genuinely hard, **print the count of lines skipped
for indentation** so a silent zero becomes a visible one.

**DONE 2026-08-27.** The match is now on `l.lstrip()`, with the table's indent
tracked so a shift still ends it. Measured on the artifact that has them:
`SPEC_constellation_cli_uniformity.md` went **82 → 85 rows checked** — three
rows the gate could not previously see. The script's footer now names indented
tables as **covered** rather than leaving the omission for a reader to discover.

**Mutation-tested, because a gate that cannot go red is a hypothesis.** Against
an indented table with a short row it exits **1** and names the line
(*"3 cells vs 4 declared"*); against the same table well-formed it exits **0**
and reports nothing. Both directions checked — the second matters as much,
since a gate red for non-defects is as corrosive as one green for everything.

**It introduced nothing.** `FOLLOWUPS.md` reported 2 malformed before the patch
(280 rows) and the same 2 after (284) — pipes inside cell content, which the
script already declares as an uncovered class and which are unrelated to
indentation.

**The general rule this instance illustrates:** a gate whose scope silently
excludes what you just changed is indistinguishable from a gate that passed on
it. This was only caught because the row count was expected to rise by five and
did not — **watching a number NOT move is a check, and it costs nothing.**

### F-262 — fork B-0's root cause: the Go decoder DISCARDS what the Rust primary keeps, so the message cannot be fixed on its own (repo: **seedhammer**, Rust-primary convergence) `#fork` `#rust-primary` `#ux`

**Traced 2026-08-26** while burning down the fork fold review's B-0 (*"the
`ErrUnsignedInputs` case still reads 'does NOT reassemble'"*).

**The screen contradicts the code's own comment.** `gui/transaction.go:186`
says of this exact case:

> *"The bytes ARE a transaction and the txid IS right. Calling that 'DOES NOT
> DECODE' would send the operator to re-encode a payload that is encoded
> perfectly well."*

…while `gui/transaction.go:831` tells the operator **"This does NOT reassemble
into a transaction."**

**Two paths, and only one is broken.**

| path | line | carries `tx`? | carries `unsigned`? | screen |
| --- | --- | --- | --- | --- |
| payload | 479–480 | yes | yes (`tx.UnsignedInputs`) | correct — takes the unsigned branch |
| **mt1 set** | 441 | **no** | **no** | **falls through to "does NOT reassemble"** |

The mt1-set path calls `substitutionFor(set, err)`, which *does* branch on
`ErrUnsignedInputs` and return `legendUnsigned` — **so the legend is right while
the screen above it is wrong, and only one of them can be true.**

**Why the message cannot simply be reworded.** `mt/mt.go:259` returns
**`Tx{}, ErrUnsignedInputs`** — a zero transaction. The set path has nothing to
display because the decoder threw it away.

**MANDATORY RUST CHECK — run, and it is the finding.** The primary does **not**
discard it:

| | Go (`mt/mt.go:259`) | Rust (`sysw/mt.rs:191`) |
| --- | --- | --- |
| on unsigned inputs | `return Tx{}, ErrUnsignedInputs` | `SetProblem::UnsignedInputs { txid, inputs }` |
| txid retained | **no** | **yes** |
| which inputs are unsigned | **no** | **yes** (`summary.unsigned_inputs`) |

And Rust keeps them *deliberately* — `sysw/mod.rs:117` states the reason: **"a
refusal that says only 'an input is unsigned' gives them nothing to look at."**

**So this is a Go-port information loss against a correct Rust primary**, which
makes it convergence rather than a behaviour change: exempt category (a) of the
Rust-primary rule, fixable in Go directly, no Rust change owed. The Rust check
was not skipped, and it is what turned a wording fix into a signature fix.

**What closes it:** give the Go decoder a variant that returns the decoded `Tx`
alongside `ErrUnsignedInputs` (the payload path at :479 already proves the data
exists at that point), populate `unsigned` on the mt1-set candidate at :441, and
let `transactionReviewLines` take the same unsigned branch both paths deserve.
**Rewording line 831 alone would replace a false sentence with a vague one.**

### F-263 — worktree hygiene: the branch outlives the tree, and the wrapper outlives both (owning phase: **standing discipline**) `#housekeeping` `#process`

**Established 2026-08-27** at the operator's direction — *"Do not lose track of
worktrees. We don't want dangling trees after merge."* — after one session
created **seven** of them.

**`git worktree remove` is only one third of the cleanup**, and the two thirds
it leaves behind are silent:

| what remains | how it shows up |
| --- | --- |
| the **branch** | `git branch --list` — 12 were found, six from cycles already closed |
| the **parent wrapper** | an empty dir `git worktree list` no longer knows about — 14 were found |
| the **registration**, if the dir was deleted by hand | `git worktree prune` |

**The three-step ritual, in order:**

```sh
git worktree remove --force <path>     # 1. the tree
git branch -D <branch>                 # 2. the branch  <-- the forgotten one
rmdir <parent>                          # 3. the wrapper
git worktree prune                      # 4. any stale registration
```

**Never step 2 before verifying the work reached `master`.** The check that
matters is not "is the report committed" but **what does this branch ADD that
master lacks**:

```sh
git diff --name-only --diff-filter=A master..<branch>
```

`<nothing>` means safe. Eleven of twelve returned exactly that; the twelfth held
a probe's throwaway `io.rs`, disposable by design. **`git branch -d` will not
help here** — a review branch's persist commit is never an ancestor of `master`
when the controller lands the report by checking the file out, so `-d` refuses
every one of them and the safety it offers is illusory.

**Sweep the whole constellation, not one repo.** Worktrees were registered in
five: `mnemonic-engrave`, `mnemonic-transaction`, `mnemonic-key`,
`mnemonic-toolkit` (under `.claude/worktrees/`) and `seedhammer`. A per-repo
`git worktree list` misses the other four.

**Related:** [[F-256]] is the disk half of this. The trees themselves are small;
what makes them expensive is each carrying its own `target/`.

### F-264 — `me`'s zsh purge recipe removes NOTHING when run immediately, under stock zsh defaults (owning phase: **P0**) `#me` `#security` `#shipped`

**Found 2026-08-27** by the step-4 probe, which wrote the positive test §6
condition 5 demands and then watched it fail. **The test was worth writing
because it failed.**

**The sequence, on this machine's own configuration** — stock zsh 5.9.2:

1. The operator puts a secret on argv. `me` refuses and prints a purge recipe.
2. They run it **immediately**, as the message invites.
3. zsh is still holding that entry **in memory**. `HISTFILE` does not contain it
   yet.
4. `sed -i` edits a file the secret is not in, **exits 0, prints nothing**.
5. At session exit, zsh writes its in-memory history — **including the secret** —
   to disk.

**The operator is told to purge, does exactly as told, sees success, and the
secret lands on disk anyway.** That is the same class of defect as `history -d`,
which this very message exists to warn against.

**Fish is worse on three counts and disarmed on a fourth.** Its recipe is
interactive, so it deletes nothing unattended; it **re-displays the secret**
while asking for confirmation; its anchored `--prefix` misses path-qualified
invocations; and on this operator's machine a `history` function in
`config.fish:105` **drops all arguments**, disarming it entirely.

**What would fix the zsh half:** the recipe must flush before it edits — `fc -W`
(or `fc -AI`) first, then `sed -i`, then `fc -R` to reload — or the message must
say plainly that the entry is still in memory and the shell must be exited
first. **Either is honest; the current text is not.**

**Not a regression and not urgent** — it is as old as the message, and an
operator who never puts a secret on argv never sees it. But it is a security
message that reports success while achieving nothing, which is precisely the
shape this repo disqualified `mt`'s text for.

**CLOSED 2026-08-27 by P0 row 5** (`crates/mnemonic-io-lib/src/remedy.rs`).
**The prescribed fix did not work and only running it showed that:** this entry
and the plan both proposed `fc -W`, `sed -i`, `fc -R`, and measured on a real
pty under stock zsh 5.9.2 that recipe still leaves the secret on disk, because
`fc -R` APPENDS the file to the in-memory list rather than replacing it. The
shipped recipe zeroes `HISTSIZE` to empty memory, restores the operator's own
value, then re-reads. **bash had the identical defect** and needed the identical
shape (`history -w`, `sed -i`, `history -c`, `history -r`); the old text said
`bash/zsh:` and was wrong for both.
**Killed by mutation:** reverting the zsh recipe to the shipped `sed -i` alone
turns `tests/history_purge.rs::the_emitted_zsh_recipe_actually_purges_the_entry`
RED — the gate reproduces this finding. The fish half is **not** closed and is
filed as **F-271**.

### F-265 — `me` can respell five refusals from exit 2 to exit 3 with all 388 tests green (owning phase: **P0**) `#me` `#tests` `#false-pass`

**Found 2026-08-27** by the fourth-split probe, with a control that makes it
unambiguous: mutating the **unmodified baseline's** own integers at five sites
leaves **388/388 passing**, and each mutated line was **proven to execute** by
watching the binary's exit code change.

| site | mutation | suite |
| --- | --- | --- |
| `refuse_write_block`, Terminal arm | 2 → 3 | **green, missed** |
| `refuse_write_block`, WorldReadable arm | 2 → 3 | **green, missed** |
| `read_records`, `--in` error | 2 → 3 | **green, missed** |
| `read_records`, stdin error | 2 → 3 | **green, missed** |
| `emit`, write failure | 2 → 3 | **green, missed** |

**Exit 2 is a usage error; exit 3 is a policy refusal.** They mean different
things to a script, and five of `me`'s refusals can swap one for the other
undetected.

**This is not introduced by any pending work** — the control proves it is the
state of the shipped binary. It is filed because P0 is about to move exactly
these functions, and a refactor over an untested distinction is how the
distinction quietly dies.

**What closes it:** assert the **digit**, not `!success()`. The tests currently
check that a refusal happened, never which kind — which is why the mutation
survives. This is also why §4's pty assertion for F-259 must pin the exit code
rather than mere failure, or it misses even the arm it is named for.

**CLOSED 2026-08-27 by P0 row 7** (`crates/me-cli/tests/exit_digits.rs`, plus
site 1 in `tests/terminal_destination.rs` where the pty machinery is).
**Killed by mutation at all five sites**, each mutated 2 → 3 in the shipped code
and run against the whole suite: Terminal arm → 3 FAIL; WorldReadable arm,
`--in` error, stdin error and `emit`'s write failure → 1 FAIL each. Every
assertion pins the digit AND a distinguishing phrase, so a different refusal
that also exits 2 cannot satisfy it, and a control pins that 0 and 3 are still
reachable — without which a build where everything exited 2 would pass.

### F-266 — **`me` echoes secret material verbatim to stderr on many argv shapes** (owning phase: **P0**, gating) `#me` `#security` `#shipped` `#critical`

**Found 2026-08-27** by R0 round 6, reproduced by the controller with a real
`ms1` secret from the repo's own fixtures:

| invocation | rc | secret in stderr |
| --- | --- | --- |
| `me <ms1>` | 2 | **YES** |
| `me bundle <ms1>` | 2 | **YES** |
| `me sysw wipe <ms1>` | 2 | **YES** |
| `me sysw show <ms1>` | 2 | **YES** |
| `me sysw pack <ms1>` | 3 | no — reaches the post-parse guard |
| `me sysw pack --nosuchflag <ms1>` | 2 | no |

**The table above is a SAMPLE, not the surface.** Later measurement found more:
`me sysw pack --in <ms1>` and `--in=<ms1>` both leak, as do `me sysw <ms1>` and
`me help <ms1>`, and the `pass:` record leaks on 7 of 8 shapes. **Do not treat
the list as exhaustive** — it was assembled by hand twice and came up short
both times.

**The mechanism is the one this repo already documented as `mt`'s.** The guard
runs after `Cli::parse()`, and only `me sysw pack`'s positional-records path
reaches it. On a surface that declares no positional, **clap rejects the
argument and names it** — printing the secret. `me sysw show` DOES declare one
(`<FILE>`, `crates/me-cli/src/main.rs:275`), so there the token is accepted as
a filename and reaches stderr through the file-open error instead (measured:
rc 2, `No such file or directory`, naming the token). And `--in <ms1>` /
`--in=<ms1>` leak as a DECLARED flag's value. `grep -c 'env::args'` over
`crates/me-cli/src/main.rs` returns **0**: nothing runs before `Cli::parse()`.

**The secret then lands in the terminal scrollback, and in whatever captured
that stderr.** It is already in the shell history and the process list by the
time `me` runs; this adds a third copy in the one place the operator is most
likely to screenshot or paste.

**Why it was not caught.** The repo's no-leak tests (`crates/me-cli/tests/cli.rs:153`,
`:177`) all use `write_stdin(...)` — **the argv path is tested nowhere**. And the
P0 plan asserted *"`me` does not currently leak this way"* on the strength of a
single probe, `me sysw pack --nosuchflag <ms1…>`, which is **the one invocation
that structurally cannot leak**: `--nosuchflag` makes clap name the *flag*
rather than the value. **A negative inheriting a scope of one, and the one
chosen was the exception.**

**OPERATOR RULING 2026-08-27: deferred, not fixed now** — *"Nobody cares about
leaks, we can file them for fixing later."* No emergency fix, no yank of v0.7.0.

**It is still what condition 8 is FOR, and P0 fixes it as a side effect** —
§6d's pre-parser guard is spec-normative for P0, and a guard running before
`Cli::parse()` closes every row of the table above. So this is deferred in the
sense of *not interrupting the cycle*, not in the sense of *unowned*.

**What must change in the plan regardless of timing:** the gate. §6d's pre-parser
ordering is not a tidiness requirement — it is the fix for this. The observable
must be *"no `ms1` in stderr for an argv clap would reject"*, asserted across
**every** surface, not one.

**`mt` is NOT affected** — checked, not assumed: its guard sits on
`std::env::args()` and runs before `Cli::parse()`, which is exactly why this
repo's spec cites it as the reference for the ordering.

**CLOSED 2026-08-27 by P0 row 6** — `argv_secret_guard` in
`crates/me-cli/src/main.rs`, running on raw `std::env::args()` before
`Cli::parse()`, normalising every token and asking `me`'s own classifier.
**Killed by mutation:** removing the guard (i.e. the shipped tree) leaks the
material on **225 of 450** generated cross-product rows — 75 canonical, 75
leading-space, 75 UPPERCASE. Dropping only the trim+lowercase leaks 90, of which
**zero are canonical**. Positive controls fail under a refuse-everything guard,
so the gate cannot be satisfied by refusing more.
**Scope correction, filed as F-272:** the plan's surface list omitted `seal` and
`hash`. `me seal --in <ms1>` leaked, and is now covered; `seal` declares
`--allow-argv-secret` so its documented positional is gated rather than deleted.

### F-267 — a secret embedded in a PATH reaches stderr, and no argv guard can catch it (owning phase: **post-P0**, documentation) `#me` `#security` `#residue`

**Found 2026-08-27** by R0 round 10, as the honest residue of P0's argv guard.
`me sysw pack --in /tmp/<ms1>.txt` leaks on five measured invocations, and
`classify()` correctly calls that token **`Unknown`** — because it *is* a
filename.

**Refusing it would refuse every legitimate path.** A guard that rejects any
token containing an HRP substring rejects `~/backups/ms1-recovery-notes.txt`,
which is a file the operator deliberately named. **This is not fixable by
classification**; the token is a path, and the secret is in the operator's own
naming choice.

**So P0's gate says "as a token", not "anywhere in argv"** — the narrower claim
is the true one, and an earlier draft promised the wider one it could not keep.

**What would actually help** is documentation, not a refusal: say plainly that a
filename containing key material puts that material in shell history, `/proc`,
and any error message naming the path. That belongs with the purge guidance, not
in a guard.

### F-268 — the flag-name argv layer is normative in §6d and built by nobody (owning phase: **P3**) `#constellation` `#security`

**Filed 2026-08-27.** Spec §6d calls the flag-name layer **the primary layer**
and assigns the union to P0. P0 builds the **value scan** — every token
classified, five argv-forbidden classes — and **does not build the flag-name
layer**, because `me` declares no flag that carries secret material, so it has
no failing gate in this donor.

**That is a fact about `me`, not a reason the layer is unnecessary.** §6d's own
example is a `--passphrase <arbitrary text>` flag: **a flag can carry material no
shape test recognises.** The value scan does not dominate it.

**Nothing in the constellation has a pre-parser guard today** — `env::args` and
`args_os` are **0** in `mnemonic-toolkit` as well as in `me`. So this is not a
gap P0 opens; it is one P0 declines to close, deliberately and in writing.

**Owning phase: P3 — corrected 2026-08-27 (R0 round 11, I-2).** The original
trigger — the first secret-bearing flag in any m-format CLI — was already
satisfied when this entry was filed: `mnemonic restore --passphrase
<PASSPHRASE>` and `--passphrase-stdin` ship in the toolkit today (declared in
its `cmd/slip39` module at line 101; its `flag_is_secret`, secrets module lines
60-68, also lists `--bip38-passphrase` and `--decrypt-password`), and spec §7
P3's gate names two of those flags. A trigger already met is an owning phase
already due, so the layer is assigned to P3 — the phase whose gate needs it —
rather than left on a condition that can never fire again.

### F-269 — operator override: fable folds R11 and R12, and fable performs the final review (owning phase: **recorded, not work**) `#process` `#record`

**Recorded 2026-08-27** so a later reader does not mistake the standing rule for
the governing one.

`CLAUDE.md` in this repo states, from the operator's own directive of
2026-08-16, that **fable is not a reviewer tier** — *"we will not use fable for
final review"*, with opus as the top of the ladder including for the final
pre-irreversible review, and an explicit instruction not to propose fable for a
gate.

**On 2026-08-27 the operator overrode that**, directing that the next two folds
go to fable and that the fold then go to fable for final review. **The
controller did not propose this and is barred from doing so; the override is the
operator's.**

**Why it is a reasonable call.** Eleven review rounds had run, and **every
Critical since round 2 was a defect the FOLD introduced**, not the plan. The
design was settled by probes; the folding was the failure. Changing the model
holding the pen tests the one variable that had not been varied.

**How independence is preserved.** The reviewer is dispatched as a **fresh agent
with no shared context** — it does not inherit the folder's reasoning and reads
the persisted report rather than the folder's account of it. *"Author ≠ reviewer
on the same artifact"* is about context, not about model identity, and a
separate dispatch satisfies it.

**What this entry does NOT do:** it does not amend `CLAUDE.md`. The standing
rule remains as written and continues to bind the controller. This is one
operator-directed exception, scoped to this artifact and these two folds.

### F-270 — `me`'s shipped post-parse gate normalises for its `tx:` prefix arm only, so a near-miss secret of any OTHER class is refused for the wrong reason (owning phase: **P0**) `#me` `#security`

**Filed 2026-08-27**, from R0 round 10's M-6 on the P0 plan, carried by round
11. The post-parse argv gate builds a normalised copy of each record —
`crates/me-cli/src/main.rs:1952` — but feeds it only to the `tx:` prefix arm
(`:1958`); `classify` receives the RAW token (`:1978`), and `classify` itself
neither trims nor case-folds. So the near-miss protection the gate's own
comment describes — refuse "for the BEARER reason rather than three screens
later for a formatting one" — exists for one class of five.

**Measured 2026-08-27**, streams separated, rc taken directly: `me sysw pack`
on ` pass:<hex>` (leading space) and on an uppercase `MS1…` record both
classify `Unknown` and are refused as *not a form this container can place* at
**rc 4**, not as *SECRET key material on ARGV* at rc 3 — the wrong reason at
the wrong code, and the message points the operator at `sysw::classify` rather
than at purging their history. (Neither message names the body; the leak
surfaces are the clap-echo ones F-266 records, not this one.)

**Owning phase: P0** — the pre-parser guard P0 builds normalises every token
before `classify`, which closes the argv path outright; the post-parse arm is
in exactly the code P0 rewrites, so it gets the same one-line normalisation in
passing. The cheapest moment to fix a gate is the phase already holding it
open.

**CLOSED 2026-08-27 by P0 row 6**, in the same commit as F-266. `read_records`'
post-parse gate now normalises with `r.trim().to_ascii_lowercase()` and feeds
the result to BOTH arms; `classify` no longer receives the raw token.
**Killed by mutation:** restoring `classify(r)` turns
`main.rs::tests::the_post_parse_argv_gate_normalises_for_every_class` RED.
That gate is a UNIT test deliberately — the pre-parser guard now shadows this
arm end-to-end, so an integration test would be measuring the guard and calling
it this.

### F-271 — `cargo publish mnemonic-io-lib 0.1.0` is AUTHORISED; the pre-flight is not yet run (owning phase: **P0 row 12**) `#irreversible` `#record`

**Operator authorisation given 2026-08-27**, recorded here because a
one-sentence approval in conversation is lost when that context ends, and this
is the only irreversible action in the phase.

**Held until rows 1–11 are green.** Not a second guess — a sequencing fact. A
published version cannot be replaced, only yanked, and a yanked version stays
downloadable to anyone pinning it exactly. **The name is consumed
permanently.** §5a already says *published when P0 closes GREEN*, and row 12 is
last in the table.

**PRE-FLIGHT — every item runs immediately before, not from this record:**

| check | why |
| --- | --- |
| `curl -A 'name-check' …/crates/serde` → **200** | proves the request is being answered; without `-A`, crates.io returns **403 for every name**, free or taken |
| `…/mnemonic-io-lib` → **404** | and `mnemonic_io_lib` → **404**; crates.io treats `-` and `_` as colliding |
| `cargo publish --dry-run` | packages and verifies without publishing |
| **no `path` or `git` dependency** in the manifest | crates.io **refuses** git deps outright, and `me-cli` already carries one pinned to a rev — if the new crate inherits anything like it, the publish fails at the last gate |
| working tree clean, all rows committed | `cargo publish` packages the tree, not the commit |
| rows 1–11 green, 388+ tests passing | the plan's own condition |
| **a fable review of the whole implementation diff, 0C/0I** | operator-directed (F-269, extended); runs **before** the pre-flight commands above, since a finding there changes what gets published |

**A 404 is availability at a moment, not a reservation.** Nothing holds the
name until the publish lands, so the check is re-run at the moment of
publishing rather than trusted from here.

### F-272 — the P0 plan's argv-guard surface list is short by two subcommands, and `seal` is the one that mattered (owning phase: **P0, closed in the same commit**) `#plan` `#me` `#security`

**Found 2026-08-27** while implementing plan §4 row 6. Recorded because a future
reader comparing the plan against the code will otherwise find the code doing
more than the plan asked, with no explanation.

**The plan's cross-product enumerates eight surfaces:** `{bare, bundle, sysw,
sysw pack, sysw show, sysw wipe, help, sysw help}`. **`me` has five top-level
subcommands** — `bundle`, `sysw`, `seal`, `hash`, `help` — so `seal` and `hash`
appear nowhere in it.

**`seal` is not a cosmetic omission.** Measured against the pre-guard binary:

```
me seal --in <ms1> --out x.uf2
  → me: cannot read ms10entrs…: No such file or directory
```

That is F-266's exact mechanism on a surface the gate never looked at. The
POSITIONAL shape on `seal` is clean — `seal` *accepts* it, so clap never errors
and never echoes — which is precisely why a surface-by-surface hand list missed
it: the shape that leaks is not the shape that looks dangerous.

**Resolution, from an architect consult
(`design/agent-reports/CONSULT-P0-row6-seal-surface.md`), folded in the same
commit:** the guard covers `seal`, and **`seal` now declares
`--allow-argv-secret`** — the same explicit override `sysw pack` carries — so
its `payload` positional, documented as *"Kept for FIXTURES AND TESTS only"*
with F-102 attached, is **gated rather than deleted**. It does not overlap
`--seal-secret`: that one says *encrypting seed material is what I meant*, this
one says *argv is safe where I am*, and each still refuses on its own.

**`hash` needed nothing, and this was measured rather than assumed** — the
consult predicted `hash` would break *"worse than `seal`"* because its
positional supposedly carries `tx:`/`mt1` legitimately. **That is false.**
Against the pre-guard binary, `me hash` already refuses all five argv-forbidden
classes with its own messages:

| input | pre-guard `me hash` |
| --- | --- |
| `ms1…` | *record 0 is secret material; the public-data hash covers public records only* |
| BIP-39 mnemonic | *non-canonical record: separator ' ' at byte 7* |
| `pass:…` | *unrecognised record: not a bech32 string* |
| `mt1…` | *mt1 records belong to the systemwide container (`me sysw pack`), not the frozen sealed payload* |
| `tx:…` | *unrecognised record: unrecognized HRP 'tx:…' (expected md, mk, ms, or mt)* |

So the guard changes `hash` only by refusing earlier, for the argv reason, with
a purge recipe attached. All four `hash` invocations in the test suite pass
md1/mk1 only and were untouched. **A consult is not a measurement.**

**The gate carries all ten surfaces**, `seal` and `hash` included:
`tests/argv_secret_guard.rs` generates 450 rows and asserts 0 leak.

**CLOSED 2026-08-27 by P0 row 6**, in the commit that found it. The gate
`crates/me-cli/tests/argv_secret_guard.rs` carries **all ten** surfaces — the
plan's eight plus `seal` and `hash` — and generates 450 rows, 0 leaking.

### F-273 — `me`'s fish purge advice cannot be verified, and `history delete --prefix` purges nothing unattended (owning phase: **P1**) `#me` `#ux` `#shipped`

**Found 2026-08-27** while building P0 row 5's F-264 gate. The zsh and bash
halves of the purge recipe were fixed and are now covered by a positive test
that runs the emitted recipe under a real interactive shell. **The fish half is
not, and this records exactly why rather than shipping an unverified recipe.**

**What was measured.** With a history file planted by hand in fish's own format
(`- cmd: …` / `when: …`) under an isolated `HOME`/`XDG_DATA_HOME`, and fish
4.8.1 on a pty:

```
history delete --prefix 'me sysw pack'   → blocked for 2 minutes, deleted nothing
```

It is **interactive**: it lists the matching commands — the secret among them —
and waits for a confirmation that never comes when the operator pastes the
recipe into a script, a `&&` chain, or a non-interactive shell. So it re-displays
the material and purges nothing, which is the pair of properties that
disqualified `mt`'s text.

**What could not be measured, and so was not claimed.** A fish session harness
(`fish -i < file` on a pty, isolated `XDG_DATA_HOME`) produced **no history file
at all**, so its control failed — the harness could not distinguish "purged"
from "never recorded". Under the rule that a negative inherits the scope of what
was searched, no fish recipe was written on the strength of it. Candidates worth
measuring when a working harness exists: `history clear-session` (coarse — drops
the whole session, but needs neither a prompt nor the secret retyped) and
`history delete --exact` (rejected on sight: it requires the operator to type the
secret a second time, which is the defect the message exists to prevent).

**What P0 shipped instead.** The fish line is **described, not prescribed**: it
states that the command prompts, lists the matches with the secret in them, and
purges nothing unattended. That is the same idiom the message already uses for
`history -d` — name it, warn against it, never offer it.

**Owning phase P1**, with `mt`'s adoption of the shared remedy text: that is the
phase that touches this text next, and a fish recipe belongs in the crate's
remedy module once one has been measured. Until then the honest description
stands.

### F-274 — CLOSED 2026-08-27 (P1 step 3) — `mt`'s argv guard did not TRIM, so a whitespace-padded bearer artifact leaked verbatim through clap (repo: **mnemonic-transaction**; owning phase: **P1**) `#mt` `#security` `#shipped` `#critical`

**Found 2026-08-27** while measuring `mt` for the P1 plan.
`looks_like_a_transaction` (`crates/mt-cli/src/validate.rs:503`) lowercases the
token for its `mt1` arm and **never trims it**. A leading or trailing space
therefore makes the material unrecognisable to the pre-clap guard, it falls
through to `Cli::parse()`, and clap echoes the whole thing to stderr — which is
the exact leak `command_line_guard` was moved before clap to prevent, recorded
in `mt`'s own source at `crates/mt-cli/src/main.rs:234`.

**Measured as a generated cross-product**, 4 verbs × 2 carrier classes (an `mt1`
set, a raw transaction) × 4 spellings:

| spelling | result on all four verbs, both classes |
| --- | --- |
| canonical | rc 1, refused by the guard, **no echo** |
| UPPERCASE | rc 1, refused by the guard, **no echo** |
| leading space | **rc 2, the material verbatim in stderr** |
| trailing space | **rc 2, the material verbatim in stderr** |

**16 of the 32 rows leak.** The canonical and uppercase rows are the positive
control: the guard is not simply refusing everything, and a fix that trims must
leave them exactly as they are.

**THE SIBLING WAS CHECKED RATHER THAN ASSUMED.** `me` refuses all four spellings
at rc 3 with no leak, measured against the built binary — P0's row 6 normalises
every token before classifying, and closed the analogous `me` defect as F-270.
So this is `mt`-only, and the fix converges on the donor rather than leading it.

**Residue, stated rather than papered over.** The hex arm also has a threshold:
a 99-character odd-length hex string is below `looks_like_a_transaction`'s
`len >= 100 && len % 2 == 0` test, and clap echoes it. That is the same class as
F-267 — material the guard cannot classify without refusing legitimate input —
and trimming does not close it.

**CLOSED 2026-08-27**, `mnemonic-transaction` `impl/p1` commit
*"P1 step 3 (F-274): the argv guard normalises BEFORE it classifies"*. One
`trim()` in `command_line_guard`, before the recogniser is consulted, plus a
`debug_assert_eq!(a, a.trim())` inside the recogniser so a second caller that
forgets it fails loudly rather than silently re-opening the leak. The generated
32-row grid is committed as
`no_spelling_of_a_bearer_argument_reaches_stderr` and is listed in
`refusals.toml`, so `mutate-refusals.sh` covers it: neuter
`command_line_guard` and all 32 rows leak. The refusal now reports the
NORMALISED character count — quoting the padded length would give an operator
comparing it against what they pasted a number that matches nothing.

**The residue above is NOT closed** and was not touched: the 99-character
odd-length hex string is still below the recogniser's threshold and is still
echoed. It remains the F-267 class.

### F-275 — `mt decode` writes broadcastable bearer hex to a world-readable stdout at exit 0, while `mt encode` refuses the identical destination (repo: **mnemonic-transaction**; owning phase: **P1** — RULED 2026-08-27, and the plan's decode-warning row now builds it) `#mt` `#security` `#shipped`

**Found 2026-08-27** while measuring which `mt` verbs `--out` should reach.

`world_readable_stdout_guard` has exactly one caller,
`crates/mt-cli/src/main.rs:701`, inside `encode`. `mt decode` emits
BROADCASTABLE HEX on stdout — its own `--help` says so in those words — and has
no gate at all. Reproduced with a 0600 control that passes:

```
mt encode --qr  > <a 0644 file>   -> exit 1, REFUSED §8.2h, 0 bytes written
mt decode       > <a 0644 file>   -> exit 0, 679 bytes of broadcastable hex
```

`file_mode_warning` is the same shape: one caller, also `encode`
(`crates/mt-cli/src/main.rs:301`), so the reading verbs warn about nothing
either.

**The inconsistency is the hazard, not just the gap.** An operator who has met
§8.2h on `encode` has been taught that `mt` refuses a world-readable output.
Nothing tells them the lesson stops at one verb, and the artifact `decode`
produces is the one that can be broadcast without any further step.

**Why it is not P1's to fix.** Closing it needs a new refusal on a verb that has
never had one, which the P0 plan's own out-of-scope rule calls a **ruling, never
a refactor** — the same rule that keeps `mt`'s terminal policy and its `0o077`
mask out of P1. And `--out` alone would NOT close it: `--out` adds a private
channel, it does not stop a plain `>` from creating 0644. Half a fix that reads
as a whole one is worse here than the honest gap.

**OPERATOR RULING 2026-08-27: WARN, do not refuse.** *"These are stupid minor
details that don't warrant optimizing yet. Just print a warning, file an issue,
and move on."*

**So: `mt decode` prints a warning naming the mode it measured, and proceeds at
exit 0.** It does not refuse, and `--out` is not added to close it.

**The reason the cheap option is the right one here**, recorded so it is not
re-litigated: the operator's umask is **022**, so a plain `mt decode > tx.hex`
creates the file at **0644** — which an `encode`-style refusal would reject. On
every default-configured machine, consistency would break the ordinary
invocation on the first try. A warning costs nothing and says the true thing.

**This entry stays open** as the record that the asymmetry is deliberate rather
than overlooked. Whoever revisits it should know `encode` refuses and `decode`
warns **by decision**, not by omission.

### F-276 — the shared crate's boundary is `me`-shaped in two places, found by the first second consumer (owning phase: **`mnemonic-io-lib` 0.2, before a third consumer**) `#mnemonic-io-lib` `#P1` `#design`

**Found 2026-08-27** writing the P1 plan. `mt` adopts **5 of the crate's 11
public items and 3 of its 7 modules**. Three of the six declines are ordinary —
`mt` has its own record reader and its own empty-input refusal, and those are
better. **Two are the crate's shape, and they are worth fixing while it is still
unpublished.**

**1. `exit::write_block` encodes `me`'s terminal policy in its control flow.**
It carries no exit integer, which was P0's whole test for policy — and its
`Destination::Terminal` arm still returns a refusal unconditionally. `mt` has no
terminal refusal: measured under util-linux `script` on a real pty, `mt encode`
exits **0** and paints 1198 bytes of `mt1` strings. So adopting the function
would give `mt` a refusal no ruling authorises, and the only way to call it
without that is to pass `stdout_is_tty: false` — a lie to a function about an
observable fact, which the next reader repairs. `WriteBlock` goes with it: its
`Terminal(PayloadKind)` variant would be unconstructible in `mt`, and a dead
variant in a shared decision type is how the policy behind it gets adopted later
by someone tidying up.

**No integer and no mask is a NECESSARY test for policy, not a sufficient one.**
Control flow is a mapping too.

**2. `observation.rs` shipped half of its own argument.** The P0 plan argues for
the module from F-259 **and** F-260 jointly — *"a message computed from the
observed mode cannot say 'read' about a write-only mode"*. What shipped is
F-259's half: `PayloadKind`, a payload-kind type. **There is no vocabulary for
describing an observed mode anywhere in the crate.** `me` did not notice because
its own mask is `0o044`, so its hard-coded *"grant read to group or others"* is
true for every mode it refuses. `mt`'s is `0o077`, so the same sentence is false
for the write-only modes it refuses — which is F-260, and P1 has to write that
wording in `mt` rather than reach for it.

Both are cheap now: `mnemonic-io-lib` is unpublished (checked 2026-08-27 with a
`serde` control: 200 for the control, **404** for both spellings of the name)
and `me` is its only consumer.

### F-277 — §6d rules the override's parse and its routing, and is SILENT on the collision with `--in`; `mt` had to invent an answer (owning phase: **the spec, before P2 gives a second tool the override**) `#spec` `#mt` `#ux`

**Found 2026-08-27** implementing P1's override work — the row named *the
override* in `IMPLEMENTATION_PLAN_P1_mt_adopts.md`.

§6d rules two things about `--allow-argv-secret` and rules them well: the
override's own parse runs on raw argv, and *"admitted material is passed to the
tool through the same internal path as `--in` content, and never re-presented to
clap as a positional"*. **It does not say what happens when BOTH are given.**

```
mt encode --allow-argv-secret <a raw transaction> --in tx.psbt
```

Two sources, one channel. Nothing in §6d picks one, and every silent choice is a
defect: whichever source loses, the operator has no way to tell which of the two
`mt` just turned into the thing they will cut into metal.

**What `mt` does now, ruled at implementation time rather than deferred**, since
the row could not be built without an answer: **`--in` wins** — it is the private
channel and the explicitly named one — and a WARNING says so, naming the
discarded material's LENGTH and the path that was read, never the material. Test:
`material_on_argv_beside_an_in_file_is_warned_about_not_dropped`
(`crates/mt-cli/tests/refusals.rs`).

**Why this is filed rather than left as `mt`'s local answer.** §6d's override is
scheduled onto every tool in the cycle, and `ms` alone has **eight** verbs with
`--in` being added to all of them. Four more implementers will each meet this
collision and each answer it differently — and the one who answers it by
silently preferring argv has built the opposite of `mt`. The spec should rule it
once. `mt`'s answer is offered as the candidate, not as precedent by seniority.

### F-278 — RESOLVED 2026-08-27: F-275 was RULED but no plan row owned it, so the operator's decision was scheduled nowhere (owning phase: **P1** — closed by adding the decode-warning row) `#mt` `#plan` `#record` `#resolved`

**Found 2026-08-27** reconciling open follow-ups against P1's rows before
starting the adoption work, per the per-phase burndown rule.

F-275 was filed with owning phase *"a ruling the operator owes, before the phase
that acts on it"*. **The operator ruled it the same day** — WARN, do not refuse;
`mt decode` prints a warning naming the mode it measured and proceeds at exit 0;
`--out` is not added to close it. That ruling is recorded in F-275 in full,
including why the cheap option is the right one (the default umask is 022, so
`mt decode > tx.hex` creates 0644 and an `encode`-style refusal would break the
ordinary invocation on every default-configured machine).

**But P1's plan predates the ruling**, and its §7 still describes F-275 as a
ruling the operator owes. **No row in the twelve-row table builds the warning**,
and F-275's own heading still carries the pre-ruling owning phase. So the item
now has a decision, an agreed shape, and no schedule — the state a burndown
sweep is least likely to catch, because a grep for open items finds it and a
reader concludes correctly that it is *waiting on the operator*, which it is not.

**It is small and it is mt-local.** `file_mode_warning` already exists and
already has exactly the shape the ruling asks for; what it lacks is a caller on
the reading verbs and a stdout-side sibling. It needs a row, not a design.

### F-279 — 14 of 15 `mt` line citations in the P1 plan are stale for the branch that will consume them, and the citation gate is green on every one (repo: **mnemonic-engrave**; owning phase: **P1**, at the merge of `impl/p1`) `#plan` `#tooling` `#gate`

**Found 2026-08-27** while adding the decode-warning row, by checking a new
citation against the worktree instead of against the live checkout.

The plan's `crates/mt-cli/` line numbers are anchored at `cf17591`. Rows 1–4
have since landed on `impl/p1`, moving them. Measured by comparing each cited
line's *content* between `cf17591` and `a4cdefa`:

```
same = 1        drift = 14        (of 15 distinct mt-cli citations)
```

`plan-cite-check.sh` reports **41/41 resolved, 0 dangling** across the same
document. It is not malfunctioning — its header states the blind spot exactly:
*"WHAT IS ON THE LINE -- only that the line exists."*

**The hazard is a wrong citation, not a dangling one.** The `validate.rs` line
the plan describes as the `is_file()` keying comment now holds the opening of
`fn looks_like_a_transaction` — a real function the plan cites separately
elsewhere. A reader following the number lands somewhere plausible and wrong.
A dangling-citation check structurally cannot see this: the line resolves.

**Mitigated, not closed.** The plan now carries an anchor warning under its
row table telling implementers to locate every site by SYMBOL and re-measure
the line before quoting it, and the decode-warning row cites by symbol only.
That is enough for the rows still to be built; it does not fix the other
fourteen numbers.

**What would close it**, in ascending cost:

1. **Re-anchor at merge** — recompute the fifteen numbers once `impl/p1` is on
   `mt`'s `main`. One pass, and correct until the next row moves them again.
2. **Teach the gate content.** A citation written `path:line:anchor` where
   `anchor` is a substring that must appear on that line. Cheap, and it turns
   the whole class from "a reader must notice" into a command. This is the one
   worth building — it generalises to every plan in the constellation, and the
   same blind spot is in `mnemonic-toolkit`'s copy of the script.
3. Cite by symbol everywhere and drop line numbers. Drift-proof, and worse to
   read: the line number is what makes a citation checkable at all.

**Do not read this as "the gate failed".** It caught nine bare-path defects on
the P1 plan alone that no reading found. It reported its own limit in its
output, and the limit is where the next defect lived.

### F-280 — `mnemonic-engrave`'s tree is `cargo fmt --check` RED at 14 files, and CI cannot see it because CI never runs `fmt` (repo: **mnemonic-engrave**; owning phase: **after the P1 rev-pin push**) `#me` `#tooling` `#gate`

**Found 2026-08-27** building P1 row 5, when the row's own `cargo fmt --check`
gate came back RED on thirteen files the row had not touched.

**Measured at `ba1f3ec`, before any P1-crate work**, by stashing the working
tree and running CI's own pinned toolchain — not nightly, so this is not a
rustfmt-version artifact:

```
cargo +1.85.0 fmt --check   ->  exit 1,  77 hunks,  14 files
```

The fourteen are `crates/me-cli/src/main.rs`, `src/sysw/{expect,mod,mt,tx}.rs`,
seven files under `crates/me-cli/tests/`, and — the one that matters for the
boundary work — `crates/mnemonic-io-lib/src/remedy.rs`, whose single hunk is on
a line P1 row 5 did not write.

**Why it accumulated: `.github/workflows/release.yml` runs `cargo test --locked`
and the Go suites, and NOTHING else.** A grep for `fmt` or `clippy` across
`.github/workflows/` in this repo returns **no hits at all** — not a weakened
step, an absent one. So `cargo fmt --check` is a gate this project asserts in
prose and its pipeline has never executed: a hypothesis, not a gate, which is
exactly the shape the P1 plan's own closure rule warns about. Fourteen files
drifted with every job reporting green.

**Not fixed in P1's crate rows, deliberately.** Reformatting fourteen untouched
files is a ~1200-line whitespace diff, and rows 5 and 6 are the last work before
`master` is pushed to earn the SHA that `mt` will pin by rev. A churn commit in
that window buys nothing and risks the pin. Both rows instead added **no new fmt
debt**: `remedy.rs` still carries exactly its one pre-existing hunk, and every
file the rows created is clean under both `1.85.0` and nightly `rustfmt`.

**What would close it**, and the order matters:

1. **Add the `fmt` and `clippy` steps to the `test (rust + go)` job first.** A
   reformat without them re-drifts, and the required check is that job — so a
   step added there is enforced on every SHA that earns its way onto `master`.
2. **Then reformat, in its own commit, touching nothing else**, so the diff is
   reviewable as whitespace and a future `git log -S` is not poisoned by it.

Doing (2) before (1) is the version of this that gets done twice.

**Sibling check owed.** `mnemonic-transaction`'s
`.github/workflows/ci.yml:16` runs `cargo fmt --check`, and `:17`–`:18` run
`cargo clippy --all-targets --locked -- -D warnings`. That is why P1's row 1
could gate on both there, and why `mt`'s tree was merely *stale* rather than
unmeasured. This repo is the one where the steps are missing, so the same audit
is owed to any other constellation repo whose workflow was copied from this one
rather than from `mt`'s.
### F-281 — should `ms` gate a world-readable stdout at all? §9a says the gate is in scope; P2's row does not carry it, so it has no owning phase (repo: **mnemonic-secret**; owning phase: **operator ruling, before the cycle closes**) `#ms` `#cli-uniformity` `#ruling-needed`

**Found 2026-08-27** while writing the P2 plan, by measuring what `ms` does
today instead of assuming it matched a sibling.

`ms` has **no mode-checking machinery of any kind**. Measured, `git grep -n`
over `crates/` for `fs::write`, `OpenOptions`, `set_permissions`, `0o600`,
`0o077`, `0o044` and `st_mode` returns **zero hits**. It never fstats its
stdout. `mt` refuses a stdout whose mode has any bit in `0o077`; `me` refuses
`0o044`; `ms` refuses nothing, and `ms encode > backup.txt` under the default
umask 022 creates **0644** at exit 0.

**The scope is genuinely ambiguous and that is the finding.** §9a's in-scope
table lists *"the world-readable and terminal gates — they guard the binary's
own stdout"*. §7's P2 row and its gate both enumerate P2's content and neither
names one. An item that is in the cycle's scope and in no phase's row is
overdue rather than deferred, which is the shape the per-phase burndown rule
exists to catch.

**The P2 plan builds nothing, deliberately**, and records why: a REFUSAL is
foreclosed three ways — §6e's own retraction argument names `ms encode` as the
case where refusing makes the exposure strictly worse (the operator must then
read the file to hand-engrave, so a screen-only exposure becomes screen plus
disk); F-275 is the operator ruling the directly analogous `mt decode` case
(human-read output, default umask 022) as **warn-and-proceed, not refuse**; and
a refusal would reject the ordinary invocation on every default machine. A
WARNING is weaker but still unspecified, and its marginal information is thin,
because `ms` already prints *"warning: stdout carries private key material (can
spend) — redirect or encrypt"* unconditionally on the same stream.

**What is being asked for:** a ruling on whether §9a obliges any `ms` mode check
at all, and if so whether that existing unconditional line already discharges
it. **If anything is ever built here it is a warning, never a refusal** — F-275
is the precedent and it is attached so the ruling is cheap.

### F-282 — `ms gen-man --out <DIR>` collides with the `--out FILE` this cycle introduces: one binary, two meanings for one flag (repo: **mnemonic-secret**; owning phase: **a later cycle, not P2**) `#ms` `#cli-uniformity` `#ux`

**Found 2026-08-27** measuring `ms`'s existing `--out` surface for the P2 plan.
The sibling plan's equivalent measurement found `mt` had **one** `--out` in the
whole repository and it was a refusal string. `ms` is different:

```
ms gen-man --out <DIR>
      --out <DIR>   Directory to write the `*.1` man pages into (created if absent)
```

It is shipped, exampled twice in `--help` (`crates/ms-cli/src/main.rs:125`),
driven by `crates/ms-cli/tests/gen_man.rs`, and invoked by CI — the
`man-release.yml` workflow runs `./target/release/ms gen-man --out man` at its
line 46 to build the `ms-man.tar.gz` release asset. A second consumer sits in
**another repository**: `scripts/install.sh:305` in `mnemonic-toolkit` drives
`<bin> gen-man --out` across every sibling that carries the verb.

**`ms gen-man --help` points at the wrong repo for that installer**, saying
*"`scripts/install.sh` invokes this post-`cargo install`"* — and
`mnemonic-secret` has no `scripts/` directory at all. Believing the help text
was one edit away from shipping a false path into the P2 plan; the file was
located instead. That doc line is worth correcting whenever this is picked up.

After P2, `--out` means **a directory of man pages** on `gen-man` and **a file
holding the artifact** on `encode`, `split` and `repair`.

**Not fixed in P2**, because renaming it breaks a release workflow for a
cosmetic gain inside a phase whose row is funds-safety work. The two meanings
coexist and the P2 plan's decline asserts that `gen-man --out` still writes a
directory afterwards, so a later tidy-up cannot take it silently.

### F-283 — `mnemonic-gui`'s schema mirror for `ms` goes stale in P2, while §7 gives its regeneration to P3 (repo: **mnemonic-gui**; owning phase: **P3**) `#ms` `#cli-uniformity` `#gui`

**Found 2026-08-27** writing the P2 plan. `ms gui-schema` is clap-derived, so
`ms`'s flag surface reaches the GUI's schema mirror automatically — and the
mirror is a **third repository** with its own CI gate
(`bg002h/mnemonic-gui`, `mnemonic-gui-schema-mirror`).

Measured baseline today: `ms gui-schema` emits **10 subcommands carrying 36
flags** — derive 9, encode 7, decode 2, inspect 1, verify 3, vectors 1,
gen-man 1, repair 2, split 8, combine 2. P2 adds `--in` to eight verbs,
`--allow-argv-secret` to eight, and `--out` to three, taking it to **55**.

§7's P3 row says *"`mnemonic-gui`'s schema mirror regenerated"*, and §8 lists
*"`mnemonic-gui`'s four schema files"* among the invocations this cycle breaks.
**Neither says that the `ms` half goes stale one phase earlier.** P2 asserts
only that `ms gui-schema` describes the new surface; the mirror itself is P3's,
and this entry exists so P3 knows it inherited a drift rather than created one.

### F-284 — after P2, `ms encode` and `ms split` disagree about their own stdout: one is ungrouped by default, the other still groups in fives (repo: **mnemonic-secret**; owning phase: **P3**, with the `md`/`mk` grouping work) `#ms` `#cli-uniformity` `#engraving`

**Found 2026-08-27** while scoping P2's grouping work, by measuring whether §3's
argument reaches `ms split` and finding that it does not.

§6a rules *"the stdout rule binds `encode` only, this cycle"*, and §3's decisive
measurement is `encode`'s. The obvious extension — that `split` has the same
defect because it has the same flag with the same default of 5 — **is false, and
was measured rather than assumed.** Feeding `me sysw pack` a `ms split` share:

```
grouped (default)      -> exit 4, "record 0 ... is not a form this container can place"
--group-size 0         -> exit 4, same message
```

`me sysw` cannot place a codex32 **share** at all, grouped or flat, so grouping
is not what blocks it and the packability argument that decides `encode` has no
purchase on `split`. The share round-trips into `ms combine -` grouped, too —
measured at exit 0, because `read_shares` strips display separators per line.

**So P2 changes `encode`'s default and leaves `split`'s at 5**, and the result
is an intra-tool inconsistency with no measured pipeline behind it either way.
The separator rule DOES reach `split`, because one `parse_separator`
(`crates/ms-cli/src/format.rs:41`) serves both verbs and cannot bind to one.

**What P3 owes:** a ruling on `split`'s default, alongside the `md`/`mk`
card-invention work §6c already gives it — and note `ms split` has stderr
labels (`share 1 of 3:`) but **no `--no-engraving-card`**, so "move the grouped
form to the card" is not free there either.

### F-285 — `ms decode` and `ms combine` write a recovered seed phrase to an unprotected stdout, and gain no `--out` in P2 (repo: **mnemonic-secret**; owning phase: **operator ruling**, alongside F-281) `#ms` `#cli-uniformity` `#funds-safety`

**Found 2026-08-27** scoping which `ms` verbs get `--out` in P2.

`ms decode <ms1>` prints the BIP-39 mnemonic; `ms combine <shares>` prints the
recovered secret. Both are the seed in plain text on stdout, and
`ms decode <ms1> > recovered.txt` creates **0644** under the default umask.
Neither gains `--out` in P2, so neither has a way to be written owner-only.

**The exclusion is by the spec's own text, not by oversight.** §6a rules
`decode`'s stdout a *"labelled multi-field report"* explicitly out of scope, and
§6b's `--out` is *"write the ARTIFACT to a file"* — a three-line labelled report
is not one. `combine` and `derive` are the same shape. So P2 scopes `--out` to
`encode`, `split` and `repair`, the three verbs whose stdout CARRIES a canonical
`ms1` or share string. **Corrected 2026-08-27 in the R0 round-0 fold of the P2
plan (its I-5):** this entry originally said their stdout *IS* the artifact, and
on `repair` that is false — `ms repair --ms1 <an ms1 with one induced error>`
exits **4** and prints two `#`-prefixed report lines before the corrected `ms1`.
The plan now rules that `repair --out` receives the artifact line alone and the
report stays on stdout; the exclusion of `decode` and `combine` recorded here is
unaffected.

**The inconsistency is nonetheless the hazard**, in the same way F-275's was for
`mt`: an operator who learns that `ms encode --out` writes 0600 will believe
`ms decode` is protected too. This is filed with F-281 because both are the
same question — how much of `ms`'s stdout the cycle is entitled to protect —
and answering them separately is how one gets answered twice.

### F-286 — `plan-cite-check.sh` strips a leading dot from a path, so every workflow-directory citation is a false DANGLING (repo: **mnemonic-engrave**; owning phase: **ownerless residue**) `#tooling` `#gate`

**Found 2026-08-27** writing the P2 plan, which needed to cite a CI workflow.

A citation whose path begins with a dot has the dot removed before lookup, so
the gate searches for a path that exists under no root and reports
*"no such file under any root"*. **Reproduced with a control:** a workflow file
that exists in **this** repo, anchored at its line 1, is reported DANGLING. The
probe string is deliberately not quoted in this entry, because quoting it makes
the containing document fail the gate — which happened once while drafting the
P2 plan, and is the same "writing it out re-creates it" shape
`SPEC_constellation_cli_uniformity.md` §8a records for section sigils.

It is a **false** DANGLING, and the script's own header argues that class is
worse than no gate, because it teaches a reader to skim the output — the
argument that added `descriptor-mnemonic` and the toolkit to `ROOTS`.

**Workaround in use:** name the workflow and its line number in prose, with no
`path:line` punctuation. **Fix:** allow a leading dot in the path pattern.
Blocks nothing; it costs one sentence per citation until then.

### F-297 — a new `ROOTS` entry can silently absorb an already-broken bare citation from a DIFFERENT repo, turning a loud DANGLING into a silent wrong-file `ok` (repo: **mnemonic-engrave**; owning phase: **ownerless residue**) `#tooling` `#gate`

**Found 2026-08-27**, fixing F-286/F-296/the `.tsv` gap, by diffing
`plan-cite-check.sh`'s full `design/*.md` corpus output before and after —
required by the fix brief to confirm no *new* dangling citations, done wider
than the three assigned plans as due diligence.

Adding `mnemonic-gui` to `ROOTS` (F-296) was checked for collisions the way
the script already knows how to detect them: does any relative path exist
under two roots at once. It does not — the four names that DO recur
(`Cargo.toml`, `CLAUDE.md`, `README.md`, `CHANGELOG.md`) were already
ambiguous across 5-7 other roots before this addition, so a 6th-8th hit
changes no citation's classification.

**That check is not the whole hazard.** `mnemonic-gui` also carries two
bare, TOP-LEVEL files no other root has at that exact relative path:
`FOLLOWUPS.md` (1174 lines) and `src/lib.rs` (21 lines) — most siblings keep
these under `design/FOLLOWUPS.md` and `crates/*/src/lib.rs`. The `AMBIGUOUS`
check only fires on a 2+-root collision *at citation time*; it cannot see a
citation that is *incomplete* (missing its true directory) and was
previously DANGLING for the honest reason "no root has this bare path" —
adding a root that coincidentally does have it flips that citation from a
loud, correctly-negative DANGLING to a silent `ok` pointing at a wholly
unrelated file in a different repo.

**Measured live, not hypothetical:** `design/CONTINUITY_2026-08-07.md:86`
cites the bare string `FOLLOWUPS.md` line 59 (missing its `design/` prefix —
already a stale/incomplete citation before this session; written with a
space above rather than a colon, deliberately, so quoting it here does not
itself re-trigger the gate — the same care F-286's own entry took). Before
F-296 it reported DANGLING; after, it reports `ok` against
`mnemonic-gui/FOLLOWUPS.md:59`, an entry about `passphrasePlateHook` residue
that has nothing to do with the slip39 KDF estimate the CONTINUITY doc is
discussing. **Not fixed here** — correcting the prefix does not make the
citation correct either, because `mnemonic-engrave/design/FOLLOWUPS.md:59`
in *this* repo is a different followup again (the line has drifted, the
F-279 blind spot); fixing it for real needs someone who knows which
followup entry the 2026-08-07 estimate correction actually meant, which is
content archaeology outside a script fix.

Two more citations sit on the identical, currently-dormant version of this
same landmine: `design/DESIGN_b2b_residency_zeroing.md:850` → the bare
string `FOLLOWUPS.md` line 1762, and
`mnemonic-engrave/design/FOLLOWUPS.md:4387` and `:4414` (this entry) → the
bare string `src/lib.rs` line 503. Both still correctly report DANGLING
today, purely because `mnemonic-gui`'s `FOLLOWUPS.md` and `src/lib.rs` are
shorter than the cited line numbers — coincidence, not a property the gate
enforces. A routine edit to either `mnemonic-gui` file crosses that line
count and both become the same silent wrong-file `ok`, with no signal to
anyone.

**Not a reason to withhold F-296** — the three actively-maintained P-plans
this cycle's fix was scoped to show zero citation-count change from adding
`mnemonic-gui`, and the live instance found is a pre-existing, already-decayed
citation in a historical continuity doc, not an active plan. Recorded because
the class recurs on every future `ROOTS` addition and the script's own
"NOT covered" block does not yet say so. **Fix, not built here (a design
decision, not a patch):** either (a) require repo-qualification
(`mnemonic-gui/FOLLOWUPS.md:59`) for bare filenames that are "generic" —
same shape as the existing repo-qualified-prefix mechanism, extended to
single-hit cases, not just multi-hit ones — or (b) accept the risk and add
one more "NOT covered" line naming it explicitly, which is cheaper and
matches this gate's existing philosophy of stating blind spots rather than
chasing all of them.
### F-291 — `mk`'s invalid-artifact 2 and its repair-uncorrectable 2 are the SAME `exit_code()` arm, so §6f's "2 → 1" as written also moves the repair code (repo: **mnemonic-key**; owning phase: **P3** — the plan's exit-code entry builds it) `#mk` `#spec` `#exit-codes`

**Found 2026-08-27** while writing the P3 plan, by reading the mapping instead
of the ruling.

`CliError::exit_code` (`crates/mk-cli/src/error.rs:108`) sends
`Codec(_)` and `MdCodec(_)` to **2** through one match arm. Both of §6f's
distinct table columns come out of it:

```
mk decode <garbage>                 -> 2      # "invalid artifact"
mk repair <BCH-uncorrectable card>  -> 2      # "repair uncorrectable"
```

§6f's table gives `md`, `ms` and `mnemonic` a **repair-uncorrectable of 2** and
an **invalid artifact of 1**. `md` gets that split by an explicit bypass whose
own comment says why (`crates/md-cli/src/cmd/repair.rs:109`): it returns
`Ok(2)` at `crates/md-cli/src/cmd/repair.rs:124`, *"bypassing the
`CliError::Codec → 1` default route so the repair exit-code contract is
honored."* `mk` has no such bypass.

**So the ruling as written closes one uniformity defect by opening another.**
The obvious one-line edit — change the arm to 1 — moves `mk repair`'s
uncorrectable to 1 and breaks a parity three other CLIs hold.

**And the suite would not notice, in either direction.** A histogram of
`.code(N)` across `crates/mk-cli/tests` gives 12 sites: four 0, four **2**,
three 5, one 64. All four 2s are in `crates/mk-cli/tests/cli_mk1_repair_reverify.rs`
(`:178`, `:194`, `:239`, `:259`) and every one pins `SetReassemblyMismatch`
(`crates/mk-cli/src/cmd/repair.rs:380`), the funds fix — which also exits 2 and
must **stay** 2. **Zero tests pin the invalid-artifact 2 by exit code.**

**The fix is `md`'s mechanism, ported**: give `mk repair` the explicit `Ok(2)`
bypass first, then move the shared arm to 1. Doing it in the other order leaves
a window where the repair contract is wrong.

**This is a defect in a GREEN spec's normative ruling, not only in code**, which
is why it is filed as well as built. §6f calls `mk`'s 2 *"the only code this
cycle changes"*; it is one of three codes that arm produces.

### F-292 — `mnemonic`'s argv-secret surface is 48 call sites across 20 files naming 11 material shapes, against the 5 channels §7's row names (repo: **mnemonic-toolkit**; owning phase: **P3** — the plan's refusal entry builds it) `#mnemonic-toolkit` `#security` `#spec`

**Measured 2026-08-27** for the P3 plan, with the spec's own two commands
re-run and then narrowed to source.

```
git ls-files '*.rs' | xargs grep -l 'secret_in_argv_warning' | wc -l     # 26 files
git ls-files '*.rs' | xargs grep -c 'secret_in_argv_warning' \
  | grep -v ':0' | awk -F: '{s+=$2} END{print s}'                        # 86 references
```

Both reproduce §7's figures exactly. Narrowed to `crates/*/src/`: **21 files**
carry it — `secret_advisory.rs` holds the definition
(`crates/mnemonic-toolkit/src/secret_advisory.rs:40`) and the other **20**
(nineteen `cmd/` modules plus `repair.rs`) hold **48 call sites**.

The distinct argv-material shapes those sites warn about are **eleven**:
`--from <node>=`, `--slot @N.phrase=`, `--share <node>=`, `--passphrase`,
`--bip38-passphrase`, `--decrypt-password`, `--phrase`, `--ms1`, `--secret`,
`--digits`, and a bare **positional `ms1`**.

§7 already rules that *"the five named channels are ASSERTIONS in P3's gate,
not the sweep boundary: the boundary is the predicate."* This entry records
what the boundary measures to, so the row cannot be satisfied by refusing five
sites out of forty-eight.

**The predicate is reachable before clap, which is what makes it usable as the
boundary.** `NodeType::is_argv_secret_bearing`
(`crates/mnemonic-toolkit/src/cmd/convert.rs:117`) is mirrored by
`secret_taxonomy::SECRET_NODE_TYPES_ARGV`
(`crates/mnemonic-toolkit/src/secret_taxonomy.rs:95`), a `pub const &[&str]` of
nine tokens. Matching a flag name in raw argv and splitting at `=` is string
work; no parse is required to reach the decision, which is the whole of §6d's
C-4. **Every one of the 48 existing sites is post-parse**, so the refusal is new
code at a new place keyed off an old predicate — not a rewrite of the advisory
layer.

### F-293 — the argv advisory prints a flag name with a trailing space, at **four** call sites on **two** different flags (repo: **mnemonic-toolkit**; owning phase: **P3** — fixed in passing by the plan's refusal entry) `#mnemonic-toolkit` `#ux` `#shipped`

**Reproduced 2026-08-27** by running the binary, not by reading the source:

```
$ mnemonic electrum-decrypt --ciphertext <base64> --decrypt-password hunter2
warning: secret material on argv (--decrypt-password ) — pipe via ...
```

**CORRECTED 2026-08-27 by the R0-P3 round-0 fold — the original said TWO sites
and it was half the defect.** The first count took the *first* string argument
of each call, but `secret_in_argv_warning`'s first argument is the writer and
the flag name is the **second**
(`crates/mnemonic-toolkit/src/secret_advisory.rs:40`). Re-derived from the
correct position, **four** sites pass a flag name with a space attached, on two
different flags:

```
crates/mnemonic-toolkit/src/cmd/electrum_decrypt.rs:101   "--decrypt-password "
crates/mnemonic-toolkit/src/cmd/import_wallet.rs:507      "--decrypt-password "   <- missed
crates/mnemonic-toolkit/src/cmd/import_wallet.rs:2331     "--decrypt-password "
crates/mnemonic-toolkit/src/cmd/seedqr.rs:157             "--digits "             <- missed, different flag
```

**So the residue is 44, not 46** (48 call sites minus these four). Reproduced by
running the binary: `mnemonic seedqr decode --digits <digits>` prints
`warning: secret material on argv (--digits ) — pipe via ...`. A literal reading
of the original entry leaves half the defect in place, including the one on a
flag the entry never named. Cosmetic, in a security message, in shipped code.

### F-294 — `records::no_records_guard`'s refusal text names `mt encode --qr`, another binary's flag, so no third consumer can adopt it (repo: **mnemonic-engrave**; owning phase: **`mnemonic-io-lib`'s next version, before a sixth consumer**) `#mnemonic-io-lib` `#P3` `#design`

**Found 2026-08-27** while drawing P3's boundary, by reading the text the
function would make `md` print.

The message advises *"pass them on argv, with --in, or on stdin"* and then
explains itself with *"An EMPTY input is what a FAILED upstream command leaves
behind -- `mt encode --qr > rec.txt` writes nothing when it refuses"*. Printing
`mt`'s flag out of `md`'s mouth is a defect nobody would file against the crate
and everybody would file against `md`.

**Both P3 tools already refuse an empty input correctly, in their own words**,
and route it to **different** codes — `md` through `BadArg` to 2, `mk` through
`UsageError` to 64. The crate is right to publish no integer; the callers are
right to keep their own text.

**This is a second instance of F-276's finding**, found by the third consumer
rather than the second, and it points the same way: the crate's *mechanism* is
reusable and its *prose* is `me`- and `mt`-shaped. The fix is to take the
example out of the shared function and let the caller supply it, or to publish
the guard as a predicate and leave all of the wording behind.

### F-295 — `mnemonic bundle` writes 6 non-artifact lines out of 12 to stdout, and §6a's stdout rule does not reach it (repo: **mnemonic-toolkit**; owning phase: **whichever cycle extends §6a past `encode` on the four encoders**) `#mnemonic-toolkit` `#spec` `#deferred`

**Measured 2026-08-27.** `mnemonic bundle --network mainnet --template bip84
--slot "@0.phrase=<a BIP-39 phrase>" --passphrase <pw>` exits 0 and writes 12
lines to stdout, of which 6 are non-artifact: three `# ms1 (entropy,
BCH-checksummed)` / `# mk1 (xpub + origin)` / `# md1 (wallet policy)` comments
and three blank separators. Each artifact line begins with its own HRP, so the
comments are **redundant with the data**.

**Ruled OUT of P3**, and consulted rather than assumed. §6a scopes the stdout
rule by an explicit per-verb table covering `md`/`mk`/`ms`/`mt` only and binds
it to `encode`, a verb `mnemonic` does not have; §2a enumerated `mnemonic`'s
involvement as grouping and argv refusal with no header item; §9a places
`mnemonic` in a tier that never feeds `me sysw pack`, which removes the
packability motive. Stripping them would break a shipped machine-readable
surface — the exact class §6a refused to break for `mk decode`, saying it
*"gets its own phase and its own gate."*

**What would reopen it:** an operator ruling that `bundle` counts as
`mnemonic`'s `encode` for §6a's purposes.

### F-296 — `plan-cite-check.sh` has no root for `mnemonic-gui`, the fourth repo P3 touches (repo: **mnemonic-engrave**; owning phase: **P3**, before the plan's GUI-mirror entry is written) — **DONE 2026-08-27** `#tooling` `#gate` `#P3`

**Found 2026-08-27** by probing the gate with the citations the P3 plan needed,
before writing them.

`ROOTS` carries `seedhammer`, this repo, `descriptor-mnemonic`,
`mnemonic-toolkit`, `mnemonic-key`, `mnemonic-secret` and
`mnemonic-transaction`. **`mnemonic-gui` is absent**, so every citation into it
reports DANGLING and the P3 plan writes its GUI references as prose — the P0
plan's workaround, which the script's own comments describe as not scaling.

**The fix is one line**: add `"/scratch/code/shibboleth/mnemonic-gui"` to
`ROOTS`.

**It is collision-free, measured rather than asserted.** Of the GUI's 504
tracked files, the only paths it shares with any existing root are top-level
`Cargo.toml`, `CHANGELOG.md`, `CLAUDE.md` and `README.md` — every one of which
is **already** ambiguous across the current roots and already reported as such,
so adding the root changes nothing about them. Its `src/schema/` tree collides
with nothing, because no other root has a top-level `src/` at all, checked
across all eight.

**Two smaller blind spots found by the same probe, recorded here rather than
filed separately.** A leading-dot path loses its dot to the citation regex, so a
bare `.github/workflows/ci.yml` dangles as `github/…` while the repo-qualified
form resolves. And `.tsv` is not in the gate's extension list, so the
display-grouping conformance corpus — byte-identical across four repos and
sha256-pinned in three CI configs — is invisible to it.

### F-301 — `me`'s shipped private-channel remedy advises a pipeline that exits 4 and writes nothing, and a source comment asserts it is verified (repo: **mnemonic-engrave**; owning phase: **before P2's sibling-remedy entry**) `#me` `#ms` `#remedy` `#cli-uniformity`

**Found 2026-08-27** by R0 round 0 on the P2 plan (its C-2), by running the line
`me` prints instead of reading it. **Reproduced independently during the fold.**

`me` refuses secret-class material on argv and prints a private-channel example.
Run exactly as emitted, against `master`'s own build:

```
ms encode --phrase - < seed.txt | me sysw pack --out p.bin
  -> rc 4
     me: record 0 (records count from 0) is not a form this container can place:
     not a BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a
     `text:`/`pass:`/`tx:` record.
  -> p.bin does not exist
```

The cause is `ms encode`'s default **grouped** stdout —
`ms10e ntrsq qqqqq …` — which `me sysw pack` cannot classify. The live control:
the same pipeline with `--group-size 0` and `--no-passphrase` exits **0** and
writes a **102-byte** payload at mode 0600.

**A source comment beside the emitted literal asserts the opposite** — it states
that `ms`'s stdin idiom *is verified to pipe into pack*. **Nothing verifies it.**
Measured: `crates/me-cli/tests/` holds **14** `.rs` files with **33**
`Command::new` sites, and **0** of them name an `ms` binary; `ms encode` appears
in `crates/me-cli/src/` exactly twice, in that comment and in the emitted string;
`seed.txt` appears once in the whole crate, inside the emitted string.

**CLASS — this is NOT the secret-handling class and it still gates.** By the
operator ruling of 2026-08-27 a defect whose harm is material becoming visible is
logged rather than blocking. This one's harm is different: the tool reports a
working path it does not have. An operator who is refused, follows the printed
remedy verbatim, and gets exit 4 with no payload has been told something false by
the tool, and the affordance the refusal still names is `--allow-argv-secret` —
the channel it just refused. §6h of the spec exists because this was shipped once
already; this is the second time.

**Why it is filed rather than fixed in the P2 plan.** P2's sibling-remedy entry
rewrites this line to the `--in` form, but that entry cannot land until P2's
ungrouped-stdout work does, and neither has shipped. An operator meets the broken
advice today. The interim repair is `me`-side and small — the advised line names
`--group-size 0`, or the advice moves to a form `me sysw pack` accepts today —
and it must be **gated by a test that RUNS the emitted line**, since the absence
of that test is why this survived. When P2's entry lands, that test is retargeted
rather than written twice.

### F-302 — `ms`'s argv surface leaks through the `=`-joined flag spelling, and a guard gated only on space-joined spellings passes its own gate while leaking (repo: **mnemonic-secret**; owning phase: **P2**, with the argv guard) `#ms` `#cli-uniformity` `#funds-safety`

**Found 2026-08-27** by R0 round 0 on the P2 plan (its C-1). Logged here per the
operator ruling of the same day.

Measured against `mnemonic-secret`'s own build at `7c12f66`, exit codes read
directly:

```
ms encode  --phrase=<the all-abandon 12-word vector>   -> rc 0, prints the ms1
ms encode  --hex=00000000000000000000000000000000      -> rc 0, prints the ms1
ms derive  <ms1> --passphrase=hunter2                  -> rc 0, 2 argv advisories
```

The whole `--flag=value` construction is **one argv token**. Its left half is not
the flag string, so a flag-keyed layer matching exact strings does not see it;
its right half is not a positional, so a value-shape layer scoped to positionals
is not even pointed at it. A cross-product built from the space-joined spellings
alone therefore passes every row it generates while the leak is untouched.

**The donor already solved it and names the case**: `argv_candidates`
(`crates/me-cli/src/main.rs:350`) extends its candidate list with every `=`-split
half at `crates/me-cli/src/main.rs:354`, and the doc comment at
`crates/me-cli/src/main.rs:347` explains that the secret is the right-hand half
of a token like `--in=<ms1>`. The P2 draft had taken three of that function's
four normalisations — trim, case-fold, whole token — and dropped the fourth,
which is the only one that is a **bypass** rather than a formatting variant.

**CLASS — the logged class, by the operator ruling.** The harm is material
becoming visible, so it holds no gate. **It is closed inside P2 anyway**, because
closing it is one normalisation in a list the plan already specifies: the guard
entry now requires `=`-splitting, and the generated gate grew from 56 rows to
**92** (4 value spellings × 14 channels, with both join forms on the 9 flag
channels and one on the 5 positional channels: 9×4×2 + 5×4). That cross-product
was generated and run during the fold rather than extrapolated: **84 of 92 pass
material at exit 0** — 58 silently, 26 with `derive`'s advisory — and **0 of 92**
leak material into stderr today.

**Not covered by the 92, and stated rather than implied**: the `--` end-of-options
form (`ms decode -- <ms1>` exits **0** today) and any shape where the material is
neither a whole token nor an `=`-delimited half. Two shapes were checked and do
**not** exist on `ms`: abbreviated long flags (`ms encode --phr <seed>` exits
**64**, `error: unexpected argument '--phr' found`) and short aliases carrying
material (measured across all eight material verbs: only `-h`, plus `split`'s
`-k`/`-n`).

### F-303 — after P2, `ms derive` from a phrase PLUS a passphrase has no one-command private form (repo: **mnemonic-secret**; owning phase: **a later cycle**, with the remaining argv work) `#ms` `#cli-uniformity` `#funds-safety`

**Found 2026-08-27** by R0 round 0 on the P2 plan (its I-3).

`ms derive --phrase <seed> --passphrase <pass>` exits 0 today, with two argv
advisories. P2's refusal closes both channels, and the private alternatives do
not compose:

- `--phrase -` with `--passphrase-stdin` hits `ms`'s existing contention refusal
  — *one stdin per invocation* — measured at rc 1.
- `--in` on `derive` reads an **`ms1`**, not a phrase. Measured:
  `ms derive - < <a file holding a BIP-39 phrase>` exits **1**,
  `error: string length 82 not in v0.1 set [50, 56, 62, 69, 75]`.

**The round-0 report concluded that NO private form remains. That is too strong,
and the counter was measured rather than argued.** A two-command private route
exists after P2, because `ms` can convert the phrase into the kind `--in` reads:

```
ms encode --in seed.txt --out card.ms1
ms derive --in card.ms1 --passphrase-stdin < pass.txt
```

Reproduced today in the closest form the binary supports — `--in`/`--out` do not
exist yet, so the phrase came in on stdin and the card went to a variable:
`ms encode --phrase - --group-size 0 < seed.txt` → rc 0, and
`ms derive "$CARD" --passphrase-stdin < pass.txt` → rc 0,
`master_fingerprint: ca2c62d2`.

**So what P2 owes is the route being written down and asserted, and it does that**
— the freed-stdin entry gates the two commands end-to-end against the fingerprint
the one-command argv form produces today, and requires the refusal's own text to
name the route. An operator who cannot find it reaches for
`--allow-argv-secret`, which is the exposure the phase exists to close.

**What is deferred here is the one-command form**: a `--passphrase-file`, or a
phrase-shaped `--in` on `derive`. Either is a new channel, and §7's P2 row
enumerates P2's content and includes none.

### F-304 — `ms encode`'s standing stdout advisory recommends a redirect that lands at 0644, while P2 adds an `--out` that gives 0600 (repo: **mnemonic-secret** + **mnemonic-toolkit**; owning phase: **P3**) `#ms` `#cli-uniformity` `#remedy` `#funds-safety`

**Found 2026-08-27** by R0 round 0 on the P2 plan (its I-8).

`ms encode` prints, on **every** invocation, a warning that stdout carries
private key material and should be redirected or encrypted, giving a plain `>`
redirect as its first example. Measured under the default umask 022:

```
ms encode --phrase - < seed.txt > w.txt   -> rc 0
stat -c '%a' w.txt                        -> 644
```

— 0644, holding an `ms1` that decodes to the seed. After P2's private-write
entry, `ms encode --out w.txt` gives **0600** for the same artifact. So the
tool's own standing advice points the operator at the weaker of two in-tool
channels, and P2 is the phase that creates the better one.

This is §6h's rule — remedy text names the channels that exist — applied to
`ms`'s own text rather than the sibling's. It is **not** F-281 (whether to *gate*
a world-readable stdout) and **not** F-285 (verbs that get no `--out` at all).

**Why P2 does not act on it.** `fn byte_parity_advisory_lines`
(`crates/ms-cli/tests/cli_output_class.rs:56`) pins all three advisory lines
byte-for-byte against `mnemonic-toolkit`'s, so any rewording is joint cross-repo
work, and under the Rust-primary rule the wording is not `ms`'s to change
unilaterally. P3 already carries `mnemonic`'s own argv and advisory work, which
is the cheapest place to reason about both sides at once.

**CLASS — the logged class by the operator ruling** (the harm is material landing
world-readable), but the round-0 finding's actual ask was that the gap have an
**owning phase**, and this entry supplies it. Nothing was downgraded to avoid
work.
**DONE 2026-08-27.** `ROOTS` gained `"/scratch/code/shibboleth/mnemonic-gui"`,
the citation regex now keeps a leading dot (both the `./` prefix and a hidden
top-level directory), and `.tsv` is in the extension list. Re-probed with the
forms the P3 plan needed — the GUI's `src/schema/…` and `tests/…`, its
`pinned-upstream.toml`, and the display-grouping corpus — **8 of 8 resolved, 0
dangling, 0 ambiguous**, and the P3 plan now cites all four repos normally
instead of writing the fourth as prose.

**One residue is deliberately NOT fixed and is documented instead**: a bare
top-level `Cargo.toml` citation is AMBIGUOUS across seven roots. That is the
gate working as designed — it reports the ambiguity loudly rather than guessing
a repo — so GUI manifest citations are written repo-qualified.

### F-311 — `mk encode --keys` silently accepts a key file carrying the same BIP-380 record twice, at exit 0 (repo: **mnemonic-key**; owning phase: **NOT P3** — a `mk` admission ruling, outside P3's row) `#mk` `#admission` `#silent-accept`

**Found 2026-08-27** by the R0-P3 round-0 fold, while testing the P3 plan's
justification for deleting the blank line `mk encode` prints between cards —
*"the card boundary is recoverable from each card's own chunk header"*. It is
not, and the counterexample is an input `mk` accepts without complaint.

**Reproduced**, with a key file holding one record twice:

```
$ mk encode --keys dup.keys --policy-id-stub 5b48af35 ; echo $?
mk1qp d8cwp qqsq4 ... mfrjw 2
mk1qp d8cwp p806l ... c36tw

mk1qp d8cwp qqsq4 ... mfrjw 2
mk1qp d8cwp p806l ... c36tw
0
```

Two **byte-identical** cards sharing one chunk-set-id (`d8cwp`), separated by
nothing except the blank line. Their headers are the same header, so once the
blank line is gone the boundary is not recoverable from them at all — and the
blank line was the only signal that a duplicate cosigner record had been
accepted in the first place.

**Deleting the blank line is still correct** under §6a of the CLI-uniformity
spec, which admits the artifact and nothing else; the P3 plan withdraws the
false justification and keeps the work. **What is filed here is the layer
underneath**: `mk encode --keys` has no duplicate-record admission check. A
2-of-2 authored with a copy-paste error produces a bundle that looks like two
cosigners and is one, at exit 0, with no warning.

**Not scheduled into P3** — P3's row is channels, presentation and one exit
code, and adding an admission rule to `mk encode` is a ruling this phase does
not get to make.

### F-312 — `mnemonic-gui`'s drift gate carries a stale comment naming a `v0.75.0` pin that is really `v0.97.0`, and it propagated a false premise into a plan (repo: **mnemonic-gui**; owning phase: **P3**, with the toolkit release) `#mnemonic-gui` `#stale-comment` `#gate`

**Found 2026-08-27** by the R0-P3 round-0 review, and the reason it is filed
rather than fixed silently is what it cost.

`tests/schema_mirror_defaults_drift.rs:36` says the CI job *"points
`MNEMONIC_BIN` at the pinned v0.75.0 binary"*. Both real pins say otherwise:
`pinned-upstream.toml:22` and the load-bearing dependency pin at
`mnemonic-gui/Cargo.toml:76` are both `mnemonic-toolkit-v0.97.0`, and the
measured toolkit is `mnemonic 0.97.0`. **The pin is exactly current.**

**The cost.** The first draft of the P3 plan read that comment and concluded the
drift gate's binary was *"far behind the CLI's current version"*, and from there
that flipping the `--group-size` default would produce *"zero GUI test
failures"*. Measured by running it — the four GUI default sites flipped, pin
unchanged — the gate produces **four drift violations** and reds. The gate is a
**lockstep** gate; the plan believed it was blind. That was one of the four
Importants of the round.

**This is the "comments outlive their conditions" class**, and the general
lesson is in the specific: a reviewer who greps for the *mechanism* (the pin
files) finds the truth, and a reader who trusts the *claim* (the comment) does
not.

**Fix:** delete or correct the version in that comment when the toolkit release
moves the pin, which the P3 plan's release entry does anyway.

### F-313 — a plan whose definition of green is `fmt + clippy + nextest + conformance` is structurally blind to `mnemonic-toolkit`'s 62 byte-compared doc transcripts (repo: **mnemonic-engrave**; owning phase: **ownerless residue** — a process item) `#process` `#gates` `#docs`

**Found 2026-08-27** by the R0-P3 round-0 review, as the general shape behind a
specific Important.

`mnemonic-toolkit/docs/manual/transcripts/` holds **169 tracked files, 62 of
them `.cmd`** — command scripts replayed against the real installed binaries and
**byte-compared against golden `.out`/`.err`** by three workflows
(`quickstart.yml`, `manual-gui.yml`, `technical-manual.yml`). **None of them is
a `cargo` invocation**, so a closure list naming only the test runner cannot see
any of them.

The P3 plan's first draft defined green exactly that way and consequently placed
**23 goldens it invalidates** — 19 needing their commands *rewritten* because a
new refusal makes them fail, 4 needing regeneration after a default flip — in no
entry, gate, closure condition or follow-up. P3 is the **second** cycle to
define green with a runner-shaped list.

**The generalisable fix is not to remember harder.** Name the doc-transcript
workflows in the standard closure list that plans in this constellation copy
from, so the surface is inherited rather than rediscovered. A plan that edits a
CLI's output or its refusals is exactly the plan that breaks documented worked
examples, and that is precisely when nobody is looking at `docs/`.

### F-320 — `mt`'s new git dependency drags `third_party/seedhammer` onto every cold CI runner, because cargo fetches a git dep's SUBMODULES (repo: **mnemonic-transaction**; owning phase: **ownerless residue** — a CI-cost item) `#ci` `#deps`

**Found 2026-08-27** while executing P1 row 7's own gate — a cold-`CARGO_HOME`
resolve of the new dependency — rather than by reading anything.

`crates/mt-cli/Cargo.toml` now takes `mnemonic-io-lib` from
`bg002h/mnemonic-engrave` by rev. Cargo clones a git dependency's submodules
**recursively**, and this repository has one:

```
    Updating crates.io index
    Updating git repository `https://github.com/bg002h/mnemonic-engrave`
    Updating git submodule `https://github.com/seedhammer/seedhammer.git`
```

Measured on an empty `CARGO_HOME`: 26 s wall, and 52 MB left in `git/`. So
`mt`'s CI now pays for a submodule it will never compile, on every cold runner,
and acquires a second upstream host as a build-time dependency —
`github.com/seedhammer/seedhammer` being reachable is now a condition for `mt`
to build.

**Not a defect and not urgent.** It is stated because it is *invisible* from
`mt`'s side: nothing in `mnemonic-transaction` mentions seedhammer, and the
first person to debug a slow or failing CI fetch there will have no reason to
look in this repository for the cause.

**Options, none taken now:** publish `mnemonic-io-lib` to crates.io (F-271
records the publish as authorised and its pre-flight as unrun), which removes
the git dep entirely; or split the library into its own repository. Cargo has no
per-dependency "do not fetch submodules" switch, so there is no cheap local fix.

### F-321 — this repo's copy of `design/SPEC_mt_v0_1.md` is now stale against `mnemonic-transaction`'s, in four places P1 changed (repo: **mnemonic-engrave**; owning phase: **P1's merge**, with F-279's re-anchoring pass) `#docs` `#drift`

**Found 2026-08-27** during P1 rows 8–13. The spec exists in **both**
repositories, and the implementation edited the `mnemonic-transaction` copy
because that is where the code and its tests are. This copy was deliberately not
edited from an implementation worktree, so the two now differ:

| § | what this copy still says | what shipped |
| --- | --- | --- |
| 8.2f | *"The purge command is **specific to the operator's shell**, detected from `$SHELL`"*, and shows `history -d 512 && fc -W` | every shell's recipe is printed, none is detected, and `history -d` does not delete on zsh 5.9.2 |
| 8.2g | *"is mode 0644 — readable by every user on this machine"* | **already false before P1** — F-252 removed that reachability claim from the code on 2026-08-25 and never from this paragraph; now also derived from the mode |
| 8.2h | *"THE REMEDIES ARE THE SHELL'S, because `mt encode` HAS NO `--out`"*, citing §3b | §6b gave `mt` `--out`, and the fold checked the citation: §3b does not say it |
| 8.2h | says nothing about `mt decode` | F-275: `decode` warns at the same destination and still exits 0 |

The §8.2g row is the one worth noticing: **a spec paragraph stayed false for two
days after the code was corrected, and no gate could see it**, because nothing
compares the two copies or compares either to the binary's output.

**Fix:** take the `mnemonic-transaction` copy as authoritative when P1 merges —
`git diff` the two files rather than re-reading them — and decide then whether
one repo should stop carrying a copy at all.

### F-322 — a mutation gate that restores the SOURCE leaves a MUTATED BINARY, and the next thing that runs measures a program with a check deleted (repo: **both**; owning phase: **ownerless residue** — fixed in `mnemonic-transaction`, unfixed here) `#gates` `#process`

**Found 2026-08-27 by walking into it**, while measuring P1 row 9's stdout-mode
differential.

`mnemonic-transaction/scripts/mutate-refusals.sh` neuters a named check, runs
one test, restores `src/` from a byte copy and touches it. It never rebuilt — so
the **last** entry's `cargo nextest` left `target/debug/mt` linked from the
mutated tree, the working tree was clean, `git status` agreed, and
`./target/debug/mt` was a program with a refusal deleted. The last entry in
`refusals.toml` names `world_readable_stdout_guard`, so straight after a GREEN
gate run:

```
mt encode --in <finalized psbt> > <a 0644 file>   ->  rc 0, 796 bytes, no refusal
```

which reads exactly like a shipped §8.2h defect on a tree whose whole suite is
green. About fifteen minutes went into looking for a bug in correct code.

**It also concealed a second, real RED.** `mnemonic-transaction`'s CI order is
refusal-coverage → refusal-mutation → **journeys**, so `scripts/journeys.sh` had
been running against that mutated binary — and journey A's very first line is
`mt encode … >"$WORK/a.out"`, which under the default umask 022 is a 0644
destination that §8.2h correctly refuses. **The journeys gate had never once run
against an unmutated binary.** Reproduced at the previous commit by rebuilding
and re-running: exit 1 at the same line. Both are fixed there — `cleanup`
rebuilds, and `journeys.sh` sets `umask 077`, which is the first remedy `mt`'s
own refusal offers.

**Why this is filed rather than closed:** the class is not specific to that
script. `mnemonic-engrave/scripts/mutation-run.py` restores from a file copy the
same way and likewise never rebuilds. It is **not** wired into CI here, so
nothing downstream of it is currently being fooled — but anyone running it and
then invoking `target/debug/me` is measuring a mutated binary, silently.

**The general rule, worth having:** a gate that mutates a tree must restore the
**artifact**, not only the source. Restoring source and touching it makes the
*next build* correct and leaves the *current binary* wrong, and the gap between
those two is exactly where a hand measurement lands.


**CONTROLLER ASSESSMENT 2026-08-27, measured rather than assumed: LATENT here,
not live.** `scripts/mutation-run.py` in this repo never rebuilds — zero hits for
`cargo build` across the file — so it does leave a stale binary exactly as the
sibling's shell script did. What differs is what runs next. In `mt` the order was
coverage → mutation → journeys, and journeys executes a **prebuilt binary**, so
the stale artifact was consumed and a real RED stayed hidden. This repo's CI runs
`cargo test --locked` and the Go suites, all of which rebuild, and it has no
prebuilt-binary gate after mutation at all. So the defect is present and
currently unreachable.

**That makes it a trap rather than a bug**: the day someone adds a journey script
or any gate that runs `target/debug/me` after the mutation step, it becomes the
`mt` defect with no warning. The fix is the same one-liner — rebuild after
restoring — and it costs nothing to apply before that day rather than after.

### F-323 — `mt decode --json --quiet` emits no JSON at all: `--quiet` silently disables `--json` (repo: **mnemonic-transaction**; owning phase: **P2**, which owns `--json`) `#cli` `#json`

**Found 2026-08-27** while wiring F-275's warning into `decode`'s `--json` path.

`decode`'s whole report block, the JSON document included, sits inside
`if !args.quiet`. So:

```
mt decode --json --quiet --in typed.txt   ->  rc 0, stdout 445 bytes of hex,
                                              stderr carries NO json document
```

A caller who asks for machine-readable output and also passes `--quiet` gets
exit 0 and nothing to parse. **This is the defect class `mt` has already ruled
on in the other direction**: `json_unsupported_guard` exists precisely because
*"a flag that cannot work must REFUSE rather than sit inert"* — `--json` was
once wired into `inspect` alone and parsed-and-did-nothing on three verbs.

Pre-existing, and untouched by P1 — measured at `a4cdefa` as well, where the
`if !args.quiet` wrapper is already there. P1 only made it visible, because the
F-275 warning is now the *only* thing on stderr in that combination (warnings
are never suppressed, so `--quiet` does not silence it).

**Not fixed here** because the remedy is a ruling, not a repair: either
`--json` wins over `--quiet` (the report is data, not chatter), or the pair is
refused. §6c places `--json` in P2, so P2 should decide. **`--json` is
explicitly out of scope for the current cycle** per `§6b`'s own note, which is
why this is filed rather than argued.

### F-324 — pinning `mnemonic-io-lib` by git rev breaks `ms`'s tag-time reproducible musl build, and the only fix is in **another repo's** shared reusable workflow (repo: **mnemonic-toolkit** + **mnemonic-secret**; owning phase: **before the next `ms-cli-v*` tag** — non-deferrable past it) `#ci` `#repro` `#deps` `#cross-repo`

**Found 2026-08-27 by P2's implementation**, executing row 4 ("PIN THE CRATE").
The plan does not mention vendoring, `vendor-freshness`, or the reusable repro
workflow, and §5's enumeration of *"`ms`'s WHOLE validation surface"* omits all
three — so this is a plan defect as well as a CI one.

**What the pin does.** `mnemonic-secret` commits a 101 MB `vendor/` tree and
builds its release binaries `--locked --offline` from it. Pinning
`mnemonic-io-lib` by GitHub rev puts the FIRST `source = "git+…"` line ever into
its `Cargo.lock`. `source.crates-io` does not serve that key.

**Measured, with an EMPTY `CARGO_HOME` so the cargo git cache could not supply
the dependency** — the isolation is load-bearing and its absence produced a
false GREEN on the first attempt:

| `--config` form | `cargo build --locked --offline -p ms-cli` |
| --- | --- |
| three-block (crates-io + the mnemonic-engrave git source + vendored-sources) | **rc 0** |
| two-block (crates-io + vendored-sources) — what every release step used | **rc 101**, `failed to load source for dependency mnemonic-io-lib` |

With a POPULATED `CARGO_HOME` the two-block form also exits 0, resolving from
`~/.cargo/git`. A check run without the empty-home isolation proves nothing.

**Three sites, and only two of them are fixable from `mnemonic-secret`.** Both
were fixed in P2's row-4 commit:

- `ci/repro/vendor-freshness.sh` — a **push- and PR-triggered** gate. It failed
  CLOSED the moment the pin landed, exactly as its own comment predicted, and
  named its own fix. Converted to the three-block form, with the rev DERIVED
  from `Cargo.lock` (mirroring the toolkit's `MINISCRIPT_REV` handling) so
  moving the pin forward cannot leave the script on the old rev. Negative
  control run: with `vendor/mnemonic-io-lib/` moved aside it exits 1.
- `.github/workflows/man-release.yml`'s `musl-binaries` job, both legs
  (aarch64 `cross build`, x86_64 re-homed `docker run`). Converted likewise.

**THE ONE THAT CANNOT BE FIXED HERE.** The same workflow's `repro` job calls
`bg002h/mnemonic-toolkit/.github/workflows/reproducible-musl-build.yml@6e37b18e50f9f857e439db1ebe2748fc91a54612`.
Read at that SHA: its **only** git-source knob is an input named
`miniscript_rev`, and the three `--config` lines it builds hard-code
`https://github.com/rust-bitcoin/rust-miniscript` as the source URL. There is no
input by which a caller can declare a different git source. `ms` passes
`miniscript_rev: ""`, which selects the two-block form — the row that exits 101
above.

**So `ms` cannot cut a release tag until a change lands in `mnemonic-toolkit`.**
`man-release.yml` triggers only on `ms-cli-v*` tags and `workflow_dispatch`, so
nothing on the push path is red; the breakage is deferred to the next release.

**The job is left CALLED rather than disabled or `if:`-guarded off.** A skipped
gate prints ok and exit 0, which is how an unrun gate passes for months.

**What the fix looks like.** A `git_source` / `git_rev` pair of inputs on the
toolkit's reusable workflow (generalising the existing miniscript stanza rather
than adding a second one), then re-pin the `uses:` SHA in `ms`'s
`man-release.yml`. That workflow is shared by **md, mk and mt**, so the change
needs its own review in `mnemonic-toolkit` under the Rust-primary rule.

**It must then be EXERCISED, not merely edited** — one `workflow_dispatch` run
of `man-release.yml` after the re-pin. A gate that has never executed is a
hypothesis, not a gate, and this one has been demonstrated to fail.

**Two alternatives were considered and both are closed by the frozen plan.**
`path =` is forbidden (`freebsd-compile-gate` and both `musl-check` targets
build from a clean checkout on foreign targets, so a path dep out of the
workspace fails there first), and publishing `mnemonic-io-lib` to crates.io is
explicitly out of scope for P2 (F-271 records the publish as authorised and its
pre-flight as unrun). Publishing would dissolve this entry entirely, which is
worth knowing when F-271 is next picked up.

### F-360 — `plan-table-check.sh` only checks rows AFTER the separator, so a malformed table HEADER passes (repo: **mnemonic-engrave**; owning phase: **the gate-hardening residue**) `#tooling` `#gate`

**Found 2026-08-27** by the P2 plan's fold, which hit it while writing a table
and worked around it in the plan rather than in the gate. Recording it so the
blind spot does not have to be re-discovered by the next author who trips on it.

The gate walks a table's rows and checks each against the header's column count.
It establishes that width **from** the header, and separately locates the
separator row — so a header carrying an escaped pipe, or one whose column count
disagrees with its own separator, is never itself checked. Every data row is
then measured against a width that is already wrong, and they all agree with it,
so the table reports clean.

**The same fold hit a second, unrelated one**: two journey paths written without
their directory prefix resolved nowhere, and were fixed in the plan rather than
in the citation gate.

**Neither was fixed in the gate.** The two plans are clean; the gates still have
the holes. That is the wrong side of the ledger to leave them on — a workaround
in one document does nothing for the next one, and the whole argument for these
gates is that they catch what reading cannot.

**What closing it looks like**, and it is small: check the header row against its
own separator before measuring anything else, and mutation-test it both ways —
a deliberately malformed header must go RED, and every existing plan must stay
green. That second half matters as much as the first: a gate that starts
reporting findings on documents already reviewed clean will be ignored within a
day.

**Related, and the reason this is worth doing rather than tolerating:** the
citation gate's own defects (F-286, F-296, and a silently skipped extension)
were each found by an author working around them, not by anyone auditing the
gate. A gate's blind spots surface only when someone hits one — so each hit is
the whole signal, and working around it in the document discards it.

### F-331 — `md encode --policy-id-fingerprint` still writes a NON-ARTIFACT line to `encode`'s stdout, which §6a forbids and P3's closure condition 6 asserts is gone (repo: **descriptor-mnemonic**; owning phase: **whichever cycle rules whether §6a binds an opt-in diagnostic flag**) `#spec` `#stdout` `#pipeline`

**Found 2026-08-27 by P3's `md` branch**, building the md ungrouping. The plan's
§1.1 inventory says *"One emission site on stdout — `crates/md-cli/src/cmd/encode.rs`,
`println!("chunk-set-id: 0x{csid:05x}")`"*. There is a **second**, and no row of
the plan touches it.

**Reproduced, and it is not theoretical:**

```
$ md encode 'wpkh(@0/<0;1>/*)' --policy-id-fingerprint
md1yqpqqxqq8xtwhw4xwn4qh
policy-id-fingerprint: 0x3ace1082

$ me sysw pack --in <that stdout> --out payload.bin
rc 4 — "record 1 ... is not a form this container can place"
```

So the exact failure P3 exists to remove survives behind one opt-in flag. §5's
closure condition 6 — *"No line of `md`'s or `mk`'s stdout on `encode` is
anything but the artifact"* — is **false** for this invocation.

**Not fixed by the md branch, deliberately.** No row names it, it is opt-in
rather than the default path, and `me sysw pack` never passes it — so the phase's
own gate is genuinely green. Moving the line to stderr is a change to a shipped
machine-readable surface, which is the class §6a explicitly refused to make for
`mk decode` and said *"gets its own phase and its own gate"*. That decision is
above an implementer.

**The two candidate rulings**, so whoever picks this up is not starting cold:
either §6a binds every line `encode` can be made to print (and the flag's output
moves to stderr beside the engraving card, where the chunk-set-id now lives), or
§6a binds the DEFAULT emission only (and the condition-6 wording needs the
qualifier it currently lacks). `mk` has no counterpart flag, so this is `md`-only.

### F-332 — `md`'s TERMINAL-write decline is asserted only indirectly; the pty half is unbuilt (repo: **descriptor-mnemonic**; owning phase: **the test-infra residue**) `#test` `#gap`

**Found 2026-08-27 by P3's `md` branch** while writing row 20 (*the decline,
asserted*). The row's gate reads *"`md`, `mk` and `mnemonic` each still write to
a **terminal** without refusing, so an adoption of `exit::write_block` that
imported `me`'s terminal gate goes RED."*

**What was built**, in `crates/md-cli/tests/cli_p3_decline.rs`:

- a source scan asserting the only path rooted at `mnemonic_io_lib::` anywhere
  under `src/` is `write::write_private` — mutation-verified (adding a
  `mnemonic_io_lib::exit::WriteBlock` return type reds it);
- `md encode` into a shell-created **mode-0644** file exits 0 and does not
  tighten it — the world-readable arm of the same gate.

**What was NOT built:** an actual terminal. Asserting the `Destination::Terminal`
arm needs a pty, and this suite runs on ubuntu / macos / windows plus an
x86_64-musl leg and an aarch64-musl leg under QEMU. `openpty` lives in `libutil`;
`posix_openpt` needs `/dev/pts` inside `cross`'s container. A test that silently
does nothing on five of seven CI legs is worse than a named gap.

**Why this is Minor rather than Important:** the source scan is the stronger of
the two checks. `write_block` cannot refuse a terminal in a binary that never
reaches `write_block`, and the scan fails closed (it asserts it found >10 files
AND at least one crate path, so an empty scan cannot report clean).

**What closing it looks like:** a `#[cfg(target_os = "linux")]` pty helper using
`posix_openpt`/`grantpt`/`unlockpt`/`ptsname` (all in `libc` proper — NOT
`openpty`, which needs `-lutil`), used by `md`, `mk` and `ms` alike. It is
constellation-shaped, which is the other reason it did not belong inside one
branch of P3.

### F-333 — the `mnemonic-io-lib` git pin lands on a `descriptor-mnemonic` release recipe that was ALREADY broken by the miniscript pin; same class as F-324, one repo over (repo: **descriptor-mnemonic** + **mnemonic-toolkit**; owning phase: **before the next `descriptor-mnemonic-md-cli-v*` tag** — non-deferrable past it) `#ci` `#repro` `#deps` `#cross-repo`

**Found 2026-08-27 by P3's `md` branch**, executing row 1 (*the pin*). The row
says *"Three files, no other edit."* In this repo that is false, and the repo's
own gate said so within one commit — `ci/repro/vendor-freshness.sh` failed
CLOSED, naming the uncovered source and its own fix, exactly as its comment
predicted. **This is the same defect P2 filed as F-324 for `mnemonic-secret`;
the plan is now 2 for 2 on repos where "no other edit" was untrue.**

**MEASURED THREE WAYS UNDER AN EMPTY `CARGO_HOME`** — the isolation is
load-bearing, and F-324 records a false GREEN from omitting it. Each run is
`cargo metadata --format-version 1 --locked --offline` with the named
`--config` set:

| `--config` form | rc | fails on |
| --- | --- | --- |
| TWO-block (crates-io + vendored-sources) — **what `man-pages.yml` passes today** | **101** | `miniscript` |
| THREE-block (+ the miniscript git source) — what `vendor-freshness.sh` had | **101** | `mnemonic-io-lib` |
| FOUR-block (+ the mnemonic-engrave git source) — what `vendor-freshness.sh` has now | **0** | — |

**READ THE FIRST ROW: `md`'s tag-time reproducible build was already broken
before P3 touched it.** The miniscript `[patch.crates-io]` git rev landed
2026-08-20; `vendor-freshness.sh` was converted to the three-block form then and
records the history in its own header, but `.github/workflows/man-pages.yml` was
not, and still passes two blocks on both `musl-binaries` legs. P3 adds a
**second** uncovered git source to a recipe that already could not resolve the
first.

**FIXED on the md branch** (commit *"P3 row 1, consequence"*): `cargo vendor
vendor/` — which added exactly one directory, `vendor/mnemonic-io-lib/`, and
moved nothing else in the 125-entry tree — and the mnemonic-engrave stanza in
`ci/repro/vendor-freshness.sh`, with the rev derived from `Cargo.lock` so it
tracks the pin, failing closed on an empty match. Green under an empty
`CARGO_HOME`; negative control (`vendor/mnemonic-io-lib/` moved aside) reds; and
a synthetic third `source = "git+…"` line in `Cargo.lock` still trips the
fail-closed guard, so widening the cover for one dependency did not turn the
guard off.

**NOT fixed on the md branch, and this is where it differs from F-324.** P2
converted `mnemonic-secret`'s `man-release.yml` legs as well. This branch left
`man-pages.yml` alone, for three stated reasons:

1. **It cannot be exercised from here.** It triggers on
   `descriptor-mnemonic-md-cli-v*` tags only and needs docker + GHCR. *A gate
   that has never executed is a hypothesis* — and so is an edit to one.
2. **A correct edit still cannot produce a release.** The same workflow's
   `repro:` job calls
   `bg002h/mnemonic-toolkit/.github/workflows/reproducible-musl-build.yml`,
   whose only git-source knob is `miniscript_rev` and whose three `--config`
   lines hard-code the rust-miniscript URL. `md` passes `miniscript_rev: ""`.
   That is the identical cross-repo blocker F-324 names, so `md` cannot tag
   until the toolkit change lands either way.
3. **P3 is not what makes it red** — row 1 of the table above.

**So the fix is one change, not two:** generalise the toolkit's reusable workflow
to a `git_source`/`git_rev` pair (or a list), re-pin the `uses:` SHA in **both**
`mnemonic-secret`'s `man-release.yml` and `descriptor-mnemonic`'s
`man-pages.yml`, and convert md's two `musl-binaries` legs to the four-block form
in the same pass — miniscript AND mnemonic-io-lib, because md needs both and
`ms` needs one. Then **exercise it**: one `workflow_dispatch` run per repo.

**Doc consequence, unfixed for the same reason:**
`descriptor-mnemonic/docs/verify-reproducibility.md` tells an external rebuilder
`md` is *"fork-free"* and to pass *"the same two `--config` overrides"* (its §4
and its §8 recipe). That instruction has been wrong since the miniscript pin and
is now wrong twice over. It should be corrected in the same pass that fixes the
workflow, so the doc and the recipe move together rather than the doc describing
a state that does not exist.

**And a note for the third branch:** `mk`'s repo should be checked for a
committed `vendor/` tree and a `vendor-freshness` gate before its pin is called
done. Two of three so far.
### F-361 — ✅ CLOSED 2026-08-27 — F-280's new `clippy` CI step is RED on arrival: the CI-pinned toolchain's clippy disagrees with the repo's default clippy on pre-existing findings (repo: **mnemonic-engrave**; owning phase: **before `fix/f280-ci-fmt` merges to master** — non-deferrable, it is the required check) `#me` `#tooling` `#gate` `#clippy`

**Found 2026-08-27 closing F-280.** F-280's own text measured only
`cargo fmt --check` (77 hunks / 14 files at `ba1f3ec`; re-measured on this
branch at `3609b0c` as 76 hunks / 13 files). It never ran `cargo clippy`
under the pinned toolchain, and neither did the brief that dispatched the
close — its verification recipe assumed `cargo clippy --all-targets
--locked -- -D warnings` would exit 0 once the reformat landed. It does not,
and never did; the reformat is unrelated (confirmed by diffing the finding
list before and after reformatting — byte-identical).

**Measured on `fix/f280-ci-fmt` at the commit that adds the CI steps:**

```
cargo +1.85.0 clippy --all-targets --locked -- -D warnings   ->  exit 101
cargo        clippy --all-targets --locked -- -D warnings   ->  exit 0   (default toolchain)
```

`+1.85.0` is `.github/workflows/release.yml`'s `RUST_TOOLCHAIN` pin, clippy
0.1.85 (2025-02-17). The repo's default toolchain here is nightly
(rustc 1.97.0-nightly, clippy 0.1.97, 2026-04-27) — over a year of clippy
lint churn apart. Nothing in CI has ever run clippy (F-280), so this drifted
unnoticed the same way the formatting did.

**13 distinct findings, 8 files**, all pre-existing (not introduced by
F-280's reformat):

- `crates/me-cli/src/sysw/record.rs:278` — `unknown lint:
  clippy::manual_is_multiple_of`. The `#[allow(...)]` at that line already
  carries the comment `// % 2 != 0 rather than is_multiple_of — unstable on
  CI's Rust`, i.e. the code was deliberately written FOR the pinned
  toolchain's rustc, but the `#[allow]` naming a not-yet-existent lint on
  that same toolchain's clippy was authored/checked against the newer local
  clippy instead — direct evidence of the same untested-against-pinned-CI
  drift F-280 describes, manifesting via clippy rather than fmt.
- `crates/me-cli/src/sysw/wire.rs:85` — same `unknown lint` shape.
- `crates/me-cli/src/sysw/record.rs:294` — `clippy::precedence` ("operator
  precedence can trip the unwary") on `hi << 4 | lo`. Checked: `<<` already
  binds tighter than `|` in Rust, so this parses as intended per the
  comment two lines above explaining the `|`/`^` equivalence; the lint asks
  for explicit parens for readability, not a correctness fix — **not a
  funds-safety defect**, confirmed by reading the surrounding
  cargo-mutants note before filing this.
- `crates/me-cli/src/sysw/coverage.rs:277` — two lints on one line,
  `clippy::nonminimal_bool` + a "comparison might be written more
  concisely" suggestion, on `assert!(... == !unbuilt.is_empty(), ...)`.
- `clippy::format_collect` ("use of `format!` to build up a string from an
  iterator") at: `crates/me-cli/src/seal/pubhash.rs:41`,
  `crates/me-cli/src/seal/pubhash.rs:62`, `crates/me-cli/src/seal/crypto.rs:81`,
  `crates/me-cli/src/seal/mod.rs:421`, `crates/me-cli/src/sysw/pubhash.rs:36`,
  `crates/me-cli/src/sysw/record.rs:270`, `crates/me-cli/src/sysw/tx.rs:288`,
  `crates/me-cli/src/sysw/tx.rs:364`, `crates/me-cli/src/sysw/vectors.rs:57`.

**Not fixed on `fix/f280-ci-fmt`, deliberately.** F-280 measured and scoped
only the fmt drift; these are real code edits (rewriting `map(...).collect()`
patterns, an `#[allow]` fix, added parens, a bool simplification) across 8
files, not a mechanical reflow, and the dispatch brief for F-280 scoped the
branch to fmt + wiring the gate, not to fixing unrelated pre-existing lint
debt. The `clippy` step is still wired into `test (rust + go)` on that
branch (matching F-280's explicit ask and the sibling `mnemonic-transaction`
shape) — so it is a real, exercised gate from the moment it lands, per the
same "a gate that has never executed is a hypothesis" principle F-280 itself
invokes. That means **the branch's tip is RED on the required check** until
this closes.

**What would close it, same order-matters shape as F-280:** the gate is
already wired (this entry's job); fix the 13 findings in their own commit,
verify `cargo nextest run --locked` still reports 430 passed / 1 skipped
(no behaviour change), and confirm `cargo +1.85.0 clippy --all-targets
--locked -- -D warnings` exits 0. Do this **before** `fix/f280-ci-fmt` is
merged to `master` — merging first would make the required `test (rust +
go)` context fail on every subsequent push until someone notices.

**✅ CLOSED 2026-08-27, on `fix/f280-ci-fmt`.** Fixed in two commits: the
two `unknown_lints` sites (`record.rs`, `wire.rs`) first, then the rest.

**Correction to the count above: it was undercounted, not overcounted —
the original 13/8 came from a build that never finished.** The lib
target's compile errors were blocking cargo from ever reaching the
`bin "me"` and integration-test targets, so clippy's first pass silently
never checked them; fixing what it *could* see surfaced 6 more findings in
two further rounds (1 in `crates/me-cli/tests/sysw_cli.rs`, reached only
once the lib compiled; then 5 in `crates/me-cli/src/main.rs` — 3
`clippy::precedence`, 2 `clippy::format_collect` — reached only once
`bin "me"` did). **True total: 18 lint instances across 11 files**, not 13
across 8. Re-run after each fix until a round introduced nothing new, per
the "closure is lens-closure, not finding-closure" rule this repo already
holds elsewhere — the same discipline would have caught this masking on
the first pass had it been applied then.

Breakdown of the 18: 2 `unknown lint: clippy::manual_is_multiple_of`
(`record.rs`, `wire.rs` — fixed by stacking `#[allow(unknown_lints)]`
above the existing `#[allow(clippy::manual_is_multiple_of)]`, which
satisfies both toolchains at once: the pinned one no longer errors on a
name it doesn't recognise, and the newer one's `allow` still suppresses
the lint it does recognise); 4 `clippy::precedence` (`record.rs:294`'s
`hi << 4 | lo`, and three in `main.rs`'s base64 encoder, `(n >> N) & 63`)
— each checked against Rust's operator table and confirmed to already
parse as intended, so the fix is parens for clarity only, not a
correctness change; 2 `clippy::nonminimal_bool` + `clippy::bool_comparison`
on one line in `coverage.rs` (`X == !Y` → `X != Y`, verified boolean-
equivalent); 10 `clippy::format_collect` (`main.rs` ×2, `seal/{crypto,mod,
pubhash}.rs`, `sysw/{pubhash,record,tx}.rs` (`tx.rs` ×2), `sysw/
vectors.rs`, `tests/sysw_cli.rs` — clippy's own suggested `fold` + `write!`
rewrite, each needing a local `use std::fmt::Write as _;`, placed inside
`mod tests` for the two sites where the helper is test-only so a
non-test `lib` build doesn't get a fresh unused-import error from the
same masking effect).

None of the 18 was a real defect — every precedence finding already
parsed as intended, and `format_collect`/`bool_comparison`/
`nonminimal_bool` are documented non-behavioural rewrites. Verified,
final state on `fix/f280-ci-fmt`:

```
cargo +1.85.0 fmt --check                                    -> exit 0
cargo +1.85.0 clippy --all-targets --locked -- -D warnings    -> exit 0
cargo        clippy --all-targets --locked -- -D warnings    -> exit 0
cargo nextest run --locked                                    -> 430 passed, 1 skipped
actionlint .github/workflows/release.yml                      -> exit 0
```

`cargo nextest run --locked`'s count is unchanged from the F-280 baseline
(430/1), confirming none of the 18 fixes altered behaviour. Full detail in
`design/agent-reports/FIX-F280-ci-fmt.md`.

### F-341 — `mk`'s tag-time reproducible musl build cannot be fixed by any input its shared workflow accepts, because that workflow can only ever redirect `rust-miniscript` (repo: **mnemonic-key** + **mnemonic-toolkit**; owning phase: **before the next `mk-cli-v*` tag** — non-deferrable past it) `#ci` `#repro` `#deps` `#cross-repo`

**Third instance of the F-324 / F-333 class, and the sharpest of the three**, because `mk` has no block list the shared workflow is *capable* of emitting that works.

`mnemonic-key/.github/workflows/musl-binaries.yml:68` calls
`bg002h/mnemonic-toolkit/.github/workflows/reproducible-musl-build.yml@6e37b18e50f9f857e439db1ebe2748fc91a54612`
with `miniscript_rev: ""` (line 80), selecting a TWO-block `--config` source list. That workflow builds its list by interpolating `MINISCRIPT_REV` into `source."git+https://github.com/rust-bitcoin/rust-miniscript?rev=…"` and has no other parameter, so `rust-miniscript` is the only git source it can ever redirect.

P3 pinned `mnemonic-io-lib` by git rev, so `mnemonic-key/Cargo.lock` now carries one `source = "git+…bg002h/mnemonic-engrave…"` entry. Measured under an **empty `CARGO_HOME`** (so nothing could resolve from a warm registry cache), against that lock:

| block list | rc |
| --- | --- |
| two-block — what the workflow passes today | **101**, `failed to get mnemonic-io-lib as a dependency of package mk-cli` |
| three-block with a **miniscript** stanza — the only other list the workflow can emit | **101** |
| three-block with the **mnemonic-engrave** stanza | **0** |

The middle row is what makes this different from F-333. `md` needed a *fourth* block added to a list that already had a miniscript entry; `mk` has **no miniscript git source at all** (no `[patch.crates-io]` anywhere, `miniscript` resolves from crates.io), so passing a non-empty `miniscript_rev` does not help and is not even meaningful here. There is no value of any existing input that makes `mk`'s repro build resolve.

**`mk` did NOT have the pre-existing breakage `md` found.** Before the pin, the two-block form resolved at **rc 0** under an empty `CARGO_HOME`. This gate was green and P3's pin is what broke it — unlike `descriptor-mnemonic`, where the miniscript pin had already broken it on 2026-08-20.

**Deliberately not fixed**, matching the sibling branches' posture: the fix is in a fourth repo's shared reusable workflow that `descriptor-mnemonic`, `mnemonic-secret` and `mnemonic-toolkit` all consume, it cannot be exercised from a subject repo, and one coordinated change closes all three of F-324, F-333 and this. The generalisable fix is a **generic git-source input** on that workflow (a list of `url,rev` pairs) rather than a `miniscript_rev` scalar.

**What IS fixed in `mnemonic-key`:** the PR-time gate. `ci/repro/vendor-freshness.sh` went two-block → three-block, deriving the rev from `Cargo.lock` so a pin bump needs no edit, and failing closed both when a git source exists that it cannot redirect and when a *second* one appears. Verified green under an empty `CARGO_HOME` and RED under two negative controls (vendor dir hidden; a second unknown git source).

---

### F-342 — `md repair --json` drops its error envelope on any codec failure, while `mk repair --json` keeps one (repo: **descriptor-mnemonic**; owning phase: **whichever cycle owns `--json` uniformity** — SPEC §6b puts it out of scope for this one) `#md` `#json` `#cli-uniformity`

Found while transplanting `md`'s repair exit-code bypass into `mk` for P3's exit-code row, by checking §6b (*"`--json` is UNCHANGED and explicitly OUT OF SCOPE this cycle"*) against the transplant rather than assuming it held.

`md repair`'s bypass is a bare `Err(e) => { eprintln!("md: repair: {e}"); return Ok(2); }`. It runs **before** the `--json` mode is consulted, so on any codec error out of the correcting decode `md repair --json` exits 2 with an **empty stdout** and a plain-text line on stderr. Measured: `md repair --json <a card the correcting decode rejects>` → exit 2, stdout empty, stderr `md: repair: codex32 decode error: …`.

A consumer that parses `md repair --json` gets nothing to parse and no signal other than the exit code — while the same tool's other verbs emit a structured envelope on failure.

**Why this is filed rather than fixed:** `descriptor-mnemonic` is another branch's repo this cycle, the defect is pre-existing rather than introduced by P3, and §6b puts `--json` out of scope. It is recorded because it nearly propagated: transplanting `md`'s shape verbatim into `mk` would have **deleted** `mk repair --json`'s existing envelope, making `mk` match `md` by losing behaviour. Measured before the P3 change, `mk repair --json <uncorrectable>` emitted `{"error":{"details":null,"exit_code":2,"kind":"BchUncorrectable",…},"schema_version":1}`; `mk`'s bypass now rebuilds that envelope with the code the bypass returns, and the result diffs byte-identical to the pre-change output.

**When it is fixed**, the two CLIs should agree, and the envelope's `exit_code` field must carry the code the process actually exits with (2), not the `CliError`'s mapped 1 — otherwise a consumer reads one number and its shell reads another.

---

### F-343 — `mk encode` binds stubs in FLAG order, not argv order, and stub order is on the wire (repo: **mnemonic-key**; owning phase: **ownerless residue** — a documentation/UX item, already documented and pinned) `#mk` `#cli` `#nit`

`--policy-id-stub`, then `--from-md1`, then `--from-md1-set`. clap does not preserve inter-flag argv position without `indices_of`, and the first two already had this ordering before P3 added the third.

It matters because stub order is **on the wire**: measured, the same eight md1 strings supplied A-then-B and B-then-A mint different `mk1` cards. Found because the first draft of `--from-md1-set`'s test asserted argv order and went red.

Now stated in the flag's help text, in the source, and pinned by a test asserting both the order and that the two orders really do differ. `mk verify` compares stubs as a multiset, so a card minted in either order still verifies (with a note), which is why this is a Nit rather than a defect. Recorded so a later reader does not "fix" it into argv order without knowing a re-mint in a different order is a different card.

### F-362 — `me`'s SECRET-class private-channel advice is UNREACHABLE: the pre-parser guard refuses every input that would select it (repo: **mnemonic-engrave**; owning phase: **a later cycle** — it blocks nothing, and the text is now correct either way) `#me` `#dead-code` `#remedy` `#gates`

**Found 2026-08-27 by P2's row 11**, while building the test that RUNS the line
`me` advises a secret-class operator.

`read_records`'s argv refusal picks between two examples:

```rust
let example = if by_prefix || class.is_bearer() { BEARER… } else { SECRET… };
```

The `else` arm is selected only when `class.is_argv_forbidden()` holds — and
`is_argv_forbidden()` is **exactly** what the pre-parser `argv_secret_guard`
refuses on, at exit 3, before `read_records` is called. Both layers normalise
identically (trim, ASCII-lowercase, `=`-split), so there is no spelling that
gets past one and is caught by the other.

**Measured over eleven argv shapes** — a BIP-39 phrase, an `ms1` in three
spellings, `pass:`, `text:`, three `tx:` forms, `md1`, `mt1` — running
`me sysw pack <shape>` and grepping stderr:

| shape class | outcome | carries the `ms encode` advice |
| --- | --- | --- |
| secret / bearer classes | **rc 3**, pre-parser refusal | **0 of 11** |
| `pass:` / `text:` malformed bodies | rc 4, body error | 0 |
| `tx:` prefixes | rc 3, the record refusal — but via `by_prefix`, so the **BEARER** example | 0 |

So the reachable half of that refusal always takes the bearer branch, and the
secret branch has never printed.

**The text was corrected anyway, and that is deliberate**: a dead branch that is
also WRONG is one refactor away from being live and wrong, and §6h's standing
instruction ("`me`'s `--phrase -` advice becomes the `--in` form when P2 ships
one") fires on the text, not on its reachability.

**A second measurement worth recording.** After P2 made `ms encode`'s stdout the
canonical ungrouped `ms1`, the OLD advice
(`ms encode --phrase - < seed.txt | me sysw pack --out p.bin`) **also runs** —
verified as a mutation: swapping the constant back leaves
`tests/ms_remedy_runs.rs` green. So F-301's live defect is closed by P2's `ms`
rather than by this text change. The `--in` form is still the right advice (one
private channel, no shell redirect), and two genuinely broken mutations
(`--nosuchflag`, and dropping `--out p.bin`) both turn the test RED, so the gate
is a gate.

**What a fix looks like, if one is ever wanted.** Either delete the secret arm
and let the bearer example serve the one reachable case, or give the pre-parser
guard a reason to defer to the record layer for secret classes. Neither is P2's,
and neither is urgent: nothing is wrong on any path an operator can walk.

### F-363 — the two `restore_test_*.py` journey drivers hard-bind `ms` to an absolute path, so no branch build can run them (repo: **mnemonic-engrave**; owning phase: **the ownerless residue** — one line each, and it blocks only a verification step) `#journeys` `#drivers` `#gates`

**Found 2026-08-27 by P2's row 12.** The plan's §1.9 measured that seven of the
eight SHELL drivers bind `MS=$C/mnemonic-secret/target/release/ms`
non-overridably, and row 12 fixed those by following
`derive-pathological-keys.sh`'s `${MS:-…}`. The two PYTHON drivers have the same
defect and were outside that row's scope:

```
design/journeys/restore_test_pathological.py:32     MS = "/scratch/code/…/mnemonic-secret/target/release/ms"
design/journeys/restore_test_tr_pathological.py:39  MS = C + "/mnemonic-secret/target/release/ms"
```

**Measured consequence.** `transcript_tr_pathological.sh` invokes
`restore_test_tr_pathological.py`, which exits 1 with
`FileNotFoundError: … /mnemonic-secret/target/release/ms`, and the transcript
then reports `FATAL: a card-only-plus-seeds restore does NOT reproduce this
wallet`. **Identically before and after P2's driver migration** — verified by
stashing the migration and re-running: `rc=1` and two `FATAL`s either way. So it
is a pre-existing precondition failure, not a regression.

P2 deliberately did NOT edit these two, because row 12's control is *"they are
NOT edited and still pass, because they already use `--phrase -`"* — and they do
use `--phrase -`, so they carry no argv material and needed no migration. The
control's *"still pass"* half could not be exercised without a release build in
the live checkout.

**The fix is one line each**, `os.environ.get("MS", <the default>)`, matching what
row 12 did for the shell drivers. It is filed rather than done because editing
them would have falsified row 12's own control in the same commit.

### F-351 — `ms-shares combine` and `slip39 combine` have NO multi-record private channel, so `--share` cannot be refused without advising an impossible remedy (repo: **mnemonic-toolkit**; owning phase: **whichever phase gives `mnemonic` an `--in`**) `#argv` `#channels` `#p3`

**Found 2026-08-27 by P3's mnemonic branch, while building the argv refusal.**
`--share` is one of the eleven argv-material shapes F-292 measured, and it is
the one shape the refusal deliberately does **not** cover.

Measured on the built binary:

```
$ printf '%s\n%s\n' "$SHARE0" "$SHARE1" | mnemonic ms-shares combine --share - --to ms1
error: ms1 codex32: InvalidLength(100)
$ mnemonic bundle ... --slot @0.phrase=- --slot @1.phrase=-
error: at most one --slot @N.<secret>=- per invocation (single stdin per invocation)
```

So `--share -` reads exactly ONE share, and a K-of-N recovery needs K ≥ 2. A
refusal on `--share` would therefore print `--share -` as the remedy for an
invocation where that remedy cannot be followed — the one thing §6h forbids
outright, and worse than the advisory it replaced, because it stops the operator
without telling them what to do instead.

`--slot` escapes this because `bundle` resolves `@env:` sentinels (verified: a
three-cosigner bundle with three `@env:` slots exits 0). `--share` has no
`@env:` path.

**The fix is a channel, not a refusal**: either `--share -` accepting a stream of
shares, or an `--in FILE`. Once one exists, add `--share` to
`crates/mnemonic-toolkit/src/argv_guard.rs`'s `TABLE` and extend
`channel_for`.

### F-352 — clap ECHOES a stray positional verbatim, so a phrase pasted where no flag names it still reaches stderr (repo: **mnemonic-toolkit**; owning phase: **a later phase — §6d's SECOND, value-shape layer**) `#argv` `#leak` `#p3`

**Found 2026-08-27 by P3's mnemonic branch, measured before the guard existed
and re-measured after.** The refusal P3 built is §6d's *first*, flag-keyed layer.
This is what only the second layer can reach.

```
$ mnemonic convert --to xpub --template bip84 "abandon abandon … about"
error: unexpected argument 'abandon abandon abandon abandon abandon abandon
abandon abandon abandon abandon abandon about' found
$?  ->  64
```

The whole phrase is printed back. This is the exact defect `mt`'s source records
from the other side, in a tool that now has a pre-parser guard — the guard sees
no *flag* to key on, so the token reaches clap and clap names it.

**Per the operator's 2026-08-27 severity ruling this is logged, not blocking.**
The remedy is §6d layer 2: value-shape detection (a BIP-39 mnemonic by wordlist,
an `ms1` by HRP) over raw argv, which `mt` and `me` both ship and `mnemonic` does
not. Note the contrast measured in the same session: a declared flag's VALUE is
**not** echoed (`--bogus-flag` after `--from phrase=<…>` names only the flag), so
the exposure is positional-only.

### F-353 — `--ms1` has no private channel on `verify-bundle` or `import-wallet`, so it is not refused there (repo: **mnemonic-toolkit**; owning phase: **whichever phase adds a second stdin channel or an `--in` to those two verbs**) `#argv` `#channels` `#p3`

**Found 2026-08-27 by P3's mnemonic branch.** `--ms1` carries seed-equivalent
material and is refused on `inspect`, `repair` and the three `xpub-search` verbs.
It is exempt on the other two carriers, and the exemption is measured rather than
assumed:

| verb | `--ms1 -` | `--ms1-stdin` |
| --- | --- | --- |
| `inspect` | **works** | absent |
| `repair` | **works** | absent |
| `xpub-search *` | taken as a literal 1-char ms1, exit 1 | **exists** |
| `verify-bundle` | not accepted | absent |
| `import-wallet` | not accepted | absent |

`verify-bundle` additionally needs `--slot @N.phrase=` at the same time, and only
one input per invocation may be `-`, so even a working sentinel would name an
impossible combination. Same shape as F-351.

### F-354 — `vendor/miniscript` is NOT the rev `Cargo.toml` pins, and the freshness gate cannot see it (repo: **mnemonic-toolkit**; owning phase: **before the next release tag** — it decides what the shipped musl binary compiles) `#vendor` `#repro` `#miniscript`

**Found 2026-08-27 by P3's mnemonic branch, while vendoring `mnemonic-io-lib`.**
Running `cargo vendor vendor/` rewrote **16 files** under `vendor/miniscript/`
that P3 had no business touching. Investigated rather than committed:

```
Cargo.toml [patch.crates-io] miniscript rev = ff4732e5f75aa555682343cb180fa72ee3e8e9d5
Cargo.lock  source = git+…rust-miniscript?rev=ff4732e5…
committed vendor/miniscript/nightly-version   -> nightly-2026-04-24
a fresh clone at ff4732e5, nightly-version    -> nightly-2026-05-08
```

**So the committed vendored tree is a DIFFERENT miniscript from the one a normal
`cargo build` resolves** — 296 insertions / 100 deletions apart, across
`descriptor/tr/`, `miniscript/satisfy/` and `psbt/finalizer.rs`. The reproducible
musl release binary compiles the vendored copy; every other build compiles
`ff4732e5`.

**`ci/repro/vendor-freshness.sh` is structurally blind to this.** It runs
`cargo metadata`, which validates dependency RESOLUTION — names and versions —
and never reads a source file. A vendored directory whose `.cargo-checksum.json`
is internally consistent passes regardless of which commit produced it.

P3 restored `vendor/miniscript` with `git checkout --` and vendored only
`mnemonic-io-lib`, because re-vendoring would silently change what the release
binary compiles in a funds-relevant dependency, outside P3's row. **Whoever fixes
this must decide which rev is intended** and move the other side to match.
See F-355: the repro scripts' own default names the *stale* rev, which is
probably where the drift entered.

**ORIGIN FOUND 2026-08-27, while archiving the branch it came from.** The pin
first advanced `95fdd1c` → `ff4732e` on `stale/experimental-taproot-depth-ge2`,
in commit `24e3a029`. That commit changed `Cargo.toml`, `Cargo.lock`,
`EXPERIMENTAL.md` and the two fuzz manifests — **and not `vendor/`. Zero commits
on that branch touch `vendor/` at all.** Master later adopted the same bump the
same way.

So nobody skipped a step they knew about: **re-vendoring was never part of how a
pin gets advanced here**, and the gate that should have said so compares
`cargo metadata` resolution rather than vendored content. The branch is archived
as `archive/experimental-taproot-depth-ge2` and its tag carries this history.

**That makes the content-aware gate the actual fix, not the re-vendor.** The
re-vendor corrects one instance; the gate is what stops the next pin bump from
doing it again, and pins get bumped routinely.


### F-355 — the tag-time and scheduled reproducible builds are ALREADY broken, pre-P3, on the miniscript rev (repo: **mnemonic-toolkit**; owning phase: **the same one that resolves F-354**) `#repro` `#ci` `#miniscript`

**Found 2026-08-27 by P3's mnemonic branch. Reported, not fixed** — P3 is not
what broke it, and it cannot be exercised from a branch (the callers are
tag-triggered and cron-triggered). Same class as **F-333** in
`descriptor-mnemonic`, found by the `md` branch the same day.

`man-pages.yml` and `repro-drift.yml` both pass
`miniscript_rev: "95fdd1c5773bd918c574d2225787973f63e16a66"` to
`reproducible-musl-build.yml`, and `ci/repro/double-build.sh`,
`ci/repro/cc-validate.sh` and `ci/repro/remap-off-negative.sh` all default to the
same value. `Cargo.lock`'s actual source key is `?rev=ff4732e5…`. A `[source]`
stanza keyed on a rev that appears in no lockfile is inert, so miniscript is left
unmapped and `--offline` cannot resolve it.

Measured, cold `CARGO_HOME`, with a CORRECT `mnemonic-io-lib` stanza present so
the failure is isolated to miniscript:

```
cargo metadata --locked --offline  (4-block config, MINISCRIPT_REV=95fdd1c5…)
error: failed to load source for dependency `miniscript`
$? -> 101
```

**Two things must move together when this is fixed**: the miniscript rev (see
F-354), and a FOURTH source block for `mnemonic-io-lib` — P3 added one to
`ci/repro/vendor-freshness.sh`, which derives both revs from `Cargo.lock` and
fails closed; the other three scripts still take theirs from an env default.
Deriving them the same way would have prevented this drift and would prevent the
next one.

**Cross-repo note recorded at the coordinator's request:** `descriptor-mnemonic`
cannot cut a tag until a `mnemonic-toolkit` reusable-workflow change lands, so
this repo is upstream of that sibling's release path. Neither side should be
edited before both statuses are read together.

### F-356 — ~39 prose command blocks still teach an argv invocation that P3 now refuses (repo: **mnemonic-toolkit**; owning phase: **the toolkit release** — the manual ships with it) `#docs` `#argv` `#p3`

**Found 2026-08-27 by P3's mnemonic branch, by a machine sweep rather than by
reading.** P3 rewrote the 24 command blocks that are *paired with a committed
transcript* (those are byte-gated in CI and had to move), and added an
authoritative reference section, *"Secret material on argv is REFUSED"*, at
`docs/manual/src/40-cli-reference/41-mnemonic.md`. The rest of the workflow and
reference chapters still show the old form.

The sweep parses every fenced `sh` block under `docs/manual/src`,
`docs/quickstart/src` and `docs/technical-manual/src`, and flags any block whose
`mnemonic` invocation carries a flag/value pair the guard refuses. It reported
**43 concrete recipe blocks** and **8 synopsis blocks**; four of the 43 are false
positives (`--ms1` on `verify-bundle`, which is exempt per F-353), so the real
figure is **~39**.

Not fixed in P3 for a stated reason rather than by omission: it is a pedagogical
rewrite of the workflow chapters that **no gate checks**, so a scripted pass
could silently teach something wrong in a funds manual. The failure mode of
leaving them is loud and self-correcting — an operator who copies one gets a
refusal naming the class, the private channel and the override.

The 8 synopsis blocks (`mnemonic repair [--ms1 <MS1>] …`) are a smaller item:
they document a flag that still exists, and would read better as
`--ms1 <MS1|->`.

### F-357 — the doc-validation surface is FIVE things, not three, and no plan has yet named all of them (repo: **mnemonic-toolkit**; owning phase: **ownerless residue** — a process item, generalises F-313) `#ci` `#docs` `#gates`

**Found 2026-08-27 by P3's mnemonic branch, by RUNNING each workflow rather than
reading the plan's census.** F-313 recorded that `cargo`-shaped green is blind to
the doc transcripts. This records what the surface actually measures to.

| the plan said | measured |
| --- | --- |
| 3 replaying workflows: `quickstart.yml`, `manual-gui.yml`, `technical-manual.yml` | the three that replay against the **local** binary are `manual.yml`, `quickstart.yml`, `technical-manual.yml`. `manual-gui.yml` installs `mnemonic-toolkit-v0.74.0` from a tag and its transcripts are **unaffected** by a branch |
| 19 goldens invalidated by the argv refusal | **24** — 19 under `docs/manual/transcripts/` plus 5 under `docs/technical-manual/transcripts/` |
| 4 goldens invalidated by the grouping flip | **5** — the fifth is `docs/technical-manual/transcripts/mnemonic-bundle-bip84-abandon.out` |
| (not named at all) | **`.examples-build/Examples.md`**, byte-gated by `examples.yml` with `git diff --exit-code` and shipped as `docs/Examples.pdf`. P3's argv row baked a REFUSAL into its worked multi-cosigner example before this was found |

**The generalisable fix** is F-313's, widened: a standard closure list that names
`manual.yml`, `quickstart.yml`, `technical-manual.yml` **and** `examples.yml`, so
a future plan inherits the surface instead of rediscovering one piece of it per
cycle. Two cycles have now each found a piece the previous one missed.

### F-358 — there is no `make regen-examples`, so a drifted golden has to be regenerated by hand-reimplementing the verifier's capture semantics (repo: **mnemonic-toolkit**; owning phase: **ownerless residue**) `#docs` `#tooling`

**Found 2026-08-27 by P3's mnemonic branch.** `docs/manual/tests/verify-examples.sh`
replays a `.cmd` and byte-compares; nothing writes the result back. P3 needed a
throwaway script that duplicated its substitution list, its per-`.cmd`
`mktemp -d` cwd, and its pair-vs-triple format branch — and the duplicate had a
bug the original does not: `$( )` strips trailing newlines, so it silently
dropped a trailing blank line from `41-bundle-inheritance-cards.out`, which the
manual includes as `lines="1-29"`. Caught by comparing line counts before and
after; a `make regen-examples` sharing the verifier's own code path could not
have had it.

### F-359 — `vendor-freshness.sh` reports a FALSE GREEN on a warm `CARGO_HOME` (repo: **mnemonic-toolkit**; owning phase: **ownerless residue**) `#ci` `#gates` `#vendor`

**Found 2026-08-27 by P3's mnemonic branch**, and it is the reason the branch
caught its own vendor breakage at all.

After the `mnemonic-io-lib` git dependency was added and BEFORE `cargo vendor`
was run, on one unchanged tree:

```
bash ci/repro/vendor-freshness.sh                       -> exit 0   "OK"
CARGO_HOME=<empty dir> bash ci/repro/vendor-freshness.sh -> exit 1
    can't checkout from 'https://github.com/bg002h/mnemonic-engrave':
    you are in the offline mode (--offline)
```

`--offline` stops the network; it does **not** stop cargo reading a git checkout
that is already in `~/.cargo/git`. On CI that cache is empty, so the gate is
sound there — but a developer running it locally to check their own work gets a
pass on a tree that reds in CI, which is the worst direction for a gate to be
wrong in.

**The fix is small**: either point `CARGO_HOME` at a scratch dir inside the
script, or make the script say in its output that it must be run cold. P3 added a
comment at the `SRC_CONFIG` block recording the measurement, which is not the
same as making the gate correct.

### F-370 — P4, the operator journey, is DEFERRED until after release (operator ruling 2026-08-27) — and it is the FIRST post-release item, not "eventually" (repo: **mnemonic-engrave**; owning phase: **immediately post-release**) `#cli-uniformity` `#journey` `#scheduled`

**Ruled 2026-08-27**: *"We will defer p4 until after release."*

So the cycle ships on P0..P3. Each of those carries its own gates, its own R0
GREEN and its own CI, and **nothing in them depends on P4 having run** —
`--expect` is built and tested in P0; P4 only exercises it at operator level.

**The consequence, named here so it is not discovered later.** The release is
validated **per-tool and not across the seam where the tools meet**. Every
individual guarantee is gated; the composition is not. That is precisely where
a journey walk has historically found what correctness review could not — on the
`mt` cycle, five clean correctness rounds closed and a step-by-step walk then
found a Critical none of them could reach, because the defect was a *silence at
a moment* rather than a *wrong thing in a section*.

So this trades a known class of finding for schedule, deliberately, with the
class named.

**Half the seam is already measured**, which is why the trade is defensible.
The `mk` branch ran `mk encode --from-md1-set` into
`me sysw pack --expect descriptor,cosigner --out` and got **exit 0, a 589-byte
payload**, once `md`'s header row lands — and **exit 4** with today's header, the
refusal firing correctly. The seam is known to close. What is deferred is
capturing it as a **journey that regenerates** and that **FAILS when one producer
is made to refuse** — the negative half, which is the half that makes a journey a
gate rather than a demo.

**When it comes due**, the method is the one recorded constellation-wide: walk it
WITH the operator rather than only dispatching a lens, because the operator
diverges in ways an agent will not imagine, and their confusion is the finding
rather than an interruption.

**Do not let this drift into the ownerless residue.** It has an owning phase —
immediately post-release — and an item whose owning phase has passed is overdue,
not deferred.

### F-371 — the stale miniscript rev survives in FOUR more places F-355 does not enumerate, including the operator-facing rebuild procedure and a second independent pin (repo: **mnemonic-toolkit**; owning phase: **the same one that resolves F-354/F-355**) `#repro` `#docs` `#fuzz` `#miniscript`

**Found 2026-08-27 while fixing F-354** (the re-vendor to `ff4732e5`), by grepping
for the stale rev rather than by reading. **Reported, not fixed** — the operator's
standing instruction is that both repos' status be recorded before either the
toolkit's or `descriptor-mnemonic`'s repro `--config` surface is edited, and this
is the same surface F-355 and `md`'s F-333 are parked on.

F-355 enumerates `man-pages.yml`, `repro-drift.yml`, `ci/repro/double-build.sh`,
`ci/repro/cc-validate.sh` and `ci/repro/remap-off-negative.sh`. A tree-wide grep
(`grep -rn 95fdd1c5 --exclude-dir=vendor --exclude-dir=target --exclude-dir=design`)
finds these **as well**:

1. **`docs/verify-reproducibility.md` — 8 sites, and this one is operator-facing.**
   It publishes the exact `cargo build --locked --offline` command an external
   rebuilder is told to run, with all three `[source]` stanzas keyed on
   `?rev=95fdd1c5…`. `Cargo.lock`'s actual source key is `?rev=ff4732e5…`, and a
   stanza keyed on a rev that appears in no lockfile is **inert** — so the
   published procedure leaves miniscript unmapped and cannot resolve offline.
   Anyone following the documented verification steps fails to reproduce, and the
   natural reading of that failure is "the binary does not match", not "the doc is
   stale". Same mechanism F-355 measured (`$? -> 101`), but on the surface aimed
   at people outside the project.

2. **`fuzz/Cargo.toml` + `fuzz/Cargo.lock` — a SECOND, independent pin, still at
   `95fdd1c5`.** `fuzz/` is its own `[workspace]`, and `[patch]` tables do not
   cross workspace boundaries, so the pin is replicated there by hand. Its own
   comment says it is *"Kept byte-identical to the toolkit root `Cargo.toml`
   `[patch.crates-io]`"* — **and it is not**: root is `ff4732e5`, fuzz is
   `95fdd1c5`. The invariant the comment asserts is false, which is the
   comments-outlive-their-conditions class: the note reads as a guarantee and is
   now the thing hiding the drift. Consequence is narrower than (1) — the fuzz
   targets exercise a different miniscript from the one the toolkit ships — but it
   means fuzz coverage does not apply to the shipped formatter.

3. **`.github/workflows/reproducible-musl-build.yml`** — the callee's own
   `miniscript_rev` default and its explanatory comments still name `95fdd1c5…`,
   including the passage arguing why the git-fork block is mandatory. F-355 lists
   the *callers*; this is the workflow they call.

4. **`CHANGELOG.md` names `95fdd1c5…` too — leave it alone.** That entry records
   what a past release actually pinned, so it is correct as history. Flagged only
   so a scripted sweep does not "fix" it.

**Why this is one item and not four.** All four are the same root cause F-354
identified — advancing the `[patch.crates-io]` pin was never accompanied by
updating anything that *restates* the rev — and they should be fixed in one pass
by the phase that resolves F-355, ideally by **deriving the rev from `Cargo.lock`
the way `ci/repro/vendor-freshness.sh` already does** rather than by editing six
more literals. F-355 makes that same recommendation; this item is the evidence
that its list is short.

**Note the vendored tree itself is now correct** (F-354 fixed, `ff4732e5`), and
`ci/repro/vendor-freshness.sh` is content-aware and would RED on a wrong-rev
vendor tree. Neither of those touches the literals above.

### F-381 — `SPEC_vendor_freshness_ci_guard.md` still describes a ONE-check gate, while the script it governs now runs four (repo: **mnemonic-toolkit**; owning phase: **the same one that resolves F-354/F-355/F-371**) `#docs` `#ci` `#gates` `#vendor`

**Found 2026-08-27 while merging F-354 into P3.** Reported, not fixed — the merge
brief scoped the work to the merge itself, and this drift predates it on *both*
parents, so it belongs to the phase that closes the F-354 family rather than to a
merge commit.

`ci/repro/vendor-freshness.sh:60` names
`design/SPEC_vendor_freshness_ci_guard.md` as its spec. That spec is 122 lines
and its `## 3. Design` section has exactly one subsection describing the check —
`### 3.1 The check`, settled to `cargo metadata`. Measured: the spec contains
**zero** occurrences of `INTEGRITY`, `REGISTRY PROVENANCE`, `GIT-FORK`,
`grounding`, or the `(n/4)` progress labels the script now prints.

F-354 rewrote that script into **four** checks — resolution, integrity (7490
files across 169 crates against recorded sha256), registry provenance plus the
unanchored-SET assertion, and a hand-grounded git-fork anchor carrying two
constants (`EXPECTED_GIT_FORK_REV`, `EXPECTED_GIT_FORK_MANIFEST_SHA256`) that a
human must re-derive whenever the `[patch.crates-io]` pin moves. None of that
reaches the spec. The re-grounding procedure exists **only** as a comment in the
script's own header.

**Why it matters more than ordinary doc drift.** Check (4) is a
trust-on-first-use anchor: its correctness rests entirely on a human having
verified the vendored tree against upstream *once*, and on the next human
repeating that verification rather than pasting the new digest in. That is a
procedure, and it currently lives in one comment block inside the artifact it
governs — so a reviewer who reads the spec to learn what the gate guarantees
learns the pre-F-354 answer, and a maintainer who moves the pin has no
spec-level statement that re-grounding is a verification step and not a
transcription step. This is the `#docs`-shaped half of the same root cause F-371
records: the pin advanced and everything that *restates* what the pin means
stayed put.

**Not urgent, and deliberately not bundled.** The gate itself is correct and
green; nothing here changes behaviour. Fix it in the pass that already has the
whole F-354 surface open, and prefer folding §3.1 into a `3.1 The four checks`
that states each check's blind spot — the script's header already has that text,
and it is better in the spec than duplicated.

---

### F-391 — `md`'s and `mk`'s vendor gates disagree on shape, and BOTH are resolution-only: a one-byte corruption of a vendored file passes them at rc 0 (repo: **descriptor-mnemonic** + **mnemonic-key**; owning phase: **the same one that resolves F-354/F-355/F-371/F-381**) `#ci` `#gates` `#vendor` `#deps`

**Found 2026-08-27** while moving `mnemonic-io-lib` from a git rev to the
published registry version in both repos (see
`design/agent-reports/FIX-io-lib-registry-md-mk.md`). Reported, not fixed — the
brief scoped the work to the dependency change, and this drift predates it.

**Two defects, one root cause: neither gate was ported from the shape
`mnemonic-toolkit` actually settled on.**

**(1) The gates assert the PRESENCE OF A NAMED GIT SOURCE, where the toolkit
asserts an UNANCHORED SET.** `mnemonic-toolkit/ci/repro/vendor-freshness.sh:204`
reads `unexpected = [d for d, _ in unanchored if d != fork_dir]` — it names the
crates allowed to *lack* a registry anchor and fails closed on any other. That
shape survives a git source being **removed**: the set simply shrinks. The two
sibling gates instead derive a rev for a named URL and fail closed when the
derivation is empty, so removing a git source **reds the gate**:

| gate | zero io-lib git sources | why |
| --- | --- | --- |
| `mnemonic-key/ci/repro/vendor-freshness.sh:58-64` | **rc 0** | fail-closed branch is guarded by `[ "$GIT_SOURCES" != "0" ]`, and `SRC_CONFIG` is built inside `if [ -n "$IO_LIB_REV" ]` — it degrades to the two-block form |
| `descriptor-mnemonic/ci/repro/vendor-freshness.sh:58-63` | **rc 1** | unconditional `if [ -z "$IOLIB_REV" ]; then … exit 1; fi` |

Measured, both repos, after the registry change. `mk`'s gate needed **no edit at
all** — `git diff` over the script across the change is empty — while `md`'s
cannot pass without reverting the gate hunk of `9914ae41` (+29/-9). A gate that
must be edited whenever a dependency stops being a git source is asserting the
wrong thing: the property worth guarding is *"every vendored crate is anchored,
or is on a grounded exemption list"*, not *"this named git URL is still present"*.

**(2) Both gates are check (1) only, and check (1) cannot see file corruption.**
Measured on `mnemonic-key`, one byte flipped inside a doc comment in
`vendor/mnemonic-io-lib/src/write.rs` (same length, still valid Rust, so `rustc`
alone would not notice):

```
bash ci/repro/vendor-freshness.sh                   -> rc 0   "OK — vendor/ satisfies Cargo.lock."
cargo build -p mnemonic-io-lib (vendored config)    -> rc 101 "error: the listed checksum of
                                                     …/vendor/mnemonic-io-lib/src/write.rs has changed"
```

**The PR-time gate is green on a tampered vendor tree; only the build reds.**
This is exactly the hole `mnemonic-toolkit` closed with its checks (2) INTEGRITY,
(3) REGISTRY PROVENANCE and (4) GIT-FORK PROVENANCE, whose header records that
check (1) alone stayed green for two months over the F-354 mis-vendored tree.
Both siblings commit a `vendor/` tree and both ship release binaries built
`--offline` from it, so they carry the same latent defect the toolkit already
paid for.

**What the registry change did buy**, and it is worth recording because it is
the only anchoring these gates currently have: tampering the `package` digest in
`vendor/mnemonic-io-lib/.cargo-checksum.json` now reds `mk`'s gate at rc 1
(`error: checksum for mnemonic-io-lib v0.1.0 changed between lock files`), because
cargo compares it against `Cargo.lock`'s checksum during resolution. Under the
git rev **both sides of that comparison were `null`** and the check did not
exist. So the ruling converted one crate from unanchored to anchored — but the
per-file integrity hole is orthogonal and still open in both repos.

**Fix, when the F-354 family's phase runs it:** port
`mnemonic-toolkit/ci/repro/vendor-freshness.sh`'s checks (2)-(4) into both
siblings, replacing the named-URL derivation with the unanchored-set assertion.
`md` keeps exactly one grounded exemption (`miniscript`); `mk` needs **none** —
after this change its `Cargo.lock` carries zero `source = "git+…"` entries, so
every vendored crate there is registry-anchored and the exemption list is empty.
