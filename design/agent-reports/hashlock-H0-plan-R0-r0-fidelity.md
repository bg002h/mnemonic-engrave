# H0 plan — R0 round 0, fidelity + design lens

**Artifact:** `design/IMPLEMENTATION_PLAN_hashlock_H0_reader_guards.md` at engrave `b0af794`.
**Trees read:** mnemonic-engrave `e06e29d` (+ the plan commit), seedhammer fork main `839fa5aa`, mnemonic-secret `4dbff0b` (spec §1/§9/§12/§13/§14), ms-codec `0.7.0` from the cargo registry.
**Method:** source tracing + machine checks; no scratch copies, no test runs, no repo writes.

**Answer to the one question: NO.** Two operator-facing doors on the flashed device still confirm a kind-`0x03` plate as a "Codex32 Secret" and cut it through `backup.EngraveSeedString`, and neither is mentioned anywhere in the plan (C-1). Separately, the predicate the plan defines is wider than the kind it means to catch, and makes roughly 1 in 256 of the *legitimate* codex32 strings the device deliberately accepts inert — including refusing a whole sealed payload as "Payload unreadable." (C-2). The plan's cited facts about `me` are true; three of its Go/Rust fragment anchors are not (I-1).

**Counts: 2 Critical / 5 Important / 2 Minor / 0 Nit — NOT GREEN.**

---

## Item 1 — the reader/engrave site enumeration

Every site in the fork that turns a codex32 string into a seed, a display of a seed, or an engrave. `codex32.New(` ×9, `ClassCodex32Secret` ×17, `DecodeMS1(` ×7, `EngraveSeedString` ×5 (non-test), traced individually.

| site | reachable by a `0x03` string? | guarded by (after the plan)? |
| --- | --- | --- |
| `sysw/classify.go:123` (`isStrictMs1`) | yes | **plan Task 2 Step 4** ✓ |
| `seal/record.go:212` (`seal.Classify`) | yes | **plan Task 2 Step 5** ✓ |
| `gui/unlock_session.go:187` (`unlockEngraveCodex32` → `:196 EngraveSeedString`) | only via `seal.Classify` | `seal.Classify` + **plan Task 2 Step 7** ✓ |
| `gui/gui.go:2869` (`newInputFlow`, `syswOffer(ClassCodex32Secret)` → `codex32.New`) | only via `sysw.Classify` | `isStrictMs1` ✓ |
| `gui/transaction.go:280` (`txClassName`) | only via `sysw.Classify` | `isStrictMs1` ✓ |
| `gui/composer_door.go:45` (inert/seed counts) | only via `sysw.Classify` | `isStrictMs1` ✓ |
| `gui/sysw_admit.go:33-74` (per-program allow-lists) | only via `sysw.Classify` | `isStrictMs1` ✓ |
| **`gui/scan.go:89`** (NFC door → `codex32.String`) | **YES, directly** | **NOTHING — C-1** |
| **`gui/codex32_polish.go:266`** (`validateMStar`, typed `M*1 STRING`) | **YES, directly** | **NOTHING — C-1** |
| `gui/gui.go:2556` (`engraveObjectFlow` case `codex32.String`) | yes, from both rows above | **NOTHING — C-1** |
| `gui/codex32_polish.go:218` (`engraveCodex32`) → `gui/gui.go:2814` (`backupSeedStringFlow`) → `backup.EngraveSeedString` | yes | **NOTHING — C-1** |
| `gui/ms1_decode.go:22` (`ms1DecodeFlow`, "Show secret") | no | `codex32.DecodeMS1` still returns `errMSBadPrefix` on `0x03` ✓ |
| `gui/codex32_polish.go:106` (`showSecret` probe in `confirmCodex32Flow`) | no | `DecodeMS1` ✓ |
| `gui/singlesig_verify.go:185`, `gui/multisig_verify.go:1237` | no | `DecodeMS1` → *"That isn't a valid ms1 secret share."* ✓ |
| `bundle/verify.go:134-138` (`ms1Entropy`) | no | `DecodeMS1` ✓ |
| `cmd/biptool/main.go:346`, `:402` | n/a | host developer tool, not firmware |
| `me` `crates/me-cli/src/seal/record.rs:176` (`validate_record`) | yes | ms-codec 0.7 prefix gate; **plan Task 1** pins it ✓ |
| `me` `crates/me-cli/src/sysw/mod.rs:232` (`sysw::classify` → `validate_record`) | yes | same ✓ (but see I-3) |

---

### C-1 — Two engrave doors are unguarded; the plate is still confirmed as a "Codex32 Secret" and cut by `EngraveSeedString`

**Plan section:** Goal; Architecture ("the one engrave path"); Task 2 Step 7; Self-review item 1.

**Trace, both doors ending at the same call the plan calls "the one call that cuts metal":**

- **NFC door.** `gui/nfc_scan.go:73` polls → `gui/scan.go:89` `} else if s, err := codex32.New(string(buf)); err == nil { return s, nil }` → `gui/gui.go:2238` `return startScreenAction{scan: cnt}, true` → `gui/gui.go:2151` `engraveObjectFlow(ctx, th, obj)` → `gui/gui.go:2555-2556` `case codex32.String: return engraveCodex32(...)`.
- **Typed door.** Start screen → Backup Wallet → `newInputFlow` falls through the two `syswOffer` calls to the `M*1 STRING` row → `gui/gui.go:2892` `inputCodex32Flow` → `gui/gui.go:1283` `validateMStar` → `gui/codex32_polish.go:266` `codex32.New(frag)` → returned as `codex32.String` → the same `engraveObjectFlow` case.

Then, identically for both: `gui/codex32_polish.go:218` `engraveCodex32` → `confirmCodex32Flow` renders the title **"Confirm Codex32 Secret"** and the line **"Unshared secret (S)"** (`gui/codex32_polish.go:207-211`, since `ParsePrefix` reports `Unshared` for threshold `0`/index `s`) → `codex32Engrave` → `gui/codex32_polish.go:231-233`:

```go
			id, _, _ := scan.Split()
			s := backup.SeedString{Title: id, Seed: scan.String(), Font: constant.Font}
			backupSeedStringFlow(ctx, th, s)
```

`Split()` returns `hash`, so the plate is titled **HASH** and cut by `backup.EngraveSeedString` (`gui/gui.go:2816`) — the seed-plate layout, the seed QR, the same function the plan guards one caller of. `DecodeMS1` never runs on this path, so the codec's own prefix refusal never fires; the only `DecodeMS1` call in `confirmCodex32Flow` (`:106`) merely *withholds* the "Show secret" button.

**This is the same defect the spec measured, on a strictly easier door.** §9's table records `unlockEngraveCodex32` as "NOT a refusal"; that path needs a sealed payload, a passphrase and a ~31 s KDF. These two need an NFC tag, or a keyboard, from the start screen. And §12 item 7 states the acceptance the plan must meet: *"A `0x03` single fed to the flashed device is INERT — `sysw.Classify` is not `ClassCodex32Secret` **and no engrave path offers it**"*. Two engrave paths still offer it. The plan never mentions `gui/scan.go`, `validateMStar`, `engraveCodex32` or `backupSeedStringFlow` (grep of the plan for `scan|polish|validateMStar|engraveCodex32|nfc` returns only `TestClassifyMirrorsScanBranchOrder`).

**A documented invariant also breaks silently.** `seal/record.go:95-98`: *"The list and the ORDER below are taken from Scan (gui/scan.go:28-81) and must stay in step with it. Order is load-bearing, not cosmetic."* Task 2 Step 5 narrows `seal.Classify` and leaves `Scan` wider, so the mirror is false. The test named for it — `TestClassifyMirrorsScanBranchOrder`, `seal/record_test.go:403-419` — is a table over `Classify` alone and never calls `Scan`, so the plan's new `{sealPreimagePlate, ClassUnknown}` row passes while the property the test is named for no longer holds.

**SUGGESTION.** Guard the shared choke point rather than three call sites: refuse in `engraveCodex32` (`gui/codex32_polish.go:218`) before `confirmCodex32Flow`, with the same named message Step 7 uses; and refuse at both doors so the operator is told at the point of entry — `gui/scan.go`'s `codex32.New` arm (fall through to `errScanUnknownFormat`, or a named refusal), and `validateMStar`'s `ms` arm (`obj=nil, valid=false`, with an ms-specific feedback string). Add the `{sealPreimagePlate, …}` expectation to a `Scan`-driven test as well as the `Classify` table, or the mirror invariant's comment should be amended to say the two now differ and why. Cheap harness for the NFC half already exists: `gui/nfc_scan_test.go:105-106` (`new(scanner)` + `oneShotNFC`).

---

### C-2 — `IsPreimage` reads *any* ms1 string's first payload byte, so ~1/256 of legitimate shares and plain BIP-93 secrets become inert — including a whole sealed payload refused as "Payload unreadable."

**Plan section:** Task 2 Step 2 (the predicate); Global Constraints, "Minimal narrowing".

The plan defines:

```go
func IsPreimage(s String) bool {
	d := s.Seed()
	return len(d) > 0 && d[0] == msPrefixPreimage
}
```

`String.Seed()` is `s.parts().data()` (`codex32/codex32.go:386-388`) — **that string's own data part**, whatever kind of string it is. Three populations reach the three call sites:

1. **K-of-N shares.** A share's data part is an SSS evaluation, not an m-format payload. The fork already says so, on the function directly above the plan's insertion point (`codex32/mspayload.go:28-33`): *"Callers MUST pass only the unshared secret — a K-of-N share carries an SSS-evaluated point, not an m-format payload."* With random ids and coefficients each share's leading byte is effectively uniform, so ≈1 share in 256 answers `IsPreimage == true`.
2. **Plain BIP-93 secrets** (48/74 characters), which the plan's own "Minimal narrowing" paragraph promises to leave untouched. Their data part *is* the master seed, so `Seed()[0]` is seed byte 0 — ≈1 in 256 again.
3. Constellation `entr`/`mnem` singles are safe (`Seed()[0]` is `0x00`/`0x02` by construction). The breakage falls entirely on the acceptance the plan promised not to touch.

**The spec says this check is singles-only.** §1, rule 2: *"The check applies to singles only: a share-set's id is random by construction and names no kind."* The plan's predicate has no `Unshared` test, no id test and no length test.

**Consequences, traced:**

- `sysw.isStrictMs1` → `ClassUnknown`: the record is dropped from every program's allow-list (`gui/sysw_admit.go:33-74`), and `composer_door.go:45` counts it as "inert".
- `seal.Classify` → `ClassUnknown` → `permitted(SectionEncrypted, ClassUnknown)` is false (`seal/record.go:230-239`) → `AdmitSection` returns `ErrRecordNotPermitted` and, per its own contract, **admits no records at all** ("rejection is whole-payload") → `seal/unlock_key.go:102-105` → `gui/unlock_kdf.go:453-455`, the `default:` arm → **"Payload unreadable."** So one unlucky share in a sealed payload makes *every* secret in it unreachable, and the operator is told the payload is unreadable — after a successful authentication and a ~31 s KDF, on bytes that are intact. That is the exact misdiagnosis `ErrCodex32TooLong` was given its own case to avoid (`gui/unlock_kdf.go:433-451`).

**Why no gate in the plan can see it — measured.** I decoded the leading payload byte of every row in `crates/me-cli/testdata/codex32_seam_vectors.json`:

| row | host/device | payload[0] |
| --- | --- | --- |
| `bip93-secret-128` | false/true | `0x31` |
| `bip93-secret-256` | false/true | `0xff` |
| `bip93-share` | false/true | `0xff` |
| `constellation-entr-128` | true/true | `0x00` |
| `constellation-entr-256` | true/true | `0x00` |
| `entr-id-but-off-profile-length-90` | false/true | `0x00` |
| `past-the-engraveable-cap-91` | false/false | `0x00` |
| `bip93-bad-checksum` | false/false | `0x31` |

None is `0x03`. Widening the scan to every `ms1` literal in the fork (`grep -rhoE '\bms1[0-9a-z]{20,200}\b' --include=*.go --include=*.json`, 82 distinct strings): **0 with payload[0] == 0x03**. So the whole Go suite, the seam corpus and the three recorded mutations all stay green while the narrowing is wrong. The plan's evidence for "Minimal narrowing … is unchanged" — *"the seam test still requires a device-only row and this plan keeps all three shapes"* — cannot falsify the claim it is offered for.

**SUGGESTION.** Make the predicate answer the question the spec asks — *is this a preimage single?* — rather than *does some byte equal 3*:

```go
func IsPreimage(s String) bool {
	f, err := ParsePrefix(s.String())
	if err != nil || !f.Unshared || f.Identifier != "hash" {
		return false
	}
	d := s.Seed()
	return len(d) == 33 && d[0] == msPrefixPreimage
}
```

(`ParsePrefix`'s `Unshared`/`Identifier` fields are already used at `gui/codex32_polish.go:207-210`.) Whatever shape is chosen, the *tests* must include the two populations that break: a K-of-N share and a plain BIP-93 secret whose leading payload byte is `0x03`, and both as `device_admits: true` rows in the shared corpus — otherwise the same green suite protects the same blind spot next time.

---

### I-1 — Three fragment anchors are wrong; two of them produce a duplicate `const`, one deletes the md1/mk1 branch

**Plan section:** File Structure table; Task 1 Step 3; Task 2 Steps 1 and 5. Checked against fork `839fa5aa` and engrave `e06e29d`.

| plan cites | actual | applying it literally |
| --- | --- | --- |
| `crates/me-cli/tests/codex32_seam.rs:15-16` (`SEAM_VECTORS_SHA256`) | **25-26**; 15-16 are `//!` doc lines | second `SEAM_VECTORS_SHA256` at line 15 → `error[E0428]` duplicate definition, plus a mangled doc comment |
| `sysw/codex32_seam_test.go:11` (`seamVectorsSHA256`) | **30**; line 11 is blank | second `seamVectorsSHA256` → `seamVectorsSHA256 redeclared in this block` |
| `seal/record.go:214-216` (Step 5, "Replace") | the `codex32.New` branch is **212-214**; 214-216 is `}` + `if codex32.ValidMD(s) \|\| codex32.ValidMK(s) {` + `return ClassMDMK` | deletes md1/mk1 classification and leaves an unbalanced brace |

The anchor *text* in each case is unambiguous, so a careful implementer recovers — but a plan whose `Modify` blocks are applied by line number is exactly the "hand-wired by the controller, then handed to someone else" case these citations exist for, and the build gate cannot catch it because the controller wired the blocks to the right places.

Correct citations verified in the same pass, for contrast: `codex32/mspayload.go:8-12` (the const block) ✓, `sysw/classify.go:116-125` and the `:123-124` two-line anchor ✓, `seal/record_test.go` imports at lines 4-5 ✓, `seal/record_test.go:441` (the 23-character `wipe` fixture, which is indeed not `New`-valid at 23 characters) ✓, `gui/unlock_session_test.go:714` (`runUnlockEngraveMnemonic`) ✓, `codex32/codex32.go:386` (`Seed()`) ✓, `seal/record.go:244` (`AdmitSection`) ✓, `record.rs:71` / `:117` ✓, `Cargo.toml:53` ✓, crate name `mnemonic-engrave` / lib `mnemonic_engrave` with `pub mod seal` and `pub mod sysw` ✓, `sessionHarness.mustReach` (`gui/unlock_session_test.go:168`) and `newPlatform` (`gui/gui_test.go:616`) ✓.

**SUGGESTION.** Re-cite the three anchors, and prefer the anchor *text* over a line range in every `Modify` block (`scripts/plan-staleness-check.sh` will keep finding these mechanically if the plan records a baseline; it already does).

---

### I-2 — The `IsPreimage` unit test cannot catch the mutation it names

**Plan section:** Task 2 Step 3.

The test carries:

```go
	// MUTATION: `d[0] == msPrefixPreimage` -> `d[0] != msPrefixEntr` would
	// call every mnem string a preimage; the entr and mnem seams below catch it.
```

Under that mutation (`return len(d) > 0 && d[0] != msPrefixEntr`), walking the test as written:

- the plate: `d[0] == 0x03`, `0x03 != 0x00` → `true` → `!IsPreimage(s)` does not fire ✓ passes
- `s.Split()` id `hash` — unaffected ✓ passes
- the loop's one fixture, `constellation-entr-128`: `d[0] == 0x00` → `false` → `IsPreimage(e)` does not fire ✓ passes
- `DecodeMS1` still `errMSBadPrefix` — unaffected ✓ passes

**The test passes under its own named mutation.** The comment says "the entr **and mnem** seams below", but the loop is a one-element slice holding only the entr string (the mnem fixture the sentence promises is not there), and the mnem population is precisely the one the mutation misclassifies. Nor was this mutation among the three the build gate ran (per the brief: the `isStrictMs1` clause, the `Classify` clause, and the `unlockEngraveCodex32` guard).

**SUGGESTION.** Add a `mnem` string (prefix `0x02`) to the loop and re-run the named mutation, or delete the claim. A mutation comment that the shipped test cannot honour is a false assurance about the test, which is the class this project treats as blocking.

---

### I-3 — `me`'s refusal message calls a preimage plate a BIP-93 secret, contradicts itself on the length, and tells the operator to re-encode it as seed entropy

**Plan section:** Task 1 (the host half); Global Constraints, "Rust-primary".

`me sysw pack` on a preimage plate: `sysw::classify` → `Class::Unknown` (correct), then the refusal's explanation comes from `unknown_reason` (`crates/me-cli/src/sysw/mod.rs:183-185`), whose last arm is `bip93_outside_the_profile` — true here, because the HRP is `ms`, `Codex32String::from_string` succeeds and `ms_codec::decode` errs. It renders (`crates/me-cli/src/main.rs:2799-2811`):

> `record 0 (records count from 0) is a VALID BIP-93 codex32 string — the checksum is good — but not a constellation` `ms1` `record, so this container cannot place it.` … `the whole string must be [50, 56, 62, 69, 75] characters (entropy) or [51, 58, 64, 70, 77] (mnemonic), and the 4-character id must be` `entr`. `This one is 75 characters.` … `re-encode the entropy as` `ms1` `rather than editing the string.`

Three defects on a preimage plate: it prints a set **containing 75** and then gives 75 as the reason the string is outside it (the real gate was the prefix byte, `ReservedPrefixViolation`); it calls a constellation `ms1` string "not a constellation `ms1` record"; and it calls the payload "the entropy" and instructs the operator to **re-encode it as `ms1`** — i.e. to turn their hashlock preimage into a seed string, which is the confusion H0 exists to remove, delivered as advice. The message was written for plain BIP-93 (48/74 characters) and its own comment says it exists because *"Unrecognised's text is FALSE here"*; on the new kind it is false in the same way.

Task 1's pin test asserts only `Err(_)` and `!= Class::Codex32Secret`, so it cannot see this, and Task 4's H1b follow-up covers the refusing *arm*, not the diagnosis.

**SUGGESTION.** One arm before the profile arm — if the string is `ms`-HRP, codex32-valid and `decode` fails with `ReservedPrefixViolation { got: 0x03 }`, name the kind: *"record N is a hashlock PREIMAGE plate (kind 0x03, id `hash`), not a seed record; this container cannot place one yet."* If that is too much for H0, file it with owning phase H1b beside the existing follow-up, because the bump changes this arm's behaviour again.

---

### I-4 — A cheaper device acceptance exists today, and the stated reason for deferring does not cover the paths that matter

**Plan section:** Task 3 Step 3 ("Device acceptance is deferred to H2, when `me` can pack a payload carrying a `0x03` record; until then the fork's `seal` admission test is the acceptance").

The premise is true for the *sealed* container — `me seal` and `me sysw pack` both route through `validate_record`, which at ms-codec `0.7` refuses the string, so neither can build such a payload (verified above). But it does not cover what H0 is actually flashing for:

- **The two doors in C-1 need no payload at all.** `cmd/emu` is a `GOOS=js` build of the shipped `gui` package ("Nothing above cmd/emu is emulated or reimplemented"), so typing the 75-character plate into `M*1 STRING`, or presenting it over the emulator's NFC (`cmd/emu/nfc.go`, `cmd/emu/nfc_presented_test.go`), walks the real flow today. A headless version is ~10 lines against the harness that already exists at `gui/nfc_scan_test.go:105-106`.
- **The sysw container is built in-tree, in Go.** `cmd/buildpayloadcomposer` and `cmd/buildpayloadcards` generate the emulator's systemwide blobs without `me`, so the Load Payload door can be accepted end-to-end today (record present → counted "inert" → offered by no program) rather than only at `sysw.Classify`'s unit level.

Only the *sealed* half genuinely waits for H2. This matters beyond tidiness: the acceptance the plan declines is the one that would have caught C-1, on a change whose entire justification is that a flashed device cuts the wrong plate.

**SUGGESTION.** Before the flash, require: (a) one `scanner.Scan` test and one `validateMStar`/`inputCodex32Flow` test asserting the plate is refused and no engrave screen is reached (mirroring Task 2 Step 7's `mustReach`); (b) one emulator walk of both doors with the plate string, recorded in the continuity entry. Keep the H2 deferral for the sealed-payload acceptance only, and say that is what is deferred.

---

### I-5 — Refusing the section is the right H0 behaviour, but its operator-visible half is "Payload unreadable." and the plan does not record it

**Plan section:** Global Constraints, "No new class"; Task 2 Step 6.

**The choice is right, and I would not change it.** `permitted` is deliberately an allow-list whose comment says a deny-list "silently admits whatever branch Scan grows next, and one of the branches it already has burns OTP fuses" (`seal/record.go:225-229`), and `AdmitSection`'s contract is whole-payload rejection with a wiped partial result. Skipping the record would (i) invent a per-record-skip semantics the container has never had, (ii) hand Phase B a payload silently missing a record the operator expects to cut, and (iii) be a *wider* change than "no new class". Loss of access to the other secrets is not reachable in practice today: `me` cannot pack a preimage record at 0.7, and after the H1b bump the refusing arm keeps it unpackable, so only a hand-built or third-party sealed payload gets there before H2 teaches the device the kind.

**What the plan does not record** is what the operator sees. `ErrRecordNotPermitted` has no case in `gui/unlock_kdf.go:417-455` and falls to `default: showError(..., "Payload unreadable.")`. The two neighbouring cases — `ErrTooManyRecords` and `ErrCodex32TooLong` — were each given a dedicated arm with an explicit rationale: *"Falling through to 'Payload unreadable.' would tell someone with a perfectly good seed card that it had been tampered with — after a successful authentication and a ~31 s key derivation."* The plan's "no new class" paragraph reasons entirely at the `seal` layer and its test asserts only that the error string contains "unknown"; nothing states that the operator's screen says the payload is unreadable, and no follow-up owns fixing that when H2 makes it reachable. (C-2 makes it reachable *today*, by accident, on an ordinary share.)

**SUGGESTION.** Not a behaviour change for H0. One sentence in "No new class" naming the rendered outcome, and a follow-up with owning phase **H2**: give `ErrRecordNotPermitted` its own arm naming the record index and class, on the same argument the two neighbouring cases already carry.

---

### M-1 — `record.rs:177` cited for the `ms_codec::decode` call

**Plan section:** Global Constraints, "Rust-primary" (*"`me` refuses at `crates/me-cli/src/seal/record.rs:177` via `ms_codec::decode`"*). The call is at **176**; 177 is `.map(|_| RecordKind::Ms)`. Task 1 Step 6 cites 176 correctly, so the two disagree. (Spec §14 carries the same `:177`.) SUGGESTION: cite `:176-178` as the arm.

### M-2 — The two containers now diverge on an unknown record, and the plan states only one

**Plan section:** Global Constraints, "No new class". After the guard, the *same* preimage record is per-record inert in the `sysw` container (it stays in the session, is offered to nobody, and is counted as "inert" on the composer door — `gui/composer_door.go:30-51`) but whole-payload fatal in the `seal` container (I-5). Both are right for their own contract, but the plan describes the `seal` behaviour as though it were the only one, and a future reader tracing "what happens to a `0x03` record on the device" gets one of two answers depending on which door they came in. SUGGESTION: one clause naming both.

---

## Items verified with no finding

- **Item 3 — Rust-primary / the pin test's tripwire value: the plan's claim is TRUE.** At ms-codec `0.7`, `decode` (`src/decode.rs:44-107`) runs the string-length gate first (75 ∈ `VALID_STR_LENGTHS = [50, 56, 62, 69, 75]`, `consts.rs:33` — passes), then `Codex32String::from_string` (passes), then `envelope::discriminate`, whose HRP/threshold/share-index gates all pass for `ms1` + `0` + `s` and whose `Tag::try_new("hash")` succeeds on the bech32 alphabet, and which then calls `dispatch_payload` (`envelope.rs:192-219`) → `other => Err(Error::ReservedPrefixViolation { got: 0x03 })`, rendered at `error.rs:202-204` as `reserved-prefix byte was 0x03, expected 0x00`. **The prefix dispatch precedes the tag accept set** (`decode.rs:84-104`), so the refusal is for the KIND and not for the `hash` tag, not for the length, and not for `MsTooLong` (75 ≤ 90, `record.rs:170-173`). `crate::classify::classify` is HRP-only (`classify.rs:42-55`), so `Format::Ms` is reached and `validate_record` really does end at `ms_codec::decode`. After the 0.8 bump the same string decodes to `Ok`, `validate_record` maps it to `RecordKind::Ms`, and the pin test's `Ok(kind) => panic!` arm fires — the tripwire is real, and the only way it stays green is a genuine refusing arm.
- **The corpus row and its sha256 reproduce exactly.** I rebuilt the file in memory from the plan's Step 1 text (tail `    }\n  ]\n}\n` → `    },\n` + row + `  ]\n}\n`) and got `4ac542ea8e0e36d92127b744bce0a83072f787870756bf7b86b9c947bb1370a5` — the plan's value, byte for byte. The plate is 75 characters with id `hash`; the substituted entr fixture is 50 characters, matching `constellation-entr-128`.
- **"Nine rows: 2 both / 4 device-only / 3 neither" is correct** — measured 2/4/2 over the eight existing rows, plus one no/no row. Neither seam test asserts a row count or an exact distribution (both only require each of the three shapes to be non-empty: `codex32_seam.rs:70-76`, `codex32_seam_test.go:78-83`), so the new row cannot break either on arithmetic.
- **Task 2 Step 1's expected failure line is consistent**: `sysw.Class` iota puts `ClassCodex32Secret` at 2 (`sysw/record.go:26-29`), matching `Classify = 2`.
- The engrave-adjacent `DecodeMS1` consumers (`ms1_decode.go`, both verify flows, `bundle/verify.go`) are already inert on `0x03` and need nothing from H0.

---

## Counts

**2 Critical / 5 Important / 2 Minor / 0 Nit — NOT GREEN.**

Both Criticals are about the same gap in the plan's model of the device: it guards the *classifiers* and the payload-sourced engrave, and does not ask what the operator can hand the machine directly (C-1) or what else the predicate catches on the way (C-2). Everything else is fixable in a fold.
