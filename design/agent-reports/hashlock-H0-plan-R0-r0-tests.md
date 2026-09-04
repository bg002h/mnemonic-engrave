# Hashlock H0 plan — R0 round 0, tests/mutation lens

Reviewer: independent sonnet tests/mutation reviewer. Plan under review:
`design/IMPLEMENTATION_PLAN_hashlock_H0_reader_guards.md` at `b0af794`. Fork:
`/scratch/code/shibboleth/seedhammer` at `main` `839fa5aa`. Spec:
`mnemonic-secret/design/SPEC_ms_hashlock.md` §1, §9.

**Scratch copies used (my own, never the controller's):** fork tar copy at
`/scratch/code/shibboleth/.tmp/h0-fork-tests`; engrave detached worktree at
`/scratch/code/shibboleth/me-worktrees/h0-tests` (`e06e29d`), removed as the
final step of this review. Go: `/scratch/code/shibboleth/.toolchain/go/bin/go`
(`go1.26.7`) first on `PATH`. Rust: `PATH=$HOME/.cargo/bin:$PATH
CARGO_TARGET_DIR=/scratch/code/shibboleth/mnemonic-engrave/target
TMPDIR=/scratch/code/shibboleth/.tmp`, package `-p mnemonic-engrave`.

**ONE QUESTION answered:** every test the plan adds CAN fail on the defect it
names (all four declared mutations reproduced, verbatim output below), and the
plan's mutation table holds. Two real gaps were found by mutations of my own
that the plan does not declare: (1) the guard's own dedicated unit test
(`TestIsPreimageReadsThePrefixByteOnly`) and the `seal` package's tests do
**not** catch `d[0] != msPrefixEntr`, `d[0] >= 0x03`, or a dropped
`isStrictMs1` guard — only `sysw`'s shared seam test does, via pre-existing
rows; (2) most significantly, **the entire corpus cannot distinguish "keyed on
the prefix byte" from "keyed on the 4-character id"** — an `IsPreimage`
implemented as `id == "hash"` passes every existing test in all four packages
(codex32, sysw, seal, gui), and is proven wrong on a constructed input the
corpus lacks (below).

---

## Corpus row hash claim — verified TRUE

Applied Task 1 Step 1's row text exactly (indentation, JSON escaping,
`--` in place of em-dash). `sha256sum` of the resulting
`crates/me-cli/testdata/codex32_seam_vectors.json`:

```
4ac542ea8e0e36d92127b744bce0a83072f787870756bf7b86b9c947bb1370a5
```

Matches the plan's claimed hash exactly.

## 1. RED steps — reproduced verbatim

**Task 1 Step 2** (before re-pinning `SEAM_VECTORS_SHA256`):

```
thread 'the_host_never_admits_what_the_device_would_refuse' panicked at crates/me-cli/tests/codex32_seam.rs:33:5:
assertion `left == right` failed: testdata/codex32_seam_vectors.json is not the file the fork's copy is pinned to; re-pin BOTH literals
  left: "4ac542ea8e0e36d92127b744bce0a83072f787870756bf7b86b9c947bb1370a5"
 right: "3d53ef88a474f02c15aa60a839f4a31071598a26c853463122a847515926eb6a"
```

Matches the plan's expected text exactly.

**Task 2 Step 1** (fork, after vendoring the corpus and re-pinning
`seamVectorsSHA256`, before touching any guard code):

```
--- FAIL: TestCodex32SeamDeviceAdmitsEverythingTheHostDoes (0.00s)
    codex32_seam_test.go:66: preimage-plate-0x03: device admits = true, want false (Classify = 2)
FAIL
```

Matches the plan's expected text exactly (one line, `Classify = 2` =
`ClassCodex32Secret`).

**Line-citation drift (Minor, not a defect):** the plan says "In
`sysw/codex32_seam_test.go` replace line 11", but the actual constant sits at
line 30 (the file's header comment is longer than at plan-writing time).
Likewise "Replace `seal/record.go:214-216`" — the actual match is at lines
212-214. Both edits still apply cleanly by content match; neither is a
functional problem, just a stale citation.

## 2. The plan's declared mutations — all four run, quoted, reverted, re-green

| # | Mutation | Result | Quoted assertion |
| --- | --- | --- | --- |
| Task 1 Step 6 | `record.rs`: `if s.starts_with("ms10hash") { return Ok(RecordKind::Ms); }` before `ms_codec::decode(s)` | CAUGHT, both tests | `validate_record admitted a 0x03 preimage plate as Ms` (new pin test); `assertion left == right failed: preimage-plate-0x03: host verdict / left: true / right: false` (seam test) |
| Task 2 Step 4 | `isStrictMs1`: `!codex32.IsPreimage(c)` clause removed | CAUGHT | `codex32_seam_test.go:66: preimage-plate-0x03: device admits = true, want false (Classify = 2)` — exactly the Step 1 line |
| Task 2 Step 6 | `seal.Classify`: `!codex32.IsPreimage(c)` clause removed | CAUGHT, both tests | `record_test.go:419: Classify("ms10hashsqw46h2at4w46h2a") = codex32 secret, want unknown format`; `record_test.go:466: AdmitSection(preimage plate, encrypted) err = <nil>, want ErrRecordNotPermitted` |
| Task 2 Step 7 | `gui/unlock_session.go`: `IsPreimage` guard block removed from `unlockEngraveCodex32` | CAUGHT | `never reached "hashlock preimage"; last frame "Insertablankplateandclosethelock.Holdbuttontostarttheengravingprocess.Theprocessisloud,usehearingprotection.EngravePlate"` |

All four reverted; both repos confirmed green afterward (host: `cargo nextest
run --locked -p mnemonic-engrave --test codex32_seam --test
preimage_plate_is_not_a_seed` → 2 passed; fork: `go test -count=1 ./codex32/
./sysw/ ./seal/` → all `ok`, plus `go test -count=1 -run 'TestUnlock' ./gui/`
→ `ok`).

Whole-crate/package check (Task 1 Step 7, Task 2 Step 8): host
`cargo nextest run --locked -p mnemonic-engrave` — the two new tests and the
rest of the seam-adjacent surface pass; 3 **pre-existing, environment-only**
failures in `history_purge` (`/usr/bin/zsh is required` — this scratch box has
no `/usr/bin/zsh` at all) are unrelated to Task 1's diff and out of scope.
`cargo clippy` reproduces exactly the plan's named pre-existing lint,
`manual implementation of .is_multiple_of()` at
`crates/me-cli/src/sysw/composer_records.rs:114:8`, nowhere in Task 1's files.
`cargo fmt --check` is clean. Fork: `go vet ./gui/` reproduces the plan's two
named pre-existing complaints (`freetext_sizeproof_golden_test.go:111`,
`transaction_golden_test.go:104`); the full `gui` package via
`scripts/gui-shard-test.sh ./gui/ 24` — 1202/1202 tests, 24/24 shards `ok`,
wall 36s.

## 3. My own mutations

| Mutation | Caught by | Quoted / notes |
| --- | --- | --- |
| `IsPreimage`: `d[0] >= msPrefixPreimage` (accepts every unallocated future prefix ≥3) | `sysw` seam test only | `bip93-secret-128/256`, `bip93-share`: `device admits = false, want true (Classify = 0)` — these rows have `Seed()[0]` = `0x31`/`0xff`/`0xff`, all ≥3. NOT caught by `codex32`'s own `TestIsPreimageReadsThePrefixByteOnly` or by any `seal` test. |
| `IsPreimage`: `d[0] != msPrefixEntr` (calls anything non-entr a preimage) | `sysw` seam test only, same 3 rows, `Classify = 0` | Survives `TestIsPreimageReadsThePrefixByteOnly` (its only fixture is the entr string, `0x00 != 0x00` = false, matches expected) and survives every `seal` test. The plan's own in-code comment — *"the entr and mnem seams below catch it"* — overstates this: there is no mnem fixture in that test, and the actual catch is a different package's pre-existing rows, not "below" in the same file. **Minor** (wording only; the mutation is genuinely caught, just not where/how the comment implies). |
| Seam row `device_admits` flipped `false → true` (JSON edited, hash re-pinned on each side to isolate the semantic check from the drift guard) | `sysw` device-side test only | `codex32_seam_test.go:66: preimage-plate-0x03: device admits = false, want true (Classify = 0)`. The **host-side Rust test does not and structurally cannot catch this** — it never invokes Go code; it only checks its own `host_admits` column and the `host <= device` inequality, which a false `device_admits: true` cannot violate. By design (each suite is compared to the file, never to the other suite), not a defect. |
| `permitted()` in `seal/record.go`: admit `ClassUnknown` in `SectionEncrypted` | Two tests | New: `record_test.go:466: AdmitSection(preimage plate, encrypted) err = <nil>, want ErrRecordNotPermitted`. Pre-existing: `record_test.go:115: unknown format must not be permitted in the encrypted section` (`TestPublicSectionRefusesAddressAndDescriptor`) — double coverage, strong. |
| GUI refusal string changed to a generic message (drops "hashlock preimage") | CAUGHT | `never reached "hashlock preimage"; last frame "Thisrecordcannotbeengravedasaseed.SealedPayload"` — confirms the test pins the *specific* wording, not just "some refusal happened." |
| `isStrictMs1`'s `!codex32.IsPreimage(c)` dropped, `seal.Classify`'s kept | `sysw` seam test **only** | `codex32_seam_test.go:66: preimage-plate-0x03: device admits = true, want false (Classify = 2)`. All of `seal` and `gui` pass unaffected (they use `seal.Classify`, not `isStrictMs1`). **`isStrictMs1`'s H0 behavior has exactly one test protecting it: the shared seam row.** |
| `DecodeMS1` given a `case msPrefixEntr, msPrefixPreimage:` arm (decodes a 0x03 payload as if it were entr entropy) | `codex32`'s own unit test only | `TestIsPreimageReadsThePrefixByteOnly` fails (its `DecodeMS1(s) != errMSBadPrefix` assertion). `sysw`, `seal`, `gui` all pass unaffected — none of them exercise `DecodeMS1` on this path (by design; H0 leaves `DecodeMS1` untouched, H2 is where it would matter). |
| Rust pin test: `Err(RecordError::MsTooLong(n)) => panic!(...)` arm deleted, folded into a bare `Err(_) => {}` | **SURVIVED** (no behavior change) | Test still PASSES identically. At 0.7 the string is refused by the reserved-prefix gate, never by `MsTooLong`; and per the spec's own text, 75 characters is always inside the engraveable cap, so `MsTooLong` is unreachable for this exact 75-char fixture regardless of ms-codec version. The branch is a dead-code invariant restatement, not a live discriminator for this vector today. **Minor** — not a false-PASS on anything reachable, but the test's comment claim ("the refusal must be about the KIND, not the length") is never actually exercised by this fixture. |
| **`IsPreimage` reimplemented as `id, _, _ := s.Split(); return id == "hash"`** (keyed on the 4-char tag, not the prefix byte) | **SURVIVED every existing test** — `codex32`, `sysw`, `seal`, `gui` all report `ok` | See §5 below — this is the headline finding. |

## 4. False-PASS hunting

**(a) `h.mustReach("hashlock preimage")`.** Read `sessionHarness.mustReach` →
`pump` (`gui/unlock_session_test.go:153-174`) → `uiContains`
(`gui/gui_test.go:645`). `pump` calls `h.frame()` up to 256 times and returns
`true` on the **first** frame whose extracted text contains the search string
anywhere, case-insensitive, both sides space-stripped. Text extraction
(`gui/op/op.go:617-626`, `d.Draw` at `:427`) collects one rune per **drawn
glyph mask**; a space glyph paints nothing, so extracted frame text never
contains literal spaces at all — which is why `uiContains` strips spaces only
from the search term (confirmed empirically: the mutation-removed guard's
"last frame" text above renders as
`Insertablankplateandclosethelock...EngravePlate`, no spaces). So yes,
structurally, `mustReach` would match the phrase on **any** reachable frame,
not just a specific screen, and does not require the Engrave screen to be
absent. In this codebase today it is not a live false-PASS: `grep -rn
"preimage" gui/*.go` (excluding this new line) turns up only differently-worded
text (`"the preimage of its hash"`, `"the preimage of a hash"` in
`composer_copy.go`) that does not contain the concatenated substring
`hashlockpreimage`, and those composer screens are unreachable from the narrow
`unlockEngraveCodex32` driver this test uses anyway. **Not a defect, but a
structural risk worth naming**: nothing pins that this is the *first* frame,
or that the Engrave screen is *absent* from the pumped set.

**(b) `TestAdmitSectionRefusesAPreimagePlateAsUnknown`'s
`strings.Contains(err.Error(), "unknown")`.** `Classification.String()`
(`seal/record.go:112-129`) has exactly seven cases; the six named ones render
"debug command", "BIP-39 mnemonic", "output descriptor", "codex32 secret",
"md1/mk1 card", "bitcoin address" — none contains "unknown" — and only the
`default` arm (which `ClassUnknown` falls into) renders "unknown format". The
three `AdmitSection` failure modes (`ErrNotLowercase`, the classify-permitted
branch, `ErrCodex32TooLong`) are independent sentinel errors
(`seal/record.go:31,32,43`), never wrapped in each other. Given
`errors.Is(err, ErrRecordNotPermitted)` is checked first, and combined with the
`String()` table above, the two assertions together are equivalent to
asserting `c == ClassUnknown` specifically — **not a false-PASS today**. (The
one latent risk: a future 8th `Classification` value added without a
`String()` case would also render "unknown format" and satisfy this check
without being `ClassUnknown` by identity — not applicable to this diff, worth
a note for whoever adds the next class.)

**(c) The seam test's three-shape rule.** Recomputed directly from the file,
independent of the test's own internal counter:

```
bip93-secret-128                  host=False device=True
bip93-secret-256                  host=False device=True
bip93-share                       host=False device=True
constellation-entr-128            host=True  device=True
constellation-entr-256            host=True  device=True
entr-id-but-off-profile-length-90 host=False device=True
past-the-engraveable-cap-91       host=False device=False
bip93-bad-checksum                host=False device=False
preimage-plate-0x03               host=False device=False
9 rows: both=2, device_only=4, neither=3
```

Matches the plan's claim exactly.

**(d) `assert_eq!(PREIMAGE_PLATE.len(), 75)` — bytes or chars.** The literal
is pure ASCII (bech32 charset, lowercase letters and digits only; confirmed
`s.isascii() == True`, byte length == char count == 75 both ways). It does not
matter for this fixture — `.len()` (bytes) and a chars count agree — but note
it is inconsistent with `codex32_seam.rs`'s own `s.chars().count()` used
elsewhere in the same file for exactly this reason (defensive against non-ASCII
rows); harmless here only because the charset structurally excludes non-ASCII.

## 5. Corpus sufficiency for the Go port — the headline finding

**One `0x03` row is not enough, and a mutation confirms it does not just
theoretically fail to distinguish — it actually does not.**

`IsPreimage` reimplemented as:

```go
func IsPreimage(s String) bool {
	id, _, _ := s.Split()
	return id == "hash"
}
```

passes every existing test: `go test ./codex32/ ./sysw/ ./seal/` all `ok`,
and `go test -run 'TestUnlock' ./gui/` `ok`. The reason: every row in the seam
corpus, and every ad-hoc fixture in `mspayload_test.go`/`record_test.go`, has
the id field and the prefix byte perfectly correlated — the one `0x03` row has
id `hash`, and no row has id `hash` with a different prefix or prefix `0x03`
with a different id.

This is not academic. Constructed two counterexamples with
`codex32.NewSeed(hrp, threshold, id, shareIdx, data)`:

- `NewSeed("ms", 0, "hash", 's', {0x00, ...15 zero bytes})` → a real
  **entr-shaped** (seed) payload mistagged with id `hash`. Correct
  (prefix-based) `IsPreimage` = `false` (right: `Seed()[0]=0x00`); id-keyed
  mutant = `true` (wrong direction, but the safe one — it would wrongly refuse
  a legitimate seed).
- `NewSeed("ms", 0, "entr", 's', {0x03, ...32 zero bytes})` → a real
  **preimage-shaped** payload (`Seed()[0]=0x03`, 33 bytes) mistagged with id
  `entr`. Correct `IsPreimage` = `true`; **id-keyed mutant = `false`** — this
  is the dangerous direction: under the id-keyed implementation, a genuine
  hashlock preimage plate carrying a corrupted/mismatched `entr` tag would be
  classified `ClassCodex32Secret` and **engraved as a seed**, silently, with
  zero test failure anywhere in the four packages.

Both strings are accepted by `codex32.New`/`NewSeed` without error — the Go
`codex32` package is a generic BIP-93 parser and does not itself enforce the
id/prefix pairing SPEC_ms_hashlock §1 rule 2 names (`TagKindMismatch`, which is
a Rust/ms-codec decode-time rule, not something this fork's generic parser
knows about). So this mismatch shape is real and constructible here, not
merely hypothetical.

**What row would catch it:** a seam-corpus row (or a dedicated
`mspayload_test.go` case) built the same way — `NewSeed` with a **prefix/id
mismatch**, either id `hash` over a non-`0x03` payload or id anything-else over
a `0x03` payload — with `device_admits`/`host_admits` set to whatever the
*prefix* implies (since prefix is supposed to be authoritative), not what the
id implies. That row is the one that falsifies "keyed on the id" while leaving
"keyed on the prefix" (the actual, correct implementation) green.

## Closing counts

- **Critical: 0.** No test in the plan produces a false PASS on the defect it
  names; every declared mutation (4/4) fails exactly as claimed and reverts
  clean.
- **Important: 1.** The corpus (and every unit test across `codex32`, `sysw`,
  `seal`, `gui`) cannot distinguish "keyed on the prefix byte" from "keyed on
  the 4-character id `hash`" — an id-keyed reimplementation of `IsPreimage`
  passes everything and is demonstrably wrong on a constructible
  prefix/id-mismatch input, in the dangerous direction (a mistagged real
  preimage would engrave as a seed). A mismatch-shaped row closes this.
- **Minor: 3.** (i) Two stale line citations in Task 2 (`sysw/codex32_seam_test.go`
  line 11 → actually 30; `seal/record.go:214-216` → actually 212-214);
  harmless, content-match still applies cleanly. (ii) The plan's in-code
  comment on the `d[0] != msPrefixEntr` mutation ("the entr and mnem seams
  below catch it") overstates what's actually tested — no mnem fixture exists,
  and the real catch is a different package's pre-existing rows. (iii) The
  Rust pin test's `MsTooLong` discrimination arm is unreachable/dead for this
  fixed 75-char vector both now and after the 0.8 bump, per the spec's own
  engraveable-cap claim — removing it changes nothing.
- **Nit: 0** beyond what's folded into the Minors above.

All temporary mutations were reverted; both scratch copies confirmed green in
their final (Task 1 + Task 2 applied, nothing else) state before this report
was written.
