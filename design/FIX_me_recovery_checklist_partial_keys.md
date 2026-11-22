# FIX: `me bundle` names the cosigner keys a partly keyed md1 does not carry (bh-me-C1)

- Source: bughunt 2026-11-19 key **bh-me-C1**, graded **Critical** (architect ruling T3, Fable upheld;
  `design/agent-reports/bug-hunt-2026-11-19.md` §C1). **Class B (funds path).** Full track: this document is the
  spec delta, the RED rows and the build order; there is no separate plan. Up to three review rounds.
- Ruling T3 (decided): the md-codec API already exposes slot indices (`TlvSection::pubkeys:
  Option<Vec<(u8, [u8; 65])>>`), so missing slots = `0..d.n` minus those indices. **No md-codec pin bump.**
  Re-checked at the current pin c8bb97a5 (md-codec 0.48.5): `tlv.rs:31-33` is unchanged, and
  `read_sparse_tlv_idx` (`tlv.rs:360-376`) refuses an index `>= n` or not strictly ascending, so present
  indices are distinct, ascending and in range.
- Release: a fixed me release per fleet rulings 2026-11-22 item 4 (no advisory, no yank).
- Base: master `2987f388`. Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22.
- Every holder-visible text below is **PROPOSED (architect confirms)**; its WORDING_REVIEW row is listed in the
  author's report, not added here.

## 1. The defect (measured on master 2987f388)

`bundle.rs:429` and `bundle.rs:464` set `keyless_template |= !d.is_wallet_policy();`, and
`is_wallet_policy()` is true as soon as the `Pubkeys` TLV holds **one** entry. So a 2-of-3 md1 that carries one
xpub is not flagged, and `Manifest::checklist` (`manifest.rs:106`) prints nothing about the two xpubs it
lacks. Without them the descriptor cannot be rebuilt (md-codec's own derivation refuses with
`MissingPubkey { idx: 1 }`), so a holder who keeps exactly the listed plates and later loses one cosigner seed
cannot restore the wallet.

Measured with the signet fixture `design/evidence/fix-me-checklist-partial-keys/k1.txt` (3 chunks, xpub on
`@0` only):

    me bundle < design/evidence/fix-me-checklist-partial-keys/k1.txt
    exit 0
    me: backup needs 3 public plates, plus one ms1 plate per seed you hold:
    me: NOTE — this policy declares 3 key slots. `me bundle` cannot know how many of those seeds are yours, so ms1 plates are NOT counted above: hold all 3 and you need 3 of them.
    (manifest: "key_slots": 3, "keyless_template": false, no other key information)

The consultant's own fixture (three mainnet chunks of a public BlueWallet xpub fixture, slot `@0` only, printed
in the bug-hunt report §C1) gives the same output; it is evidence only and is not used by any test (signet-only
rule).

## 2. Decisions (all decided here; nothing left to the builder)

- **D1. Scope of "policy".** One md1 *policy* = one unchunked md1 string, or one chunk set. Each is decoded
  already (`bundle.rs:418-428`, `:450-457`). The rules below run once per policy.
- **D2. Present and missing slots.** present = the indices in `d.tlv.pubkeys` (empty when the TLV is absent),
  in wire order (ascending). missing = every `i` in `0..d.n` not in present, ascending.
- **D3. Fingerprint of a missing slot.** The entry of `d.tlv.fingerprints` **whose index equals the slot**
  (looked up by index, never by position), rendered as 8 lowercase hex digits. When the TLV is absent or has no
  entry for that slot, there is no fingerprint: it is never fabricated or borrowed from another slot.
- **D4. Exit code and refusals: unchanged.** The command still exits 0 for a partly keyed policy (same as the
  F-602 template case); this FIX adds information and refuses nothing. No new flag, type of input or command.
- **D5. No mk1 binding.** `me bundle` does not try to match a supplied mk1 card to a key slot. An mk1 card for a
  missing slot in the same bundle changes neither the note nor the JSON entry; the note says so in its last
  sentence. (Consultant: "When a supplied mk1 cannot be bound to the slot, leave that requirement explicit";
  binding by fingerprint would fail for `--privacy-preserving` cards and for two cards sharing an origin, and a
  wrong binding would silence the warning. Never binding is the fail-closed reading.)
- **D6. Templates keep the F-602 note.** A policy with **no** xpub keeps exactly today's F-602 TEMPLATE note
  (`manifest.rs:129-137`, text and condition unchanged) and gets **no** partial-keys note. It does get a JSON
  entry (§3.1), so the JSON field alone answers "which slots lack an xpub" for every policy.
- **D7. Everything else unchanged.** The headline ("backup needs N public plate(s)..."), `wallet_plates`,
  `key_slots`, `keyless_template`, `hashlock_kinds`, `sets`, `plates`, the plate lines, the key-slots note and
  the hashlock note keep their texts, values and conditions. stdout differs only by the new field.

## 3. Spec delta (`design/SPEC_me_bundle_phaseA.md`)

### 3.1 §6 Manifest schema: new field `missing_keys` (PROPOSED)

Append to §6 after the `card_fingerprint` / `card_path` bullet:

> - **`missing_keys`** lists every md1 policy whose card does not carry the xpub of each of its key slots (bh-me-C1).
>   One entry per policy (an unchunked md1 string, or a chunk set) that lacks at least one xpub, in plate order;
>   a key-less template has an entry too, with every slot missing. The field is **omitted** when no policy lacks
>   an xpub. It sits after `keyless_template` in the object. Entry keys:
>   - `md1_plates`: the `plate` numbers of that policy's md1 plates, ascending.
>   - `key_slots`: that policy's declared key slots (`Descriptor.n`).
>   - `present_slots`: the slot indices whose xpub the card carries, ascending (may be empty).
>   - `missing_slots`: one object per slot without an xpub, ascending by `slot`: `{"slot": <index>}` plus
>     `"fingerprint": "<8 lowercase hex>"` when the card's fingerprint record has an entry **for that slot**;
>     the key is omitted otherwise, never fabricated.
>   - `me bundle` does not match supplied mk1 cards to slots, so an mk1 card in the bundle does not remove a slot
>     from `missing_slots`.

Example (fixture `k1`, xpub on `@0` only, fingerprints on all three slots):

    "missing_keys": [
      { "md1_plates": [1, 2, 3], "key_slots": 3, "present_slots": [0],
        "missing_slots": [ { "slot": 1, "fingerprint": "b8688df1" }, { "slot": 2, "fingerprint": "28645006" } ] }
    ]

### 3.2 §7 Guided workflow checklist: the partial-keys note (PROPOSED)

Append to §7 (after the collision-suffix paragraph):

> **Partly keyed policy note (bh-me-C1).** For each `missing_keys` entry whose `present_slots` is **not** empty,
> in entry order, the checklist prints one line. These lines come after the F-602 TEMPLATE note (if any) and
> before the key-slots note. An entry whose `present_slots` is empty (a key-less template) prints no such line;
> the TEMPLATE note covers it. The line is:
>
>     me: NOTE — the md1 policy on <PLATES> carries the <XPUB> of key <SLOT> <PRESENT> only. It does NOT carry <MISSING-CLAUSE> `me bundle` does not match mk1 cards to key slots.
>
> - `<PLATES>`: `plate <a>` when the policy has one md1 plate, else `plates <a>-<b>` (first and last of
>   `md1_plates`, joined by a hyphen-minus).
> - `<XPUB>` `<SLOT>`: `xpub` `slot` when `present_slots` has one element, else `xpubs` `slots`.
> - `<PRESENT>`: each present slot as `@<i>`, joined by `, `.
> - `<MISSING-CLAUSE>`, with `<M>` = each missing slot as `@<i> [<fingerprint>]`, or `@<i>` when it has no
>   fingerprint, joined by `, `:
>   - one missing slot: `the xpub of key slot <M>, and the wallet cannot be restored without it: keep an mk1 card (or the seed) for it.`
>   - two or more: `the xpubs of key slots <M>, and the wallet cannot be restored without them: keep an mk1 card (or the seed) for each of them.`
>
> The dash after `NOTE` is U+2014, as in the other notes; the backticks around `me bundle` are printed. One line,
> no wrapping.

Rendered (all measured on a throwaway prototype; these are the exact bytes the gate compares):

    k1     (present @0):        me: NOTE — the md1 policy on plates 1-3 carries the xpub of key slot @0 only. It does NOT carry the xpubs of key slots @1 [b8688df1], @2 [28645006], and the wallet cannot be restored without them: keep an mk1 card (or the seed) for each of them. `me bundle` does not match mk1 cards to key slots.
    k02    (present @0, @2):    me: NOTE — the md1 policy on plates 1-4 carries the xpubs of key slots @0, @2 only. It does NOT carry the xpub of key slot @1 [b8688df1], and the wallet cannot be restored without it: keep an mk1 card (or the seed) for it. `me bundle` does not match mk1 cards to key slots.
    k1nofp (no fingerprint @1): me: NOTE — the md1 policy on plates 1-3 carries the xpub of key slot @0 only. It does NOT carry the xpubs of key slots @1, @2 [28645006], and the wallet cannot be restored without them: keep an mk1 card (or the seed) for each of them. `me bundle` does not match mk1 cards to key slots.

`plate <a>` cannot occur on the CLI today (one 65-byte xpub already forces a chunked md1), so its row (T9) is a
unit test on the renderer; the rule is still normative.

### 3.3 Existing §7 text

No other §7 sentence changes. (§7's sample headline "backup needs 4 plates (3 public + ms1 on device)" is stale
since F-580; that is not this FIX's and is not touched.)

## 4. Code changes (prose; the builder writes the code)

- `crates/me-cli/src/manifest.rs`: two new serialisable types for an entry and a missing slot (field names
  exactly as §3.1; `fingerprint` skipped when `None`); a `missing_keys` field on `Manifest` placed after
  `keyless_template` (`manifest.rs:93`), skipped when empty (the `hashlock_kinds` pattern, `manifest.rs:81-82`);
  the §3.2 line rendered in `checklist()` between the TEMPLATE note (`:129-137`) and the key-slots note
  (`:138`). The two test literals at `:325` and `:410` gain an empty `missing_keys`.
- `crates/me-cli/src/bundle.rs`: at both decode sites (`:429` unchunked, `:464` chunked) record the policy's
  first plate position (`plates.len()` before its plates are pushed), its plate count (1, or the chunk count), and
  D2/D3's present/missing/fingerprint data; when building the `Manifest` (`:571`) emit one entry per policy with
  a non-empty missing list. Plate numbers are position + 1, which is what the renumbering at `:564-568` assigns.
  `keyless_template` keeps its current computation.
- `crates/me-cli/CHANGELOG.md` `[Unreleased]`, new `### Fixed` entry (PROPOSED):
  > - **`me bundle` names the cosigner keys a partly keyed md1 does not carry** (bughunt-1119 bh-me-C1,
  >   Critical). A 2-of-3 md1 carrying one xpub got a checklist that said nothing about the other two, at exit 0;
  >   F-602 had covered only the key-less template. Each such policy now gets a NOTE naming its missing key slots
  >   (with each slot's fingerprint when the card carries it), and the manifest gains `missing_keys`: one entry
  >   per md1 policy that lacks any xpub, key-less templates included. Exit codes and all other fields are
  >   unchanged.
- No toolkit-manual lockstep: `mnemonic-toolkit` master `docs/manual/src/40-cli-reference/` has no `me` page
  (files 41-mnemonic, 42-md, 43-ms, 44-mk-cli, 45-mr, 46-mr-sitting; no `me bundle` text, checked at
  `cdedd8a2`). README.md has no checklist text.

## 5. RED rows (TRACE)

Fixtures: `design/evidence/fix-me-checklist-partial-keys/` — `k0` (no xpub), `k1` (`@0`), `k1only` (`@1`),
`k02` (`@0`, `@2`), `k3` (all), `k1nofp` (`@0`; fingerprints on `@0` and `@2` only), `mk1-slot1` (the mk1
card of slot `@1`, b8688df1, bound to `k1`'s stub). Signet, BIP-39 published test-vector seeds, produced by
`make-fixtures.sh` there (`--check` reproduces them byte for byte). `<case>.notes` holds the exact `me: NOTE`
lines and `<case>.missing.json` the exact `missing_keys` value (`null` = absent) for each case in `cases.txt`.

Rows T1-T8 are CLI tests in a new `crates/me-cli/tests/partial_keys.rs`. Each embeds its fixture strings as
constants, byte-identical to the evidence files (gate step C), runs `me bundle` on them via stdin, and asserts:
exit 0; the stderr lines starting `me: NOTE` equal the case's `.notes` lines exactly, in order, nothing extra;
the manifest's `missing_keys` equals the case's `.missing.json`. T9 is a unit test in `manifest.rs`'s test
module. The RED commit must compile (see §8 step 1), so every row fails on an assertion, not a build error.

| Row (test name) | Input | RED on master (observed by the gate, §8) | GREEN expected | Mutant it catches |
| --- | --- | --- | --- | --- |
| T1 `partial_keys_one_of_three_names_two_missing_slots` | k1 | NOTE lines = key-slots note only; `missing_keys` absent | `k1.notes` (the k1 line of §3.2, then the key-slots note); `k1.missing.json`; also the first stderr line is exactly `me: backup needs 3 public plates, plus one ms1 plate per seed you hold:` (D7) | note gated on `!is_wallet_policy()` (today's condition); note dropped |
| T2 `partial_keys_middle_slot_only_names_slots_zero_and_two` | k1only | as T1 | missing `@0 [73c5da0a], @2 [28645006]`, present `@1`; `k1only.*` | present computed as `0..pubkeys.len()` (would name `@1, @2`) |
| T3 `partial_keys_two_of_three_names_the_one_missing_slot` | k02 | as T1 | the k02 line of §3.2 (singular clause, `plates 1-4`); `k02.*` | always-plural clause; present list truncated to one slot |
| T4 `partial_keys_missing_fingerprint_renders_bare_slot` | k1nofp | as T1 | the k1nofp line (`@1` bare); `{"slot":1}` with no `fingerprint` key | fingerprint looked up by position (would give `@1 [28645006]`); a placeholder like `[]` or `[unknown]` |
| T5 `fully_keyed_policy_has_no_missing_keys` (control) | k3 | passes (key-slots note only, field absent) | same | note or entry emitted unconditionally |
| T6 `keyless_template_lists_every_slot_and_keeps_the_template_note` | k0 | NOTE lines correct (TEMPLATE + key-slots); `missing_keys` absent | `k0.notes` (TEMPLATE note, key-slots note; no partial note); entry with `present_slots: []` and all three slots with fingerprints | templates left out of `missing_keys`; partial note also printed for templates |
| T7 `template_and_partial_policies_are_reported_in_plate_order` | k1 then k0 (stdin order) | TEMPLATE + key-slots notes only; field absent | `mix.notes`: TEMPLATE note, then the partial note naming `plates 2-4`, then key-slots note; `mix.missing.json`: the k0 entry (`md1_plates [1]`) then the k1 entry (`[2, 3, 4]`) | plate numbers counted per policy from 1; entries in input order instead of plate order; one aggregated entry |
| T8 `partial_keys_note_fires_with_the_mk1_card_present` | k1 then mk1-slot1 | key-slots note only; field absent | `k1mk.notes` = the k1 line unchanged + key-slots note; `k1mk.missing.json` still lists slot 1 | binding an mk1 card to a slot by fingerprint and dropping it from the list |
| T9 `partial_keys_note_single_plate_form` (unit) | a `Manifest` built in the test: one entry `md1_plates [2]`, `key_slots 2`, `present_slots [0]`, `missing_slots [{slot 1, no fingerprint}]` | the line is absent (renderer prints nothing for the field) | `checklist()` contains exactly `me: NOTE — the md1 policy on plate 2 carries the xpub of key slot @0 only. It does NOT carry the xpub of key slot @1, and the wallet cannot be restored without it: keep an mk1 card (or the seed) for it. `me bundle` does not match mk1 cards to key slots.` | `plates 2-2`; range rendered from the plate count instead of first/last |

Unchanged rows that must stay green (named in the gate): `a_keyless_template_says_the_cosigner_cards_are_missing`
(bundle.rs:901; the F-602 note's text and condition) and `bundle_manifest_golden` (after the §6 re-bless).

## 6. Coverage map (rule -> rows)

| Rule | Rows |
| --- | --- |
| D1 one entry/note per policy | T7 (two policies, two entries), T1 |
| D2 present/missing by index | T1, T2 (non-prefix present set), T3, T6 (no TLV: all missing) |
| D3 fingerprint by slot index; omitted, never fabricated | T4, T1, T6 |
| D4 exit 0, nothing refused | T1-T8 (each asserts exit 0) |
| D5 no mk1 binding | T8 |
| D6 template keeps F-602 note, no partial note, has an entry | T6, T7, `a_keyless_template_says_the_cosigner_cards_are_missing` |
| D7 other fields/texts unchanged | T1 (headline line), T5, the exact `.notes` comparison in T1-T8 (other notes byte-identical), `bundle_manifest_golden` |
| §3.1 field omitted when empty | T5 |
| §3.1 entry order = plate order | T7 |
| §3.2 placement between TEMPLATE and key-slots notes | T7 (`mix.notes` order), T1 |
| §3.2 singular/plural forms | T1 (1 present, 2 missing), T3 (2 present, 1 missing), T9 (1 and 1) |
| §3.2 `plate <a>` vs `plates <a>-<b>` | T9, T1 |

## 7. Tests that change

- `crates/me-cli/tests/vectors/bundle-md1-mk1.json` (golden; read by `cli.rs:363` `bundle_manifest_golden` and
  `cli.rs:928` `preview::no_preview_flag_is_byte_for_byte_phase_a`): its md1 `md1yqpqqxqq8xtwhw4xwn4qh` is a
  1-slot key-less template, so the manifest gains, after `"keyless_template": true`:
  `"missing_keys": [ { "md1_plates": [1], "key_slots": 1, "present_slots": [], "missing_slots": [ { "slot": 0 } ] } ]`.
  Re-bless that one field only, in its own commit. Measured on the prototype: exactly these two tests fail before
  the re-bless; no other existing test changes (`705 run`, the other failures were the uninitialised submodule).
- `crates/me-cli/src/manifest.rs` test literals at `:325` and `:410`: add the empty field (compile only).
- No test is deleted.

## 8. Build order and gate

1. **RED commit.** Add the two types and the `missing_keys` field, always empty and not rendered (so the tree
   compiles), the two test-literal updates, `tests/partial_keys.rs` (T1-T8) and T9. Run the gate (below):
   T1-T4 and T6-T9 fail on their assertions, T5 passes; paste the failure lines in the commit message.
2. **GREEN commit.** Populate the field (D1-D3) and render the §3.2 line; spec delta §3.1-§3.2 into
   `SPEC_me_bundle_phaseA.md`; CHANGELOG entry. All rows pass except the golden pair.
3. **Re-bless commit.** Only the golden field named in this document's §7 (Tests that change).
4. Gate GREEN once on the head; paste its summary into the PR.

Gate (committed): `bash scripts/fix-me-checklist-partial-keys-gate.sh` (needs
`git submodule update --init third_party/seedhammer`; `--cli` runs step A only). It runs: A the eight CLI cases
against the built `me` (exit 0, exact NOTE lines, exact `missing_keys`); B the fixture producer in `--check`
mode; C every fixture string present in `tests/partial_keys.rs`; D the whole nextest suite once
(`ME_REQUIRE_GO=1`, Go on PATH, `--locked`, `--build-jobs 6 -j 6`), no FAIL line, and each of the eleven named
tests PASS exactly once; E doc-tests, `cargo fmt --check` and the CI `test` job's two clippy runs on 1.85.0.
**Not covered:** the CI jobs this change does not touch (Go tests of the me-preview sidecar and ndef oracle,
OTP e2e, shellcheck, SH2 install links, picotool-probe, cross builds; the PR's CI runs them) and wording
quality.

Gate run on unmodified master `2987f388` (RED shown): step A FAIL for k1, k1only, k02, k1nofp, k0, mix, k1mk and
`ok k3`; B ok (7/7); C FAIL (no test file); D `705 tests run: 705 passed, 2 skipped` and the nine new names
`passed 0 times`; E clean. On a throwaway prototype of §4: `gate --cli: PASS` (8/8).

## 9. FOLLOWUPS

None appended here (PR rule). The author's report lists, in the 2026-11-20 format: the owed-to-build line this
FIX's GREEN commit closes, and three lines it files (multi-policy note aggregation, the F-602 note's "not counted
here", the local lint gate's nightly leg).
