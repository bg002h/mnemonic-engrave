# Hashlock H1b — post-implementation adversarial execution review

**Reviewer:** one independent opus agent, no sub-agents. Read-only on `master` and on
`/scratch/code/shibboleth/me-worktrees/hashlock-h1b` (never touched).
**Brief:** `design/agent-briefs/hashlock-H1b-post-impl-brief.md`.
**Under review:** `git diff d723cac..278a0e4` (branch `hashlock-h1b`), 9 files.
**Built and run in:** a separate detached worktree `/scratch/code/shibboleth/me-worktrees/h1b-review`
at `278a0e4`, `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h1b-review-target`. Removed on exit;
every mutation reverted from a byte copy and `touch`ed, `git status --porcelain` empty after each.

**Verdict: GREEN — 0 Critical / 0 Important.** 3 Minor, 2 Nit.

The one question, answered: **no.** Over the whole diff I could not construct an `ms1` string
for which `me` at ms-codec 0.8 places a preimage-kind payload, names one wrongly in a way that
matters, or refuses a seed/share/BIP-93 secret it accepted at 0.7. Every test the diff adds
dies on the defect it names, verified by mutation, including one mutation (M10) built
specifically to isolate a single new assertion. Every count in the implementation report that I
could measure at `278a0e4` is true.

---

## 1. The kind space, exhaustively, through both verbs

Strings generated with `ms_codec::codex32::Codex32String::from_seed("ms", …)` in a throwaway
test in the review worktree (deleted), so the id, threshold, share index, prefix byte and X
length are each controlled independently. Both verbs run with the binary built at `278a0e4`
(`me 0.8.1`), stderr first line, exit code captured immediately after each pipeline. `class` is
`mnemonic_engrave::sysw::classify`.

| # | family | string | `me sysw pack` | `me seal --seal-secret` | class |
| --- | --- | --- | --- | --- | --- |
| 1 | entr, 16 B | `ms10entrsqz46h2at4w46h2at4w46h2at4w4sna8r2pfm392lu` | **exit 0**, `sealing: SEALED — this payload holds secret material (record 0 (codex32 secret))` | **exit 0**, `me: wrote 512 bytes` | `Codex32Secret` |
| 2 | entr, 20 B | `ms10entrsqz46h2at4w46h2at4w46h2at4w46h2at4v7yllsts06n65d` | **exit 0**, placed | **exit 0**, placed | `Codex32Secret` |
| 3 | entr, 24 B | `ms10entrsqz46h2at4w46h2at4w46h2at4w46h2at4w46h2atd3z62yh7dm9tn` | **exit 0**, placed | **exit 0**, placed | `Codex32Secret` |
| 4 | entr, 28 B | `ms10entrsqz46h2at4w…4w46h2ct27g5vtg9qyva` (69 ch) | **exit 0**, placed | **exit 0**, placed | `Codex32Secret` |
| 5 | entr, 32 B | `ms10entrsqz46h2at4w…4w46kdv3c0wn2hx0lq` (75 ch) | **exit 0**, placed | **exit 0**, placed | `Codex32Secret` |
| 6 | mnem, 32 B (0x02, lang 0) | `ms10entrsqgqvmnwdehxu…umnghdm565pwd3frn` (77 ch) | **exit 0**, placed | **exit 0**, placed | `Codex32Secret` |
| 7 | **well-formed plate**, id `hash`, 32 B X | `ms10hashsqvg3zyg3zy…g3zyl0sajh7rfj2z` (75 ch) | exit 4, `me: record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), not a seed record; this container cannot place one yet.` | exit 4, `me: this record is a hashlock PREIMAGE plate (kind 0x03), not a seed record; …` | `Unknown` |
| 8 | the same plate, **UPPERCASE** | `MS10HASHSQVG3ZY…RFJ2Z` | exit 4, `… is a hashlock PREIMAGE plate (kind 0x03) …` | exit 4, `me: record has an uppercase character at byte 0 — records must be lowercase, or the same wallet has two different public-data hashes (§6.4)` | `Unknown` |
| 9 | plate, **corrupted checksum** (last char) | `…l0sajh7rfj2q` | exit 4, `me: record 0 … is not a form this container can place: not a BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, …` | exit 4, `me: invalid record: invalid short checksum (input withheld)` | `Unknown` |
| 10 | 0x03 under id **`entr`** (mismatch) | `ms10entrsqvg3zy…g3z8kt0l839zcv36` | exit 4, `me: record 0 … is an ms1 string whose 4-character id and kind byte disagree; it is refused rather than read by either field (SPEC_ms_hashlock §1 rule 2).` | exit 4, `me: this ms1 string's 4-character id and kind byte disagree; …` | `Unknown` |
| 11 | **reverse** mismatch: 0x00 under id `hash` | `ms10hashsqz46h2at4w…46kw948dm43kh3yc` | exit 4, same mismatch words | exit 4, same mismatch words | `Unknown` |
| 12 | 0x03 under id **`test`** (unknown id) | `ms10testsqvg3zy…g3zmwwwe8yq0rhcj` | exit 4, `… is a hashlock PREIMAGE plate (kind 0x03) …` | exit 4, `… is a hashlock PREIMAGE plate (kind 0x03) …` | `Unknown` |
| 13 | 0x03 as a **2-of-3 share** | `ms12testcqvg3zy…g3zyh4sdxlma7ydj` | exit 4, `me: record 0 … is a VALID BIP-93 codex32 string — the checksum is good — but not a constellation \`ms1\` record …` (`Bip93OutsideTheProfile(75)`) | exit 4, `me: invalid record: this is one share of a K-of-N set (threshold '2', index 'c'); use \`ms combine\` …` | `Unknown` |
| 14 | 0x03, **X = 16 B**, id `hash` (50 ch) | `ms10hashsqv3zyg3zyg3zyg3zyg3zyg3zyg3qsqayfyu0dd7mu` | exit 4, `… is a hashlock PREIMAGE plate (kind 0x03) …` | exit 4, same | `Unknown` |
| 15 | 0x03, **X = 33 B**, id `hash` (77 ch) | `ms10hashsqv3zy…g3zygsluyywd2nfr8xz` | exit 4, `… is a hashlock PREIMAGE plate (kind 0x03) …` | exit 4, same | `Unknown` |
| 16 | 0x03, **X = 18 B**, id `hash` (**53 ch**) | `ms10hashsqv3zyg3zyg3zyg3zyg3zyg3zyg3zygsy4fq486lcf7n3` | exit 4, `… is a VALID BIP-93 codex32 string … not a constellation \`ms1\` record` (`Bip93OutsideTheProfile(53)`) — **see M-1** | exit 4, `me: invalid record: string length 53 outside v0.1 set [50, 56, 62, 69, 75]` | `Unknown` |
| 17 | plain BIP-93 16 B secret, first byte 0x03 (48 ch) | `ms10testsqdzyg3zy…gsmf8j2wj0wpqqf` | exit 4, `Bip93OutsideTheProfile(48)` | exit 4, `me: invalid record: string length 48 outside v0.1 set …` | `Unknown` |
| 18 | 33 B unshared, first byte **0x31** (75 ch) | `ms10testsx924242…42e57kwjt5g4vya` | exit 4, `Bip93OutsideTheProfile(75)` | exit 4, `me: invalid record: reserved-prefix byte was 0x31, expected 0x00` | `Unknown` |
| 19 | legitimate entr **2-of-3 share** (50 ch) | `ms12testcqqqqq…qahy36vrhu3rae` | exit 4, `Bip93OutsideTheProfile(50)` | exit 4, `me: invalid record: this is one share of a K-of-N set …` | `Unknown` |

**No preimage-kind payload is placed anywhere.** Rows 7, 8, 12, 14, 15 — every string the
device's `codex32.IsPreimage` shape or the codec's `PreimageLengthMismatch` reaches — exit 4 on
both verbs with the kind named. The destination directory was listed after every case: **no
`.bin` / `.uf2` was written for any refused row**, so the refusal precedes the write.

**No legitimate 0.7-accepted seed is refused.** Rows 1-6 place on both verbs (exit 0). The
0.7→0.8 codec diff is additive on the accept side — `diff` of the two vendored crates shows
`consts.rs` gaining `PREIMAGE_PREFIX` / `TAG_HASH` / `VALID_PREIMAGE_STR_LENGTHS` and `hash` in
`RESERVED_ID_BLOCKLIST` (a share-*generation* list, never consulted on decode), and `decode.rs`
gaining rule 6b plus the `TAG_HASH` arm. Nothing an `entr`-tagged 0x00/0x02 single passed at
0.7 fails at 0.8: rule 6b compares `tag != payload.kind().single_tag()`, and
`PayloadKind::{Entr, Mnem}::single_tag()` is `Tag::ENTR`.

Two further reachability checks:

- **Multi-record**, plate at index 2 behind a `text:` and a legit seed →
  `me: record 2 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03) …`. The index
  is right.
- **CRLF input.** `printf '%s\r\n' <legit entr-32> | me sysw pack` still places (exit 0);
  `printf '%s\r\n' <plate>` is still refused as a preimage plate. `preimage_plate` trims, and
  `main.rs:1010` trims each record before `validate_record`.

**`seal::record::validate_record` is the single decode seam.** `grep -rn 'ms_codec::'
crates/me-cli/src/` outside `seal/record.rs` returns only `main.rs:2822-2823`
(`VALID_STR_LENGTHS` / `VALID_MNEM_STR_LENGTHS` in an error string). There is no second path
into `RecordKind::Ms`, and `sysw::classify_with` maps `Ok(RecordKind::Ms) => Class::Codex32Secret`
as its only route to that class. So SPEC_ms_hashlock §9 item 5's reader shape ("decode
succeeded, therefore this is a seed") is closed at the one place it existed.

---

## 2. The wildcard arm

`validate_record`'s `Ok(_) => Err(RecordError::Invalid("an ms1 payload kind this me does not
know; refusing to place it as a seed"))` **refuses with a named message**, and it is
**unreachable today**.

`ms-codec 0.8.0`'s `payload.rs` declares `#[non_exhaustive] pub enum Payload` with exactly three
variants — `Preimage(Zeroizing<[u8;32]>)`, `Entr(Vec<u8>)`, `Mnem { language, entropy }` — all
three enumerated by the two arms above it, so no value can reach the wildcard from this codec.
The arm is not dead code the compiler would flag: deleting it (lines 219-222 of
`crates/me-cli/src/seal/record.rs`) fails the build —

```
error[E0004]: non-exhaustive patterns: `Ok((_, _))` not covered
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern
error: could not compile `mnemonic-engrave` (lib) due to 1 previous error
```

— which is exactly the hazard SPEC_ms_hashlock §9 item 3 asks downstream crates to sweep
(`_ => <value>` arms as much as `_ => unreachable!`). `me` has no other catch-all over
`Payload`, `PayloadKind` or `InspectKind`: it never names those types outside this match.
**Verdict: sound, and it fails closed.**

---

## 3. The predicate's negative space

`seal::record::preimage_plate` and `id_kind_mismatch` measured directly (throwaway test,
deleted). Quoted verbatim:

| input | `id_kind_mismatch` | `preimage_plate` | `bip93_outside_the_profile` | `unknown_reason` |
| --- | --- | --- | --- | --- |
| 2-of-3 share, data begins 0x03, id `test` | `false` | **`false`** | `true` | `Bip93OutsideTheProfile(75)` |
| plain BIP-93 48-char secret, first byte 0x03 | `false` | **`false`** | `true` | `Bip93OutsideTheProfile(48)` |
| 33-byte **0x31** unshared single, id `test` | `false` | **`false`** | `true` | `Bip93OutsideTheProfile(75)` |
| 33-byte **0x03** unshared, id `test` | `false` | **`true`** | `true` | `PreimagePlate` |
| entr-id mismatch (`ms10entrsqv0…5gz69g08wwtz9`) | **`true`** | **`false`** | `true` | `TagKindMismatch` |
| 0x03 / 16-byte X, id `hash` (50 ch) | `false` | **`true`** | `true` | `PreimagePlate` |
| 0x03 / 33-byte X, id `hash` (77 ch) | `false` | **`true`** | `true` | `PreimagePlate` |
| the well-formed plate | `false` | **`true`** | `false` (it decodes) | `PreimagePlate` |

Every row is the answer the brief asks for, in the direction it asks for. The share and the
0x31 row are excluded by the `unshared` conjunct and the `d[0] == 0x03` test respectively; the
mismatch is excluded twice (once by the explicit `if id_kind_mismatch(s) { return false; }`
inside `preimage_plate`, once by the arm order in `unknown_reason`); and the 48-char plain
secret is excluded by `d.len() == 33`, which is exactly the fork's `codex32.IsPreimage` shape.

The seam corpus rows are all still classified as the corpus asserts: the untouched
`codex32_seam.rs` and `record_corpus.rs` suites pass at the tip, and both go red under mutation
M1 (below), so they are live tripwires rather than decoration.

---

## 4. Mutation table — do the diff's tests fail on the defect they name?

Filter `test(/preimage/) | test(/misdiagnosed/) | test(/the_host_never_admits/) |
test(/the_codec_decodes/) | test(/corpus/)` (11 tests) unless noted. Every mutation reverted
from a byte copy then `touch`ed.

| # | mutation | result | killing assertion (verbatim) |
| --- | --- | --- | --- |
| **M1** | `Ok((_, Payload::Preimage(_)))` arm → `Ok(RecordKind::Ms)` (delete the refusal) | **7 failed / 4 passed** | `a_preimage_plate_is_not_a_seed_record`: `validate_record admitted a 0x03 preimage plate as Ms`; **the witness** at `preimage_plate_is_not_a_seed.rs:137:5` — `assertion failed: matches!(validate_record(PREIMAGE_PLATE), Err(RecordError::PreimagePlate))`, i.e. **its SECOND assertion**, `decoded` still succeeding; `the_host_never_admits_what_the_device_would_refuse`: `preimage-plate-0x03: host verdict` `left: true` / `right: false`; `every_corpus_record_classifies_as_it_did_before_s2`: `left: "Codex32Secret"` / `right: "Unknown"`; `a_preimage_plate_is_named_not_misdiagnosed`: `left: Ok([77, 78, 69, 77, 83, 89, 83, 87, …])` — **it packed**; both CLI tests at `:61:5` and `:104:5` (the acceptance asserts) |
| **M2** | shape clause `d.len() == 33 && d[0] == 0x03` → `false` | **2 failed / 9 passed** | `sysw_pack_names_a_preimage_plate_and_never_echoes_it` at `:65:5` — `stderr does not name the kind:`; `a_preimage_plate_is_named_not_misdiagnosed` at `sysw/mod.rs:873:9` — `left: Err(Unclassifiable(0, Unrecognised))` / `right: Err(Unclassifiable(0, PreimagePlate))`. `seal_names_…` **stays green**, as the plan predicts: the success-path arm refuses independently of the predicate |
| **M3** | `Err(PreimageLengthMismatch { .. })` clause → an error that can never match | **1 failed / 10 passed** | `a_preimage_plate_is_named_not_misdiagnosed` at `sysw/mod.rs:902:9` — `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(50)))` / `right: Err(Unclassifiable(0, PreimagePlate))`. This is the **third new assertion** (the 16-byte-X row) and nothing else |
| **M4** | drop `d.len() == 33` (any 0x03-leading unshared payload is a plate) | **1 failed / 10 passed** | `a_preimage_plate_is_named_not_misdiagnosed` at `sysw/mod.rs:891:9` — `assertion failed: !matches!(pack(vec!["ms10testsqv0qqqqqqqqqqqqqqqqqqqqqqq8mzk8tjfdnjn5".into()], …` — the **second new assertion**, the negative one |
| **M5** | remove the `id_kind_mismatch` arm from `unknown_reason` | **1 failed / 10 passed** | `an_id_kind_mismatch_is_named_not_misdiagnosed` at `sysw/mod.rs:926:9` — `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))` / `right: Err(Unclassifiable(0, TagKindMismatch))` |
| **M10** | narrow the shape arm to `… && &b[4..8] == b"hash"` | **1 failed / 4 passed** (5-test filter) | `a_preimage_plate_is_named_not_misdiagnosed` at `sysw/mod.rs:880:9` — `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))` / `right: Err(Unclassifiable(0, PreimagePlate))`. Built to isolate the **first new assertion** (the `test`-id 33-byte row), which M2 masks by failing earlier. It pins "the KIND is the prefix byte, not the id" and nothing else |

**Every test the diff adds or edits fails on the defect it names, and each of the three new
`sysw` assertions has a mutation that kills it alone (M10, M4, M3).** No false PASS found on
any spec-normative guarantee.

### Survivors (all message-only; none changes admission) — see M-3

| # | mutation | result |
| --- | --- | --- |
| M6 | drop the `unshared` conjunct (a 0x03 K-of-N share becomes a "plate") | **11 passed** |
| M7 | share-index compare case-**sensitive** (`*c == b's'`, dropping `eq_ignore_ascii_case`) | **11 passed** |
| M8 | remove the `if id_kind_mismatch(s) { return false; }` guard inside `preimage_plate` | **11 passed** |
| M9 | swap the mismatch and preimage arms in `unknown_reason` | **11 passed** |

M8 and M9 survive because they are the *same* guard expressed twice — either alone is
redundant, and neither alone changes an outcome. That is defence in depth, not a defect; it is
worth knowing only because a future editor removing *both* would silently rename a mismatch a
plate. M6 and M7 change what an operator is told and nothing else (details in M-3).

---

## 5. Records

| item | measured at `278a0e4` | verdict |
| --- | --- | --- |
| version `0.8.1` | `crates/me-cli/Cargo.toml` `version = "0.8.1"`; built binary reports `me 0.8.1` | ✅ |
| one `[Unreleased]` CHANGELOG section | `grep -n '^## \[' crates/me-cli/CHANGELOG.md` → `8:## [Unreleased]`, `69:## [0.8.0] - 2026-09-02`, `142:## [0.7.0]`, `172:## [0.3.0]` — exactly one, undated | ✅ |
| it carries H0's, the `+`-sign entry's and H1b's items | H0's bullet + seam-corpus bullet, the `+`-signed `key:` bullet at line 55 of the file, and the two new H1b bullets, all inside `[Unreleased]` | ✅ |
| H0's sentence in the past tense | `- … H0 shipped against ms-codec 0.7, where the codec's prefix gate refused the string; H1b moved the refusal onto the codec's success path …` | ✅ |
| F-473 closed with the SHA | `### F-473 — ~~…~~ **CLOSED 2026-09-04 by \`51f25c9\`**`, and `git log` confirms `51f25c9` is the bump commit | ✅ |
| F-454 advanced | `**ADVANCED 2026-09-04 (H1b …)** … **Still OPEN**: the version bump is not a release` | ✅ |
| M-6 filed under H2, `testdata/` NOT edited | `F-475 … (owning phase: **H2**)`; `git diff --stat d723cac..278a0e4 -- crates/me-cli/testdata/` is **empty** | ✅ |
| `Cargo.lock` moved only `ms-codec` | see below | ✅ |

**Lockfile.** The whole-branch diff is 8 lines: `mnemonic-engrave` `0.8.0`→`0.8.1` (its own
version bump), `ms-codec` `0.7.0`→`0.8.0` with its checksum, and `pbkdf2` + `sha2` appearing in
`ms-codec`'s *dependency list*. Both of those crates were already in the lockfile at `d723cac`
(lines 672 and 972), so **no package is added or removed**: `[[package]]` count is 142 at both
revisions, and `diff <(git show d723cac:Cargo.lock | grep '^name = ' | sort) <(git show
278a0e4:Cargo.lock | grep '^name = ' | sort)` is empty. No crate other than `ms-codec` changed
version. The brief's Important trigger does not fire.

**F-475's own measurement re-verified independently.** I called `ms_codec::decode` at 0.8.0 on
the three strings in its table: `ms10testsqvrsu9…` (id `test`) → `Err(unknown tag "test"; not a
member of RESERVED_TAG_TABLE)`; `ms10entrsqv0…` → `Err(tag "entr" does not name the kind the
prefix byte 0x03 carries…)`; `ms10hashsqw46h2…` → `Ok(Payload::Preimage)`. F-475's claim that
the seam row names the wrong 0.8 error is **correct**, and its "the row's verdict is right"
conclusion is correct too — `preimage_plate` names that string from its shape, not from the
codec error (verified: row 4 of §3's table).

---

## 6. The implementation report — every measurable claim

| report claim | measured | verdict |
| --- | --- | --- |
| diffstat at the code tip: 8 files, 276 insertions, 45 deletions | `git diff --stat d723cac..6f4edf8` → `8 files changed, 276 insertions(+), 45 deletions(-)`, per-file counts identical to the table | **true** |
| the report commit adds only the markdown | `git show --stat 278a0e4` → `1 file changed, 398 insertions(+)` | **true** |
| Task-1 lock diff is 4 insertions / 2 deletions, no `name =` line moved | `git show --stat 51f25c9 -- Cargo.lock` → `1 file changed, 4 insertions(+), 2 deletions(-)`; `grep -cE '^[+-]name = '` → `0` | **true** |
| Task-4 lock diff is that one version line | `git show 6f4edf8 -- Cargo.lock` → `-version = "0.8.0"` / `+version = "0.8.1"` and nothing else | **true** |
| staleness: `git diff 0f5ce23..d723cac -- crates/ Cargo.lock Cargo.toml` is empty | 0 lines | **true** |
| no file overlap with `master` | `comm -12` of the branch's changed files and `d723cac..308a905`'s → **empty**, still true against today's tip (the branch owns `design/FOLLOWUPS.md`, master's nine commits since `d723cac` do not) | **true, and still true** |
| final gate `619 tests run: 616 passed, 3 failed, 2 skipped` | reproduced exactly; the three FAILs are `history_purge::{editing_the_file_alone_is_the_trap_the_message_warns_about, the_harness_records_history_at_all, the_emitted_zsh_recipe_actually_purges_the_entry}`, each `/usr/bin/zsh is required` — box-local | **true** |
| `cargo fmt -p mnemonic-engrave -- --check` exit 0, no output | rc 0, 0 bytes | **true** |
| clippy's only error is the pre-existing `manual_is_multiple_of` | rc 101, one `error: manual implementation of \`.is_multiple_of()\`` at `crates/me-cli/src/sysw/composer_records.rs:114:8`, a file the branch never touches | **true** |
| the base was `617 run … 2 skipped` (tip 619 = 617 + the witness + the mismatch test) | the diff adds exactly **2** `#[test]` functions (`grep -c '^+.*#\[test\]'` → 2: `the_codec_decodes_the_plate_and_me_still_refuses_it`, `an_id_kind_mismatch_is_named_not_misdiagnosed`), and 619 − 2 = 617 | **arithmetically consistent** |
| mutation (a): 6 failed, the witness on its **second** assertion | M1 reproduces it — 7 failures on my wider filter (which adds `record_corpus`), the witness at the second assertion. See **N-1** for the line number | **true in substance** |
| mutation (b): `sysw_pack_names_…` at `:65:5` "stderr does not name the kind", `a_preimage_plate_is_named_not_misdiagnosed` `left: Unrecognised`, `seal_names_…` stays green | M2 reproduces all three, including the `Unrecognised` (not `Bip93OutsideTheProfile`) left-value | **true** |
| Task-3 mutations (i)/(iii) → `Bip93OutsideTheProfile(75)` | M5 reproduces the value; the report's `:923` / `:926` / `:920` line cites are each exactly consistent with how many lines that state of the tree was missing | **true** |
| "No output container was written in any of the six cases" | extended: no container written in **any** of the 13 refused rows of §1 | **true** |

**No false count found.** One stale line number (N-1).

---

## Findings

### M-1 — the CHANGELOG and `preimage_plate`'s doc comment both claim a coverage the code does not have (owning phase: **H2**)

`crates/me-cli/CHANGELOG.md`, the H1b bullet, says:

> so a `0x03` single under any id — or with a wrong X length, which the codec names
> `PreimageLengthMismatch` — is named a preimage plate on both host verbs rather than falling
> through to "outside the profile".

and `crates/me-cli/src/seal/record.rs`'s `preimage_plate` doc says:

> a `0x03` single under any other id, **or with a wrong X length**, is named the same way

Measured false. `me sysw pack` on a `hash`-id `0x03` single with an 18-byte X:

```
$ printf '%s\n' "ms10hashsqv3zyg3zyg3zyg3zyg3zyg3zyg3zygsy4fq486lcf7n3" | me sysw pack --out /tmp/p.bin
me: record 0 (records count from 0) is a VALID BIP-93 codex32 string — the checksum is good — but not a constellation `ms1` record, so this container cannot place it.
      `ms1` is a two-gate PROFILE over BIP-93: the whole string must be [50, 56, 62, 69, 75] characters (entropy) or [51, 58, 64, 70, 77] (mnemonic), and the 4-character id must be `entr` (a seed) or `hash` (a hashlock preimage plate). This one is 53 characters.
```

That is `UnknownReason::Bip93OutsideTheProfile(53)` — literally "falling through to outside the
profile". The boundary is arithmetic and exact: `preimage_plate` names a wrong-X `0x03` single
only when the codec reaches `PreimageLengthMismatch`, which needs the string length to be in
`VALID_STR_LENGTHS ∪ VALID_MNEM_STR_LENGTHS`. String length is `22 + ceil(8·(1+X)/5)`, so the
covered X values are exactly **{16, 17, 20, 21, 24, 25, 28, 29, 32, 33}**; X ∈ {18, 19, 22, 23,
26, 27, 30, 31, 34, …} is not. Measured, four of them:

| X | chars | `me sysw pack` reason |
| --- | --- | --- |
| 18 | 53 | `Bip93OutsideTheProfile(53)` |
| 19 | 54 | `Bip93OutsideTheProfile(54)` |
| 31 | 74 | `Bip93OutsideTheProfile(74)` |
| 34 | 78 | `Bip93OutsideTheProfile(78)` |

**Minor, not Important, and I want to be explicit about why**, because the brief's rubric says
"a `0x03` single named 'outside the profile' is Important". The *behaviour* is right: such a
string is not a preimage plate by the device's `codex32.IsPreimage` either (its payload is not
33 bytes), it is refused on both verbs with exit 4, and the sentence it gets is **true of the
string it is about** — 53 really is outside the profile's length sets, and the message says so
and names them. Nothing is mis-refused and no wrong outcome is worse than silence. What is
wrong is the **claim**, in two records — a records defect.

It still wants fixing, and H2 is the right owner: H2 writes the device-side convergence table,
and an author building it from this sentence would either write a seam row that goes red or
"fix" `preimage_plate` to widen a predicate that is currently, correctly, the device's shape.
Suggested narrowing: *"or with a wrong X length the codec can name (`PreimageLengthMismatch`)"*.

### M-2 — the argv secret guard does not cover a preimage plate, and H1b's design is why (secret-handling; owning phase: **a follow-up**)

```
$ me ms10hashsqvg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyl0sajh7rfj2z
error: unrecognized subcommand 'ms10hashsqvg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyl0sajh7rfj2z'
```

clap echoes the whole plate. The control, the same surface with an entr-32 single:

```
me: argument 1 on ARGV (arguments count from 0, and 0 is `me` itself) is SECRET key material.
```

`argv_secret_guard` (`main.rs:551`) asks `sysw::classify(token)` then
`Class::is_argv_forbidden()` — deliberately, so that "the classifier defines the set, so there
is no list to be short". A preimage plate is `Class::Unknown`, so the guard does not fire, and
a 32-byte hashlock preimage — spend-path material, `Zeroizing` inside ms-codec — reaches
`/proc`, `ps` and the shell history.

**Not a regression**: at ms-codec 0.7 the plate was also `Unknown` and also uncovered. But H1b
is where the reason becomes permanent — L22/H0's "inert, never `Codex32Secret`, no class of its
own" is exactly what keeps a preimage out of `is_argv_forbidden`'s five classes. That coupling
is nowhere written down, and it is the shape the repo already has a lesson about ("a guard
downstream of the parser has lost").

**Severity is capped at Minor by the operator ruling of 2026-08-27** — a failure to handle
secret material secretly is never Critical and never Important; log it. Filing it is the whole
remedy here. If it is ever taken up, the honest options are a `Class` for the kind (which H0
rejected) or a shape test in the guard that does not go through `Class`.

### M-3 — four guards in the new predicate have no test; each mutant only changes the message

M6/M7/M8/M9 in §4 all survive the whole suite. Two are redundancy (M8, M9 — the mismatch is
excluded twice, so neither exclusion alone is observable) and are fine as they stand. The other
two are load-bearing and uncovered:

- **M6, the `unshared` conjunct.** Dropping it makes a 2-of-3 share whose SSS point begins
  `0x03` report `hashlock PREIMAGE plate` instead of the BIP-93-share diagnosis. The seam
  corpus *has* the case — `bip93-share-payload-0x03` — but it cannot catch this, because
  `host_admits` is `false` under both the original and the mutant: `me` refuses a share either
  way. Only the words change.
- **M7, `eq_ignore_ascii_case` on the share index.** Making it case-sensitive drops the
  **uppercase** plate (row 8 of §1) from `PreimagePlate` to `Unrecognised` — the operator would
  read `not an md1/mk1/ms1/mt1 string` for a BIP-93-legal uppercase form of a real preimage
  plate. Uppercase codex32 is the QR-alphanumeric form, so this is a shape a person can
  actually hold. Nothing asserts it.

Minor because neither mutant changes what is admitted, only what is said. Two one-line unit
assertions in `sysw::tests` would close both — a `0x03` 2-of-3 share asserted **not**
`PreimagePlate`, and the uppercase plate asserted `PreimagePlate`.

### N-1 — the implementation report cites `preimage_plate_is_not_a_seed.rs:136` twice; at `278a0e4` the assertion is at `:137`

The report's mutation (a) and D-1 both say the witness fails at
`crates/me-cli/tests/preimage_plate_is_not_a_seed.rs:136:5`, "which is `assert!(matches!(` — the
second assertion". Reproduced under M1:

```
thread 'the_codec_decodes_the_plate_and_me_still_refuses_it' panicked at crates/me-cli/tests/preimage_plate_is_not_a_seed.rs:137:5:
assertion failed: matches!(validate_record(PREIMAGE_PLATE), Err(RecordError::PreimagePlate))
```

At the tip, `:136` is `);` — the **first** assertion's closing line — and `:137` is the second
assertion. The claim the number supports ("the second assertion") is **true**; the number is
stale by one, because D-1's own correction added a line to the doc comment above it after the
measurement was taken. Mildly self-referential: D-1 exists precisely because "a MUTATION
comment is a claim a future reader will re-run". Fixing the two `:136`s to `:137` costs nothing.

### N-2 — the widened profile sentence names `hash` as an admissible id, and a `hash` record is the one `me` will not place

The diff widens three prose sites to "the 4-character id must be `entr` (a seed) or `hash` (a
hashlock preimage plate)". Read literally against the sentence that follows it in
`bip93_outside_the_profile`'s doc — "so plain BIP-93 secrets … are perfectly valid codex32 and
still **not records this tool can place**" — the widening implies a `hash`-id string *is* a
record this tool can place, which is exactly false.

In practice it never misleads: a well-formed `hash` string is caught by the `PreimagePlate` arm
long before this message, and the only `hash`-id strings that reach the widened sentence failed
the *length* gate, which the same sentence names correctly (M-1's row 16 prints "This one is 53
characters"). Nit, not Minor, for that reason — but if M-1 is folded, this is the same sentence
and costs one clause: *"or `hash` (a hashlock preimage plate, which is refused for its kind —
see above)"*.

---

## Deviations in the implementation report — a verdict each

| | claim | verdict |
| --- | --- | --- |
| **D-1** | the plan's witness MUTATION comment said the failure lands on `decoded`; measured, it lands on the second assertion, comment corrected | **Accepted, and independently reproduced.** M1 confirms `decoded` still succeeds under a `validate_record` mutation and the panic is at the second assertion. The correction is right and minimal. One residue: N-1 |
| **D-2** | Task 2's `git add` list omitted `main.rs` and `sysw/mod.rs`, which Task 2's own steps edit; both staged at Task 2 so the committed tree is the gated tree | **Accepted, and it was the right call.** `git show --stat 2bb3f3b` carries only Task 3's two files; `git show --stat 51f25c9` carries the five Task-1/2 files. Nothing of Task 3 leaked into the earlier commit, and each commit is independently buildable |
| **O-1** | adding `UnknownReason::TagKindMismatch` without operator words fails `bin "me"` with `E0004` — the compiler forbids a refusal reason with nothing to say | **Confirmed as a real structural guard.** `main.rs`'s `U::…` match is exhaustive over `UnknownReason`, which is a crate-local enum with no `#[non_exhaustive]`. Worth carrying into H2 exactly as the report says |
| **O-2** | date labels are 2026-09-04 per git's clock, not the plan's 2026-09-05 header | **Correct**, and consistent with the repo's standing rule |
| — | "the seam corpus was not edited; F-475 owns it under H2" | **Confirmed**: `git diff --stat d723cac..278a0e4 -- crates/me-cli/testdata/` is empty, and F-475 exists with the measurement. See M-1, which is the same *class* of records defect one file over and should fold with it |

---

## Counts

**0 Critical · 0 Important · 3 Minor (M-1, M-2, M-3) · 2 Nit (N-1, N-2)**

### GREEN

No finding blocks. The funds-relevant property the stage exists for — *a kind-`0x03` payload is
never placed as a seed by either host verb, and every seed that placed at ms-codec 0.7 still
places at 0.8* — holds across all 19 families measured through both verbs, is enforced at the
single decode seam, fails closed on a payload kind a future minor adds, and is pinned by six
tests each of which I killed with a mutation aimed at the guard it names. M-1 is a records
defect that should fold with F-475 under H2 before the device convergence table is written;
M-2 is capped Minor by the 2026-08-27 operator ruling and wants a follow-up entry; M-3 is two
missing one-line assertions.
