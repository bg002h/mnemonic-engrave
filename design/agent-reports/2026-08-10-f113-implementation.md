# F-113 implementation — `ms1` records must be ENGRAVEABLE to be admitted (SPEC §10.2.1a)

Date: 2026-08-10
Order: **Rust first, then Go** (Rust-primary rule — this is normative admission behaviour).
Both branches are **unmerged**. The controller merges.

---

## 1. Branches and SHAs

| repo | worktree | branch | base | head SHA |
| --- | --- | --- | --- | --- |
| `mnemonic-engrave` | `/scratch/code/shibboleth/me-wt-f113` | `f113-ms1-engraveable` | `master` @ `e061fab` | **`ccc884a6e429273458d77a90e792caa00908146a`** |
| `seedhammer` (fork) | `/scratch/code/shibboleth/sh-wt-f113` | `f113-ms1-engraveable` | `b2b` @ `c0c958d` | **`8d2d53067c8522ddb87954e37130f97c7e4942cc`** |

One commit each. Nothing merged into `master` or `b2b`.

> Note on the base: the session-start snapshot showed `mnemonic-engrave` master at `0f19d6e`, but the
> real `master` at worktree-creation time was `e061fab` (`git rev-parse HEAD master` — both `e061fab`).
> The worktree branched off the real tip.
>
> `master` has since moved to `cdf88c8` (7 commits). Measured: `git diff --stat e061fab..master`
> touches only `design/FOLLOWUPS.md`, `design/SPEC_encrypted_payload_delivery.md` and one new
> agent-report — **no code**, so the branch rebases cleanly. Two things in that range matter here:
> `7d94571` records **R0 round 2 on §10.2.1a as GREEN, 0C/0I, loop closed**, so this was implemented
> against a GREEN spec rather than a GREEN-pending one; and the only §10.2.1a edit in the range is a
> **text reflow** of the "Why 90" paragraph — no normative change, verified by reading the diff.

### Files changed

**Rust** (`ccc884a`, 233 insertions / 6 deletions):

```
crates/me-cli/src/seal/record.rs | 209 +++++++++++++++++++++++++++++++++++++--
crates/me-cli/tests/seal_cli.rs  |  30 ++++++
```

**Go** (`8d2d530`, 557 insertions / 6 deletions):

```
backup/backup.go               |  24 +++-
backup/engraveable_test.go     | 123 +++++++++++++++++++++   (new)
gui/unlock_engraveable_test.go |  74 +++++++++++++           (new)
gui/unlock_kdf.go              |  19 ++++
seal/engraveable_test.go       | 241 +++++++++++++++++++++++++++++++++++++++++ (new)
seal/record.go                 |  82 +++++++++++++-
```

---

## 2. What changed, and why it is where it is

### 2.1 Rust — `crates/me-cli/src/seal/record.rs`

Home is **`record::validate_record`**, where record classification already happens — not `me seal`'s
argument handling. `me seal` reaches it through `seal::record_or_mnemonic`, `me hash` through
`main.rs:479`, and both therefore refuse what the device will refuse.

- `pub const MAX_ENGRAVEABLE_MS1_LEN: usize = 90`, with a doc comment that states plainly it is a
  **literal** here (no QR encoder exists in this crate) and names the Go test as the authority.
- `RecordError::MsTooLong(usize)` — a **separate variant**, not an `Invalid`. That is the §6.4
  requirement: every other record failure renders to the operator as "payload unreadable", and this
  record is not unreadable.
- The check runs in the `Format::Ms` arm **before `ms_codec::decode`**. It has to: `ms-codec`'s own
  length gate fires first otherwise and reports `UnexpectedStringLength`, which renders through
  `Invalid` — precisely the misdiagnosis the rule exists to remove.
- The check is **gated on the BIP-93 parse** (`ms_codec::codex32::Codex32String::from_string`), not
  on the HRP alone, so `ms1` + garbage is still reported as invalid rather than as too long. That
  mirrors the device, where `Classify` reaches the length check only through `codex32.New`. The parse
  is attempted only once the length has already failed, so the ordinary path allocates nothing extra;
  `Codex32String` derives `zeroize::ZeroizeOnDrop`, so the copy it takes is scrubbed.

### 2.2 Go — `seal/record.go`

`MaxEngraveableCodex32Len = 90` and `ErrCodex32TooLong`, checked in **`AdmitSection`'s per-record
pass**, with `wipe(out)` before returning:

- **Not the post-loop section block.** There it would leak every `ms1` already copied into `out`,
  unreachable to both `Payload.Wipe` and `RecordsResident()`.
- **After the §10.2.1 allow-list, not before.** An over-length codex32 secret in the *public* section
  is a secret shipped in the clear — a far more serious finding than a plate that does not fit — and
  the operator must be told the serious one. Checking first turns `ErrRecordNotPermitted` into
  `ErrCodex32TooLong` for exactly that case. Pinned by
  `TestPublicSectionStillReportsAnOverLongSecretAsNotPermitted`.
- **`ms1` only.** `ClassMDMK` and `ClassMnemonic` are untouched.

### 2.3 Go — `gui/unlock_kdf.go` (NOT in the brief; added because the rule requires it)

`unlockSealedFlow`'s switch has a `default` arm that renders every unhandled error as
`"Payload unreadable."`. **A new sentinel with no case of its own is invisible** — the operator would
still be told their intact backup was corrupt. So a case was added:

> "This payload holds a codex32 secret longer than 90 characters, which this machine cannot engrave.
> Nothing was opened."

The number comes from `seal.MaxEngraveableCodex32Len` via `fmt.Sprintf`, never from a literal in the
string: F-117/F-118 may raise the plate's QR cap, and a screen that still says 90 while the machine
refuses at another length is worse than no number.

`TestUnlockNamesAnUnengraveableSecretInsteadOfCallingItUnreadable` drives the whole flow — hash
screen, passphrase entry, real KDF, refusal — and asserts the drawn frame names the limit, does not
contain "unreadable", and that the flow leaves rather than looping for another passphrase. Mutation
G9 confirms `seal`'s own tests stay green through the removal of this case, so only the flow-level
test sees it.

### 2.4 Go — `backup/backup.go` (behaviour-neutral, enables the pin)

```go
const (
	seedQRLevel   = qr.M
	seedQRMaxSize = 33
)
```

`EngraveSeedString` now reads these instead of writing `qr.M` and `33` inline. **Behaviour is
identical** — these are the values that were already there. The **33 cap is not changed and QR
version support is not touched** (F-117/F-118, deliberately deferred).

They exist so the derivation test can *read* them rather than restate them. Measured: with `qr.M`
duplicated in the test (mutation **G7**), switching `EngraveSeedString` to `qr.Q` left the derivation
test **green at 90** while the machine would refuse at 67. After binding (**G7b**), the same mutation
turns it red.

---

## 3. Vector table — every length MEASURED, not asserted

Generated in the fork, not hand-written:

```sh
head -c N /dev/zero | go run ./cmd/biptool seed -seedlen N -id entr
```

Measured `N → character count` (`| tr -d '\n' | wc -c`):

| entropy bytes | chars | in vectors? | expected | Go result | Rust result |
| --- | --- | --- | --- | --- | --- |
| 16 | 48 | no | — | — | — |
| 32 | 74 | no | — | — | — |
| 40 | 86 | no | — | — | — |
| 41 | 88 | no | — | — | — |
| **42** | **90** | **yes** | **ADMITTED** | admitted, `Class = ClassCodex32Secret`, record unaltered | **rule does not fire** (see §5 gap) |
| **43** | **91** | **yes** | **REFUSED** | `ErrCodex32TooLong` | `RecordError::MsTooLong(91)` |
| **44** | **93** | **yes** | **REFUSED** | `ErrCodex32TooLong` | `RecordError::MsTooLong(93)` |
| 45 | 94 | no | dead zone | — | — |
| 62 | 124 | **no — deliberately** | dead zone; `codex32.New` rejects it | asserted rejected by the engine | — |
| **63** | **125** | **yes** | **REFUSED** | `ErrCodex32TooLong` | `RecordError::MsTooLong(125)` |
| **64** | **127** | **yes** | **REFUSED** | `ErrCodex32TooLong` | `RecordError::MsTooLong(127)` |

- **92 is not constructible.** A short code is `9 + ceil(8N/5) + 13` characters, which steps
  90 → 91 → 93. No 92 vector was invented.
- **124 is not used as a vector.** It is in the dead zone between codex32's bands (short 48–93, long
  125–127). `TestEngraveableVectorsAreWhatTheyClaim` asserts that against the real engine
  (`codex32.New` on a 124-char probe must error), so the claim is machine-checked rather than left in
  prose — and if 124 ever becomes valid, that assertion fires.

Every vector's length **and** `codex32.New` validity **and** `Classify` result are asserted inside the
tests, so a mistyped vector fails loudly instead of degenerating into a test of the allow-list.

The vectors themselves (all-zero entropy, `-id entr`, index `s`):

```
 90  ms10entrsqqqq…qqqqutd7mdh2lc8h2
 91  ms10entrsqqqq…qqqqq2uk6ly9a0dmw4
 93  ms10entrsqqqq…qqqqqqqmtf88e60hz9eu
125  ms10entrsqqqq…qqqqt042k235w95p5rd
127  ms10entrsqqqq…qqqqqmk6rc3gq4c88nvp
```

### The QR boundary, swept against the real encoder

`qr.Encode(strings.Repeat("Q", n), qr.M).Size`, n = 40…130 — reproduces §10.2.1a's table exactly:

```
n= 40 size=29
n= 62 size=33
n= 91 size=37      <-- first size > 33
n=123 size=41
```

`TestEngraveableLimitIsDerivedFromTheRealQREncoder` logs, from the sweep:

```
engraveable limit: qr.M -> 90 characters, qr.Q -> 67
```

which is the spec's claim measured rather than repeated.

---

## 4. Mutation testing

Every mutation was applied, the tests run, and the file restored from a byte-for-byte backup.

### 4.1 Rust — 5 applied, **5 killed**

| # | mutation | killed by | result |
| --- | --- | --- | --- |
| M1 | length check removed entirely | `refuses_an_ms1_longer_than_the_machine_can_engrave`, `refuses_an_ms1_too_long_for_the_seed_plate` (CLI) | **RED** |
| M2 | `>` relaxed to `>=` (fires at 90) | `does_not_fire_at_the_ninety_character_boundary` | **RED** |
| M3 | codex32-validity gate removed (HRP+length only) | `does_not_fire_on_a_long_non_codex32_ms1_string` | **RED** |
| M4 | constant drifts 90 → 91 | `the_cap_is_a_literal_whose_authority_is_the_go_test`, `refuses_an_ms1_longer_…`, CLI test | **RED** |
| M5 | message collapsed to `"payload unreadable"` | `the_too_long_message_names_the_length_and_the_cap`, CLI test | **RED** |

### 4.2 Go — 11 applied, **10 killed, 1 survivor (pre-existing and documented)**

| # | mutation | seal | backup | gui | result |
| --- | --- | --- | --- | --- | --- |
| G1 | length check removed from `AdmitSection` | FAIL ×2 | ok | FAIL | **RED** |
| G2 | `>` relaxed to `>=` (fires at 90) | FAIL (`TestAdmitsACodex32SecretAtTheEngraveableLimit`) | ok | ok | **RED** |
| G3 | check **moved to the post-loop section block** | FAIL (`TestTooLongSecretIsCaughtInThePerRecordPass`) | ok | ok | **RED** |
| G4 | check moved **before** the allow-list | FAIL (`TestPublicSectionStillReportsAnOverLongSecretAsNotPermitted`) | ok | ok | **RED** |
| G5 | `ms1`-only guard dropped | FAIL ×2 (MDMK + mnemonics) | ok | FAIL | **RED** |
| G6 | constant drifts 90 → 91 | FAIL | FAIL ×2 | FAIL | **RED** |
| G7 | `qr.M → qr.Q` *(before the seedQRLevel bind)* | ok | FAIL ×1 only | ok | RED — but the **derivation** test survived it; this is what motivated §2.4 |
| G7b | `seedQRLevel qr.M → qr.Q` *(after the bind)* | ok | **FAIL ×2** | ok | **RED** |
| G8 | `seedQRMaxSize 33 → 37` (F-117 lands, limit not updated) | ok | FAIL ×2 | ok | **RED** |
| G9 | GUI case removed → falls through to "Payload unreadable." | ok | ok | FAIL | **RED** |
| G11 | **`wipe(out)` call removed from the new path** | ok | ok | ok | **SURVIVED** |

**G3 deserves a note** — it is the spec's named hazard, and it is caught. A post-loop check returns
the same sentinel and the same "nothing admitted", so the obvious assertions cannot tell the two
apart. What *is* observable is **which record the section stops at**: put an over-length secret at
index 0 and an allow-list failure (`"command: lock-boot"`) at index 1. The per-record pass reports
`ErrCodex32TooLong`; the post-loop version runs the whole loop first and reports
`ErrRecordNotPermitted`. `TestTooLongSecretIsCaughtInThePerRecordPass` asserts exactly that.

**G11 survived, and this is honest and pre-existing rather than new.** `out` is never returned on
that path and every `AdmittedRecord.Record` is its own allocation, so no test can reach those bytes
through the public API without `unsafe`. `seal/record.go`'s `wipe` doc comment already recorded this
for the two allow-list call sites; it now says **three**, records the 2026-08-10 re-measurement
(deleting the third left `go test ./seal/ ./backup/ ./gui/` green), and points at the placement test
above as what *is* checkable. `TestWipeZeroesAPartialResult` still catches a no-op `wipe`; a removed
*call* is not catchable here.

---

## 5. GAPS — stated, not silent

### 5.1 The Rust side cannot demonstrate "90 admitted" — and the reason is a real divergence

`ms-codec` 0.7 is the **constellation `ms1`** codec, not a general BIP-93 codex32 parser. Its accept
set (`src/consts.rs`) is:

```
VALID_STR_LENGTHS      = [50, 56, 62, 69, 75]   (entr)
VALID_MNEM_STR_LENGTHS = [51, 58, 64, 70, 77]   (mnem)
```

So it **tops out at 77 characters**, and `validate_record` on the 90-char vector returns, measured:

```
Err(Invalid("string length 90 outside v0.1 set [50, 56, 62, 69, 75]"))
```

The F-113 rule correctly does not fire — but `me` still refuses the record, for a separate,
pre-existing reason with nothing to do with plate geometry. The Rust boundary test therefore asserts
**not-`MsTooLong`**, not `Ok`, and its doc comment says so in full. The Go test is where outright
admission at 90 is asserted, because `codex32.New` has no such narrowing.

**This exposes a pre-existing divergence worth a follow-up (not introduced here):**
the device admits *any* valid BIP-93 codex32 secret up to 90 characters; `me seal` will only seal a
*constellation* `ms1` at one of ten specific lengths ≤ 77. An operator with an ordinary third-party
BIP-93 backup can engrave it on the device but cannot seal it with `me`. Either the device should
narrow to the constellation set or `me` should widen to BIP-93 — that is a design decision above my
brief, and I did not make it. I flag it rather than paper over it.

### 5.2 The `wipe(out)` call site is still not regression-testable

See G11 above. Not fixed, because fixing it means `unsafe` or an API change; documented at the call
site and in `wipe`'s doc comment, and the *placement* it protects is separately pinned by G3's test.

### 5.3 Scope kept

- `EngraveSeedString`'s 33 cap: **unchanged** (only renamed to `seedQRMaxSize`). QR version support:
  **untouched**. F-117 / F-118 remain deferred.
- `biptool`: **not capped**. It still warns (F-116) and still emits the long codes these vectors
  need.
- No merge into `master` or `b2b`.

---

## 6. Verification output — every line from a command that was run

### 6.1 Rust (`/scratch/code/shibboleth/me-wt-f113`)

```
$ cargo test
CARGO TEST EXIT=0
  unittests src/lib.rs   -> ok. 123 passed; 0 failed
  unittests src/main.rs  -> ok.   1 passed; 0 failed
  tests/cli.rs           -> ok.  30 passed; 0 failed
  tests/cross_lang.rs    -> ok.   1 passed; 0 failed
  tests/golden.rs        -> ok.   3 passed; 0 failed
  tests/preview_cross_lang.rs -> ok. 1 passed; 0 failed
  tests/prop.rs          -> ok.   6 passed; 0 failed
  tests/seal_cli.rs      -> ok.  14 passed; 0 failed
  (doctests)             -> ok.   0 passed; 0 failed
TOTAL: 179 passed, 0 failed, 0 FAILED lines in the log   [counted by tool, not by hand]

$ cargo clippy --all-targets
CLIPPY EXIT=0        (0 warning/error lines)

$ cargo fmt --check
FMT EXIT=1 — 7 diffs, ALL PRE-EXISTING.
Baseline on master:  7 diffs (lib.rs, preview.rs, tests/cli.rs ×2, cross_lang.rs ×2,
                     preview_cross_lang.rs) under rustfmt 1.9.0-nightly (52b6e2c208 2026-04-27).
Worktree after change: the SAME 7. `record.rs` was rustfmt'd so the 2 diffs my edit
introduced are gone; neither of my two files appears in the list.
```

Behavioural red before implementing (the TDD gate), verbatim:

```
---- seal::record::tests::refuses_an_ms1_longer_than_the_machine_can_engrave stdout ----
assertion `left == right` failed: a 91-character codex32 secret must be refused by §10.2.1a
  left: Err(Invalid("string length 91 outside v0.1 set [50, 56, 62, 69, 75]"))
 right: Err(MsTooLong(91))
```

— i.e. the pre-change behaviour was literally the "payload unreadable" class.

New Rust tests (7): `refuses_an_ms1_longer_than_the_machine_can_engrave`,
`does_not_fire_at_the_ninety_character_boundary`, `the_too_long_message_names_the_length_and_the_cap`,
`does_not_fire_on_a_long_non_codex32_ms1_string`, `does_not_cover_md_or_mk_records`,
`the_cap_is_a_literal_whose_authority_is_the_go_test`, and CLI
`refuses_an_ms1_too_long_for_the_seed_plate`.

### 6.2 Go (`/scratch/code/shibboleth/sh-wt-f113`)

```
$ nix develop /scratch/code/shibboleth/seedhammer --command go test ./...
GO TEST EXIT=1
  48 packages: ok
  FAIL  seedhammer.com/cmd/kdfbench [setup failed]
  FAIL  seedhammer.com/cmd/sealread [setup failed]
```

Both failures are `package machine is not in std` — TinyGo-only, and **confirmed pre-existing** by
running the same two packages on `seedhammer-b2b` (byte-identical output). No other package fails.

```
$ gofmt -l seal/ gui/ backup/
gui/bip85_test.go
gui/md1_expand_fuzz_test.go
gui/multisig_build_test.go
gui/multisig_match.go
gui/multisig_testhelpers_test.go
```

Identical list on `b2b` (baseline). **None of the six files touched here appears.**

```
$ go vet ./seal/ ./gui/ ./backup/
VET EXIT=1
backup/backup_test.go:393:48:            testing.ArtifactDir requires go1.26 or later (file is go1.25)
backup/freetext_test.go:240:48:          testing.ArtifactDir requires go1.26 or later (file is go1.25)
gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)

$ diff vet-base.log vet-new.log   ->  (no output)   VET DIFF vs baseline: none
```

Baseline `go vet` on `b2b` is exit 1 with the identical three diagnostics; my change adds none, and
none of them is in a file I touched.

Behavioural red before implementing (the TDD gate), verbatim:

```
=== RUN   TestRefusesACodex32SecretTooLongToEngrave
    engraveable_test.go:106: 91-character secret: 1 records admitted on a rejected payload
    engraveable_test.go:110: 91-character secret: admitted, want ErrCodex32TooLong
    engraveable_test.go:106: 93-character secret: 1 records admitted on a rejected payload
    engraveable_test.go:110: 93-character secret: admitted, want ErrCodex32TooLong
    engraveable_test.go:106: 125-character secret: 1 records admitted on a rejected payload
    engraveable_test.go:110: 125-character secret: admitted, want ErrCodex32TooLong
    engraveable_test.go:106: 127-character secret: 1 records admitted on a rejected payload
    engraveable_test.go:110: 127-character secret: admitted, want ErrCodex32TooLong
--- FAIL: TestRefusesACodex32SecretTooLongToEngrave (0.00s)
=== RUN   TestTooLongSecretIsCaughtInThePerRecordPass
    engraveable_test.go:146: got <nil>, want ErrCodex32TooLong
--- FAIL: TestTooLongSecretIsCaughtInThePerRecordPass (0.00s)
```

— i.e. before the change, all four over-length secrets were **admitted**.

Final state of the new Go tests (10 new, all green):

```
--- PASS: TestEngraveableVectorsAreWhatTheyClaim
--- PASS: TestAdmitsACodex32SecretAtTheEngraveableLimit
--- PASS: TestRefusesACodex32SecretTooLongToEngrave
--- PASS: TestTooLongSecretIsCaughtInThePerRecordPass
--- PASS: TestEngraveableLimitDoesNotCoverMDMKRecords
--- PASS: TestEngraveableLimitDoesNotCoverMnemonics
--- PASS: TestPublicSectionStillReportsAnOverLongSecretAsNotPermitted
ok      seedhammer.com/seal
--- PASS: TestEngraveableLimitIsDerivedFromTheRealQREncoder
--- PASS: TestEngraveSeedStringCutsAt90AndRefusesAt91
ok      seedhammer.com/backup
--- PASS: TestUnlockNamesAnUnengraveableSecretInsteadOfCallingItUnreadable
ok      seedhammer.com/gui
```

(The `TestEngraveSeedStringTooLong`, `TestPassphraseQRTooLong` and `TestPassphraseEntryRejectsTooLong`
lines that also match the filter are pre-existing tests, still green.)

---

## 7. What a reviewer should look at first

1. **§5.1** — the Rust/Go divergence in what counts as an `ms1`. It is pre-existing, but F-113 is the
   first thing to make it visible, and the follow-up decision is not mine.
2. **`gui/unlock_kdf.go`** — not named in the brief. Without it the sentinel is invisible to the
   operator and §10.2.1a's distinguishability requirement is unmet. Mutation G9 is the evidence.
3. **`backup/backup.go`'s two new constants** — behaviour-neutral, but they are a production-file
   change made for a test's benefit. Mutation G7 vs G7b is the justification.
4. **G11** — the one surviving mutation.
