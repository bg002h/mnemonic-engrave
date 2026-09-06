# SPEC — Hashlock H6: preimage plates on the SeedHammer II

**STATUS: R0 GREEN (0 Critical / 0 Important open); plan-round fold `a2a031fa` verified GREEN (sonnet, `hashlock-H6-spec-plan-round-verification.md`: 26/26 applied, every number re-measured).** The R0 gate closed
0C/0I; the plan round then reopened this spec, and `## Plan-round fold` at the
end lists every change with its own measurement. Brainstorm walked live with the operator (nine decisions + Group A rulings + the C-1 encoder ruling, recorded in `design/CONTINUITY_composer_2026-09-01.md`); draft `a0f832d0` by the opus design author; round 0 (three lenses): fidelity + design (opus, `hashlock-H6-spec-R0-r0-fidelity.md`, 4C/7I/7M/3N), journey walk (opus, `-journey.md`, 1C/7I/7M/2N), tests + citations (sonnet, `-tests.md`, 0C/3I/1M/1N); fold `4881474f` by an opus fold author (every number its own measurement; steps that could be run were run: budget fuzz, scale-2 arm); r1 fold verification (sonnet, `hashlock-H6-spec-R0-r1-fold-verification.md`) **GREEN**, every claim re-derived from scratch. Lens-closure: fidelity, journey, tests/citations, fold-verification. §16 names the two items only the plan (budget values) and a physical plate (the QR scan gate) can settle. Written
from `design/BRAINSTORM_hashlock_H6_preimage_plates.md` against the operator's
rulings of 2026-09-05 (decisions 1-9 of
`design/agent-briefs/hashlock-H6-brainstorm-draft-brief.md`, and the Group A/B
rulings that settled `design/agent-reports/hashlock-H6-brainstorm-draft-report.md`
and `design/agent-reports/hashlock-H6-brainstorm-journey-questions.md`). Round 0
of the R0 gate ran three lenses -- fidelity (4C/7I/7M/3N), journey (1C/7I/7M/2N)
and tests (0C/3I/1M/1N) -- and section 16 records what each finding changed.
**Every number in this spec is a measurement of the text and the geometry as
written.** The round-0 fold re-took every one of them in its own detached fork
worktree at `fb0dd04` (`/scratch/code/shibboleth/.tmp/h6-fold`, Go 1.26.7,
removed after the run), by running the fork's own functions -- `qr.Encode` at
ECC-L, `ConstantQR` with section 7 applied literally, `backup.CharsPerLine` /
`LinesPerPlate` / `fixedCharWidth`, `assertModalBodyFits` / `modalHeadroom`,
`codex32.NewSeed`. Numbers that did not reproduce were REPLACED by the fold's
measurement, never re-stated. Citations measured at engrave `a0f832d0`, fork main
`fb0dd04`, mnemonic-secret `504ff46`; re-grep at plan time.

**What this stage does, in one sentence.** It puts a hashlock preimage on steel:
either as the ms1 kind-`0x03` string `ms hashlock` prints, or as the phrase and
its method in plain text, on a plate layout of its own that says `NOT A SEED`.

**What it reverses, named so nothing is folded silently.** Ruling L7 scoped the
device to the digest alone -- *"It never stores, shows, engraves or sources a
preimage"* -- and H2 implemented that literally: `hashlockPhraseRoute` derives X
on the stack and drops it when the function returns
(`gui/composer_hashlock.go:43-86`). H6 lifts three of those four verbs (store,
show, engrave) and leaves the fourth (source: reading a preimage plate back into
a seed flow) refused. **FIVE shipped records become false and are rewritten by
THIS stage** -- the fourth added by the R0 round-0 journey lens, the fifth by the
PLAN round's (journey I-2); the fifth is a device COPY body rather than a
comment, so it is rewritten where its fit gate is (plan Task 8a Step 5) rather
than with the others:

1. `gui/composer_hash.go:27-28` -- *"THE COMPOSER DERIVES A PREIMAGE IN RAM FOR
   ONE SCREEN (H2) AND NEVER STORES, SHOWS OR ENGRAVES IT."*
2. `gui/composer_hashlock.go:17-20` -- *"The preimage lives on the stack here and
   is dropped when this function returns (L7, L15)."*
3. `SPEC_wallet_policy_composer.md` §6c and its §14 row -- *"From H2 the composer
   derives a preimage in RAM for the length of one screen and never stores, shows
   or engraves it."*


4. `crates/ms-cli/src/cmd/hashlock.rs:352` -- the engraving card's *"write the
   method line next to your phrase; **it is on no plate**"*. §6.2 and §8.6 put the
   method line ON the phrase-form plate, so that sentence becomes false. It errs
   safe (it asks for a copy the operator no longer strictly needs), but it is a
   record this stage falsifies in a repo the spec otherwise treats as primary,
   and it was missing from this list. (Round-0 journey, closing note.)
5. `composerCopyHashlockConfirm` (`gui/composer_copy.go:426` at fork `fb0dd04`,
   the sentence itself at `:437`), and
   `SPEC_hashlock_H2_device.md` §4.5's blockquote of it -- *"The phrase and
   method are not on this device."* (**plan round 0, journey I-2**). §2.2 stores
   both in `hashlockHeld` for the composition's lifetime and §6 engraves both
   onto a plate; in `hashlockPhraseRoute` the falsification is ONE STATEMENT
   WIDE, because the next production statement after this modal is accepted is
   `composerHoldHashlockMaterial`. Unlike record 4 this one errs in the
   DANGEROUS direction, and it is the direction §10.1's held arms were written to
   fix -- *"saying a backup does not exist when it is about to be cut is the
   direction that costs the operator a plate"* -- except that those arms are
   guarded by `composerEveryPathHashed`, which §10.1 records is false the moment
   one path is keyed. So the stage fixed the sentence on the banner drawn NOWHERE
   and left it false on the modal drawn on EVERY phrase route. It becomes
   *"Write down this phrase, the method and this digest now. This composition
   holds them until it ends. Without both, this path can never be spent."* --
   MEASURED at **342 drawn / headroom 107**, one character shorter than the
   sentence it replaces, because this body has only 27 characters of room against
   `modalBodyMargin = 80` and a longer rewrite is REFUSED by
   `TestConfirmScreensThisBlockTouchesAreDrawnInFull` (364 drawn / headroom 64).

`codex32/mspayload.go:63-93`'s `IsPreimage` header keeps its second half (*"a
hashlock preimage is not a seed; engraved as one it exposes a spend secret as a
backup"*) -- still true, and still the reason the plate this stage cuts is not a
seed plate -- and loses its *"the device learns to USE a preimage in stage H2,
not here"* sentence, which H6 makes false.

---

## §1. Scope

**In:**

1. Two plate FORMS on one dedicated layout (§6): the ms1 preimage string
   (text only), and the phrase with its method (optional QR).
2. Raising the constant-time QR encoder from v5 to **v9**, AND giving its module
   engraver a scale-2 arm, so the decided QR text can be cut at all and cut
   without leaking the phrase through the toolpath (§7). Both halves are
   normative: without the scale-2 arm `ConstantQRCmd.Engrave` panics on the only
   scale the plate fits at, and without the raised `constantTimeQRModules`
   budget every raised version refuses (§7.3, §7.4).
3. A new payload record class for the ms1 preimage string, and a NEW
   `phrase:` record for a phrase and its method -- Rust first, with corpus rows
   (§3), ported to Go (§4).
3a. `codex32.EncodeMS1Preimage` on the DEVICE, so a composer-derived preimage has
   an ms1 string to engrave at all (§3.5). The Rust encoder this ports already
   exists and is the one `ms hashlock` prints the plate with, so there is no
   new Rust-first deliverable here -- only the Go port and its lockstep.
3b. **A PUBLISHED `ms-codec` 0.9.0 carrying the phrase rule and the QR text, and
   it is the stage's FIRST deliverable** (§3.1, §12 item 0). `me` depends on the
   codec and not on the CLI, so the rule §3.1 requires it to apply is not
   reachable until the move is released and `me-cli`'s dependency is bumped;
   everything in §3 is blocked on it. (Plan round, finding 1.)
4. `me sysw pack --pack-preimage`, the sealing consequence, and four host
   warnings (§3).
5. The `hash` id requirement on the H6 admission path, both sides (§4.3).
6. `Which hash?` gains preimage and phrase rows (§5.1); a **Hashlock plates**
   flow under the Wallet Policy door (§5.2); a per-plate PICK step and the Done
   review, the latter folded into the `Plates To Cut` census (§5.3).
7. Retention of phrase, method and preimage for the composition's lifetime,
   keyed by digest, scrubbed by the flow-exit defer (§2.2).
8. `PREIMAGE REQUIRED` marking on the composer's md1 **and mk1** plates
   (§10.3); the two abort arms (§8.4); provenance-aware §8h copy (§10.1).
9. The free-text and passphrase ms1 warning (§9), and the Password-program
   notice that names where a hashlock phrase is used (§8.8).

**Out (§13):** reading a preimage plate back into any seed flow; the sealed
**Sealed Payload** container (`me seal`), whose refusal stands unchanged; the
typed `M*1 STRING` door; `ms hashlock` learning to parse the QR text (§8.6 pins
the text; the parser is a follow-on); `ms split` of a preimage (F-468); the
salt/iteration parameters (F-469); a words-plus-SeedQR secret plate (F-455); a
concrete-descriptor plate (F-457).

---

## §2. The two paths, the state model, and retention

### §2.1 The two paths

| | **(a) composer-native** | **(b) payload-delivered** |
| --- | --- | --- |
| entry | the phrase route at HOLD (`gui/composer_hashlock.go:69-70`) | an unsealed or unlocked `me sysw pack` payload |
| holds | phrase bytes, method, X, H | X and H (ms1 form); phrase, method, X and H (`phrase:` form) |
| lifetime | one `composerFlow` call | the session's records (`gui/sysw_session.go:54`) |
| cut by | the composer's engrave step (§5.3) | the Hashlock plates flow (§5.2) **or** the composer, when a path carries the digest |
| in flash? | never | **yes** -- cleartext when unsealed, ciphertext when sealed (§3.4) |

### §2.2 Retention -- NORMATIVE

`composerState` (`gui/composer_state.go:26-79`) gains ONE field, beside
`phraseDigests` (`:53`) and keyed the same way:

```go
// hashlockHeld is the material THIS COMPOSITION may cut onto a preimage
// plate, keyed by digest exactly as phraseDigests is (H5 §2's C16 reasoning:
// "Remove path" splices the slice, so an index is not an identity).
hashlockHeld map[[32]byte]hashlockMaterial
```

where `hashlockMaterial` carries `phrase []byte`, `method hashlockMethod`,
`preimage [32]byte` and `provenance` (`hashlockFromPhrase` |
`hashlockFromPayload`). Rules:

1. **One insertion site**, `composerHoldHashlockMaterial(st, h, m)`, which
   ALLOCATES when the map is nil. `composerState` is built at its one production
   site (`gui/composer_flow.go:48`) as
   `&composerState{reg: &seedRegistry{}, bound: composerBoundFrom(ctx.sysw)}` --
   two fields set and every other field zero -- and the same way in every test,
   so `hashlockHeld` arrives nil and an assignment into a nil map panics --
   in the GUI goroutine, at the moment the operator holds to confirm a hash that
   gates funds. This is `composerNotePhraseDigest`'s rule
   (`gui/composer_state.go:280-286`) applied to the second map, and it is
   normative for the same demonstrated reason. (That function's own comment cites
   `gui/composer_flow.go:34` for the construction site, which is stale -- the
   literal is at `:48`. Fixing the comment is a nit for this stage's fold.)
2. **Nothing deletes.** A digest no path carries is not removed; it is REPORTED
   (§5.3 item 4). Deletion is what `phraseDigests`' own comment
   (`gui/composer_state.go:46-50`) refuses, and the same argument holds here.
3. **The scrub is the flow-exit defer.** `composerFlowExit(st)`
   (`gui/composer_flow.go:20-23`, installed at `:59` before any secret can
   exist) gains `composerScrubHashlockHeld(st)`, which `wipeBytes` every
   `phrase` and zeroes every `preimage`. **It goes in the EXISTING defer, not
   beside it:** `composerFlowExit`'s own comment records that a second
   `defer` costs 96 B of firmware flash because TinyGo removes the empty stub's
   CALL and not the defer bookkeeping around it.
4. **Retention survives a Back out of the engrave step.** `composerEngraveStep`
   returning false sends `composerFlow` round its loop
   (`gui/composer_flow.go:47-131`) with the state intact; the material is gone
   only when `composerFlow` returns or power is lost. §8.4a's copy says
   exactly that and no more.
5. Secret-handling defects in this field are non-gating (operator ruling
   2026-08-27) and are logged as follow-ups. F-483 already records that the
   typed phrase lives in `kbd.Fragment`, an immutable Go string, before H6
   stores anything.
6. **THIS FIELD AND ITS CONSUMER LAND TOGETHER, IN ONE COMMIT -- NORMATIVE**
   (plan round, finding 7). `composerHoldHashlockMaterial` is a production
   function whose only callers are §5.1's two payload routes and the typed
   phrase route's own HOLD. Landing the retention on its own leaves it with no
   production caller, and the fork's
   `TestComposerEveryScreenFunctionHasAProductionCaller`
   (`gui/composer_join_test.go:35`) reds -- correctly, and deterministically. The
   red was measured twice on a retention-only tree, by the plan author and again
   by the build gate, each reporting `[composerHoldHashlockMaterial]` from one
   shard; MEASURED here, the same test PASSES on the fully wired tree, naming
   only the pre-existing `composerDescriptorCeilingChars` exemption. That guard is the one two R0 lenses earned after finding
   fourteen unreachable production functions at once with a green suite, so
   **the exemption table (`gui/composer_join_test.go:87-90`) is NOT the
   instrument here**: it exists for a consumer DEFERRED with a follow-up number
   (`composerDescriptorPlateFits` -> F-457), and this consumer is in this stage.
   Adding a name to it to make the retention green alone would quiet the one
   gate that catches this defect class.

### §2.3 Payload-delivered material is NOT copied into `composerState`

The Hashlock plates flow (§5.2) reads `ctx.sysw` and holds nothing: decision 4
says it cuts "without a composition". When the composer takes a payload preimage
onto a path (§5.1), the material IS entered into `hashlockHeld` with
`provenance = hashlockFromPayload`, because the composer's review and abort copy
must speak about everything it may cut.

---

## §3. Host: the records, `me sysw pack`, sealing -- Rust first

### §3.1 The two carriers -- NORMATIVE

| carrier | wire form | why this shape |
| --- | --- | --- |
| **preimage** | the bare ms1 kind-`0x03` string, id `hash`, 75 characters | it is what `ms hashlock --out` writes (`crates/ms-cli/src/cmd/hashlock.rs:299-306`), from `ms_codec::encode(Tag::HASH, &Payload::Preimage(x))` at `:298`; a prefixed re-encoding would make the operator transform a file they already hold |
| **phrase** | `phrase:<hex of the UTF-8 text "<method>,<phrase>">` | the `now:` idiom exactly (`sysw/composer_records.go:136-160`): hex of a UTF-8 text, one `,`, cut on the FIRST comma so the phrase may contain commas |

`<method>` is `hardened` or `sha256` -- the spelling
`hashlockMethod.String()` already uses (`gui/composer_hashlock.go:35-40`) and
`ms hashlock --method` already accepts. **The wire carries a method SELECTOR and
the plate carries the method DEFINITION (§8.6), and the difference is
deliberate:** a wire record is read by a tool that already knows the parameter
set, while a plate is read by a person who may have neither the tool nor this
firmware.

Prefix reservation follows `sysw/record.go:14-21` and
`sysw/composer_records.go:24-31`: `phrase:` joins `key:`, `hash:`, `now:` as a
RESERVED prefix, so a `phrase:` record whose body is not valid lowercase hex is
`ClassUnknown` and refused rather than treated as free text.

**Body validation, each failure `ClassUnknown` and refused with its own line:**
the body must be even-length lowercase hex (`unhexLower`,
`sysw/composer_records.go:82-97`); the decoded bytes must be valid UTF-8; the
text must contain at least one `,`; the field before the first `,` must be
exactly `hardened` or `sha256`; everything after it must pass the phrase rule of
`SPEC_ms_hashlock` §4.3 in the host's order -- non-empty, printable ASCII
`0x20..=0x7E`, not ms1-shaped, at most 100 characters, not exactly 64 hex
characters -- byte for byte the rule the device's `hashlock.ValidatePhrase`
(`hashlock/hashlock.go:92-111`) already applies.

**THE RULE MOVES INTO `ms-codec`, AND A PUBLISHED RELEASE IS THIS STAGE'S FIRST
DELIVERABLE -- NORMATIVE** (plan round, finding 1). The first draft named
`ms-cli`'s `validate_phrase` as the implementation `me` must call. MEASURED:
`validate_phrase` is `ms-cli`'s (`crates/ms-cli/src/hashlock_phrase.rs:118`), so
is `looks_like_ms1` (`crates/ms-cli/src/argv_guard.rs:148`), and
`crates/me-cli/Cargo.toml:53` depends on `ms-codec = "0.8"` and on nothing of
`ms-cli`. **`me` cannot reach either function.** A literal reading leaves only
one route -- a THIRD copy of the rule -- and a third copy is the defect the rule
exists to prevent, whose whole point is that no two readers of a phrase disagree
about what one is. So:

1. `validate_phrase`, `PhraseRefusal`, `looks_like_ms1`,
   `HASHLOCK_PHRASE_MAX_CHARS` and `qr_text` (§8.6) move into
   `crates/ms-codec/src/hashlock.rs`, beside the constants they already read
   (`:27,30,32`). `ms-cli` DELEGATES and keeps its message rendering, so every
   refusal sentence an operator sees is unchanged and there is still exactly one
   implementation.
2. **`ms-codec` 0.9.0 is PUBLISHED before anything in §3 is built**, through
   `mnemonic-secret/design/RELEASE_PROCESS.md`'s checklist. Item 1 of that
   checklist is what fixes the version: *"Any subsequent change to the corpus
   that would alter the SHA requires a SemVer minor bump (`0.X+1.0`)"*, and §11.2
   adds seven `qr_text` rows to `hashlock-v0.8.json`. MEASURED, the corpus moves
   from the sha the H1 release recorded to
   `4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4`, which is
   the literal the CHANGELOG entry carries and the one the fork re-pins at §11.2.
   The precedent is H1's own entry, `## ms-codec [0.8.0] -- 2026-09-05`, which
   recorded `a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30`
   the same way; items 3, 7 and 8 (CI green, `cargo publish --dry-run`, the
   `ms-codec-v0.9.0` tag) bind unchanged.
3. **Then `crates/me-cli/Cargo.toml:53` bumps `ms-codec = "0.8"` to `"0.9"`.**
   Nothing else in §3 compiles until it does. A `[patch.crates-io]` standing in
   for the release is a scratch device for building the rest and is never
   committed.


**The ms1 preimage ENCODER already exists in Rust and is not re-decided here.**
`ms_codec::encode` (`crates/ms-codec/src/encode.rs:16`) takes `(Tag, &Payload)`
and writes the `0x03` prefix through `envelope::payload_wire_bytes`
(`crates/ms-codec/src/envelope.rs:261`); `PayloadKind::Preimage`'s
`single_tag()` is `Tag::HASH` (`crates/ms-codec/src/payload.rs:23-28`), and
`encode` REFUSES any `tag != payload.kind().single_tag()` with
`Error::TagKindMismatch` (`encode.rs:24-31`). So the Rust primary structurally
cannot emit a preimage under id `entr`, the corpus already pins the string, and
§3.5 is a Go PORT, not a new Rust-first deliverable. (Round-0 journey C-1 read
the gap as a missing Rust encoder; the operator's ruling was "add the encoder,
Rust first", and the controller then established that the Rust half is already
shipped. §3.5 is that ruling applied to the half that is actually missing.)

**Rust first.** `crates/me-cli/src/sysw/composer_records.rs` gains
`PHRASE_PREFIX`, `PhraseRecord`, `phrase_record(method, phrase)`, the parse arm
and `CASES` rows (`:306`); the corpus
`crates/me-cli/testdata/record_class_vectors.json` is regenerated by its own
test and re-vendored to `sysw/testdata/record_class_vectors.json` with the
provenance pin re-recorded (today: commit
`c05074f1d45970ca416785dfa9d9a812aaa21dbd`, sha256
`5b3960cad7f924f6f1e7f19ef49599814733cee4874d0f5eb48c28af4cd8b312`, 47 rows).
Nothing is decided in Go.

### §3.2 `--pack-preimage`

One flag, both carriers, in the pattern of `--seal-secret`
(`crates/me-cli/src/main.rs:126`, its refusal at `:932-937`):

```
/// Admit a hashlock PREIMAGE into this payload: an ms1 kind-0x03 plate
/// string, or a `phrase:` record carrying a hashlock phrase and its method.
///
/// Both are BEARER material -- whoever holds the preimage can spend any
/// key-less hashlock path it unlocks -- so admission is explicit, exactly as
/// `--seal-secret` makes encrypting seed material explicit. It is NOT
/// `--seal-secret` and the two do not substitute.
#[arg(long)]
pack_preimage: bool,
```

**Where the flag acts -- NORMATIVE, and it is ADMISSION, never classification
(round-0 fidelity C-3).** The spec's first draft named the flag and its refusal
but never said which function consults it, and the two readings an implementer
could take each break something funds-relevant. This settles it:

1. **Classification is UNCONDITIONAL on both sides.** `Classify` /
   `classify_with` answer `Preimage` and `Phrase` for these records whatever the
   admission, exactly as they answer `Key`, `Hash` and `Now`. The flag is not a
   parameter of what a record IS.
2. **`Admission` (`crates/me-cli/src/sysw/mod.rs:233-236`) gains
   `pack_preimage: bool`, and `admit_check` (`:463-470`) gains a SECOND refusal
   rule** beside its existing `Class::Unknown` one: a record classifying
   `Preimage` or `Phrase` is refused by index unless `adm.pack_preimage`. It is a
   new `SyswError` arm, not a `Class::Unknown` / `UnknownReason`, because the
   record is not unclassifiable -- it is perfectly well understood and simply not
   asked for. §8.1.1 is that arm's text.
3. **`decide_sealing` (`crates/me-cli/src/main.rs:2353-2410`) is UNCHANGED and
   still calls `sysw::classify` (`sysw/mod.rs:246-248`).** Because
   classification is unconditional, the strict classifier already sees the two
   classes, `is_secret()` is true for both, and the payload SEALS by default
   (§3.4 item 1) with no second call site to keep in step.
4. **The vendored corpus keeps ONE truth per row.** `CASES`
   (`crates/me-cli/src/sysw/composer_records.rs:306`) generates
   `record_class_vectors.json` with `classify`, and the fork's
   `TestComposerRecordsClassifyExactlyAsTheHost`
   (`sysw/composer_records_test.go:64-75`) compares it against the device's
   `Classify`. Unconditional classification is what lets the H6 rows carry the
   same class on both sides; an admission-gated classifier would make the host
   row say `Unknown` and the device say `ClassPreimage`, and the lockstep gate
   would red.

**Why not thread `admission` into `decide_sealing` instead** (round-0 fidelity
C-3's own suggested remedy, declined with a reason): that fix works, but it adds
a SECOND call site that must be kept in step with the first, which is the exact
defect the shipped `--expect` comment records at `main.rs:1488-1494` -- built
without `admission`, `--expect` produced *"a false refusal carrying a false
message, on the funds path, inside the feature added to prevent exactly that."*
Gating admission rather than classification leaves `decide_sealing` byte-unchanged
and has no second site to forget.

Without the flag, `me sysw pack` refuses by index through `admit_check`
(`crates/me-cli/src/sysw/mod.rs:463-470`) with §8.1.1. `unknown_reason`'s
preimage arm (`:210`) is UNTOUCHED and keeps the case it actually covers today --
a kind-`0x03` single under an id outside `{entr, hash}`, which really is
unclassifiable (§4.3).

### §3.3 Host warnings -- NORMATIVE, all four (§8.2)

`me sysw pack --pack-preimage` prints, on stderr:

1. **Always, when a preimage or `phrase:` record is admitted:** the transit
   warning (§8.2.1).
2. **When the flag is given and no preimage or `phrase:` record is present:** the
   no-op warning (§8.2.2). A warning, never a refusal -- the flag loosens
   admission and loosening it over nothing costs nothing.
3. **When an admitted preimage OR `phrase:` record's digest matches none of the
   payload's `hash:` records:** the orphan warning (§8.2.3). **It covers BOTH
   carriers, and the `phrase:` half is the one that matters** (round-0 journey
   I-3): a preimage record is produced by `ms hashlock --out` and is correct by
   construction, while a `phrase:` record has no CLI producer at all -- §3.1
   specifies a Rust function, §3.2 adds only a flag -- so the operator hand-builds
   it as hex, following the `text:`/`pass:` precedent the `pack` help already sets
   (`crates/me-cli/src/main.rs:219`). Two hand-build errors pass every check §3.1
   lists and are caught by nothing else:
   - **a space after the comma.** §3.1 cuts on the FIRST `,` so the phrase may
     contain commas, and the remainder must be printable ASCII `0x20..=0x7E` -- a
     leading `0x20` is printable, so `hardened, my phrase` is admitted and derives
     a DIFFERENT preimage from `my phrase`. The `now:` idiom §3.1 follows is safe
     from this only because both of its fields are digit-constrained
     (`ParseNowRecord`, `sysw/composer_records.go:136-160`, via `digitsInRange`);
     a free-form phrase field does not inherit that safety.
   - **the wrong method selector.** `sha256,<phrase>` when the host derived with
     `hardened` is a valid record and a valid method. §5.1 removes the method
     PICK; nothing checked the RECORD.

   Both are one PBKDF2 run away on the host -- milliseconds there against ~10 s on
   the device, after a pick, at a screen with no copy for a digest that matches
   nothing. So the host derives an admitted `phrase:` record's preimage at pack
   time and warns when its digest matches no `hash:` record.

   **Giving `phrase:` a producer is OUT of scope this stage and named in §13**, so
   the hand-build path is the one this warning has to cover.
4. **When the payload is SEALED and holds a preimage or `phrase:` record:** the
   sealed-transit note (§8.2.4).

**Where they print, and in what order against the passphrase ceremony --
NORMATIVE (round-0 fidelity M-6).** All four are payload-wide, so none belongs in
the per-record `admit_check`. Warnings 1-3 print in `main.rs`'s pack arm
**BEFORE** the passphrase ceremony, alongside the existing `--expect` and
unsigned-override reporting, which `main.rs:1474-1494` already orders that way and
says why: F-246 -- *"a warning the operator reads after writing a passphrase down
is a warning about work already done"*. Warning 4 prints **AFTER** the sealing
line, because *"SEALED and holds a hashlock preimage"* is a follow-on from the
sealing determination rather than a non-sequitur. **It names no direction**
(plan round 0, fidelity I-6 = journey M-1): the first draft's wording was *"the
device needs the passphrase above"*, and the passphrase is printed BELOW it --
MEASURED, the sealing line one line up says *"opens only with the passphrase
below"*, so two consecutive lines pointed opposite ways at the same passphrase,
and with `--passphrase-ask` the prompt has not been shown at all when the note
prints. `this payload's passphrase` is true on every path and keeps the
ordering. §3.3's earlier draft placed the orphan check "with `pack_with`
(`crates/me-cli/src/sysw/mod.rs:335`) beside the 'at most one `now:`' rule"; that
is wrong on both halves -- the `now:` rule is enforced in `split`
(`sysw/mod.rs`) and again in `main.rs:1597` for ordering reasons, and `pack_with`
is the entry point above them, with no stderr of its own.

### §3.4 Sealing -- NORMATIVE, and one correction to the ruling's premise

`ClassPreimage` and `ClassPhrase` **are secret**: `Class.IsSecret()`
(`sysw/record.go:60-66`) answers true for both, and `is_secret()` on the Rust
side likewise. Three consequences, all automatic:

1. `decide_sealing` (`crates/me-cli/src/main.rs:2353-2410`) SEALS by default,
   because it seals iff some record's class is secret. `--no-passphrase`
   (`crates/me-cli/src/main.rs:244`) produces the cleartext payload; the sealing
   line already says which way it went and why, and now names the class.
2. On the device, `syswFlags` (`gui/sysw_admit.go:143-162`) derives
   `flagSecretInPlaintext` from `c.IsSecret()` alone, so an unsealed payload
   holding a preimage raises F1 -- *"A SECRET is stored unencrypted in flash."*
   (`gui/sysw_load.go:283`) -- at LOAD, with no new wiring. A sealed one with a
   weak passphrase raises F2 the same way.
3. `bundleSetCarriesASecret` and the seed-plate arms are untouched: a preimage
   plate is not a `bundleCard` (§5.3).

**CORRECTION, measured (report §2, deviation D1).** The ruling's premise for
warning 4 -- *"the device cannot unlock it (the F-474 arm refuses a preimage in a
sealed section)"* -- does not hold for this container. `sysw.Open`
(`sysw/open.go:36-73`) runs no admission at all, and `syswSession.load`
(`gui/sysw_session.go:79-110`) appends `p.Public` **and** `p.Secret` into
`s.records`, so a sealed `sysw` payload's preimage IS reachable after unlocking.
The F-474 arm (`gui/unlock_kdf.go:415-420`, its noun at `:433`) belongs to
`unlockSealedFlow` (`gui/unlock_flow.go:98`) over `seal.Payload` -- the frozen
**Sealed Payload** container, a different product, whose `AdmitSection`
(`seal/open.go:149`) does refuse a preimage plate and whose host half refuses it
at pack time already (`crates/me-cli/src/seal/record.rs:130-136`). §8.2.4 states
what is true of each container and claims nothing about the other.

### §3.5 `codex32.EncodeMS1Preimage` -- the device half, a PORT -- NORMATIVE

Without it a composer-derived preimage has no ms1 string to engrave, and the
string form -- decision 1's DEFAULT -- silently becomes payload-only.
`codex32.EncodeMS1` (`codex32/msencode.go:17-31`) is the fork's only ms1 encoder
and is `entr`-only by construction: its own header pins the id as the *"FIXED
literal `entr`"*. The primitive it wraps is general, so this is a sibling
wrapper, not new codec design:

```go
// EncodeMS1Preimage encodes a hashlock preimage X as the ms1 kind-0x03 plate
// string: NewSeed("ms", 0, "hash", 's', [0x03‖X]). It is the Go port of
// ms_codec::encode(Tag::HASH, &Payload::Preimage(x)) (SPEC_ms_hashlock §1
// rule 2) and is DOWNSTREAM of it -- the Rust side decides the wire form.
//
// The id is the FIXED literal "hash" and there is no parameter for it. That is
// the whole point: NewSeed will mint a kind-0x03 payload under id "entr" quite
// happily (MEASURED -- see below), and the Rust encoder refuses that shape
// outright, so the id must not be reachable from a caller here either.
func EncodeMS1Preimage(x [32]byte) (string, error)
```

**MEASURED against the ms corpus, in the fold's own worktree.** The wrapper
reproduces `hashlock-v0.8.json`'s `kind` row byte for byte:

```
X        = 0xab * 32
encoded  = ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c
corpus   = ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c
len=75  matchesCorpus=true  id="hash"  IsPreimage=true
DecodeMS1Preimage round-trip = true
```

and the copy-paste hazard is real rather than hypothetical:

```
EncodeMS1(entr) on the same 32 bytes
  = ms10entrsqz46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kdv3c0wn2hx0lq
    matches corpus entr32_pair = true   IsPreimage = false
NewSeed("ms", 0, "entr", 's', [0x03‖X])          <-- NOT refused by NewSeed
  = ms10entrsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kp9wv63u5a0u7q
    IsPreimage = true   IsPreimagePlate = false
```

The third line is the finding: **the Go primitive has no equivalent of the Rust
encoder's `tag != payload.kind().single_tag()` refusal**, so a mistagged plate is
one parameter away, it satisfies the wide `IsPreimage`, and §4.3's admission
predicate then refuses it on the way back in -- an operator's only backup of a
spend secret, on steel, that no tool will read. Hence the fixed id and §11.1's
mutation.

**Its tests (§11.1):** the corpus `kind` row lockstep (id `hash`, kind `0x03`,
75 characters, the `ms1` field byte for byte); `DecodeMS1Preimage(New(
EncodeMS1Preimage(x))) == x` over random X; and the negative -- **the function
cannot emit id `entr` for any input**, asserted on the OUTPUT (`Split()`'s id is
`"hash"`), not on the source, so a future refactor that reintroduces an id
parameter reds. The provenance pin records `ms-codec` and the corpus sha.

### §3.6 The two classes are BEARER, so the COMMAND LINE is refused -- NORMATIVE

**Making them bearer moves the argv surface, and the first draft said nothing
about it** (plan round, finding 5). `Class::is_bearer`
(`crates/me-cli/src/sysw/record.rs:102-104`) is `Mt | Tx` today and gains
`Preimage | Phrase`; `is_argv_forbidden` (`:118-120`) is
`is_secret() || is_bearer()`, so `argv_secret_guard`
(`crates/me-cli/src/main.rs:551`) refuses either carrier on the command line
**before the parser runs**. Two consequences, both normative:

1. **Every invocation that carries a preimage or a `phrase:` record uses the
   private channel.** `me sysw pack --pack-preimage --no-passphrase <ms1>` --
   §12 item 3's own acceptance invocation as the first draft wrote it -- is
   REFUSED, and `--in <file>` is the route the guard's own message already
   offers. **§12 item 3 is the one acceptance item that packs a record, and it is
   written to `--in` for that reason** (items 3a, 4, 5 and 5a describe payloads
   and screens and carry no `pack` invocation); so is every test of §11.1. The
   refusal is CORRECT: argv is public, and /proc,
   `ps` and the shell history all keep a copy.
2. **The guard's MESSAGE was false for these classes, and that is a defect in
   what the tool claims to have found.** Its bearer arm
   (`crates/me-cli/src/main.rs:566`) says *"BEARER material -- a signed
   transaction, or the mt1 set carrying one. Anyone who can read it can broadcast
   it"*, which describes neither carrier. The arm gains a hashlock case naming
   the plate string or the phrase a `phrase:` record carries, and saying that for
   a key-less hashlock path it alone spends the coins. A refusal that names the
   wrong material is not a nicety; it tells the operator to look for something
   they are not holding.

Secret-handling defects are non-gating (2026-08-27), and this is not one: the
guard already refuses correctly. What changes is the acceptance path and the
sentence the refusal prints.

---

## §4. Device admission and classification

### §4.1 Two new classes, admitted at one program

`sysw.Class` gains `ClassPreimage` and `ClassPhrase`
(`sysw/record.go:24-54`). `admitted` (`gui/sysw_admit.go:32`) gains both to
`progWalletPolicy` **and to no other row** (`:64-73`), exactly as `ClassKey`,
`ClassHash` and `ClassNow` are. The row comment states, normatively, that a
hashlock phrase is **not** a BIP-39 passphrase and is never admitted at
`progPassword`: the terminology ruling L2 exists because interchanging them
opens a different wallet, and `progPassword`'s row stays `{ClassPassphrase}`.

### §4.2 The classifier

`Classify` (`sysw/record.go:111-139`) gains a `PhrasePrefix` arm beside the
other reserved prefixes, and `classifyConstellation`'s ms1 arm learns the
preimage class through a NEW predicate. `isStrictMs1`
(`sysw/classify.go:116-127`) is **unchanged**: its final line
(`return err == nil && !codex32.IsPreimage(c)`) is H0's inertness and keeps a
preimage out of every seed class. The preimage class is answered BEFORE it, by
`isPreimagePlateRecord`, so the two rules never overlap.

**EXACTLY ONE CORPUS ROW CHANGES CLASS, AND THE RE-PINNING IS PART OF THE SAME
LEG -- NORMATIVE** (plan round, finding 9). `codex32_seam/preimage-plate-0x03`
goes `Unknown` -> `Preimage` in `crates/me-cli/testdata/record_corpus_pre_s2.json`,
a capture whose own test is named *"not one of these records may change class"*,
so the move is RECORDED in that test's doc comment rather than absorbed. Its
sibling `codex32_seam/preimage-shape-entr-id` STAYS `Unknown`, and **that pair is
§4.3's id narrowing stated as data**: the kind byte alone is not enough to reach
the flow that engraves. Three corpora and their pins, each carried identically on
both sides so a change cannot land in one repo alone:

| corpus | what H6 does to it | where the pin lives |
| --- | --- | --- |
| `record_class_vectors.json` | REGENERATED from `CASES`; 47 -> 68 rows, sha256 `3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf` | `crates/me-cli/tests/sysw_composer_records.rs:399` **and** the fork's `sysw/testdata/record_class_vectors.provenance.json` -- both re-pinned, with the row count |
| `hashlock-v0.8.json` | §11.2's seven `qr_text` rows; sha256 moves to `4f1819cd...aba21d4` | the ms CHANGELOG entry (§3.1) **and** the fork's `hashlock/hashlock_test.go:13` `corpusSHA256`, which reads `a46c197a...11d30` until H6 |
| `record_corpus_pre_s2.json` | the ONE row above | `crates/me-cli/tests/record_corpus.rs` -- a `me`-only capture with no fork twin |

**And the two-repo `codex32_seam_vectors.json` does NOT move -- MEASURED, and it
is stated so nobody edits it.** Its `device_admits` column is
`Classify(s) == ClassCodex32Secret` (`sysw/codex32_seam_test.go`) and its
`host_admits` column is the host's seed profile; a preimage plate classifies
`ClassPreimage`, which is not `ClassCodex32Secret`, so BOTH columns stay `false`
and the row's meaning is unchanged. Its sha256
`2c2fbb3fa4d38c8858b9de4769d876d275478956c76ca491005c70d9f6bd541b` is pinned as a
literal in `crates/me-cli/tests/codex32_seam.rs:25` and
`sysw/codex32_seam_test.go:30`, and editing it would red both suites for no
reason.

### §4.3 The `hash` id is required on the H6 path -- NORMATIVE (ruling A7)

`codex32.IsPreimage` (`codex32/mspayload.go:94-101`) does not consult the id. The
host's `preimage_plate` (`crates/me-cli/src/seal/record.rs:287-320`) consults it
**only to route an `entr`/`hash` mismatch elsewhere** -- it opens with
`if id_kind_mismatch(s) { return false; }` (`:292-295`, the predicate at
`:269-276`, keyed on `ms_codec::Error::TagKindMismatch`) and for every OTHER id
tests `unshared && len(data)==33 && data[0]==0x03`. (Round-0 fidelity I-5: the
first draft said it does not consult the id at all, which is false and made
§8.1.2's row untestable for the most likely wrong id.) That was the
safe direction under H0, where the consequence of a false positive was a
REFUSAL: *"A refusal costs a re-encode; a wrong cut exposes a spend secret"*
(`codex32/mspayload.go:78-92`). H6 inverts the consequence -- a false positive
now routes a string INTO a flow that engraves it under a band reading
`NOT A SEED` -- so the H6 admission path adds the id:

```go
// codex32.IsPreimagePlate is IsPreimage PLUS the id `hash` (SPEC_ms_hashlock
// §1 rule 2, ruling L14). H0's kind-byte rule is unchanged and still governs
// inertness everywhere else; this narrower predicate governs ADMISSION to a
// flow that ENGRAVES, where a plain BIP-93 33-byte secret beginning 0x03
// (roughly 1 in 256 of them) must not arrive.
func IsPreimagePlate(s String) bool {
	if !IsPreimage(s) {
		return false
	}
	id, _, _ := s.Split()
	return id == "hash"
}
```

`s.Split()` is `codex32/codex32.go:394-401`. The Rust primary gains the same
narrowing as `preimage_plate_admissible(s)`, leaving `preimage_plate` -- the
DIAGNOSTIC predicate behind the refusal message -- unchanged, so a mistagged
plate is still named a preimage plate when it is refused and still not admitted
when `--pack-preimage` is passed.

**`preimage_plate_admissible` takes THREE conjuncts, not the one this section
first gave it, and the two extra ones came from a SHIPPED test rather than from
review** (plan round, finding 4). `a_preimage_plate_is_named_not_misdiagnosed`
(`crates/me-cli/src/sysw/mod.rs`) already carried both rows, and an
implementation that stopped at the id reddened on each:

| conjunct | the row that forces it |
| --- | --- |
| `preimage_plate(s)` -- the shipped kind-`0x03` unshared single test | — |
| the id is exactly `hash`, **compared CASE-SENSITIVELY** | the UPPERCASE spelling of a plate, which `preimage_plate` DOES name as a plate. `codex32.IsPreimagePlate` reads the id out of `String.Split()` and compares it to the literal `"hash"`, and §5.3 hashes a record in its canonical lowercase form, so an uppercase string is not the record it looks like |
| the payload decodes to exactly **33 bytes beginning `0x03`** | a kind-`0x03` single under the id `hash` whose X is 16 bytes -- the codec's `PreimageLengthMismatch`, 50 characters. `preimage_plate` deliberately answers true for it so the DIAGNOSTIC names it; ADMITTING it would put a string `DecodeMS1Preimage` refuses into a flow that engraves |

**The device was already right and the Rust needed narrowing.**
`codex32.IsPreimage` (`codex32/mspayload.go:94-101`) has required
`len(d) == 33 && d[0] == 0x03` since H0, and `Split()`'s id comparison is
case-sensitive -- so the third conjunct is the same shape the Go side already
tests and the two agree by construction rather than by review. This is the
Rust-primary rule's *"whenever a defect is found in a Go port we MUST check the
Rust"* running in reverse, and the fix lands in Rust, with the vectors, exactly
as the rule requires. MEASURED, each mutation run against the shipped test:
dropping the well-formedness conjunct and comparing the id with
`eq_ignore_ascii_case` BOTH give
`left: Err(PreimageNotAdmitted(0, Preimage))` against
`right: Err(Unclassifiable(0, PreimagePlate))` -- the malformed and the
uppercase rows become a CLASS, and `--pack-preimage` would then admit them.
§11.1 carries both.

**The three ids, and which refusal each one gets -- NORMATIVE** (round-0
fidelity I-5 + journey I-4; the id partition is what makes each row testable):

| id, on a kind-`0x03` unshared single | what answers | copy |
| --- | --- | --- |
| `hash` | admissible under `--pack-preimage`; refused without it | §8.1.1 |
| `entr` | `id_kind_mismatch` -> `U::TagKindMismatch` (`sysw/mod.rs:207`), **already shipped** | `main.rs:2799-2804`, UNCHANGED |
| anything else | `preimage_plate` -> `U::PreimagePlate` (`sysw/mod.rs:210`) | §8.1.2 |

**There is no NEW arm.** The first draft added §8.1.2 as a fresh
`unknown_reason` arm; measured, `entr` never reaches it (the mismatch is
diagnosed first, and the shipped `TagKindMismatch` body already cites
`SPEC_ms_hashlock §1 rule 2`), and every other id already lands on
`U::PreimagePlate`. So §8.1.2 is the EXISTING `U::PreimagePlate` body rewritten,
not a second one. §9's own rule -- *"two near-identical bodies is how one of them
goes stale"* -- is the reason.

**H0's inertness elsewhere is untouched:** `DecodeMS1` keeps refusing `0x03` at
all five callers (`gui/ms1_decode.go:22`, `gui/codex32_polish.go:106`,
`gui/singlesig_verify.go:185`, `gui/multisig_verify.go:1237`,
`bundle/verify.go:138`); the scan door refuses upstream (`gui/scan.go:89`);
`engraveCodex32` refuses at the choke point (`gui/codex32_polish.go:232-235`);
`seal.Classify` and `seal.AdmitSection` are not touched.

---

## §5. Screens

### §5.1 `Which hash?` gains two row bands

`composerHashRows` (`gui/composer_hash.go:157-175`) builds the row set once and
records each named band's index; the switch in `composerHashEdit`
(`:184-224`) dispatches on those names and its `default` PANICS rather than
assigns (H2 §5, r2 C-4). Row order:

1. the payload's `hash:` digests -- `hash <i>  <first8>..<last8>`, unchanged;
2. **the payload's preimage records** -- `preimage <i>  <first8>..<last8>`;
3. **the payload's `phrase:` records** -- `phrase record <i> (derive to see the
   digest)` before derivation, `phrase <i>  <first8>..<last8>` after;
4. `Type a hashlock phrase`;
5. `Type 64 hex`;
6. `No hash lock`.

Indices are 1-based positions among records of that class, as
`composerHashRow(i+1, d)` already is (`gui/composer_hash.go:161`). MEASURED at
`sh2DisplaySize`: every row above draws on ONE line in `composerPageLines`' band
(411 px, a 23 px label) -- `preimage 10  b867db87..edbc96cb` (31 chars, 276 px)
and `phrase record 10 (derive to see the digest)` (43 chars, 336 px) included.

**`composerHashRows` GAINS A SECOND PARAMETER, `st *composerState` (nil-safe) --
NORMATIVE** (plan round, gate fix F4). Band 3's row FORM depends on what THIS
composition has already derived, and `composerHashRows(s *syswSession)`
(`gui/composer_hash.go:157-175`) cannot see it. The answer comes out of §2.2's
`hashlockHeld` -- the map that already holds the derived material -- and not out
of a second, index-keyed map, so there is one record of what has been derived
rather than two that can disagree.

**THE FIRST PAGE HOLDS FIVE ROWS, NOT SIX, AND "23 px per row" WAS THE WRONG
NUMBER** (plan round, gate fix F6). 23 px is the LABEL height; the row PITCH is
29 px (`sz.Y + 6`, `gui/composer_paged.go:147`), the content box is 224 px, and
the lead and its spacer are drawn through the same band before any row. MEASURED
on a payload holding a `hash:` record, a preimage record and a `phrase:` record
-- six rows -- **page 1 draws 5 touch targets and `No hash lock` is on page 2**.
`composerPickScreen` pages on Button2 and draws the pager icon only when a second
page exists, so the row is reachable; §11.5 pins both the count and the
reachability. That `No hash lock` is the row an operator reaches for to UNDO a
lock makes this worth a follow-up rather than a redesign: it is measured,
reachable, and non-gating.

**Derivation is LAZY (ruling B).** A `phrase:` row derives on PICK, behind the
existing `Deriving` countdown (H2 §4.4), and the result is entered into
`hashlockHeld` so the row shows its digest for the rest of the composition. A
hardened phrase costs about 10 s at the measured 9,715 iterations/s, and three
records would be a 30 s stall before a list could be drawn; the cost belongs on
an action the operator chose.

**The method is the RECORD's, never a pick.** A `phrase:` record names its
method (§3.1); the device does not offer the method screen for it, so the
mistake J4-1 describes -- picking the wrong method for a phrase the operator did
not choose -- cannot occur.

**Which means a payload phrase CANNOT use `hashlockPhraseRoute`, and this stage
names the path it uses instead** (round-0 journey N-2). `hashlockPhraseRoute`
(`gui/composer_hashlock.go:43-86`) does both of the things a payload phrase must
not do: it calls `hashlockMethodPick` (`:53`) and it ends on
`composerCopyHashlockReconcile` (`:83`). The payload path is a DERIVE-ONLY
sibling -- no phrase screen, no method pick, no reconcile screen -- reusing
`hashlockDeriveFlow` for the countdown and `composerCopyHashlockConfirm` (with
`hashlockRelationLine` and `hashlockOtherPathLine`) for the confirm, then
assigning the digest and entering the material into `hashlockHeld` with
`provenance = hashlockFromPayload`. The provenance distinction §10 needs is then
a property of WHICH PATH RAN, not a runtime test.

**A payload PREIMAGE record takes the same shape WITHOUT the KDF, and it needs a
CONFIRM BODY OF ITS OWN -- NORMATIVE** (plan round, gate fix F3). A preimage
record carries X directly, so there is no countdown, no phrase and no method.
`composerCopyHashlockConfirm` is phrase-shaped: it prints `method: <m>` and
`chars: <n>` and tells the operator to write down the phrase and the method, and
a preimage record has none of the three -- reusing it draws
`method: hardened   chars: 0` on the screen that gates funds, which is a
measurement of nothing wearing the clothes of one. The preimage route draws
`composerCopyHashlockPreimageConfirm` instead: the digest, that it came from a
preimage record in this payload, the same two relation lines, and -- in place of
the write-down instruction -- the thing that is actually true here, that the
preimage is in the payload and a plate is how it leaves. MEASURED through
`confirmWarningBody` wrapped in `composerConfirmBody`, longest variant (both
relation lines present): **274 drawn / headroom 186**, the tightest body this
stage adds and still 106 characters clear of the 80-character margin.

**When a `hash:` digest and a preimage or `phrase:` record in the same payload
carry the SAME digest, the `hash:` row says so** (round-0 journey M-4). Packed at
full fidelity -- `hash:` from `ms hashlock`'s stdout, the preimage from its
`--out` -- bands 1 and 2 draw `hash 1  b867db87..edbc96cb` and
`preimage 1  b867db87..edbc96cb`: identical digest text, adjacent, differing by
one word, and with different consequences (band 1 assigns the digest and holds
NO material, so no plate is offered at Done and §5.3 item 4 will not list it
either; band 2 cuts a plate). The `hash:` row is annotated
`hash <i>  <first8>..<last8>  (in payload)`.

MEASURED at `sh2DisplaySize` in `composerPageLines`' 411 px band, on the longest
two-digit form: **41 characters, 343 px, ONE line.** The longer wording the
round-0 finding suggested, `(preimage in payload)`, is 50 characters and **348 px
over two lines**, so it is not taken -- a two-line row in a picker band is what
`composerPickScreenMaxRows` budgets against.

**AN UNDERIVED `phrase:` RECORD CANNOT BE COUNTED HERE, and that is a property of
this section rather than an omission** (plan round, gate fix F5). Its digest is
not knowable without the KDF that lazy derivation forbids at row-build time. So
the annotation is EXACT for a preimage record, and becomes true for a `phrase:`
record the moment that record is derived -- `composerHashEdit` rebuilds the row
set on every pass of its loop (`gui/composer_hash.go:184-224`), so the `hash:`
row gains its `(in payload)` on the next draw. Nothing is derived to decide
whether to draw an annotation.

`taking` (`gui/composer_hash.go:194`), which fires the §8i rule modal, extends to
the two new bands. `composerPickScreenMaxRows` (`gui/composer_paged.go:243`) is
checked against the longest row set.

### §5.2 The Hashlock plates flow

A fourth route on the Wallet Policy door, after "Scan cards", "From payload" and
"Build a new policy" (`composerDoorFlow`, `gui/composer_door.go:98-119`). It is
**conditional**, gated by `composerDoorHasPreimage(s *syswSession)` -- the same
shape and the same argument as `composerDoorHasConsumablePolicy`
(`gui/composer_door.go:86-90`; the first draft's `:93-97` is
`composerDoorFlow`'s own comment and is corrected here): offered only when the
loaded payload holds at least one `ClassPreimage` or `ClassPhrase` record. A door
row that names a route it cannot take is the F-437 defect the door exists to
remove. **It takes the SESSION and not the `Context`**, which is what makes the
predicate testable without a running flow (plan round, gate fix F15).

**AND THE DOOR'S ONE CALLER MUST DISPATCH IT -- NORMATIVE, same fix.** The door
only REPORTS a route; `gui/wallet_policy.go:46-58` is the loop that takes one,
and a fourth route nothing dispatches is the F-437 defect one level in -- a row
the operator can highlight and select that returns them to the door. The route
is a member of that loop, so Back from the flow lands on the door.

The flow: list the payload's preimage and phrase records; the operator picks;
per record it offers form and QR (§5.3 item 3); then it cuts. It builds no
composition and reads nothing from `composerState`.

**A `phrase:` record DERIVES ON PICK, and the result lives for the flow --
NORMATIVE** (round-0 fidelity I-7 = journey I-2). §6.3 makes the digest row
unconditional, and for a `phrase:` record there is no digest in the payload: it
is `sha256(PBKDF2-HMAC-SHA256(phrase, "ms-hashlock-v1", 100000, 32))`
(`hashlock/hashlock.go:21,24,27`), about 10 s at the measured 9,715
iterations/s. So:

1. **The LIST draws without deriving.** A phrase row reads
   `phrase record <i> (derive to see the digest)`, exactly as §5.1's band 3
   does. Deriving to draw the list is the *"three records would be a 30 s stall
   before a list could be drawn"* §5.1 rejects, and this flow has more phrase
   records than the composer typically does, not fewer.
2. **Picking one derives ONCE, behind the `Deriving` countdown** (H2 §4.4) --
   never silently, because a 10 s stall with no screen is a hang to the operator.
3. **The result lives in a LOCAL of the flow**, keyed by record index, scrubbed
   on return. Not `composerState`: §2.3's "holds nothing" stays true, and a
   redraw, a page or a Back must not re-run the KDF.
4. **The locator's `hash` row is printed from that result**, so it is never
   blank. A phrase-form plate with no locator at all -- a bearer phrase in plain
   text with no digest, no path, and no payload position -- is the worst artifact
   this stage can cut, and it is what a literal reading of the first draft
   produced.
   **THE PROPERTY IS THE DIGEST, NOT THE PRESENCE OF A ROW** (plan round 0,
   journey C-1). `hashlockPlateLocator` always appends the row, so "never blank"
   is true by construction and a plate built BEFORE the derive is equally
   never-blank -- it reads `hash  00000000..00000000`, matches no `hash:` record
   and names nothing. What §11.5 asserts is therefore that the digest on the
   plate EQUALS the one the host derives for that phrase, on a PHRASE record and
   THROUGH the flow, so the derive-then-locate ORDER is what is under test.

**An abort in THIS flow needs no §8.4 warning, and must not borrow one**
(round-0 journey N-1). §8.4's arms say the phrase *"dies with this composition"*
and speak about a run that also cuts policy plates. Neither is true here: this
flow builds no composition, and the material it cuts stays in the payload, in
flash. Telling the operator a secret is gone when it is still in flash is false
in the dangerous direction. Reusing the arm is the obvious implementation, which
is why it is refused by name.

**The door's lead names these records** (round-0 journey M-5). `composerDoorCounts`
(`gui/composer_door.go:37-52`) counts `ClassKey`, `ClassMnemonic`/
`ClassCodex32Secret` and `ClassUnknown` and nothing else, so `composerDoorLines`'
`default` arm draws `composerCopyNoKeys()` -- a lead saying there are no keys,
above a door offering the Hashlock plates route for exactly that payload. It
gains a preimage/phrase count so the lead and the routes agree. This is the
screen that tells the operator the tap worked, and §12 item 3's payload is the
one that reads wrongest.

**Its locator header** (§6.3) prints the digest always, the matching `hash:`
record's position when one matches, and the `mk1 stub` **only when it can compute
one from an md1 record in the same payload** -- no payload record carries an id
(`sysw/record.go:14-21`, `sysw/composer_records.go:24-31`), so an md1 in the same
payload is the only source, and the field is omitted otherwise.

### §5.3 The Done review -- a PICK step, then the `Plates To Cut` census

**The per-plate decisions do NOT live on `confirmReviewScreen`, and the first
draft's claim that they did was unimplementable** (round-0 fidelity C-4 =
journey I-1). `confirmReviewScreen` (`gui/multisig_build.go:1895`) is paged and
READ-ONLY, and every input the machine has is already bound:
`backBtn = Button1`, `contBtn = Button3 + Center`, `pageBtn = Button2`. Its body
lines are `widget.Labelw` ops offset into the frame, **not `Clickable`s**, so
they are not touch targets -- and the SH2's only production input is the ft6x36
panel. W-2 measured exactly this failure once already: 205 taps that moved
nothing, on a screen whose rows were not targets. Three per-row interactions
(pick form, toggle QR, decline) had no control to attach to. `confirmReviewScreen`
is also SHARED -- `buildReviewFlow` draws the Policy Review through it -- so
rewriting it into a picker changes a screen the build path depends on.

So the review is TWO steps, both inside `composerEngraveStep`
(`gui/composer_flow.go:335-394`), in this order:

**(A) The per-plate PICK step**, one `composerPickScreen`
(`gui/composer_paged.go:278`) per held digest, before the census. That primitive
already has what this needs and nothing else does: a tap on a row selects it,
Button3 takes the highlighted row, Button1 declines, Button2 pages, and
`composerPickScreenMaxRows = 24` (`:243`) bounds the hit areas -- **but the page,
not that constant, is what binds** (§5.1's F6 measurement): the label is 23 px and
the ROW PITCH is 29 px. Its rows, MEASURED at `sh2DisplaySize` in the 411 px
band, each ONE line:

| row | chars | px |
| --- | --- | --- |
| `preimage string` | 15 | 128 |
| `phrase + method` | 15 | 138 |
| `phrase + method + QR` | 20 | 180 |
| `do not cut this preimage` | 24 | 196 |

The QR rows are offered only when the device holds the phrase (decision 1), and
taking one fires §8.5's warning. **Back contract, CORRECTED** (plan round 0,
fidelity I-7 = journey I-5): Button1 on this screen is `do not cut` for the
highlighted plate, matching `composerPickScreen`'s shipped decline arm, and
**backing out of the STEP is NOT offered**. One button cannot do both, and the
first draft asserted both -- a control the implementation deliberately does not
build, recorded until now only inside a Go comment. MEASURED:
`composerPickScreen` returns `(0, false)` on Button1, `composerPreimagePlatePick`
maps `!ok` to `hashlockPlateDecline`, and neither it nor `composerPreimagePlateStep`
has a failure return at all. §2.2 item 4 keeps the composition intact through
`composerFlow`'s own loop, and a Back that unwound the whole step would leave the
operator no way to answer the question for the remaining digests. **The exit is
the CENSUS's Button1** one screen later, which returns false from
`composerEngraveStep` and sends `composerFlow` round its loop with the state
intact. Reaching the pick step therefore commits the operator to answering for
every held digest: an operator who realises here that they chose the wrong
engrave mode -- both mode picks run BEFORE step (A) -- pays for every pick
decision already made. That is the accepted cost of not inventing a third return
state on a screen with no spare input, and it is stated here rather than left in
a comment.

**THE MASKED LEAD IS TWO LINES, AND THE NUMBER IS NORMATIVE** (plan round, gate
fix F13). `composerPickScreen` draws the lead as a per-page header THROUGH
`composerPageLines`, so every line the lead spends is a ROW lost from page 1.
MEASURED at `sh2DisplaySize`: a two-line lead -- `hash  <first8>..<last8>`
optionally with `   path <n>`, then `phrase: <n> characters   method: <m>` --
puts **all four rows on page 1**; a four-line lead, with the digest, the path,
the character count and the method each on their own line, leaves **three**, and
the row it displaces to page 2 is `do not cut this preimage`, the row an operator
reaches for to UNDO. §11.5 pins the four-on-page-1 count.

**THE PLATE ORDER IS DETERMINISTIC -- NORMATIVE** (plan round, gate fix F14).
The list of held plates may NOT be produced by ranging `hashlockHeld`: Go
randomises map iteration, so the same composition would list its plates in a
different order on every frame, on the screen whose job is to be read against
the bench. The order is: the hashed PATHS first, in path order, deduplicated by
digest; then every held digest no path carries, in digest order. §11.5 pins it
over repeated builds of the same state.

**(B) The census**, drawn at `gui/composer_flow.go:389-390` -- where
`confirmReviewScreen(ctx, th, "Plates To Cut", composerCensusLines(...))` stands
today -- which REPORTS the choices already made and stays read-only: its
contract, honoured rather than stretched.

**BUT THE CALL BECOMES `composerReadScreen`, NOT `confirmReviewScreen` --
NORMATIVE** (plan round, gate fix F10). Item 2 below requires the block to be
drawn in `composerPageLines`' 411 px band, and `confirmReviewScreen`
(`gui/multisig_build.go:1895`) wraps at 464 px and centres on the whole panel, so
naming it here contradicted the requirement two paragraphs down.
`composerReadScreen` (`gui/composer_paged.go:173`) has `confirmReviewScreen`'s
exact contract -- Button1 backs, Button2 pages, Button3 continues -- inside the
band that keeps ink off the buttons. **The two are indistinguishable from
outside**: both draw a paged read-only body of the same shape, `ExtractText`
collects a glyph's rune wherever it lands, and the real frame draws the nav
buttons over it -- so §11.5's row is an AST assertion on `composerEngraveStep`'s
call set, not a raster probe.

Normative:

1. **The preimage plate is not a `bundleCard` and does not enter `plan`.** It
   follows S6b's passphrase-plate precedent (`gui/multisig_build_census.go:88-93`:
   entering `plan` or the inventory *"would tell a reader it travels WITH the
   set"*), so `buildPlateCensusLines`' count and its *"a set is only a backup
   when all of it exists"* claim (`:66-72`) are about the policy plates alone
   and stay byte-unchanged.
2. **Its own census block**, appended by `composerCensusLines`
   (`gui/composer_census.go:86`): a heading, one row per plate, and the
   apart-storage line (§8.3). **Its signature must change and the first draft did
   not say so** (round-0 fidelity N-3): it is
   `composerCensusLines(params engrave.Params, cards []bundleCard)`, which can see
   neither the plate decisions of step (A) nor `hashlockHeld`. It gains the
   accepted-plate list as a third parameter -- a value, computed by step (A) --
   rather than reading state, so the census reports a decision instead of
   recomputing one.

   **These rows are drawn through `composerPageLines`' 411 px band, not
   `confirmReviewScreen`'s own wrap** (round-0 fidelity M-7). MEASURED at
   `sh2DisplaySize`: the panel is 480 px, `confirmReviewScreen` wraps at
   `dims.X - 2*8 = 464` px and centres on the whole panel, the navigation column
   starts at **427 px**, so any row wider than **374 px** has its right edge
   under a button. The SHIPPED completeness line (*"Each plate takes minutes to
   cut ... a set is only a backup when all of it exists"*) is **459 px**, which
   is why this is named as a pre-existing property rather than something H6
   invents. H6 still may not add five more rows to it: this is the W-3 class the
   composer's paged screens were rebuilt to remove, and `composerPageLines` is
   the surface that already solves it. The census also joins
   `composerPagedScreens` (`gui/composer_paged_geometry_test.go:141-142`) so the
   shipped W-3 gate covers every page of it.

   **THE FIVE NEW ROWS' WIDTHS ARE DIGEST-DEPENDENT AND ARE NOT PINNED AS
   LITERALS** (plan round, gate fix F11). Two of them carry a
   `first8..last8` whose glyph widths differ per digest, so the first draft's
   405 / 423 / 441 / 447 / 448 px are not reproducible as written and are
   WITHDRAWN. The normative property is digest-INDEPENDENT: **every new row is
   over the 374 px threshold, and none of them draws under a button in the
   411 px band.** On the gate's fixture they measure, at the 464 px wrap,
   419 / 385 / 434 / 442 / 441 px -- all over 374 -- and in the band
   387 / 385 / 404 / 383 / 387 px, none under the column. §11.5 asserts the
   property and logs the fixture's values rather than asserting them.
3. **Form and QR are chosen PER PLATE**, on step (A), because two preimages in
   one policy may want different forms. The QR is offered only on the phrase form
   (decision 1) and only when the device holds the phrase. The census (B) then
   NAMES the choice per row (§8.3), so the operator confirms what they picked.
4. **A retained preimage no current path carries is LISTED, never cut**, for BOTH
   provenances (ruling B; journey Q4): a row
   `preimage <first8>..<last8>: not on any path, will not be cut`. Nothing is
   deleted from `hashlockHeld` to achieve this (§2.2 item 2).
5. **A declined plate is dropped from the plan and NAMED in the census** as
   `declined, will not be cut` -- declined on step (A), reported on step (B).
   Declining does not abort the run: a preimage
   plate is not part of the policy set, so `bundleEngrave`'s set-level
   "a partial bundle can't be used" reasoning (`gui/bundle_flow.go:625-631`) does
   not reach it.
6. **No cap on the number of preimage plates.** One per path that carries held
   material, one per payload record in the Hashlock plates flow.
7. **A phrase is MASKED on screen, and NOT revealed here** (journey Q13). The
   review and the Hashlock plates flow print `phrase: <n> characters` and the
   method, and show no characters at all. **The first draft called this "the
   affordance the operator already met on the phrase keyboard ... revealed only
   while the toggle is held", and both halves are wrong** (round-0 journey I-1):
   the shipped affordance is `{label: "show", action: ppReveal}`
   (`gui/passphrase_keyboard.go:141`), a KEY ON A KEYBOARD GRID, and it LATCHES
   (`k.revealed = !k.revealed`, `:221`) rather than being held. Neither a pick
   screen nor a paged confirm screen has a key grid to put it on. Item 7 itself
   argues the right answer: *the plate preview is the one screen that must show
   the phrase, and the review is not* -- so the review does not show it, and no
   new control is invented for a screen with no spare input. This applies to BOTH
   provenances: a payload-delivered phrase is a secret the operator never typed,
   may not own, and did not ask to see, in whatever room the machine lives in.
   Secret-handling and therefore non-gating either way.

### §5.4 Cut order -- preimage plates FIRST

`composerEngraveStep` cuts every accepted preimage plate BEFORE calling
`bundleEngrave`, in its own loop:

```go
for _, pl := range plates {           // §6, one Plate per accepted preimage
	if !NewEngraveScreen(ctx, pl).Engrave(ctx, &engraveTheme) {
		return composerAbortNoPreimage(ctx, th, st)   // §8.4
	}
}
return bundleEngrave(ctx, th, "Wallet Policy", cards, markTitle, "") == bundleEngraveDone
```

`NewEngraveScreen` is `gui/gui.go:3296`; the plate is built by §6's own layout
function and `toPlate` (`gui/gui.go:3620`), on the `ppBuildPlate` pattern
(`gui/passphrase_flow.go:570-588`). Ordering removes the window in which the md1
plates exist and the preimage does not. The ms1 secret cards keep their place at
the head of `cards` (`gui/composer_flow.go:381`), so the whole run is
secrets-then-policy.

**Ordering opens a SECOND window, and both get copy** (round-0 fidelity I-1 =
journey I-7). The order creates two distinct aborts, not one:

| window | what the operator holds | arm |
| --- | --- | --- |
| a preimage plate's own `Engrave` returns false, none cut yet | no plate of any kind; the phrase held only by this composition | §8.4a |
| at least one preimage plate cut, then the run ends | a BEARER plate on the bench and no usable policy set | §8.4b |

**THE MECHANISM IS A COUNTER, AND WITHOUT IT §8.4b CAN NEVER FIRE -- NORMATIVE**
(plan round, gate fix F9). A loop that returns `composerAbortNoPreimage`
unconditionally on a failed engrave makes the second arm unreachable, and a
`bundleEngrave` whose result is never read back makes it unreachable a second
way. So `composerEngraveStep` keeps a count of plates actually cut: inside the
loop, a failure with `cut > 0` draws §8.4b and `cut == 0` draws §8.4a; after
`bundleEngrave`, `!done && cut > 0` draws §8.4b. Both arms are then reachable and
§11.5 drives each of them, including the row that neither fires on a completed
run.

The second is the one the new order creates -- before §5.4 the preimage plate did
not exist, and this decision is what puts it on the bench first. Its danger is
specific: `bundleEngrave`'s existing set-level copy tells the operator a partial
bundle cannot be used, whose natural response is to run the composition again;
§5.3 item 6 sets no cap, so the second run cuts a SECOND bearer plate for the
same secret, one of which the operator has no record of, while §8.3's own line
tells them to store them apart from each other. §8.4b is what stops that.

---

## §6. The plate -- a dedicated layout

### §6.1 The type

A new `backup.Hashlock`, on the `backup.Passphrase` pattern
(`backup/passphrase.go:23-49`) and **never** through `validateMdmkStrings`
(`gui/gui.go:2626-2648`), whose single-string arm offers `TEXT + QR` / `TEXT
ONLY` / `QR ONLY` and QR-encodes THAT STRING -- which decision 1 forbids.

```go
type Hashlock struct {
	// Form selects the band text and the body (§6.2).
	Form HashlockForm // HashlockString | HashlockPhrase
	// MS1 is the kind-0x03 plate string, for HashlockString. Engraved VERBATIM.
	MS1 string
	// Phrase and Method are for HashlockPhrase. Phrase is engraved VERBATIM,
	// with every space rendered as backup.SpaceMark; Method is §8.6's line.
	Phrase string
	Method string
	// Locator rows, pre-formatted by the caller (§6.3). backup takes no
	// dependency on md or hashlock, exactly as Passphrase takes none.
	Locator []string
	// QR is opt-in and legal only on HashlockPhrase (§6.4).
	QR   bool
	Font *vector.Face
}
```

**The string form is cut LOWERCASE and UNGROUPED, and "VERBATIM" means exactly
that** (round-0 journey M-1). The fork's shipped ms1 secret plate does the
opposite -- `EngraveSeedString` upper-cases (`strings.ToUpper`) and groups in
tens, and carries a QR -- and `ms hashlock`'s own engraving card groups too
(`render_grouped`, `crates/ms-cli/src/cmd/hashlock.rs:340`). So the default plate
this stage cuts is the only one of the three renderings of the same 75 characters
that is a single unbroken lowercase run with no machine-readable copy, and that is
a decision rather than an oversight:

- MEASURED: at 6.0 mm the plate holds 19 characters per line. The raw string is
  75 characters = **4 rows**; grouped in tens it is 82 characters and, because a
  10-character group plus a separator plus the next group is 21 and will not fit,
  it takes **8 rows** -- +24.0 mm against a 65 mm budget the string form uses
  60.0 mm of. **Grouping does not fit at 6.0 mm.** It fits at 5.0 mm (23
  characters per line holds two groups, so grouped and raw are both 4 rows), which
  would move §6.5's string row down a rung and change every golden.
- It is not a data-loss question either way: bech32 carries a checksum and the
  fork ships BCH correction (`codex32.Correct`), so a mistranscription is refused,
  never silently wrong. The cost of the ungrouped form is re-read effort, once,
  into a tool that checksums it.
- The QR is declined for this form by ruling A2, so grouping cannot be traded
  against a machine-readable copy here.

The plate is therefore cut at 6.0 mm, ungrouped, lowercase. §11.4's goldens pin
that rendering so "VERBATIM" cannot drift into "as the sibling does it".

`SpaceMark` (`backup/passphrase.go:21`) and its legend
(`passphraseLegend`, `:168`) are reused unchanged on the phrase form: *"one space
and two look identical ... while 'hunter2 ' is a different wallet from
'hunter2'"* is true of a hashlock phrase for the same reason -- `Correct Horse`
and `correct horse` derive different preimages, and H2 §2 forbids every
normaliser by name. The legend row is drawn only when the phrase contains a
space.

### §6.2 The bands -- form-specific (ruling A4)

| form | Title (plate row 0) | Footer (last plate row) |
| --- | --- | --- |
| string | `HASHLOCK PREIMAGE` | `NOT A SEED` |
| phrase | `HASHLOCK PHRASE` | `NOT A SEED` |

MEASURED lengths: 17, 15, 10 -- all inside `MaxTitleLen = 18`
(`backup/backup.go:71`), the cap `TestTitleCapFitsAtEveryRung`
(`backup/freetext_test.go:109`) proves clears the screw holes at 6.0 mm by
0.620 mm. Both are engraved VERBATIM, never through `TitleString`, which
upper-cases and truncates (`TitleString`, `backup/backup.go:111`).

**The phrase form's method line is the first body row, directly beneath the
title band** (ruling A4). It is a BODY row, not a band row.

### §6.3 The locator header -- body rows, not band rows (ruling A5)

Rows, in order, immediately after the method line (phrase form) or the title
(string form):

```
path <n>                            (composer-native only)
hash  <first8>..<last8>             (always)
mk1 stub (policy): <8 hex>          (when every slot is seated)
mk1 stub (template): <8 hex>        (otherwise, and when no policy id exists)
matches hash <i> in the payload     (Hashlock plates flow, when one matches)
```

The stub labels are `gui/composer_stub.go:53-70`'s literal labels, so the plate
and the screen the operator copied into their notebook use the same words. A
key-less or partially seated composition has no Policy-ID
(`gui/composer_engrave.go:80`: *"no id yet"*; `gui/composer_stub.go:56-72`
adds the keyed pair only when keyed chunks exist), which is why the template stub
is the fallback rather than an empty field.

**They are body rows because a band holds at most TWO LINES and this stage has
already spent both** -- not because they are too wide (round-0 fidelity I-2,
M-2). The first draft justified them on width and the justification is
measurably false. MEASURED at 3.0 mm, `constant.Font`, W advance 600 against
`Metrics{Ascent:800, Height:900}`: the advance is exactly 12,800 units =
2.0000 mm, so the passphrase plate's 64 mm band ceiling
(`backup/passphrase_test.go:521`, 409,600 units) holds **32 characters** at
3.0 mm, 25 at 3.8 mm and 16 at 6.0 mm. Against that cap the locator rows are:

| row | chars |
| --- | --- |
| `path <n>` | 6 |
| `hash  <first8>..<last8>` | 24 |
| `mk1 stub (policy): <8 hex>` | 27 |
| `mk1 stub (template): <8 hex>` | 29 |
| `matches hash <i> in the payload` | 30 |

**The longest is 30, and every one of them fits a band at 3.0 mm.** The
33-character `path 2   hash  b867db87..edbc96cb` the first draft cited is a
CONCATENATION that appears nowhere in the row list above, which puts `path <n>`
and `hash ...` on separate rows.

The real constraint is the LINE COUNT. `passphraseLayoutFor`'s own comment
records it (`backup/passphrase.go:250-253`): *"At most two lines fit -- a band
offers innerMargin 10 - outerMargin 3 = 7mm, and three 3mm lines need 9mm and run
off the plate edge (spec 4.3)."* §6.2 already spends the top band on the title
and the bottom band on the space legend plus `NOT A SEED`, so there is no room
for up to four locator rows at any width. The body wraps at the full 79 mm width
instead (39 characters at 3.0 mm).

### §6.4 The QR -- phrase form only, BELOW the text (rulings A2, A3)

- **Only the phrase form carries a QR.** The string form is text-only; the ruling
  declines the brainstorm's B5 (a QR of the ms1 string), so a preimage-string
  plate has no machine-readable copy by design.
- **The QR encodes §8.6's text**, never the ms1 string (decision 1).
- **It is built with `engrave.ConstantQR`** (`engrave/engrave.go:418`), never
  `engrave.QR` (`:277`): the latter engraves in a content-dependent pattern and
  *"would leak the secret through timing"* (`backup/passphrase.go:112-114`). The
  phrase is a spend secret and the rule is the passphrase plate's, unchanged.
- **It is stacked BELOW the text**, as `passphraseLayoutFor` stacks its own
  (`backup/passphrase.go:283-292`, `l.envY = l.textY + l.blockH + gap` at `:289`),
  with `gap = 2 mm` (`passphraseQRGap`, `:74`).
- **Envelope 53 modules, scale 2.** The size is VARIABLE with the phrase length,
  so the layout reserves the worst case and centres the actual code inside it,
  as `passphraseQREnvelope = 37` does (`:70-72`).

### §6.5 Geometry -- MEASURED, and the fit is tight

All at `sh2.Params()` (`internal/sh2/params.go:43-53`: `Millimeter = 6400`,
`StrokeWidth = 1920`), `constant.Font`, plate 85 mm, `outerMargin = 3`,
`innerMargin = 10`.

| rung | chars/line (79 mm) | lines/plate | advance |
| --- | --- | --- | --- |
| 6.0 mm | 19 | 13 | 4.000 mm |
| 5.0 mm | 23 | 15 | 3.333 mm |
| 4.4 mm | 26 | 17 | 2.933 mm |
| 3.8 mm | 31 | 20 | 2.533 mm |
| 3.4 mm | 34 | 23 | 2.267 mm |
| 3.0 mm | 39 | 26 | 2.000 mm |

**The advance cell at 5.0 mm was 3.435 mm in the first draft and does not
reproduce** (round-0 fidelity M-3 = tests I-1). `fixedCharWidth(constant.Font,
F(5.0)) = 600 * 32000 / 900 = 21,333` units = **3.3333 mm** -- the only value
consistent with the linear pattern every other rung satisfies
(`advance = 2.0 mm * rung/3.0`) and with this table's own 5.0 mm row below
(80.0 mm / 3.3333 = 24.0 exactly; / 3.435 = 23.28, not a whole line count), so
the wrong number never reached a verdict. Every other cell reproduces exactly.

**The body wraps at exactly `charsPerLine` CHARACTERS, never on word boundaries**
(round-0 fidelity M-4), which `passphraseLayoutFor` already does
(`backup/passphrase.go:275`: `l.rows = (len(l.glyphs)+l.rowLen-1)/l.rowLen`). It
is forced rather than chosen: a 100-character phrase and a 75-character ms1
string are SINGLE TOKENS, so a word wrapper has nothing to break them on. Every
row count in the table below is `ceil(len / charsPerLine)`. The consequence worth
stating is that an 8-hex stub splits mid-token at the narrow rungs; that is
accepted, because the alternative is a wrapper that cannot lay out the plate's
main content at all.

The vertical budget for the centred group is **65 mm** (85 − 2 × `innerMargin`),
the span `passphraseLayoutFor` centres inside while the bands hold the title and
footer.

**THE BODY-ROW ORDER IS ONE ORDER, AND IT IS §§6.2's AND 6.3's** (plan round,
finding 8). The first draft of this table listed the worst case as *"header ... a
blank, the ... method line, a blank, and a 100-character phrase"* -- locator
FIRST -- while §6.2 makes the method line *"the first body row, directly beneath
the title band"* and §6.3 puts the locator *"immediately after the method
line"*. The normative sections win, and this table is corrected to match them so
there is nothing left to reconcile. MEASURED on the gated layout, the phrase
form's worst case at 3.0 mm draws, in this order:

| # | rows at 3.0 mm | what |
| --- | --- | --- |
| 1 | 2 | the 73-character hardened method line |
| 2 | 1 | a blank |
| 3 | 3 | `path 2`, `hash  <first8>..<last8>`, `mk1 stub (template): <8 hex>` |
| 4 | 1 | a blank |
| 5 | 3 | the 100-character phrase |

**2 + 1 + 3 + 1 + 3 = 10 rows = 30.00 mm.** The total is 10 either way, so **no
measurement in the table below moves** -- which is why this is a consistency fix
and not a re-measurement. The string form is the same shape without rows 1 and 2.

Worst-case bodies -- the 73-character hardened method line, a blank, header
`path 2` + `hash  <first8>..<last8>` + `mk1 stub (template): <8 hex>`, a blank,
and a 100-character phrase; or the same header, a blank and the 75-character ms1
string:

| form | rung | text | + gap + QR | total | 65 mm |
| --- | --- | --- | --- | --- | --- |
| phrase + QR (scale 2) | 3.0 mm | 30.0 mm | 2 + 31.80 | **63.80 mm** | **FITS, 1.20 mm spare** |
| phrase + QR (scale 2) | 3.4 mm | 37.4 mm | 2 + 31.80 | 71.20 mm | OVER |
| phrase + QR (scale 3) | 3.0 mm | 30.0 mm | 2 + 47.70 | 79.70 mm | OVER |
| phrase, no QR | 4.4 mm | 57.2 mm | — | 57.2 mm | FITS |
| phrase, no QR | 5.0 mm | 80.0 mm | — | 80.0 mm | OVER |
| string, no QR | 6.0 mm | 60.0 mm | — | 60.0 mm | FITS |

**Three normative consequences.**

1. **The QR scale is 2, not the passphrase plate's 3 -- and the constant-time
   engraver must LEARN scale 2 for that to be possible at all** (round-0 fidelity
   C-2). MEASURED: at 53 modules scale 3 is 47.70 mm and does not fit at any rung
   (3.0 mm gives 79.70 mm against a 65 mm budget); scale 2 is 31.80 mm. The module
   pitch matches the free-text plate's (`freeTextQRScale = 2`, `backup/fit.go:16-19`:
   *"0.6mm modules against the 0.9mm every other plate uses"*) but NOT its code
   path. `ConstantQRCmd.Engrave` renders every module through `engraveModule`
   (`engrave/engrave.go:689-709`), whose `switch scale` has only `case 3` and
   `case 4` and a `default: panic("unsupported module scale")` -- so as the first
   draft stood, the one configuration the plate fits at PANICS. §7.3 makes the
   `case 2` arm a normative deliverable; **this whole table depends on it.**

   Re-fitting at a supported scale was considered and is not available: scale 3
   needs the text block down to 5 rows at 3.0 mm, and the worst-case phrase (3
   rows) plus the method line (2 rows) is already 5 with zero locator rows and
   zero blanks -- and the locator is ruling A5. There is no rung at which the
   phrase form carries a scale-3 QR.
2. **The worst-case phrase-plus-QR plate fits at exactly one rung, with
   1.20 mm to spare.** The layout AUTO-FITS down `backup.FontSizes`
   (`backup/backup.go:83`) and REFUSES at the bottom rung rather than drawing
   what it cannot lay out, as `EngraveText` does
   (`backup/backup.go:388-400`) and as `toPlate` enforces. The refusal names the
   measured ceiling.
3. **The method line is the largest single term and must not grow.** At 73
   characters it is 2 lines at 3.0 mm; a 79th character would make it 3 and put
   the worst case 3.0 mm over budget. §11 pins this with a mutation.

---

## §7. Raising the constant-time QR encoder to v9 -- NORMATIVE (ruling A1)

`ConstantQR` refuses anything over 37 modules today
(`engrave/engrave.go:418-426`), because `bitmapForQRStatic` (`:394-414`)
tabulates 21/25/29/33/37 and a larger version would reach its
`panic("unsupported qr code version")`. Its own comment states the bound's origin
and the rule for changing it: *"ECC-L caps at 106 bytes and the passphrase caps
at 100 (spec O6) ... Raise both together or not at all."*

### §7.1 The ceiling is v9 = 53 modules

MEASURED with the fork's own encoder, ECC-L byte mode:

| payload | bytes | modules | version | `ConstantQR` today |
| --- | --- | --- | --- | --- |
| §8.6 text, hardened, 100-char phrase (**worst case**) | 194 | 53 | **9** | refuses |
| §8.6 text, sha256, 100-char phrase | 135 | 45 | 7 | refuses |
| §8.6 text, hardened, 28-char anchor phrase | 122 | 41 | 6 | refuses |
| §8.6 text, sha256, 1-char phrase | 36 | 29 | 3 | accepts |

ECC-L thresholds, measured: ≥1 → 21 (v1), ≥18 → 25, ≥33 → 29, ≥54 → 33,
≥79 → 37 (v5, today's cap, 106 bytes), ≥107 → 41 (v6), ≥135 → 45 (v7),
≥155 → 49 (v8), **≥193 → 53 (v9)**, ≥231 → 57 (v10).

**The ruling names "v7 = 53 modules"; that is v9.** QR side = 4 × version + 17,
so v7 is 45 modules and v9 is 53. The 194-byte worst case needs **v9**, and the
new ceiling is `dim > 53`.

### §7.2 The alignment-pattern change

Derived EMPIRICALLY from the encoder's own bitmap by testing the 5×5 ring shape
at every candidate centre (§11 keeps the derivation as a test, so the table
cannot be transcribed wrong):

| version | dim | rings | centres |
| --- | --- | --- | --- |
| 2 | 25 | 1 | (18,18) |
| 3 | 29 | 1 | (22,22) |
| 4 | 33 | 1 | (26,26) |
| 5 | 37 | 1 | (30,30) |
| **6** | **41** | **1** | **(34,34)** |
| **7** | **45** | **6** | (22,6) (6,22) (22,22) (38,22) (22,38) (38,38) |
| **8** | **49** | **6** | (24,6) (6,24) (24,24) (42,24) (24,42) (42,42) |
| **9** | **53** | **6** | (26,6) (6,26) (26,26) (46,26) (26,46) (46,46) |

`fillMarker` takes a TOP-LEFT, so each entry is `centre − 2`. Normative:

1. **v6 is free.** Its single ring sits at `(dim−9, dim−9)` = (32,32), which is
   exactly the formula `bitmapForQRStatic` already uses for 25/29/33/37; adding
   `41` to that case arm is the whole change.
2. **v7-v9 need the six-ring layout.** `bitmapForQRStatic` gains a per-version
   centre table for 45/49/53 (the three rows above, verbatim), each expanded to
   six top-lefts by dropping the three combinations the position markers occupy.
   The `default: panic` arm STAYS -- it is what keeps an unhandled version from
   being drawn as if it had no alignment patterns at all.
3. **`ConstantQR`'s bound becomes `dim > 53`**, and its comment pairs the
   ADMITTED version with **ITS OWN** capacity -- **(v9, 230 bytes)** against a
   194-byte §8.6 worst case, i.e. 36 bytes of headroom -- plus the same "raise
   both together" rule. **The first draft paired it with the PREVIOUS version's
   capacity** (*"192 bytes at the last full step below 194"*), which is v8's cap
   and reads as "the bound holds 192 and the content needs 194" -- already over,
   telling the next raiser the bound must move again for exactly the content it
   was raised for (plan round 0, fidelity M-1). MEASURED against the fork's own
   encoder and pinned by `TestECCLThresholdsAreWhatTheBudgetAssumes` (which now
   sweeps 1..240 bytes, not 79..240): the first byte count reaching dim 53 is
   **193** and the first reaching dim 57 is **231**.
4. **`constantTimeQRModules` GAINS AN ARM PER NEW VERSION, and without it every
   raised version refuses** (round-0 fidelity C-1 = tests I-3). The first draft
   cited that function only as one that "takes `dim`". Read in full
   (`engrave/engrave.go:349-374`), it tabulates 21/25/29/33/37 and returns **0**
   for everything else -- *"Not supported, return a low number to force error"* --
   and `nmod` is both the budget `ConstantQR` checks (`if len(modules) > nmod`)
   and the loop count `ConstantQRCmd.Engrave` runs (`for range nmod`). Applying
   items 1-3 literally and nothing else, MEASURED:

   ```
   constantTimeQRModules(41)=0  (45)=0  (49)=0  (53)=0
   v6 dim=41  ConstantQR err=too many dims 41 QR modules for constant time engraving
   v7 dim=45  ConstantQR err=too many dims 45 QR modules for constant time engraving
   v9 dim=53  ConstantQR err=too many dims 53 QR modules for constant time engraving
   ```

   So the headline deliverable -- a phrase plate carrying a QR -- is produced at
   none of the four versions the raise admits. The fork already says this is the
   shape of the work: `TestPassphraseQRFitsSupportedVersion`'s header
   (`engrave/engrave_test.go:696-707`) records that module COUNT varies with
   content at a fixed version (*"578..664 measured at dim 37"*), that the bound
   *"was derived by fuzzing, not by this test"*, and that a raise means
   *"constantTimeQRModules needs a v6 entry derived by fuzzing. Do NOT fall back
   to the non-constant-time engrave.QR for a secret."*

   **The number cannot be invented, because it is content-dependent**, and a
   budget between the observed min and max makes the plate cuttable for some
   phrases and refused for others at the SAME QR version -- an operator-visible,
   content-dependent failure on the funds path, which is the exact class
   `ConstantQR` exists to remove.

   **AND SAY WHICH DIMS §8.6 CONTENT REACHES, because it is not only the four
   this section raises** (plan round 0, fidelity I-5). §8.6 text is
   `21 + len(method) + len(phrase)` bytes: a `sha256` plate is `35 + len(phrase)`
   and a `hardened` one `94 + len(phrase)`, so over the 1..100-character phrase
   range this content reaches **29, 33, 37, 41, 45, 49 and 53 and nothing else** --
   1..18-character `sha256` phrases at dim 29, 19..43 at dim 33, and dim 37 taking
   both a 44..71 `sha256` phrase and a 1..12 `hardened` one. The three SHIPPED
   budgets it reaches (391 / 547 / 684) were derived from 18.5M
   **passphrase**-shaped executions, and this stage hands them a new content
   class, so the argument above applies to them verbatim and the in-suite row
   samples all seven (§11.3). MEASURED at 400 payloads per dim, no shipped entry
   moves and none needs to: headroom 44 / 69 / 67 at v3/v4/v5 and 56 / 95 / 65 /
   51 at v6..v9.

   **Deliverable: four fuzzing runs on the protocol the v5 entry records**
   (`engrave/engrave.go:363-371`: 18.5M executions, 32 min, converged with 24.9
   min of quiet), each recording observed max, sample count and the
   max/`dim`² ratio in the code comment as v5's does, and each buffered upward
   where the ratio falls below the trend, for the reason v5's comment gives
   verbatim: *"an underestimate produces content-dependent engrave-time failures
   on a permanent plate, an overestimate costs ~2% more engraving time."*

   **The fold ran a partial campaign and its numbers are a FLOOR, not the
   answer.** 216,000 §8.6-shaped payloads per dim, sampled across the phrase
   lengths that reach each version, in the fold's worktree:

   | dim | v | samples | observed max | max/dim² | `findPath` errors |
   | --- | --- | --- | --- | --- | --- |
   | 41 | 6 | 216,000 | 813 | 0.4836 | 0 |
   | 45 | 7 | 216,000 | 945 | 0.4667 | 0 |
   | 49 | 8 | 216,000 | 1161 | 0.4835 | 0 |
   | 53 | 9 | 216,000 | 1369 | 0.4874 | 0 |

   against the shipped ratios 0.3764 / 0.4176 / 0.4590 / 0.4977 / 0.4850 at
   21/25/29/33/37. Two things follow and both are normative:

   - **The campaign has NOT converged and a short one will underestimate.** An
     independent 4,000-payload sample during the same round saw maxima of 785 at
     dim 41 and 1281 at dim 53; 216,000 payloads found **813** and **1369**. The
     ceiling was still climbing by +28 and +88 between those sample counts. A
     plan-time entry that comes in BELOW the floor above is proof of an
     under-converged run, and that is a checkable gate rather than a hope.
   - **v7 is the suspect entry.** Its ratio, 0.4667, sits below its neighbours
     (0.4836, 0.4835, 0.4874); the trend-implied max at ~0.485 is about 982
     against an observed 945. That is precisely the signal the v5 comment used to
     justify a buffer of 20 rather than the historical 5, and v7's buffer is set
     the same way.
   - **Zero `findPath` failures in 864,000 payloads** -- no "QR modules spaced too
     far for constant time engraving" at any raised version. That is evidence the
     path-finder scales, not proof; §11.3 keeps it as a row.

   **THE CAMPAIGN RAN AT PLAN TIME AND THE FOUR ENTRIES ARE SETTLED** (plan
   round). Four 32-minute runs on 24 cores over §8.6-shaped payloads,
   43,458,059 payloads in total with ZERO `findPath` failures, are recorded with
   their sample counts, ratios and last-improvement samples in
   `design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md` Task 4 Step 3 and
   `design/agent-reports/hashlock-H6-plan-author-report.md` §2. Every observed
   maximum CLEARS the floor above -- **823 / 960 / 1179 / 1379** against
   813 / 945 / 1161 / 1369, by +10 / +15 / +18 / +10 -- which is the checkable
   gate this item asks for, and
   v7 is the entry whose ratio (0.4741 against a 0.4905 mean) and 2.8 minutes of
   quiet mark it as the under-converged one, so its buffer is widened to 53
   rather than left at 20. **MEASURED in the gated tree, the four arms return
   `constantTimeQRModules(41) = 843`, `(45) = 1013`, `(49) = 1199`,
   `(53) = 1399`, and the five shipped entries 171 / 266 / 391 / 547 / 684 are
   untouched.** §11.3 carries both the fuzzed row and the pin.

5. **The constant-time argument is re-earned, not inherited -- and the first
   draft's test could not earn it.** It asked for a test that *"the emitted move
   count is a function of `dim` ALONE -- identical for two different payloads of
   the same version"*. `Engrave` loops `for range nmod` and pads every move to
   `maxDur` via `DelayMove`, so the emitted count is a function of `dim` alone
   **for any value of `nmod`, correct or not** -- the test passes on a budget of
   0, on 700, and on a correct one. It is a regression guard, not a proof. The
   property that actually needs proving is the OTHER one: that `nmod`
   upper-bounds `len(modules)` over all payloads at that version. §11.3 states
   both, separately, and says which is which.
6. **Goldens.** One golden per newly-admitted version (v6, v7, v8, v9) on a
   fixed payload, at **scale 2**, in `backup/testdata/`, alongside the plate
   goldens of §11.

### §7.3 `engraveModule` gains a scale-2 arm -- NORMATIVE (round-0 fidelity C-2)

`engraveModule` (`engrave/engrave.go:689-709`) has `case 3`, `case 4` and
`default: panic("unsupported module scale")`. §6.5 puts the phrase plate's QR at
scale 2, which is the only scale it fits at, so the arm is a deliverable of this
stage rather than a nicety.

**Its geometric contract, and it is not "copy case 3 with smaller numbers".**
`centerOf` (`engrave/engrave.go:628-631`) is
`(p*scale + 1)*sw + sw/2`, which puts the centre of module `p` at
`p*scale*sw + 1.5*sw` -- correct for scale 3 (the cell centre) but **half a
stroke off-centre for scale 2**, exactly as it is for scale 4, whose arm
compensates with an asymmetric shape. So the scale-2 arm is asymmetric too: the
cell is `[p*2sw, p*2sw+2sw]`, the centre sits at `+1.5sw` inside it, so relative
to the centre the painted extent must be `[-1.5sw, +0.5sw]` and the PATH must run
`[-sw, 0]` on both axes -- a closed square on the corners `(-sw,-sw)`, `(0,-sw)`,
`(0,0)`, `(-sw,0)`.

**MEASURED, implemented literally in the fold's worktree:**

```
scale=2 p={0 0} cmds=5 ink x=[0,3840] want [0,3840] OK | y=[0,3840] want [0,3840] OK
scale=2 p={1 0} cmds=5 ink x=[3840,7680] want [3840,7680] OK
scale=2 p={5 7} cmds=5 ink x=[19200,23040] OK | y=[26880,30720] OK
scale=3 p={5 7} cmds=5 ink x=[28800,34560] OK | y=[40320,46080] OK   (control)
```

3,840 units = **0.6 mm**, the cell exactly, at every probed position, with no
overlap into a neighbour.

**It emits FIVE commands per module -- the same constant as `case 3` -- so the
constant-time argument survives**, and that is the property the arm has to carry:
`ConstantQRCmd.Engrave` pads every move to `maxDur` and runs `for range nmod`, so
a per-module command count that is constant in the CONTENT is what keeps the
toolpath content-independent. A scale-2 arm whose command count varied by module
would re-open the leak the whole section exists to close. §11.3 asserts the count.

### §7.4 The two halves are ONE deliverable

The version raise (§7.1-§7.2) and the scale-2 arm (§7.3) do not ship apart. With
the raise alone, `ConstantQR` returns a command for a 53-module code and
`Engrave` panics on the only scale it fits at; with the arm alone, `ConstantQR`
refuses the code before `Engrave` is reached. Both together, and only then, does
the phrase form's QR exist. §11.3's rows are written so that removing either half
reds.

---

## §8. Copy -- operator-facing strings

**ASCII only, and it is a HARD RULE with a mechanism, not house style**
(round-0 fidelity I-6 = tests I-2). `font/bitmap/bitmap.go:33` sets
`indexLen = unicode.MaxASCII` and `glyphFor` rejects `int(r) >= indexLen`, and the
consequence is not a dropped character: **an unrenderable rune blanks the ENTIRE
body of its frame**, while `ExtractText` still reports the text as present, so a
string assertion cannot see it. MEASURED on the same body at the same length,
only the marked glyph differing:

```
section-sign   the modal drew only 5004 ink pixels (floor 6000) -- near blank
em-dash        the modal drew only 5004 ink pixels (floor 6000) -- near blank
ascii-S        drawnFully=true drew=153 want=153
```

The first draft opened with "ASCII only" and then broke it in five of its own
bodies. **Every DEVICE body in this section is now ASCII**, and the rule is
enforced by §11.5's table rather than by care.

**Host lines are stderr, carry no panel budget, and are EXEMPT** -- they may
carry an em dash, and the shipped ones already do (`crates/me-cli/src/seal/record.rs:130-136`,
`main.rs:2805-2810`). Each body below is LABELLED with which it is:

- **HOST (stderr, exempt):** §8.1.1, §8.2.1, §8.2.2, §8.2.3, §8.2.4.
- **DEVICE (ASCII, measured):** §8.3, §8.4a, §8.4b, §8.5, §8.8, §9, §10.1.
- **§8.1.2 is a HOST body kept ASCII deliberately**, and measured on the device
  renderer anyway, because its content is the one refusal a future stage is most
  likely to surface on-device (§4.3's collision). It is exempt from the rule and
  written to it.

**A body may not claim a device measurement unless it has been measured**, which
is the specific failure §8.1.2 committed in the first draft.

Every DEVICE body below was measured with `assertModalBodyFits`
(`gui/modal_fits_test.go:202`), which renders the specific body and
binary-searches its headroom with `modalHeadroom` (`:183`), requiring at least
`modalBodyMargin = 80` normalised characters (`:52`).

### §8.1 Host refusals

**§8.1.1 (HOST, stderr).** The refusal `admit_check`'s new rule raises for a
well-formed preimage plate (id `hash`) or a `phrase:` record packed without the
flag. It reuses `crates/me-cli/src/main.rs:2805-2810`'s wording and adds decision
7's sentence:

> record {i} (records count from 0) is a hashlock PREIMAGE plate (kind 0x03),
> not a seed record; this payload did not ask for one. A preimage backs a
> hashlock spend path, not a wallet — keep it with the policy it unlocks, and do
> not re-encode it as entropy. Re-run with --pack-preimage if that is what you
> intend.

**The final sentence is emitted ONLY when the id is `hash`** (round-0 journey
I-4). For any other id the advice is affirmatively false -- `--pack-preimage`
will refuse the record on the next run (§4.3), and the operator is then sent to
`ms hashlock`, which refuses it a third time in a message written for a different
audience. Three refusals, and none of them says the one thing that is true and
actionable. So a wrong-id record gets §8.1.2 straight away, from the no-flag path
too, and sees ONE refusal.

**§8.1.2 (HOST, stderr).** The EXISTING `U::PreimagePlate` body
(`main.rs:2805-2810`), rewritten for the case it actually covers -- a kind-`0x03`
single whose id is outside `{entr, hash}` (§4.3's table; an `entr` mismatch
reaches the shipped `U::TagKindMismatch` text at `:2799-2804` and that text is
UNCHANGED):

> record {i} (records count from 0) is a kind-0x03 preimage payload whose
> 4-character id is not `hash`. A preimage plate is kind 0x03 under the id
> `hash` (SPEC_ms_hashlock rule 2), and --pack-preimage admits only that.
> Re-encode it with `ms hashlock` rather than editing the string. If this string
> is a 33-byte seed backup that happens to begin 0x03, it is not a preimage:
> roughly 1 in 256 of them look like this.

**The last sentence is the point** (round-0 journey I-4). §4.3 exists for exactly
one case, which `codex32/mspayload.go:78-92` states outright -- *"A plain BIP-93
33-byte seed that begins 0x03 is indistinguishable from a preimage plate ...
Roughly 1 in 256 of 33-byte seeds"* -- and the first draft's copy never named it,
so the operator most likely to hit this refusal was the one least told what it
meant.

**Its device-side twin is not a modal**: such a record is `ClassUnknown` and
inert, visible only in the door's not-understood count
(`gui/composer_door.go:41-56`).

**The first draft's measurement for this body is WITHDRAWN.** It claimed *"165
characters drawn, headroom 397 on `errorScreenBody` so that it can be surfaced
later without a re-measure"*, and the text as printed carried a `§`, so measured
it **blanks the frame** (5,004 ink pixels against a 6,000 floor) and draws
nothing. The ASCII body above measures **236 drawn / headroom 320** on
`errorScreenBody`, and with the collision sentence **340 drawn / headroom 223** --
both clear the 80-character margin, and both are host bodies measured on the
device renderer only so the text is ready if it is ever surfaced there.

### §8.2 Host warnings -- all four are HOST, stderr, and exempt from the ASCII rule

**§8.2.1, transit -- always, when a preimage or `phrase:` record is admitted:**

> me: WARNING — this payload carries a hashlock PREIMAGE. Anyone who holds the
> tag can read it, and for a key-less hashlock path the preimage alone spends
> the coins. Treat this payload as bearer material until it is on the machine
> and erased.

**§8.2.2, the flag with nothing to admit:**

> me: --pack-preimage was passed and this payload holds no preimage plate and no
> `phrase:` record. Nothing was admitted that would otherwise have been refused.

**§8.2.3, orphan digest -- TWO bodies, one per carrier** (round-0 journey I-3;
the two are distinguishable because the operator's next move differs):

> me: WARNING — record {i} (records count from 0) is a preimage whose digest
> {first8}..{last8} matches no `hash:` record in this payload. Nothing here
> tells the device which policy it unlocks, and the Hashlock plates flow will
> print the digest alone.

> me: WARNING — record {i} (records count from 0) is a hashlock phrase whose
> digest {first8}..{last8} matches no `hash:` record in this payload. Check the
> method selector and the text after the first comma — a space after the comma
> is part of the phrase and derives a different preimage.

**When the payload holds NO `hash:` record at all, both drop to a note rather
than a `WARNING —`** (round-0 journey M-6). §12 item 3's own acceptance path --
`ms hashlock --out X.txt` then `me sysw pack` -- produces exactly that payload,
because `ms hashlock` writes only the ms1 string to `--out` and prints `hash:`
to stdout, so the minimal correct journey would fire a WARNING every single time,
which is how a warning stops being read. The distinction the operator needs is
**incomplete** (no `hash:` records were packed; nothing is inconsistent) versus
**contradictory** (the payload holds `hash:` records and this digest matches none
of them), and only the second is a warning:

> me: note — this payload holds no `hash:` record, so nothing here says which
> policy the preimage unlocks. The Hashlock plates flow will print the digest
> alone.

**§8.2.4, sealed transit** (see §3.4's correction -- each clause is true of the
container it names):

> me: this payload is SEALED and holds a hashlock preimage, so the device needs
> this payload's passphrase before it can reach it. `me seal` — the Sealed
> Payload container — refuses a preimage plate outright; this one does not.

### §8.3 The census block (device rows, paged by `composerReadScreen`)

Heading, then one row per plate, then the apart-storage line:

> Plus {n} preimage plate(s), cut first and NOT part of this backup:

> path {n}  {first8}..{last8}  {phrase, hardened, QR | phrase, sha256 | preimage string}

The third field is the choice made on §5.3's step (A), reported here read-only.

> preimage {first8}..{last8}: not on any path, will not be cut

> preimage {first8}..{last8}: declined, will not be cut

> Keep each preimage plate apart from the policy plates and from the others.

MEASURED at `sh2DisplaySize`, every one of these rows is over
`confirmReviewScreen`'s 374 px centred-safe width -- as is the SHIPPED
completeness line at 459 px -- which is why §5.3 item 2 routes the block through
`composerPageLines`' 411 px band instead of adding five more rows to a
pre-existing W-3 overlap. **The individual widths are digest-dependent and the
first draft's 405-448 px are withdrawn**; §5.3 item 2 carries the property that
is pinned and the fixture values that are only logged (plan round, gate fix
F11).

**`plate(s)` is kept VERBATIM, against this file's own house style, and it is
filed rather than fixed.** `composerSlotWord` renders "slot @3" or "slots @3 and
@4" precisely so a refusal never reads "slots @3", and the heading above breaks
that rule. Changing spec copy while the shipped string is being diffed against
it puts the two out of step in the middle of a gate, so the wording is a records
follow-up owned by the stage's last task, not an edit made here.

The stand-alone notice form of the fourth row, for a review whose only entry is
an unused preimage, measured **107 drawn / headroom 455**:

> One preimage this composition holds is on no path of this policy. It will not
> be cut. Go back and set a path's hash to it, or leave it.

### §8.4 The abort arms -- BOTH on the composer's own screens

**`bundleAbortWarningText` is UNCHANGED, and the first draft's placement was
dead code** (round-0 fidelity I-1). It said the clause was *"prepended"* to
`bundleAbortWarningText` (`gui/bundle_flow.go:780-792`) and *"fires when the run
ends before every accepted preimage plate is cut"*. That text is reachable only
from `bundleAbortWarning` at `gui/bundle_flow.go:625` and `:640`, both INSIDE
`bundleEngrave` -- and §5.4 cuts every accepted preimage plate BEFORE calling it.
At every site where the text can be built, every accepted plate is already cut.
The clause could never fire.

It also could not be plumbed. `bundleAbortWarningText(p bundlePlate, secret bool)`
has no channel for the new fact, and the shipped comment at
`gui/bundle_flow.go:610-616` PROHIBITS the obvious workaround by name: a variadic
tail *"would leave order and arity unchecked on the value deciding whether a
plate says PASSWORD REQUIRED"*.

So both arms are the COMPOSER's screens, drawn by `composerEngraveStep` after the
fact -- `bundleEngrave` already draws its own abort warning and then returns
`bundleEngraveAborted`, so the operator sees the set-level screen and then this
one. Each is measured ALONE, which is what it is drawn as.

**§8.4a, `composerAbortNoPreimage` -- no preimage plate was cut:**

> NO PREIMAGE PLATE WAS CUT. The phrase dies with this composition. Do not fund
> this wallet.

MEASURED on `errorScreenBody`: **75 drawn / headroom 476.**

**§8.4b, `composerAbortPreimageCut` -- at least one was, and no policy set:**

> A PREIMAGE PLATE WAS CUT and no policy plate was. Store or destroy it now; do
> not leave it with the blanks.

MEASURED on `errorScreenBody`: **86 drawn / headroom 476.**

**The first draft's numbers for this section did not reproduce and are
withdrawn** (round-0 fidelity I-6, tests I-2). It stated *"288 drawn / headroom
244 alone"*; measured, §8.4a alone is **75 / 476**. Its companion figure was
`411 / 121` for a "LONGEST variant" in which the arm was prepended to the shipped
seed clause -- that combination measures **414 / 121** here and is in any case a
body this stage no longer builds, because the arms are separate screens. (The
3-character gap is the caller context: `bundleAbortWarningText` interpolates
`cardIdx`, `cardTotal` and `label`, which the first draft never pinned. Nothing
in §8 now depends on it.)

**Four longer drafts of §8.4a were measured and rejected** in the first draft,
because headroom was a LINE budget in the combined body and each cost a line:
488/**39**, 455/**79**, 428/**79**, 441/**79**. Those numbers belong to the
combined body and no longer bind: on its own screen §8.4a has 476 characters of
headroom. They are kept only as the record that the wording was chosen, not
defaulted -- **any replacement is re-measured on `errorScreenBody` alone.**

**It says "dies with this composition", not "is now gone"** (author report §2,
deviation D2). An abort inside `bundleEngrave` returns `bundleEngraveAborted`, so
`composerEngraveStep` returns false and `composerFlow` loops back to the shape
with the state intact (`gui/composer_flow.go:47-131`) -- the phrase is still
held. Saying it is gone would be false on the screen whose job is to stop a
funding decision, which is the defect class H5 §1.2 removed from the confirm
modal when "and this digest" made a claim false of the one item that WAS on the
plates.

### §8.5 The QR toggle warning

Confirm-to-proceed, on the model of `ftWarnQR`
(`gui/freetext_flow.go:1214-1216`), which already warns for strictly less
dangerous content:

> The QR makes the phrase readable by any camera. A photograph of the plate is a
> copy of the phrase, and the phrase spends this path.

MEASURED through `confirmWarningBody`, wrapped in `composerConfirmBody`: **126
drawn / headroom 378**.

### §8.6 The QR text -- NORMATIVE, byte for byte (decision 2)

Three labelled lines, LF-separated, **no trailing newline**, the phrase LAST:

```
hashlock v1
method: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32
phrase: <phrase>
```

or, for the sha256 method, the middle line is exactly:

```
method: sha256
```

Rules:

1. `hashlock v1` is the VERSION TAG and is version 1 of this text, not of the
   derivation. A future parameter set gets `hashlock v2`.
2. The method line names the algorithm IN FULL, so a reader with the plate and no
   tool can reproduce the derivation. Its parameters are `HASHLOCK_SALT`,
   `HASHLOCK_ITERATIONS` and `HASHLOCK_DKLEN`
   (`crates/ms-codec/src/hashlock.rs:27,30,32`), rendered lowercase; the device
   reads them from `hashlock.Salt`, `hashlock.Iterations` and
   `hashlock.PreimageLen` (`hashlock/hashlock.go:21,24,27`) and never from a
   literal, so a parameter change cannot leave the plate lying.
3. The phrase is the LAST line so a reader knows where it ends: it may itself
   contain `:` and spaces, and everything after `phrase: ` on the final line is
   the phrase, verbatim, with real `0x20` spaces and never `SpaceMark` -- the
   passphrase plate's rule (`backup/passphrase.go:93-100`: *"a scanner that saw
   it would hand a wallet different bytes"*).
4. The method line is 73 characters and the whole text is **194 bytes** at a
   100-character phrase (§6.5, §7.1). It is pinned by corpus rows in the
   `ms hashlock` corpus (§11.2), because `ms hashlock` learning to parse it is a
   follow-on and the text must be fixed before the parser exists.
4a. **THE DEVICE NEEDS BOTH LINES AS FUNCTIONS, AND NOTHING IN THE FIRST DRAFT
   PRODUCED THEM -- NORMATIVE** (plan round, gate fix F8). §6.1's plate carries
   `Method` and `QRText` as fields the CALLER fills, and the caller is the
   composer (§5.3) or the Hashlock plates flow (§5.2) -- so a plate built from
   production had nothing to put in either. `hashlock.MethodLine(hardened bool)`
   and `hashlock.QRText(hardened bool, phrase string)` are deliverables of this
   stage: the Go twins of `ms_codec::hashlock::qr_text`, built from
   `hashlock.Iterations`, `hashlock.Salt` and `hashlock.PreimageLen`
   (`hashlock/hashlock.go:21,24,27`) and never from a literal, so a parameter
   change cannot leave the plate lying -- which is rule 2 above, enforced instead
   of stated. They are DOWNSTREAM of the Rust primary and are pinned against the
   vendored `qr_text` rows (§11.2), not against themselves. MEASURED: the
   hardened line is **73 characters**, which is §6.5's own pin.
5. **The plate names the ALGORITHM and not the `--method` selector, and that is
   declined here with a reason rather than foreclosed by arithmetic** (round-0
   journey M-2). A year-later operator scanning the QR reads
   `method: pbkdf2-hmac-sha256 ...` and must map it to
   `ms hashlock --method hardened`, a mapping on no part of the plate; the
   `sha256` plate has no such gap, because its line IS the selector. Appending
   the selector to the method line costs 32 characters (105 total, MEASURED),
   and §6.5 pins the line at 73 with a 79th character pushing the worst case over
   budget -- so it cannot go THERE. It could go on its own body row
   (`ms hashlock --method hardened`, 29 characters, one line at 3.0 mm), and it
   is not taken: the plate's job is to let a reader reproduce the derivation with
   ANY tool, which the algorithm line does and a flag name does not, and
   `args.method.unwrap_or(Method::Hardened)`
   (`crates/ms-cli/src/cmd/hashlock.rs:184`) already makes `hardened` the default
   the plate does not need to name. Recorded so the next reader sees a decision
   rather than a constraint.

### §8.7 The plate's own words

Band texts are §6.2's **three distinct literals** -- `HASHLOCK PREIMAGE`,
`HASHLOCK PHRASE`, `NOT A SEED` -- in four table cells, because `NOT A SEED` is
the footer of both forms (round-0 fidelity N-1; §6.2's own "MEASURED lengths: 17,
15, 10" gives three numbers, and "four band literals" was a miscount). The phrase
form's space legend is `passphraseLegend` (`backup/passphrase.go:168`), reused
verbatim.

### §8.8 The Password program says where a hashlock phrase is used -- NORMATIVE

**The refusal at `progPassword` is correct, structural, and completely invisible**
(round-0 journey I-6). §4.1 is emphatic that a hashlock phrase is never admitted
there, and `admitted[progPassword]` is `{ClassPassphrase: true}`
(`gui/sysw_admit.go:35`). But the mechanism that enforces it draws nothing:

```go
// gui/sysw_session.go:270-278
func syswOfferAlt(ctx *Context, th *Colors, want sysw.Class, title, lead, alt string) (string, bool) {
	if ctx.sysw == nil || !ctx.sysw.has(want) {
		return "", false          // <-- no screen is drawn
	}
```

So the operator packs their hashlock phrase, taps, opens the program whose NAME
matches the thing they are holding, and gets the ordinary passphrase keyboard --
no offer, no mention that the payload holds a phrase, no reason. The obvious next
move is the harmful one: re-pack the phrase as a `pass:` record so it "works",
which is the substitution ruling L2 exists to prevent and whose stated stake is a
different wallet.

Note the asymmetry this leaves without §8.8: §9 gives the passphrase program a
warning for an operator who TYPES an ms1 string at it -- the rarer mistake, with
no funds consequence, since free text and passphrase both cut as typed -- while
the operator whose payload literally CONTAINS a hashlock phrase gets nothing.

Silence is today's behaviour, so the test is whether the wrong outcome is worse
than saying nothing. Here saying nothing is what routes the operator around the
guard, so it is worse. **One modal, drawn at `progPassword` when the loaded
payload holds a `ClassPhrase` record and no `ClassPassphrase` record:**

> This payload holds a HASHLOCK PHRASE, not a BIP-39 passphrase. They are not
> interchangeable: using one as the other opens a different wallet. A hashlock
> phrase is used in the Wallet Policy program.

MEASURED on `errorScreenBody`: **165 drawn / headroom 397.**

### §8.9 The bodies this section does NOT blockquote, measured -- NORMATIVE

**MEASURED: this stage adds twenty `composerCopy*` bodies; thirteen are
blockquoted in §8 or §10.1 and SEVEN are not, and that is recorded here rather
than left for a reader to discover** (plan round, the gate's second deliberate
non-change). Each is a `composerCopy*` function with a row in
`composerCopyTable`, which is what carries §12 item 5's four gates -- the glyph
check, the raster floor, the `assertModalBodyFits` measurement and a
fires-on-its-condition test -- so none of them ships unmeasured. What they lack
is a blockquote in this document to be diffed against, and closing that gap is a
records follow-up owned by the stage's last task, not an edit made inside a build
gate.

| body | where it is drawn | renderer | drawn / headroom |
| --- | --- | --- | --- |
| the payload preimage-record confirm, longest variant | §5.1 | `confirmWarningBody` + `composerConfirmBody` | **274 / 186** |
| the masked pick lead (two lines) | §5.3 step (A) | `composerPickScreen` header | see §5.3 -- a lead, not a modal |
| the preimage-plate refusal (built or fit failed) | §5.3, §5.2 | `errorScreenBody` | **99 / 455** |
| the Hashlock plates flow's list lead | §5.2 | `composerPickScreen` header | a lead, not a modal |
| the Hashlock plates flow's own abort | §5.2 | `errorScreenBody` | **80 / 476** |
| the Hashlock plates flow's empty-payload refusal | §5.2 | `errorScreenBody` | **46 / 513** |
| the door's preimage/phrase count line (§5.2's lead fix) | §5.2 | `composerDoorLines` | a lead, not a modal |
| **the HOLD confirm modal, longest variant** — H2's body, whose middle sentence §0 item 5 rewrites; blockquoted in `SPEC_hashlock_H2_device` §4.5, not here | §2.2 / H2 §4.5 | `confirmWarningBody` + `composerConfirmBody` | **342 / 107** (was 343 / 107) |

Every modal figure is a measurement of the text as written, taken with
`assertModalBodyFits` on the renderer production uses; the margin is 80
characters, and the tightest of the bodies this stage ADDS clears it by 106. **The
one body this stage EDITS is the tightest on the screen**: the HOLD confirm modal
has 27 characters of room, so §0 item 5's rewrite had to come in one character
under what it replaced, and a longer wording measured 364 / 64 and was refused by
the gate. The three leads are
measured as ROWS instead -- a lead is drawn through `composerPageLines` and its
budget is the page's, which is what §5.3 step (A) and §5.1 pin.

---

## §9. The free-text and passphrase warning (ruling A8)

**Never a refusal.** Both programs cut as typed. `engraveTextFlow` and
`engravePassphraseFlow` gain a confirm-to-proceed at OK on the text-entry screen
-- earlier than the confirm summary, which is already paged
(`ftConfirmFlow`, `gui/freetext_flow.go:1362-1402`; `ppConfirmFlow`,
`gui/passphrase_flow.go:497`). It fires once per composition, re-armed by an
edit.

**The predicate is `hashlock.IsMS1Shaped`** (`hashlock/hashlock.go:122-148`), the
host's `looks_like_ms1` ported byte for byte: trim, ASCII-lowercase, strip the
display separators, then ≥ 48 characters, an `ms1` prefix and only bech32
characters -- **no checksum**. That is deliberate and is H2 §2 rule 3's own
argument: a GROUPED plate is what `ms hashlock`'s card prints
(`crates/ms-cli/src/cmd/hashlock.rs:340,348`, `render_grouped`) and therefore what
an operator retypes, and `codex32.IsPreimage` would answer false for it.

**The body**, as ruled, unchanged by measurement:

> This looks like an ms1 string. A seed plate comes from a payload; a marked
> hashlock plate comes from the Wallet Policy program, from a phrase typed there
> or a preimage packed on the host. Continue here to cut it as plain text.

MEASURED: **204 drawn / headroom 302** through `confirmWarningBody` wrapped in
`composerConfirmBody`, and **184 / 378** on `errorScreenBody`. It fits as ruled;
no wording change is taken.

The passphrase program shows the SAME body: the string is about to become a
BIP-39 passphrase rather than a plate, but the sentence that matters -- what it
looks like, and where the marked plate comes from -- is identical, and two
near-identical bodies is how one of them goes stale.

---

## §10. Provenance-aware copy and marking

### §10.1 §8h gains a cut-in-this-set form

`composerCopyHashEveryPathFor` (`gui/composer_copy.go:527-532`) chooses today
between the plain form (`:182-186`) and the phrase form (`:520-525`). It gains a
third arm, chosen when EVERY hashed path's digest has material in
`hashlockHeld` that this run will cut:

> HASH ON EVERY PATH
> Every way to spend this wallet needs the preimage of a hash. This composition
> holds the preimage for each one and can cut a plate for it at Done. Store those
> plates apart from these, and apart from each other.

and, when EVERY one of those paths' material carries a phrase:

> HASH ON EVERY PATH
> Every way to spend this wallet needs a hashlock preimage. This composition
> holds the phrase and method for each one and can cut a plate at Done. Store
> those plates apart from these, and apart from each other.

MEASURED on `errorScreenBody`: **185 drawn / headroom 360** and **186 / 360.**

**THE FOURTH ARM'S PREDICATE IS `every`, NOT `at least one`, AND THE BODY IS WHAT
FORCES IT -- NORMATIVE** (plan round, gate fix F12). The first draft chose this
arm when *"at least one of those digests came from a phrase typed here"*, and its
own body says the composition *"holds the phrase and method **for each one**"*.
On a mixed policy where one path's material is a payload PREIMAGE record -- which
carries X and no phrase at all (§5.1) -- "for each one" is then FALSE, on the
banner whose whole job is to say what spending needs. The first draft's wording
also excluded a payload-delivered PHRASE the device does hold, whose backup
burden is identical to a typed one. So the predicate is: every hashed path has
material in `hashlockHeld`, AND every one of those materials carries a phrase.
MEASURED, with the predicate mutated to `at least one`: the mixed row fails with
*"a mixed composition claims to hold the phrase for each path"* against the
held-phrase body. §11.5 carries a mixed row and a partial row for this reason.

**Both are in the PRESENT TENSE of what is HELD, not the future tense of what
will be cut, and that is a correction** (round-0 fidelity I-4). The first draft
said *"This run cuts a preimage plate for each one"*.
`composerCopyHashEveryPathFor` has one call site,
`gui/composer_shape.go:442-443`, inside `composerShapeFlow`, which `composerFlow`
runs at line 75 -- while `composerEngraveStep`, where §5.3's accept/decline
happens, runs at line 125. The banner is drawn several steps BEFORE the plate set
exists: before the form pick, before the census, and before the operator can
decline a plate, which §5.3 item 5 explicitly permits. An operator who read "this
run cuts a plate for each one" and then declined one would be left with a backup
instruction that was false, on the screen whose whole job is to say what spending
needs. That is the class H5 §1.2 removed from the confirm modal, and §8.4 cites
the same standard in its own reasoning; it now applies here too. The two measured
figures above are of the CORRECTED wording; the first draft's 153/378 and 159/378
belong to the withdrawn future-tense text.

When some hashed path has no plate in this run, the SHIPPED forms stand
unchanged -- they say the preimage is "not on these plates", which is then true
of at least one path, and H5 §2.5 already recorded that this family overcounts in
the safe direction.

**Where the load actually falls: NOT here, on the ordinary wallet** (round-0
journey M-3). The call site is guarded by `composerEveryPathHashed`
(`gui/composer_state.go:257-259`), which is false the moment ONE path is keyed --
i.e. on the ordinary mixed hashlock wallet. The fork already records this:
`gui/composer_hashlock.go:70-79` says §8h *"is guarded by composerEveryPathHashed
... which is false the moment ONE path is keyed -- so on the ordinary mixed wallet
the line was drawn nowhere at all"*, which is why H5 moved the reconcile line into
the phrase route. H6 does not make it worse and does not change the guard. It is
written down so a plan built from §10.1 does not add an arm believing it is the
surface that carries this information: **on a mixed policy it is §8.3's census
block at Done that tells the operator a preimage plate is being cut**, for every
composition, and that is why §8.3 is not optional.

### §10.2 The reconcile screen is provenance-correct BY CONSTRUCTION -- no guard

**The first draft added a runtime guard here and it would have changed nothing**
(round-0 fidelity I-3). It said the reconcile screen *"is drawn only when the
digest's provenance is `hashlockFromPhrase`"*.
`composerCopyHashlockReconcile` (`gui/composer_copy.go:484-490`) has exactly ONE
call site in the whole tree -- `gui/composer_hashlock.go:83`, inside
`hashlockPhraseRoute`, immediately after a device-typed phrase's confirm. A
payload `phrase:` record is picked from `Which hash?`'s band 3 and takes §5.1's
derive-only path, which never enters `hashlockPhraseRoute`. **The screen is
already unreachable for payload provenance**, so the guard is dead code and the
§11.5 mutation the first draft paired with it (*"key §10.2 on `phraseDigests`
instead of provenance"*) could not fail either: at that call site no keying draws
the screen for a payload row, because the payload row does not execute that code.

So there is **no guard and no new test**. What this section records instead is
WHY the property holds, so the next reader does not re-open it: the two
provenances run different code paths (§5.1), and the reconcile screen belongs to
one of them. Its TEXT is unchanged -- H5 §1.1's needle *"run ms hashlock with this
phrase"* survives, and
`TestHashlockReconcileScreenIsReachableOnAMixedPolicy`
(`gui/composer_hashlock_test.go:920`) and the walk
(`cmd/emu/walk_hashlock_phrase.js:458`) keep theirs.

**If a payload phrase should ALSO get a reconcile-style screen, that is a
different screen and this stage does not add one** -- for a phrase that came from
the host in the payload the host already has it, and *"run ms hashlock with this
phrase and method on the host and check the digest matches"* is a no-op that
reads as a task. §13 records it as out of scope rather than leaving it implied.

### §10.3 `PREIMAGE REQUIRED` on the composer's md1 AND mk1 plates

`composerEngraveStep` passes a marking to `bundleEngrave`
(`gui/composer_flow.go:393`, today `"", ""`): `markTitle = "PREIMAGE REQUIRED"`
when any path of the composition carries a hash, `""` otherwise; the footer stays
empty. This is the single-sig mechanism unchanged
(`singleSigPlateMark`, `gui/singlesig.go:365-374`, whose `"PASSWORD REQUIRED"` is
also 17 characters against `MaxTitleLen = 18`, `backup/backup.go:71`).

**It marks the md1 plates AND the mk1 key cards, and the first draft described an
effect it does not have** (round-0 journey I-5). `bundlePlateMark`
(`gui/bundle_flow.go:573-578`) excludes exactly one kind:

```go
func bundlePlateMark(kind bundleCardKind, title, footer string) (string, string) {
	if kind == cardMS1 { return "", "" }
	return title, footer
}
```

and the template form's card list is `cardMD1` **plus** the minted `cardMK1` key
cards (`composerMintCards`, `gui/composer_cards.go:69-74`). So a run-level
`markTitle` stamps every mk1 key card too -- cards whose title band is empty
today, and which in a multisig composition are the artifacts that LEAVE for other
cosigners.

**That behaviour is taken, deliberately, and stated rather than discovered.** It
is `singleSigPlateMark`'s own precedent, where `PASSWORD REQUIRED` reaches mk1 the
same way, and it is what F-132 asks for: a cosigner holding a key card should
learn that the wallet needs a preimage, because they are exactly the year-later
reader who otherwise has nothing telling them one exists. The alternative --
passing the marking per card kind -- would suppress the message on the only
artifact that travels.

What is NOT acceptable is the first draft's test, which could not tell the
difference: *"marks the md1 plates when any path is hashed and no plate when none
is, and never marks a `cardMS1`"* passes whether or not the key cards are marked.
§11.5's row now names the mk1 cards; a row that does not is proving nothing about
the sentence it sits under. The seed (`cardMS1`) plates in the same run stay
unmarked with no change.

It closes the plate half of F-132 (`design/FOLLOWUPS.md:4298`): the year-later
reader holding only the descriptor plates is the one person nothing on the device
currently tells that a preimage exists.

---

## §11. Tests, each with the mutation that must fail it

### §11.1 Host, Rust first

- `phrase:` classification, every `CASES` row: method `hardened`/`sha256`, a
  phrase with a comma, an empty phrase, 100 and 101 characters, a 64-hex phrase,
  an ms1-shaped phrase, non-UTF-8 bytes, uppercase hex, an unknown method.
  MUTATION: cut on the LAST comma → the comma-bearing phrase row fails.
  MUTATION: accept an unknown method → the unknown-method row classifies.
- **Classification is unconditional:** `classify` answers `Preimage`/`Phrase`
  under `Admission::default()` AND under `pack_preimage`. MUTATION: gate the
  classifier on admission → the corpus row and the device's `Classify` disagree
  and the vendored lockstep test reds.
- `--pack-preimage` admits a plate and a `phrase:` record through `admit_check`'s
  new rule; without it both are refused by index with §8.1.1's text. MUTATION:
  drop the new `admit_check` rule → the no-flag rows are admitted.
- **`decide_sealing` is reached with the flag and SEALS.** `--pack-preimage`
  alone (no `--no-passphrase`) seals and the sealing line names the class.
  MUTATION: make `classify` admission-gated and leave `decide_sealing` on the
  strict classifier → the row reports `NOT SEALED` and the payload ships bearer
  material in cleartext. **This row is why §3.2 gates admission and not
  classification**, and it is the funds-relevant one on the host.
- The id rule (§4.3): a kind-`0x03` single under id `hash` is admissible; under
  an id outside `{entr, hash}` it is refused with §8.1.2; **under `entr` it is
  refused with the SHIPPED `TagKindMismatch` text**, with and without the flag.
  MUTATION: drop the id test from `preimage_plate_admissible` → the mistagged row
  is admitted. MUTATION: use `entr` as the "any other id" case → the row must
  expect the shipped text, not §8.1.2. **The first row is the funds-relevant
  one**: without it a plain BIP-93 33-byte secret beginning `0x03` -- roughly 1 in
  256 of them -- reaches a plate flow.
- **`preimage_plate_admissible`'s OTHER TWO conjuncts (§4.3), each with the
  SHIPPED row that already carries it.** The rows live in
  `a_preimage_plate_is_named_not_misdiagnosed`: a kind-`0x03` single under the id
  `hash` whose X is 16 bytes stays `Unclassifiable(PreimagePlate)`, and so does
  the UPPERCASE spelling of a plate. Both are named a preimage plate by the
  DIAGNOSTIC and admitted by neither predicate.
  MUTATION: drop the `33 bytes beginning 0x03` conjunct → the malformed row
  becomes `PreimageNotAdmitted(0, Preimage)` -- MEASURED, `left:
  Err(PreimageNotAdmitted(0, Preimage))` against `right: Err(Unclassifiable(0,
  PreimagePlate))` -- and `--pack-preimage` then admits a 50-character string
  `DecodeMS1Preimage` refuses into a flow that engraves.
  MUTATION: compare the id with `eq_ignore_ascii_case` → the uppercase row gives
  the same failure, and the two sides stop agreeing: `codex32.IsPreimagePlate`
  compares `Split()`'s id to the literal `"hash"`.
- **§8.1.1's final sentence is emitted only for id `hash`.** MUTATION: emit it
  unconditionally → the wrong-id row sends the operator to a flag that will
  refuse them.
- **`codex32.EncodeMS1Preimage` (§3.5), three rows.** (a) lockstep: it reproduces
  the `hashlock-v0.8.json` `kind` row's `ms1` byte for byte for that row's
  `preimage_hex` -- id `hash`, kind `0x03`, 75 characters. (b) round trip:
  `DecodeMS1Preimage(New(EncodeMS1Preimage(x))) == x` over random X. (c) **it can
  never emit id `entr`**, asserted on `Split()`'s id of the OUTPUT rather than on
  the source, over the same random X. MUTATION: pass `"entr"` to `NewSeed` →
  (a) and (c) both red, and the string produced satisfies `IsPreimage` while
  failing `IsPreimagePlate` -- which is the copy-paste hazard, reproduced.
  MUTATION: reintroduce an `id string` parameter → (c) reds.
- `decide_sealing` seals a payload holding a preimage and says so; with
  `--no-passphrase` it does not and says so. MUTATION: make the class non-secret
  → the seal row reports NOT SEALED.
- All four warnings of §8.2 fire on their condition and not otherwise, in the
  §3.3 order against the passphrase ceremony (1-3 before the sealing line, 4
  after). MUTATION: drop the orphan check → the orphan row prints no warning.
  MUTATION: move warning 1 after the ceremony → the F-246 ordering row reds.
- **The orphan check covers `phrase:` records**: a `phrase:` record whose derived
  digest matches no `hash:` record warns; one that matches does not; a payload
  holding NO `hash:` record gets the note, not the `WARNING —`. MUTATION: drop the
  phrase arm → the hand-built rows print nothing. MUTATION: hex
  `hardened, my phrase` (a space after the comma) → the digest differs from
  `hardened,my phrase` and the warning fires, which is the row that catches the
  hand-build error.
- The corpus is regenerated and re-vendored; the Go side asserts the sha256
  literal. MUTATION: edit one vendored row → the sha test reds.

### §11.2 The QR text corpus (mnemonic-secret)

Rows in `crates/ms-codec/tests/vectors/hashlock-v0.8.json`, whose sha and
provenance are re-pinned on both sides (H2 §7.1 requires the fork to assert the
literal): the exact §8.6 text for the anchor phrase under both methods, for a
100-character phrase, for a phrase containing `:` and for a phrase containing a
trailing space. MUTATION: emit a trailing newline → every row fails.
MUTATION: put `phrase:` before `method:` → every row fails.

### §11.3 The constant-time QR raise

- The alignment table of §7.2 is DERIVED in the test from the encoder's own
  bitmap and compared against the table the code carries, for v2..v9.
  MUTATION: change one centre → the derivation disagrees. (Re-derived in the
  round-0 fold and every row of §7.2 reproduced exactly.)
- `ConstantQR` accepts 41/45/49/53 and refuses 57. MUTATION: raise the bound to
  57 without the table → `bitmapForQRStatic` panics, which the test asserts it
  no longer does.
- **THE ROW THAT CAN ACTUALLY FAIL AT RUNTIME: `len(modules) <=
  constantTimeQRModules(dim)` for N fuzzed §8.6 payloads at each of the SEVEN
  dims §8.6 content reaches -- 29, 33, 37, 41, 45, 49, 53** (plan round 0,
  fidelity I-5; the first draft sampled v6..v9 only, so the three shipped
  budgets this content also reaches were watched by nothing) -- and `findPath`
  returns no error on any of them. This is the budget proof against FRESH
  content, and it is stated separately because the move-count row below cannot
  substitute for it. The shape enumerator and the byte→dim table are extended
  with it, and `TestECCLThresholdsAreWhatTheBudgetAssumes` sweeps from 1 byte so
  the table cannot drift and quietly stop sampling a version. MUTATION: lower
  the SHIPPED dim-29 entry to 340 → MEASURED, *"dim 29, payload 9: ConstantQR:
  too many dims 29 QR modules for constant time engraving n: 342 waste: 8"*;
  before the extension that mutation was invisible to the whole suite.
- **A SECOND ROW, A PIN, BECAUSE THE MUTATION THIS SECTION FIRST NAMED CANNOT
  FIRE FROM ANY IN-SUITE SAMPLE** (plan round, finding 3). The first draft said:
  *"set any of the four arms to the observed maximum MINUS ONE → that version's
  row reds, which is also the check that the entry was derived rather than
  guessed."* MEASURED, with the plan-time campaign's maxima minus one
  (822 / 959 / 1178 / 1378), **the fuzzed row still PASSES** -- its 400 payloads
  per dimension observe far below the altered budgets, and reaching a true
  maximum took between 7 and 14 MILLION payloads, which a suite cannot spend.
  So the mutation is retired and the property it was reaching for gets its own
  test: `TestConstantTimeQRBudgetEntriesAreTheFuzzedOnes` PINS the four entries
  as `<observed> + <buffer>` together with the campaign that produced each, and
  asserts the five SHIPPED entries are untouched. It is not a weaker test but a
  different one -- **the fuzzed row proves the entry BOUNDS fresh content, the
  pin proves the entry is the one that was DERIVED** -- and changing an entry now
  means changing the table, which means saying where the new number came from.
  MUTATION: change any of the four arms (measured at v9, `1379 + 20` → `1378`) →
  `constantTimeQRModules(53) = 1378, want 1399 (1379 observed + 20 buffer)`,
  naming the dimension, both values and the campaign. The same edit leaves the
  fuzzed row GREEN, which is the finding stated as two tests.
- **THE REGRESSION GUARD, labelled as one: for each of v6..v9, two different
  payloads of the same version emit the SAME move count.** MUTATION: make the
  move list depend on a module's colour → the two payloads differ.
  **This row passes on a WRONG budget and on a budget of 0**, because `Engrave`
  loops `for range nmod` and pads every move to `maxDur` -- so it proves the
  padding, not the bound. It is kept for what it does prove and named for what it
  does not.
- **Scale 2 (§7.3):** `engraveModule` emits a constant FIVE commands per module at
  scale 2, the same as scale 3; the painted extent of a module at `p` is exactly
  `[p*2sw, p*2sw+2sw]` on both axes; and a v9 `ConstantQRCmd.Engrave(..., 2)`
  completes without panicking. MUTATION: remove the `case 2` arm → the panic
  returns and the phrase plate cannot be built at the only scale it fits at.
  MUTATION: make the arm symmetric about the centre (the `case 3` shape scaled
  down) → the extent assertion reds, because `centerOf` puts the scale-2 centre
  half a stroke off the cell centre.
- Goldens for v6, v7, v8, v9 on fixed payloads, **at scale 2**.

### §11.4 The plate

- Golden bytes for both forms at their fitting rungs, with and without the QR.
  The string form's golden pins it LOWERCASE and UNGROUPED at 6.0 mm (§6.1), so
  "VERBATIM" cannot drift into `EngraveSeedString`'s upper-cased, grouped
  rendering. MUTATION: group in tens → at 19 characters per line the body goes
  from 10 rows to 14 and the fit gate reds at 6.0 mm.
- The §6.5 fit gate: the worst-case phrase plate (100-character phrase, hardened
  method line, three header rows, QR) lays out at 3.0 mm and REFUSES at any
  larger rung, measured on the real render rather than on arithmetic.
  MUTATION: grow the method line to **79 characters** → it wraps to 3 rows at
  3.0 mm instead of 2, the body is 11 rows, and `EngraveHashlock` refuses with
  `11 rows at 3.0mm need 427520 units against a budget of 416000` (66.80 mm
  against 65.00 mm). MUTATION: set the QR scale to 3 → the envelope is 47.70 mm,
  3.0 mm needs 510080 units (79.70 mm), and every rung reds.

  **SEVENTY-NINE, AND THE FIRST DRAFT'S "ADD ONE CHARACTER" CANNOT FAIL** (plan
  round, finding 2). At 39 characters per line a 73-character line wraps to 2
  rows anywhere from 40 to 78 characters, so a 74th changes nothing. MEASURED,
  all five: 74, 75, 76, 77 and 78 characters ALL still fit and the gate stays
  green; 79 reds with the tail above. §6.5 consequence 3's own number -- *"a 79th
  character would make it 3"* -- is the threshold, and the mutation is written to
  it so that it is one that fires.
- **WHAT THE PLATE COSTS TO CUT, LOGGED ON EVERY RUN** (plan round 0, journey
  I-4), because §12 item 8's gate is budgeted in this number and its cost is
  what decides whether the gate is run. MEASURED at production
  `internal/sh2.Params()` by `engrave.TimePlan`: the worst-case phrase plate with
  its v9 QR **43m31s**, the same plate without the QR **14m39s**, the string form
  **12m14s**, the v9 QR alone at scale 2 **32m12s**, and one 6 mm character
  **7s**. The assertion is a TRIPWIRE and not a pin -- durations move with the
  stepper profile and the layout, and one hour is the threshold a layout change
  that DOUBLED this plate would cross, which is the class worth catching because
  a plate that no longer fits in one sitting changes the acceptance procedure.
  MUTATION: cut the production engraving speed to a third → *"worst-case phrase
  plate WITH the v9 QR takes 1h14m43s to cut, over the 1h0m0s bound"*. (The
  obvious mutation, the QR at scale 3, reds EARLIER at the fit gate -- 510080
  units against 416000 -- so it does not exercise this row.)
- Band budget: the three band literals are within `MaxTitleLen`; every locator
  row is a BODY row and **no locator ink lands in a screw-hole band**.
  MUTATION: draw any locator row at `l.topY` or `l.bottomY` → the ink-in-band
  assertion fires. **The first draft's mutation could not fail** and is replaced:
  it claimed moving `mk1 stub (template): <8 hex>` into a band would red because
  "the 33-character `path` row is not [inside the cap]", but measured, every
  locator row is 6-30 characters against a 32-character cap at 3.0 mm, and the
  33-character row is a concatenation §6.3 does not produce. The constraint is
  placement and line count, so the mutation tests placement.
- **The wrap rule:** the body wraps on characters, not words. MUTATION: wrap on
  word boundaries → the 100-character phrase and the 75-character ms1 string are
  single tokens with no break point and the layout fails or overflows.
- The QR encodes §8.6's text and NOT the ms1 string. MUTATION: encode
  `plate.MS1` → the decode assertion fails.
- Spaces: a phrase with a leading, trailing or doubled space renders `SpaceMark`
  in the text block and REAL spaces in the QR. MUTATION: mark the QR too → the
  QR decodes to a different phrase.
- The plate is not a `bundleCard`: `bundlePlatePlan` over the run's cards returns
  the same plan with and without preimage plates. MUTATION: add the plate as a
  card → the census count and the "all of it exists" claim change.

### §11.5 The device flows

- `Which hash?` by LABEL with 0, 1 and 2 records of each new class; every row
  does what its label says; `Type 64 hex` never clears the lock (H2's C-4
  regression test). MUTATION: reintroduce an index-keyed `default` that assigns
  → the displaced-row rows fail.
- A `phrase:` row shows the derive marker before it is picked and its digest
  after. MUTATION: derive at row-build time → a timing assertion on the screen's
  first frame fails.
- **The Hashlock plates flow does the same** (§5.2): its list screen draws its
  first frame without running the KDF; picking a `phrase:` record derives ONCE
  behind the countdown; the plate it cuts carries a `hash` locator row whose
  digest EQUALS the one the host derives for that phrase. **The row is asserted
  on a PHRASE record and THROUGH the flow, never by a unit call on an
  already-derived one** (plan round 0, journey C-1): a preimage record arrives
  derived (`hashlockPlatesRecords` decodes X at list time), so a locator test
  built only from one exercises neither the phrase branch nor the ORDER, and
  "non-empty" is true by construction whatever the digest is. The want value is
  computed in the test from `hashlock.PreimageSHA256` / `PreimageHardened`,
  never read back out of the flow, and the plate the flow builds is observed at
  `composerHashlockPlateBuiltHook` -- the in-file seam of
  `freetextEngraveHook`'s shape -- because a locator is ENGRAVED and never
  drawn, so no screen assertion can reach it. MUTATION: derive per frame → the
  once-per-pick assertion fails. MUTATION: omit the locator's `hash` row when
  the record is a phrase → MEASURED, *"locator = [], want a `hash
  e7e68d52..476e1407` row"* in the unit row and *"the plate's locator = []"*
  through the flow. MUTATION: build the locator one statement earlier, before
  `hashlockPlatesDerive` → MEASURED, *"the plate's locator = [\"hash
  00000000..00000000\"]"* -- the bearer plate with no locator at all, which the
  first mutation alone left green across all 24 shards.
- **A `hash:` row whose digest is matched by a preimage or `phrase:` record in
  the same payload is annotated `(in payload)`** and draws on one line.
  MUTATION: drop the annotation → the both-present row shows two adjacent rows
  with identical digest text and different consequences.
- `hashlockHeld` on a zero-value `composerState` built exactly as `composerFlow`
  builds it: one HOLD, no panic, the material present. MUTATION: assign without
  the nil check → panic.
- `composerFlowExit` scrubs it. MUTATION: remove the scrub call → a test reading
  the map through the `!tinygo` seam after exit finds the phrase.
- **The review is two steps and the per-plate choices are on the PICK step.** By
  LABEL: the pick screen offers `preimage string` / `phrase + method` /
  `phrase + method + QR` / `do not cut this preimage`, the QR rows only when the
  phrase is held; taking a QR row fires §8.5. The census is READ-ONLY and reports
  the choice. MUTATION: put the form/QR/decline controls on `confirmReviewScreen`
  → there is no control left to bind and the test cannot drive them, which is the
  point. MUTATION: offer a QR row when the device holds no phrase → the row
  assertion fails.
- The review: a declined plate is dropped and named; an unused preimage is listed
  and not cut; the plan's remaining plates are unchanged. MUTATION: drop the
  declined plate silently → the census row assertion fails.
- **The census block is drawn through `composerPageLines`' band**, and no census
  row's ink lands under a navigation button, on EVERY page and not only the
  first. **The instrument is an AST assertion on `composerEngraveStep`'s call
  set, not a raster probe** (plan round, gate fix F10): `composerReadScreen` and
  `confirmReviewScreen` both draw a paged read-only body of the same shape,
  `ExtractText` collects a glyph's rune wherever it lands, and the real frame
  draws the nav buttons over it, so no text or raster assertion on the live
  screen can tell the two apart. MUTATION: draw the block through
  `confirmReviewScreen` → MEASURED, *"composerEngraveStep draws through
  confirmReviewScreen, which wraps at dims.X-2*8 and centres on the whole panel:
  every census row over 374 px has its right edge under a navigation button
  (W-3)"*.
- **`Which hash?`'s first page holds FIVE rows** (§5.1): a payload with a
  `hash:`, a preimage and a `phrase:` record draws six, page 1 draws five touch
  targets, and `No hash lock` is reachable by paging. MUTATION: assert six →
  the count row fails; MUTATION: drop the paging assertion → a row that has
  moved off page 1 forever still passes.
- **The pick step's lead is TWO lines and all four rows are on page 1** (§5.3
  step A). MUTATION: give the lead four lines (the digest, the path, the count
  and the method each on their own) → MEASURED, *"page 1 draws 3 of the four
  rows; the lead is spending them"*, and the row displaced to page 2 is
  `do not cut this preimage`.
- **The plate list is in a deterministic order** (§5.3): paths first in path
  order, deduplicated by digest, then unused digests in digest order, over
  repeated builds of the same state. MUTATION: range `hashlockHeld` directly →
  the order changes between builds, on the screen read against the bench.
- Cut order: the preimage plates are engraved before `bundleEngrave` is called.
  MUTATION: move the loop after it → an order assertion on the engrave hook
  fails.
- **BOTH abort arms**, on the composer's own screens: §8.4a fires when the run
  ends with NO preimage plate cut; §8.4b fires when at least one was cut and no
  policy set completed; NEITHER fires on a completed run. MUTATION: fire either
  unconditionally → the success row fails. MUTATION: attach either to
  `bundleAbortWarningText` → the arm is unreachable, because §5.4 cuts every
  accepted preimage plate before `bundleEngrave` is called, and the test that
  asserts §8.4b appears after an abort inside `bundleEngrave` fails.
- **The Hashlock plates flow does NOT draw either arm** (§5.2). MUTATION: reuse
  §8.4a there → an assertion that the flow's abort draws no "dies with this
  composition" body fails, because the material is still in the payload.
- §10.1's third §8h form fires on its predicate and not otherwise, **and its body
  does not claim a plate was cut**: a run in which a plate is DECLINED at §5.3
  step (A) leaves the §8h body true. MUTATION: restore the future-tense wording
  ("This run cuts a preimage plate for each one") → the decline row asserts a
  body that is false about what the run did.
- **§10.1's FOURTH form takes a MIXED row and a PARTIAL row** (plan round, gate
  fix F12): a composition where one path's material is a payload preimage record
  with no phrase must draw the third arm, not the fourth, and a composition
  holding material for only one of two hashed paths must draw neither held arm.
  MUTATION: choose the fourth arm on "at least one" phrase rather than every →
  MEASURED, *"a mixed composition claims to hold the phrase for each path"*.
- **§10.2 gets no test**, because it gets no guard: the reconcile screen has one
  call site, inside `hashlockPhraseRoute`, which a payload phrase never enters
  (§5.1). The existing
  `TestHashlockReconcileScreenIsReachableOnAMixedPolicy` continues to cover the
  device-typed path unchanged.
- `PREIMAGE REQUIRED` marks the md1 plates **and the mk1 key cards** when any
  path is hashed, marks no plate when none is, and never marks a `cardMS1`.
  **The mk1 assertion is required, not optional**: without it every assertion in
  this row passes whether or not the key cards are marked, and the row would be
  proving nothing about §10.3's sentence. MUTATION: mark unconditionally → the
  unhashed row fails. MUTATION: suppress the marking on `cardMK1` → the key-card
  row fails.
- §9's warning fires for a grouped ms1 plate string, an ungrouped one and an
  UPPERCASE one, and not for ordinary text; the plate is still cut as typed.
  MUTATION: use `codex32.IsPreimage` instead of `IsMS1Shaped` → the grouped row
  fails. MUTATION: refuse instead of warning → the still-cut assertion fails.
- **§8.8's Password-program notice** is drawn when the payload holds a
  `ClassPhrase` record and no `ClassPassphrase` record, and NOT when it holds a
  `pass:` record. MUTATION: drop the notice → the operator sees the ordinary
  keyboard with no explanation, which is today's behaviour and the finding.
- Every DEVICE body §8 adds is in `modal_fits_test.go`'s table and passes
  `assertModalBodyFits`: §8.1.2, §8.3's rows and its stand-alone notice, §8.4a,
  §8.4b, §8.5, §8.8, §9's body and §10.1's two arms. Each is measured ALONE, as
  it is drawn.
- **Every DEVICE body is ASCII**, asserted mechanically over the table (`r <=
  unicode.MaxASCII` for every rune), not by inspection. MUTATION: put a `§` or an
  em dash in any body → `assertModalBodyFits` reports a near-blank frame (5,004
  ink pixels against a 6,000 floor) rather than a truncation, and the ASCII
  assertion names the character. **This mutation is the one that must be in the
  suite**: the first draft carried non-ASCII in five bodies and claimed a
  measurement for one of them that the harness cannot produce.

### §11.6 Whole gates

Four packages; the 24 `gui` shards (`scripts/gui-shard-test.sh <pkg> 24`);
`gofmt`; `go vet`; `cargo nextest run --locked` on the Rust side; the firmware
size stated for the STAGE against a named baseline, re-measured at `fb0dd04`.

**THE DELTA IS NOW A NUMBER** (plan round). MEASURED with
`nix develop -c tinygo build -size short -o /dev/null -target pico-plus2
-stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`, on a
`git ls-files | tar` copy of `fb0dd04` and on the fully wired gate tree:

| tree | flash | ram |
| --- | --- | --- |
| pristine `fb0dd04` | 1,599,208 B | 62,856 B |
| the whole stage, wired | **1,643,580 B** | **63,272 B** |
| delta | **+44,372 B** | **+416 B** |

**Re-measured after the plan round's R0 round 0** (which changed three non-test
files compiled into the firmware -- one copy string one character shorter, a nil
test seam in `composer_preimage_plate.go`, and a comment in `engrave.go`):
**1,643,580 B flash / 63,272 B ram, byte-identical.** `./cmd/emu/build.sh`, which
is not the firmware, moved 11,010,867 -> 11,011,084 B.

**11,011,084 B is an OBSERVATION here, not a pin (F-505, measured 2026-09-06):**
`cmd/emu/emu.wasm` is neither the firmware nor a shipped artifact, a rebuild
from the same source moved it a further two bytes (11,011,082 B), and it is not
byte-reproducible across rebuilds the way the flash/RAM figures above are. A
spec that needs a reproducible identity for the emulator should pin
`cmd/emu/walk_hashlock_phrase.js`'s sha256 instead
(`dfb9e6d5cf5521349db0c116cf7426039e7ff6c177f86e269f92105ddc9bc581` at fork
`e089a539`, unchanged at the current tip) — the walk file is source, and
hashes exactly, where the wasm build does not.

Nothing here is over a ceiling; the estimate above had no number behind it and
now does. The SHAPE is worth recording because it is not what a reader would
guess: stubbing `backup.EngraveHashlock` out of `composerHashlockPlateFor` --
production's only reference to §6's whole plate layout -- measures
**1,639,020 B**, so making the layout REACHABLE costs **4,560 B**, about a tenth
of the stage. The rest is the screens, the flow and the copy bodies. The
corollary binds any partial measurement of this stage: a tree that wires the
codec half without a production caller has TinyGo drop it, so its delta is not
the codec half's cost either.

**AND THE `gui` PACKAGE GATE IS PART OF §2.2 ITEM 6.** A tree carrying the
retention field without §5.1's routes reds `TestComposerEveryScreenFunctionHasA
ProductionCaller` on one shard, deterministically; the whole-package run is only
green once the consumer lands. That is the gate working, not a flake, and the
exemption table is not the way past it.

### §11.7 The walk

`cmd/emu/walk_hashlock_phrase.js` gains an H6 arm: type the anchor phrase, HOLD,
reach the census, accept a preimage plate, and assert the census row carries the
same `first8..last8` the confirm modal carried.

**THE WALK'S COMPOSITION NEEDS A SECOND, KEYED PATH -- NORMATIVE** (plan round).
`md.Compose` refuses a wholly key-less composition (*"every path is key-less; at
least one path must hold a key"*), so a composition of ONE hashed key-less path
cannot reach Done at all: it stops at a refusal, not at a census, and the arm
would assert against a screen that never draws. The arm adds a 2-of-3 path and
leaves it UNSEATED -- which is §12 item 3's own shape.

**And §8.3's block does not fit on the census's first page, so the walk PAGES to
reach it.** MEASURED on the gate's own fixture -- one accepted phrase+QR plate,
one declined and one unused, over a one-card md1 template -- the census is
**11 lines over 3 pages**: page 1 is the shipped supply block, §8.3's heading and
its first three rows are on page 2, and the apart-storage line is on page 3.
`composerReadScreen` withholds the continue affordance until the last page has
been laid out once, which is the shipped contract rather than a new one, so a
walk that asserted only the first frame would assert against the wrong screen. H5 §4.1's doctrine binds -- *"a
walk may READ state only to assert that what the screen shows equals what is
stored; it never drives through a hook"* -- so the walk asserts the SCREEN and
the goldens assert the plate; no third hook carrying a preimage is added.

---

## §12. Acceptance

H6 is done when, on the flashed device:

0. **`ms-codec` 0.9.0 is PUBLISHED and `me` depends on it** (§3.1). The phrase
   rule, `looks_like_ms1` and §8.6's `qr_text` are in the codec, `ms-cli`
   delegates with its refusal sentences unchanged, the CHANGELOG entry records
   the corpus sha `4f1819cd...aba21d4`, `crates/me-cli/Cargo.toml:53` reads
   `ms-codec = "0.9"`, and no `[patch.crates-io]` remains in either workspace.
   **This item is FIRST in fact and not only in the list**: nothing else in §3
   builds until it lands.

1. The operator types the anchor phrase, reaches Done, accepts a preimage plate
   in the phrase form with a QR, and the plate is cut; a phone scan of the QR
   returns §8.6's text byte for byte, and `ms hashlock --hashlock-phrase-stdin`
   over the phrase on that plate reproduces the digest the composer showed.
2. The same composition's md1 plates carry `PREIMAGE REQUIRED`.
3. `ms hashlock --out X.txt` **plus its `hash:` record**, then
   `me sysw pack --pack-preimage --no-passphrase --in <records file>`, a tap, the
   Hashlock plates flow, and a cut plate whose ms1 string round-trips through
   `ms hashlock --in`. **`--in` is required, not stylistic** (§3.6): the two
   classes are BEARER, so `argv_secret_guard` refuses the record on the command
   line before the parser runs, and this item's earlier invocation --
   `me sysw pack --pack-preimage --no-passphrase <ms1>` -- could never have been
   walked.
   The `hash:` record is in the invocation deliberately: `ms hashlock` writes only
   the ms1 string to `--out` and prints `hash:` to stdout, so the minimal
   invocation packs a payload with no `hash:` record at all and the acceptance
   walk would take §8.2.3's orphan path every time (round-0 journey M-6). The
   orphan path is exercised as its OWN item (5a), not as the happy one.
3a. **A device-derived preimage cut in the STRING form** -- the default form
   (decision 1) on the composer-native path: type the anchor phrase, reach Done,
   pick `preimage string`, and the cut plate's 75 characters round-trip through
   `ms hashlock --in` to the digest the composer showed. This item exists because
   nothing else in this list cuts a string-form plate from device-derived
   material, which is what §3.5's encoder is for.
4. A `phrase:` record in the same payload appears on `Which hash?` as
   `phrase record 1 (derive to see the digest)`, derives on pick, and its digest
   matches the host's.
5. A kind-`0x03` string under an id outside `{entr, hash}` is refused on the host
   with §8.1.2, one refusal and not three; the same payload under `entr` is
   refused with the shipped `TagKindMismatch` text; both are inert on the device.
5a. A payload holding a preimage and no `hash:` record draws §8.2.3's NOTE, and a
   payload holding `hash:` records none of which match draws its WARNING.
6. A preimage plate presented to any seed flow is still refused (H0's walk).
7. A payload holding only a `phrase:` record, opened at the Password program,
   draws §8.8's notice (round-0 journey I-6). This is the one refusal in the
   stage whose correctness is otherwise unobservable from outside the code.
8. **The QR is a GATE, not an assumption** (round-0 journey M-7). Before the QR
   toggle ships, one test plate is cut at the WORST case -- a 100-character
   hardened phrase, v9, 53 modules, scale 2, 0.6 mm modules -- and scanned with a
   phone. 53 modules at scale 2 has no precedent in this tree on either axis, and
   §11.3's gates are all bytes and toolpath. If it does not scan, the QR toggle
   does not ship and the phrase form is text-only; the plate is still complete,
   because §6.4 makes the text authoritative. **BUDGET ONE ATTEMPT PER SESSION**
   (plan round 0, journey I-4): MEASURED by `engrave.TimePlan` at production
   `internal/sh2.Params()`, the worst-case plate is a **43m31s** cut, the same
   plate without the QR is 14m39s, and **the QR alone at scale 2 is 32m12s**.
   The single-character test-plate pattern this item used to name -- *"~2 s a
   try … rather than a full plate (~21 min)"* -- **does not apply**, on both
   halves: 21 minutes is the md1 plate's figure, not this plate's, and a
   constant-time QR is INDIVISIBLE by construction, because
   `ConstantQRCmd.Engrave` runs `for range nmod` and pads every move to `maxDur`
   -- which is the whole reason the toolpath is content-independent. The only
   reduction available is cutting the QR alone onto a blank: 32m12s against
   43m31s, a 26% saving and not a 900× one. This matters because the gate fails
   OPEN by design, so its cost is what decides whether it is run at all, and a
   gate budgeted at two seconds and costing half an hour gets deferred -- which
   here ships an untested 53-module QR carrying a spend secret. §11.4 logs all
   four durations on every run.

Until the operator walks it, §11.7's emulator arm and §11.4's goldens are the
acceptance. **The SH2 has no camera**, so nothing on the device can read a plate
it cut; item 1's read-back is a phone and the host, and no acceptance may be
written that assumes otherwise.

---

## §13. Out of scope (this stage)

Reading a preimage plate back into any seed flow; the Sealed Payload container
(`me seal`), whose `RecordError::PreimagePlate` refusal stands
(`crates/me-cli/src/seal/record.rs:130-136`); the typed `M*1 STRING` door;
`ms hashlock` parsing §8.6's text (the text is pinned here, the parser is a
follow-on); `ms split` of a preimage (F-468); the salt and iteration parameters
(F-469); a words-plus-SeedQR secret plate (F-455); a concrete-descriptor plate
and its census refusal (F-457); a restore document for the composer; secret
handling of the retained phrase (F-483, non-gating by the 2026-08-27 ruling).

Added by round 0, so each is a decision rather than an omission:

- **A CLI producer for `phrase:` records** -- `ms hashlock ... --emit-phrase-record`
  or `me sysw record phrase --method ...`. §3.1 gives the wire form a Rust
  function and §3.2 a flag; no verb emits one, so the operator hand-builds hex
  every time. §3.3's phrase-orphan warning is what covers that path this stage,
  and it is a mitigation, not a fix. A wire form with no writer is worth its own
  follow-up.
- **A reconcile-style screen for a PAYLOAD phrase** (§10.2). The device-typed
  screen's instruction is a no-op for a phrase the host already has, and what a
  payload phrase should be told instead is a separate design question.
- **Cross-run awareness of preimage plates already cut** (§5.3 item 6, §8.4b).
  The census cannot know what a previous run of the same composition cut, and
  this stage does not give it a way to find out; §8.4b tells the operator a
  bearer plate exists so that they can, which is the honest half.

Added by the plan round, each measured and each non-gating:

- **`Which hash?`'s first page holds FIVE rows** (§5.1), so a payload carrying a
  `hash:`, a preimage and a `phrase:` record puts `No hash lock` -- the row an
  operator reaches for to UNDO a lock -- on page 2. Reachable by paging and
  pinned by §11.5; a layout that gave it more room is a later device cycle.
- **§8.3's `plate(s)`** is the spelling this package's own house rule refuses
  (§8.3). A records fix, not a copy change made inside a gate.
- **The SEVEN bodies §8.9 lists have no blockquote in §8** (measured: the stage
  adds twenty `composerCopy*` bodies and thirteen are blockquoted). They are
  measured and gated through `composerCopyTable`; what is missing is a document
  to diff the shipped string against.
- **A CLI producer for `phrase:` records** is already above, and the plan round
  confirms it: nothing in this stage emits one, so §3.3's orphan warning is the
  whole of the mitigation.

Added by the plan round's R0 round 0, each measured and each a DECISION taken
here rather than an omission:

- **A DECLINE-ALL COMPLETED RUN REACHES §8.4a'S END STATE AND DRAWS NO ARM**
  (round 0, journey M-4). §8.4a exists for *"NO PREIMAGE PLATE WAS CUT. The
  phrase dies with this composition."* and fires only on an engrave FAILURE with
  `cut == 0` (`gui/composer_flow.go:422-441`). Declining every plate empties
  `accepted`, so the loop never runs, `cut` stays 0, `bundleEngrave` completes,
  and the run ends with `PREIMAGE REQUIRED` on the md1 and mk1 plates, no
  preimage plate, and the phrase scrubbed at `composerFlowExit` -- the identical
  end state, with no arm drawn. §5.3 item 5 is right that declining must not
  abort the run (the operator may already hold the plate; cross-run awareness is
  out of scope above), so a REFUSAL would be wrong; what is missing is a notice
  at the moment of no return, and adding a third arm on the same counter is
  behaviour this round does not take. It is recorded here because §5.3's
  corrected Back contract makes the state reachable by ONE Button1 press at the
  first pick screen. The mitigation that does exist is measured: the census
  withholds Button3 until its last page, so the `declined, will not be cut` row
  is on a page the operator must visit.
- **`--expect` has no vocabulary for the two new classes** (round 0, journey
  M-2). `--expect`'s whole argument is that a backup can be silently incomplete,
  and a scripted H6 pack whose `X.txt` came back empty packs a payload with no
  preimage at exit 0. MEASURED: `--expect preimage` → *"unknown --expect kind
  \"preimage\"; want one or more of descriptor, cosigner, transaction, mnemonic,
  secret"*, and `--expect secret` is NOT satisfied by a preimage plate. Neither
  `--pack-preimage` (a loosening flag whose no-op case is a warning) nor
  `--expect` can make a preimage's presence a REQUIREMENT. The remedy is one
  line -- add `preimage` to `sysw::expect`'s vocabulary, satisfied by
  `Class::Preimage | Class::Phrase` -- and it is satisfiable, so it does not fall
  under the flag's own `address`/`passphrase` exclusion. A later `me` cycle.
- **The Hashlock plates flow returns to an UNMARKED list after a successful cut**
  (round 0, journey N-2). Its loop `continue`s back to the same rows; a phrase
  row does change (`phrase record 1 (derive to see the digest)` → its digest) but
  a preimage row is identical before and after. §13's cross-run entry above is
  about *previous runs*; within one flow the information exists and is discarded,
  and a 32-minute cut makes an accidental repeat expensive. One field on
  `hashlockPlatesRecord` and one word on the row; a later device cycle.

---

## §14. Citations -- measured at fork `fb0dd04`, engrave `a0f832d0`, ms `504ff46`

**The rows the plan round added were re-grepped at fork `fb0dd04`, mnemonic-secret
`504ff46` and engrave `75f00b56`** (the plan's me baseline; identical at engrave
master `55950604`), which is what `scripts/plan-staleness-check.sh` compares
against.

| claim | where |
| --- | --- |
| the phrase route drops X on return; HOLD assigns the digest and notes provenance | `gui/composer_hashlock.go:17-20`, `:43-86`, `:69`, `:70` |
| the composer's own record that H6 makes false | `gui/composer_hash.go:27-28`; `SPEC_wallet_policy_composer.md` §6c, §14 |
| `composerState`, `phraseDigests`, the nil-map rule, the value-set predicate | `gui/composer_state.go:26-79`, `:53`, `:46-50`, `:280-286`, `:299` |
| state construction, the ONE defer, the 96 B second-defer measurement | `gui/composer_flow.go:20-23`, `:48`, `:59`, `:53-57` |
| the engrave step, the ms1-secrets-first append, the census call, `bundleEngrave`'s marking parameters | `gui/composer_flow.go:335-394`, `:381`, `:389-390`, `:393` |
| `Which hash?` rows, the label-keyed switch, the `taking` predicate, the row form | `gui/composer_hash.go:157-175`, `:184-224`, `:194`, `:38-41`, `:161` |
| the phrase route: method pick and reconcile, the two things a payload phrase must skip | `gui/composer_hashlock.go:43-86`, `:53`, `:83` |
| the pick screen with row touch targets, its row cap, the text band | `gui/composer_paged.go:278`, `:243`, `:49-54` |
| the `show` key: a keyboard-grid key, and it LATCHES | `gui/passphrase_keyboard.go:141`, `:221` |
| `syswOfferAlt` returns without drawing | `gui/sysw_session.go:270-278` |
| the minted mk1 key cards the marking reaches | `gui/composer_cards.go:69-74` |
| `composerEveryPathHashed` and the §8h guard already on the record | `gui/composer_state.go:257-259`; `gui/composer_shape.go:442-443`; `gui/composer_hashlock.go:70-79` |
| the glyph rule that BLANKS a frame | `font/bitmap/bitmap.go:33` |
| the door, its counts, its conditional route, the no-keys lead | `gui/composer_door.go:37-52`, `:41-56`, `:86-90` (`composerDoorHasConsumablePolicy`), `:98-119` |
| the census and its completeness claim; the passphrase-plate exclusion precedent; the signature that must change | `gui/multisig_build_census.go:63-73`, `:88-93`; `gui/composer_census.go:86` |
| `confirmReviewScreen` (paged) | `gui/multisig_build.go:1895` |
| `bundleEngrave`, the plate plan, the set-level abort, the marking rule | `gui/bundle_flow.go:616-657`, `:466`, `:625-631`, `:565-579`, `:780-792` |
| the standalone-plate cut pattern | `gui/passphrase_flow.go:570-588`, `gui/gui.go:3296` (`NewEngraveScreen`), `:3620` (`toPlate`) |
| `validateMdmkStrings` offers TEXT+QR / TEXT ONLY / QR ONLY for one string | `gui/gui.go:2626-2648` |
| the admission table and its one-program rule; the flags derive from `IsSecret` | `gui/sysw_admit.go:1-14`, `:32`, `:64-73`, `:143-162` |
| F1/F2's text at load | `gui/sysw_load.go:261-290`, `:283` |
| the session holds Public AND Secret records | `gui/sysw_session.go:54`, `:79-110` |
| `sysw.Open` runs no admission | `sysw/open.go:36-73` |
| the F-474 arm belongs to the Sealed Payload | `gui/unlock_flow.go:98`, `gui/unlock_kdf.go:415-420`, `:433`; `seal/open.go:149` |
| the classifier, the reserved prefixes, `IsSecret` | `sysw/record.go:14-21`, `:24-54`, `:60-66`, `:111-139` |
| the vendored lockstep test | `sysw/composer_records_test.go:64-75` |
| the composer record parsers and the hex rule | `sysw/composer_records.go:24-31`, `:58`, `:82-97`, `:100`, `:136-160` |
| H0's ms1 inertness in the classifier | `sysw/classify.go:116-127` |
| `IsPreimage` (no id), its stated trade, `DecodeMS1Preimage`, `Split` | `codex32/mspayload.go:78-101`, `:113-128`; `codex32/codex32.go:394-401` |
| the five `DecodeMS1` callers and the two door refusals | `gui/ms1_decode.go:22`, `gui/codex32_polish.go:106`, `:232-235`, `gui/singlesig_verify.go:185`, `gui/multisig_verify.go:1237`, `bundle/verify.go:138`, `gui/scan.go:89` |
| the SIX live `IsPreimage` call sites, so a plan-time grep re-decides none of them | `gui/codex32_polish.go:232`, `gui/scan.go:89`, `gui/unlock_session.go:197`, `seal/record.go:260`, `:298`, `sysw/classify.go:126` |
| the phrase rule, the ms1 shape test, the constants | `hashlock/hashlock.go:21,24,27`, `:92-111`, `:122-148` |
| the passphrase plate: struct, space mark, legend, QR rules, bands, stacking, gap | `backup/passphrase.go:21`, `:23-49`, `:70-74`, `:93-100`, `:112-114`, `:168`, `:239`, `:250-253` (the two-line band), `:275` (the character wrap), `:283-292` (`:289`) |
| the 64 mm band ceiling | `backup/passphrase_test.go:521` |
| plate constants, the size ladder, the capacity functions, `EngraveText`'s refusal | `backup/backup.go:71`, `:83`, `:88-96`, `:388-400` |
| the free-text QR scale and the title cap | `backup/fit.go:16-19`; `TitleString` at `backup/backup.go:111`, `MaxTitleLen` at `:71`, `TestTitleCapFitsAtEveryRung` at `backup/freetext_test.go:109` |
| `ftWarnQR` and the free-text confirm | `gui/freetext_flow.go:1214-1216`, `:1362-1402`, `:1485` |
| the passphrase confirm | `gui/passphrase_flow.go:497` |
| `ConstantQR`'s bound and its "raise both together" rule; `bitmapForQRStatic`; `engrave.QR` | `engrave/engrave.go:418-426`, `:394-414`, `:277`, `:377-379`, `:430-470` |
| `constantTimeQRModules` and the v5 fuzzing provenance | `engrave/engrave.go:349-374`, `:363-371` |
| `engraveModule`'s scale switch and its panic; `centerOf` | `engrave/engrave.go:689-709`, `:628-631` |
| `ConstantQRCmd.Engrave`'s `for range nmod` and `DelayMove` padding | `engrave/engrave.go:633-687` |
| the fork's own "a raise needs a fuzzed entry" record | `engrave/engrave_test.go:696-707` |
| **the two SHIPPED tests the raise falsifies** (plan round, finding 6), both rewritten at v10 (dim 57) rather than deleted -- the fail-closed property and the beyond-reach property are unchanged and only the version each is asserted at moves | `engrave/engrave_test.go:544-568` (`TestConstantQRLargeVersionsFailClosed`, which asserts dim 41 is REFUSED and whose payload is 120 bytes); `backup/passphrase_test.go:774-791` (`TestPassphraseQRTooLong`, whose 200-character passphrase is dim 53) |
| the third record the raise falsifies: the boundary comment calling dim 41 *"deliberately unsupported"*, rewritten with them so a 100-character PASSPHRASE still never reaches it and the passphrase plate's scale-3 goldens do not move | `engrave/engrave_test.go:696-707` |
| the row pitch that decides how many picker rows a page holds -- 29 px, not the 23 px label | `gui/composer_paged.go:147` |
| `composerReadScreen`, the band-correct twin of `confirmReviewScreen`; the shipped W-3 gate the census joins | `gui/composer_paged.go:173`; `gui/composer_paged_geometry_test.go:141-142` |
| the door's ONE caller, where the fourth route must be dispatched | `gui/wallet_policy.go:46-58` |
| the join guard and its exemption table (§2.2 item 6) | `gui/composer_join_test.go:35`, `:87-90` |
| `md.Compose`'s refusal of a wholly key-less composition, which is why §11.7's walk adds a second path | `md/compose.go:93` (`ErrComposeNoKeyedPath`) |
| the phrase rule and the ms1 shape test as `ms-cli` owns them TODAY, before §3.1 moves them into the codec | `crates/ms-cli/src/hashlock_phrase.rs:118`; `crates/ms-cli/src/argv_guard.rs:148` |
| the `ms-codec` dependency §3.1 bumps; the release checklist and the H1 precedent it follows | `crates/me-cli/Cargo.toml:53`; `mnemonic-secret/design/RELEASE_PROCESS.md` items 1, 2, 3, 7, 8; `mnemonic-secret/CHANGELOG.md` `## ms-codec [0.8.0]` |
| the argv guard, its bearer arm and the predicate that routes the two classes into it (§3.6) | `crates/me-cli/src/main.rs:551`, `:566`; `crates/me-cli/src/sysw/record.rs:102-104`, `:118-120` |
| the corpus pins the plan round moves, each carried IDENTICALLY on both sides | `crates/me-cli/tests/sysw_composer_records.rs:399` and `sysw/testdata/record_class_vectors.provenance.json` (the class corpus); `hashlock/hashlock_test.go:13` (the ms corpus); `crates/me-cli/tests/codex32_seam.rs:25` and `sysw/codex32_seam_test.go:30` (the seam corpus, which does NOT move -- see §4.2's note) |
| machine parameters | `internal/sh2/params.go:43-53` |
| the fit gate: per-body render, headroom search, margin 80, "no capacity constant" | `gui/modal_fits_test.go:202`, `:183`, `:52`, `:33-35` |
| the composer confirm surface and its HOLD | `gui/composer_shape.go:77`; `gui/composer_copy.go:36-38` |
| the §8h forms and their chooser; the reconcile body | `gui/composer_copy.go:182-186`, `:520-525`, `:527-532`, `:484-490` |
| the single-sig marking this reuses | `gui/singlesig.go:365-374` |
| the stub labels the locator copies | `gui/composer_stub.go:54`, `:67`; *"no id yet"* at `gui/composer_engrave.go:80` |
| `me sysw pack`: the flag pattern, the preimage refusal, admission, sealing, `--no-passphrase` | `crates/me-cli/src/main.rs:126`, `:244`, `:932-937`, `:2353-2410`, `:2805-2810`; `crates/me-cli/src/sysw/mod.rs:209-211`, `:335`, `:463-470` |
| the host preimage predicate and its id/kind exclusion | `crates/me-cli/src/seal/record.rs:287-320`, `:292-295`; `id_kind_mismatch` at `:269-276`; the Sealed Payload refusal at `:130-136` |
| the shipped `TagKindMismatch` and `PreimagePlate` bodies; their `unknown_reason` arms | `crates/me-cli/src/main.rs:2799-2804`, `:2805-2810`; `crates/me-cli/src/sysw/mod.rs:207`, `:210` |
| `Admission`, `classify`/`classify_with`, `admit_check`; the F-246 ordering and the `--expect` admission precedent | `crates/me-cli/src/sysw/mod.rs:233-236`, `:246-248`, `:252`, `:463-470`; `crates/me-cli/src/main.rs:1474-1494`, `:1696` |
| the host composer records, Rust-primary | `crates/me-cli/src/sysw/composer_records.rs:28-32`, `:306` |
| the vendored corpus and its provenance pin | `sysw/testdata/record_class_vectors.provenance.json` (commit `c05074f1…1dbd`, sha256 `5b3960ca…b312`, 47 rows) |
| the derivation constants and the phrase cap, Rust-primary | `crates/ms-codec/src/hashlock.rs:27,30,32`; `crates/ms-cli/src/hashlock_phrase.rs:24`, `:118` |
| `ms hashlock`'s method line, its grouped plate output, `--out` | `crates/ms-cli/src/cmd/hashlock.rs:281-288`, `:299-306`, `:340,348` |
| the ms1 preimage string is 75 characters, id `hash`, kind `0x03` | `SPEC_ms_hashlock` §1; corpus `crates/ms-codec/tests/vectors/hashlock-v0.8.json` `kind[0]` (`preimage_hex`, `digest`, `ms1`, `entr32_pair_ms1`) |
| the Rust encoder the plate comes from, and its id/kind refusal | `crates/ms-codec/src/encode.rs:16`, `:24-31`; `crates/ms-codec/src/payload.rs:23-28`; `crates/ms-codec/src/envelope.rs:248`, `:261`; used at `crates/ms-cli/src/cmd/hashlock.rs:298` |
| the fork's entr-only encoder and the `NewSeed` primitive | `codex32/msencode.go:17-31`; `codex32/codex32.go:279` |
| F-132 (a preimage required, absent from the backup, unmentioned by it) | `design/FOLLOWUPS.md:4298` |
| F-483 (the phrase in an unwipeable Go string) | `design/FOLLOWUPS.md:16003` |

---

## §15. The rulings this spec applies

| ruling | where it lands |
| --- | --- |
| decisions 1-9 (2026-09-05) | throughout; §1 lists what each fixes |
| A1 constant time kept; raise the encoder | §7 (ceiling **v9 = 53 modules**, correcting "v7") |
| A2 QR only with the phrase form | §6.4; B5 of the draft report is declined |
| A3 QR below the text, passphrase stacking | §6.4, §6.5 (scale **2**, not the passphrase plate's 3 -- measured), and §7.3, which is what makes scale 2 reachable |
| A4 form-specific bands | §6.2 (three distinct literals in four cells) |
| A5 the `mk1 stub` locator, policy then template | §6.3 |
| A6 the classes are secret; seal by default | §3.4, with the F-474 premise corrected; §3.2 gates ADMISSION so `decide_sealing` sees them |
| A7 the `hash` id on the H6 admission path | §4.3 (three-id partition), §8.1.1's conditional sentence, §8.1.2 |
| A8 the free-text warning names the route | §9 (fits as ruled; no wording change) |
| B: preimage plate cut first + abort arm | §5.4 (two windows), §8.4a and §8.4b |
| B: its own layout, own census line, excluded from the completeness claim | §6.1, §5.3 items 1-2 |
| B: retain phrase + method + preimage keyed by digest, scrubbed at flow exit | §2.2 |
| B: unused retained material listed, both provenances | §5.3 item 4, §8.3 |
| B: lazy derivation of payload phrase records | §5.1 |
| B: review at the engrave Done, per-plate form and QR, declined plates named | §5.3, as a pick step (A) plus a read-only census (B) |
| B: `PREIMAGE REQUIRED` marking | §10.3 (md1 AND mk1, stated and tested) |
| B: provenance-aware §8h and reconcile | §10.1; §10.2 records that provenance is correct BY CONSTRUCTION and needs no guard |
| B: `phrase:` at `progWalletPolicy` only, never Password | §4.1 |
| B: the QR toggle carries an `ftWarnQR`-shaped warning | §8.5 |
| B: a `SpaceMark` equivalent | §6.1 |
| B: wire shape and method spelling follow the siblings, Rust first | §3.1 |
| B: the ms hashlock corpus owns the QR text rows | §11.2 |
| C-1 ruling (2026-09-05): the ms1 preimage encoder, Rust first | §3.5 -- the Rust encoder already exists (`ms_codec::encode`) and is cited; the Go port is the deliverable |
| journey Q1-Q14 | Q1 §5.4+§8.4a/b, Q2 §5.3, Q3 §6.1, Q4 §5.3.4, Q5 §8.5, Q6 §6.3, Q7 §5.1, Q8 §10.3, Q9 §2.2, Q10 §10.1-§10.2, Q11 §4.1, Q12 §5.3.6, Q13 §5.3 (masked; no reveal control -- item 7), Q14 §9 |

---

## §16. R0 round 0 folded here

Three lenses ran against `a0f832d0`: **fidelity** (4C/7I/7M/3N,
`design/agent-reports/hashlock-H6-spec-R0-r0-fidelity.md`), **journey**
(1C/7I/7M/2N, `-journey.md`), **tests** (0C/3I/1M/1N, `-tests.md`). Every
Critical and Important is folded or declined below with its reason; Minors and
Nits are folded where they were truth fixes. **Every number the fold introduces
or replaces is its own measurement**, taken at fork `fb0dd04` in
`/scratch/code/shibboleth/.tmp/h6-fold`.

### Criticals

| finding | change | measurement |
| --- | --- | --- |
| fidelity C-1 = tests I-3 -- the raise omits `constantTimeQRModules` | §7.2 item 4: a normative arm per version, the fuzzing protocol named, the fold's 216,000-payload floor tabulated, v7 flagged as the under-converged entry; §11.3 splits the budget row from the regression row | reproduced the failure (all four dims return 0, every raised version errors); fuzzed 216,000 §8.6 payloads per dim: max **813 / 945 / 1161 / 1369** at v6/v7/v8/v9, ratios 0.4836 / 0.4667 / 0.4835 / 0.4874, **0** `findPath` errors in 864,000 |
| fidelity C-2 -- scale 2 panics `engraveModule` | **Scale 2 is kept and the engraver gains the arm** (new §7.3, §7.4); §6.5 consequence 1 states the dependency and why re-fitting is unavailable | implemented `case 2` and ran it: **5 commands per module** (the scale-3 constant), ink exactly `[p*2sw, p*2sw+2sw]` = **0.6 mm** at every probed position; scale 3 at 53 modules is 47.70 mm and over budget at every rung |
| fidelity C-3 -- `--pack-preimage` has no stated place | §3.2: **admission, never classification**. `Admission.pack_preimage`, a second `admit_check` rule, `decide_sealing` UNCHANGED, one corpus truth | read `decide_sealing` (`main.rs:2353-2410`), its call at `:1696` without admission, the admission value at `:1478`, `classify`/`classify_with` (`sysw/mod.rs:246`, `:252`), `admit_check` (`:463-470`), the lockstep test (`sysw/composer_records_test.go:64-75`) |
| fidelity C-4 = journey I-1 -- the review has no input budget | §5.3 splits it: a `composerPickScreen` step (A) then the read-only census (B); Back contract stated; the `show` toggle DROPPED with its true mechanism | `confirmReviewScreen`'s three bound buttons and label-not-`Clickable` rows (`gui/multisig_build.go:1895`); `composerPickScreen`'s row targets (`gui/composer_paged.go:278`); pick rows measured 128/138/180/196 px, all one line in the 411 px band; `ppReveal` is a keyboard key and LATCHES (`gui/passphrase_keyboard.go:141`, `:221`) |
| journey C-1 -- no ms1 preimage encoder | New §3.5, per the operator's ruling **as corrected by the controller**: the Rust encoder already exists and is what `ms hashlock` prints the plate with, so §3 gains NO Rust-first deliverable; the **Go** wrapper `codex32.EncodeMS1Preimage` is the gap, with the corpus row as its vectors | the wrapper reproduces the corpus `kind` row **byte for byte** (75 chars, id `hash`), round-trips through `DecodeMS1Preimage`; and `NewSeed` will mint a kind-0x03 payload under id `entr` **unrefused** -- the copy-paste hazard, reproduced, which is why the id is fixed and §11.1 asserts it on the output |

### Importants

| finding | change |
| --- | --- |
| fidelity I-1 -- §8.4's clause could never fire | §8.4 rewritten: both arms are the composer's screens, `bundleAbortWarningText` UNCHANGED (its `:610-616` comment prohibits the variadic tail the plumbing would need) |
| fidelity I-2 + M-2 -- the band mutation cannot fail; the width justification is false | §6.3 re-justified on LINE COUNT with the shipped two-line record; §11.4's mutation replaced with a placement one. Measured: locator rows are 6/24/27/29/30 chars against a 32-char cap -- **all fit** |
| fidelity I-3 -- §10.2 guards an unreachable screen | §10.2 rewritten: **no guard, no test**; the property is by construction (one call site, inside `hashlockPhraseRoute`) and the reason is recorded so it is not re-opened |
| fidelity I-4 -- §10.1 asserts a decision not yet made | Both arms moved to the present tense of what is HELD; §11.5 gains a decline row. Re-measured: **185/360** and **186/360** |
| fidelity I-5 -- the `preimage_plate` sentence is false | §4.3 corrected and given the three-id partition table; §8.1.2 becomes the EXISTING `PreimagePlate` body rewritten, not a new arm, because `entr` never reaches it |
| fidelity I-6 = tests I-2 -- non-ASCII and two unreproducible numbers | §8's preamble carries the blanking mechanism and the host/device split; every device body is ASCII; §11.5 asserts it mechanically. §8.1.2: **236/320** (ASCII), **340/223** with the collision sentence -- the claimed 165/397 is withdrawn. §8.4a alone: **75/476** -- the claimed 288/244 is withdrawn |
| fidelity I-7 = journey I-2 -- the flow cannot compute a phrase digest | §5.2 gains the four-part derive rule: list without deriving, derive once on pick behind the countdown, keep it in a flow-local, print the locator from it |
| journey I-3 -- the orphan check misses `phrase:` records | §3.3 item 3 extended to both carriers with the two hand-build errors named; §8.2.3 gains the phrase body; §11.1 gains the space-after-comma mutation |
| journey I-4 -- three refusals, none naming the collision | §8.1.1's final sentence is conditional on id `hash`; §8.1.2 gains the 1-in-256 sentence and fires from the no-flag path too. Measured to fit |
| journey I-5 -- the marking reaches mk1 too | §10.3 states md1 **and** mk1, gives F-132 as the reason, and §11.5's row now names the mk1 cards -- without which every assertion passed either way |
| journey I-6 -- the Password refusal is invisible | New §8.8, measured **165/397**, with a §11.5 row and §12 item 7 |
| journey I-7 -- the new order opens a second abort window | §5.4 tables both windows; §8.4b is its arm, measured **86/476**, with the re-run/second-bearer-plate reasoning that makes it necessary |
| tests I-1 = fidelity M-3 -- the 5.0 mm advance | §6.5's cell corrected to **3.333 mm**; measured 21,333 units = 3.3333 mm |

### Minors and Nits folded (truth fixes)

fidelity M-1 (the `composerState` literal, plus the stale `:34` comment noted),
M-2 (with I-2), M-4 (the character-wrap rule stated, with its reason and a
§11.4 row), M-5 (**all five drifted citations corrected**: `freetext_test.go:109`,
`composer_engrave.go:80`, `composer_door.go:86-90`, `composer_flow.go:381`,
`TitleString` at `backup/backup.go:111`), M-6 (the warning site and the F-246
ordering), M-7 (the census block routed through `composerPageLines`, with the
measurement: new rows 405-448 px -- **WITHDRAWN by the plan round, because those
widths are digest-dependent; §5.3 item 2 carries what replaces them** -- the
SHIPPED completeness line **459 px**, which still reproduces, against a 374 px
centred-safe width, so the overlap is pre-existing and H6 does not join it), N-1 (**three** distinct band literals, not four), N-2 (the
inventory of **six** live `IsPreimage` call sites in §14), N-3
(`composerCensusLines`' signature must gain the plate list).
Journey M-1 (the string form's rendering decided and measured), M-2 (the
selector declined with a reason in §8.6 rule 5), M-3 (the §8h guard recorded and
the load placed on §8.3), M-4 (the `(in payload)` annotation, measured to one
line at **41 chars / 343 px**; the longer wording it replaced is **2 lines**),
M-5 (the door lead), M-6 (§12 item 3 packs the `hash:` record; the orphan path
becomes item 5a and §8.2.3 distinguishes incomplete from contradictory), M-7 (the
text-is-authoritative statement and §12 item 8's scan gate), N-1 (§5.2 forbids
reusing §8.4's arms), N-2 (the derive-only path named in §5.1). The journey
lens's closing note is folded as the **fourth** falsified shipped record in §0.

### Declined, with reasons

- **fidelity C-3's prescribed remedy** (thread `admission` into `classify_with`
  and `decide_sealing`). The DEFECT is folded; the remedy is not. It adds a
  second call site that must be kept in step, which is the exact failure the
  shipped `--expect` comment records at `main.rs:1488-1494` (*"a false refusal
  carrying a false message, on the funds path"*), and it forces the corpus to
  carry two answers per row. Gating admission leaves `decide_sealing`
  byte-unchanged and keeps one truth per row.
- **journey C-1's option (a) as literally worded** (a new Rust-first encoder
  deliverable). The controller established that `ms_codec::encode` already exists,
  already carries the id/kind refusal, and is already the encoder `ms hashlock`
  prints the plate with, with the corpus row already pinned. Adding a Rust
  deliverable would have specified work that is shipped. The Go half -- the
  actual gap -- is folded in full.
- **journey I-1's `show`-toggle alternatives** (a Center-press latch). Neither is
  taken: item 7's own argument is that the plate preview is the screen that shows
  the phrase, so the review shows none, and no new control is invented for a
  screen with no spare input. Secret-handling and non-gating either way
  (2026-08-27 ruling).
- **journey M-1's grouping suggestion.** Measured and declined: grouping in tens
  does not fit at 6.0 mm (8 rows against 4) and would move the form to 5.0 mm and
  change every golden. The ungrouped rendering is stated instead, with its reason.
- **journey M-2's fourth QR line.** The QR has room (226 bytes is still under
  v10's 231 threshold) but the plate's job is tool-independent reproduction;
  declined in §8.6 rule 5 with the reasoning, so it reads as a decision.
- **tests M-1** is explicitly not a finding against the spec (a reconstruction
  gap in `bundleAbortWarningText`'s unpinned caller context). It is moot now that
  §8.4's arms are measured alone; recorded in §8.4.
- **fidelity M-7's "measure and accept" option.** The overlap is pre-existing,
  but adding five more rows to it is the W-3 class the paged screens exist to
  remove, so the block is routed through `composerPageLines` instead.

### Not consistent, and named

- ~~**The four `constantTimeQRModules` values are NOT settled by this spec.**~~
  **CLOSED by the plan round.** The floor stood, the four 32-minute campaigns ran
  (43,458,059 payloads, zero `findPath` failures), every observed maximum cleared
  the floor, and the entries are §7.2 item 4's `843 / 1013 / 1199 / 1399` --
  measured in the gated tree, not pasted. What the round ALSO found is that the
  mutation this spec paired with them could not fire, which §11.3 now replaces
  with a pin.
- **§12 item 8's scan is unrun.** 53 modules at 0.6 mm on steel has no precedent
  in this tree and the SH2 has no camera, so nothing before a physical test plate
  can settle it. The spec states the fallback (text-only phrase form) so a
  negative result is a decision rather than a re-plan.


---

## Plan-round fold

**Written by the plan-gate fold author (opus).** The R0 gate closed this spec
0C/0I under four lenses; the PLAN round then built it. Nine things the plan
author could not implement as this spec wrote them
(`design/agent-reports/hashlock-H6-plan-author-report.md` §4) and seventeen the
build gate had to change to make Tasks 8b-12 compile and run
(`design/agent-reports/hashlock-H6-plan-gate-8b-12.md` §6) are folded below.
**Every number in this section is this author's own measurement**, re-taken in
private copies of the three gated trees
(`/scratch/code/shibboleth/.tmp/h6-gate`, `-ms`, `-me`) rather than carried from
either report: modal bodies through `assertModalBodyFits` on the renderer
production uses, row widths through `widget.Labelw` in `composerTextBand`, plate
geometry through `backup.CharsPerLine` / `hashlockLayoutFor`, budgets through
`constantTimeQRModules`, corpora through `sha256sum`, and firmware through
`tinygo build -size short`. Each replaced mutation was RUN against the tree it
names, and its failure is quoted.

### The plan author's nine

| # | change | measurement |
| --- | --- | --- |
| 1 | §1 item 3b, §3.1 and §12 item 0: the phrase rule, `looks_like_ms1` and §8.6's `qr_text` move into `ms-codec`; a PUBLISHED **0.9.0** is the stage's first deliverable, through `RELEASE_PROCESS.md`; `me-cli`'s dependency is bumped after it | `validate_phrase` is `crates/ms-cli/src/hashlock_phrase.rs:118`, `looks_like_ms1` is `crates/ms-cli/src/argv_guard.rs:148`, and `crates/me-cli/Cargo.toml:53` depends on `ms-codec = "0.8"` and nothing of `ms-cli` -- **`me` cannot reach either**. The version is forced by checklist item 1 (a corpus sha change is `0.X+1.0`): the corpus moves to `4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4`, measured on both copies, where H1's `## ms-codec [0.8.0]` entry recorded `a46c197a…11d30` |
| 2 | §11.4's method-line mutation replaced by the **79-character** threshold | 74, 75, 76, 77 and 78 characters ALL still fit -- five runs, five passes, the gate green each time; **79** reds with `11 rows at 3.0mm need 427520 units against a budget of 416000` (66.80 mm against 65.00 mm). §6.5 consequence 3's own number was already the threshold |
| 3 | §11.3's budget mutation replaced by a **PIN** of the four entries and their campaigns | with the arms at each campaign maximum minus one, the fuzzed row **still passes**; the pin reds with `constantTimeQRModules(53) = 1378, want 1399 (1379 observed + 20 buffer)`. The entries measured in the gated tree: **843 / 1013 / 1199 / 1399**, the five shipped ones 171 / 266 / 391 / 547 / 684 untouched |
| 4 | §4.3 and §11.1: `preimage_plate_admissible` takes **three** conjuncts -- `preimage_plate`, the id `hash` compared CASE-SENSITIVELY, and a payload of exactly 33 bytes beginning `0x03` | both extra conjuncts run as mutations against the SHIPPED `a_preimage_plate_is_named_not_misdiagnosed`: dropping the well-formedness conjunct AND comparing the id with `eq_ignore_ascii_case` each give `left: Err(PreimageNotAdmitted(0, Preimage))` / `right: Err(Unclassifiable(0, PreimagePlate))`. Go was already right (`len(d) == 33 && d[0] == 0x03`); the Rust needed narrowing, which is the Rust-primary rule in reverse |
| 5 | New §3.6, and §12 item 3 -- the one acceptance item that packs a record -- rewritten to `--in`: the classes are BEARER, so `argv_secret_guard` refuses either carrier on the command line, and the guard's message named the wrong material | `is_bearer` (`crates/me-cli/src/sysw/record.rs:102-104`) gains both classes and `is_argv_forbidden` (`:118-120`) is the OR of `is_secret()` and `is_bearer()`; the guard is `crates/me-cli/src/main.rs:551` and the false bearer arm is `:566`. §12 item 3's own invocation could never have been walked |
| 6 | §14 lists the **two shipped tests the raise falsifies**, with their new expectations at v10 (dim 57), plus the third record rewritten with them | `engrave/engrave_test.go:544-568` asserts dim 41 is REFUSED (its payload is 120 bytes); `backup/passphrase_test.go:774-791` uses a 200-character passphrase, which is dim 53. Both properties survive; only the version moves. Ranges re-grepped at `fb0dd04` |
| 7 | §2.2 item 6: the retention field and its consumer land in ONE commit, and the exemption table is not the instrument | the retention-only red was measured twice (plan author, then build gate), each naming `[composerHoldHashlockMaterial]` from one shard; MEASURED here, the guard PASSES on the fully wired tree with only the pre-existing exemption logged. The guard is `gui/composer_join_test.go:35`; its exemption table (`:87-90`) is for a consumer DEFERRED with a follow-up number, and this consumer is in this stage |
| 8 | §6.5 states ONE body-row order, the one the gated layout draws: method line, blank, locator, blank, secret | the layout's own segment list, measured: method (2 rows at 3.0 mm), blank, `path 2` + `hash  …` + `mk1 stub (template): …` (3), blank, phrase (3) = **2 + 1 + 3 + 1 + 3 = 10 rows = 30.00 mm**, total 63.80 mm against a 65.00 mm budget, **1.20 mm spare**. The count is 10 either way, so **no measurement in §6.5 moved** |
| 9 | §4.2: exactly one corpus row moves (`preimage-plate-0x03`, `Unknown` -> `Preimage`), its `entr` sibling stays, and the corpora H6 touches are tabulated with their pins on both sides | class corpus `3575ccb0…a1c4abaf`, 68 rows, identical in `crates/me-cli/tests/sysw_composer_records.rs:399` and the fork's provenance pin; ms corpus `4f1819cd…aba21d4`, identical in both copies. **MEASURED and stated so nobody edits it: the two-repo `codex32_seam_vectors.json` does NOT move** -- its `device_admits` column is `Classify(s) == ClassCodex32Secret`, and `ClassPreimage` is not that, so both columns stay `false` and its sha `2c2fbb3f…6bd541b` stands |

### The build gate's seventeen

Five are corrections to the PLAN's own blocks and touch nothing normative (F1
`sysw.Record` did not exist, F2 an uncompilable sketch, F7 two signatures, F16
three test paths that do not exist, F17 an annotation inside a cited fragment);
they are recorded in the plan. The twelve that changed what this spec says:

| # | where | change and measurement |
| --- | --- | --- |
| F3 | §5.1 | a payload PREIMAGE record gets its OWN confirm body -- `composerCopyHashlockConfirm` would draw `method: hardened   chars: 0` on the screen that gates funds. **274 drawn / headroom 186** through `confirmWarningBody` + `composerConfirmBody`, longest variant: the tightest body this stage adds, 106 characters clear of the margin |
| F4 | §5.1 | `composerHashRows` gains `st *composerState` (nil-safe); band 3's row form depends on what THIS composition derived, answered out of `hashlockHeld` rather than a second map |
| F5 | §5.1 | the `(in payload)` annotation is EXACT for a preimage record and becomes true for a `phrase:` record once derived -- an underived one's digest needs the KDF that lazy derivation forbids at row-build time |
| F6 | §5.1, §11.5, §13 | **the first page holds FIVE rows, not six.** 23 px is the LABEL height; the PITCH is 29 px (`gui/composer_paged.go:147`), the content box 224 px. Measured on a six-row payload: page 1 draws **5** touch targets and `No hash lock` is on page 2, reachable by paging. Filed |
| F8 | §8.6 rule 4a | `hashlock.MethodLine` and `hashlock.QRText` are deliverables -- §6.1's plate has `Method`/`QRText` as caller-filled fields and nothing produced them. Built from `hashlock.Iterations` / `Salt` / `PreimageLen`, pinned against the vendored `qr_text` rows. Measured: the hardened line is **73 characters** |
| F9 | §5.4 | §8.4b's mechanism is a CUT COUNTER: without it the arm can never fire, because an unconditional `composerAbortNoPreimage` and an unread `bundleEngrave` result each make it unreachable |
| F10 | §5.3 (B), §11.5 | the census is drawn by `composerReadScreen` (`gui/composer_paged.go:173`), not `confirmReviewScreen` -- the first draft named a screen that contradicted its own band requirement. The gate is an AST assertion; MUTATION run: *"composerEngraveStep draws through confirmReviewScreen, which wraps at dims.X-2*8 … every census row over 374 px has its right edge under a navigation button (W-3)"* |
| F11 | §5.3 item 2, §8.3 | the five census-row widths are DIGEST-DEPENDENT; 405/423/441/447/448 px are withdrawn. Measured: panel **480**, nav column **427**, `confirmReviewScreen` wrap **464**, centred-safe **374**, band **411**, the SHIPPED completeness line **459**; the new rows on the gate's fixture are **419 / 385 / 434 / 442 / 441** px at the wrap and **387 / 385 / 404 / 383 / 387** in the band. The pinned property is digest-independent |
| F12 | §10.1, §11.5 | §10.1's fourth arm is chosen on `every` held path having a phrase, not `at least one` -- its body says *"for each one"*. MUTATION run: *"a mixed composition claims to hold the phrase for each path"* |
| F13 | §5.3 step (A), §11.5 | the masked lead is TWO lines. MUTATION run at four lines: *"page 1 draws 3 of the four rows; the lead is spending them"*, and the row displaced is `do not cut this preimage` |
| F14 | §5.3, §11.5 | the plate order is deterministic -- paths in path order deduplicated by digest, then unused digests in digest order. Ranging `hashlockHeld` would reorder the screen on every frame |
| F15 | §5.2 | `composerDoorHasPreimage` takes the SESSION (`composerDoorHasConsumablePolicy`'s own shape, `gui/composer_door.go:86-90`; the first draft's `:93-97` is corrected), and the door's ONE caller `gui/wallet_policy.go:46-58` must dispatch the fourth route |

**The gate's two deliberate non-changes are folded as records, not fixes.**
§8.3's `plate(s)` stays verbatim and §13 files it; the bodies with no §8
blockquote are tabulated with their measurements in the new **§8.9** -- MEASURED
as SEVEN of the twenty this stage adds: four modals (**274/186**, **99/455**,
**80/476**, **46/513**) and three leads -- and filed the same way. Changing
shipped copy inside a build gate puts the string and the document it is diffed
against out of step, which is the one thing a gate must not do.

### Also folded from the round

- **§7.2 item 4 and §16:** the four fuzzing campaigns RAN at plan time and every
  observed maximum cleared this spec's floor, so §16's "not settled" item is
  closed. **The campaign itself is the plan author's measurement, cited rather
  than re-run** -- 43M payloads is 128 minutes of wall clock and re-running it
  would not make it more true. What THIS round measured is the outcome the spec
  now states: `constantTimeQRModules` returns 843 / 1013 / 1199 / 1399 in the
  gated tree, the five shipped entries are untouched, and the in-suite fuzzed row
  passes against fresh content.
- **§11.6:** the stage's firmware delta is **+44,372 B flash / +416 B RAM**
  (1,599,208 -> 1,643,580 B; 62,856 -> 63,272 B), both ends built here from a
  `git ls-files` copy of `fb0dd04` and from the wired tree. The plate layout is
  **4,560 B** of it, measured by stubbing its one production caller
  (1,639,020 B).
- **§11.7:** the walk's composition needs a SECOND, keyed path, because
  `md.Compose` refuses a wholly key-less one (`md/compose.go:93`); and the census
  measures **11 lines over 3 pages** on the gate's fixture, with §8.3's heading on
  page 2 and its last row on page 3, so the walk pages to reach the block it
  asserts against.
- **§5.2's citation** of `composerDoorHasConsumablePolicy` corrected from
  `:93-97` to `:86-90`, re-grepped at `fb0dd04`.

### Nothing folded here was declined

Every one of the twenty-six items above is either normative spec text now or a
recorded non-change with its reason. The two items the round could not settle
are unchanged and still named in §16: **§12 item 8's physical QR scan**, which
needs a test plate and a phone, and the emulator walk, which §11.7 specifies and
Task 12 has run four times but which is not acceptance on hardware.

### The plan's R0 round 0, folded here

Three lenses ran against the PLAN at engrave `e6d84d9c` with this spec at
`5bb46948` (fidelity 0C/8I/4M/2N, journey 1C/5I/4M/2N, tests 0C/0I/2M), and the
spec fold's own verification closed GREEN. Nine of their findings are spec text,
each measured by the fold author on the gated trees rather than transcribed:

- **journey C-1 — §5.2 item 4 and §11.5.** The phrase-form locator's property is
  the DIGEST, not the presence of a row: `hashlockPlateLocator` always appends
  one, and both documents named a mutation that leaves 1286/1286 green. §11.5 now
  requires the assertion on a PHRASE record and THROUGH the flow, against a
  digest the test derives itself, and names the mutation that fires (the locator
  built one statement before `hashlockPlatesDerive`, measured
  `["hash  00000000..00000000"]`).
- **journey I-2 — §0 gains a FIFTH falsified record**, `composerCopyHashlockConfirm`'s
  *"The phrase and method are not on this device"*, with its replacement measured
  at 342 drawn / headroom 107 (§8.9 carries the row). The finding's own suggested
  wording measures 364 / 64 and the fit gate refuses it.
- **journey I-3 = fidelity M-4 — §3.3 item 2 and §8.2.2**: the no-op warning is
  silent when a carrier-SHAPED record is present, because every shape §4.3
  narrows out classifies `Unknown` and the warning printed directly above a
  refusal naming the same record.
- **journey I-4 — §12 item 8 and §11.4**: the QR scan gate's cost model was wrong
  by ~900× and named a technique that cannot be applied. MEASURED: the worst-case
  plate is 43m31s and its QR alone 32m12s; a constant-time QR is indivisible.
- **journey I-5 = fidelity I-7 — §5.3 step (A)'s Back contract** now says what
  the implementation does: Button1 declines the highlighted plate and backing out
  of the STEP is not offered; the exit is the census's Button1, at the cost of
  every pick decision already made.
- **fidelity I-4 — §8.6 rule 4a is DELIVERED**: `MethodLine`/`QRText` are pinned
  against the vendored `qr_text` rows, not a literal. The counterexample was run:
  reordering `salt=` and `iterations=` with a re-vendored corpus left
  `go test ./hashlock/` green.
- **fidelity I-5 — §7.2 item 4 and §11.3** say which dims §8.6 content reaches
  (29..53, not only the four this stage admits) and the in-suite budget row
  samples all seven; no shipped entry moves.
- **fidelity I-6 = journey M-1 — §8.2.4 and §3.3** name no direction: the
  passphrase is printed BELOW the note that used to point above it.
- **fidelity M-1 — §7.2 item 3** pairs the admitted version with its own
  capacity: v9 holds **230** bytes against a 194-byte worst case; 192 was v8's.

§13 gains three recorded decisions (journey M-4's decline-all end state, M-2's
`--expect` vocabulary, N-2's unmarked list). §11.6's firmware figures were
re-measured over the folded tree and are byte-identical.
