# Hashlock H1b plan R0 round 1 — fold verification

**Reviewer:** independent, sonnet tier, fold-verification. **Fold commit:** `b7ced42`
over gate-green draft `e672194`, responding to `hashlock-H1b-plan-R0-r0-fidelity.md`
(`9458bd3`, 0C/2I/6M/2N) and `hashlock-H1b-plan-R0-r0-tests.md` (`b923e41`, 0C/4I/2M).
**Worktree:** own, detached at `b7ced42`, `/scratch/code/shibboleth/me-worktrees/h1b-verify`
(never the controller's `h1b-gate`), `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h1b-verify-target`
— both removed at the end of this review. Every step below was applied FROM THE PLAN'S
OWN TEXT (never copied from the fold's self-description), building and testing at each
stage with `PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp
CARGO_TARGET_DIR=.../h1b-verify-target -p mnemonic-engrave`.

**One question:** did the fold address every Important from both reports, with the
plan's code blocks now matching what actually runs, and without a contradiction or a
false claim of its own? **Answer: yes — every Important is FIXED and reproduces
exactly as the fold's message and the plan's re-written text claim. One new Minor
(an anchor-quote mismatch) and one documentation note found; no Critical, no
un-fixed Important.**

## 1. Finding table

| id | verdict | plan section carrying the fix | reproduced |
| --- | --- | --- | --- |
| fidelity I-1 (predicate narrowing → "re-encode the entropy" on 3 `0x03` families) | **FIXED** | Task 2 Step 2 — `preimage_plate` re-pointed by SHAPE (unshared, 33-byte payload, first byte 0x03) + the codec's `PreimageLengthMismatch`; `id_kind_mismatch` excludes the L24 case | Yes — malformed 50-char plate named "hashlock PREIMAGE plate" at exit 4 on both verbs; seam rows tested individually (§3 below) |
| fidelity I-2 (`Ok(_) => RecordKind::Ms` fails open on the next payload kind) | **FIXED** | Task 2 Step 1 — positive `Entr \| Mnem` arm; wildcard `Ok(_)` now REFUSES | Yes, and re-run as a mutation (§4): merging the positive arm back into the wildcard passes the WHOLE crate (616/619, only `history_purge`) — confirms the plan's own claim that no test today can catch this (ms-codec 0.8.0 has exactly 3 `Payload` variants, so there is no third kind to reach the wildcard) |
| tests I-1 (RED narrative wrong for 2/5 failures) | **FIXED** | Task 1 Step 3 rewritten: "every witness... fails by ADMISSION"; `record_corpus` named as a 6th failure | Yes — reproduced 5-of-5 filtered failures verbatim, plus the whole-crate 6th (`record_corpus`) with its exact panic text (§2) |
| tests I-2 (fmt-check gap, 109-col literal in Task 3) | **FIXED** | Task 3 Step 1 — literal now wrapped; explicit "run cargo fmt after every block" instruction | Yes — `cargo fmt --check` clean after Task 2 Step 4 and after Task 3, applying the plan's text verbatim (no extra edit) |
| tests I-3 (malformed-length 0x03 misdiagnosed as "outside the profile") | **FIXED** | Task 2 Step 2 — `PreimageLengthMismatch` clause; Task 2 Step 4 — new assertion | Yes — `me sysw pack`/`me seal` on the 50-char malformed plate both name it PREIMAGE at exit 4 |
| tests I-4 (`record_corpus` blast-radius undercount) | **FIXED** | Task 1 Step 3 — names `record_corpus::every_corpus_record_classifies_as_it_did_before_s2` and states "SIX failures beyond `history_purge`" | Yes — whole-crate RED run: 9 failed (6 hashlock-related + 3 `history_purge`), `record_corpus`'s panic text quoted verbatim matches |

**Minors/Nits — one line each:**

| id | verdict |
| --- | --- |
| M-1 (lockfile "gains" wording) | FIXED — Step 1 Expected rewritten; `cargo update -p ms-codec` measured: one package moves, zero new `[[package]]` blocks, 2 dependency edges added |
| M-2 (CHANGELOG/F-454) | FIXED — Task 4 keeps the single `[Unreleased]` section (confirmed on disk: `## [Unreleased]` holds both the H0 `### Added` and the `+`-sign `### Changed` blocks, exactly as described), rewrites the H0 sentence past-tense, notes F-454 due (confirmed in `design/FOLLOWUPS.md:15419`, owning phase text matches) |
| M-3 (`me seal`/`me sysw pack` asymmetry on a mismatch) | FIXED — `RecordError::TagKindMismatch` + Display added; measured both verbs name the mismatch with matching words at exit 4 |
| M-4 (witness prints wrong crate's version) | FIXED — message no longer names a version |
| M-5 (stale comments/message) | FIXED — all three text edits applied and compile; the sysw unit test's mutation comment is also corrected (see note in §5) |
| M-6 (seam-corpus prose correction) | Filed with owning phase H2 in Task 4, as stated — no code change expected here |
| N-1 (HRP gate inside the helper) | FIXED — `id_kind_mismatch` centralizes the `classify(Format::Ms)` gate |
| N-2 (heading style) | FIXED — Task 4 keeps `## [Unreleased]`, no em-dash heading introduced |

## 2. Applied from the text — Task 1 only (RED)

Bumped `ms-codec = "0.8"`, ran `cargo update -p ms-codec`:
```
Updating ms-codec v0.7.0 -> v0.8.0
```
`Cargo.lock` diff: `version = "0.8.0"`, zero new `[[package]]` blocks, only `+ "pbkdf2"` / `+ "sha2"` edges added to the existing `ms-codec` stanza (M-1 confirmed). `cargo build --locked -p mnemonic-engrave` builds clean.

`cargo nextest run -E 'test(/preimage/) | test(/the_host_never_admits/)'`: **5 failed, 0 passed**, matching the plan. `a_preimage_plate_is_not_a_seed_record` panics `validate_record admitted a 0x03 preimage plate as Ms` — confirmed by ADMISSION, not the profile arm.

Whole-crate `--no-fail-fast`: **9 failed** (`617 tests run: 608 passed, 9 failed, 2 skipped`) = the 5 above + `history_purge`×3 + the 6th:
```
thread 'every_corpus_record_classifies_as_it_did_before_s2' panicked at .../record_corpus.rs:147:9:
assertion `left == right` failed: codex32_seam/preimage-plate-0x03: class moved under S2
  left: "Codex32Secret"
 right: "Unknown"
```
Matches the fold's rewritten Task 1 Step 3 text exactly (tests I-1, I-4).

## 3. Applied from the text — Task 2 + Task 3 (GREEN)

Replaced the `validate_record` decode arm and `preimage_plate`/`id_kind_mismatch` verbatim from the plan's code blocks; added `RecordError::TagKindMismatch` + Display; applied the three M-5 text edits; appended the witness test; added the three shape assertions to `a_preimage_plate_is_named_not_misdiagnosed`; ran `cargo fmt -p mnemonic-engrave`; added Task 3's `UnknownReason::TagKindMismatch` variant, the `unknown_reason` arm, the `main.rs` Display arm, and `an_id_kind_mismatch_is_named_not_misdiagnosed`.

- `cargo nextest -E 'test(/preimage/) | test(/the_host_never_admits/) | test(/the_codec_decodes/)'`: **6 passed** (plan's claim).
- `cargo fmt -p mnemonic-engrave -- --check`: clean (0) — tests I-2 confirmed fixed.
- `cargo nextest -E 'test(/misdiagnosed/) | test(/an_id_kind_mismatch/)'`: **2 passed**.
- Whole crate: `619 tests run: 616 passed, 3 failed, 2 skipped` — all 3 `history_purge` — **exact match** to the fold commit message.
- `cargo clippy --all-targets -- -D warnings`: exactly the one pre-existing `manual_is_multiple_of` at `sysw/composer_records.rs:114` — **exact match**.

**Binary checks (brief item 2):**
```
$ me sysw pack --no-passphrase --out p1.uf2 "ms10hashsqw46h2at4w46h2at4w46h2at4w4ssrnvvaudn2k4d"
me: record 0 ... is a hashlock PREIMAGE plate (kind 0x03), not a seed record; ...
exit: 4
$ me seal --out p2.uf2 --seal-secret <<< "ms10hashsqw46h2at4w46h2at4w46h2at4w4ssrnvvaudn2k4d"
me: this record is a hashlock PREIMAGE plate (kind 0x03), not a seed record; ...
exit: 4
$ me sysw pack --no-passphrase --out p3.uf2 "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9"
me: record 0 ... is an ms1 string whose 4-character id and kind byte disagree; ...
exit: 4
$ me seal --out p4.uf2 --seal-secret <<< "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9"
me: this ms1 string's 4-character id and kind byte disagree; ...
exit: 4
```
Both verbs name both cases identically, exit 4 — matches the fold commit message.

## 4. The predicate's negative space (brief item 3)

Note: the brief's cited row `bip93-plain-33-byte-payload-0x31` **does not exist** in
`testdata/codex32_seam_vectors.json` (`grep -n '"name"'` lists 13 rows, none named
that) — a citation issue in the review brief, not the plan. I tested the seam rows
that DO exist for the same negative-space question:

```
$ me sysw pack --no-passphrase --out p.uf2 "ms10testsqv0qqqqqqqqqqqqqqqqqqqqqqq8mzk8tjfdnjn5"   # bip93-plain-payload-0x03, 48 chars, 16-byte payload
me: ... is a VALID BIP-93 codex32 string ... not a constellation `ms1` record ...
exit: 4   (NOT named PREIMAGE — correct, MUST NOT answer preimage)

$ me sysw pack --no-passphrase --out p.uf2 "ms10testsqvrsu9guyv4rzwplgex4gkmzd9c8wl593jfe4gdg47mtm3xt6tv7qh3pm4xrfdlvvp"  # bip93-plain-33-byte-payload-0x03, THE COLLISION
me: ... is a hashlock PREIMAGE plate (kind 0x03) ...
exit: 4   (named PREIMAGE — correct, MUST answer preimage)

$ me sysw pack --no-passphrase --out p.uf2 "ms12testaqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdq7pl8qdc5tsp"  # bip93-share-payload-0x03, 2-of-N share
me: ... is a VALID BIP-93 codex32 string ... not a constellation `ms1` record ...
exit: 4   (NOT named PREIMAGE — correct, MUST NOT answer preimage: it fails the `unshared` byte check)
```

All three answer as the plan's predicate intends. The share row is itself
"a K-of-N share whose data begins 0x03" per its own `source` field
(`codex32.NewSeed("ms", 2, "test", 'a', 33 bytes with byte 0 = 0x03)`) — this
satisfies the brief's K-of-N-share check; I did not additionally brute-force one via
`ms split`, since a share's SSS y-value is not directly steerable to a chosen first
byte and the corpus already carries a deliberately-constructed example of exactly
this shape.

## 5. Mutations (brief item 4)

**Fidelity I-2's guard, re-run**: merged the positive `Entr | Mnem` arm back into the
wildcard (`Ok(_) => Ok(RecordKind::Ms)`, `Preimage` arm untouched). Whole-crate result:
`619 tests run: 616 passed, 3 failed` — **identical to the un-mutated tree**; only the
3 box-local `history_purge` tests fail. **No test catches this today.** This confirms
the plan's own claim verbatim ("the plan says none can and the wildcard is the guard")
— acceptable, because ms-codec 0.8.0's `Payload` enum has exactly 3 variants
(`Preimage`, `Entr`, `Mnem` — confirmed by reading `payload.rs` in the crates.io
0.8.0 source), so there is no third kind today to exercise the refusing arm. This is
a real gap in TEST coverage but not a false claim — the plan frames the wildcard as a
compiler-enforced guard against a FUTURE ms-codec minor, never as something today's
suite exercises, and it says so.

**Task 2 Step 4 mutation (a)** (delete the `Preimage` arm): 6 failed, 0 passed — exact
match to "six in all (tests lens: `6 failed, 0 passed`)".

**Task 2 Step 4 mutation (b)** (drop the shape clause, `d.len()==33 && d[0]==0x03` →
`false`): `sysw_pack_names_a_preimage_plate_and_never_echoes_it` FAILS `stderr does not
name the kind`; `a_preimage_plate_is_named_not_misdiagnosed` FAILS
(`left: Err(Unclassifiable(0, Unrecognised))`); `seal_names_...` stays green — exact
match.

**Task 3 mutation** (remove the `id_kind_mismatch` arm): `an_id_kind_mismatch_is_named_not_misdiagnosed`
FAILS `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`; the preimage test
stays green — exact match to the fold commit message.

All three mutations reverted and diffed byte-identical to the pre-mutation state
before proceeding.

## New findings (not in either R0 round-0 report)

**N-new-1 (Minor).** Task 2 Step 2's anchor prose says to find `pub fn preimage_plate`
"from its doc comment's first line `/// Is `s` a hashlock PREIMAGE plate`" — the
actual current source's doc comment first line reads `/// Is `s` a string of the
hashlock PREIMAGE KIND (SPEC_ms_hashlock §1: kind` (confirmed: `grep -n "^/// Is
\`s\`" crates/me-cli/src/seal/record.rs` finds no line matching the plan's quoted
text). An implementer grepping the literal anchor would find no match. Not a build or
test defect — the function name is unique and unambiguous, so any implementer reading
the full instruction (not just the parenthetical) lands on the right target, as I did.

**N-new-2 (documentation note, not a defect).** Task 2 Step 4's prose says to "fix
that test's recorded mutation comment" (fidelity M-5) but its own ```rust code block
does not show the corrected comment text — only the three new assertions. I composed
reasonable replacement text to proceed ("MUTATION: swap the two arms in
`unknown_reason` -> `Unrecognised` ... R0 r0 fidelity M-5"). Harmless (it is a
comment, not behavior), but a future implementer following the plan literally would
need to compose their own wording too, same as I did.

**N-new-3 (Nit, markdown formatting).** In the Self-review section, the fold's new
paragraph "**R0 round 0 folded here.** ..." is inserted directly after list item 2
with no blank line before it and no list marker — under CommonMark this renders as a
lazy continuation of item 2's paragraph rather than its own block, before item 3
restarts cleanly (a blank line precedes it). Purely cosmetic; the prose is fully
legible either way.

None of the three rises to Important: none causes a wrong build, a wrong test result,
or a false claim about behavior — all are anchor/prose precision issues resolvable
unambiguously by an implementer reading the surrounding instruction.

## Closing counts

- Fidelity: 2/2 Important FIXED and reproduced; 6/6 Minor + 2/2 Nit FIXED.
- Tests: 4/4 Important FIXED and reproduced.
- New: 0 Critical, 0 Important, 2 Minor (anchor drift, doc-note), 1 Nit (list formatting).

**GREEN.** Both R0 round-0 reports' Importants are fixed and reproduce exactly from
the plan's own text; the re-run build gate matches the fold commit message in every
number quoted (9/9 targeted, 619/616/3 whole-crate, fmt clean, the one pre-existing
clippy lint). This closes R0 (lens-closure: fidelity, tests/mutation, fold-verification).
