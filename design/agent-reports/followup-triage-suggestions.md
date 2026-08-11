# Follow-up ledger triage — code-first, 2026-08-11

Scope: every `F-` entry in `design/FOLLOWUPS.md` (3,372 lines, **67 `### F-` headings
= 63 unique F-numbers**; F-59, F-62, F-99 and F-100 each carry two headings — a
closure banner plus the retained original).

Read against:

- fork `/scratch/code/shibboleth/seedhammer`, `main` @ `823499c`
- host `/scratch/code/shibboleth/mnemonic-engrave`, `master` @ `4d5ef3f`
- `design/SPEC_encrypted_payload_delivery.md`

**Status was determined from the code, not from headings or entry prose.** Where a
claim was machine-checkable it was measured; the command and its output are quoted.

Machine checks actually run for this report (beyond the ones supplied in the brief):

| check | result |
| --- | --- |
| `./scripts/plan-cite-gate.sh design/SPEC_encrypted_payload_delivery.md` | exit **0**, 0 FAIL |
| positive control: a file citing `gui/gui.go:999999` + `nosuchfile.go:1` | 2 FAIL, exit **1** |
| `go test ./gui/ -run 'TestProgressStyleRendersNoPercentSign\|TestPlateLabelSeparatorRenders\|TestUnauthenticatedWarningFitsThePanel\|TestUnlockEngraveMnemonicZeroesM' -v` | all PASS |
| measured widths (today, fork `823499c`) | `width("ab")=22 width("a\|b")=27 width("a·b")=22`; `progress: width("50")=57 width("50%")=56 (delta -1)`, `lead: delta 12` |

Both width tests are written to **fail** if the font gains the glyph. They pass, so
F-78 and F-86 are live at HEAD — measured, not inferred.

---

## 1. Full status table (all 63 F-numbers)

Claimed status = what the ledger's heading and body assert. TRUE status = what the
code says.

| F | subject | claimed | TRUE | owning phase | evidence |
| --- | --- | --- | --- | --- | --- |
| F-58 | total input wedge on the Footer entry screen | OPEN | **OPEN** | GUI | `gui/event.go:266` — `Next` still registers filters as a side effect (`r.filters = append(r.filters, filters...)` is the first statement) and examines `r.events[0]` only; `Reset` (`:281`) discards only head events matching no registered filter. Unchanged. Local workaround note still at `gui/codex32_polish.go:316`. |
| F-59 | `font/constant` has no curves; cusps pile dots | WITHDRAWN 2026-08-06 | **WITHDRAWN** (not open) | — | Cause was Y-axis play; resolution banner in `design/RECON_cusp_dot_pileup.md`. Two headings (banner + retained original). |
| F-60 | single-character test plates, top-left, uncentred | OPEN (standing directive) | **not work** — a practice, no code owed | every engraving investigation | `backup/freetext.go:116` `centerInset` centres title (`:130`) and footer (`:136`); a body row is left-aligned. Entry itself says "No code change needed". |
| F-61 | `preview/params.go` is a fourth, stale copy of the motion params | OPEN | **OPEN** | next `me` preview cycle | `mnemonic-engrave/preview/params.go:18` `EngravingSpeed: 8 * mm`; device is `engravingSpeed = 4 * mm` (`seedhammer/cmd/controller/platform_sh2.go:227`) and host copy `EngravingSpeed = 4 * MM` (`internal/sh2/params.go:29`). `preview/params_test.go` pins only `mm`, `strokeWidth` and a **spatial** bbox golden — nothing binds speed or duration. Submodule still `713aee2` (upstream v1.4.2). |
| F-62 | curving a `font/constant` glyph panics the constant-time engraver | OPEN | **OPEN** | before any curve lands | `engrave/engrave.go:1148` `panic("unaligned delay")` still in `timeScaler`. Entry's cite `:1126` has drifted. |
| F-63 | strike CURRENT is a lever firmware cannot reach | OPEN (recorded fact) | **recorded, not work** | any future depth investigation | `cmd/controller/platform_sh2.go:125` `Ichop = 0`, `:132` `S_SENSE = machine.NoPin`, `:135` `P_ADC = machine.NoPin`. Unchanged. |
| F-64 | `VOLTPROOF!` — engrave the strike conditions | OPEN (idea) | **OPEN** | next depth investigation | `gui.Platform` (`gui/gui.go`) exposes `LockBoot/AppendEvents/Wakeup/Engraver/NFCReader/PayloadReader/EngraverParams/DisplaySize/Dirty/NextChunk/Features/HardwareVersion` — still no power accessor. (It gained `PayloadReader` since the entry was written; nothing about voltage.) |
| F-65 | back up the SH2 boot signing key | OPEN | **OPEN — and now DUE** | after the encrypted-payload cycle ships | The cycle shipped: tag `v0.5.0` (2026-08-11) / `fork-v0.0.0-g93ee004`. `grep -rn "sh2key\|BootKeyMnemonic"` over both repos → 0 hits. |
| F-66 | carry arbitrary plain text over the sealed payload path | OPEN | **OPEN — now unblocked** | own gated cycle, after the cycle ships | `seal/record.go:103-109` — the `Classification` set is still `ClassUnknown/DebugCommand/Mnemonic/Descriptor/Codex32Secret/MDMK/Address`. No text record kind. |
| F-67 | Go `MDDataSymbols` lacks Rust's 93-symbol cap | CLOSED 2026-08-07 | **CLOSED** | Plan B | `codex32/mdmk.go:49` `mdRegularMaxLen = 93`, `:55` `mkRegularMaxLen = 93`, with the β-order-93 aliasing rationale in the comment. |
| F-68 | `plan-build-gate.sh` compiles the CLI tests but never runs them | CLOSED 2026-08-07 | **NOT SATISFIED — mis-attributed** | before Plan B's plan review | `scripts/plan-build-gate.sh:163` still `cargo test -p mnemonic-engrave --test seal_cli --no-run`; its own header line 30 still reads "tests/seal_cli.rs is COMPILED BUT NOT RUN". See §3. |
| F-69 | amend §9 / §12 item 6 for `--seal-secret` | CLOSED 2026-08-07 | **CLOSED** | — | SPEC `:1162` synopsis carries `[--seal-secret]`; `:1165` and `:2169` document it. |
| F-70 | `--seal-secret` covers `ms1` only | CLOSED 2026-08-07 | **CLOSED** | with F-69 | `crates/me-cli/src/main.rs:399` `if !seal_secret && secret.iter().any(is_seed)` — a seed predicate, not an `Ms`-only match. |
| F-71 | Nits from the Plan A whole-diff review | OPEN | **OPEN in the letter; neither is work** | ownerless residue | (a) `seal/wire.rs:119,122` cap each section at `MAX_SECTION_LEN = 8191` before `:163` tests `total > REGION_LEN` (65,536); 52+8191+8191+16 = 16,450, so `:164` is unreachable — confirmed. (b) `pub fn public_data_hash(records: &[&str], sealed: bool)` (`pubhash.rs:26`) takes no salt, so salt-dependence is unrepresentable; still no test. |
| F-72 | md-codec 0.40→0.42 rode into the Task 1 commit | historical note, do NOT rewrite | **not work** | none | The entry is its own disposition. |
| F-73 | XIP read at the normative `0x10E00000` | CLOSED 2026-08-07 | **CLOSED** | — | `design/HARDWARE_RESULT_2026-08-07_phaseB1.md`. |
| F-74 | a build gate covers a Go plan's code | CLOSED 2026-08-08 | **CLOSED** | — | `scripts/plan-build-gate-go.sh` present, 11,694 bytes, executable. |
| F-75 | stale `gui/bundle_flow.go:224` citations outside the SPEC | OPEN | **OPEN in the letter; DECIDED in substance** | ownerless residue | Both copies remain (`design/CONTINUITY_2026-08-07b.md:148`, `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseA.md:638`); `func bundleReviewFlow` is at `gui/bundle_flow.go:227`. The entry already rules them kept-as-history per F-72's precedent. |
| F-76 | inspecting a payload-sourced card | OPEN | **OPEN — now DUE** | after B2b | `gui/md1_gather.go:87` and `gui/mk1_inspect.go:163` still open `ctx.Platform.NFCReader()`; no in-memory priming path exists. `gui/unlock_platelist.go:193-201` documents the deliberate omission and names F-76. B2b has merged. |
| F-77 | encrypted section's md1/mk1 cards have no grouping | heading says GATING (open-looking); body says CLOSED | **CLOSED** | B2a-i Task 1 | `seal/label_encrypted.go:28` `labelEncryptedCards`; wired at `seal/record.go:329-332` under `if section == SectionEncrypted`; production reach `seal/unlock_key.go:102` `AdmitSection(recs, SectionEncrypted)`; tests `seal/label_encrypted_test.go:16` and `:60`. |
| F-78 | `·` has no glyph in the display font | OPEN | **OPEN** | post-merge polish | Measured today: `width("a·b") = width("ab") = 22` (`gui/unlock_platelist_test.go:40`). Still rendered in 4 shipped gui files, **8 live sites** (`grep -rn '·' gui/*.go \| grep -v _test`): `gui/bundle.go:306` (×2 in one expression), `gui/bundle_flow.go:339`, `gui/codex32_polish.go:26,30,49,182,286`, `gui/slip39_polish.go:237`. |
| F-79 | payload buffer retains 64 KB for the GUI's lifetime | heading open-looking; body CLOSED 2026-08-10 | **CLOSED** | B2a-i Task 2 | `seal/read_tinygo.go` `Read()`: `n, err := boundBlob(region)` then `out := make([]byte, n)`; `Probe()` maps `clampRegion(len(Magic))` and allocates nothing. |
| F-80 | residue from the B1 whole-diff review | PARTIALLY CLOSED (2 of 3 B2 bullets) | **further narrowed — 2 MORE bullets now satisfied, unrecorded** | B2a-ii / ownerless | See §2, closures 1–2. Remaining: `layoutMainPager` pixel pin (open, with F-78); `"Sealed Payload"` duplicated as a literal at `gui/gui.go:1835` against `const unlockTitle` at `gui/unlock_flow.go:21` (open); `PlateIndex` positional-vs-`ChunkHeader` doc line (partly addressed at `seal/record.go:148-155`, which explains the grouping origin but never says "positional"). |
| F-81 | withdrawn before it was ever open | WITHDRAWN 2026-08-08 | **WITHDRAWN** | — | — |
| F-82 | `seal.Deriver` / folded `DeriveKey` have no Rust counterpart | OPEN (recorded) | **not work** | ownerless residue | The entry is its own disposition; the six `derived_key_hex` vectors are the binding. |
| F-83 | plate cannot be wiped until the engrave finishes | ACCEPTED LIMITATION | **accepted, not open** | — | Operator ruling (anchor). |
| F-84 | `SeedScreen` gains `NoEdit` | CLOSED 2026-08-10 | **CLOSED** | B2a-ii Task 6 | `gui/gui.go:2341` `NoEdit bool`; guards at `:2388` and `:2464`; production site `gui/unlock_session.go:291` `ss := &SeedScreen{NoEdit: true}`. |
| F-85 | §2.2 does not name the during-engrave residency | CLOSED 2026-08-10 | **CLOSED** | before the release tag | SPEC `:187` item 13 ("The plate under the needle, for the whole of its cut"); operator half at `:338`. |
| F-86 | `%` renders as zero pixels in the KDF progress screen | OPEN | **OPEN** | post-merge polish | Measured today: `progress: width("50")=57 width("50%")=56 (delta -1)`; `lead: delta 12`. `TestProgressStyleRendersNoPercentSign` (`gui/unlock_kdf_test.go:855`) PASSes, and it is written to fail if the face gains the glyph. |
| F-87 | nothing pins `unlockEngraveMnemonic`'s deferred wipe | OPEN (NARROWED to one leg) | **OPEN — narrowing confirmed exactly** | post-merge polish | Seam exists: `gui/unlock_mnemonic_seam.go:13` `unlockMnemonicParsedHook`, fired at `gui/unlock_session.go:284`. `grep -n "^func TestUnlockEngraveMnemonic" gui/unlock_session_test.go` returns exactly two: `...ZeroesMOnConfirmDiscard` (`:795`) and `...ZeroesMOnEngraveSeedError` (`:845`). The `masterFingerprintFor`-error leg has no test. |
| F-88 | three more seed-equivalent copies on the mnemonic engrave path | OPEN | **OPEN** | post-merge polish | `bip39/bip39.go:217-226` `MnemonicSeed` still builds `sentence` by `append` and never wipes it. `gui/gui.go:538-548` `engraveSeed` builds `qrc` (`qr.Encode(string(seedqr.QR(m)), qr.M)`) and `words []string` with no wipe. |
| F-89 | B2b's idle wipe MUST unwind the flow | CLOSED 2026-08-10 | **CLOSED** | B2b | `gui/run_flow.go:282-288` — armed-wipe branch sets `wiping = true; ctx.Done = true; break`. `seal/session.go:20-51` `RecordsResident` carries the narrowed contract and the rename rationale. |
| F-90 | the `ms1` engrave arm is the under-examined one | OPEN | **OPEN for items 1 and 3; item 2 CLOSED with F-89** | post-merge polish | `grep -rn "unlockCodex32Hook" --include='*.go' .` → **0 hits**. `gui/unlock_session.go:186-225` `unlockEngraveCodex32` clears only `rec`; `string(rec)`, `s` (`codex32.String`), `id` from `Split()` and `s.String()` stay live. Item 2 is discharged — see §2, closure 3. |
| F-91 | normative `vectors.json` digest asserted | CLOSED 2026-08-09 | **CLOSED** | — | `seal/vectors_test.go:148` `TestVectorFileMatchesTheDigestTheREADMERecords`. |
| F-92 | `tinygo test` cannot build `seal` | DECLINED 2026-08-10 | **decided, not open** | — | Operator ruling (anchor). |
| F-93 | the screensaver parks a spec-legal derivation | CLOSED 2026-08-10 | **CLOSED** | B2b | `ctx.KeepAwake()` at `gui/unlock_kdf.go:334` (sole non-test caller, verified by `grep -rn "KeepAwake()" --include='*.go' . \| grep -v _test.go`); `&& !armed` at `gui/run_flow.go:251`; both killer tests present (`gui/run_flow_test.go:671`, `:702`). |
| F-94 | the 64-byte seed and the BIP-32 master key are unpinned | OPEN | **OPEN** | post-merge polish | The three wipes exist — `defer wipeBytes(seed)` in `deriveMasterKey` (`gui/gui.go:245`) and `defer mk.Zero()` in `masterFingerprintFor` (`gui/gui.go:563`) — but the **seam does not**: `grep -rn "deriveSeedHook\|deriveMasterKeyHook" --include='*.go' .` returns only the prose at `gui/unlock_session.go:263`. Every one of them is still deletable with the suite green. |
| F-95 | §10.2.3's warning clears the panel by 3 px | OPEN | **OPEN** | post-merge polish | `gui/gui.go:653-656` — `fadeClip` is still `return o.Offset(image.Pt(0, 0))` with the real mask commented out one line above. The fit is pinned (`gui/unlock_flow_test.go:398`, PASSes at 5 record counts) but the copy has not been shortened and `Warning` still has no touch scroll. |
| F-96 | §11.3 mutation runner uncommitted | CLOSED 2026-08-10 | **CLOSED** | B2b | `scripts/mutation-run.py` present, **31,599 bytes** (grown past the 26,827 the entry records, by F-101's crash-safety work). |
| F-97 | plan and record corrections owed to the B2a-ii artefacts | CLOSED 2026-08-09 | **CLOSED** | — | Plan line 1716 now reads "(304 lines) … (237) and … (83)"; `design/PHASE_REPORT_encrypted_payload_deviceB_phaseB2a_ii.md` exists. |
| F-98 | two citations in the GREEN spec do not resolve | CLOSED 2026-08-10 | **CLOSED** | with F-85 | Gate run today: exit **0**, 0 FAIL. Positive control (deliberate bad cites) → 2 FAIL, exit 1. |
| F-99 | §10.2.4 row 1 did not fix WHEN the warning starts | CLOSED 2026-08-09 | **CLOSED** | B2b Task 8 | SPEC `:1619` "appears at **3:00** … the wipe fires at **3:30**"; `:1622` names the rejected warn@2:30 reading. Two headings. |
| F-100 | §11.5 "confirm firmware reflash preserves the blob" | CLOSED 2026-08-09 | **CLOSED** | — | `design/HARDWARE_RESULT_2026-08-09_phaseB2b.md`. Two headings. |
| F-101 | `mutation-run.py` is not crash-safe | CLOSED 2026-08-10 | **CLOSED** | before the release tag | `scripts/mutation-run.py:491` traps `SIGINT/SIGTERM/SIGHUP`; `:549-555` `recover_sentinel()` runs **before** `preflight_clean()`; `scripts/test/mutation-run-crashtest.py` present. |
| F-102 | `me seal` takes SEED MATERIAL on argv | CLOSED 2026-08-10 | **CLOSED** | before the release tag | `crates/me-cli/src/main.rs:82-83` `#[arg(long = "in")] in_path: Option<PathBuf>`, threaded at `:152`/`:158` into `run_seal_cli`; argv doc at `:69-75` warns and names F-102. |
| F-103 | protective screen film silently disables the wipe | OPEN | **OPEN — mechanism unchanged** | post-merge polish | `gui/run_flow.go:251` `if len(evts) > 0 || (ctx.keepAwake && !armed) { a.idle.start = now }`. Still keyed on **raw** events, not effective input. |
| F-104 | four more members of the unreachable-seed-residue class | OPEN | **OPEN — all four** | post-merge polish | (1) `x/crypto/pbkdf2` HMAC state, untouched. (2) `bip39/bip39.go:177-196` `splitMnemonic` still returns `entBytes` built by `append(padding, entBytes...)`, no wipe. (3) `gui/unlock_session.go:186-225` — the `ms1` arm's `ToUpper`/QR copies, unwiped. (4) `gui/passphrase_keyboard.go:51` `Fragment string`, reset by `k.Fragment = ""` at `:244` — drops the reference, does not zero. |
| F-105 | a typed passphrase is wiped by nothing until submitted | CLOSED 2026-08-10 (hardware) | **CLOSED** | B2b Task 9 | Anchor + `gui/unlock_kdf.go:143` `ctx.B.Scrub()` in the passphrase bracket; `gui/unlock_passphrase_wipe_test.go`. |
| F-106 | §10.2.4's window runs 2× | CLOSED 2026-08-10 (hardware) | **CLOSED** | B2b | Anchor + `gui/run_flow.go:65` pre-block arm-edge call ("That is F-106, and this call is the fix"); regression test `gui/idle_late_arm_edge_test.go:12`. |
| F-107 | rendered seed scrubbed only on the wipe path | CLOSED 2026-08-10 | **CLOSED** | B2b | `ctx.B.Scrub()` now has **three** non-test callers — `gui/unlock_session.go:104`, `gui/unlock_kdf.go:143`, `gui/run_flow.go:326` — against the one the entry measured. |
| F-108 | `plate.Spline` never zeroed after the cut | CLOSED 2026-08-10 | **CLOSED in code; one record defect it flagged survives** | B2b | `engrave/engrave.go:1046-1050` `defer func() { clear(knotBuf[:cap(knotBuf)]); if cap(spline) != cap(knotBuf) { clear(spline[:cap(spline)]) } }()` inside `planEngraving`; `engrave/engrave.go:1745` `ClearHistory`; `gui/engraver.go:137` `releaseResumeState`; `gui/engraver.go:268` `defer clear(c)`. See §3, item 3. |
| F-109 | ~35 K in ~81 reachable objects survives every wipe | OPEN | **OPEN** | post-merge polish | The entry's own closing measurement does not exist: `grep -rn "SetFinalizer" --include='*.go' .` returns **only** `gui/op/release_test.go:91` — the `op`-level test, not the `gui`-level sweep over the blob / decrypted records / passphrase buffer / parsed words. |
| F-110 | abandoned engrave job's resume state never zeroed | brief's anchor says CLOSED; ledger body says two halves remain | **OPEN (both halves), OVERDUE** | B2b (passed) | See §3, item 1. |
| F-111 | `knotBuf` unzeroed where a plate is built and no cut happens | CLOSED 2026-08-10 (subsumed) | **CLOSED** | B2b | The `planEngraving` defer fires on **iterator** exit, so build-only paths (`bspline.Measure`, back-out-before-cut, `ErrTooLarge`) are all covered by one line — exactly the subsumption claimed. |
| F-112 | six legacy seed-rendering flows in no `Scrub` bracket | CLOSED as ACCEPTED under §2.2 item 12 | **CLOSED (accepted)** | — | All six still unbracketed — `gui/gui.go:2194`, `gui/derive_xpub.go:82`, `gui/bip85.go:269`, `gui/slip39_polish.go:229`, `gui/seedxor_polish.go:40`, `gui/gui.go:584` — which is what the acceptance says, and the program-scope ruling in my brief re-confirms it. |
| F-113 | codex32 LONG CODES admitted then never engraveable | CLOSED 2026-08-10 | **CLOSED** | post-B2b | Device: `seal/record.go:74` `MaxEngraveableCodex32Len = 90`, refusal at `:287` with `ErrCodex32TooLong` (`:43`). The 90 is bound, not duplicated: `backup/backup.go:141-142` `seedQRLevel = qr.M` / `seedQRMaxSize = 33`. Host: `crates/me-cli/src/seal/record.rs:29` `MAX_ENGRAVEABLE_MS1_LEN = 90`, enforced at `:152`. |
| F-114 | a resumed cut approaches its safe point from the origin | OPEN | **OPEN** | post-merge polish | `engrave/engrave.go:1667` `move = appendLine(move, conf, false, bezier.Point{}, s.safePoint)` inside `SafePointer.Resume`. Unchanged; entry's cite `:1664` has drifted. |
| F-115 | `plan-cite-gate.sh` resolves by BASENAME | CLOSED 2026-08-10 | **CLOSED** | before the release tag | `scripts/plan-cite-gate.sh:69-70` prunes `third_party`, `target`, `.git`, `node_modules`; `:74` prints `AMBIGUOUS: %s files match -- cite a repo-relative path`. |
| F-116 | `biptool seed -seedlen` emits unengraveable strings silently | CLOSED 2026-08-10 | **CLOSED** | before the release tag | `cmd/biptool/main.go:319` `warnUnengraveable(stderr, key.String())`; definition `:340`. |
| F-117 | seed plate cannot engrave a QR above 33 modules | OPEN | **OPEN** | post-release feature | `backup/backup.go:142` `seedQRMaxSize = 33`, refusal `:151`; `:135` states "Raising `seedQRMaxSize` is F-117/F-118 and deliberately not done here." |
| F-118 | engraving a LONG codex32 share needs QR v6 | OPEN | **OPEN** | post-release feature | `engrave/engrave.go:420` `if dim > 37`; `bitmapForQRStatic` tabulates `case 21/25/29/33/37` at `:354-362`, `default:` at `:410`. |
| F-119 | `backup.go:368`'s comment describes a fallback order the code does not implement | OPEN | **OPEN** | post-merge polish | Comment survives at `backup/backup.go:388` (cite drifted from `:368`): "their TEXT+QR -> TEXT-ONLY -> QR-ONLY fallback depends on toPlate rejecting overflow". The caller `gui/gui.go:2105-2109` does **not** fall back at all — it enumerates all three variants and keeps every one that fits (`validLabels` / `validEngravings`). |
| F-120 | device engraves `ms1` strings `me seal` will not seal | OPEN | **OPEN** | post-merge polish | `crates/me-cli/src/seal/record.rs:323-330` records the divergence in its own test doc (`VALID_STR_LENGTHS = [50,56,62,69,75]`, `VALID_MNEM_STR_LENGTHS = [51,58,64,70,77]`, "tops out at 77"); the device's `codex32.New` bands are unchanged. The design call named in the entry has not been made. |

### Counts

Three buckets, disjoint, summing to 63.

| bucket | n | members |
| --- | --- | --- |
| **A. Ledger says closed/withdrawn/declined, and the code agrees** | **32** | F-59, F-67, F-69, F-70, F-73, F-74, F-77, F-79, F-81, F-83, F-84, F-85, F-89, F-91, F-92, F-93, F-96, F-97, F-98, F-99, F-100, F-101, F-102, F-105, F-106, F-107, F-108, F-111, F-112, F-113, F-115, F-116 |
| **B. Ledger reads open, but no work is owed → closure candidates** | **6** | F-60, F-63, F-71, F-72, F-75, F-82 |
| **C. Genuinely OPEN work** | **25** | F-58, F-61, F-62, F-64, F-65, F-66, F-68, F-76, F-78, F-80, F-86, F-87, F-88, F-90, F-94, F-95, F-103, F-104, F-109, F-110, F-114, F-117, F-118, F-119, F-120 |

Of bucket C, **two are recorded as closed** and should not be (F-68, F-110 — §3).
Bucket A's F-108 is closed in code but left one record defect behind (§3.3).
Beyond the 6 whole entries in bucket B there are **3 closeable bullets** inside
still-open entries (F-80 ×2, F-90 item 2), and **3 items whose owning phase has now
arrived** and should be re-dated rather than re-read as deferred (F-65, F-66, F-76).

---

## 2. Suggested closures — with the evidence that closes them

**Suggestions only.** Each is a code fact you can re-check with one command.

**6 whole entries + 3 bullets.** None of these is a judgement call about whether the
work is worth doing; each is a case where the work is already done, was subsumed, or
was never a defect by the entry's own terms.

### Whole entries

**C1 — F-75** *(stale `gui/bundle_flow.go:224` citations outside the SPEC)*
Nothing is owed. The entry's own text already rules the two survivors kept as
history, per F-72's precedent. Confirmed present and confirmed harmless:

```
$ grep -rn "bundle_flow.go:224" design/*.md | grep -v FOLLOWUPS
design/CONTINUITY_2026-08-07b.md:148
design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseA.md:638
$ grep -n "func bundleReviewFlow" ../seedhammer/gui/bundle_flow.go
227:func bundleReviewFlow(ctx *Context, th *Colors, cards []bundleCard) bool {
```

Both files are merged-work records. Close as "decided: kept as history".

**C2 — F-60** *(single-character test plates)*
Never a defect and never work. It is an operator directive, and the entry says so
in its own second line. Verified against the code it asserts: `backup/freetext.go:130`
centres the title via `centerInset`, `:136` the footer; a body row is not centred.
Close as "standing practice, recorded".

**C3 — F-63** *(the strike CURRENT is a hardware lever)*
Recorded fact, unchanged, no work implied by its own text ("Worth knowing before
anyone spends a session looking for it in software"). `cmd/controller/platform_sh2.go:125`
`Ichop = 0`; `:132`/`:135` `machine.NoPin`. Close as "recorded".

**C4 — F-72** *(md-codec bump rode into the Task 1 commit)*
Its owning phase is literally "none — historical note, do NOT rewrite". Close as
"historical record, no action".

**C5 — F-82** *(`seal.Deriver` has no Rust counterpart)*
Its own text is the disposition: the Rust-primary rule does not bind a
byte-identical PBKDF2, and the contract is the six `derived_key_hex` vectors the
Go tests assert. Nothing outstanding. Close as "recorded, rule does not bind".

**C6 — F-71** *(Plan A whole-diff nits)*
Both nits are structurally satisfied and the entry states neither gates. Verified:

- `WireError::TooLarge` unreachable — `crates/me-cli/src/seal/wire.rs:119,122` cap
  each section at `MAX_SECTION_LEN = 8191` **before** `:163`'s `total > REGION_LEN`
  (65,536). 52 + 8191 + 8191 + 16 = 16,450. The branch at `:164` cannot be reached.
- §11.4's salt invariance — `pub fn public_data_hash(records: &[&str], sealed: bool)`
  (`pubhash.rs:26`) takes no salt parameter, so a salt dependency is unrepresentable
  in the signature.

Close as "recorded, structurally satisfied; no test owed for an unrepresentable
dependency". *(If you prefer a test for the second, it is one assert — but it can
only re-state the type signature.)*

**C7 — F-90, item 2 only** *(correct `p.SecretsResident()`'s contract)* — **subsumed by F-89.**
The predicate was renamed and its contract narrowed in the same change that closed
F-89. `seal/session.go:20-51`:

> "…which is NOT 'no seed material is resident' — on the `ms1` arm (six of the seven
> canonical vectors) it goes false while four string copies of the share are still
> live … §10.2.4 as amended (2026-08-09) resolves the confusion the old name invited
> by **FORBIDDING this predicate as the residency timer's key**: B2b's timer
> (`gui/wipe_guard.go`) keys on the secret SESSION BRACKET's lifetime instead — which
> is also why this function was renamed from `SecretsResident` to `RecordsResident`."

F-90's own scope line already excludes item 2 ("B2c is secret-residency cleanup:
F-88, **F-90 items 1 and 3**, F-94"), so this is bookkeeping, not a decision.
**Items 1 and 3 stay open** — `grep -rn "unlockCodex32Hook"` → 0 hits, and
`unlockEngraveCodex32` still clears only `rec`.

**C8 — F-80's `unlockWarnUnauthenticated` bullet** — **satisfied, unrecorded.**
The entry says it "formats the digest without checking `p.HasHash`". It now does:

```go
// gui/unlock_flow.go:88-93
// Step 3 — the hash, shown ONLY when the payload has a public section.
// HasHash is false exactly when pub_len == 0, and the digest of an empty
// record set is a CONSTANT: showing the same number on every fully
// encrypted payload would teach the operator it is furniture.
if p.HasHash {
    showNotice(ctx, th, "Public Data Hash", unlockHashBody(p))
}
```

The remedy the entry asked for ("a guard costs nothing") is present, with the
rationale in place.

**C9 — F-80's `groupCards` bullet** — **satisfied by the second of its two options.**
The entry: "Either fold them onto `groupRecords` or say in the doc comment that it
is test-facing." The doc comment now says it:

```go
// seal/record.go:429-431
// groupCards is groupRecords' cards-only view: the partition without the
// per-record index. It is what §6.3 is ABOUT -- which records form a card --
// and the two grouping tests assert against it directly.
```

Confirmed still zero production callers, which is the premise:

```
$ grep -rn "groupCards" --include='*.go' .
seal/grouping_test.go:14   (comment)
seal/record_test.go:263,265,323,325   (the two tests)
seal/record.go:429,432     (doc + definition)
```

### Scheduling, not closure — three items whose owning phase has arrived

These are **not** closure candidates; they are overdue-or-due and should be
re-dated rather than left reading as deferred.

| item | phase condition | status of that condition |
| --- | --- | --- |
| **F-65** | "after the encrypted-payload cycle ships; NOT during it" | Shipped. `git tag` → `v0.5.0` (2026-08-11); fork `fork-v0.0.0-g93ee004`. **Due now.** |
| **F-66** | "AFTER the encrypted-payload cycle is GREEN and shipped" | Same. **Unblocked now.** |
| **F-76** | "after B2b; NOT B2a" | B2b merged (fork `93ee004`). **Due now.** |

---

## 3. Wrongly closed — entries the record treats as done that the code does not satisfy

These matter more than the open ones, because nobody is looking at them.

### 3.1 — F-110 is OPEN and OVERDUE, not closed

Your brief lists F-110 among "closed during the cycle, with evidence in their
entries". **The ledger entry itself does not say that, and the shipped code
contradicts it in two places by name.**

The entry (`design/FOLLOWUPS.md:2341-2377`) closes only the `catchup` half and then
says verbatim: *"What remains open: 1. Two non-terminal returns skip the zeroing …
2. `SafePointer.history` grows by `append` …"*. It is also absent from the ledger's
own 2026-08-10 sweep list (`:416`, which names F-79, F-105, F-107, F-108, F-111).

The code names both halves as open F-110 holes, with a measurement:

```go
// gui/engraver.go:126-132  (releaseResumeState)
// TWO non-terminal returns skip this, not one: Engrave returning on ctx.Done
// (the wipe) AND the double-Back return in engraveStopping, where the goroutine
// is still winding down. Neither is covered elsewhere -- the wipe unwind is
// ctx.B.Scrub() + Drawer.Release() and reaches no engrave state. That hole is
// F-110, not a covered case.
```

```go
// engrave/engrave.go:1722-1730  (ClearHistory)
// Knot grows history with a bare `append`, which is an UNFUNNELLED growth site of
// exactly the class op.Buffer's appendArgs/appendRefs fix: every reallocation
// orphans an array still holding the knots written before it, and nothing here
// can reach those. Measured on a real plate under a lockstep driver: 4 orphaned
// arrays holding 15 knots, rising to 23 arrays / 119,891 knots if the driver
// reports no progress so the trim never fires. That residue is F-110, not
// something this function covers.
```

Both halves are **seed-derived geometry**. Owning phase is B2b, which merged with
this open — so per the standard it is **overdue, not deferred**, and it needs an
owning phase in the same bucket as F-88/F-90/F-94/F-104 (post-merge polish and
hardening) or an explicit acceptance like F-83's.

*Note: item 1 carries its own reasoned "skipping is still the right call" (zeroing
under a live goroutine races it), so item 1 may be closeable as an accepted
limitation. Item 2 — up to 119,891 orphaned knots — has no such argument.*

### 3.2 — F-68's closure is mis-attributed; the gap it describes is still open

F-68 is: *"`plan-build-gate.sh` compiles the CLI tests but never runs them."*
The ledger closes it (`:3150`) with: *"closed by `scripts/plan-cite-gate.sh` (`7cdcbfc`),
which resolves every `file:line` and `pkg.Symbol` in a plan against real source."*

That is a different tool solving a different problem. The gate still does not run
the tests:

```
$ grep -n "no-run\|NOT RUN" scripts/plan-build-gate.sh
30:#   2. tests/seal_cli.rs is COMPILED BUT NOT RUN. It drives the `me` binary
163:  if cargo test -p mnemonic-engrave --test seal_cli --no-run 2>&1 \
185:echo "   NOT covered: ... and tests/seal_cli.rs is"
```

**Severity is low and the entry itself says why**: once implementation exists,
`cargo test --all` runs the real suite (180 passed today), so the gap only bites
when reviewing a *future* Rust plan that carries `tests/seal_cli.rs` fragments. The
gate also declares its own blind spot honestly, which is the standard's requirement.
But the record is wrong, and F-68's body explicitly says the concern "binds again at
Plan B's plan review" — which happened, against a different script.

**Suggested disposition:** re-close with the correct attribution — either "superseded
by `plan-build-gate-go.sh` (F-74), which is the Go analogue Plan B actually needed",
or re-open the Rust-side item with a fresh owning phase. Do not leave it reading as
closed by `plan-cite-gate.sh`.

### 3.3 — F-108 is closed in code, but the citation drift it flagged survives

F-108's entry says: *"The `:2651-2656` anchor this entry used is WRONG … the real
sites are `gui/gui.go:2715`, `:2726` and, the one that matters, `:2747`
`s.job.Start()`. **The shipped comment at `gui/unlock_session.go:200` carries the
same drift.**"*

The shipped comment was never corrected. It is now at `:215`:

```
$ grep -n "2651-2656" gui/*.go
gui/unlock_session.go:215:	// return (gui/gui.go:2651-2656 calls Stop() and keeps rendering), so the
```

And `gui/gui.go:2645-2652` is now `DescriptorScreen.Layout`'s nav/`ctx.Frame` tail
ending in `return Plate{}, false` — not the engrave Back path, which lives at
`gui/gui.go:2718-2735`. This is the exact [[comments-outlive-their-conditions]]
class that produced F-119, sitting in a funds-path comment that justifies wipe
ordering. One-line fix; no behaviour change.

### 3.4 — the ledger's own reconciliation table is stale

`design/FOLLOWUPS.md:426-440` — *"THE BURNDOWN LIST — six items, open, owning phase
already passed"* — lists **F-77, F-80, F-84, F-87, F-89, F-93**. Four of those six
now carry CLOSED banners with mutation evidence in their own entries (F-77, F-84,
F-89, F-93), all verified above. The table's closing line — *"Five of the six look
unrecorded rather than undone. They stay OPEN anyway"* — is no longer true of four
of them. Since this table is the thing a future reader greps, it is the highest-value
single edit in the file.

The same section (`:442-456`) already names the root cause and calls it a finding:
closure is marked in three places and three words, so no `grep` reconciles. That
diagnosis is still exactly right, and this triage had to be done entirely by hand
because of it.

### 3.5 — citation drift in still-open entries (minor, but it costs the next reader)

| entry | cites | actual today |
| --- | --- | --- |
| F-62 | `engrave/engrave.go:1126` | `:1148` |
| F-114 | `engrave/engrave.go:1664` | `:1667` |
| F-119 | `backup/backup.go:368` | `:388` |
| F-78 | `codex32_polish.go:49,182,286` | also `:26` and `:30` — 5 sites in that file, 8 across 4 files |

---

## 4. Off-ledger finding: the v0.5.0 release binary reports version 0.4.0

Not an F-number, but it surfaced while dating the "after the cycle ships" phases and
it is a shipped-artifact defect.

```
$ for t in v0.3.0 v0.4.0 v0.5.0; do echo -n "$t -> "; git show "${t}:crates/me-cli/Cargo.toml" | grep -m1 '^version'; done
v0.3.0 -> version = "0.3.0"
v0.4.0 -> version = "0.4.0"
v0.5.0 -> version = "0.4.0"
```

`crates/me-cli/src/main.rs:13` is `#[command(name = "me", version, about)]`, so clap
prints `CARGO_PKG_VERSION`. **The shipped `v0.5.0` binary prints `me 0.4.0`.** The
two prior releases were consistent, so the bump was simply forgotten.

**The preview sidecar gate is *not* broken by this** — I checked, because it was the
obvious second-order risk. `.github/workflows/release.yml:122` derives the injected
`me-preview` version from `Cargo.toml`, not from the tag:

```sh
VERSION="$(grep -m1 '^version' crates/me-cli/Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')"
```

so both halves say `0.4.0` and `main.rs:559`'s exact-match gate passes. The damage is
confined to what the operator sees from `me --version`, and to any future bisect that
trusts it.

---

## 5. What I could not determine, and why

1. **F-119's ordering claim.** I confirmed the comment is unchanged at
   `backup/backup.go:388` and that the caller (`gui/gui.go:2105-2109`) *enumerates*
   all three variants and keeps every one that fits rather than falling back at all —
   which is sufficient to say the comment describes something the code does not do.
   I did **not** re-derive the specific measured claim that "QR-ONLY fails BEFORE
   TEXT-ONLY"; that needs a plate-fit run per variant, which is a new test, and the
   brief lists F-119 as settled.

2. **F-103's attribution to the film.** Unresolvable from code, and the entry already
   says so ("that particular incident's cause is plausible and unproven"). The
   *mechanism* is confirmed live at `gui/run_flow.go:251`; the 2026-08-09 incident's
   cause is not.

3. **F-109's 81 objects.** Cannot be named without either the `gui`-level finalizer
   test the entry specifies (which does not exist — only `gui/op/release_test.go:91`)
   or a device run. I can confirm the work is undone; I cannot shrink the question.

4. **F-58's wedge.** No reproduction exists in the tree, and the entry says the test
   *is* the deliverable. I confirmed the structural hazard is unchanged; I cannot say
   whether the observed wedge is that hazard.

5. **F-65's key-backup state on disk.** I did not read `~/.sh2/`; the entry's
   measurement ("exactly one copy, 223 bytes, sha256 `cd3f86b3214d1b43…`") is from
   2026-08-04 and I treated the operator's home directory as out of scope for a
   code-first read. The phase condition (cycle shipped) I *did* verify.

6. **F-83 and F-92** were not re-examined — both are operator rulings per the brief,
   and I took them as given rather than re-deriving.

7. **`gui` package tests beyond the four I ran.** The brief supplied the full-suite
   result (48 ok, 0 FAIL); I ran only the four tests whose pass/fail *is* the evidence
   for F-78, F-86, F-95 and F-87, plus a positive control on the cite gate. I did not
   re-run the suite.
