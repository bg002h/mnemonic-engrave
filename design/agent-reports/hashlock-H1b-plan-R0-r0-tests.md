# Hashlock H1b plan R0 round 0 — tests/mutation review

**Reviewer:** independent, sonnet tier. **Plan:** `design/IMPLEMENTATION_PLAN_hashlock_H1b_me_bump.md` at mnemonic-engrave `e672194`. **Worktree:** own, detached at `e672194`, `/scratch/code/shibboleth/me-worktrees/h1b-tests`, `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h1b-tests-target` — removed at the end of this review; `git diff --stat` against `e672194` was empty before removal. Every command below ran with `PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h1b-tests-target -p mnemonic-engrave`.

**One question:** can every test the plan adds or relies on actually FAIL on the defect it names, does the plan's RED/GREEN/mutation story hold when run from its text, and which mutations of the new arms survive every test?

## 1. RED, verbatim (Task 1 only: bump + lock, no source edits)

`cargo update -p ms-codec` → `ms-codec v0.7.0 -> v0.8.0`. `cargo build --locked -p mnemonic-engrave` builds clean (no exhaustive-match deviation needed). `cargo nextest run --locked -p mnemonic-engrave -E 'test(/preimage/) | test(/the_host_never_admits/)'` → **5 failed, 0 passed**, matching the plan's count:

```
thread 'a_preimage_plate_is_not_a_seed_record' panicked at crates/me-cli/tests/preimage_plate_is_not_a_seed.rs:29:21:
validate_record admitted a 0x03 preimage plate as Ms

thread 'the_host_never_admits_what_the_device_would_refuse' panicked at crates/me-cli/tests/codex32_seam.rs:58:9:
assertion `left == right` failed: preimage-plate-0x03: host verdict
  left: true
 right: false

thread 'sysw::tests::a_preimage_plate_is_named_not_misdiagnosed' panicked at crates/me-cli/src/sysw/mod.rs:859:9:
assertion `left == right` failed
  left: Ok([...regular pack success bytes...])
 right: Err(Unclassifiable(0, PreimagePlate))

thread 'sysw_pack_names_a_preimage_plate_and_never_echoes_it' panicked at crates/me-cli/tests/preimage_plate_is_not_a_seed.rs:61:5:
sysw pack accepted a preimage plate: sealing:  SEALED — this payload holds secret material (record 0 (codex32 secret)), so it is encrypted ...

thread 'seal_names_a_preimage_plate_and_never_echoes_it' panicked at crates/me-cli/tests/preimage_plate_is_not_a_seed.rs:104:5:
me seal accepted a preimage plate: me: wrote 512 bytes to .../p.uf2 ...
```

**Does Step 3's text match?** Partly. The panic for `a_preimage_plate_is_not_a_seed_record` and the seam test's `host verdict` failure match verbatim. **The other two do not match the given mechanism.** The plan says these fail because `preimage_plate` no longer returns true so "`me sysw pack` falls to the profile arm and `me seal` admits the plate" — implying `Bip93OutsideTheProfile`. What actually happens: `sysw::classify` calls `validate_record` directly (`Ok(RecordKind::Ms) => Class::Codex32Secret`, `sysw/mod.rs:291`), and at the bare bump `validate_record`'s `.map(|_| RecordKind::Ms)` already succeeds for the plate — so the record is **admitted outright as a valid codex32 secret**, never reaching `unknown_reason`/the profile arm at all. `sysw_pack_names_a_preimage_plate_and_never_echoes_it`'s left-hand panic is a *successful pack* (`sealing: SEALED ...`), not a misdiagnosis message — confirming this. Minor: the plan's causal narrative for 2 of 5 RED tests is wrong, though the failure identities and count are correct.

## 2. Declared mutations (Task 2 Step 4 a/b; Task 3 Step 2)

| # | Mutation | Claim | Observed | Match |
|---|---|---|---|---|
| a | Delete `Ok((_, Payload::Preimage(_)))` arm (→ `Ok(_) => Ok(RecordKind::Ms)`) | all 5 preimage/seam tests + witness's 2nd assertion FAIL | `6 failed, 0 passed` (`the_codec_decodes_...` failed on `matches!(validate_record(...), Err(PreimagePlate))`; `a_preimage_plate_is_not_a_seed_record` failed `"admitted a 0x03 preimage plate as Ms"`; all others failed identically) | **Exact match** |
| b | Drop `Ok((_, Payload::Preimage(_)))` alternative from `preimage_plate` | `sysw_pack_names_...` FAILS `stderr does not name the kind`; `a_preimage_plate_is_named_not_misdiagnosed` FAILS ("the profile arm claims the plate again"); `seal_names_...` stays green | `sysw_pack_names_...` failed exactly as claimed (`stderr does not name the kind:` + raw "not a form this container can place" text); `seal_names_...` stayed green as claimed. **`a_preimage_plate_is_named_not_misdiagnosed` did NOT fail the way claimed**: `left: Err(Unclassifiable(0, Unrecognised))`, not `Bip93OutsideTheProfile(75)` — `bip93_outside_the_profile` also requires `ms_codec::decode(s).is_err()`, which is false for this plate at 0.8 (it decodes fine), so the profile arm can't claim it either; the record falls through both special arms to `Unrecognised`. | **Important** — mutation behaves differently from the plan's claim (still FAILS as asserted, but for the wrong reason; anyone reasoning from the plan's text about *why* the guard holds would be wrong) |
| Task 3 | Remove the `TagKindMismatch` arm | new test FAILS `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`; preimage test stays green | Reproduced exactly: `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`, `right: Err(Unclassifiable(0, TagKindMismatch))`; `a_preimage_plate_is_named_not_misdiagnosed` stayed green | **Exact match** |

Whole-crate GREEN after Task 2+3, measured: `619 tests run: 616 passed, 3 failed, 2 skipped` — all 3 failures are `history_purge` (box-local), exactly as the plan states. `cargo fmt -p mnemonic-engrave -- --check` **does NOT pass** on Task 3's literal text: the `const MISMATCH: &str = "...";` line the plan tells the implementer to append is 109 columns (repo has no `rustfmt.toml`, default `max_width = 100`), so rustfmt wants it wrapped — deterministic, not a toolchain artifact. Task 3 carries no fmt/clippy step of its own (only Task 2 Step 5 runs `cargo fmt --check`, and that runs *before* Task 3's text exists), so nothing in the plan's own steps catches this before commit. **Important** — contradicts the plan's own "measured after Task 3 as well: ... fmt clean" claim. `cargo clippy -- -D warnings` fails on one pre-existing, plan-unrelated lint (`manual_is_multiple_of` at `sysw/composer_records.rs:114`) exactly as the plan documents ("local nightly's, green in CI") — reproduced, not a new finding.

## 3. My own mutations

| Mutation | Caught by / SURVIVED |
|---|---|
| (i) `Ok((_, Payload::Preimage(_))) => Ok(RecordKind::Ms)` (arm present, wrong kind) | Caught — 6 of 7 hashlock tests fail (`a_preimage_plate_is_not_a_seed_record`, seam, sysw unit test, both binary tests, witness); only `an_id_kind_mismatch_...` (unaffected path) stays green |
| (ii) `preimage_plate` returns `true` for every `ms`-HRP `Format::Ms` string | Caught — 4 failures beyond `history_purge`: `does_not_fire_at_the_ninety_character_boundary`, `a_valid_bip93_codex32_names_the_profile_not_the_classifier` (`left: Err(Unclassifiable(0, PreimagePlate))`, `right: Err(Unclassifiable(0, Bip93OutsideTheProfile(48)))`), `the_profile_arm_is_gated_on_a_real_bip93_parse`, `a_valid_bip93_string_is_told_it_is_bip93_and_not_a_constellation_ms1` |
| (iii) `TagKindMismatch` arm moved AFTER the profile arm | Caught, by the arm's own dedicated test: `an_id_kind_mismatch_is_named_not_misdiagnosed` fails `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`, `right: Err(Unclassifiable(0, TagKindMismatch))` — the profile arm (`bip93_outside_the_profile`, which only checks `decode(s).is_err()`) claims the mismatch string first once order is reversed |
| (iv) `TagKindMismatch` arm matches `Err(_)` (any codec error) | Caught, but **not** by `an_id_kind_mismatch_is_named_not_misdiagnosed` itself (still passes — a mismatch string is still an `Err` either way). Caught by 6 *other* pre-existing tests: `a_valid_bip93_codex32_names_the_profile_not_the_classifier`, `an_unclassifiable_record_is_refused_with_its_index` (`left: Err(Unclassifiable(1, TagKindMismatch))`, `right: Err(Unclassifiable(1, Unrecognised))`), `the_profile_arm_is_gated_on_a_real_bip93_parse`, `a_record_of_no_class_at_all_still_names_the_classifier`, `an_unpackable_record_is_refused_before_a_passphrase_is_minted`, `a_valid_bip93_string_is_told_it_is_bip93_and_not_a_constellation_ms1`. Answers the brief's question directly: nothing in the *hashlock* test set distinguishes a mismatch from a bad checksum — the pre-existing BIP-93/profile tests do. |
| (v) `validate_record` maps `Payload::Preimage` to `RecordError::Invalid(..)` instead of `PreimagePlate` | **Mixed.** Caught by `a_preimage_plate_is_not_a_seed_record` (`refused as Invalid("preimage kind 0x03"), not as a preimage plate`), `seal_names_a_preimage_plate_and_never_echoes_it` (`stderr does not name the kind: ... me: invalid record: preimage kind 0x03`), and the witness's 2nd assertion. **Survives** in `sysw_pack_names_a_preimage_plate_and_never_echoes_it` (stays PASS) — `me sysw pack`'s diagnosis runs through `unknown_reason`/`preimage_plate()` independently of `validate_record`'s `RecordError` variant, so it never observes this mutation. Not a defect (the two host verbs are architecturally independent here and the `me seal` path does catch it), but worth recording: the binary-level guard for `sysw pack` does not exercise `validate_record`'s error variant at all. |

## 4. False-PASS hunting

**(a) Witness's second assertion — could it pass for a reason other than the new arm?** No. `MAX_ENGRAVEABLE_MS1_LEN = 90` (`record.rs:29`); `PREIMAGE_PLATE` is 75 chars, so the length gate (`len > 90`) never fires, and even if it did it could only produce `RecordError::MsTooLong`, which would fail the assertion, not pass it. Confirmed empirically: mutation (a) above deletes the arm and the witness's second assertion fails exactly as the plan predicts. No false-PASS.

**(b) Seam test at 0.8 — which rows changed verdict, and are all three shapes present?** Instrumented the seam test (temporarily, reverted) to print every row's computed-vs-expected verdict at the bare bump (Task 1 only): **exactly one row changes** — `MISMATCH preimage-plate-0x03: expected host_admits=false, computed=true`. No other row (including `preimage-shape-entr-id` and `entr-id-but-off-profile-length-90`) flips at any point; after Task 2's fix the row returns to `false`, matching the pinned file. Post-fix counts (both Task 1+2+3 applied): `both=2 device_only=6 neither=5 total=13` — all three shapes present, invariant holds.

**(c) `a_preimage_plate_is_named_not_misdiagnosed`'s control is an entr-32; is there a 0x03 input for which `unknown_reason` still says "outside the profile"?** **Yes — confirmed, reproducible, and this is the headline finding.** A malformed hashlock preimage — valid codex32 checksum, id `hash`, prefix byte `0x03`, but a payload of the *wrong* byte length (constructed via `ms_codec::codex32::Codex32String::from_seed`, the same public forging entry point ms-codec's own test suite uses in `tests/hashlock_kind.rs`) decodes to a **third, distinct** ms-codec 0.8 error, `Error::PreimageLengthMismatch`, which neither `preimage_plate()`'s match arms (`Ok(Preimage)` / `Err(ReservedPrefixViolation{got:3})`) nor Task 3's new `TagKindMismatch` arm cover:

```
s = "ms10hashsqw46h2at4w46h2at4w46h2at4w4ssrnvvaudn2k4d" (50 chars, id "hash", prefix 0x03, 16 payload bytes)
ms_codec::decode(s)                 = Err(PreimageLengthMismatch { got: 16 })
preimage_plate(s)                   = false
bip93_outside_the_profile(s)        = true
validate_record(s)                  = Err(Invalid("preimage payload is 16 bytes after the prefix; ..."))
`me sysw pack` stderr               = "record 0 ... is a VALID BIP-93 codex32 string — the checksum is good —
                                        but not a constellation `ms1` record ... re-encode the entropy as `ms1`
                                        rather than editing the string."
```

So a damaged or forged hashlock preimage plate of this shape is told to "re-encode the entropy as `ms1`" — the exact misdiagnosis §1/H0/H1b exist to prevent — and on `me seal` it surfaces the raw ms-codec internal string via `RecordError::Invalid`, not a hashlock-aware message either. This directly contradicts the plan's own doc-comment claim (Task 2 Step 2's replacement text for `preimage_plate`, echoing the pre-existing H0-era comment): *"a malformed `0x03` string is named the same way"* as a well-formed plate. No RED, GREEN, or mutation step in the plan exercises this shape — it is a **missing case**, not a false-PASS in an existing test (no test claims to cover it). **Important** (borderline Critical: it is a wrong, actively misleading result on a real codec error path, but no existing test asserts the false guarantee, so it fails the letter of this brief's Critical bar — "a test that cannot fail on the defect it names").

## 5. The bump's blast radius

`cargo tree -p mnemonic-engrave -i ms-codec --locked`: `ms-codec v0.8.0 └── mnemonic-engrave v0.8.0`. Lockfile diff is exactly 3 lines changed in the `ms-codec` package block: version/checksum bump, plus `+ "pbkdf2"` and `+ "sha2"` added to its `dependencies` list. **Zero `[[package]]` blocks entered or left the lockfile** — `pbkdf2` and `sha2` were already present (already direct dependencies of `mnemonic-engrave` itself, for its own seal encryption), and `hmac` gains **no** new edge at all. The plan's Step 1 text ("the lockfile also gains `pbkdf2`, `hmac`, `sha2` (and their deps) as ms-codec 0.8.0 requires them") reads as if these are new arrivals; they are not — only two dependency *edges* change, and `hmac` isn't among them. **Minor** (wording).

Whole-crate run at the bare bump (Task 1 only, before Task 2), `--no-fail-fast`: **9 failed** (not 5+3=8). Beyond the 5 named tests and the 3 `history_purge` failures, a **6th, unnamed failure**: `record_corpus::every_corpus_record_classifies_as_it_did_before_s2`, panicking `codex32_seam/preimage-plate-0x03: class moved under S2 — left: "Codex32Secret", right: "Unknown"`. This is the *same* underlying regression (the plate now decodes and is admitted), pinned by a third, independent harness (`testdata/record_corpus_pre_s2.json` captures the same seam-vector strings under a separate invariant). It is not named anywhere in the plan (no File Structure row, no self-review line), though Task 2's fix does incidentally repair it too — confirmed: the post-Task-2+3 whole-crate run has no `record_corpus` failure, back to 619/616/3. **Important**: the plan's RED accounting ("five failures... history_purge... box-local") undercounts the bare bump's actual blast radius by one test; nothing in the plan's own text would have surfaced this specific test by name had Task 2's fix not accidentally also covered it — only the whole-crate Step 5 run (which the plan does prescribe) would catch it in practice.

## Closing counts

- RED reproduced: yes (5 failures, exact panics for 3 of 5; causal narrative wrong for 2 of 5 — Minor).
- Declared mutations: 3 run, 2 exact matches, 1 (b) reproduces the FAIL but not the claimed mechanism — Important.
- Own mutations: 5 run. 4 caught (i, ii, iii, iv — iv not by its "own" test, by neighbors); 1 (v) caught on 2 of 3 relevant tests, survives on the third by design (not a defect).
- False-PASS hunting: (a) none found; (b) exactly one row changes, invariant holds; (c) **confirmed gap** — malformed-length 0x03 strings are misdiagnosed as "outside the profile" / raw `Invalid`, contradicting the plan's own doc claim — Important.
- Blast radius: 0 new crates entered the lockfile (wording issue only — Minor); **6th failure at the bare bump** (`record_corpus`) beyond the plan's named 5 — Important.

**Total: 0 Critical, 4 Important (mutation-b narrative, fmt-check gap in Task 3, item-4c malformed-preimage misdiagnosis, record_corpus blast-radius undercount), 2 Minor (Step-3 narrative for 2/5 RED tests, lockfile "gains" wording).**
