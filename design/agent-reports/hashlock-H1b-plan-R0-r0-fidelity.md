# Hashlock H1b plan — R0 round 0, fidelity + design lens

**Artifact:** `design/IMPLEMENTATION_PLAN_hashlock_H1b_me_bump.md` at engrave `e672194`.
**Baselines read:** engrave `0f5ce23` (anchors), mnemonic-secret `cd0a60f` (= `ms-codec-v0.8.0`,
verified `git diff --stat cd0a60f HEAD -- crates/ms-codec` is empty), ms-codec `0.7.0` at tag
`ms-codec-v0.7.0` (for the before/after comparison in I-1).
**Question asked:** if an implementer follows this plan literally, does `me` at ms-codec 0.8
refuse a kind-`0x03` PREIMAGE plate BY NAME on every path that packs or seals a record — and
does the plan claim nothing false about the code and API it cites?

**Answer in one line:** for a *well-formed* plate, yes on every path — but the plan's re-pointed
`preimage_plate` is strictly NARROWER than the 0.7 predicate it replaces, so three families of
`0x03` string that H0 named "a hashlock PREIMAGE plate … do not re-encode it as entropy" go back
to "outside the profile … re-encode the entropy as `ms1`" (I-1); and the arm's `Ok(_) =>
Ok(RecordKind::Ms)` wildcard makes `me` fail OPEN on the next payload kind ms-codec adds (I-2).

**Counts: 0 Critical / 2 Important / 6 Minor / 2 Nit.**

---

## Item 1 — every path in `me` that reaches `ms_codec::decode`

`grep -rn "ms_codec::" crates/me-cli/src crates/me-cli/tests` at `0f5ce23` returns exactly
**three** `decode` call sites and four non-decode uses. **No call site anywhere in `me` binds the
decoded `Payload`** — every one discards it (`.map(|_| …)`, `.is_err()`, `matches!(…)`), so there
is no route by which preimage BYTES become a record. That is the reason there is no Critical here.

| # | call site (`0f5ce23`) | reached by | 0.8 answer, well-formed plate | after the plan |
| --- | --- | --- | --- | --- |
| 1 | `src/seal/record.rs:193` — `validate_record`'s `Format::Ms` arm | `me seal` (`seal/mod.rs:163` public, `:270` secret), `me hash` (`main.rs:1012`), `sysw::classify` (`sysw/mod.rs:290`), `sysw::record::card_hrp` (`:299`), `mdmk_unconfirmed` (`:208`), `decode_public_set` (`record.rs:284`) | `Ok((hash, Payload::Preimage))` | **guarded** — Task 2 Step 1 returns `Err(RecordError::PreimagePlate)` on the SUCCESS path; `RecordKind::Ms` unreachable for `Preimage` |
| 2 | `src/seal/record.rs:233` — `bip93_outside_the_profile`, `ms_codec::decode(s).is_err()` | `unknown_reason` (`sysw/mod.rs:201`) | flips `true`→**`false`** (the plate now decodes) | **not modified.** Harmless for the well-formed plate (the `preimage_plate` arm runs first) but load-bearing for I-1 |
| 3 | `src/seal/record.rs:259` — `preimage_plate` | `validate_record`'s error path; `unknown_reason` (`sysw/mod.rs:198`) | `ReservedPrefixViolation` no longer returned | **re-pointed** — Task 2 Step 2 |
| 4 | *(new)* `unknown_reason`, before the `preimage_plate` arm | `me sysw pack`/`show` | `Err(TagKindMismatch{..})` for `preimage-shape-entr-id` | **added** — Task 3 |
| — | `record.rs:187`, `:232`, `:549` — `codex32::Codex32String::from_string` | as above | parse-only, no payload | unaffected |
| — | `main.rs:2815-2816` — `consts::VALID_STR_LENGTHS` / `VALID_MNEM_STR_LENGTHS` | the `Bip93OutsideTheProfile` message | constants only | unaffected |

Verbs that never reach a decode at all: `me` (convert) refuses `Format::Ms` at `lib.rs:78` and
`me bundle` at `bundle.rs:112` + the pre-scan at `:208`, both on the HRP, before `validate`. A
preimage plate can therefore never reach NDEF or the preview sidecar. `validate.rs:116` keeps its
`unreachable!("ms1 is refused before validation")` honestly.

Traced verb by verb, well-formed plate, plan applied:

- `me sysw pack` → `classify` → `validate_record` → `Err(PreimagePlate)` → not `host_admits` →
  `Class::Unknown` → `SyswError::Unclassifiable(i, UnknownReason::PreimagePlate)` → `main.rs:2795`
  message naming the kind. ✅
- `me seal` public section → `check_public` `Err(e) => SealError::Record(e)` → `RecordError::PreimagePlate`
  Display (`record.rs:120`). ✅ Secret section → `record_or_mnemonic` → same. ✅
- `me hash` → `main.rs:1013` `eprintln!("me: record {i}: {e}")` → the same named refusal, on a THIRD
  verb the plan does not mention (no change needed; recording it because the plan's Global
  Constraints say "BOTH host verbs" and there are three). ✅
- `sysw::record::card_hrp` / `mdmk_unconfirmed` / `decode_public_set` → `validate_record` errs →
  `None` / fail-closed / propagated. ✅
- seam corpus, `sysw::classify` verdicts at 0.8 with the plan: `preimage-plate-0x03` → `Unknown`;
  `preimage-shape-entr-id` (`Err(TagKindMismatch)`) → `Unknown`; `bip93-plain-33-byte-payload-0x03`
  (`Err(UnknownTag{got:b"test"})`) → `Unknown`; `bip93-plain-payload-0x03` (48 chars,
  `Err(UnexpectedStringLength)`) → `Unknown`; `bip93-share-payload-0x03` (`Err(IsShareNotSingleString)`)
  → `Unknown`. **`host_admits: false` holds for all five**, so `the_host_never_admits_what_the_device_would_refuse`
  stays green — and that is exactly why it cannot see I-1.

---

## Item 2 — `#[non_exhaustive]`, and anything relying on 0.7's refusal

- No `match` on a codec type exists in `me` outside `validate_record`: `grep -rn
  "InspectKind|ms_codec::Payload|ms_codec::PayloadKind|ms_codec::inspect" crates/me-cli/src
  crates/me-cli/tests` returns **nothing**. The plan's Global Constraint ("`me` never matches on
  `InspectKind`") is TRUE, and spec §10's source-breaking warning does not bind `me`.
- The only code that relied on 0.7 refusing `0x03` is `preimage_plate` itself, which the plan
  re-points. No test or doc asserts `ReservedPrefixViolation` by name anywhere in `crates/`
  (`grep -rn "ReservedPrefixViolation|reserved-prefix byte"` hits only `design/` and one test
  NEGATIVE assertion, `seal_names_a_preimage_plate_and_never_echoes_it`'s
  `!err.contains("reserved-prefix byte")`, which stays satisfied).
- `testdata/record_corpus_pre_s2.json` carries all five `0x03` rows expecting class `Unknown`;
  every one still resolves to `Unknown` at 0.8 (above), so `record_corpus.rs` is unaffected.

---

## Findings

### I-1 — the re-pointed `preimage_plate` is strictly NARROWER than the 0.7 predicate, and three families of `0x03` string regress from "hashlock PREIMAGE plate" to "re-encode the entropy as `ms1`"

**Where:** Task 2 Step 2 (`preimage_plate`'s new body and its new doc comment).

**The counterexample is a row already in the repo.** `testdata/codex32_seam_vectors.json`,
`bip93-plain-33-byte-payload-0x03`:

```
ms10testsqvrsu9guyv4rzwplgex4gkmzd9c8wl593jfe4gdg47mtm3xt6tv7qh3pm4xrfdlvvp
```

75 characters, id `test`, prefix byte `0x03`, 32-byte X — the corpus calls it "THE COLLISION, and
it is deliberate … byte-for-byte indistinguishable from a hashlock PREIMAGE plate".

*At 0.7* (`ms-codec-v0.7.0`, `envelope.rs:175` — `dispatch_payload` runs INSIDE `discriminate`,
before `decode.rs`'s tag accept-set check, so the tag is irrelevant): `decode` →
`Err(ReservedPrefixViolation { got: 3 })` → `preimage_plate` **true** → `me sysw pack` prints
*"is a hashlock PREIMAGE plate (kind 0x03), not a seed record … keep it with the policy it
unlocks, and do not re-encode it as entropy."*

*At 0.8 with the plan applied*: `discriminate` builds `Payload::Preimage`, then `decode.rs:125`'s
`_ =>` arm returns `Err(Error::UnknownTag { got: *b"test" })` — rule 6b's `TagKindMismatch` guard
requires the tag to be in `{entr, hash}`, and `test` is neither. The new predicate matches only
`Ok((_, Payload::Preimage(_))) | Err(ReservedPrefixViolation { got: 0x03 })`, so it answers
**false**. `unknown_reason` then falls through Task 3's arm (not a `TagKindMismatch`) to
`bip93_outside_the_profile`, which is **true** (codex32-valid, `decode` errs), and `main.rs:2805-2814`
prints:

> "`ms1` is a two-gate PROFILE over BIP-93: … **the 4-character id must be `entr`**. This one is
> 75 characters. … re-encode the **entropy** as `ms1` rather than editing the string."

That is the exact sentence H0's CHANGELOG says it removed ("instead of misdiagnosing it as
'outside the profile' and telling the operator to re-encode a spend secret as entropy"), and the
exact sentence `sysw_pack_names_a_preimage_plate_and_never_echoes_it` asserts must NOT appear
(`!err.contains("re-encode the entropy")`). For the collision string the advice is not merely
unhelpful: if the operator holds a preimage and follows it, they re-encode a spend secret as
`entr` — and *that* plate `me` will admit and the device will engrave as a seed. H0 refused this
string on both sides precisely because the two readings cannot be told apart; the 0.7 wording was
safe under both, the 0.8 wording is safe under only one.

**The regression set, enumerated** (0.7 `preimage_plate` was true for ANY `ms1` string past the
wire gates with prefix byte `0x03`; 0.8's is true only for `Ok(Payload::Preimage)`, i.e. tag
`hash` + exactly 32 bytes ⟹ exactly 75 chars):

| `0x03` string | 0.8 `decode` | 0.7 diagnosis | 0.8 diagnosis after the plan |
| --- | --- | --- | --- |
| tag `hash`, 32-byte X (well-formed) | `Ok(Preimage)` | PreimagePlate | PreimagePlate ✅ |
| tag `entr`, 32-byte X (`preimage-shape-entr-id`) | `Err(TagKindMismatch)` | PreimagePlate | TagKindMismatch ✅ (deliberate, L24) |
| **tag `test`/any non-`{entr,hash}`, 32-byte X** | `Err(UnknownTag)` | PreimagePlate | **`Bip93OutsideTheProfile(75)`** ❌ |
| **any tag, X ≠ 32 bytes** (e.g. 77 chars / 33-byte X — `Err(PreimageLengthMismatch{got:33})`, `envelope.rs:222`) | `Err(PreimageLengthMismatch)` | PreimagePlate | **`Bip93OutsideTheProfile(77)`** ❌ |
| **tag `seed`/`xprv`/`prvk`, 32-byte X** | `Err(ReservedTagNotEmittedInV01)` | PreimagePlate | **`Bip93OutsideTheProfile(75)`** ❌ |

Row 4 is not hypothetical drift — it is behaviour H0's post-implementation review **M-1**
deliberately chose, and the shipped doc comment at `record.rs:242-250` states it as the contract:
*"a malformed `0x03` string — a 77-character one carrying a 34-byte payload, say, which §1 calls a
`PreimageLengthMismatch` — is named the same way. That is the intended direction … a predicate
that checked the length would let the malformed one fall through to 'outside the profile'."*
77 < `MAX_ENGRAVEABLE_MS1_LEN` (90), so it reaches `decode` and is not diverted to `MsTooLong`.

**And the plan's own replacement text asserts the opposite of what its own body does.** Task 2
Step 2's new doc comment keeps the sentence *"a well-formed plate is 75 characters with id `hash`;
a malformed `0x03` string is named the same way"* above a body that no longer names it the same
way. That is a false claim about the code the plan is writing, in the same fragment.

**No gate can catch this.** The seam test asserts only `classify(s) == Codex32Secret`, and every
affected string keeps `host_admits: false`; the H0 tripwires all use the well-formed `hash` plate.
The build gate ran green with the defect present.

**SUGGESTION.** Key the predicate on the PREFIX BYTE — the same thing the device's guard tests, so
host and device stay convergent in what they NAME, not only in what they admit — and let the L24
mismatch be the one deliberate exception:

```rust
pub fn preimage_plate(s: &str) -> bool {
    let s = s.trim();
    if !matches!(classify(s), Ok(Format::Ms)) {
        return false;
    }
    // An id/kind MISMATCH is diagnosed separately (L24), never as a plate.
    if matches!(ms_codec::decode(s), Err(ms_codec::Error::TagKindMismatch { .. })) {
        return false;
    }
    // The KIND is the prefix byte (SPEC_ms_hashlock §1), not the id and not the
    // length -- so a malformed or mistagged 0x03 string is named the same way.
    match ms_codec::codex32::Codex32String::from_string(s.to_string()) {
        Ok(c) => c.parts().data().first() == Some(&0x03),
        Err(_) => false,
    }
}
```

(`Codex32String::parts` is `pub` at `ms crates/ms-codec/src/codex32/mod.rs:259` and `Parts::data`
at `:446`; bind `c` before calling `parts()`, which borrows. `me` already calls `from_string` at
`record.rs:187` and `:232`, so the pattern is established.) A minimal alternative is to add
`| Err(ms_codec::Error::PreimageLengthMismatch { .. })` to the `matches!`, which closes row 4 only
— rows 3 and 5 are unreachable from the error value, since neither `UnknownTag` nor
`ReservedTagNotEmittedInV01` carries the prefix byte.

Whichever spelling is chosen, add the two regressing corpus rows to
`a_preimage_plate_is_named_not_misdiagnosed` as extra assertions, so the next codec bump cannot
silently re-narrow the predicate.

---

### I-2 — `Ok(_) => Ok(RecordKind::Ms)` makes `me` fail OPEN on the next payload kind ms-codec adds, and `#[non_exhaustive]` guarantees the compiler will not say so

**Where:** Task 2 Step 1's arm; the Architecture line *"every other payload → `RecordKind::Ms`"*;
the Global Constraint *"Source-breaking surface, **checked by the compiler**"*.

`Payload` is `#[non_exhaustive]` (ms `payload.rs:44`), so a downstream `match` *cannot* be
exhaustive — a wildcard is mandatory. What is optional is what the wildcard MEANS, and the plan
makes it mean **"this is a seed record, place it"**.

**Counterexample.** ms-codec adds `Payload::Xprv(..)` in 0.9 (not speculative: `consts.rs:71`
`RESERVED_NOT_EMITTED_V01 = [seed, xprv, prvk]` are reserved *pending* kinds, and `Mnem` arrived
this way in v0.2). `me` on a `"0.8"` caret pin picks it up on the next `cargo update`. `decode`
returns `Ok((tag, Payload::Xprv(..)))` → `Ok(_)` → `RecordKind::Ms` → `RecordKind::is_secret()`
true (`record.rs:43`) → `Class::Codex32Secret` → `me sysw pack` seals it and the device engraves
it as a seed plate. **No test in `me` goes red**: every existing tripwire and every corpus row is
about `0x03`. This is F-473's exact defect one kind later, with the difference that F-473 *had* a
tripwire written for it in advance and this would not.

The Global Constraint's phrase "checked by the compiler" is true of *this* bump and false of the
next one — and it is the arm this plan introduces that disarms the compiler.

**SUGGESTION.** Spell the seed kinds positively and let the mandatory wildcard refuse:

```rust
match ms_codec::decode(s) {
    Ok((_, ms_codec::Payload::Preimage(_))) => Err(RecordError::PreimagePlate),
    Ok((_, ms_codec::Payload::Entr(_) | ms_codec::Payload::Mnem { .. })) => Ok(RecordKind::Ms),
    // `Payload` is #[non_exhaustive]: a kind added by a future ms-codec minor
    // arrives HERE, silently, and must not be placed as a seed until `me` has
    // decided what it is. Refuse; the compiler cannot warn.
    Ok(_) => Err(RecordError::Invalid(
        "an `ms1` kind this version of `me` does not place; upgrade `me`".into(),
    )),
    Err(e) => { /* unchanged */ }
}
```

Cost is bounded and in the safe direction: at a future minor `me` refuses a new *seed* kind until
it opts in, which is a refusal an operator can act on, versus engraving material nobody has
classified. If the plan instead keeps the fail-open arm, say so as a stated decision with its
reason, rather than leaving it as a formatting consequence of `#[non_exhaustive]`.

---

### M-1 — Task 1 Step 1's "Expected" is false: `pbkdf2`, `hmac` and `sha2` are ALREADY in `Cargo.lock`

**Where:** Task 1 Step 1, *"the lockfile also gains `pbkdf2`, `hmac`, `sha2` (and their deps) as
ms-codec 0.8.0 requires them"*; repeated in the File Structure table's `Cargo.lock` row ("its new
transitive deps (pbkdf2, hmac, sha2 …)").

Measured: `crates/me-cli/Cargo.toml:45-46` already declares `pbkdf2 = { version = "0.12",
default-features = false, features = ["hmac"] }` and `sha2 = "0.10"` as DIRECT dependencies of
`mnemonic-engrave` (`me`'s own KDF), and `Cargo.lock` already carries `hmac 0.12.1` (line 505),
`pbkdf2 0.12.2` (672) and `sha2 0.10.9` (972), with all three listed under
`[[package]] name = "mnemonic-engrave"`. `cargo update -p ms-codec` will add dependency EDGES to
the existing stanzas, not new packages. An implementer checking the stated expectation will find
no new package and has no way to tell that from a failed update.

**SUGGESTION:** replace the Expected with what is actually checkable — `grep -A1 'name =
"ms-codec"' Cargo.lock` shows `version = "0.8.0"`, and `ms-codec`'s own `dependencies` list in the
lock gains `getrandom`, `pbkdf2`, `sha2`, `zeroize`.

---

### M-2 — Task 4's CHANGELOG step strands two entries and leaves a sentence that H1b itself falsifies; F-454's owning phase is met and not reconciled

**Where:** Task 4 bullet 1 (`## [0.8.1] — unreleased`, *"'H0 shipped in 0.8.x unreleased' folded
in as the record shows"*).

`crates/me-cli/CHANGELOG.md` currently has one `## [Unreleased]` section holding **two** entries:
the H0 `### Added` block and a `### Changed` block for the `+`-signed `key:` path tightening.
Creating a second unreleased heading beside it leaves the crate at `version = "0.8.1"` while the
release record for 0.8.1 omits both — and the `+`-sign entry is the change **F-454** exists for
(`design/FOLLOWUPS.md:15419`, owning phase *"cut me 0.8.1 with the next host change or before
S4"*). This plan IS the next host change, so F-454 is due here, and Task 4 mentions only F-473.

The sentence to point at, in the H0 `### Added` entry:

> "At the pinned ms-codec `0.7` the codec's prefix gate already refuses the string;
> `tests/preimage_plate_is_not_a_seed.rs` is the tripwire that goes red at the `0.8` bump if the
> refusing arm is forgotten (follow-up F-473)."

Both halves describe a pin that no longer exists by the time this section is released, in the same
unreleased window.

**SUGGESTION:** rename the existing `## [Unreleased]` to `## [0.8.1] - <date>` (or keep
`[Unreleased]` and bump only at release time), fold the H1b items into it, rewrite that sentence
in the past tense ("H0 shipped against ms-codec 0.7, where the codec's prefix gate refused the
string; H1b moved the refusal onto the codec's success path"), and close/advance F-454 in the same
step. Also `RELEASE`-side: nothing else pins the version — `.github/workflows/release.yml:193` and
`:349` both read it out of `crates/me-cli/Cargo.toml`, and the `me-preview` lockstep check at
`:222-228` compares against that same value, so the bump is self-consistent.

---

### M-3 — `me seal` and `me hash` get the codec's raw Display for an id/kind mismatch; only `me sysw pack` learns the words (answers the brief's question)

**Where:** Task 3 (variant + `unknown_reason` arm + `main.rs` Display) — all three are on the
`me sysw pack` side.

At 0.8, `preimage-shape-entr-id` through `me seal` yields `RecordError::Invalid(e.to_string())`
(`record.rs:107`, `"invalid record: {e}"`) over ms-codec's Display (`error.rs:220-224`):

> `me: invalid record: tag "entr" does not name the kind the prefix byte 0x03 carries; refusing
> rather than reading one kind as another`

**Is that acceptable?** Yes, on the merits: SPEC_ms_hashlock §1 rule 2 requires *refusal* and
names no message, the text states the mechanism accurately, and it echoes only the 4-character id.
But it is the same asymmetry H0's post-implementation review **I-2** judged a defect for the
sibling case ("the named diagnosis must exist on both verbs, not just `me sysw pack`"), and it
carries no record index — the second half of what I-2 complained about — so in a multi-record seal
the operator still cannot tell which record. Recorded as Minor rather than Important because the
refusal itself is correct and the wording is not misleading (contrast I-1, where it is).

**SUGGESTION:** either add `RecordError::TagKindMismatch` alongside `PreimagePlate` and match it in
`validate_record`'s `Err` arm (three lines, and it makes the two verbs symmetric as the plan's own
Global Constraint pattern promises), or state in the plan that `me seal` deliberately keeps the
codec's words for this case and why.

---

### M-4 — the new witness prints `mnemonic-engrave`'s version under the label "ms-codec"

**Where:** Task 2 Step 3, `"ms-codec {} did not decode the plate as Payload::Preimage: {decoded:?}", env!("CARGO_PKG_VERSION")`.

In an integration test, `CARGO_PKG_VERSION` expands to the package being compiled —
`mnemonic-engrave`, i.e. `0.8.0` today and `0.8.1` after Task 4 — never the dependency's. The
message fires exactly when someone is diagnosing a codec-pin problem and will print a version that
is not the codec's.

**SUGGESTION:** drop the version from the message (`"the plate did not decode as
Payload::Preimage: {decoded:?}"`), or name the crate whose version it is.

---

### M-5 — three shipped comments and one operator message become false at 0.8, and the plan touches none of them

**Where:** not in the plan — that is the finding (the "a diff falsifies text it never touches" lens).

1. `sysw/mod.rs:848` — `a_preimage_plate_is_named_not_misdiagnosed`'s recorded mutation:
   *"MUTATION: swap the two arms in `unknown_reason` -> `Bip93OutsideTheProfile(75)`."* At 0.8 the
   plate DECODES, so `bip93_outside_the_profile` is **false** for it and the swap yields
   `Unrecognised`, not `Bip93OutsideTheProfile(75)`. The test still fails under the mutation (no
   false PASS), but the recorded result is wrong.
2. `record.rs:214-215` — `bip93_outside_the_profile`'s doc: *"the 4-character id must be `entr`"*.
   At 0.8 an id of `hash` also decodes, so the profile has two admitted ids.
3. `sysw/mod.rs` — the `Bip93OutsideTheProfile` variant doc repeats *"then the 4-character id
   `entr`"*.
4. `main.rs:2810` — the operator-visible text *"the 4-character id must be `entr`"*, same problem,
   and it is the message I-1 routes three families of `0x03` string into.

**SUGGESTION:** fix (1) and (4) in Task 2 (they are one line each and (4) is operator-facing);
(2) and (3) can ride along.

---

### M-6 — the seam corpus's own prose about `bip93-plain-33-byte-payload-0x03` at 0.8 is wrong, and correcting it costs a two-repo re-pin

**Where:** `testdata/codex32_seam_vectors.json`, that row's `source`:

> "`me` refuses this exact string too (ms-codec 0.7 at the prefix gate, **0.8 as a
> TagKindMismatch**), measured."

At 0.8 it is `Err(UnknownTag { got: b"test" })`, not `TagKindMismatch`: `decode.rs`'s rule-6b guard
is `x if (x == TAG_ENTR || x == TAG_HASH) && tag != payload.kind().single_tag()`, and `test` is in
neither set, so control reaches the `_ =>` arm at `decode.rs:125`. The row's *verdict*
(`host_admits: false`) is still correct, so no test fails — but the sentence is the justification
a future reader will lean on, and it is the same row I-1 is about.

**SUGGESTION:** correct it in the same fold as I-1, and note the cost explicitly in the plan:
editing this file requires re-pinning `SEAM_VECTORS_SHA256` in **both** `crates/me-cli/tests/codex32_seam.rs:26`
and the fork's `sysw/codex32_seam_test.go`, or one of the two suites goes red. If the plan prefers
not to touch the fork, say so and file the correction with H2 as its owning phase.

---

### N-1 — Task 3's new arm calls `ms_codec::decode` without the HRP gate both neighbouring predicates use

`preimage_plate` and `bip93_outside_the_profile` both open with `matches!(classify(s), Ok(Format::Ms))`
before touching the codec; Task 3's arm calls `ms_codec::decode(record.trim())` on every record
that reached `unknown_reason`. No false positive is possible — `discriminate` returns
`Error::WrongHrp` before any tag work for a non-`ms` HRP — but it parses arbitrary unclassifiable
records (including bodies that fell through the composer arms) into an unscrubbed
`Codex32String`/`String`. Secret-handling never gates (operator ruling 2026-08-27); this is
consistency, and it is one `matches!` to fix.

### N-2 — `## [0.8.1] — unreleased` does not match the file's own heading style

`CHANGELOG.md` uses `## [Unreleased]` and `## [0.8.0] - 2026-09-02` (ASCII hyphen, Keep a
Changelog). Task 4 specifies an em dash and a non-standard "unreleased" suffix.

---

## Items 4 and 5 — records and anchors

**Records** (item 4) are M-2 and M-6, plus the confirmation that `release.yml` needs no edit
(both version reads are `grep -m1 '^version' crates/me-cli/Cargo.toml`, lines 193 and 349, and the
`me-preview` lockstep check at 222-228 compares against that same string). The "me 0.8.1" mentions
in `design/BRAINSTORM_hashlock_phrase.md:58` (L22) and `:225` describe H1b's own delivery and stay
true; `design/CONTINUITY_composer_2026-09-01.md:2005` already anticipates this plan.

**Anchors** (item 5) — every `Modify` anchor exists **exactly once** at `0f5ce23`, machine-checked
with `git show 0f5ce23:<file> | grep -Fc`:

| count | file | anchor |
| --- | --- | --- |
| 1 | `crates/me-cli/Cargo.toml` (line 53) | `ms-codec = "0.7"` |
| 1 | `crates/me-cli/Cargo.toml` (line 3) | `version = "0.8.0"` |
| 1 | `src/seal/record.rs` | `ms_codec::decode(s).map(\|_\| RecordKind::Ms).map_err(\|e\| {` |
| 1 | `src/seal/record.rs` | `pub fn preimage_plate` |
| 1 | `src/seal/record.rs` | `Err(ms_codec::Error::ReservedPrefixViolation { got: 0x03 })` |
| 1 | `src/sysw/mod.rs` | `    PreimagePlate,` (unique *in that file*; `seal/record.rs:81` has the same text for a different enum — the plan scopes it correctly) |
| 1 | `src/sysw/mod.rs` | `if crate::seal::record::preimage_plate(record) {` |
| 1 | `src/sysw/mod.rs` | `a_preimage_plate_is_named_not_misdiagnosed` |
| 1 | `src/main.rs` | `U::PreimagePlate => format!(` |

Every line citation in the plan's Self-review §3 is TRUE at the stated revision (checked with
`git show <rev>:<file> | sed -n '<n>p'`): ms `cd0a60f` `decode.rs:46` = `pub fn decode(s: &str) ->
Result<(Tag, Payload)> {`; `payload.rs:46` = `Preimage(zeroize::Zeroizing<[u8; 32]>),`;
`error.rs:67` = `TagKindMismatch {`; `error.rs:76` = `ReservedPrefixViolation {`; engrave
`0f5ce23` `record.rs:81` and `sysw/mod.rs:158` both = `    PreimagePlate,`. `ITER` exists at
`sysw/mod.rs:585`. Task 3's `MISMATCH` constant is byte-identical to the corpus's
`preimage-shape-entr-id` string and is 75 characters; the existing `PREIMAGE_PLATE` constant is
byte-identical to `preimage-plate-0x03`.

---

## Closing counts

| severity | n | ids |
| --- | --- | --- |
| Critical | 0 | — |
| Important | 2 | I-1 (predicate narrowing → "re-encode the entropy" on three `0x03` families), I-2 (`Ok(_) => RecordKind::Ms` fails open on the next kind) |
| Minor | 6 | M-1 … M-6 |
| Nit | 2 | N-1, N-2 |

**NOT GREEN.** Both Importants live in the same six lines (Task 2 Step 1's arm and Step 2's
predicate), so one fold closes both; M-5's items (1) and (4) are worth taking in the same fold
because I-1 changes which of those messages an operator sees.
