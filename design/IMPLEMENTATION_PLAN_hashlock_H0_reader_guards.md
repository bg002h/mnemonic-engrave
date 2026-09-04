# Hashlock H0 — Reader Guards Implementation Plan

**STATUS: DRAFT 2026-09-04 — BUILD GATE GREEN (controller hand-wire, both repos; output in this file's commit message); R0 not yet dispatched.**
H0 is the prerequisite `SPEC_ms_hashlock.md` §9 places BEFORE ms-cli 0.18.0
(a controller default awaiting the operator: H0 precedes the release rather
than following it as H2). Its ordering is the operator's; its content is
needed under either ordering.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make a kind-`0x03` hashlock PREIMAGE plate string inert on the two
readers that would today take it for a seed — the SeedHammer fork's two
classifiers and `me`'s record validator — with one shared vector row that
turns red on whichever side ever regresses, and get the fork change flashed.

**Architecture:** One new predicate in the fork's `codex32` package,
`IsPreimage(String) bool`, reads the first payload byte; the two device
classifiers (`sysw.isStrictMs1`, `seal.Classify`) and the one engrave path
(`gui.unlockEngraveCodex32`) consult it and treat a preimage string as
UNKNOWN (no new class). On the host, `me` at its ms-codec `0.7` pin already
refuses the string at the codec's prefix gate; H0 adds the row and a pin test
so the coming `0.8` bump (H1b) cannot map the codec's new success to
`RecordKind::Ms` unnoticed. The shared `codex32_seam_vectors.json` corpus,
pinned by sha256 in both suites, carries the row.

**Tech Stack:** Rust (`me-cli`, ms-codec `0.7` pinned), Go 1.26 at
`/scratch/code/shibboleth/.toolchain/go/bin/go` (fork tests) and the nix
flake's Go for firmware (`nix develop -c tinygo build ...`), `~/bin/sh/sh2-flash`.

**Spec:** `mnemonic-secret/design/SPEC_ms_hashlock.md` §1 (kind `0x03`, id
`hash`, 75 characters), §9 (the reader table and H0), §14 (citations).

**Baselines (for `scripts/plan-staleness-check.sh`):** mnemonic-engrave
`e06e29d`; seedhammer fork main `839fa5aa`; mnemonic-secret `3592532` (the
H1 plan whose gate produced the vector string).

## Global Constraints

- **Rust-primary rule (CLAUDE.md):** this is a CONVERGENCE fix, exemption (a).
  The Rust primary of both device classifiers is `me`'s `validate_record` →
  `sysw::classify`, which at ms-codec `0.7` refuses every `0x03` string
  (`reserved-prefix byte was 0x03, expected 0x00`, measured — the H1 plan's
  downgrade row). The Go ports accept it because `codex32.New` pins no prefix.
  Go converges on Rust's current answer; nothing is decided in Go. The Rust
  check was done first: `me` refuses at `crates/me-cli/src/seal/record.rs:177`
  via `ms_codec::decode`; the row below pins it.
- **Minimal narrowing.** ONLY prefix `0x03` becomes inert. The fork's known
  wider acceptance (plain BIP-93 strings at 48/74 characters, the
  `device_admits: true` rows) is unchanged; the seam test still requires a
  device-only row and this plan keeps all three shapes. Whether the device
  should converge fully on the constellation profile is a follow-up
  (`FOLLOWUPS.md`, filed in Task 4), not this plan.
- **No new class.** A preimage string classifies `ClassUnknown` on the device
  (`seal`) and `ClassUnknown` in `sysw`; on the host `validate_record` returns
  `Err`. `AdmitSection` therefore refuses an encrypted section that carries
  one with `ErrRecordNotPermitted`, exactly as it refuses any unknown record.
  `me` `0.7` cannot pack such a payload, so this refusal is reachable only by
  hand-built blobs until H2 gives the device a real `0x03` arm.
- **Secret-handling defects never gate** (operator ruling 2026-08-27). The
  vector string is a real preimage plate produced by the gated H1 binary; it
  is a fixture, not anyone's secret, and it is public in two repos.
- **The vector file is shared byte for byte.** Its sha256 after Task 1 is
  `4ac542ea8e0e36d92127b744bce0a83072f787870756bf7b86b9c947bb1370a5`
  (measured on the exact row text below); both literals are re-pinned to it.
- **Fork commits** signed + DCO, author Brian Goss; branch `hashlock-h0` off
  fork `main`; small PR; `Co-Authored-By` / `Claude-Session` trailers as the
  engrave repo uses. **Stage paths explicitly** (no `git add -A`).
- **Flash only via `~/bin/sh/sh2-flash -y`, at the operator's word**, never
  `picotool` by hand. Firmware size is measured before and after.

---

## File Structure

| File | Change | Responsibility |
| --- | --- | --- |
| mnemonic-engrave `crates/me-cli/testdata/codex32_seam_vectors.json` | Modify (append row 9) | the shared host/device corpus — gains `preimage-plate-0x03` |
| mnemonic-engrave `crates/me-cli/tests/codex32_seam.rs:15-16` | Modify (fragment) | re-pin `SEAM_VECTORS_SHA256` |
| mnemonic-engrave `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs` | Create | the host pin: `validate_record` is `Err`, `sysw::classify` is not `Codex32Secret` |
| fork `codex32/mspayload.go` | Modify (fragment) | `msPrefixPreimage = 0x03`; `IsPreimage(String) bool`; `DecodeMS1` UNCHANGED (still `errMSBadPrefix` on `0x03`) |
| fork `codex32/mspayload_test.go` | Modify (append) | `IsPreimage` true on the vector, false on entr/mnem; `DecodeMS1` still refuses |
| fork `sysw/classify.go:116-125` | Modify (fragment) | `isStrictMs1` requires `!codex32.IsPreimage` |
| fork `sysw/testdata/codex32_seam_vectors.json` | Replace (vendored copy) | byte-identical to the primary |
| fork `sysw/codex32_seam_test.go:11` | Modify (fragment) | re-pin `seamVectorsSHA256` |
| fork `seal/record.go:214` | Modify (fragment) | `Classify` requires `!codex32.IsPreimage` before `ClassCodex32Secret` |
| fork `seal/record_test.go:402-418` | Modify (append row + new test) | branch-order row `ClassUnknown`; `AdmitSection` refuses the section |
| fork `gui/unlock_session.go` (`unlockEngraveCodex32`) | Modify (fragment) | named refusal behind the allow-list, defence in depth |
| fork `gui/unlock_session_test.go` | Modify (append) | the refusal text is shown and `EngraveSeedString` is never reached |
| mnemonic-engrave `design/FOLLOWUPS.md`, `CHANGELOG`s, continuity | Modify | records |

**Gate coverage.** Neither `scripts/plan-build-gate-me.sh` (anchors
`sysw/composer_*.rs`, `tests/sysw_composer*.rs`) nor `plan-build-gate-go.sh`
(anchors `md/compose*.go`, `sysw/composer_*.go`, ...) recognises these paths,
and every Go edit here is a fragment of an existing file. So this plan's gate
is the controller hand-wiring every block below into scratch copies of BOTH
repos and running the named test commands before review; its output goes in
the fold commit's message. The reviewer is told what ran.

---

### Task 1: The corpus row and the host pin (mnemonic-engrave)

**Files:**
- Modify: `crates/me-cli/testdata/codex32_seam_vectors.json` (append after row 8, `bip93-bad-checksum`)
- Modify: `crates/me-cli/tests/codex32_seam.rs:15-16`
- Package name for `-p` is `mnemonic-engrave` (the crate at `crates/me-cli/`), not `me-cli`.
- Create: `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs`

**Interfaces:**
- Consumes: `mnemonic_engrave::seal::record::{validate_record, RecordKind, RecordError}` (`pub fn validate_record(s: &str) -> Result<RecordKind, RecordError>`), `mnemonic_engrave::sysw::classify`, `mnemonic_engrave::sysw::record::Class`.
- Produces: the row `preimage-plate-0x03` and the sha256 above, which Task 2 vendors and re-pins.

- [ ] **Step 1: Append the row.** The file ends `    }\n  ]\n}\n`. Replace that tail with `    },\n` + the row + `  ]\n}\n`, indentation exactly as the existing rows (four spaces for the object, six for its keys):

```json
    {
      "name": "preimage-plate-0x03",
      "string": "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c",
      "chars": 75,
      "host_admits": false,
      "device_admits": false,
      "source": "SPEC_ms_hashlock section 1, kind 0x03 with id `hash`: a hashlock PREIMAGE plate, produced by the H1 plan's gate-wired `ms hashlock` (mnemonic-secret 3592532, gate run 13); ms-codec 0.7 refuses it with `reserved-prefix byte was 0x03, expected 0x00` at exit 2 (that plan's downgrade row). NOT A SEED: cut as one it exposes a spend secret as a backup, loaded as one it derives keys from a hashlock preimage. H0 (SPEC_ms_hashlock section 9) makes it INERT on both sides -- never Codex32Secret, no class of its own -- and this row is the tripwire: it goes red on the host the day `me` bumps to ms-codec 0.8 without a refusing arm, and on the device if the prefix test is ever removed."
    }
```

- [ ] **Step 2: Run the seam test to see it fail on the hash.**

Run: `cargo nextest run --locked -p mnemonic-engrave --test codex32_seam`
Expected: FAIL with `testdata/codex32_seam_vectors.json is not the file the fork's copy is pinned to; re-pin BOTH literals`.

- [ ] **Step 3: Measure and re-pin.**

Run: `sha256sum crates/me-cli/testdata/codex32_seam_vectors.json`
Expected: `4ac542ea8e0e36d92127b744bce0a83072f787870756bf7b86b9c947bb1370a5`. If it is not, the row text differs from Step 1 byte for byte — fix the row, do not pin a different hash.

In `crates/me-cli/tests/codex32_seam.rs` replace lines 15-16:

```rust
const SEAM_VECTORS_SHA256: &str =
    "4ac542ea8e0e36d92127b744bce0a83072f787870756bf7b86b9c947bb1370a5";
```

- [ ] **Step 4: Run it again.**

Run: `cargo nextest run --locked -p mnemonic-engrave --test codex32_seam`
Expected: PASS. The host verdict for the new row is `false` already: `sysw::classify` reaches `validate_record`, whose `ms_codec::decode` (0.7) refuses prefix `0x03`. Nine rows: 2 both / 4 device-only / 3 neither.

- [ ] **Step 5: Write the pin test** — green at 0.7 by construction; its job is the 0.8 bump. Create `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs`:

```rust
//! H0 (SPEC_ms_hashlock §9): a kind-0x03 hashlock PREIMAGE plate is never a
//! seed record on the host.
//!
//! At ms-codec 0.7 the codec's own prefix gate refuses the string, so this
//! passes without any change to `validate_record`. When `me` bumps to
//! ms-codec 0.8 (stage H1b) the codec DECODES it as `Payload::Preimage`, and
//! `validate_record`'s `.map(|_| RecordKind::Ms)` would call a preimage a
//! seed. This test is what goes red that day. The seam corpus row
//! `preimage-plate-0x03` pins the same fact through `sysw::classify`.
use mnemonic_engrave::seal::record::{validate_record, RecordError};
use mnemonic_engrave::sysw::record::Class;

/// The H1 plan's downgrade-row string: 75 characters, id `hash`, prefix 0x03.
const PREIMAGE_PLATE: &str =
    "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c";

#[test]
fn a_preimage_plate_is_not_a_seed_record() {
    assert_eq!(PREIMAGE_PLATE.len(), 75);
    // MUTATION: `.map(|_| RecordKind::Ms)` admitting Payload::Preimage after
    // the 0.8 bump -> this arm receives Ok(RecordKind::Ms) and panics.
    match validate_record(PREIMAGE_PLATE) {
        Ok(kind) => panic!("validate_record admitted a 0x03 preimage plate as {kind:?}"),
        // Not `MsTooLong`: 75 characters is inside the engraveable cap, and
        // the refusal must be about the KIND, not the length.
        Err(RecordError::MsTooLong(n)) => panic!("refused as too long ({n}), not as a preimage"),
        Err(_) => {}
    }
    assert_ne!(
        mnemonic_engrave::sysw::classify(PREIMAGE_PLATE),
        Class::Codex32Secret,
        "sysw::classify called a preimage plate a codex32 secret"
    );
}
```

- [ ] **Step 6: Run it.**

Run: `cargo nextest run --locked -p mnemonic-engrave --test preimage_plate_is_not_a_seed`
Expected: PASS (1 test). Then prove it can fail: in a scratch copy, insert `if s.starts_with("ms10hash") { return Ok(RecordKind::Ms); }` immediately before `record.rs:176`'s `ms_codec::decode(s)` and re-run both tests — Expected (measured at the gate): `a_preimage_plate_is_not_a_seed_record` FAILS with `validate_record admitted a 0x03 preimage plate as Ms`, and `the_host_never_admits_what_the_device_would_refuse` FAILS with `preimage-plate-0x03: host verdict`. Revert.

- [ ] **Step 7: Whole-crate check and commit.**

Run: `cargo nextest run --locked -p mnemonic-engrave && cargo clippy --locked -p mnemonic-engrave --all-targets -- -D warnings && cargo fmt -p mnemonic-engrave -- --check`
Expected: all green under the toolchain CI uses. (Measured at the gate: tests and fmt green; the local nightly's clippy reports one pre-existing `manual implementation of .is_multiple_of()` in the library, not in these files — CI's clippy at `917d4e3` is green, so that lint is the local toolchain's, not this task's. If CI's clippy ever adopts it, fix it in its own commit.)

```bash
git add crates/me-cli/testdata/codex32_seam_vectors.json crates/me-cli/tests/codex32_seam.rs crates/me-cli/tests/preimage_plate_is_not_a_seed.rs
git commit -m "seam corpus: preimage-plate-0x03 row (host false / device false) + host pin test (hashlock H0)"
```

---

### Task 2: The device guard (seedhammer fork)

**Files:**
- Modify: `codex32/mspayload.go:8-12` (consts) and append `IsPreimage`
- Modify: `codex32/mspayload_test.go` (append)
- Modify: `sysw/classify.go:116-125` (`isStrictMs1`)
- Replace: `sysw/testdata/codex32_seam_vectors.json` (vendored copy of Task 1's file)
- Modify: `sysw/codex32_seam_test.go:11`
- Modify: `seal/record.go:214`
- Modify: `seal/record_test.go` (row in `TestClassifyMirrorsScanBranchOrder`; new test)
- Modify: `gui/unlock_session.go` (`unlockEngraveCodex32`)
- Modify: `gui/unlock_session_test.go` (append)

**Interfaces:**
- Consumes: `codex32.String` (`Seed() []byte`), `seal.AdmitSection([][]byte, Section) ([]AdmittedRecord, error)`, `seal.ErrRecordNotPermitted`, `sysw.Classify(string) Class`.
- Produces: `codex32.IsPreimage(s String) bool`.

Work on branch `hashlock-h0` from fork `main` (`839fa5aa`). Go is
`/scratch/code/shibboleth/.toolchain/go/bin/go`.

- [ ] **Step 1: Vendor the corpus and re-pin; watch the seam test go RED for the right reason.**

```bash
cp ../mnemonic-engrave/crates/me-cli/testdata/codex32_seam_vectors.json sysw/testdata/codex32_seam_vectors.json
sha256sum sysw/testdata/codex32_seam_vectors.json   # 4ac542ea…70a5
```

In `sysw/codex32_seam_test.go` replace line 11:

```go
const seamVectorsSHA256 = "4ac542ea8e0e36d92127b744bce0a83072f787870756bf7b86b9c947bb1370a5"
```

Run: `go test -count=1 -run TestCodex32SeamDeviceAdmitsEverythingTheHostDoes ./sysw/`
Expected (measured at the gate): FAIL with exactly one line, `codex32_seam_test.go:66: preimage-plate-0x03: device admits = true, want false (Classify = 2)` (`sysw.Class` has no `String()`; 2 is `ClassCodex32Secret`). This is the measurement the spec's reader table records, now as a failing test.

- [ ] **Step 2: The predicate.** In `codex32/mspayload.go` replace lines 8-12 (the const block):

```go
const (
	msPrefixEntr     = 0x00 // RESERVED_PREFIX: payload = [0x00][entropy]
	msPrefixMnem     = 0x02 // MNEM_PREFIX:     payload = [0x02][language][entropy]
	msPrefixPreimage = 0x03 // PREIMAGE_PREFIX: payload = [0x03][32-byte hashlock preimage] (SPEC_ms_hashlock §1)
	msMaxLanguage    = 9    // MNEM_LANGUAGE_NAMES indices 0..9
)
```

and append at the end of the file:

```go
// IsPreimage reports whether a New-valid string carries the m-format HASHLOCK
// PREIMAGE kind (SPEC_ms_hashlock §1: payload = [0x03][32 bytes], id `hash`).
//
// H0 (SPEC_ms_hashlock §9): such a string is INERT on this device — never a
// codex32 SECRET and no class of its own — because every path that admits
// ClassCodex32Secret ends at backup.EngraveSeedString, and a hashlock
// preimage is not a seed: engraved as one it exposes a spend secret as a
// backup. DecodeMS1 is deliberately unchanged and still refuses the prefix;
// the device learns to USE a preimage in stage H2, not here.
//
// Reads one byte of the payload. Seed() returns the parsed data the String
// already holds; nothing new is retained.
func IsPreimage(s String) bool {
	d := s.Seed()
	return len(d) > 0 && d[0] == msPrefixPreimage
}
```

- [ ] **Step 3: Its test.** Append to `codex32/mspayload_test.go`:

```go
// H0: the preimage kind is recognised by its prefix byte and by nothing else,
// and DecodeMS1 keeps refusing it (the seed decoder must not learn a kind that
// is not a seed).
func TestIsPreimageReadsThePrefixByteOnly(t *testing.T) {
	const plate = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c"
	s, err := New(plate)
	if err != nil {
		t.Fatalf("New(plate): %v", err)
	}
	if !IsPreimage(s) {
		t.Fatalf("IsPreimage(plate) = false, want true (Seed()[0] = %#x)", s.Seed()[0])
	}
	if id, _, _ := s.Split(); id != "hash" {
		t.Errorf("id = %q, want hash", id)
	}
	// MUTATION: `d[0] == msPrefixPreimage` -> `d[0] != msPrefixEntr` would
	// call every mnem string a preimage; the entr and mnem seams below catch it.
	for _, v := range []string{
		"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f", // constellation-entr-128, prefix 0x00 (the seam corpus's yes/yes row)
	} {
		e, err := New(v)
		if err != nil {
			t.Fatalf("New(%q): %v", v, err)
		}
		if IsPreimage(e) {
			t.Errorf("IsPreimage(%q) = true, want false", v)
		}
	}
	if _, _, _, err := DecodeMS1(s); err != errMSBadPrefix {
		t.Errorf("DecodeMS1(plate) err = %v, want errMSBadPrefix: the seed decoder must not decode a preimage", err)
	}
}
```

Run: `go test -count=1 -run TestIsPreimageReadsThePrefixByteOnly ./codex32/`
Expected: PASS. (Settled at the gate: the 23-character `ms10entrsqqg5y2z9pzs3gg` that `seal/record_test.go:441` uses for `wipe` is NOT `New`-valid — `codex32: invalid length` — so the entr fixture is the seam corpus's 50-character `constellation-entr-128` string above, which `New` accepts.)

- [ ] **Step 4: `isStrictMs1`.** Replace `sysw/classify.go:123-124` (`_, err := codex32.New(record)` / `return err == nil`):

```go
	c, err := codex32.New(record)
	// H0 (SPEC_ms_hashlock §9): a hashlock preimage plate is BCH-valid and
	// inside the cap, and it is not a seed. Inert here — no class of its own.
	return err == nil && !codex32.IsPreimage(c)
```

Run: `go test -count=1 ./sysw/`
Expected: PASS, the seam test included (9 rows: 2 both / 4 device-only / 3 neither). Mutation, measured: with the `!codex32.IsPreimage(c)` clause removed again, the seam test fails on exactly the Step 1 line.

- [ ] **Step 5: `seal.Classify`.** Replace `seal/record.go:214-216`:

```go
	if c, err := codex32.New(s); err == nil && !codex32.IsPreimage(c) {
		return ClassCodex32Secret
	}
```

- [ ] **Step 6: seal tests.** In `TestClassifyMirrorsScanBranchOrder` (`seal/record_test.go:403-418`) add a row after `{d.Public[2], ClassMDMK}`:

```go
		{sealPreimagePlate, ClassUnknown}, // H0: a hashlock preimage plate is not a secret and has no class
```

and append to the file:

```go
// H0 (SPEC_ms_hashlock §9): the 75-character kind-0x03 preimage plate from the
// shared seam corpus (sysw/testdata/codex32_seam_vectors.json, row
// preimage-plate-0x03). BCH-valid, inside the cap, and NOT a seed.
const sealPreimagePlate = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c"

// An encrypted section carrying a preimage plate is refused whole, exactly as
// one carrying any unknown record is — the payload never reaches the unlock
// screen with an "ms1" it would cut as a seed.
func TestAdmitSectionRefusesAPreimagePlateAsUnknown(t *testing.T) {
	_, err := AdmitSection([][]byte{[]byte(sealPreimagePlate)}, SectionEncrypted)
	if !errors.Is(err, ErrRecordNotPermitted) {
		t.Fatalf("AdmitSection(preimage plate, encrypted) err = %v, want ErrRecordNotPermitted", err)
	}
	if !strings.Contains(err.Error(), "unknown") {
		t.Errorf("error %q does not name the class unknown", err)
	}
	// MUTATION: drop `!codex32.IsPreimage(c)` from Classify -> admitted as
	// codex32-secret, err == nil, this test fails on the first check.
}
```

(`errors` and `strings` are already imported by `seal/record_test.go`, lines 4-5.)

Run: `go test -count=1 ./seal/`
Expected: PASS. Mutation, measured: with Step 5's `!codex32.IsPreimage(c)` removed, `TestClassifyMirrorsScanBranchOrder` fails with `Classify("ms10hashsqw46h2at4w46h2a") = codex32 secret, want unknown format` and `TestAdmitSectionRefusesAPreimagePlateAsUnknown` fails with `err = <nil>, want ErrRecordNotPermitted`.

- [ ] **Step 7: Defence in depth on the engrave path.** In `gui/unlock_session.go`, `unlockEngraveCodex32`, after the `codex32.New` error check and before `id, _, _ := s.Split()`:

```go
	if codex32.IsPreimage(s) {
		// Unreachable behind seal.Classify's H0 guard, which never admits a
		// preimage plate as ClassCodex32Secret. Named rather than assumed:
		// this is the one call that cuts metal.
		showError(ctx, th, unlockTitle, "This record is a hashlock preimage, not a seed. It is not engraved as one.")
		return
	}
```

Append to `gui/unlock_session_test.go` (the harness is `runUnlockEngraveMnemonic` at `:714`; this is its twin for the ms1 arm — there was no codex32 driver before):

```go
// runUnlockEngraveCodex32 is runUnlockEngraveMnemonic's twin for the ms1 arm.
func runUnlockEngraveCodex32(t *testing.T, pf Platform, rec []byte) *sessionHarness {
	t.Helper()
	ctx := NewContext(pf)
	returned := false
	frame, drawer, quit := runUITouch(ctx, func() {
		unlockEngraveCodex32(ctx, &descriptorTheme, rec)
		returned = true
	})
	h := &sessionHarness{t: t, ctx: ctx, done: &returned}
	h.frame, h.drawer = frame, drawer
	t.Cleanup(quit)
	return h
}

// H0 (SPEC_ms_hashlock §9), defence in depth: even if a kind-0x03 preimage
// plate reached the one call that cuts metal, it is refused by name and no
// engrave screen is shown. seal.Classify never admits one, so this is the
// second guard, not the first.
func TestUnlockEngraveCodex32RefusesAPreimagePlate(t *testing.T) {
	const plate = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c"
	h := runUnlockEngraveCodex32(t, newPlatform(), []byte(plate))
	// MUTATION: drop the IsPreimage check in unlockEngraveCodex32 -> the flow
	// reaches the EngraveSeed screen for the plate, and this never sees the
	// refusal text.
	h.mustReach("hashlock preimage")
}
```

Run: `go test -count=1 -run 'TestUnlock' ./gui/`
Expected: PASS. Mutation, measured: with the guard removed the new test fails with `never reached "hashlock preimage"; last frame "Insert a blank plate and close the lock. Hold button to start the engraving process. ... Engrave Plate"` — the device, handed a preimage plate through this path, would cut it. That frame is the whole reason H0 exists.

- [ ] **Step 8: Whole surface, then commit.**

Run: `go vet ./codex32/ ./sysw/ ./seal/ && go test -count=1 ./codex32/ ./sysw/ ./seal/ && ../mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24` (the shard script lives in mnemonic-engrave `scripts/`; run it from the fork checkout as the S-stages did).
Expected: all green. (`go vet ./gui/` under the 1.26.7 toolchain reports two PRE-EXISTING complaints, `testing.ArtifactDir requires go1.26 or later (file is go1.25)` in `freetext_sizeproof_golden_test.go:111` and `transaction_golden_test.go:104`; they are on `main` today and not this task's. The gui package's tests still compile and run.)

```bash
git add codex32/mspayload.go codex32/mspayload_test.go sysw/classify.go sysw/testdata/codex32_seam_vectors.json sysw/codex32_seam_test.go seal/record.go seal/record_test.go gui/unlock_session.go gui/unlock_session_test.go
git commit -s -m "hashlock H0: a kind-0x03 preimage plate is inert (never a codex32 secret) on both classifiers and the engrave path"
```

---

### Task 3: Size, flash, boot (operator's word required for the flash)

- [ ] **Step 1: Measure the firmware.** From the fork checkout with nix on `PATH`:

Run: `nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`
Expected: flash within a few hundred bytes of `1,582,628` (main `839fa5aa`); RAM `62,800`. Record both numbers in the fold.

- [ ] **Step 2: Merge to fork `main`** after the post-implementation review (Task 4) is GREEN, then flash: `~/bin/sh/sh2-flash -y` **only at the operator's word**, with the SH2 in BOOTSEL. Boot judgement is the operator's (PD negotiation can be slow: a dark screen is not a rejected signature).

- [ ] **Step 3: Device acceptance** is deferred to H2, when `me` can pack a payload carrying a `0x03` record; until then the fork's `seal` admission test is the acceptance. State that plainly in the continuity entry.

---

### Task 4: Records and review

- [ ] `design/FOLLOWUPS.md`: file the wider question — should the device converge fully on the constellation profile (refuse plain BIP-93 at 48/74 characters)? Owning phase: none (operator decision). Also record that `me`'s 0.8 bump (H1b) MUST add an explicit `Payload::Preimage` refusal arm in `validate_record`, guarded by Task 1's tests — owning phase H1b.
- [ ] `crates/me-cli/CHANGELOG.md` (unreleased): the corpus row and the pin test. Fork `CHANGELOG` if it keeps one.
- [ ] Post-implementation review (risk set: a reader that cuts seeds): ONE opus adversarial execution review over the whole diff of both repos, brief in `design/agent-briefs/hashlock-H0-post-impl-brief.md`, report to `design/agent-reports/hashlock-H0-post-impl.md`; GREEN before the fork merge and the flash.
- [ ] Continuity entry + memory; push engrave via `scripts/push-via-staging.sh master`; fork via its normal PR to `bg002h/seedhammer` main.

---

## Self-review

1. **Spec coverage.** §9 H0 (a): `isStrictMs1` / `seal.Classify` gain the prefix test — Task 2 Steps 4-5, plus the engrave path (Step 7) the spec's reader table named (`unlockEngraveCodex32`). "With the record-class vector row" — Task 1's seam row is that row; both suites read it. "Merged and flashed" — Task 3. §9 H0 (b): `me`'s `validate_record` treats `0x03` as inert "in the same release window as the 0.8 bump" — Task 1's pin test is green at 0.7 and is the tripwire for H1b; the refusing arm itself belongs to the bump (recorded as an H1b-owned follow-up in Task 4), because at 0.7 there is no `Payload::Preimage` to match.
2. **Placeholders.** Task 2 Step 3's alternate fixture and Step 7's harness are conditionals the controller's gate resolves before review, and the plan says so; neither is "TBD".
3. **Type consistency.** `IsPreimage(s String) bool` is defined once (Task 2 Step 2) and called with a `codex32.String` at all three sites; `Seed()` is `func (s String) Seed() []byte` (`codex32/codex32.go:386`). `AdmitSection(records [][]byte, section Section) ([]AdmittedRecord, error)` (`seal/record.go:244`). `validate_record(s: &str) -> Result<RecordKind, RecordError>` (`record.rs:117`); `RecordError::MsTooLong(usize)` exists (`record.rs:71`); `validate_record`'s `ms_codec::decode(s)` arm is at `record.rs:176`.
