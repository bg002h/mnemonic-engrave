# FIX: the sh2 bug-hunt Importants — seedhammer fork device paths, codecs and font tool

Small-fix track for the 22 Important findings the sh2 bug hunt graded in the
seedhammer fork, plus the UNRESOLVED 39 that this round owns. Each fix is the
grade's ruled fix (its "Fixes" table), not the raw triage patch where the
grade amended it.

Sources, all in `design/agent-reports/`:
hunt report (5131d8ec); triage `bughunt-triage-sh2-s1.md` (432334ae), `-s2`
(f7617532), `-s3` (80afb2cd), `-s4` (ec299e64), `-s5` (06c36ced); grade
`bughunt-grade-sh2.md` (b6196244; "Fixes", "Deferred" and "Product rulings"
decide); architect ruling `architect-sh2-s3q3.md` (b4bae63e).

Scope: fork packages `gui`, `bip380`, `nonstandard`, `codex32`, `picobin`,
`driver/otp`, `cmd/vectorfont`, `cmd/controller` (39's seam); in this repo
(mnemonic-engrave) only the 6b/50 delta: the descriptor-seam vectors, their
sha pin, and `design/SPEC_descriptor_input.md` §4.2. The grade's line holds:
"No toolkit manual change is needed: these are device and fork-internal
paths, and no `mr` or `md-cli` flag or printed text changes." Every M and N
is deferred (§9.2). Four points this document could not decide are STOPs
(§10) and are not written into any build item.

S3-Q3: keep `bc/*`. It is upstream code, and F-234 Q2 may use it. UR items stay deferred until wired (architect-sh2-s3q3).

**How this document was checked.** The grade's ruled fixes were applied as
scratch patches to copies of the five triage slice worktrees
(`/scratch/code/shibboleth/sh-worktrees/bughunt-triage-s1..s5`, heads s1
bc7c1cb, s2 1df27de, s3 28d5bbd, s4 6a9fddb, s5 0129a2c) and each slice's RED
rows were run before and after. Patches and logs are in
`/scratch/cache/bughunt/sh2/fixdoc-probe/` (not part of the build):

| patch | slice | sha256 |
|---|---|---|
| `fx1.diff` | s1 | `2c1de758c8187b104c944b63e6d0ef4ab9d61cad611fa9332bbae21ce82fe51c` |
| `fx2.diff` | s2 | `319aac96fa087cf1d41ad2576b30b6560fe69f83bedbeff187d63ce61121622a` |
| `fx3.diff` | s3 | `45770d6a2a265a5249321d37f058bd77601d853e921656cc2612ab53b430f1df` |
| `fx4.diff` | s4 | `d9b40a97c6d043c9c0f788e5ba4d89a8598739fb595e968889bc5914c0738a38` |
| `fx5.diff` | s5 | `9d91ba93e81c28d5d158d0c0c142d16c982135e3148dc3f20a300ba695adce1e` |
| `s39.diff` | 39 seam | `1d853245aaab416c75e102350919da2d3bfc8d6a662b578d1504f67ef17f41c4` |

Go: `export PATH=$HOME/.local/go/bin:$PATH CGO_ENABLED=0` (go1.26.4).
tinygo is not installed here, so the CI tinygo `./cmd/controller` build is
owed to CI. me: `PATH=$HOME/.cargo/bin:$PATH`; CI pins Rust 1.85.0, the
active toolchain here was cargo 1.97.0-nightly.

| check | command (run in the slice worktree or its fixed copy) | result |
|---|---|---|
| RED s1 | `go test -count=1 -run '^TestBughuntSh2' ./gui/` (s1) | 02, 03, 04, 31a, 31b, 32 FAIL with the texts in §6 |
| RED s2 | same, s2 | 21, 22 FAIL |
| RED s3 | `go test -count=1 -run '^TestBughuntSh2' ./bip380/ ./nonstandard/ ./codex32/` | 01, 06a, 06c, 06b, 50a, 50b, 50c, 11 FAIL |
| RED s4 | `go test -count=1 -run '^TestBughuntSh2' ./picobin/ ./driver/otp/` | 07, 13, 14 FAIL |
| RED s5 | `go test -count=1 -run '^TestBughuntSh2' ./cmd/vectorfont/` | 30c, 30d (with the new `c` subtest), 49 FAIL |
| RED 39 | `go test -count=1 -run '^TestBughuntSh239' ./cmd/controller/` (s39 copy, base `writeOTPValues`) | FAIL, see §6 |
| GREEN s1 | RED s1 command on the fx1 copy | 6 PASS, `ok  	seedhammer.com/gui	0.069s` |
| GREEN s2 | on fx2 | 2 PASS, `ok  	seedhammer.com/gui	0.007s` |
| GREEN s3 | on fx3 | bip380 3 PASS `ok ... 0.005s`; nonstandard 4 PASS `0.002s`; codex32 1 PASS |
| GREEN s4 | on fx4 | picobin 1 PASS; otp 2 PASS |
| GREEN s5 | on fx5 | 3 PASS, `ok  	seedhammer.com/cmd/vectorfont	0.002s` |
| GREEN 39 | on the s39 copy | PASS, `ok  	seedhammer.com/cmd/controller	0.001s` |
| tests that change | `go test -count=1 ./...` on each fixed copy | the failures in §7, nothing else |
| font regen | §5 | base and fixed both `generate exit=0`, `diff exit=0` |
| me seam today | `PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked -p mnemonic-engrave -E 'binary_id(/descriptor_seam/) \| test(/descriptor_seam/)'` | `Summary [   0.038s] 19 tests run: 19 passed, 665 skipped` |
| fork seam today | `go test -count=1 -run '^TestDescriptorSeam' ./nonstandard/` (s3) | `ok  	seedhammer.com/nonstandard	0.027s` |
| host build with 39 seam | `go build ./...` (s39 copy) | FAIL: `runtime.main_main·f: function main is undeclared in the main package` (STOP 4) |

## 1. Summary

22 Important fixes plus 39 (resolved by a host seam), in 7 fork packages and
one two-repo delta (6b/50):

| fix | package · function | one line |
|---|---|---|
| 02 | `gui` · `singleSigInputs` | Back on the passphrase keyboard re-asks instead of engraving with "" |
| 03 | `gui` · `bip85DeriveFlow` | same, BIP-85 |
| 04 | `gui` · `seedEntryFlowTypedOnlyTitled` | Back mid-typing no longer confirms a half-typed seed |
| 31a | `gui` · `supplyMultisigPolicyFlow` | Back on the passphrase keyboard ends the supply flow |
| 31b | same | Back on the passphrase question ends the supply flow |
| 32 | `gui` · `recoverSLIP39Flow` | Back on the passphrase question ends recovery |
| 21 | `gui` · `composerSelfCheck` | an unseated slot's origin must be §4f's default |
| 22 | `gui` · `composerPickScreenFrom` | wrapping never leaves the cursor off-page |
| 01 | `bip380` · `Key`, `ParseKey`, `ExtendedKey` | xpub depth and child number survive re-encode |
| 06a | `bip380` · `Parse` | multikey threshold range check |
| 06c | `bip380` · `Parse` | script/type mismatch refused |
| 06b, 50a, 50b, 50c | `nonstandard` · `parseBlueWalletDescriptor` | BlueWallet header, key and threshold checks; Derivation applied to every key |
| 11 | `codex32` · `partsInner` | short data errors instead of panicking |
| 07 | `picobin` · `HashData` | each zero-fill entry hashes its own size word |
| 13 | `driver/otp` · `WriteWhiteLabelString` | string placement keyed by entry, not by the caller's index |
| 14 | `driver/otp` · `ReadWhiteLabelString` | valid bit tested with `&` |
| 30c, 30d, 49 | `cmd/vectorfont` · `parseSegments`, `optimizeSegments` | repeated `s` and relative `l`/`h`/`v`/`c` pen tracking; stale pen in optimize |
| 39 | `cmd/controller` · `writeOTPValues` (new, host seam) | a label-address write error stops the OTP sequence |

Counts: C 0 · I 22 (2, 3, 4, 31a, 31b, 32, 21, 22, 1, 6a, 6b, 6c, 11, 50a,
50b, 50c, 7, 13, 14, 30c, 30d, 49) · M 26 · N 11 · FP 1 (40) · UNRESOLVED 4
(15, 16, 17, 44; 39 resolved by the seam) · open product questions 0 ·
STOPs 4.

## 2. Holder-visible behaviour, per fix

### 2.1 fix-02, fix-03 — passphrase keyboard Back (`gui/singlesig.go` `singleSigInputs`; `gui/bip85.go` `bip85DeriveFlow`)

Behaviour: Back on the passphrase keyboard returns to the passphrase question
instead of continuing with passphrase "". Code: `pass, pok :=
syswPassphraseFlow(ctx, th); if !pok { continue }; passphrase = pass` (fix-03
the same pattern, `continue`). New or changed strings: none. Spec: none.

### 2.2 fix-04 — typed seed Back (`gui/derive_xpub.go` `seedEntryFlowTypedOnlyTitled`)

Behaviour: Back after typing some words no longer returns a partly filled
mnemonic as confirmed. Code: `_, confirmed := inputWordsFlow(...); if
confirmed && !isEmptyMnemonic(mnemonic) {`. Strings: none. Spec: none.

### 2.3 fix-31 (31a, 31b) — supply multisig passphrase (`gui/multisig.go:145` `supplyMultisigPolicyFlow`)

Behaviour (S1-Q2): Back on the passphrase question or the passphrase keyboard
ends the supply flow and returns to the menu. Code: `sel, ok :=
askBIP39Passphrase(ctx, th, ppChoice); if !ok { return }; if sel == 1 { pass,
ok := syswPassphraseFlow(ctx, th); if !ok { return }; passphrase = pass }`.
Strings: none. Spec: none.

### 2.4 fix-32 — SLIP-39 passphrase question Back (`gui/slip39_polish.go:290` `recoverSLIP39Flow`)

Behaviour: Back on the passphrase question ends recovery with no seed. Code:
`psel, pok := ppChoice.Choose(ctx, th); if !pok { return nil, false }`.
Strings: none. Spec: none.

### 2.5 fix-21 — self-check of unseated slots (`gui/composer_selfcheck.go` `composerSelfCheck`)

Behaviour: an unseated slot's declared origin must equal
`md.DefaultOrigin(st.list.Wrapper, account)` at the lowest free account
(taken origins keyed by `composerOriginKey`); otherwise the self-check
refuses. New error string, as the grade rules it:
`"self-check: unseated slot @%d declares %s, not §4f's default %s"`.
This string fails `TestProductionStringsAreDrawable` (the `§` has no glyph);
its final wording is STOP 2. Spec: none.

### 2.6 fix-22 — pick screen wrap (`gui/composer_paged.go` `composerPickScreenFrom`)

Behaviour: after a wrap the cursor lands on a row the page shows. Code: `_,
nd, _ := composerPickPageLayout(ctx, th, dims, lead, lines[start:], -1);
nshown := max(nd-rowBase, 1); if sel >= start+nshown { sel = start }`.
Strings: none. Spec: none.

### 2.7 fix-01 — xpub depth and child number (`bip380/bip380.go`)

Behaviour: a parsed key re-encodes to the same xpub string. `Key` gains
`xpubMeta bool`, `xpubDepth uint8`, `xpubChildNumber uint32`; `ParseKey` sets
them from `xpub.Depth()` and `xpub.ChildIndex()`; `ExtendedKey` uses them
instead of `uint8(len(k.DerivationPath))`. Strings: none. Spec: none.

### 2.8 fix-06a, fix-06c — `bip380` `Parse` checks

New errors, word for word:
- 06a: `if threshold < 1 || threshold > len(keys) { return nil, fmt.Errorf("bip380: multikey threshold %d out of range for %d keys", threshold, len(keys)) }`
- 06c: `if r.Type == Singlesig && !r.Script.Singlesig() || r.Type == SortedMulti && r.Script.Singlesig() { return nil, fmt.Errorf("bip380: %v does not take a %v expression", r.Script, r.Type) }`

On the device both surface through `OutputDescriptor` as the generic
"nonstandard: unrecognized output descriptor format". Spec: none here; their
seam-vector consequence is STOP 3(b).

### 2.9 fix-06b/50 (06b, 50a, 50b, 50c) — BlueWallet parser (`nonstandard/parse.go` `parseBlueWalletDescriptor`)

Behaviour: checks run in this order, each a new error:
1. `"bluewallet: missing Policy header"`
2. `"bluewallet: missing Format header"`
3. `"bluewallet: no keys"`
4. `"bluewallet: invalid threshold %d of %d"`

then `desc.Keys[i].DerivationPath = path` for every key, so a `Derivation:`
header after the key lines applies to all keys. `OutputDescriptor` still
shows the generic "nonstandard: unrecognized output descriptor format". A
Derivation after the keys is now accepted on the device with the path
applied; me still refuses it. Spec: §4.2, §4 below.

### 2.10 fix-11 — codex32 short data (`codex32/codex32.go` `partsInner`)

Behaviour: `New` on too-short data returns `errInvalidLength` instead of
panicking. Code: `if len(res) < 6+checkLen { return nil, errInvalidLength }`.
Strings: none new (existing error). Spec: none.

### 2.11 fix-07 — picobin zero-fill hash (`picobin/picobin.go:328` `HashData`)

Behaviour: each zero-fill entry hashes its own size word. Code: `eidx+8`
becomes `eidx+i*12+8`. Strings: none. Spec: none.

### 2.12 fix-13, fix-14 — OTP white label (`driver/otp/otp.go`)

- 13 (`WriteWhiteLabelString`, :253): `switch idx` becomes `switch uint8(i)`,
  so a string is placed after the UTF-16 product string instead of over it.
- 14 (`ReadWhiteLabelString`, :185): `flags|(0b1<<idx) == 0` becomes
  `flags&(0b1<<idx) == 0`, so an entry with its valid bit clear reads "".

Strings: none. Spec: none.

### 2.13 fix-30c, fix-30d, fix-49 — vectorfont (`cmd/vectorfont/main.go`)

- 30c (`parseSegments`): `p1 := pen.Mul(2).Sub(ctrl2)` becomes `p1 :=
  newPen.Mul(2).Sub(newCtrl2)`.
- 30d (`parseSegments`): `h` and `v` set `pen = p`; the `l` loop uses `p =
  p.Add(newPen).Sub(pen)` for i>0; the `c` loop computes `d :=
  newPen.Sub(pen)` and adds `d` to p1, p2 and p3 for i>0. Relative repeated
  `s` is not covered (STOP 1).
- 49 (`optimizeSegments`): `p0 = p1` before `continue skip`.

Strings: none. Spec: none. Committed font output does not change (§5).

### 2.14 fix-39 — OTP label-address error stops the lock sequence (`cmd/controller`)

Decision: a host seam (no device test needed, no new type or command). New
file `cmd/controller/lockboot_values.go`, `//go:build !refugium`, holding the
`otp*` consts (`otpVolumeLabel="SHII"` and the rest) moved from
`lockboot_default.go` (`tinygo && rp && !refugium`), function vars
`otpWriteWhiteLabelAddr`, `otpWriteWhiteLabelString`, `otpAddBootKey`, and
`func writeOTPValues(keyHash, version string) error`, which returns the
label-address error instead of `fmt.Printf("label addr err: %v", err)` and
then continuing. Removed string: `"label addr err: %v"`. Behaviour: on a
failed label-address write no white-label string and no boot key is written.
The host build consequence is STOP 4. Spec: none.

## 3. Code delta (production)

| file | change |
|---|---|
| `gui/singlesig.go` | fix-02 |
| `gui/bip85.go` | fix-03 |
| `gui/derive_xpub.go` | fix-04 |
| `gui/multisig.go` | fix-31 |
| `gui/slip39_polish.go` | fix-32 |
| `gui/composer_selfcheck.go` | fix-21 (string per STOP 2) |
| `gui/composer_paged.go` | fix-22 |
| `bip380/bip380.go` | fix-01, fix-06a, fix-06c |
| `nonstandard/parse.go` | fix-06b/50 |
| `codex32/codex32.go` | fix-11 |
| `picobin/picobin.go` | fix-07 |
| `driver/otp/otp.go` | fix-13, fix-14 |
| `cmd/vectorfont/main.go` | fix-30c, fix-30d, fix-49 |
| `cmd/controller/lockboot_values.go` (new), `cmd/controller/lockboot_default.go` | fix-39 seam |
| `nonstandard/testdata/` (vendored seam file) | copied byte-identically from me after §8 item 1 |

mnemonic-engrave: `scripts/descriptor-seam-vectors/` output regenerated and
`crates/me-cli/tests/descriptor_seam.rs:46` re-pinned (§8 items 1-2; held by
STOP 3).

## 4. Spec delta (this repo: `design/SPEC_descriptor_input.md` §4.2)

The table (lines 380-388) is unchanged: it records the pre-fix measurement.

Old lead-in (lines 391-392):

> Three of those rows are defects in the Go parser, and `me` must not reproduce
> them:

New lead-in:

> Three of those rows were defects in the Go parser, and `me` must not reproduce
> them. The sh2 bug hunt fixed all three on the device (fix-06b/50; see the
> amendment after item 4):

New paragraph, inserted after item 4 (which ends "...it amends in P3.5, with
the fix that falsifies it.") and before the **NORMATIVE:** paragraph, which
is unchanged:

> **AMENDED (sh2 bug-hunt, fix-06b/50).** `parseBlueWalletDescriptor` now
> refuses, in this order, with `bluewallet: missing Policy header`,
> `bluewallet: missing Format header`, `bluewallet: no keys` and
> `bluewallet: invalid threshold %d of %d`, and applies the `Derivation:`
> path to every key. Defects 1 and 2 are refused on the device; the holder
> still sees the generic "nonstandard: unrecognized output descriptor
> format" from `OutputDescriptor`. Defect 3 is accepted on the device with
> the path applied to every key; `me` still refuses it. The table above
> records the pre-fix measurement, and the seam's "narrowed" covers note is
> dropped for these rows.

## 5. Font regeneration (30c, 30d, 49)

Run in the fork worktree, on the base tree and on the fixed copy (both run):

```
go generate ./font/sh ./font/constant; git status --short font/; git diff --exit-code --stat font/
```

Both report `generate exit=0` and `diff exit=0` with no files listed: the
committed `font/sh` and `font/constant` output does not change, so there is
no glyph to review as a font change. After the fix the build runs the same
command and expects the same result.

## 6. RED rows

Fork rows run with `go test -count=1 -run '^<name>$' ./<pkg>/`. Today's text
is the measured failure on the base tree.

### 6.1 Rows fixed by this build

| package · row | after the build | RED today |
|---|---|---|
| `gui` · `TestBughuntSh202SingleSigPassphraseCancel` | PASS | :71 `Back on the passphrase keyboard continued to the engrave-mode choice with passphrase ""` |
| `gui` · `TestBughuntSh203Bip85PassphraseCancel` | PASS | :108 `...continued to the child-parameter picker with passphrase ""` |
| `gui` · `TestBughuntSh204TypedSeedBackHalfTyped` | PASS | :158 `Back after typing 3 words returned a 12-slot mnemonic with 3 words filled as CONFIRMED (ok=true)` |
| `gui` · `TestBughuntSh231aSupplyMultisigPassphraseCancel` | PASS | :270 `Back on the passphrase keyboard carried on with passphrase "" to "Your seed is at slots"` |
| `gui` · `TestBughuntSh231bSupplyMultisigQuestionBack` | PASS (not skipped) | :290 `Back on the passphrase question carried on with passphrase "" to "Your seed is at slots"` |
| `gui` · `TestBughuntSh232Slip39PassphraseQuestionBack` | PASS | :331 `...recovered a seed with the empty passphrase: ok=true entropy=61cf4d6c0d8a07d8c2fd3cff22432664` |
| `gui` · `TestBughuntSh221SelfCheckUnseatedSlotOrigin` | PASS | :162 ACCEPTED for m/84'/0'/0' and m/48'/0'/7'/2' |
| `gui` · `TestBughuntSh222PickScreenWrapHidesCursor` | PASS | :232 `Button3 returned row 2, but the page on screen draws only rows 0..1 (the cursor was left on a row page 0 does not show).` |
| `bip380` · `TestBughuntSh201XpubDepthChildRewritten` | PASS | :56/:59 `key 0/1 re-encodes to a different xpub` |
| `bip380` · `TestBughuntSh206aParseThreshold` | PASS | :77 thresholds 5, 3, 0, -1 of 2 keys accepted |
| `bip380` · `TestBughuntSh206cScriptTypeMismatch` | PASS | :104 wpkh/pkh/tr(sortedmulti) and wsh(key) accepted |
| `nonstandard` · `TestBughuntSh206bBlueWalletThreshold` | PASS | :31 Policy "0 of 2", "3 of 2", "-1 of 2" accepted |
| `nonstandard` · `TestBughuntSh250aBlueWalletNameOnly` | PASS | :40 `Name-only file accepted: Title="x" Threshold=0 Script=Unknown keys=0` |
| `nonstandard` · `TestBughuntSh250bBlueWalletNoFormat` | PASS | :58 `file without Format accepted: Script=Unknown (unknown=true), 2 keys` |
| `nonstandard` · `TestBughuntSh250cBlueWalletKeyBeforeDerivation` | PASS | :84 `key 0: origin path m with Derivation after the key lines, want m/48h/0h/0h/2h (or a refusal)` |
| `codex32` · `TestBughuntSh211NewShortDataPanics` | PASS | :32 `New panicked: runtime error: slice bounds out of range [6:5]` |
| `picobin` · `TestBughuntSh207PicobinLoadMapZeroFill` | PASS | :57 `HashData = 78e910ef5bbb0202590af3cabb353ac2c63bf5e2876e00687d9978f2c3dae25a, want 0230c6046d8fbd63fa9cfa7f931dacd7bc462819256d80313802c580e5fb5d84 (each zero-fill entry hashes its own size word)` |
| `driver/otp` · `TestBughuntSh213WhiteLabelSpaceUTF16Neighbour` | PASS | :54 `new string placed at table row 21, want >= 26 (overlaps the UTF-16 product string at rows 16..25)`; :63 `UTF-16 product string overwritten: 41 00 41 00 41 00 41 00 41 00 53 48 49 49 41 00 41 00 41 00, want 41 00 41 00 41 00 41 00 41 00 41 00 41 00 41 00 41 00 41 00` |
| `driver/otp` · `TestBughuntSh214WhiteLabelValidBit` | PASS | :99 `ReadWhiteLabelString = "SHII" for an entry whose valid bit is clear, want ""` |
| `cmd/vectorfont` · `TestBughuntSh230cRepeatedS` | PASS | :72 `second curve's first control point = {0 0}, want {3 3}` |
| `cmd/vectorfont` · `TestBughuntSh230dRepeatedRelative` (subtests `l`, `h`, `v` and the new `c` subtest `{"m0,0 c1,0 1,0 1,0 1,0 1,0 1,0", bezier.Pt(2, 0)}`) | PASS, all four subtests | :97 `"m0,0 l1,0 1,0" ends at {1 0}, want {2 0}`; `"m0,0 h1 1" ends at {1 0}, want {2 0}`; `"m0,0 v1 1" ends at {0 1}, want {0 2}`; new subtest `"m0,0 c1,0 1,0 1,0 1,0 1,0 1,0" ends at {1 0}, want {2 0}` |
| `cmd/vectorfont` · `TestBughuntSh249OptimizeStalePen` | PASS | :118 `optimized path [{5 5} {7 5} {6 9}], want [{5 5} {7 5} {6 5} {6 9}] (the backtrack to (6,5) must survive)` |
| `cmd/controller` · `TestBughuntSh239LockBootStopsOnLabelAddrError` (new file `cmd/controller/bughunt_sh2_39_test.go`, written with the seam) | PASS | stdout `label addr err: otp: unsupported modification`; :22 `writeOTPValues = <nil>, want otp: unsupported modification`; :25 `after a failed label address write: 8 string writes, 1 boot key writes; want 0, 0` |

The 30d `c` subtest is added to the existing row's table; 39's row is new,
written RED against the seam with base `writeOTPValues` behaviour first.

### 6.2 Deferred rows (each gets `t.Skip("deferred-minor: <slug>")`, body unchanged)

After the build each reports SKIP with that text. Slugs as §9.2.

| package · row | slug |
|---|---|
| `gui` · `TestBughuntSh209ScannerOverflowSticky` | `sh-nfc-scanner-overflow-sticky` |
| `gui` · `TestBughuntSh233MultisigVerifyPassphraseCancel` | `sh-multisig-verify-pp-back` |
| `gui` · `TestBughuntSh242BuildWhichMd1Back` | `sh-build-which-md1-back` |
| `gui` · `TestBughuntSh248ComposerSeatResumeBack` | `sh-composer-resume-back-seat` |
| `gui` · `TestBughuntSh205Md1GatherIndexPastTotal` | `sh-md1-gather-index-past-total` |
| `gui` · `TestBughuntSh223JobTimeCarriesMinutesIntoHours` | `sh-jobtime-60-min` |
| `gui` · `TestBughuntSh224TruncateForDisplayAtMostN` | `sh-truncate-doc` |
| `gui` · `TestBughuntSh245LegacyFirstKeylessPathRefusedAtCreation` | `sh-legacy-keyless-path-at-creation` |
| `gui` · `TestBughuntSh251PayloadMergeNeedsIdenticalBytes` | `sh-payload-merge-same-txid` |
| `bc/fountain` · `TestBughuntSh210aSeqLenZero`, `10bDataLengthMismatch`, `10cHugeSeqLen`, `10dSeqNumZero` | `sh-ur-fountain-bounds` (the existing `t.Skip("RLIMIT_AS child needs linux")` at :110 stays) |
| `bc/urtypes` · `TestBughuntSh236HDKeyIndexAtLeast2p31` | `sh-ur-hdkey-index-range` |
| `bc/urtypes` · `TestBughuntSh237RangeDerivationShape` | `sh-ur-range-shape` |
| `bc/urtypes` · `TestBughuntSh238ScriptTypeMismatch` | `sh-ur-script-type-mismatch` |
| `bip32` · `TestBughuntSh235ParsePathUnhardenedOverflow` | `sh-parsepath-unhardened-overflow` |
| `bip39` · `TestBughuntSh212ValidPanicsOnEmptyAndLong` | `sh-bip39-valid-bounds` |
| `cmd/policyprobe` · `TestBughuntSh252OneChunkOfSetKeepsRealError` | `sh-md1-single-chunk-error` |
| `codex32` · `TestBughuntSh247NewSeedOutputRejectedByNew` | `sh-codex32-newseed-length` |
| `md` · `TestBughuntSh234PkhLeafDescribedAsPk` | `sh-tapleaf-pkh-as-pk` |
| `nfc/ndef` · `TestBughuntSh218NdefEmptyTextURIRecord` | `sh-ndef-empty-record` |
| `nfc/type2` · `TestBughuntSh208Type2LargeTag` | `sh-type2-large-tag` |
| `uf2` · `TestBughuntSh219PayloadSizeOverData` | `sh-uf2-payload-size` |
| `uf2` · `TestBughuntSh220StartAddrZero` | `sh-uf2-start-addr-zero` |
| `bezier` · `TestBughuntSh241SampleNoOrigin` | `sh-bezier-degenerate-sample` |
| `bezier` · `TestBughuntSh243BoundsEmpty` | `sh-bounds-empty-inverted` |
| `bspline` · `TestBughuntSh226InterpolateNil` | `sh-bspline-nil` |
| `cmd/assets` · `TestBughuntSh229AssetsPalette256` | `sh-assets-palette-256` |
| `cmd/biptool` · `TestBughuntSh228Bip39ShortSeed` | `sh-biptool-short-seed` |
| `cmd/glyphtrace` · `TestBughuntSh246ShortChannelNotWorst` | `sh-glyphtrace-short-channel` |
| `cmd/vectorfont` · `TestBughuntSh230aSpliceFirstLine` | `sh-vectorfont-first-line` |
| `cmd/vectorfont` · `TestBughuntSh230bRunMissingInput` | `sh-vectorfont-missing-input` |
| `image/alpha4` · `TestBughuntSh225Alpha4WideSpan` | `sh-alpha4-wide-span` |
| `internal/golden` · `TestBughuntSh227aRoundTrip`, `27aCompareBSplineUpdateThenCheck` | `sh-golden-roundtrip` |
| `internal/golden` · `TestBughuntSh227bZeroCountHeader` | `sh-golden-zero-count` |
| `internal/golden` · `TestBughuntSh227cVectorizeAspect` | `sh-golden-svg-aspect` |

`driver/ap33772s` · `TestBughuntSh240LimitCurrentZero` is FP and passes
today; no skip. Rows 15, 16, 17 and 44 have no host row (UNRESOLVED,
`sh-needs-seam-tinygo-drivers`).

Counts: 23 rows fixed (22 existing plus new 39), 37 skipped, 1 FP passing.

## 7. Tests that change

| test | today | after the build |
|---|---|---|
| `gui` · `TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput`, case "the decoded md1 puts two slots at ONE origin with no fingerprints" (`composer_selfcheck_test.go:167`) | want-substring "without two distinct fingerprints" | the case now gets `self-check: unseated slot @1 declares m/48h/0h/0h/2h, not §4f's default m/48'/0'/1'/2'`; its want-substring changes to fix-21's string (final wording per STOP 2) |
| `gui` · `TestProductionStringsAreDrawable` (`font_coverage_test.go:239`) | passes | with the grade's string it fails: `composer_selfcheck.go:200 carries '§' (U+00A7): no face has a glyph for it`, :245 `1 undrawable rune site(s)` (STOP 2) |
| `bip380` · `TestDescriptors` (`bip380_test.go:202`, 6 descriptors) | DeepEqual on `Key` | expectations gain `xpubMeta`, `xpubDepth`, `xpubChildNumber` per key (fix-01) |
| `nonstandard` · `TestDescriptors` (`parse_test.go:107`, 3 BlueWallet fixtures) | DeepEqual on `Key` | same new fields (fix-01) |
| `nonstandard` · `TestDescriptorSeamDeviceColumn` (`descriptor_seam_test.go:158`) | passes | 11 rows flip with `device admits = false, want true (OutputDescriptor err = nonstandard: unrecognized output descriptor format)`: 2 graded BlueWallet rows (`no-format-header`, `zero-cosigner-lines`) and 9 from 06a/06c (STOP 3) |

No other fork test fails on the fixed copies (`go test -count=1 ./...` per
copy; s4 all ok).

## 8. Build impact

Each item is one `owed-to-build` line in the §9.1 entry.

1. **Fix the fork (§3)**, me side of 6b/50 first: regenerate
   `scripts/descriptor-seam-vectors/` output for the two graded BlueWallet
   rows (`no-format-header`, `zero-cosigner-lines`) and re-pin its sha256
   (today `ed3706e29fde0ee86e13accd0ecc6492ea2ad9391ff2d911d798d69bb03760f4`)
   in `crates/me-cli/tests/descriptor_seam.rs:46`. Held by STOP 3.
2. Copy that output byte-identically to the fork's `nonstandard/testdata/`
   and re-pin the same sha in `nonstandard/descriptor_seam_test.go:42`
   (`wantDeviceTrue = 40`, `wantDeviceFalse = 34` at :73-74 move with it).
   Held by STOP 3.
3. Apply the §4 spec delta to `design/SPEC_descriptor_input.md` §4.2.
4. Apply the fork fixes in §3 and the §7 test expectations (fix-21 string per
   STOP 2).
5. Add the 39 seam (`cmd/controller/lockboot_values.go`, the move out of
   `lockboot_default.go`) and its row `bughunt_sh2_39_test.go`; host
   `go build ./...` must still pass (STOP 4).
6. Update the comment at `cmd/emu/confinement_test.go:89-96`, which
   describes the OTP consts as living only in `lockboot_default.go`.
7. CI's tinygo `./cmd/controller` build and `GOOS=js GOARCH=wasm go vet
   [-tags refugium] ./cmd/emu/` pass on the head (tinygo is not available
   locally).
8. Run §5's font regeneration; expect no diff.
9. Add `t.Skip("deferred-minor: <slug>")` to every §6.2 row.
10. Append §9 to `design/FOLLOWUPS.md` and the F-234 sentence (§9.3).

**Gate (not a change).** Fork: `go test -timeout 20m ./...`, oraclelive
`-run '^$'`, the wasm vets, tinygo (CI), and every §6.1 row by exact name
with its count quoted (23). me: `cargo nextest run --locked`, `cargo test
--locked --doc`, `cargo fmt --check`, `cargo clippy --locked --all-targets --
-D warnings`, and the seam filter of the "How checked" table. No mutation
gate (Go); the whole-diff review keeps both slices.

## 9. FOLLOWUPS entries (exact text)

### 9.1 Append to `design/FOLLOWUPS.md` (owed to this build)

~~~markdown
### `sh2-bughunt-importants` — the sh2 bug-hunt Importants (repo: **seedhammer fork + mnemonic-engrave**; owning phase: **sh2 FIX build**)

- **Surfaced:** sh2 bug hunt; grade `design/agent-reports/bughunt-grade-sh2.md`; FIX `design/FIX_bughunt_sh2.md`.
- **Grade:** I ×22 (2, 3, 4, 31a, 31b, 32, 21, 22, 1, 6a, 6b, 6c, 11, 50a, 50b, 50c, 7, 13, 14, 30c, 30d, 49) plus 39 (host seam).
- **Companion:** seedhammer fork (`gui`, `bip380`, `nonstandard`, `codex32`, `picobin`, `driver/otp`, `cmd/vectorfont`, `cmd/controller`).
- **Build impact (1):** me seam vectors regenerated for the graded BlueWallet rows and sha re-pinned in `crates/me-cli/tests/descriptor_seam.rs`: `owed-to-build`
- **Build impact (2):** fork `nonstandard/testdata/` copied byte-identically and sha re-pinned in `nonstandard/descriptor_seam_test.go`: `owed-to-build`
- **Build impact (3):** `SPEC_descriptor_input.md` §4.2 lead-in and AMENDED paragraph per FIX §4: `owed-to-build`
- **Build impact (4):** fork fixes per FIX §3 and test expectations per FIX §7: `owed-to-build`
- **Build impact (5):** 39 host seam and its RED row; host `go build ./...` passes: `owed-to-build`
- **Build impact (6):** `cmd/emu/confinement_test.go:89-96` comment updated: `owed-to-build`
- **Build impact (7):** CI tinygo and wasm vet green on the head: `owed-to-build`
- **Build impact (8):** font regeneration shows no diff: `owed-to-build`
- **Build impact (9):** `t.Skip("deferred-minor: <slug>")` on every deferred row: `owed-to-build`
- **Build impact (10):** FOLLOWUPS deferred entries and the F-234 cross-link appended: `owed-to-build`
- **Status:** `open`
- **Tier:** v1
~~~

### 9.2 Append to `design/FOLLOWUPS.md` (status `deferred-minor`)

Each entry has this shape; the table gives the fields that vary. "Fix (for
later)" is the grade's fix-NN; "RED row" is the §6.2 row.

~~~markdown
### `<slug>` — <What> (repo: **seedhammer fork**; owning phase: **<owner>**)

- **Surfaced:** sh2 bug hunt, finding <#>; grade `design/agent-reports/bughunt-grade-sh2.md`.
- **Where:** <package>
- **What:** <What>
- **Fix (for later):** the grade's fix-<NN>.
- **RED row:** <row>, skipped with `t.Skip("deferred-minor: <slug>")`.
- **Why deferred:** graded <M|N>.<trigger>
- **Status:** `deferred-minor`
- **Tier:** v1
~~~

| # | grade | slug | What | owner |
|---|---|---|---|---|
| 9 | M | `sh-nfc-scanner-overflow-sticky` | The NFC scanner keeps the overflow error after the oversized tag ends; clear on EOF or error (fix-09). | end-of-project minors |
| 33 | M | `sh-multisig-verify-pp-back` | Back on the verify-flow passphrase keyboard goes forward to "not checked yet"; step back (fix-33). | end-of-project minors |
| 42 | M | `sh-build-which-md1-back` | Back on "Which md1?" is taken as Full policy; abort the flow (fix-42). | end-of-project minors |
| 48 | M | `sh-composer-resume-back-seat` | Back on the first resumed slot releases @1's seat; keep it (fix-48). | end-of-project minors |
| 5 | M | `sh-md1-gather-index-past-total` | A chunk index past the total is accepted; classify as gatherIgnored (fix-05). | end-of-project minors |
| 23 | N | `sh-jobtime-60-min` | Job time prints "60 min" or "2 h 0 min"; carry into hours and print "2 h" (fix-23). | end-of-project minors |
| 24 | N | `sh-truncate-doc` | The truncateForDisplay doc says at most n; the contract is n plus the marker; fix the doc (fix-24). | end-of-project minors |
| 45 | M | `sh-legacy-keyless-path-at-creation` | A legacy keyless first path is refused at finish, not at creation (fix-45). | end-of-project minors |
| 51 | M | `sh-payload-merge-same-txid` | Signed and unsigned copies of one txid count as two candidates; merge on txid (fix-51). | end-of-project minors |
| 10a-d | M | `sh-ur-fountain-bounds` | The UR fountain decoder panics or OOMs on seqLen 0, a length mismatch, a huge seqLen or seqNum 0; latent, becomes I when wired (fix-10). | UR wiring (trigger below) |
| 12 | M | `sh-bip39-valid-bounds` | bip39 `Valid` panics on 0 or 27 words (fix-12). | end-of-project minors |
| 34 | M | `sh-tapleaf-pkh-as-pk` | TapLeavesChunks describes a pkh leaf as pk; latent, becomes I when wired (fix-34). | end-of-project minors |
| 35 | M | `sh-parsepath-unhardened-overflow` | ParsePath wraps an unhardened index of 2^31 or more (fix-35). | end-of-project minors |
| 36 | M | `sh-ur-hdkey-index-range` | A UR HD key index of 2^31 or more wraps; latent, becomes I when wired (fix-36). | UR wiring (trigger below) |
| 37 | M | `sh-ur-range-shape` | The UR encoder's range shape is refused by its own parser; latent, becomes I when wired (fix-37). | UR wiring (trigger below) |
| 38 | M | `sh-ur-script-type-mismatch` | UR accepts wpkh or pkh(sortedmulti); latent, becomes I when wired (fix-38). | UR wiring (trigger below) |
| 47 | M | `sh-codex32-newseed-length` | NewSeed emits lengths that New refuses; add a library check behind F-116 (fix-47). | F-116 |
| 52 | N | `sh-md1-single-chunk-error` | A lone md1 chunk reports "not supported" instead of "chunk set incomplete" (fix-52). | end-of-project minors |
| 8 | M | `sh-type2-large-tag` | A Type 2 read stops at 848 bytes with a clean EOF; return an error or read it all (fix-08). | end-of-project minors |
| 18 | M | `sh-ndef-empty-record` | Minimal or zero-length Text/URI records are mis-sized (fix-18). | end-of-project minors |
| 19 | M | `sh-uf2-payload-size` | A uf2 payload size of 477..479 panics; refuse (fix-19). | end-of-project minors |
| 20 | M | `sh-uf2-start-addr-zero` | A uf2 at address 0 is reported as non-contiguous (fix-20). | end-of-project minors |
| 25 | N | `sh-alpha4-wide-span` | alpha4.New panics on very wide bounds (fix-25). | end-of-project minors |
| 26 | N | `sh-bspline-nil` | InterpolatePoints(nil) panics (fix-26). | end-of-project minors |
| 27a | M | `sh-golden-roundtrip` | The golden test encoder does not round-trip (fix-27a). | end-of-project minors |
| 27b | N | `sh-golden-zero-count` | The golden decoder accepts a zero-count header (fix-27b). | end-of-project minors |
| 27c | N | `sh-golden-svg-aspect` | The golden SVG width is 0 from integer aspect math (fix-27c). | end-of-project minors |
| 28 | M | `sh-biptool-short-seed` | biptool derive -path m bip39 -words 12 panics (fix-28). | end-of-project minors |
| 29 | M | `sh-assets-palette-256` | The assets tool accepts 256 colours (fix-29). | end-of-project minors |
| 30a | M | `sh-vectorfont-first-line` | vectorfont panics on a path starting with a line (fix-30a). | end-of-project minors |
| 30b | N | `sh-vectorfont-missing-input` | vectorfont reports EOF for a missing input file (fix-30b). | end-of-project minors |
| 41 | N | `sh-bezier-degenerate-sample` | A degenerate bezier samples the origin (fix-41). | end-of-project minors |
| 43 | N | `sh-bounds-empty-inverted` | Bounds.Empty is wrong for an inverted y (fix-43). | end-of-project minors |
| 46 | N | `sh-glyphtrace-short-channel` | glyphtrace ranks a short channel worst (fix-46). | end-of-project minors |
| S1-Q2 | N | `sh-supply-reask` | Back in supply multisig could re-ask the passphrase question instead of returning to the menu. | end-of-project minors |
| 15, 16, 17, 44 | UNRESOLVED | `sh-needs-seam-tinygo-drivers` | pio, st25r3916 and clrc663 reports are UNRESOLVED and need a host seam or a device test. | device-test work |

**UR wiring trigger (architect fold 2).** The "Why deferred" line of
`sh-ur-fountain-bounds` (fix-10), `sh-ur-hdkey-index-range` (fix-36),
`sh-ur-range-shape` (fix-37) and `sh-ur-script-type-mismatch` (fix-38) ends
with: "owned by the first change that imports `seedhammer.com/bc/` from
outside `bc/` (a UR scan door, or F-234 choosing UR); that change applies
fix-NN, un-skips the row and regrades to I first", with NN its fix number.
Entry 37's What also carries "becomes I when wired" (as in the table).

**Architect notes 3 and 4** (ungraded; same shape, "RED row: none yet; one
is written when `bc/*` is wired", same trigger and owner):

| slug | Where | What |
|---|---|---|
| `sh-ur-hdkey-depth-noop` | `bc/urtypes/urtypes.go` `hdkeyFor` (from line 86) | The UR HD key decoder treats depth as a no-op. Ungraded; needs a RED row when wired. |
| `sh-ur-multisig-threshold-unbounded` | `bc/urtypes/urtypes.go`, around line 403 | `desc.Threshold = m.Threshold` is unbounded (the UR counterpart of 6a). Ungraded; needs a RED row when wired. |

### 9.3 F-234 cross-link (architect fold 5)

Anchor: `design/FOLLOWUPS.md` F-234 (heading at line 9848), open question 2
at line 9950, which begins "2. **Raw bytes or UR?** The fork vendors
`bc/ur`. UR is self-describing and multi-part, which is worth something for
recovery, at a size cost — and it is arguably a *third* format to know, which
cuts against the whole point." Append to that item, as its last sentence:

> Choosing UR wires the fork's `bc/*`, which carries deferred defects `sh-ur-*` (sh2 bughunt); that phase takes them first.

## 10. STOPs

Not build items; each needs a ruling before the build starts its item.

1. **Relative repeated `s` (vectorfont).** `m0,0 s1,0 1,0 1,0 1,0` ends at
   {0 0} on base and {1 0} after the ruled fix-30d/30c; correct is {2 0}. The
   grade extended 30d to `c` only; whether 30d also covers relative `s` (fix
   and subtest) is undecided.
2. **fix-21's ruled string contains `§`.** `"self-check: unseated slot @%d
   declares %s, not §4f's default %s"` fails `TestProductionStringsAreDrawable`
   (`composer_selfcheck.go:200 carries '§' (U+00A7): no face has a glyph for
   it`). The replacement wording (or a glyph) is undecided.
3. **The 6b/50 seam regeneration exceeds the graded delta.** (a) `rows.py`
   lacks the 2 F-530 rows, so a regeneration today writes 72 rows, not 74
   (`seam-fix.json`: 72 rows, sha `e1a426c4…`). (b) 06a/06c flip 9 more
   device rows (tr/wpkh/pkh/sh-wpkh-sortedmulti, wsh-of-key, sh-of-key,
   threshold-zero/-negative/-exceeds-keys), touching spec §4.7, §2.3 and §7
   (lines 984, 1519, 1664-1666, 1735, 1887, 2054); the device column becomes
   29/45. (c) The `panic:encode` marker goes stale (`wantPanicEncode` at
   :175, `POP.device_probe`, `rows.py:291/295`, spec §7 line 1664). (d) The
   16-key canonical and the me sha both change. All are beyond the two-repo
   delta the brief allows.
4. **The 39 seam breaks host `go build ./...`** (`function main is undeclared
   in the main package`). A host stub `main` would fix it (precedent
   `cmd/emu/main_notjs.go`), but that is a new file whose name, build tag
   and wording are undecided.
