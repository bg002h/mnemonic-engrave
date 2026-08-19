# RECON — round-trip journeys that exist in `seedhammer` (bg002h fork)

**Date:** 2026-08-19. **Scope:** read-only inventory of `/scratch/code/shibboleth/seedhammer`
at commit `5bfc118fb6524a2ab8722aa643ccfae853c9c99f` (2026-08-18). Governed by
`mnemonic-engrave/design/DRAFT_round_trip_journey_definition.md` §§1-8, including
the §8 rulings (preview-not-toolpath decoder for T4, fixed test seed OK, inventory
only what exists, passphrase/network/account-index are variations not journeys).
Per operator ruling §8.3, this inventories what **exists** and records what it
**covers** — it does not propose what should exist, except where §7 requires
flagging a missing field as a finding.

## What I actually ran

This was almost entirely close reading of source, because the definition's own
method-discipline rules (never hand-count, verify negatives a second way) apply
to a Go/JS tree the same as anywhere else. Concretely:

- `git log -1`, `git status --short`, `find . -maxdepth 2 -type d` to orient.
- `grep -rl` / `grep -rn` passes (checked for exit code, not just empty stdout,
  per "empty grep is not proof of absence") for: `journey`, the `shTap/shSysw/…`
  walk-API symbols, `walk_*.js` references across Go/YAML/Markdown, `RoundTrip`,
  `WalletPolicyId`/`WalletDescriptorTemplateId`/receive-address vocabulary,
  `gozxing|zbar|zxing|decode.?QR|OCR|decodeGlyph`, `seedqr\.`, `seedqr\.Parse(`,
  `bundle.Verify(`, `runUITouchRaster(`, `kortschak-qr`.
- `wc -l`, `ls -la oracle/gaterecords/` to get exact counts rather than describe
  them.
- Read in full: `cmd/emu/walk_js.go`, `walk_trace_a.js`, `walk_trace_b.js`, and
  the relevant halves of `walk_build_policy.js` / `walk_s3_nested.js` /
  `walk_s4_gate.js`; `cmd/emu/needle_test.go`, `gaterecord_anchor_test.go`,
  `engraved.go`; `cmd/gaterecord/main.go`; `oracle/record.go`'s function list and
  `oracle/expect.go`'s function list; the two committed gate-record JSON sets
  (`S0-trace-a.*`, `S5-trace-b.*`) in full; `.github/workflows/test.yml` in full;
  `backup/qrdecode_test.go` in full and its one call site in `passphrase_test.go`;
  `gui/multisig_build_walk_test.go`, `gui/s6b_restore_doc_test.go`,
  `gui/multisig_engrave_tail_walk_test.go` (incl. `s5StubVerifyFn`),
  `gui/multisig_verify_policy_test.go`, `gui/multisig_verify_flow_test.go`
  (`s5DriveVerify`/`s5DriveVerifyRec`), `gui/gui_test.go`'s `testEngraver`,
  `gui/s6b_passphrase_plate_test.go`'s policy-id test, `cmd/journeykeys/main.go`,
  `cmd/buildpayloadcards/main.go` (header + constants), `cmd/plateview/main.go`
  and `cmd/glyphtrace/main.go` (headers), `seedqr/seedqr.go` in full.
- No tests were executed (reading was sufficient; the brief does not require
  running the ~282s-if-unfiltered suite, and none of the questions here turned
  on a live run). No code was changed. No firmware built or flashed.

---

## Headline findings (read this first)

1. **`bundle.Verify` is the one mechanism in this repo that could supply the
   definition's functional equality (fingerprint, xpub, origin path) plus a
   real structural check (md1 exact string, ms1 recovered entropy, mk1↔md1 stub
   binding) — and no discovered journey exercises it end to end.** Every T3
   emulator walk that reaches a verify offer taps **Skip**. The one in-process
   test family that presses "Verify now" (`TestBothEngraveFlowsDriveTheRetryLoop`)
   does so against a **stubbed** verify function (`s5StubVerifyFn` replaces
   `multisigVerifyFn` wholesale) — it proves the retry UI state machine, not that
   a real verify passes. The only tests that call the **real**, non-stubbed
   `multisigVerifyFlow` (`TestVerifyStillPassesItsOwnPolicy`,
   `TestVerifyRefusesPlatesFromADifferentPolicy` in
   `gui/multisig_verify_policy_test.go`) start from an **already-built** md1/mk1
   (custodial), not from a driven build. No single journey joins "build a policy"
   to "really verify it."
2. **§3.1's write-only claim is correct for T3/T4 and for the general engraved
   surface, but not unconditionally true of the whole tree.** A genuine,
   independent, geometry-based decoder exists for exactly one artifact:
   `backup/qrdecode_test.go` + `backup/passphrase_test.go:TestPassphraseQRIsByteExact`
   decode the **passphrase plate's** QR code back from the real engrave planner's
   knot geometry (not from the preview, not sharing glyph code with the writer)
   and assert byte-exact equality against 13 input passphrases. It is T1
   (in-process), test-only, and scoped to one plate type — it does not cover
   SeedQR-on-cards, free-text plates, or the printed constant/vector-font TEXT
   (no OCR/glyph decoder exists anywhere — confirmed by grep for
   `OCR|decodeGlyph|decodeText|readGlyph|glyphDecode|gozxing|zbar|zxing` returning
   only comments that explicitly disclaim OCR). It is also the single cleanest,
   most reproducible round trip in the repo: `go test ./backup/ -run
   TestPassphraseQRIsByteExact`, one command, runs under default `go test ./...`
   in CI.
3. **`seedqr.Parse` (the SeedQR/CompactSeedQR *decode* entry point) is never
   called anywhere in this repo** — production or test (`grep -rn
   "seedqr\.Parse("` returns nothing, verified by exit code). Every use of the
   `seedqr` package is `seedqr.QR`/`seedqr.CompactQR` (encode only).
4. **The T3 emulator walks' own "structural" census check is a write-time
   instrumentation echo, not a read-back decode.** `cmd/emu/engraved.go`'s
   `engravedRecorder` accumulates strings via `gui.EngravedAware.PlateText`/
   `PlateEngraved` — hooks the production GUI code calls to *announce* what it is
   about to cut — correlated with a completion signal from the (fake/browser)
   engraver. The file's own comment: "no exported call turns an arbitrary md1
   back into a plate." So even Trace A/B's byte comparison is (GUI's self-report)
   vs. (independently-invoked Rust `ms`/`md`/`mk` CLI derivation) — a real,
   valuable cross-implementation check, but not a geometric read-back.
5. **Of the five `cmd/emu/walk_*.js` drivers, only two have ever produced a
   persisted, checkable result** (`oracle/gaterecords/S0-trace-a.*`,
   `S5-trace-b.*` — measured: `ls oracle/gaterecords/` returns exactly 8 files,
   2 bases). `walk_build_policy.js`, `walk_s3_nested.js`, `walk_s4_gate.js`
   exist, are needle-tested statically, and (per their own headers) at least the
   latter two *engrave* by default — but nothing in this repo shows they were
   ever run to a recorded, independently-checked conclusion. This is UNKNOWN,
   not "never run": I found no gate record, no walk.json, and no doc reference
   for any of the three (`grep -rl` for their filenames across `*.md` returns
   nothing, verified by exit code 1), but a human could have run one in a
   browser and simply not minted a record.
6. **None of the `cmd/emu/walk_*.js` drivers are executed by CI.**
   `.github/workflows/test.yml` runs `GOOS=js GOARCH=wasm go vet ./cmd/emu/` —
   a compile/type-check only. Running a walk requires a human: build+serve
   `cmd/emu`, drive it from a browser console per each file's own header
   snippet, save the returned JSON by hand, then run `cmd/gaterecord` by hand.
   **By contrast**, the in-process `gui`-package walk family (18 files using
   `runUITouchRaster`, real `EventRouter` button events, real rendered frames)
   carries no build tag and **does run** under the default `CGO_ENABLED=0 go
   test -timeout 20m ./...` on every push — genuinely the most-frequently-executed
   tier of journey in this repo, at the cost of a weaker terminal assertion (see
   Journey 6 below: `testEngraver.Write` discards the actual toolpath bytes, so
   there is no byte-level oracle check at all in that family, only plate counts
   and on-screen text).

---

## Journeys catalogued (§7 schema)

### J1 — "Trace A" (gathered-bundle walk)

| field | value |
| --- | --- |
| name | Trace A — `cmd/emu/walk_trace_a.js` |
| kind | **custodial**. Loads a pre-built payload of already-encoded mk1 chunks (3 cosigner cards, BIP-39 published vectors) and re-engraves them verbatim as a gathered bundle. No new key material is derived by the device in this walk. |
| tier | T3 (browser/WASM emulator, real screens via `window.shScreen()`, real input transport via `window.shTap`/`shPress`/`shRelease` posting `gui.PointerEvent`, real NFC-chunk queue via `window.shNFC.present`) |
| origin artifact | `cmd/emu/sysw_cards_payload.bin`, digest `25271e583f3eaa03ae18f359c72b76e3`, 3 masters (BIP-39 published vectors A/B/C), each supplying one mk1 card (2 chunks) |
| ordered invocations | Build+serve `cmd/emu` on a fresh port → browser console `const w = await import("./walk_trace_a.js"); await w.run()` → drives Load Payload → Engrave Bundle → presents 3 cards' chunks over the simulated NFC queue → holds/confirms through 6 engrave-style plates → reads `window.shToolpath.strings()` for the census → **operator manually saves the JSON return value to a file** → `go run ./cmd/gaterecord -stage S0 -walk /tmp/walk.json -inputs oracle/gaterecords/S0-trace-a.inputs.json -base S0-trace-a` (this step shells out to installed `ms`/`mk` Rust binaries, a **third repo's** primary implementation, from `~/.cargo/bin`) → writes `oracle/gaterecords/S0-trace-a.{record,walk,expect}.json` |
| structural assertion | `oracle.CompareCensus`: the 6 engraved mk1 strings, byte-for-byte, in order, equal what `mk encode` independently derives (from the Rust primary CLI) from the same seed words + origin/fingerprint. Re-checked with no toolchain by `oracle.TestEveryGateRecordCensusMatchesItsCommittedExpectation` (part of the untagged `go test ./...` CI suite), so the **persisted comparison** is re-verified every push even though the **walk itself** is not re-run. |
| functional assertion | **NONE independently exercised — FINDING.** Master fingerprints appear in `S0-trace-a.expect.json` as metadata attached to each derived artifact, but they are encoded *inside* the same bytes the structural check already compares; nothing decodes the mk1 back and independently re-derives/compares the fingerprint, xpub, or an address at a different layer. No receive/change address anywhere. §4's "neither alone is sufficient" is unmet — only one of the two required equalities exists. |
| the ONE command | **FINDING — none exists.** Reproducing this journey requires (a) a human driving a browser, (b) a human manually saving the returned JSON, (c) a manual `cmd/gaterecord` invocation naming a Rust toolchain's bin directory. The only single-command piece is re-verifying the **already-persisted** record (`go test ./oracle/... `). |
| stated non-coverage | **FINDING — no formal §6 statement.** The file's header discusses timing/caching/queueing traps at length but the walk's own JS-object return value carries no "what this did not cover" field. |

Gate-executed status: **yes, once** — `S0-trace-a.walk.json` records `elapsedSec: 181`, dated 2026-08-14/15 (file mtimes). Not re-run since; no CI trigger re-runs it.

### J2 — "Trace B" (built-policy walk, the S5 flagship)

| field | value |
| --- | --- |
| name | Trace B — `cmd/emu/walk_trace_b.js` |
| kind | **mixed, predominantly generative.** @0/@1 (master A, one payload-sourced ClassMnemonic, two BIP-48 accounts) and @2 (master B, **typed on an on-device keyboard driver this walk had to add**, `MASTER_B_WORDS` = BIP-39's published vector) are device-**minted** mk1/ms1 cards under this policy's own id stub. @3 is filled from a payload cosigner card (C@0) taken as supplied — custodial for that one slot. The definition's generative/custodial dichotomy does not cleanly label a walk that is genuinely both at once; recorded as a finding on the taxonomy rather than forced into one box. |
| tier | T3, same mechanism as J1, plus a from-scratch on-device BIP-39 keyboard driver (`typeWord`/`typePhrase`, tap-point geometry cross-checked against `gui/keyboard_geometry_test.go`) |
| origin artifact | `sysw_cards_payload.bin` (same digest as J1) for master A + card C@0, plus the literal `MASTER_B_WORDS` for master B |
| ordered invocations | Same build/serve pattern as J1 → `w.run()` drives: Load Payload → Engrave Multisig → Build policy → wsh, n=4/k=3 → the S5 multi-select "Do you hold another slot?" picker three times (@0,@1,@2) → cosigner gather (assertNoNFC proves zero NFC records crossed the reader — cards come only from payload+keyboard) → picks `skip,skip,skip` on the over-supply picker to land on C@0 → seed entry for @0/@1 (payload) and @2 (typed, full keyboard drive with **per-letter post-condition checks**) → Key-sources review, asserting the multi-account and multi-account-notice sentences → Policy Review, asserting **both** `ORIGINS_EXPECTED` paths appear (`m/48h/0h/0h/2h`, `m/48h/0h/1h/2h`) → EXPERIMENTAL hold-confirm → Full mode → plate census (17 plates: 2 ms1 + 7 mk1 + 8 md1, counted from `S5-trace-b.expect.json`) → engrave tail → **Skip the verify offer** (row 1 of 2, explicit) → restore doc → same manual save + `cmd/gaterecord` handoff as J1 |
| structural assertion | Same mechanism as J1: `CompareCensus` against a live Rust-primary derivation, persisted and re-checked in CI. |
| functional assertion | **NONE — same finding as J1.** The verify offer that would exercise `bundle.Verify` (fingerprint + xpub + origin + stub-binding, see headline finding 1) is explicitly skipped: `walk_trace_b.js:680, await tap([240, rowY(1, 2)], 300);` (comment: "Skip the verify (row 1 of 2)"). |
| the ONE command | **FINDING — same as J1**, plus this walk additionally requires the operator to have added the keyboard driver correctly (its own header calls out that F-181 called a keyboard driver "genuinely optional" until this walk needed one). |
| stated non-coverage | **FINDING — same as J1**, no formal statement, though the file's extensive header does enumerate what makes this walk distinct from earlier ones. |

Gate-executed status: **yes, once** — `S5-trace-b.walk.json` exists, dated 2026-08-16.

### J3 — "Build-policy driver" (`cmd/emu/walk_build_policy.js`)

| field | value |
| --- | --- |
| name | Build-policy driver |
| kind | generative (single-slot, self key typed or from payload depending on parameters) |
| tier | T3 |
| origin artifact | `sysw_cards_payload.bin` + optional typed phrase, parameterized (`payload`, `n`, `k`, `selfSlot`, `seedFrom`, …) |
| ordered invocations | Reaches "Engrave Multisig → Build policy" and, **only if called with `{engrave: true}`**, drives the tail to a completed bundle; the file's own documented default invocation (`w.run().then(...)`, header line 5, annotated "seconds, no engraving") does **not** engrave at all. |
| structural assertion | If `engrave:false` (the default): none past screen-text needles. If `engrave:true`: census non-empty + `unattributed===0`, but **no `CompareCensus`/oracle step exists for this driver** — no `oracle/gaterecords/*build-policy*` files exist (confirmed: `ls oracle/gaterecords/` lists only `S0-trace-a.*` and `S5-trace-b.*`). |
| functional assertion | None. |
| the ONE command | Not reproducible by one command; same manual-browser requirement as J1/J2, and even more so since the default invocation is documented as a *partial* run. |
| stated non-coverage | Not stated. |
| execution evidence | **UNKNOWN.** No gate record, no `.md` reference anywhere in this repo (checked with exit-code-verified `grep -rl`). `needle_test.go`'s comment "measured 2026-08-14 by driving each" suggests a human ran this driver at least once to compare screen text against `Engrave Bundle`, but that is a static needle-uniqueness measurement, not a completed, recorded engrave. I could not determine whether an `{engrave:true}` run was ever recorded — I looked for a gate record and a doc mention and found neither, which is the strongest negative this repo supports. |

### J4 — "S3 nested-segwit driver" (`cmd/emu/walk_s3_nested.js`)

| field | value |
| --- | --- |
| name | S3 nested-segwit driver |
| kind | generative |
| tier | T3 |
| origin artifact | `sysw_cards_payload.bin` (parameterized like J3) |
| ordered invocations | Front door → Build policy → template row 1 (sh(wsh)) → n/k/slot pickers → cosigner gather/pick → seed entry → Policy Review (asserts `NEEDLE_NESTED_NOTE`, the sh(wsh)-only origin sentence) → EXPERIMENTAL → engrave tail (engraves by default — header: "minutes: it engraves") → **skips the verify offer** (`await tap([240, rowY(1, 2)], 300);`, row 1) → restore doc, asserting `NEEDLE_NESTED_NAME` ("P2SH-P2WSH") is present and the legacy-P2SH sentence is **not** |
| structural assertion | Plate census non-empty/`unattributed===0` **plus** a content-needle check that the restore doc names the nested-segwit script correctly. This is a real, useful content check, but it is string-containment on a rendered screen, not a byte-exact comparison against an independently-derived oracle artifact — **no gate record / `CompareCensus` exists for this driver** (same `ls oracle/gaterecords/` evidence as J3). |
| functional assertion | **NONE — same headline finding 1.** The comment at line ~85 explicitly names `"First receive:"` as a string that appears on the restore doc (two production sites) and explicitly declines to treat it as a needle — and, confirmed by grep, the walk never reads or asserts that value at all. The receive address this build would show is never captured. |
| the ONE command | Not reproducible by one command (same manual-browser requirement). |
| stated non-coverage | Not stated. |
| execution evidence | **UNKNOWN**, same basis as J3 — no gate record, no doc mention found. |

### J5 — "S4 seed↔key gate driver" (`cmd/emu/walk_s4_gate.js`)

| field | value |
| --- | --- |
| name | S4 seed↔key gate driver, both arms |
| kind | generative, with a deliberate **negative control**: one payload/driver serves both a "pass" arm (`arm:"pass"`, default) and a "fail" arm (`arm:"fail"`), differing only in which payload card fills the operator's own slot — same seed, same n=2, only the card assignment changes, so a green happy path and a red refusal cannot both be explained by driver noise. |
| tier | T3 |
| origin artifact | `sysw_cards_payload.bin`, n=2, card-order-dependent picks |
| ordered invocations | Front door → Build policy → n=2/k=2 → slot picker → cosigner pick (`use,skip,skip` for pass; `skip,skip` for fail) → seed entry → the device's own internal seed↔key derivation gate → **pass arm**: engrave tail to completion, **skips verify** (row 1) → **fail arm**: races for a named refusal screen, asserts it names the slot and says nothing was engraved |
| structural assertion | Pass arm: census non-empty, `unattributed===0`. Fail arm: census **empty** (`census.strings.length===0`) plus refusal-text checks. As with J3/J4, **no `CompareCensus`/gate record exists** for either arm. |
| functional assertion | The device's own internal seed-derives-key check IS, in spirit, a functional check — but it is entirely internal to the device under test: the walk observes only the device's own PROCEED/FAIL verdict, with no independently-computed fingerprint/xpub comparison outside the device. This is weaker than an external oracle but is a legitimate negative control (§5's "must not assert against a value the journey itself produced with no independent source" is arguably satisfied here differently than elsewhere, since the pass/fail *divergence* — not a specific value — is what's being tested, and the divergence is driven by an input change the walk controls). |
| the ONE command | Not reproducible by one command. |
| stated non-coverage | Not stated. |
| execution evidence | **UNKNOWN**, same basis as J3/J4. |

### J6 — `TestBuildWalkTypedSeed` (in-process GUI walk, `gui/multisig_build_walk_test.go`)

| field | value |
| --- | --- |
| name | TestBuildWalkTypedSeed |
| kind | generative for the self key (typed via a real on-device keyboard driver, `fixtureMasterA`), custodial for cosigners (`cosignerCardRecords(t,4)` fixture, payload-shaped records) |
| tier | Functionally T3-equivalent but architecturally distinct from J1-J5: real production GUI code (`buildMultisigPolicyFlow`), real rendered frames (`op.Drawer.Draw`/`ExtractText`), real input transport — but via `EventRouter.Events(ButtonEvent{...})` (physical-button semantics), in-process as a native Go test with `testing/synctest` virtual time, no browser, no WASM, no `cmd/emu`. **This is a taxonomy gap, not a defect**: the tier table's T3 row is worded around "the emulator," and this harness proves the same thing (a user reaching the screen and completing the flow) through a different, CI-resident mechanism. Recorded as a finding for whoever extends §3's tier table. |
| origin artifact | `fixtureMasterA` (BIP-39 vector, typed) + `cosignerCardRecords(t,4)` |
| ordered invocations | One Go test, in-process: template/n/k/slot pickers → S5 multi-select (single slot held) → fingerprints/self-source pickers → cosigner gather → over-supply picker (skip/use/use to land on B@0,C@0) → **on-device keyboard drive** of `fixtureMasterA`, letter-by-letter with post-condition checks (`typeWords`) → passphrase skip → Key-sources review → Policy Review, asserted to contain the BIP-48 origin sentence and "Check each key below"/`xpub` across paged content → EXPERIMENTAL hold → engrave-mode/plate-census screen, asserted to read "This engraves 9 plates" → **every one of the 9 plates actually engraved** (`engraveOnePlate` loop, waiting on the fake engraver's `Close()`) → **Skip the verify offer** (`click(&ctx.Router, Down) // Skip`) → restore doc |
| structural assertion | Plate **count** (9, hand-derived by the test author from "ms1(1)+mk1(2)+md1(6)", not cross-checked against the Rust primary) + specific on-screen text (origin sentence, `xpub` presence, "Descriptor:" on the restore doc). **No byte-level content check exists**: `testEngraver.Write(steps []uint32) (int, error)` (`gui/gui_test.go:480`) discards the step payload entirely — it returns `len(steps), nil` and records nothing — so there is no analogue of `cmd/emu`'s `engravedRecorder`/`shToolpath.strings()` census in this whole 18-file test family. This is a firm, code-read finding, not inference. |
| functional assertion | **NONE — same headline finding 1.** Verify explicitly skipped. |
| the ONE command | **YES — this journey has one.** `go test ./gui/ -run TestBuildWalkTypedSeed`. Single command, in-process, and (no build tag) runs automatically under the default `CGO_ENABLED=0 go test -timeout 20m ./...` in CI on every push. This is the strongest "gate has executed" story of any journey found. |
| stated non-coverage | Not printed by the test itself (Go tests don't have an output channel for this), though the file's extensive header comments state scope precisely in prose. |

### J7 — `TestRestoreDocReflectsARealCutPassphrasePlate` (`gui/s6b_restore_doc_test.go:275`)

| field | value |
| --- | --- |
| name | TestRestoreDocReflectsARealCutPassphrasePlate |
| kind | generative — `abandonAboutPhrase()` (the canonical all-"abandon" BIP-39 vector), typed on-device, plus a typed passphrase `"hunter2"` |
| tier | Same in-process family as J6 |
| origin artifact | The all-abandon BIP-39 vector + literal passphrase "hunter2" |
| ordered invocations | `engraveSingleSigFlow` (singlesig, **watch-only** mode) → BIP-84 wallet type → passphrase entry → watch-only engrave mode → full policy md1 → 2 cards / 5 plates cut for real (via the same discarding `testEngraver`) → ms1-reminder skip → **Verify Bundle offer reached and explicitly skipped** (`click(&ctx.Router, Down) // Skip verify`, line 345) → Passphrase Plate offer **accepted** (diverges deliberately from the sibling test that stops at the offer) → preloaded source/passphrase/QR screens driven to a real passphrase-plate cut → final restore doc read |
| structural assertion | The restore doc's "passphrase plate was cut" boolean is asserted to be **computed from the real cut**, not a stub value — this is a genuine, narrow, valuable check (its own header cites a prior defect class: "a swapped argument compiles, renders, and looks entirely healthy") but it is a single boolean, not a byte round trip of engraved content. |
| functional assertion | None — watch-only mode carries no private-key verify path at all in this test, and the Verify Bundle offer it does reach is skipped exactly as in J6. |
| the ONE command | `go test ./gui/ -run TestRestoreDocReflectsARealCutPassphrasePlate` — single command, CI-resident. Cost self-disclosed in the file's own comment: ~26-28s. |
| stated non-coverage | Same as J6 — prose in the header, not a runtime output field. |

### J8 — `TestBothEngraveFlowsDriveTheRetryLoop` (`gui/multisig_engrave_tail_walk_test.go:365`) — **not a journey by this definition**

Drives both the supply-policy and build-policy flows through a real engrave to the
verify offer, and **does** press "Verify now" rather than "Skip" (`s5AssertRetryLoop`,
line 299). But `calls := s5StubVerifyFn(t, verifyIncomplete)` (line 369/398) replaces
the package-level `multisigVerifyFn` variable with a stub returning a canned
`verifyIncomplete` verdict — confirmed by reading `s5StubVerifyFn` in full
(`gui/multisig_engrave_tail_walk_test.go:98-116`): it reassigns `multisigVerifyFn`
wholesale and restores it via `t.Cleanup`. `bundle.Verify` is never called. This test
proves the retry-offer UI state machine is wired correctly (exactly 2 verify entries,
correct row-to-outcome mapping, CONTINUE actually leaves) — a real and useful
property — but it produces no origin-to-terminal equality at all and is recorded here
only because its header language ("press the row saying 'Verify now'") could be
misread as satisfying headline finding 1. It does not.

### J9 — `TestPassphraseQRIsByteExact` (`backup/passphrase_test.go:250` + `backup/qrdecode_test.go`)

| field | value |
| --- | --- |
| name | TestPassphraseQRIsByteExact |
| kind | **Does not fit generative/custodial.** Origin is an arbitrary passphrase string (13 cases: `"hunter2"`, `"correct horse battery staple"`, leading/trailing/double spaces, all-printable-ASCII, 100-char max-length, alphanumeric- and numeric-QR-mode edge cases), not a seed or a custodial backup artifact — a passphrase is auxiliary secret material, not itself a wallet-identity artifact. **Recorded as a taxonomy finding**: this is unambiguously a bona-fide journey by every other clause of §1 (named, single-command, origin → real production encode path → independent decode → terminal equality), and the kind taxonomy in §2 has no label for it. |
| tier | T1 codec — in-process, no GUI, no emulator, no transport — but notably **stronger than a typical T1 bytes-in/bytes-out test**: it exercises the real engrave-planner geometry, not a bit-buffer shortcut (see invocations). |
| origin artifact | 13 literal passphrase strings (see above) |
| ordered invocations | `Passphrase{Passphrase: tc.in, QR: true, Font: constant.Font}` → `passphrasePoints(t, plate)` (the **real** production engrave/toolpath planner — the same code the firmware calls) → `passphraseQRGrid` **geometrically reconstructs the QR module grid from the engraved knot centres** (`backup/passphrase_test.go:205-244`; it does not read the encoder's own bit buffer — it checks which knot positions inside the QR box are on-module and marks those black) → `decodeQR` (`backup/qrdecode_test.go`, package-private, test-only): brute-forces all 8 QR masks against the grid's fixed function-pattern modules to recover the mask (does not trust a mask the writer chose), reads codewords in standard interleaved order via `github.com/seedhammer/kortschak-qr/coding` (generic QR bit-plan math, **not** glyph-rendering code — satisfies the §8.1 "must not share glyph-rendering code between writer and decoder" mitigation), and parses the segment header back into a string. |
| structural assertion | **YES, byte-exact.** `if out != tc.in { t.Errorf(...) }`. |
| functional assertion | Not applicable — the artifact under test is not a wallet key, so no fingerprint/address/wallet-id exists to check. Not counted as a gap, since the definition's functional-equality clause is scoped to "something that controls funds." |
| the ONE command | `go test ./backup/ -run TestPassphraseQRIsByteExact` — single command, in-process, fast, runs under default `go test ./...` in CI. **The cleanest, most reproducible, most-frequently-executed round trip found in this recon.** |
| stated non-coverage | Not printed at runtime; scoped narrowly by construction: passphrase-plate QR only. Explicitly declines error correction (damage must surface as a decode failure, not be silently repaired) and refuses QR versions above 5 (the pinned ECC-L range). |
| §5 anti-requirement check | Passes every clause I could check: does not read an intermediate nothing writes (reads from the real planner's own knot output); does not assert against a self-produced value with no independent source (the mask is recovered by brute force, not supplied by the encoder); no skip-passes-as-ok path found; the decoder's own comment states plainly what it does not cover (ECC, multi-block versions). |

This test is, in effect, exactly what §3.1's now-struck bullet asked for (decode
from the toolpath rather than the preview) — it simply predates the T4/emulator
discussion entirely and is scoped to one plate type. It does **not** satisfy §8's
actual ruling for T4-sim (preview-based, not toolpath-based), and it is not wired
to the emulator or to any simulated/real engrave — it is a pure T1 in-process test.

### J10 — `TestPolicyIDMatchesTheMK1StubOnBothForms` (`gui/s6b_passphrase_plate_test.go:48`) — supporting cross-check, not a full journey

Calls `deriveSingleSigBundle`/`templateizeBundle` directly (library-level, no GUI,
no screens — so it does not "pass through every layer a real operator would
traverse" and is excluded from the journey table on that ground) but is worth
recording because it answers part of "does anything decode something back": it
calls the **real, general-purpose** `mk.Decode(mk1)` (not a test-only decoder) on
an mk1 string this run's own code produced, and compares the decoded stub against
an independently-computed `md.FormAwareStubChunks` hash of the corresponding md1,
for both the full-policy and template-only forms, with an explicit non-vacuity
check that the two forms' ids actually differ. Fixed origin: `abandonAboutMnemonic()`.

### J11/J12 — real (non-stubbed) `multisigVerifyFlow` exercises

`TestVerifyStillPassesItsOwnPolicy` and `TestVerifyRefusesPlatesFromADifferentPolicy`
(`gui/multisig_verify_policy_test.go`) are the **only** tests in this repo that
call the real `multisigVerifyFlow` (confirmed: `s5DriveVerify`/`s5DriveVerifyRec`
in `gui/multisig_verify_flow_test.go:107-147` call `multisigVerifyFlow` directly,
not through the stubbable `multisigVerifyFn` indirection, and feed card strings
through `ctx.syswBundleSeeds` — the same seam a scanned card uses — plus a real
keyboard-typed seed via `driveWords`). `TestVerifyStillPassesItsOwnPolicy` re-presents
a wallet's own already-built md1 + one already-cut mk1 card and types the matching
seed, and asserts the flow reports "Verify OK" — a genuine, real, functional-equality
outcome from `bundle.Verify` (fingerprint/xpub/origin/stub-binding all run for
real). `TestVerifyRefusesPlatesFromADifferentPolicy` is the adversarial arm: a
byte-valid plate set from a **different**, self-consistent wallet (same cosigners,
different threshold) must be refused, and is.

**But — kind is custodial-only, and it never touches a build.** `s5PolicyPair`
constructs `engravedMd1`/`otherMd1`/`otherPlate` via `assembleBuildPolicy` and
`buildEngraveTail` called **directly**, not via a driven GUI build flow, and no
screens are exercised before the verify step. So this pairs with J6
(generative build, verify skipped) as the other half of headline finding 1: the
pieces exist, separately, and have never been joined into one continuous journey.
Reproducible with `go test ./gui/ -run 'TestVerifyStillPassesItsOwnPolicy|TestVerifyRefusesPlatesFromADifferentPolicy'`,
CI-resident, no build tag.

---

## §5 anti-requirement sweep — violations found

- **"A skipped step must fail, not pass."** No violation found in the walks
  themselves — J1/J2's `ok` fields and J6/J7's `t.Fatalf` calls all fail loudly
  on an unrecognised or stalled screen (`HANDLERS.find` returning undefined
  pushes an `"act": "STALLED"` and breaks the loop rather than continuing).
  However, the **operator-facing "Skip verify" button** that every journey
  presses is a *legitimate, offered* skip in the production UI, not a harness
  defect — flagged under headline finding 1 as a coverage gap, not an
  anti-requirement violation.
- **"It must not read an intermediate that nothing in the journey writes."** No
  violation found in-repo. J1/J2's handoff to `cmd/gaterecord` requires a
  **human** to write `/tmp/walk.json` by hand from the browser console — this is
  documented as the intended handoff, not an undocumented read of a stale file,
  so it does not match F-210's shape (a script silently reading a rotted path)
  even though it shares the "not one command" weakness.
- **"It must not assert against a value the journey itself produced with no
  independent source."** J1/J2 satisfy the strong form (cross-implementation:
  Go GUI vs. Rust primary CLI). J6/J7/J8's plate counts are **hand-derived by a
  human reading the production formula**, not cross-checked against an
  independent implementation — weaker, but not self-referential in the strict
  F-170 sense (I-1's own gate, `TestWalkOkContainsNoDriverSuppliedPlateCount`,
  specifically polices the JS walks for exactly this pattern and I did not find
  a violation of it). J9's decoder brute-forces the mask rather than trusting
  the encoder's choice — this is the strongest anti-self-reference guarantee
  found anywhere in the recon.
- **"Every gate in it must have executed at least once."** **Violated, or at
  minimum unverifiable, for J3/J4/J5** (headline finding 5): no persisted
  evidence in this repo that any of the three ever completed with a recorded,
  checkable outcome. Recorded as UNKNOWN rather than a confirmed violation,
  per the method-discipline instruction to not guess — but the repo currently
  supports no stronger claim.
- **"Empty output is not proof of absence."** Applied throughout: every
  negative claim in this report (`seedqr.Parse` uncalled, no OCR/glyph decoder,
  no doc mentions of the three unrun walks, no gate records beyond two) was
  checked via `grep`'s **exit code**, not just blank stdout, and cross-checked
  a second way where practical (e.g. the "no plate decoder" claim is
  corroborated independently by `cmd/emu/engraved.go`'s own comment "no
  exported call turns an arbitrary md1 back into a plate," not just by an
  absence of hits).

---

## §3.1 decoder-absence claim — confirmed, with one precise carve-out

**Confirmed** for the claim as it actually matters (T3/T4, the general engraved
surface, SeedQR-on-cards, printed text): no code anywhere reads a **simulated or
real engraved plate** — rendered preview, toolpath, or photograph — back into a
string. `plateview` and `glyphtrace` are both write-side visualizers (toolpath →
SVG/PNG), never the reverse. `seedqr.Parse` (data-level SeedQR decode) exists but
is never called. `cmd/emu`'s own census mechanism is a write-time instrumentation
echo (`gui.EngravedAware`), explicitly disclaimed by its own author's comment as
not a decode. No OCR, no QR-image-scanning library (`kortschak-qr` is the encoder;
its `coding` subpackage is generic bit-plan math, reused by the one decoder found).

**One precise exception**: `backup/qrdecode_test.go` is a genuine, independent,
geometry-based QR decoder — see J9. It is scoped to the passphrase plate only, is
T1 (in-process, no emulator/simulated-engrave/photograph involved), and is
test-only code with no path to being reused for T4. It does not contradict
§3.1's conclusion ("T4 needs a reader that does not exist yet") but it does mean
the flat statement "nothing anywhere decodes a plate back into a string" needs a
footnote: something decodes **one plate type's QR region**, from real geometry,
independently of the writer, today.

---

## Known blind spot (per §8.3, stated rather than investigated further)

This recon stayed inside `seedhammer` per its brief. Two things it therefore
cannot see and does not claim to: (1) whether `cmd/buildpayloadcards`'s output
(the committed `sysw_cards_payload.bin` both Trace A and Trace B depend on) is
actually reproducible — regenerating it requires piping through `me sysw pack`,
a **different repo's** CLI (confirmed only by the regeneration instructions in
`cmd/emu/sysw_cards_payload_host_test.go:46`; not verified by running it, which
would be a cross-repo action outside this recon's scope); (2) any journey
definition, catalogue, or gap analysis that lives in `mnemonic-engrave/design/`
rather than in this repo — not read, per the brief's repo boundary.
