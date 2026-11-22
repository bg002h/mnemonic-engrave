# FIX: descriptor intake refuses duplicate JSON members and out-of-range path indices (bh-me-I2, bh-me-I15)

- Source: bughunt 2026-11-19 keys **bh-me-I2** and **bh-me-I15**, both graded **Important** (grade-me.md:65, :78;
  `design/agent-reports/bug-hunt-2026-11-19.md` §I2, §I15). Architect table row #27, **small** track, one Opus
  review round. Not class B by the table; I2 changes which wallet a host reads (address and quorum), so the
  reviewer should read §1.1 as a funds-path defect.
- Ruling **T31** (decided, architect-bughunt-1119-questions.md:73): *refuse a JSON object with duplicate members,
  exact or case-variant. Host and device read different members today; no normalisation can pick the right one.*
  The I15 half needs no ruling: BIP-380 lists an unhardened index of 2^31 or more as invalid (§1.2).
- Base: master `73db74e07f8d20c7d22a47ddf928fb7d2f2d1161`. Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22.
- The one new holder-visible text (§4, T-DUP) is **PROPOSED (architect confirms)**. The I15 refusals reuse existing
  texts unchanged. §7 lists three points this document decides that the rulings do not literally state; each is
  marked PROPOSED there and in the author's report.
- **Files the build changes** (for build ordering against other me rows):
  `crates/me-cli/src/descriptor/cascade.rs`, `crates/me-cli/src/descriptor/gate.rs`,
  `crates/me-cli/src/descriptor/refusal.rs`, `crates/me-cli/src/descriptor/cascade_tests.rs`,
  `crates/me-cli/tests/descriptor_refusals.rs`, `design/SPEC_descriptor_input.md`, `crates/me-cli/CHANGELOG.md`.
  This document, the gate script `scripts/fix-me-descriptor-input-strict-gate.sh` and the fixtures under
  `design/evidence/fix-me-descriptor-input-strict/` are already committed on this branch.

## 1. The defects (measured on master 73db74e0)

All fixtures are signet: three `tpub` cosigners at `m/48'/1'/0'/2'` from the BIP-39 reference vectors 1-3
(fingerprints `73c5da0a`, `b8688df1`, `28645006`), `wsh(sortedmulti(2,…/<0;1>/*))`. `B` below is the same
descriptor with `sortedmulti(1,`. Every command is
`me sysw pack --no-passphrase [--as descriptor|--as md1] --out <f> --in design/evidence/fix-me-descriptor-input-strict/<fixture>`.

### 1.1 I2: a `{label, descriptor}` JSON with two members for one field

`parse_json` (`cascade.rs:887-945`) reads the document into `serde_json::Value`, whose map keeps only the LAST of
two byte-identical members, then the `field` closure (`cascade.rs:901-912`) takes the exact-case member first and
otherwise the first ASCII-case-insensitive one. The device (`nonstandard/parse.go:44-55`, and the record classifier
`sysw/descriptor.go:157-159`) uses Go `encoding/json` into a struct: each member that matches a field overwrites it,
so the **last** match wins, and matching uses Unicode simple case folding, which adds exactly two non-ASCII runes
to ASCII folding: U+017F LATIN SMALL LETTER LONG S `ſ` (≡ `s`) and U+212A KELVIN SIGN (≡ `k`) (enumerated over every rune with go1.26.7).

Measured, host on base vs a host Go probe of `encoding/json` (go1.26.7, the struct shape of `parse.go:44-47`):

| fixture | members, in order | host reads (exit, all `--as`) | Go reads |
| --- | --- | --- | --- |
| `i2-lower-first.json` | `descriptor`=A, `Descriptor`=B | **2-of-3**, exit 0 | **1-of-3** |
| `i2-long-s.json` | `descriptor`=A, `deſcriptor`=B | **2-of-3**, exit 0 | **1-of-3** |
| `i2-pretty.json` | `label`, `descriptor`=A, `DESCRIPTOR`=B | **2-of-3**, exit 0 | **1-of-3** |
| `i2-three.json` | `descriptor`=A, `label`, `Descriptor`=B, `DESCRIPTOR`=B | **2-of-3**, exit 0 | **1-of-3** |
| `i2-upper-first.json` | `Descriptor`=B, `descriptor`=A | 2-of-3, exit 0 | 2-of-3 |
| `i2-exact-dup.json` | `descriptor`=A, `descriptor`=B | 1-of-3, exit 0 | 1-of-3 |
| `i2-label-dup.json` | `label`="x", `Label`="y", `descriptor`=A | title "x", exit 0 | title "y" |
| `i2-null-dup.json` | `descriptor`=null, `Descriptor`=A | refused, exit 3 (missing-descriptor text) | 2-of-3 |
| `i2-unknown-dup.json` | `descriptor`=A, `devices`, `devices` | 2-of-3, exit 0 | 2-of-3 |

In the first four rows `me` prints the 2-of-3 wallet's address 0 and packs it, while the device shows the 1-of-3
wallet from the same file. Without `--as` the host exits 2 with the identification block of the 2-of-3 wallet.

### 1.2 I15: an unhardened path index of 2^31 or more is rewritten as hardened

`parse_path_element` (`cascade.rs:1137-1149`, a transcription of the fork's `bip32.ParsePathElement`,
`bip32/bip32.go:69-84`) accepts an unsuffixed index up to 2^32-1, so `2147483696` (= 2^31 + 48) becomes the value
of `48h`. BIP-32: "The normal child keys use indices 0 through 2^31-1"; BIP-380 (bip-0380.mediawiki:73) path
elements are `/NUM` or `/NUMh`, and its invalid vector at :251 is "Derivation index out of range: …/2147483648".
The function serves three places, and all three rewrite:

| fixture | where | base result |
| --- | --- | --- |
| `i15-origin-2147483696.txt` | key origin `[73c5da0a/2147483696/1h/0h/2h]` | exit 0; the `--as descriptor` and `--as md1` containers are **byte-identical** to `base.txt`'s (`/48h/`) |
| `i15-origin-2147483648.txt` | key origin | exit 0, packed as `/0h/` (wallet-id `79ebb513fffc4d432a6226564021c1c5`, base's is `02b9dd8f…`) |
| `i15-origin-4294967295.txt` | key origin | `--as descriptor` exit 0 as `/2147483647h/`; `--as md1` exit 3 naming "varint value 2147483647" |
| `i15-bw-2147483650.txt` | BlueWallet `Derivation: m/48'/1'/0'/2147483650` | exit 0; container **byte-identical** to `bw-base.txt`'s (`/2'`) |
| `i15-usesite-2147483696.txt` | use-site `/2147483696/*` | exit 3, but the echo prints `/48h/*` and the refusal is the **hardened use-site** text, about a step the holder did not write |

Addresses are unchanged (the origin is metadata), so the wrong thing is the backup's recorded origin, which a
restoring wallet uses to find the key.

## 2. Decisions

- **D1 (T31). Duplicate members are refused.** When `parse_json` claims the document as a top-level JSON object,
  it lists that object's member names in document order (a serde `MapAccess` visitor over the same input, values
  skipped as `IgnoredAny`; `serde_json::Value` cannot be used because it has already merged exact duplicates).
  If any two names are equal under the fold of D2, the branch fails with a new `JsonError::DuplicateMember`
  carrying every name of the first colliding group: the group of the earliest name in document order that has a
  later equal, all members of that group, as written, in document order.
- **D2. The fold is Go `encoding/json`'s, for every member name** (PROPOSED, see §7 P-1): ASCII letters compare
  case-insensitively, U+017F compares equal to `s`/`S`, U+212A compares equal to `k`/`K`, and every other
  character compares exactly. This is the complete set of non-ASCII runes whose simple-fold orbit contains an
  ASCII character (measured), so the host refuses every pair the device would treat as one field name, for the two
  fields the device reads.
- **D3. Order of checks.** The duplicate check runs after the document is claimed (the `NotClaimed` rules are
  unchanged: non-object, non-null top level, or a non-string `label`/`descriptor`), and **before** field lookup.
  So `{"descriptor":null,"Descriptor":"…"}` gets the duplicate text, not the missing-descriptor text, and a
  duplicate whose values are both malformed descriptors gets the duplicate text, not the inner-parse text.
  A top-level `null` has no members and is unchanged.
- **D4. Scope is the top-level object** (PROPOSED, see §7 P-2). Members of nested values (for example the
  `devices` array of objects that exports carry) are not inspected: `i2-nested-dup.json` stays admitted.
- **D5. Field lookup is unchanged.** `field` keeps exact-then-ASCII-case-insensitive matching (spec §4.4). After
  D1 at most one member can match each field, so lookup order no longer matters. A lone `deſcriptor` member
  (no `descriptor`) stays a missing-descriptor refusal; the device admits it; that divergence refuses, so it is
  left alone (§7 P-1).
- **D6. Refusal row and exit.** `DuplicateMember` maps to §6 row `json-inner-malformed` (row 5) with its own
  sentence, exactly as F-438's missing-descriptor sentence does (`refusal.rs:517-538`). **`EXIT_REFUSED` (3)**
  on both `--as` paths. Without `--as` the existing §5.1 gate decides, unchanged: every fixture whose text holds
  a descriptor gets the same text at 3 (measured on the probe), and `{"label":"a","label":"b"}` keeps today's
  record refusal at 4 (F-429's invariant). §6 stays 35 data rows; `Row::ALL` is unchanged.
- **D7. Path indices.** `parse_path_element` refuses an element whose number (the part before an optional `h` or
  `'`) is greater than 2147483647, hardened or not. For hardened elements this changes nothing (the existing
  overflow check already refuses them); for unhardened ones it is the fix. The existing error variants carry the
  refusal, so the texts are unchanged: `InvalidOriginPath`, `InvalidChildrenPath` and BlueWallet
  `InvalidDerivation`, each reached through §6 row 1 (`unparseable`) with the branch's reason. Exit 3, all modes.
- **D8. Unchanged.** The device, the Go twin (`bip32.ParsePathElement` in the fork, FOLLOWUPS
  `sh-parsepath-unhardened-overflow`, deferred Minor), the shared seam vectors (`descriptor_seam_vectors.json`
  carries no row for either defect, and adding one needs a device measurement), the `+`/`-` sign spelling (§7
  P-3), and every text other than T-DUP.

## 3. Spec delta (`design/SPEC_descriptor_input.md`)

- **§4.3**, the table "Shapes the Go parser **accepts and should not**": insert the row directly after the `` `wsh(sortedmulti(2, 21 keys))` `` row and before the `` `sh(wpkh(sortedmulti(2,…)))` `` row, so the AMENDED note's "last three rows" still names the three rows the device now refuses:
  `| an unhardened path index of 2^31 or more — `[fp/2147483696/…]`, `Derivation: m/…/2147483650`, `…/2147483696/*` | ACCEPT, read as the hardened index 48 (2) | BIP-32 reserves 0 to 2^31-1 for normal children and BIP-380 lists `…/2147483648` as invalid; the backup would record a path the holder did not write. `me` refuses it (§6 row 1, the branch's "is not a path" reason) — FIX_me_descriptor_input_strict D7. |`
- **§4.4**, after the paragraph ending "…is still a needless divergence.", add:
  `**AMENDED (FIX_me_descriptor_input_strict, bughunt-1119 I2, ruling T31): a member named twice is REFUSED.** Go's `encoding/json` lets the LAST matching member win and folds case with Unicode simple folding (beyond ASCII, U+017F `ſ` ≡ `s` and U+212A KELVIN SIGN ≡ `k`); `me`'s lookup took the exact-case member first. With two members for one field the two sides read different wallets (measured: host 2-of-3, device 1-of-3). No reading is safe to pick, so `me` refuses a top-level object in which two member names are equal under that fold, exact repeats included, whatever the field. Nested values are not inspected. The check runs before field lookup. §6 row 5 carries the text.`
- **§6 row 5** (`a wrapper whose inner descriptor is malformed — or absent`), append inside the cell, after the
  F-438 amendment:
  `*(AMENDED, FIX_me_descriptor_input_strict: a member named twice gets its own sentence — "…names one field more than once: `descriptor`, `Descriptor`. A wallet export names each field once, and field names match without regard to case, so this file does not say which value is meant. Re-export from your wallet software. If you edit the file instead, keep only the member whose value is your wallet's, and check the quorum and address `me` prints next." Same row, `EXIT_REFUSED` (3).)*`

## 4. Printed texts and exit codes

| id | trigger | stderr line (exact) | exit |
| --- | --- | --- | --- |
| **T-DUP** (new, PROPOSED) | D1 | ``me: the `{label, descriptor}` JSON names one field more than once: <names>. A wallet export names each field once, and field names match without regard to case, so this file does not say which value is meant. Re-export from your wallet software. If you edit the file instead, keep only the member whose value is your wallet's, and check the quorum and address `me` prints next.`` | 3 |
| T-ORIGIN (existing) | D7, key origin | `me: this is not a wallet descriptor in any of the four forms … It looks most like a plain BIP-380 descriptor, which failed because: the origin path is not a path: `<fp/path as written>`.` | 3 |
| T-USESITE (existing) | D7, use-site | same frame, reason ``the use-site path is not a path: `<tail as written>` `` | 3 |
| T-DERIV (existing) | D7, BlueWallet | same frame, "looks most like a BlueWallet `Key: value` setup file", reason ``the `Derivation:` header is not a path: `<value>` `` | 3 |

`<names>` is each name of the group in backticks, escaped by `refusal::quote_operator`, joined by `, `. The full
expected lines for every fixture are the files in `design/evidence/fix-me-descriptor-input-strict/want/`.
stdout stays empty on every refusal. No flag, type or command is added.

## 5. RED rows

The rows are `design/evidence/fix-me-descriptor-input-strict/cases.txt` (66 rows: fixture, `--as` mode, exit,
expected line), run by `bash scripts/fix-me-descriptor-input-strict-gate.sh --cli`.

- **On base 73db74e0 (run by the author): 44 FAIL, 20 ok; with the three `i2-escaped.json` rows folded in at r1, 47 FAIL, 20 ok (re-measured at the FIX head and at the build head).** The 47 are every I2 and I15 row: exit 0 (or 2
  without `--as`) where 3 is wanted; `i2-null-dup` and `i2-label-only` (`--as`) print the missing-descriptor
  sentence; `i15-usesite-2147483696` prints the hardened-use-site sentence; `i15-origin-4294967295` `--as md1`
  prints the md1 varint sentence. The 20 ok are the unchanged-behaviour controls (`base.txt`, `bw-base.txt`,
  `i2-control.json`, `i2-nested-dup.json` at 0/0/2; `i15-origin-2147483647.txt` at 0, 3 with the md1 varint
  sentence, 2; `i15-usesite-range.txt` at 3 with its existing reason; `i2-label-only.json` without `--as` at 4)
  and the container check (the 2147483647 control packs a container different from `base.txt`'s).
- **GREEN target: 67 ok (66 rows plus the container check), `gate --cli: PASS`.** The author reached it on an uncommitted probe of D1-D7 (never
  committed; the builder writes the real change test-first), which also ran the whole workspace suite: 715 passed,
  2 skipped, 0 failed, so no existing test changes.

Rows at a glance (each I2/I15 fixture runs in all three modes at exit 3 unless noted):

| fixture | expected |
| --- | --- |
| `i2-lower-first.json` | T-DUP, names `` `descriptor`, `Descriptor` `` |
| `i2-upper-first.json` | T-DUP, `` `Descriptor`, `descriptor` `` |
| `i2-exact-dup.json` | T-DUP, `` `descriptor`, `descriptor` `` |
| `i2-label-dup.json` | T-DUP, `` `label`, `Label` `` |
| `i2-long-s.json` | T-DUP, `` `descriptor`, `deſcriptor` `` |
| `i2-pretty.json` | T-DUP, `` `descriptor`, `DESCRIPTOR` `` |
| `i2-unknown-dup.json` | T-DUP, `` `devices`, `devices` `` |
| `i2-null-dup.json` | T-DUP, `` `descriptor`, `Descriptor` `` (D3) |
| `i2-three.json` | T-DUP, `` `descriptor`, `Descriptor`, `DESCRIPTOR` `` |
| `i2-escaped.json` | T-DUP, `` `descriptor`, `Descriptor` `` (the second name written `Descr\u0069ptor`) |
| `i2-label-only.json` | `--as` both: T-DUP `` `label`, `label` ``; no `--as`: the existing record refusal, exit 4 |
| `i15-origin-2147483696.txt` | T-ORIGIN, `` `73c5da0a/2147483696/1h/0h/2h` `` |
| `i15-origin-2147483648.txt` | T-ORIGIN, `` `73c5da0a/2147483648/1h/0h/2h` `` |
| `i15-origin-4294967295.txt` | T-ORIGIN, `` `73c5da0a/4294967295/1h/0h/2h` `` |
| `i15-usesite-2147483696.txt` | T-USESITE, `` `2147483696/*` `` |
| `i15-bw-2147483650.txt` | T-DERIV, `` `m/48'/1'/0'/2147483650` `` |

## 6. Tests

No existing test changes (probe-measured, §5). New tests, each named exactly as here (the gate counts each
PASS once):

- `cascade_tests.rs`:
  - `json_duplicate_members_are_refused_exact_or_case_variant`: every I2 fixture shape (exact, case-variant,
    three-way, unknown member) gives `JsonError::DuplicateMember` with the group in document order.
  - `json_duplicate_detection_folds_like_go_encoding_json`: `deſcriptor`/`descriptor` and a KELVIN SIGN pair
    (`{"\u212ax":1,"kx":2,"descriptor":…}`, JSON escape as written) collide; `{"Ä":1,"ä":2,"descriptor":…}` does
    not (D2 compares every other character exactly).
  - `json_duplicate_check_runs_before_field_lookup`: the null-plus-variant input and a duplicate of two malformed
    descriptors both give `DuplicateMember`, not `MissingDescriptor` or `Inner`.
  - `json_nested_duplicates_are_not_inspected`: `i2-nested-dup` shape parses.
  - `path_element_refuses_an_index_above_2_pow_31_minus_1`: `2147483648`, `2147483696`, `4294967295` refused
    unhardened; the origin, use-site (`/2147483696/*`) and `Derivation:` (`parse_path_m`) callers all return
    their existing error variants.
  - `path_element_admits_2147483647_unhardened_and_hardened`: `2147483647` → 2147483647, `2147483647h` →
    0xFFFFFFFF, `0` and `0h` unchanged.
- `tests/descriptor_refusals.rs` (names do not start with `row_`, so the 35-row count test is unchanged):
  - `json_duplicate_member_gets_its_own_sentence`: `assert_row("json-inner-malformed", <lower-first doc>,
    ["--as","descriptor"], 3, <T-DUP line>)`, plus: stderr does not carry "not the JSON" and does not carry
    "carries no `descriptor` field".
  - `out_of_range_path_index_is_refused_not_rewritten`: T-ORIGIN for `/2147483696/` at 3 under both `--as`, and
    stderr never prints `48h` for that input.
- The CHANGELOG `[Unreleased]` / `### Fixed` gets one entry for both keys (wording is the builder's; it states
  the two refusals and that exit codes elsewhere are unchanged).

## 7. Points this document decides that the rulings do not literally state (PROPOSED, architect confirms)

- **P-1 (D2/D5) fold set.** Options: (a) ASCII case only; (b) ASCII case plus the two non-ASCII runes whose Go
  fold reaches an ASCII letter (U+017F, U+212A), everything else exact; (c) Go's full simple folding for every
  name (so an unknown `Ä`/`ä` pair also collides). **Chosen (b)**: every pair the device can read as `label` or
  `descriptor` is caught, which (a) misses (`i2-long-s`: host 2-of-3, device 1-of-3). (c) is the most literal
  reading of T31's "case-variant", but it only adds unknown non-ASCII names the device never reads, and Rust's
  std has no simple-fold table, so it would need a generated table or a new dependency. Field lookup
  stays ASCII (D5), so a lone `deſcriptor` is still refused as missing; widening lookup would admit a new input
  class and is not needed to close I2.
- **P-2 (D4) nesting.** Options: (a) top-level only; (b) every object at any depth. **Chosen (a)**: T31's reason
  is that host and device read different members; neither reads nested values, and (b) would refuse exports
  whose unread device metadata is sloppy.
- **P-3 sign spelling (not in this FIX).** `parse_path_element` (and the fork's `strconv.ParseInt`) also accept
  `+48h` and `-0`, re-encoded as `48h` and `0`; measured on the mainnet twin, `/+48h/` packs a container
  byte-identical to `/48h/`. Same class as I15 (a non-BIP-380 spelling silently rewritten), but not a graded key.
  `composer_records.rs:534-542` already refuses `+` for the composer parser. Options: (a) add a digits-only rule
  to D7 here (no new text; same T-ORIGIN/T-USESITE/T-DERIV reasons; two more fixtures); (b) file it as its own
  FOLLOWUPS line. **Recommend (a)** if the architect wants it in this build; the document as written does (b).
