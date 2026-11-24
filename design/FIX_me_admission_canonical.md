# FIX: `me sysw pack` admits only the spelling it writes (bh-me-I1, I6, I9, I17; closes F-245)

- Source: bughunt 2026-11-19 keys **bh-me-I1, bh-me-I6, bh-me-I9, bh-me-I17**, all graded **Important**
  (C/agent-reports/bughunt-1119/grade-me.md:64, :69, :72, :79; proposed FIX 1 at :108-113;
  `design/agent-reports/bug-hunt-2026-11-19.md` §I1 :40-44, §I6 :84-88, §I9 :102-106, §I17 :150-154).
  Architect table row #28, **small** track, one Opus review round. Not class B.
- Rulings applied (decided; not re-raised here): **B1 / P1** normalise a byte-order mark and CRLF line ends only,
  refuse every other near-canonical variant (C/memory/product-rulings-sweep-bughunt-2026-11-22.md:24;
  architect report P1); **T30** `text:` and `pass:` require UTF-8 at admission, `tx:` stays bytes; **T29** the I1
  secret half is run-to-resolve (run, §8); **T33** F-245's owning phase is this FIX.
- Base: master `62fdf379f8283083390cd0086fab8bdf7d4df3c5`. The graded head was `ff5ecfea`; between them ME-1 (mr1
  records) and the I7/I8 FIX landed. `classify_with` and `admit_check` are unchanged by them; every anchor below is
  at 62fdf379. Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-23.
- Four new holder-visible sentences (§4: T-SEP, T-LF, T-PAD, T-UTF8BODY), all **PROPOSED (architect confirms)**.
  No new flag, command or exit code. One new library error variant (§2). §7 lists what this document decides that
  the bug report and rulings do not literally state.
- **Files the build changes** (for build ordering against other me rows): `crates/me-cli/src/sysw/mod.rs`,
  `crates/me-cli/src/main.rs`, `crates/me-cli/tests/sysw_cli.rs`, `crates/me-cli/CHANGELOG.md`,
  `design/SPEC_systemwide_payloads.md`. **Not changed:** `crates/mnemonic-io-lib` (it has a crates.io consumer,
  the toolkit), `sysw::classify`/`classify_with`, the record-class vectors, the fork's Go. This document and the
  gate script `scripts/fix-me-admission-canonical-gate.sh` are already committed on this branch.
- The toolkit manual (`mnemonic-toolkit` master `docs/manual/src/40-cli-reference/`) mentions `me sysw pack` only as
  a pipe target (43-ms.md:89, :411, :438-446, :691-700) and documents none of its admission rules, so there is no
  lockstep text (read from the toolkit clone's working tree, 2026-11-23).

## 1. The defects (measured on master 62fdf379)

`bash scripts/fix-me-admission-canonical-gate.sh --cli` on master: every R row fails, every control passes, then
`gate --cli: FAIL` (log: C/agent-reports/fix-doc-bh-me-admission-canonical.md). In short:

| Key | Input | Today |
|-----|-------|-------|
| I1 | a 12-word mnemonic with one doubled space, a TAB or a NBSP as a separator, `--expect mnemonic` | exit 0; the wire record keeps the separator (94/93/94 bytes); the device classifies it Unknown (§8) |
| I6 | library `sysw::pack(vec![doc])` with the trimmed `formats-happy/bluewallet-sh-fixture` document (classifies `Descriptor`) | `Ok`; `open` returns 14 public records |
| I6 | CLI argv: the same mnemonic wrapped onto two lines as ONE argument, `--allow-argv-secret --expect mnemonic` | exit 0; the wire holds one LF inside it, read back as two records |
| I9 | `text:ff` / `pass:ff` on stdin | exit 0; 7-byte record whose body `decode_text` refuses (`NotUtf8`) |
| I17 / F-245 | `"$D "` or `" $D"` on argv **or on a stdin line** | exit 0; the padding is on the wire (25 bytes) |
| B1 | a stdin line ending `\r\r\n` | exit 0; the record ends in `\r` on the wire |
| B1 | a UTF-8 BOM before the first line of stdin or `--in` | exit 4, "not a form this container can place" |

The bug report's "ordinary line-based input trims its lines" (§I17) is **not true at this head**: stdin and `--in`
go through `mnemonic_io_lib::split_record_stream` (records.rs:16), which is `str::lines()` and drops only
whitespace-only lines. Padding on a line reaches the wire exactly as on argv (R9, R10).

Root cause: `admit_check` (sysw/mod.rs:593-629) asks only `classify_with` (mod.rs:350-440), and every classifier arm
accepts a looser spelling than the one written: `seal::record::validate_record` trims first (seal/record.rs:145),
`bip39::Mnemonic::parse_normalized` splits on any whitespace (mod.rs:392), `descriptor::host_admits` takes a
multi-line document, and the `text:`/`pass:` arms check hex only (mod.rs:369-382). `split` (mod.rs:631-675) then
writes the record as given and joins the section with LF.

## 2. Fix shape

1. **Admission spelling rules, in `admit_check` only** (mod.rs:593). In the per-record loop, directly after the
   existing `Unknown` refusal (mod.rs:596-598) and before the preimage and phrase checks, a record that classified
   as anything but `Unknown` is refused when the first of these holds, checked in this order:
   1. **A-LF:** it contains `'\n'` or `'\r'` anywhere.
   2. **A-PAD:** its first or last `char` is whitespace by `char::is_whitespace` (the same set Go's
      `strings.TrimSpace` removes on the device).
   3. **A-SEP:** its class is `Mnemonic` and it is not exactly its `split_whitespace()` words joined by one U+0020.
   4. **A-UTF8:** its class is `FreeText` or `Passphrase` and `record::decode_text` (record.rs:199) returns
      `Err`. `tx:` is not checked (T30).
   The refusal is a new `SyswError` variant carrying the index and which rule (the builder names it; suggested
   `NotCanonical(usize, Spelling)` with `Spelling::{LineBreak, Padding, MnemonicSeparator, NotUtf8Body(&'static str)}`,
   the `&'static str` being `record::TEXT_PREFIX` or `PASS_PREFIX`). It carries no record content.
2. **Classification is unchanged.** `classify`/`classify_with` keep their broad reading, so the argv secret guard
   (main.rs:3285-3368, it calls `classify` on the trimmed record), `--expect` (main.rs:1786-1803), `show`, the record-class
   vectors and the fork's lockstep test all see what they see today. A doubled-space mnemonic on argv is still refused
   as SECRET key material at exit 3 before admission runs.
3. **Channel normalisation: one BOM.** Exactly one U+FEFF at the very start of the raw text read from `--in`
   (main.rs:3257) or from stdin (`read_stdin_raw`, main.rs:3466, which also serves the `-` splice) is removed before
   the text is split into records and before it becomes `document`. Nothing else is removed. A second U+FEFF, a
   U+FEFF on argv or anywhere later in a stream stays and is refused by the existing `Unrecognised` sentence. CRLF
   needs no change: `str::lines()` already splits `\r\n` like `\n`; a `\r` not followed by `\n` stays in the record
   and A-LF refuses it. The helper lives in `me-cli` (not `mnemonic-io-lib`).
4. **CLI.** `sysw_error` (main.rs:3694) gains one arm printing §4's sentences. The call site (main.rs:1876-1879)
   already exits 4 and runs before the passphrase ceremony and before any write; nothing else moves.
5. **Readers unchanged.** `open`, `show`, `region_image` and the device keep reading old containers as they do.

## 3. Spec delta (`design/SPEC_systemwide_payloads.md`)

Insert after §5.3.2 (before `### 5.4`, line 740) a new subsection, verbatim:

> #### 5.3.3 Admission spelling — NORMATIVE (bughunt-1119 B1, 2026-11-23)
>
> `me sysw pack` writes each record exactly as admitted, so admission checks the spelling that will be written,
> not only its class. A record that classifies is still refused, exit 4, when (first match wins): it contains LF or
> CR; it begins or ends with whitespace; it is a BIP-39 mnemonic whose words are not separated by single U+0020
> spaces; or it is a `text:` or `pass:` record whose decoded body is not UTF-8 (`tx:` bodies are bytes).
> Classification itself is unchanged, and readers keep reading. The only changes to a record's SPELLING are one byte-order mark removed from the start of an `--in` file or of stdin, and CRLF line ends in those streams; whitespace-only lines in a stream are skipped, as before. Nothing is trimmed. (The `hash:` producer rule, SPEC_hashlock_kinds §6, and `--as`'s canonical re-encode rewrite a record's form, not its spelling, and are unchanged.)

And in §5.3.1, after the paragraph beginning "**Decoding is normative too:**", append one sentence:
"`me sysw pack` refuses a `text:` or `pass:` record whose body does not decode to UTF-8 (§5.3.3)."

## 4. Printed texts and exit codes (all **PROPOSED (architect confirms)**)

Each is one stderr line prefixed `me: `, `{i}` the record's index as given; exit **4**; nothing is written and no
passphrase is generated. None names any part of the record.

- **T-LF** (A-LF): `record {i}, as given (records count from 0) contains a line break (LF or CR). A record is one
  line, and records are stored separated by LF, so this one would be read back as several records. Put the record on
  one line and delete any stray CR (it does not show on screen; "$(cat file)" of a CRLF file keeps one), one record
  per line (--in or stdin) or per argument, and re-run; to pack a whole descriptor document, pass it with --as.`
- **T-PAD** (A-PAD): `record {i}, as given (records count from 0) begins or ends with whitespace. Records are stored
  exactly as given, so the whitespace would be engraved as part of this one. Remove it and re-run.`
- **T-SEP** (A-SEP): `record {i}, as given (records count from 0) is a BIP-39 mnemonic whose words are not separated
  by single spaces (a doubled space, a tab or another space character). The device reads a seed only in that
  spelling and would not recognise this record as one. Separate the words with one ordinary space each and re-run.`
- **T-UTF8BODY** (A-UTF8), option (b), final (coordinator ruling): for `text:`, two lines, the second indented six
  spaces exactly as `NonHexBody`'s remedy (main.rs, the `U::NonHexBody` arm): `record {i}, as given (records count
  from 0) begins `text:` and its body is lowercase hex, but the bytes it decodes to are not UTF-8 text (§5.3.1).
  Encode the text's own UTF-8 bytes:` then `      printf '%s' 'your text here' | xxd -p -c 256`. For `pass:`,
  one line and NO remedy line: `record {i}, as given (records count from 0) begins `pass:` and its body is lowercase
  hex, but the bytes it decodes to are not UTF-8 text (§5.3.1).`

(The sentences in the gate script are the same strings with `{i}` = 0; the script is the authority for spacing.)
Unchanged: every other refusal and its exit code; a BOM-prefixed stream that used to exit 4 now packs (exit 0).

## 5. RED rows (run on master 62fdf379; no keys or funds involved)

Material: `D=md1yqpqqxqq8xtwhw4xwn4qh` (README card); `M` = the BIP-39 all-zero test vector, eleven `abandon` and
`about`, single spaces; `M2`/`MT`/`MN` = `M` with its first separator a doubled space / TAB / U+00A0; `W6` = `M` with
its sixth separator an LF; `BOM` = EF BB BF. Every command is `me sysw pack ... --out o.bin` in a private dir
(umask 077). Gate: `bash scripts/fix-me-admission-canonical-gate.sh --cli`.

| Row | Key | Command (abridged; the gate has it verbatim) | Master | After |
|-----|-----|-----------------------------------------------|--------|-------|
| R1 | I1 | `printf '%s\n' "$M2" \| me sysw pack --no-passphrase --expect mnemonic` | exit 0, separator on the wire | exit 4, T-SEP, no file |
| R2 | I1 | same with `$MT` | exit 0 | exit 4, T-SEP |
| R3 | I1 | same with `$MN` | exit 0 | exit 4, T-SEP |
| R4 | I6 | `me sysw pack --no-passphrase --allow-argv-secret --expect mnemonic "$W6"` | exit 0, one LF inside the record | exit 4, T-LF |
| R5 | I9 | `printf 'text:ff\n' \| me sysw pack --no-passphrase` | exit 0 | exit 4, T-UTF8BODY `text:` |
| R6 | I9 | `printf 'pass:ff\n' \| ...` (stdin: `pass:` on argv is refused as secret first) | exit 0 | exit 4, T-UTF8BODY `pass:` |
| R7 | I17 | `me sysw pack --no-passphrase "$D "` | exit 0, trailing 0x20 on the wire | exit 4, T-PAD |
| R8 | I17 | `... " $D"` | exit 0, leading 0x20 | exit 4, T-PAD |
| R9 | I17 | `printf '%s \n' "$D" \| ...` | exit 0, trailing 0x20 | exit 4, T-PAD |
| R10 | I17 | `printf ' %s\n' "$D" \| ...` | exit 0, leading 0x20 | exit 4, T-PAD |
| R11 | B1 | `printf '%s\r\r\n' "$D" \| ...` | exit 0, record ends `\r` | exit 4, T-LF |
| R12 | B1 | `printf "$BOM%s\n" "$D" \| ...` | exit 4, Unrecognised | exit 0, wire = `D` |
| R13 | B1 | the same bytes in `in.txt`, `--in in.txt` | exit 4 | exit 0, wire = `D` |
| R14 | B1 | R12's stdin, `... - text:6869` (the `-` splice) | exit 4 | exit 0, wire = `D\ntext:6869` |
| R15 | order | `printf '%s \n' "$M2" \| ...` (padded AND doubled space) | exit 0 | exit 4, **T-PAD** (A-PAD before A-SEP) |
| R16 | order | `printf '%s\n' "$M2" \| me sysw pack --passphrase-words 6` | exit 0, passphrase printed | exit 4, T-SEP, no "write this down" line |
| C1 | control | `printf '%s\n' "$M" \| ... --expect mnemonic` | exit 0, wire = `M` | unchanged |
| C2 | control | `text:c3a9` on stdin | exit 0 | unchanged |
| C3 | control | `printf '%s\r\n' "$D"` (CRLF) | exit 0, wire = `D` | unchanged |
| C4 | control | `"$D"` on argv | exit 0 | unchanged |
| C5 | control | two BOMs then `D` on stdin | exit 4, Unrecognised | unchanged (one BOM only) |
| C6 | control | `"$BOM$D"` on argv | exit 4, Unrecognised | unchanged (streams only) |
| C7 | control | `D\r\n\r\nD\r\n` on stdin | exit 0, wire = `D\nD` | unchanged |

Library rows (run on master with a scratch test, 6 of 7 failed as expected; they become the tests in §6):

| Row | Call | Master | After |
|-----|------|--------|-------|
| L1 | I6: `sysw::pack(vec![bluewallet doc trimmed], None, MIN_ITERATIONS)` | `Ok`; `open` gives 14 records | `Err(NotCanonical(0, LineBreak))` |
| L2 | I17: `pack(vec![format!("{D} ")], ..)` | `Ok`, 25-byte record | `Err(NotCanonical(0, Padding))` |
| L3 | I17: `pack(vec![format!(" {D}")], ..)` | `Ok` | `Err(.., Padding)` |
| L4 | I1: `pack(vec![M2], ..)` | `Ok`, 94-byte record | `Err(.., MnemonicSeparator)` |
| L5 | I9: `pack(vec!["text:ff"], ..)` | `Ok` | `Err(.., NotUtf8Body("text:"))` |
| L6 | I9: `pack(vec!["pass:ff"], ..)` | `Ok` | `Err(.., NotUtf8Body("pass:"))` |
| L7 | control: `pack(vec![D, "text:c3a9"], ..)` then `open` | records equal the input | unchanged |
| L8 | classify: `classify` of `M2`, `MT`, `MN`, `"{D} "`, the bluewallet doc, `text:ff`, `pass:ff` | `Mnemonic` x3, `MdMk`, `Descriptor`, `FreeText`, `Passphrase` | **unchanged** |

## 6. Tests

New (each doc comment names its row; names are the gate's):

- `crates/me-cli/src/sysw/mod.rs` `mod tests`:
  - `admission_refuses_a_record_with_a_line_break` (A-LF: `D` followed by a lone CR, `D` preceded by an LF, `W6`, and L1's document; an LF or a CR in the MIDDLE of an `md1` is asserted to stay `Unclassifiable(0, UnknownReason::Unrecognised)`, §7 item 3).
  - `admission_refuses_a_padded_record` (A-PAD: leading and trailing space, TAB and U+00A0 on `D`; a padded `ms1` and a padded `mt1`; L2, L3; `text:6869 ` is asserted to stay `Unclassifiable(0, UnknownReason::NonHexBody("text:"))`, §7 item 3).
  - `admission_refuses_a_mnemonic_without_single_space_separators` (A-SEP: `M2`, `MT`, `MN`, and `M` with every
    separator doubled; `M` itself admitted; L4).
  - `admission_refuses_a_text_or_pass_body_that_is_not_utf8` (A-UTF8: L5, L6, `text:c0af` overlong; `text:c3a9` and
    `pass:` six `abandon` hex admitted; a valid `tx:` record still admitted).
  - `admission_rule_order_is_line_break_padding_separator_utf8` (a padded `M2` gives `Padding`; `M2` with a trailing
    LF gives `LineBreak`; the index of the first failing record is reported).
  - `pack_refuses_a_multi_line_descriptor_document` (L1 through `pack`, not only `admit_check`).
  - `pack_refuses_a_padded_card_and_writes_the_plain_one` (L2, L3 refused; L7's plain record is on the wire byte for byte).
  - `classification_of_non_canonical_spellings_is_unchanged` (L8).
- `crates/me-cli/tests/sysw_cli.rs`:
  - `pack_refuses_a_mnemonic_without_single_space_separators` (R1-R3: exit 4, T-SEP as a whole stderr line, no file).
  - `pack_refuses_a_wrapped_mnemonic_on_argv` (R4: exit 4, T-LF; and without `--allow-argv-secret` still exit 3
    with the existing SECRET-on-argv refusal).
  - `pack_refuses_text_and_pass_bodies_that_are_not_utf8` (R5, R6; C2 packs).
  - `pack_refuses_padded_records_on_every_channel` (R7-R10, plus `--in` with a padded line; T-PAD each).
  - `pack_refuses_a_lone_carriage_return` (R11; C3 and C7 pack with the plain records on the wire).
  - `pack_strips_one_leading_byte_order_mark_from_a_stream` (R12-R14 pack with the expected wire bytes; C5, C6 still
    exit 4 with the Unrecognised sentence).
  - `pack_refuses_before_the_passphrase_ceremony_on_a_spelling_fault` (R16: no "write this down" line, no file).
- `crates/me-cli/src/main.rs` `mod tests`: `strip_leading_bom_removes_exactly_one` (none, one, two BOMs; a BOM after
  the first byte is kept).

Existing tests: **none changes.** The author applied a scratch version of §2 (not committed; patch path in
C/agent-reports/fix-doc-bh-me-admission-canonical.md) and ran the whole suite once (`ME_REQUIRE_GO=1 cargo nextest run --locked --no-fail-fast --build-jobs 2
-j 2`): `828 tests run: 828 passed, 2 skipped`. `record.rs:809` (`a_record_whose_card_identity_cannot_be_read_is_unconfirmed`)
calls `classify` on a padded `md1`, not `pack`, and stays as is because classification is unchanged. If the
builder's suite shows an existing test changing, the GREEN commit names it.

CHANGELOG (`crates/me-cli/CHANGELOG.md`, `## [Unreleased]` -> `### Fixed`, appended after the existing bullets),
**PROPOSED**: `- **\`me sysw pack\` refuses a record whose spelling differs from the one it would write**
(bughunt-1119 bh-me-I1, I6, I9 and I17, all Important; closes F-245). A record with a line break, leading or
trailing whitespace, a BIP-39 mnemonic whose words are not separated by single spaces, or a \`text:\`/\`pass:\` body
that is not UTF-8 used to pack at exit 0 and was written as given: padding was engraved, a wrapped record came back
as several, and the device did not recognise the seed. Each now exits 4 with a sentence naming the rule, before any
passphrase is generated. One byte-order mark at the start of \`--in\` or stdin is now removed (it used to be refused).
Classification and readers are unchanged.`

Gate: `bash scripts/fix-me-admission-canonical-gate.sh` (A: CLI rows R1-R16 and C1-C7; B: the whole nextest suite
once, each named test PASS exactly once, no FAIL line; C: doc-tests, `cargo fmt --check`, `ci/code-docs-check.sh`,
both clippy runs on 1.85.0). Its header states what it does not cover.

## 7. Points this document decides that the bug report and rulings do not literally state (PROPOSED, architect confirms)

1. **The four sentences** (§4). The rulings fix refuse-vs-normalise; the wording is new, modelled on the
   `record {i}, as given (records count from 0)` sentences in `sysw_error`.
2. **Admission, not classification** (§2 items 1-2). T29 names the I1 admission rule as P1's; the triage scratch fix
   for I9 changed `classify_with` instead. Changing the classifier would desynchronise the host from the device's
   lockstep classifier and the record-class vectors, and would weaken the argv secret guard, which classifies the
   trimmed record. So only admission changes.
3. **Spelling rules run only on a record that classifies.** An `Unknown` record keeps its existing, more specific
   sentence; e.g. a CR inside a `text:` body is still "not lowercase hex".
4. **A-PAD uses Unicode whitespace** (`char::is_whitespace`), so a NBSP or U+3000 pad is refused, not only ASCII.
   U+FEFF is not whitespace in Rust or Go; outside the one stripped stream BOM it reaches the existing Unrecognised
   sentence.
5. **"BOM" means one U+FEFF at the start of a stream channel** (`--in`, stdin, the `-` splice). Argv has no byte-order
   mark to normalise, so an argv record starting U+FEFF is refused as today (C6). The BOM is removed silently, with no
   note, as CRLF is today.
6. **Stream lines are not trimmed either.** B1 forbids normalising padding, so a padded line in a file is refused
   (R9, R10). This is a behaviour change for any operator file with trailing spaces; today those spaces are engraved.
7. **`--expect` still runs before admission** (main.rs:1786). For R1, `--expect mnemonic` is met and the run then
   exits 4 on T-SEP; `--expect` prints nothing when met, so no line vouches for the refused record.
8. **T-UTF8BODY prints the remedy line for `text:` only.** For `pass:` that line would invite typing the passphrase
   into a shell command, which shell history keeps; the `pass:` sentence therefore ends after "(§5.3.1)." with no
   remedy line. The existing `NonHexBody` sentence (main.rs:3748-3754) has the same exposure; changing it is a
   separate small item, filed as a Nit secret-handling line in C/agent-reports/fix-doc-bh-me-admission-canonical.md.

§7 items 1-7: confirmed as the review r1 recommends. Item 8: option (b), final (coordinator ruling, E9b).

## 8. T29 (I1, secret half): run, not a question

Scratch Go test on the fork at the me pin `7b6f2fbb` (not committed; kept at /scratch/cache/wt-me-admission-canonical.evidence/bh1119_t29_test.go): an unsealed payload
holding `M` raises F1 (`A SECRET is stored unencrypted in flash.`) and `take(ClassMnemonic)` returns it; the same payload
holding `M2`, `MT` or `MN` classifies `ClassUnknown`, `take` returns nothing, the composer door counts it inert,
and **no F1 warning is shown**. Unknown records reach no screen and no program (composer_door.go:35-37), so there is
no public-record display or transmit, which was T29's SECRET condition. The missing F1 warning for a seed in
cleartext flash is filed separately as a Minor secret-handling item (C/agent-reports/fix-doc-bh-me-admission-canonical.md); this FIX stops the host
producing such a payload, and the device is not changed here.
