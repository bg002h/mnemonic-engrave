# F-113 whole-diff execution review — SPEC §10.2.1a

Date: 2026-08-10
Reviewer: independent (author ≠ reviewer). Post-implementation adversarial execution review.
Scope: the code as built, not the design. R0's 0C/0I on the design was taken as settled and
not re-derived.

**Reviewed at:**

| repo | worktree | branch | head |
| --- | --- | --- | --- |
| `mnemonic-engrave` | `/scratch/code/shibboleth/me-wt-f113` | `f113-ms1-engraveable` | `ccc884a6e429273458d77a90e792caa00908146a` |
| `seedhammer` (fork) | `/scratch/code/shibboleth/sh-wt-f113` | `f113-ms1-engraveable` | `8d2d53067c8522ddb87954e37130f97c7e4942cc` |

Both worktrees were `git status --porcelain` clean at those SHAs before and after this review.
Every mutation below was applied to a working copy, run, and restored from a byte-for-byte
backup; final state re-verified clean at the same two SHAs.

---

## VERDICT

**GREEN — 0 Critical, 0 Important.**

1 Minor, 2 Nits. None blocks the merge.

---

## 1. The wipe (brief item 1) — PASS

**In the per-record pass: yes.** `seal/record.go:285-291`. The check sits inside
`AdmitSection`'s `for i, r := range records` loop, after the §10.2.1 allow-list and before
`out = append(...)`. It is the third failure path in that loop and the third `wipe(out)` call
site.

**Every early return wipes: yes.** The new code has exactly one early return, and it calls
`wipe(out)` immediately before `return nil, ...`. `out` is built with
`make([]AdmittedRecord, 0, len(records))` — capacity is preallocated, so `append` never
reallocates and no stale backing array holds copies the wipe misses. The current record `r` is
not yet copied at the point of the check; it is a slice into the caller's `plaintext`, which
`UnlockWithKey` covers with `defer clear(plaintext)`.

**The placement test genuinely discriminates — VERIFIED BY MUTATION, not by reading.**
Applied G3 myself (deleted the in-loop block; inserted an equivalent
`for i, rec := range out { ... }` scan immediately before the `section == SectionPublic`
block; confirmed the diff applied with `git diff` before running):

```
--- FAIL: TestTooLongSecretIsCaughtInThePerRecordPass (0.00s)
    engraveable_test.go:171: got seal: record classification not permitted in this section:
    record 1 classifies as debug command, which the encrypted section does not permit,
    want ErrCodex32TooLong -- an over-length secret at index 0 must stop the section THERE.
```

That is the discriminator working exactly as its comment claims: a post-loop check runs the
whole copy loop first, meets `"command: lock-boot"` at index 1, and reports the wrong sentinel.

**The G11 survivor is real, and the "unobservable" claim is true.** Applied G11 (removed the
`wipe(out)` call from the new path only) and ran the three affected packages:

```
ok  seedhammer.com/seal    17.270s
ok  seedhammer.com/backup   3.709s
ok  seedhammer.com/gui     40.711s
```

Survived, as reported. I checked the one facility that might have caught it:
`Payload.RecordsResident()` (`seal/session.go:51`) reads `p.Secret` only, and `out` is never
assigned to `p.Secret` on this path — so it cannot reach those bytes either. The claim
"unobservable through the public API without `unsafe`" holds. This is the same property the two
pre-existing allow-list call sites already had, and the `wipe` doc comment now records three
call sites and the 2026-08-10 re-measurement. **The call itself is present** — only its removal
is undetectable. Not a finding.

## 2. The error path (brief item 2) — PASS

**The operator sees a distinguishing message on the real path.** Ran the flow-level test
unmutated: PASS. Then applied G9 (deleted the whole `case errors.Is(err, seal.ErrCodex32TooLong)`
arm, 1077 bytes, confirmed by `git diff --stat` = 19 deletions):

```
unlock_engraveable_test.go:48: never reached "cannot engrave";
    last frame "Payloadunreadable.SealedPayload"
--- FAIL: TestUnlockNamesAnUnengraveableSecretInsteadOfCallingItUnreadable
```

So the sentinel really was invisible without `gui/unlock_kdf.go`, and the test really sees it.
That failure output also settles a question the assertions leave open: the extracted frame is
the *drawn* text (`Drawer.ExtractText` appends a glyph's rune at `gui/op/op.go:427`, which is
reached only after `clip := state.clip.Intersect(dst.Bounds()); if clip.Empty() { break }`), so
off-screen glyphs are not collected. The test asserting `"Nothing was opened"` — the message's
last four words — therefore proves the whole ~110-character message renders on one screen and
is not clipped or paged. That was the live risk with a message roughly twice the length of the
`ErrTooManyRecords` one beside it.

**Nothing is mis-routed.** The new arm is inserted as case 5 of 6, ahead of `default` only, so
it can only capture errors that previously fell through to `"Payload unreadable."`. It matches
on `errors.Is(err, seal.ErrCodex32TooLong)`, and `ErrCodex32TooLong` has exactly one
construction site (`seal/record.go:287`). A genuine integrity failure cannot reach it:
`Open` verifies the AEAD tag *before* `AdmitSection` runs (`seal/unlock_key.go:83-102`), so a
tampered or corrupt blob returns `ErrAuthentication` from the case above and still reads as
"Wrong passphrase, or this payload has been altered." I checked every other renderer of
"Payload unreadable." (`gui/unlock_flow.go:35,43,77`) — all three are pre-unlock paths behind
`Inspect`, whose only `AdmitSection` call is `SectionPublic` (`seal/open.go:149`), where the
allow-list refuses `ClassCodex32Secret` before the length check is reached. `ErrCodex32TooLong`
is structurally unreachable there, and
`TestPublicSectionStillReportsAnOverLongSecretAsNotPermitted` pins it.

Returning `false` rather than looping is correct: §10.2.1a refuses the payload whole, so
retyping the passphrase cannot help, and `unlockSealedFlow`'s contract keeps a false return
from falling through to the plate list.

## 3. Scope: `ms1` only (brief item 3) — PASS

Verified by mutation rather than by reading. Applied G5 (dropped the
`c == ClassCodex32Secret &&` guard, leaving a bare length check) and ran `./seal/`:

```
--- FAIL: TestUnlockWithKeyFailsClosedOnAWrongKey
    record 0 is a codex32 secret of 111 characters; this machine can engrave at most 90
--- FAIL: TestUnlockWithKeyTwiceWipesTheFirstResult
    record 0 is a codex32 secret of 143 characters ...
--- FAIL: TestUnlockWithKeyZeroesTheDecryptedPlaintext
    record 3 is a codex32 secret of 111 characters ...
```

— i.e. the existing fixtures already carry 111- and 143-character md/mk and mnemonic records in
*both* sections, and the class guard is the only thing keeping them admitted. `Classify`
(`seal/record.go`) reaches `codex32.New` only after the mnemonic and descriptor branches, and
vector D's public md1/mk1 records over 90 characters are admitted as `ClassMDMK` — asserted by
`TestEngraveableLimitDoesNotCoverMDMKRecords`, which fails its own premise if no such record
exists. No md/mk or BIP-39 record can be refused by this rule at any length.

I also checked the length measure itself. Go uses `len(r)` (bytes) and Rust `s.chars().count()`;
both are reached only after a successful codex32 parse, whose charset is ASCII bech32, and pass 1
has already refused uppercase — so bytes == characters on every reachable input. And the string
the check measures is the string the engraver receives: `codex32.New` stores the input verbatim
(`String{s}`, `codex32/codex32.go:110`) and `String()` returns it, so
`gui/unlock_session.go:196`'s `Seed: s.String()` cannot be a different length from the admitted
record.

## 4. The boundary test (brief item 4) — PASS

Both halves of the implementer's claim reproduce exactly.

**After the bind** (`seedQRLevel = qr.M` → `qr.Q`):

```
engraveable_test.go:41: the real encoder caps the seed plate at 67 characters,
    but seal.MaxEngraveableCodex32Len is 90 -- §10.2.1a's constant has drifted
engraveable_test.go:62: premise broken: qr.Q (67) should be STRICTER than qr.M (67)
--- FAIL: TestEngraveableLimitIsDerivedFromTheRealQREncoder
--- FAIL: TestEngraveSeedStringCutsAt90AndRefusesAt91
```

**Before the bind** (counterfactual: `qr.Encode(seed, qr.Q)` inline while `seedQRLevel` stays
`qr.M`, i.e. what a duplicated literal in the test would have produced):

```
engraveable_test.go:60: engraveable limit: qr.M -> 90 characters, qr.Q -> 67
--- PASS: TestEngraveableLimitIsDerivedFromTheRealQREncoder
--- FAIL: TestEngraveSeedStringCutsAt90AndRefusesAt91
```

So the derivation test does re-derive 90 from the real encoder rather than restating it, and the
bind is what makes it sensitive to the level. See Nit N2 for the one over-claim this measurement
exposes.

The QR gate is also parameter-independent — `EngraveSeedString`'s only error sources are
`qr.Encode`, the `qrc.Size > seedQRMaxSize` cap and `ConstantQR`; `engraveSeedString` returns an
`Engraving` with no error — so the test's `params` fixture cannot produce a false "90 engraves".

## 5. Undeclared scope (brief item 5) — PASS

Eight files total, and no undeclared production file:

- Rust: `crates/me-cli/src/seal/record.rs`, `crates/me-cli/tests/seal_cli.rs`.
- Go: `seal/record.go`, `gui/unlock_kdf.go`, `backup/backup.go` + three new `_test.go` files.

`gui/unlock_kdf.go` is **necessary**, and G9 above is the proof: without it §10.2.1a's
distinguishability requirement is unmet on the only path an operator ever takes.
`backup/backup.go` is behaviour-neutral (verified by diff: `qr.M` and `33` moved into two named
constants, no value changed) and defensible; see Nit N2 for the caveat.

## 6. Mutation honesty (brief item 6) — PASS

Seven mutations applied by hand, each confirmed applied via `git diff` before the run:

| # | mutation | reported | measured |
| --- | --- | --- | --- |
| G3 | check moved to the post-loop block | RED | **RED**, by the named test, with the named message |
| G5 | `ms1`-only class guard dropped | RED ×2 | **RED**, 4 tests in `./seal/` |
| G7 | `qr.Q` inline, pre-bind | RED, derivation test survived | **matches exactly** |
| G7b | `seedQRLevel → qr.Q`, post-bind | RED ×2 in `./backup/` | **RED ×2** |
| G9 | GUI case removed | gui RED, seal ok | **RED**, seal/backup unaffected |
| G11 | `wipe(out)` removed | **SURVIVED** | **SURVIVED** — seal, backup and gui all ok |
| M3 (Rust) | codex32-validity gate removed | RED | **RED**, `does_not_fire_on_a_long_non_codex32_ms1_string` |

No no-op mutants: every one changed observable behaviour in at least one direction, and G11's
survival was checked against the one API (`RecordsResident`) that could plausibly have caught it.
The mutation table in `2026-08-10-f113-implementation.md` is accurate on every point I checked.

**Vectors cross-checked mechanically**, not by eye: the 90/91/93/125/127-character constants are
byte-identical across `crates/me-cli/src/seal/record.rs`, `crates/me-cli/tests/seal_cli.rs`,
`seal/engraveable_test.go`, `backup/engraveable_test.go` and `gui/unlock_engraveable_test.go`
(set comparison over all five files; the only vector not shared is the pre-existing 75-character
`MS1`). Their codex32 validity and `Classify` result are asserted against the real engine inside
`TestEngraveableVectorsAreWhatTheyClaim`, which passes.

## 7. Suite re-run

| check | result |
| --- | --- |
| `cargo test` | 179 passed, 0 failed (123 + 1 + 30 + 1 + 3 + 1 + 6 + 14 + 0) |
| `cargo clippy --all-targets` | 0 warning/error lines |
| `cargo fmt --check` | 7 diffs, all pre-existing, **none in either touched file** |
| `go test ./seal/ ./backup/ ./gui/` | all ok |
| `gofmt -l seal/ gui/ backup/` | 5 files, all pre-existing, none touched here |
| `go vet ./seal/ ./gui/ ./backup/` | 3 `testing.ArtifactDir` diagnostics, identical to `b2b` |

End-to-end CLI, run by hand against the built binary:

```
$ me seal <91-char ms1> --seal-secret --out …
me: this codex32 secret is 91 characters; the machine can engrave at most 90 (§10.2.1a).
    The record is INTACT — it is too long to cut, not unreadable. …
exit=4, no output file
```

---

## FINDINGS

### Minor M1 — Rust reports the plate-geometry problem where Go deliberately reports the serious one

`crates/me-cli/src/seal/mod.rs:99-118` (`check_public`), against
`seal/record.go:271-276`'s committed rationale.

The Go implementation places §10.2.1a **after** the §10.2.1 allow-list on purpose, and says why
in the code: *"An over-length codex32 secret in the PUBLIC section is a secret shipped in the
clear — a far more serious finding than a plate that does not fit — and the operator must be
told the serious one."* `TestPublicSectionStillReportsAnOverLongSecretAsNotPermitted` pins it.

Rust's `check_public` calls `validate_record(r).map_err(SealError::Record)?` *before* it looks at
`.is_secret()`, so §10.2.1a wins. Measured on the real binary:

```
$ me seal --plaintext <91-char ms1> --out …
me: this codex32 secret is 91 characters; the machine can engrave at most 90 (§10.2.1a). …

$ me seal --plaintext <75-char ms1> --out …
me: record 0 is secret material and cannot ride in the public section — it would be
    engraved and readable in the clear (§6.3)
```

**Scenario.** An operator building a payload puts a 91-character codex32 secret in
`--plaintext` by mistake. They are told only that it is too long to engrave. The natural
remedy — shorten it — is the wrong lesson: the actual problem is that they were about to publish
a seed in the clear, and they only learn that on the second attempt. Nothing is emitted either
way (`exit=4`, no file), so there is no leak and no wrong result; this is a diagnosis-quality
defect and an asymmetry between two implementations of one spec section.

**Not a regression** — before this change the same input produced
`invalid record: string length 91 outside v0.1 set […]`, which was equally silent about the
public-section violation.

**Smallest fix.** In `check_public`, reorder so the secret-in-public verdict wins:

```rust
match record::validate_record(r) {
    Ok(k) if k.is_secret() => return Err(SealError::SecretInPublic(i)),
    Err(record::RecordError::MsTooLong(_)) => return Err(SealError::SecretInPublic(i)),
    Err(e) => return Err(SealError::Record(e)),
    Ok(_) => {}
}
```

with a test mirroring `TestPublicSectionStillReportsAnOverLongSecretAsNotPermitted`. Owning
phase: whenever F-120 is settled, since both concern how `me` classifies a BIP-93 codex32 that
is not a constellation `ms1`.

### Nit N1 — an unzeroized `String` copy of near-seed material on the Rust reject path

`crates/me-cli/src/seal/record.rs:147-151`. `Codex32String::from_string(s.to_string())` allocates
a heap copy; on `from_string`'s **error** path that copy is dropped without being scrubbed. The
comment discloses this honestly and argues "a string that fails the codex32 checksum is not a
seed" — which is true of garbage but not of a mistyped real backup.

No concrete failure scenario, hence Nit: `Payload.secret` is already a plain `Vec<String>`
(`seal/mod.rs:23`), so this crate already holds those records unzeroized, and the new copy adds
no new *class* of residue. Recording it only so nobody later reads the comment as a guarantee.

### Nit N2 — "Nothing else in the tree would have noticed" is measurably false

`backup/engraveable_test.go`, doc comment on
`TestEngraveableLimitIsDerivedFromTheRealQREncoder`.

The comment justifies the `seedQRLevel`/`seedQRMaxSize` extraction with: *"at qr.Q the limit
drops to 67 … Nothing else in the tree would have noticed."* Measured (§4 above): with
`EngraveSeedString` moved to `qr.Q` and the test still reading a duplicated `qr.M`,
`TestEngraveSeedStringCutsAt90AndRefusesAt91` — the sibling test in the same file — **fails**
with `a 90-character share must engrave, got seed too long to engrave QR`.

So the bind buys a *second, earlier-diagnosing* detector, not the only one. The implementer's own
report is accurate about this ("FAIL ×1 only", "the derivation test survived it"); only the
in-tree comment over-claims, and this project's standing rule is that comments outlive the
conditions that made them true. Smallest fix: replace the last sentence with *"Only
`TestEngraveSeedStringCutsAt90AndRefusesAt91` would have noticed, and it reports a refused
plate rather than a drifted constant."*

---

## What I did NOT re-review

The design, the 90 boundary as a design choice, the operator ruling, and whether the rule should
exist — all settled by R0 over three rounds. F-120 (the `ms-codec` vs `codex32.New` accept-set
divergence the implementer flagged as §5.1) is **already filed** in `design/FOLLOWUPS.md` with an
owning phase of *post-merge polish and hardening*; I confirmed the entry exists and did not
re-derive it.
