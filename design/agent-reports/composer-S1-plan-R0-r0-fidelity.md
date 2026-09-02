# Composer S1 plan — R0 round 0, PLAN-TO-SPEC FIDELITY lens

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md`, unchanged since
`108fd4c`. Reviewed against repo state `46fc91b`; the tree moved to `b05f3c4` during the
review (push agent + the `now:`-default ruling landing) and the plan file is byte-identical
at both (`git diff 46fc91b b05f3c4 -- <plan>` is empty).

**Lens:** does the plan, executed as written, produce the records, refusals and outputs
`SPEC_wallet_policy_composer.md` §6a / §8n / §8r / §10 / §12 item 8 define — and what does
it make false elsewhere (payload spec, `me`'s records/tests/fixtures, `ms`'s templates and
tests, the composer spec's own host statements, the sibling specs).

**Scope:** read-only on every repo; nothing committed; the only file written is this one.
Not re-derived (per brief): the spec's rulings, compile-ability of the plan's new files, the
current pass/fail state, the bip48-p2tr oracle. Out of scope: Stage 2/3 device work.

**What I ran** (full list at the end): I extracted the plan's `composer_records.rs` (Task 1
step 3 + Task 3's `CASES`) into a standalone `bitcoin 0.32` crate in the scratchpad and
executed `parse()` over ~55 constructed edge inputs, including real depth-2/3/4/5 xpubs and
tpubs derived in the same probe; re-derived all four Task 5 oracle xpubs with a THIRD
implementation (rust-bitcoin bip32 over the Python-computed BIP-39 seed); byte-compared the
four §8n blockquotes against the strings `line()` produces; ran the repo's `plan-cite-check`,
`plan-table-check`, `plan-glyph-check` and `plan-staleness-check` on the plan; read the fork's
Go `bip380.ParseKey` / `bip32.ParsePath` for lockstep; machine-checked that no existing
record or vector begins with the three new prefixes.

## VERDICT: 0C/6I/10M/3N

---

## Findings

### I-1 — The plan implements auto-append option (a); the ruling that landed is (c)-narrowed, and the plan has no `--now`

**Plan:** line 19 (Open question), line 27 (Global Constraints), Task 4 step 3 (lines
1081-1104), Task 4 step 4 (line 1148), Task 4 tests (lines 985-1005).

**What landed:** commit `7612066` (on `b05f3c4`), subject verbatim —
*"RULING (c) narrowed: auto-append now: only when a key:/hash: record is present; --now
opt-in; --no-now opt-out; supplied now: wins"*. `Nothing folded yet.`

**Constructed input, under the ruling:** `me sysw pack --no-passphrase --out p.bin
text:48656c6c6f2c20576f726c6421`. The payload holds no `key:`/`hash:` record, so nothing is
appended and no `appended now:` line prints.

**Plan's output vs required:** the plan's own test
`pack_appends_the_pack_time_when_no_now_record_is_given_and_says_so` (lines 986-995) asserts
`err.contains("appended now:")` and `s.contains("public record 1: pack time (now:)")` on
exactly that invocation — it asserts the behaviour the ruling forbids. Two further
consequences: `no_now_suppresses_the_auto_append_...` (997-1005) becomes vacuous (nothing
would have been appended anyway), and the `--now` opt-in flag the ruling requires is defined
nowhere in the plan — only `--no-now` is (lines 1068-1077). Task 4 step 4's instruction to add
`--no-now` to six named pre-existing tests also becomes wrong: none of those six packs a
`key:` or `hash:` record, so none of them changes at all under the ruling.

**Severity: Important** (a false claim an implementer would act on; the plan's stated option is
superseded).

**Remedy:** fold the ruling into line 19, line 27, Task 4 step 3's `if`, Task 4 step 4 and the
two CLI tests, and add the `--now` flag; the per-option table below says exactly what moves.

---

### I-2 — Default auto-append breaks `SPEC_descriptor_input.md`'s "One descriptor, one container, one invocation", and the plan's remedy hides it

**Plan:** Task 4 step 3 (lines 1081-1104) and step 4 (line 1148, which names
`descriptor_as::item_1_every_format_packs_one_descriptor_record` and
`item_2_...` among the six tests that "gain `--no-now`").

**Spec clause, `SPEC_descriptor_input.md:897-900`:**
> "One descriptor, one container, one invocation. Packing a descriptor TOGETHER with other
> records (the Engrave Bundle case — one container carrying `Descr` plus `MDMK`) is
> deliberately out of this cycle: it needs its own flag design and is filed as **F-414**
> rather than half-specified here."

and its §11 acceptance item 1 (`SPEC_descriptor_input.md:2100-2101`): *"The host half of this
item — **one record per format**, classifying `Descriptor` …"*.

**Constructed input:** `me sysw pack --no-passphrase --as descriptor --in desc.txt --out c.bin`.

**Plan's output vs required:** under Task 4 step 3 the container holds TWO public records —
the canonical descriptor and an auto-appended `now:` — so `payload.public.len() == 2`. The
shipped assertion is `crates/me-cli/tests/descriptor_as.rs:349-355`,
`assert_eq!(payload.public.len(), 1, "§5.2's record is ONE record")`. The plan's fix is to add
`--no-now` to that invocation, which preserves the assertion while moving it OFF the default
path — so after the fold nothing measures §11 item 1 for the command an operator actually
types. Downstream: the `now:` record is admitted at Wallet Policy alone
(`SPEC_systemwide_payloads.md:362-372`), so the same container packed for Engrave Bundle now
carries a record that program refuses by name.

**Severity: Important** (pre-existing spec-stated behaviour changed without the plan saying so).

**Remedy:** under the landed ruling this vanishes (a descriptor-only payload holds no
`key:`/`hash:`); if the operator ever reverts to (a) or to the plan's WIDE (c) — which lists
"a descriptor" as composer-relevant — the plan must instead exempt `--as` or reopen F-414.

---

### I-3 — The single-`now:` refusal fires only inside `split`, i.e. AFTER the passphrase ceremony — the exact defect F-246 hoisted `admit_check` out of `split` to prevent

**Plan:** Task 2 step 3 item 5 (lines 666-675) puts the rule in `split`, and the CLI (Task 4)
adds no pre-ceremony check.

**Spec clause, §6a (spec lines 292-294):**
> "The payload-wide rule 'at most ONE `now:` record' is enforced at the two sites that see the
> whole payload: host `pack_with` and device `syswSession.load`"

— which the plan satisfies literally. What it does not satisfy is the CLI's own documented
invariant, `crates/me-cli/src/main.rs:1552-1562`:
> "F-246 — ADMISSION BEFORE THE CEREMONY. `pack_with` rejects an unplaceable record, but it
> runs AFTER the passphrase has been generated, printed, and captioned 'write this down and
> store it APART from the machine'. The operator who obeys that instruction is left holding
> twelve words that protect no artifact, immediately above an error saying the run failed."

**Constructed input** (a payload with a secret, so `decide_sealing` at `main.rs:1655` returns
true and the ceremony runs), records file `f`:
```
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
now:31373536363834383030
now:31373536363834383031
```
`me sysw pack --in f --out p.bin`

**Plan's output vs required:** `admit_check` at `main.rs:1563` passes (all three records
classify). The passphrase is generated and printed with the "write this down" caption at
`main.rs:1676-1682`. Only then does `pack_with` at `main.rs:1701` reach `split`
(`sysw/mod.rs:431`) and return `SyswError::SecondNow(2)`, printing
`me: record 2: a second now: record; only one is allowed. Remove one.` at exit 4. Required by
the invariant the file states: refused before the ceremony, like every other admission
failure.

**Severity: Important.**

**Remedy:** add a `composer_records::now_indices` check beside `sysw::admit_check` in the
CLI's Pack arm (keeping the `split` check as the library backstop), and say so in Task 4.

---

### I-4 — The new `sysw_error` arm breaks the stated invariant that every unclassifiable arm opens `record N (records count from 0)`, and the plan's Global Constraints claim the opposite of what its own fragment does

**Plan:** line 28 claims —
> "the plan keeps the house style prefix and the §8n wording after it, so the spec's line is a
> SUBSTRING of what prints"

but the fragment it prescribes (Task 2 step 6, lines 682-689) emits `e.line(*i)` first and no
house-style prefix at all: `record 1: key: needs [fingerprint/path]xpub … (detail). Build the
record with …`.

**The invariant it breaks, `crates/me-cli/tests/descriptor_seam.rs:846-855`:**
> "**The shipped record-classification refusal, recognised by its own vocabulary.** Every arm
> of `sysw_error`'s unclassifiable case — the reserved prefixes, the unparseable transaction,
> the unsigned inputs, the BIP-93 profile miss, the catch-all — opens `record N (records count
> from 0)`, and nothing else `me` prints does. Matching one arm's tail instead was the first
> version of this helper and it reported `unclassified` for five of the six hostile-payload
> rows: invariant 1 is about the SURFACE, not one message."

`fn is_record_refusal(err) { err.contains("(records count from 0)") }`.

**Constructed input:** `me sysw pack --no-passphrase key:7870756236446b…` (the bare-xpub
`key:` record, plan line 1032). Printed: `me: record 0: key: needs [fingerprint/path]xpub with
an origin; a bare xpub is not a key record (no [origin]: a bare xpub). Build the record …` —
which contains no `(records count from 0)`, so `classify_run` would answer `"unclassified"`
for it. No test goes red today only because `descriptor_seam`'s corpus feeds descriptors, not
composer records; the invariant is silently false from the moment Task 2 lands. (`E::SecondNow`
is outside the `Unclassifiable` family, so it never claimed the invariant — but it is likewise
unrecognised.)

**Severity: Important** (pre-existing behaviour/invariant changed without the plan saying so,
plus a plan statement that contradicts its own code).

**Remedy:** either carry `(records count from 0)` in the new arm too, or amend
`is_record_refusal` and its doc comment in the same commit — the plan must pick one and name it.

*Verified separately and PASSING:* the four §8n blockquotes, unwrapped and with `N`→index, are
byte-exact substrings of what the plan prints — machine-compared, 4/4 MATCH.

---

### I-5 — The §12 item 8 lockstep fixture omits several §6a malformations and the valid depth-3 shape; a wrong Go port passes all 27 rows

**Plan:** Task 3 `CASES`, lines 853-881, and the coverage test at lines 772-793.

**Spec clause, §12 item 8 (spec lines 898-902):**
> "**Record classes, lockstep.** A cross-language vector set: each `key:`, `hash:`, `now:`
> record (valid and **each §6a malformation**) classifies identically on the host and on the
> device …"

**Malformations and valid shapes §6a names that have NO row** (I ran every one of these through
the plan's own `parse`, so each is a real, distinct verdict the fixture does not pin):

| missing row | plan's verdict when I ran it |
| --- | --- |
| valid `key:` at **depth 3** with a 3-component origin (§6a: "an xpub at depth **3 or 4**") | `ADMITTED Key depth=3` |
| `key:` xpub at depth 2 | `REFUSED "xpub depth is not 3 or 4"` |
| `key:` xpub at depth 5 | `REFUSED "xpub depth is not 3 or 4"` |
| `key:` **uppercase fingerprint** `[73C5DA0A/48'/0'/0'/2']…` | `REFUSED "fingerprint is not 8 lowercase hex characters"` |
| `key:` fingerprint of 7 hex characters | `REFUSED` |
| `key:` `[73c5da0a]xpub…` — fingerprint, no `/path` | `REFUSED "origin has no path"` |
| `key:` origin LONGER than depth (only "shorter" has a row) | `REFUSED` |
| `key:` body not UTF-8 (`now:` has such a row, `key:` does not) | `REFUSED "body is not UTF-8"` |
| `key:` unterminated `[origin` | `REFUSED` |
| a **testnet `tpub`** `key:` record | `ADMITTED Key` |
| `now:` body in UPPERCASE hex (§6a: "Uppercase hex anywhere is not valid hex") | `REFUSED` |

**Why this is not cosmetic:** the fork's own `bip380.ParseKey` reads the fingerprint with
`hex.DecodeString(originAndPath[:8])` (`third_party/seedhammer/bip380/bip380.go:379`), and
Go's `encoding/hex` is case-INSENSITIVE — so an uppercase fingerprint is precisely where the
Go classifier will diverge, and no row exists to catch it. Likewise a Go port that ignores
depth entirely, or that hard-codes depth 4, or that rejects `tpub`, passes all 27 rows. The
plan's own coverage gate (`the_fixture_covers_every_class_and_every_8n_line_at_least_twice`)
cannot see any of this: it counts classes and §8n lines, not malformations.

**Severity: Important** (a Stage-1-scope spec requirement not fully implemented).

**Remedy:** add one row per line of the table above (each is a one-line `Case`), and re-pin
`FIXTURE_SHA256`.

*Verified separately and PASSING:* all 27 existing rows agree with the plan's own parser —
class and `host_line` — with zero mismatches; `CASES.len() == 27`, matching the Task 3 commit
message.

---

### I-6 — Task 5 leaves the module doc of the very file it edits stating that no taproot template exists

**Plan:** Task 5 step 1 (lines 1185-1224) replaces only the
`an_unregistered_script_type_is_refused` test; step 3 (lines 1235-1274) names two `derive.rs`
doc comments. Nothing names the test file's module doc.

**The sentence left behind, `mnemonic-secret/crates/ms-cli/tests/cli_derive_bip48.rs:18-21`:**
> "and it registers exactly two script types — `1'` nested segwit (p2sh-p2wsh) and `2'` native
> segwit (p2wsh), the latter being the recommended default. **There is no registered Taproot
> value, so none is offered here; inventing one would put funds at a path no other wallet
> looks at.**"

After Task 5 the file's first test derives at `m/48'/0'/0'/3'`, so the second sentence is false
in both halves ("none is offered here"; "no other wallet looks at" — the plan's own replacement
text cites Coldcard `bip48_3` and Liana `p2tr_deriv`). This is the same claim the follow-up
`ms-derive-taproot-justifications-stale` was filed against, at a third site the follow-up does
not list (it names `derive.rs:102-105` and `:126-133` only).

**Severity: Important** (a false claim an implementer would leave standing, in the file the
task rewrites, and it is the justification a future reader will trust).

**Remedy:** add the module doc (lines 14-21) to Task 5 step 1's edit list.

---

### M-1 — Task 2 step 8 sends the implementer to a `match` in `expect.rs` that does not exist

**Plan line 706:** *"`crates/me-cli/src/sysw/expect.rs` (the `Kind` mapping: none of the three
is a `--expect` kind, so they fall to the arm the `Address` variant uses — read the match and
mirror `Address`)"*. Measured: `expect.rs` contains no `match` on `Class` at all — it uses
`==` and `matches!` (`expect.rs:122, 129, 131, 132`), and `Class::Address` appears in that file
only inside a doc comment (`expect.rs:33`). The repo has exactly TWO exhaustive matches on
`Class`: `main.rs:2300` (`class_name`) and `record_corpus.rs:66` (`class_name`), both of which
the plan already names correctly. The File Structure table (line 53) and the Task 2 commit
(line 719) both stage `expect.rs` for a change that has no content.

**Remedy:** drop `expect.rs` from step 8, the table and the commit.

### M-2 — Two drifted citations in Task 4 step 3

Plan line 1106: *"`print_mdmk_confirmation` (`crates/me-cli/src/main.rs:2060`) … calls
`print_mt_confirmation(&records); print_descriptor_confirmation(&records);` (`:2117-2118`)"*.
Measured at baseline `b44fb61`: the function is at `main.rs:2063` (2060 is a doc-comment line —
`plan-cite-check.sh` printed it as "ok", the F-279 shape its own banner warns about) and the
two calls are at `:2088-2089`, not `:2117-2118` (that range is inside
`print_descriptor_confirmation`).

### M-3 — Task 5 states two different `script_type_label` strings, and the label is unreachable

Line 29 and line 1172 say the label is `"3' p2tr (taproot multisig)"`; step 3 item 3
(line 1263) says `"3' p2tr (taproot multisig; Coldcard/Liana convention)"`. Separately, the
label is printed only under `args.template.filter(|t| t.script_type_defaulted())`
(`derive.rs:504-518`) and `script_type_defaulted` is `matches!(self, Template::Bip48)` — so for
`Bip48P2tr` the label can never print. The Interfaces line's *"labelled …"* is therefore an
unobservable claim.

### M-4 — `show`'s `now:` line asserts provenance `show` cannot know

Plan lines 1128-1137 print `pack time (now:) — {when} (the pack time; a lower bound the device
echoes, never a locktime)` for EVERY `now:` record. §6a (spec lines 294-296) says *"an
operator-supplied `now:` wins silently and pins a deliberate bound"* — for that record the
parenthetical "the pack time" is false, and `show` has no way to tell the two apart. ("never a
locktime" is correct: §6a, *"the record affects ONLY echoes and refusals (§6b), never an
encoded operand"*.) The line also never mentions that the height field bounds heights.

### M-5 — The fixture has no 31-byte `hash:` row, which §12 item 4 names explicitly

§12 item 4 (spec line 873) requires *"a `hash:` of 31 and 33 bytes"*. `CASES`' `hash-63-chars`
(line 864) is a 63-CHARACTER body — an odd length, which fails on length and on hex parity
before any 32-byte rule is reached — and `hash-66-chars` is 33 bytes. A well-formed 62-hex
(31-byte) digest has no row. The unit test (`"a8".repeat(31)`, plan line 189) and the CLI test
(line 1036) both do cover 31 bytes, so this is a fixture gap only.

### M-6 — Task 6 is already executed and closed, but reads as to-do

Task 6 (lines 1309-1346) is `- [ ]` throughout. The fold landed at `12e0659` and closed under
its own R0 at `fdf7671`, both ancestors of HEAD: `SPEC_systemwide_payloads.md` already carries
the three `ClassKey`/`ClassHash`/`ClassNow` rows (`:337-339`), the CREATED Wallet Policy row
with all ten cells (`:371`) and the three prefixes in §5.3 (`:622-624, :654-655`) — matching
the plan's prescription, including the `progTransaction` note (`:406-410`). A plan handed to an
implementer should say so.

### M-7 — Sentences elsewhere the plan would falsify and does not name

(Grouped; details in the section below.) `SPEC_sh2_sysw_consumption.md:265` N9;
`main.rs:2693-2698`'s `U::Unrecognised` message; `SyswCmd::Pack`'s `--help` doc
(`main.rs:183-206`), which never mentions the three prefixed forms this whole stage adds;
`record.rs:25-31`'s reserved-prefix doc and `decode_body`'s three-prefix strip.

### M-8 — Staged plan S1's Exit and the plan's hand-off disagree about vendoring

`STAGED_PLAN_wallet_policy_composer.md:97` — *"**Exit:** `me` and `ms` published; the lockstep
fixture vendored into the fork."* The plan's Task 7 step 4 (line 1390) instead ends the stage
before vendoring: *"Stage 2 begins only then, and it vendors `record_class_vectors.json` with
its pinned sha256."* One of the two must move.

### M-9 — `48H` (uppercase hardened marker) is refused; §6a says "MUST parse as BIP-380 key-origin notation"

Measured: `[73c5da0a/48H/0H/0H/2H]xpub…` → `REFUSED "path does not parse"` (rust-bitcoin
`DerivationPath::from_str` returns `InvalidChildNumberFormat` for `H`; `48h`/`48'` both parse).
BIP-380 is generally read as admitting `h`, `H` and `'`. **No lockstep break:** the fork's
`bip32.ParsePathElement` (`third_party/seedhammer/bip32/bip32.go:71`) also accepts only `h` and
`'`, so host and device agree. Recorded because the decision is invisible — there is no fixture
row and no sentence saying `H` is out of scope. (I did not verify BIP-380's text directly.)

### M-10 — Task 5's JSON hint points at a test that does not parse JSON

Plan line 1226: *"the existing `json_carries_the_assumption_flag` test shows how this file
parses JSON — follow it."* That test (`cli_derive_bip48.rs:270-295`) does substring matching on
`"\"script_type_defaulted\":true"`; it never constructs a `serde_json::Value`. The dependency
question the same sentence hedges is settled: `serde_json = "1"` is a regular dependency of
`ms-cli` (`crates/ms-cli/Cargo.toml`), and Cargo puts regular dependencies in scope for
integration tests, so the new test compiles. The keys it asserts all exist
(`derive.rs:485-487`).

### N-1 — Global Constraints line 25 lists a rule the code does not implement

*"`key:` body … path components hardened with `'` or `h`"*. Neither §6a nor `parse_key`
requires hardening — the plan's own test comment (line 172-174) says unhardened components are
a legal path, refused only when they mismatch the xpub's child number. An implementer reading
the constraints as the rule list could add a hardening check and refuse a record §6a admits.

### N-2 — Two of Task 5's three arms are not compiler-enforced

`script_type` ends `_ => None` (`derive.rs:174`) and `script_type_label` ends `_ => ""`
(`derive.rs:197`), so omitting `Template::Bip48P2tr` from either compiles cleanly and silently
derives `m/48'/0'/0'` at depth 3. Only `purpose()` is exhaustive. The new test's
`assert!(s.contains("m/48'/0'/0'/3'"))` catches it; the plan should say the arms are not
compiler-checked so the implementer does not rely on the build.

### N-3 — Task 5 step 4's snapshot hedge is a no-op (recorded so nobody hunts)

*"If `gui_schema`/`gen_man` snapshot tests enumerate template values and now differ, regenerate
them"*. Measured: neither does. `gen_man.rs` derives its expected page set from the
`gui-schema` SUBCOMMAND inventory; `gui_schema_emits_spec_v7_json.rs` asserts on `encode
--language`'s dropdown choices only. Neither file contains the string `template`. Adding
`Bip48P2tr` changes neither, and `the_single_sig_template_names_are_unchanged`
(`cli_derive_bip48.rs:186-191`) and `bg002h_wsh_is_not_labelled_as_nested_segwit` (`:356-381`)
are both unaffected — the latter asserts the absence of `p2sh-p2wsh` in `bg002h-wsh`'s output,
which no new arm touches.

---

## Under each auto-append option

The ruling at `7612066` selected **(c)-narrowed**, which is NOT the plan's own (c). Recorded
for all four so the fold is mechanical.

| | (a) keep default (what the plan implements) | (b) opt-in `--now` | (c) plan's WIDE conditional (`key:`, `hash:`, a descriptor **or an md1/mk1 card**) | **(c)-narrowed — the landed ruling** (`key:`/`hash:` only, `--now` opt-in, `--no-now` opt-out) |
| --- | --- | --- | --- | --- |
| Task 4 step 3 `if` | as written | `if *now && …` | `if !*no_now && payload_is_composer_relevant(&recs) && …` | `if (*now \|\| has_key_or_hash(&recs)) && !*no_now && now_indices(&recs).is_empty()` |
| `--now` flag | not needed | **REQUIRED — the plan defines none** | not needed | **REQUIRED — the plan defines none** |
| Task 4 step 4's six pre-existing tests | correct as written | all six become no-ops; the `--no-now` edits must be dropped | `descriptor_as::item_1`/`item_2` still need `--no-now` (a descriptor is composer-relevant); the four `sysw_cli` ones do not | **all six become no-ops; drop every `--no-now` edit and the `git add … tests/sysw_cli.rs` on line 1156** |
| `pack_appends_the_pack_time_…` (lines 986-995) | passes | **FAILS** — must pass `--now` | **FAILS** — a `text:`-only payload appends nothing; must pack a `key:`/`hash:` | **FAILS** — must pack a `key:` or `hash:` record |
| `no_now_suppresses_…` (997-1005) | passes | vacuous | vacuous | **vacuous** — must pack a `key:`/`hash:` for the flag to suppress anything |
| I-2 (descriptor spec conflict) | **live** | resolved | **STILL LIVE** — the plan's (c) lists a descriptor | **resolved** |
| Line 19's own sentence | correct | must be rewritten | must be rewritten | must be rewritten (it recommends the WIDE (c)) |
| Line 27 Global Constraint | correct | must be rewritten | must be rewritten | must be rewritten |
| Spec §6a lines 294-296, 308-311 and §10 item 2 (spec 815-819) | unchanged | need a fold | need a fold | **need a fold** — they state the default unconditionally |
| I-1, I-3, I-4, I-5, I-6, M-1..M-10, N-1..N-3 | unaffected by the choice | " | " | " |

All other findings are option-independent.

## What the plan would make false elsewhere

1. **`design/SPEC_sh2_sysw_consumption.md:265`** — *"**N9. Reserved prefixes fail closed.**
   `text:`, `pass:` and `tx:` require **lowercase** hex bodies; a body that is not is `Unknown`
   and refused …"*. After Task 2 there are six reserved prefixes obeying that rule. The plan
   folds the payload spec (Task 6) and this one not at all.
2. **`crates/me-cli/src/main.rs:2693-2698`**, `U::Unrecognised`'s message — *"not a
   `text:`/`pass:`/`tx:` record. Addresses are not classifiable here …"*. It is the message an
   operator sees when they hold a record `me` cannot place, and it now omits half the placeable
   prefixed forms. (It is not *reachable* for a `key:`/`hash:`/`now:` record — `unknown_reason`
   returns `Composer` first — so this is misleading, not wrong-arm.)
3. **`crates/me-cli/src/main.rs:183-206`**, `SyswCmd::Pack`'s help — *"A record is a BIP-39
   mnemonic, an md1/mk1/ms1/mt1 string, or one of the prefixed forms … `text:` … `pass:` …
   `tx:` …"* plus *"The prefixes are RESERVED (spec §5.3.1)"*. The plan adds three record
   classes and three refusal lines and never touches `-h`/`--help`, so the stage's entire input
   surface is undiscoverable from the tool. (No help-snapshot test exists in `me-cli/tests`, so
   nothing goes red.)
4. **`crates/me-cli/src/sysw/record.rs:25-31`** — the module doc's reserved-prefix paragraph and
   the three `pub const … PREFIX` declarations beside it, and `decode_body` (`:135-142`), which
   strips exactly `text:`/`pass:`/`tx:`. After Task 2, `record.rs` owns `Class::{Key,Hash,Now}`
   while their prefixes and rules live in `composer_records.rs`. **No behavioural defect:** I
   traced every `decode_body`/`decode_text` call site (`mod.rs:152, 219, 233, 240`;
   `main.rs:2029, 2153`; `record.rs:146`) and each is guarded by its own prefix test, so no
   caller ever hands it a `key:` record. The plan's own module doc (lines 258-261) explains the
   duplication; what it does not do is add the one sentence to `record.rs` that stops a future
   reader reading its prefix list as exhaustive.
5. **`design/SPEC_descriptor_input.md:118-121`** quotes the `U::Unrecognised` text in a run
   transcript. Already historical by design (it predates S2's descriptor arm); noted only so a
   later grep for the string does not read as a live claim.
6. **Not falsified — checked and clean:** `sysw/coverage.rs` (its `COVERAGE` table maps
   `SPEC_sh2_sysw_consumption` §8.3's 23 ids; none concerns record classes, and no id changes);
   `crates/me-cli/testdata/sysw_vectors.json` (generated by `sysw::vectors` through
   `pack_deterministic`, `vectors.rs:70` — the library appends nothing, so Task 7 step 2's claim
   holds); `record_corpus.rs`'s capture (I scanned every JSON fixture and every quoted record in
   `src`/`tests`/`testdata`: **zero** begin `key:`, `hash:` or `now:`, so nothing reclassifies);
   `Class::is_argv_forbidden` (`record.rs:106-108` is `is_secret() || is_bearer()`, and the plan
   adds neither, so a `key:` record on argv is correctly allowed — `argv_secret_guard`'s
   "**five** classes" count at `main.rs:510` also stays true); §6a's own citation of
   `pack_with` at `mod.rs:288` and `admit_check` at `:416` (both exact).

## Attacks tried that found nothing

- **Can a sniffer claim a composer record first?** No. `classify_with`'s arms above the
  insertion point are `starts_with` on `tx:`/`pass:`/`text:`, none a prefix of the three; the
  BIP-39, `mt1`, codex32 and `descriptor::host_admits` arms all sit below it. The same holds in
  `unknown_reason` (the composer check goes after the `TX_PREFIX` block, before the
  `PASS`/`TEXT` loop and before `bip93_outside_the_profile`).
- **Can the auto-append produce the second `now:` itself?** No — it is gated on
  `now_indices(&recs).is_empty()`, and `now_indices` counts only records that parse
  `Some(Ok(Now))`.
- **Can a malformed `now:` plus a valid one slip through as "one"?** No — the malformed one is
  `Class::Unknown` and `admit_check` refuses it (at `main.rs:1563`, before `split` is reached).
  I ran the mixed case: `now_indices(["text:…", now(valid), "now:zz", now(valid)]) == [1, 3]`.
- **Does `SecondNow` name the right index?** Yes: `[now, text, now] → SecondNow(2)` and
  `[now, now] → SecondNow(1)`, both from `nows[1]`.
- **Do the §8n lines print verbatim?** Yes — 4/4 byte-exact after unwrapping the blockquotes
  and substituting `N`; the house-style question is I-4, not a text mismatch.
- **`now:` leading zeros** (`0100`, `0000000001`, `1756684800,0910000`): all ADMITTED, and §6a's
  own regex `^[0-9]{1,10}(,[0-9]{1,9})?$` admits them, so the plan is compliant. Go's
  `strconv.ParseUint` would agree, so no lockstep hazard.
- **`now:` boundaries:** `0` refused, `1` admitted, `2147483647` admitted, `2147483648` refused,
  `12345678901` refused, `,910000` / `1756684800,` / `,` / `1756684800,,1` / `+1756684800` /
  `1756684800.0` / leading and trailing space all refused, height `0` refused, `499999999`
  admitted, `500000000` refused. Every §6a band boundary is right in both directions.
- **`hash:` boundaries:** 62, 63, 64-lower, 64-upper, 66 and empty all behave per §6a.
- **`key:` structural attacks:** trailing junk after the xpub, a doubled `]`, `xprv` in place of
  `xpub`, a slip132 `zpub`, `[fp/]`, `[fp/m/48'/…]`, whitespace padding, odd-length body,
  uppercase body — all refused, none panics. `origin.as_ref().last().expect("non-empty")` is
  guarded by the `origin.is_empty()` check two lines above it, and `m/` parses to an EMPTY path
  (measured) rather than an error, so the guard is load-bearing and present.
- **`Task 5` oracle:** all four xpubs re-derived byte-for-byte by a third implementation
  (rust-bitcoin bip32 over the BIP-39 seed of abandon×11+about), master fingerprint `73c5da0a`
  confirmed, and `m/48'/0'/0'/2'` and `m/48'/0'/1'/2'` reproduce the file's shipped
  `P2WSH_ACCT0`/`P2WSH_ACCT1` — so the helper reproduces published values as well as new ones.
- **`md admits depth 3 or 4`** (Task 5's rewritten comment): true —
  `descriptor-mnemonic/crates/md-cli/src/parse/keys.rs:130`, `if !matches!(depth, 3 | 4)`.
- **Plan gates:** `plan-table-check` 17 rows / 0 malformed; `plan-glyph-check` 94 strings / 0
  undrawable; `plan-staleness-check` against `b44fb61` 0 drifted (it checks bytes, which is why
  M-2 is invisible to it); `plan-cite-check` 3/3 resolved (M-2 is one of the three, printed as
  "ok" on the wrong line).

## What I ran

```
git log/diff/merge-base on mnemonic-engrave (46fc91b, b05f3c4, b44fb61, fdf7671, 12e0659)
python3  — extract the plan's rust blocks; unwrap and byte-compare the four §8n blockquotes;
           scan every me-cli JSON fixture for records starting with the three new prefixes
cargo build/run --offline  (scratchpad crate `probe`, bitcoin 0.32, edition 2021):
  bin probe   — parse() over the plan's own edge set + ~35 constructed inputs; CASES self-check
  bin derive  — real depth-0/2/3/4/5 xpubs and tpubs; DerivationPath::from_str spellings
  bin edge    — depth/network/origin-length/H-marker/fingerprint cases against parse()
  bin oracle  — the four Task 5 xpubs re-derived from the BIP-39 seed of abandon x11 + about
./scripts/plan-cite-check.sh   design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md
./scripts/plan-table-check.sh  design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md
./scripts/plan-glyph-check.sh  design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md
./scripts/plan-staleness-check.sh <plan> . b44fb61
reads: crates/me-cli/src/{main.rs, sysw/{mod,record,expect,coverage,advice,vectors}.rs},
       crates/me-cli/tests/{descriptor_as,descriptor_seam,record_corpus,sysw_cli}.rs,
       design/{SPEC_wallet_policy_composer,SPEC_systemwide_payloads,SPEC_descriptor_input,
               SPEC_sh2_sysw_consumption,STAGED_PLAN_wallet_policy_composer}.md,
       mnemonic-secret/crates/ms-cli/{Cargo.toml, src/cmd/derive.rs,
               tests/{cli_derive_bip48,gen_man,gui_schema_emits_spec_v7_json}.rs},
       mnemonic-secret/{CHANGELOG.md, design/FOLLOWUPS.md},
       descriptor-mnemonic/crates/md-cli/src/parse/keys.rs,
       third_party/seedhammer/{bip380/bip380.go, bip32/bip32.go}
```

No `.jsonl` file was read. Nothing was committed in any repository; the only file this review
wrote is this report.
