# R0 round 18 — VERIFICATION of the r17 fold

**Target:** `design/SPEC_descriptor_input.md` at `7bc5e8f` ("spec: fold R0 r17 --
disjunct precision, all fifteen rows covered"). `7bc5e8f` is HEAD; tree clean
(`git status --short` empty).
**Source of truth for what was required:**
`design/agent-reports/R0-descriptor-input-spec-r17.md` (0C/1I/2M/1N).
**Scope, as briefed:** (1) verify ONLY r17's four edits; (2) re-derive the gate
column against a complete adversarial input set — every cell one derivable
answer, gate verdict → refusal class → §6 text → exit code, no false vocabulary;
(3) diff containment plus the two standing sweeps. Everything from r1–r17 —
including §4.5's fifteen-row table and r15's 60-cell decision table — taken as
settled. **No fresh audit was performed.**
**Reviewer:** independent context, opus tier. Read-only; nothing modified,
committed or pushed.
**Diff read in full:** `git diff 6a12beb..7bc5e8f`. The fold `7bc5e8f` touches
**one file, 2 hunks, +18/−12**; the range's other commit `7cb3daf` is the r17
report's own persist (+355, report only). Persist and fold are two commits, in
that order.
**Binaries used:** `/home/bcg/.cargo/bin/me` (`me 0.7.0`), `cargo-nextest`
0.9.140, `python3` for the sweeps and the base58check constructions.

---

## Counts — NEW findings

| severity | count |
| --- | :-: |
| **Critical** | **0** |
| **Important** | **2** |
| Minor | 2 |
| Nit | 1 |

**The spec does NOT re-close GREEN this round.** All four of r17's edits land at
the sites they name and the diff is contained. But the adversarial re-derivation
finds the same failure mode a fourth consecutive time, now on the *other* two
disjuncts: the `(`-test was narrowed from the whole input to **the input's first
token** while the `": "`-test next to it is scoped **per line**, and that
asymmetry silently falsifies a cell of the settled 60-cell table — r15's class-10
exemplar (`mnemonic + descriptor, two lines`, mnemonic FIRST) no longer passes
the gate at all. Separately, r17's own instruction that any repair to the token
test *"should be mirrored in §6 step 4, or the two diverge again"* was not
carried out, so the two normative shape tests now differ while §5.1 still claims
they are aligned.

---

# Disposition of r17's four edits

| edit | verdict | evidence |
| --- | --- | --- |
| **1. the token test keys on the leading segment before any `/` (all fifteen §4.5 rows)** | **LANDS in §5.1; NOT propagated to §6 step 4 — new-I2** | §5.1 L818–821 now reads *"a single token that begins with `[`, OR whose leading segment before any `/` is a 78-byte base58check payload"*. I re-derived all fifteen rows (table below): **15/15 now fire T3**, including row 13 (`xpub…/<0;1>/*`) which failed all four tests at `6a12beb`. The generalisation r17 filed is also closed: **all five** members of §4.7 conjunct 7's use-site set (`absent`, `/*`, `/i/*`, `<i;i+1>`, `<i;i+1>/*`) leave a leading `xpub…` segment, so all five fire. The claim *"all fifteen §4.5 rows"* is TRUE — row count measured mechanically off the table header, **15**, not hand-counted. **But** `grep -n '78 bytes\|78-byte\|base58check'` returns exactly two sites — L819 (the gate) and **L1230 (§6 step 4, unchanged)** — and r17's finding required both. |
| **2. the parenthesis test requires identifier-then-`(`** | **LANDS for the case it was filed for; introduces new-I1** | §5.1 L812–814 now reads *"an input whose first token is an identifier immediately followed by `(` — a script expression, not any parenthesis"*, with r17's own exemplar inline. r17's new-M3 is closed: `text: my wallet (2 of 3)` and `pass: hunter (2)` both stay records (measured below, rc=4, RESERVED-prefix text preserved). The narrowing is from *the whole input* to *the input's first token* — two axes at once, and the second one is not what the finding asked for. See new-I1. |
| **3. the naming/describing distinction** | **FIXED at the site r17 named; the absolute is still contradicted elsewhere — new-M1** | §5.3 L1041–1044 now reads *"No refusal names a flag that refuses in the current build; a neither-path refusal may DESCRIBE a different re-export's future path — describing routes nothing"*. Checked against the one exempted text (§6 L1273's `multi`-form replacement, incorporated by reference at L1291): its tail *"(Re-exporting as a `sortedmulti` policy … needs the scannable-plate path.)"* **is** a different re-export's future path, so the new clause selects exactly that site and the contradiction r17 filed is gone there. The distinction is also *sound*: walk W11's ban exists to stop a refusal ROUTING the operator at a dead flag, and a caveat warning them off a re-export routes nothing. |
| **4. the record-not-mnemonic citation** | **FIXED** | §5.1 L823–826 now reads *"pinned by a green test of the record-refusal surface (`sysw_cli.rs:1928`): a mistyped RECORD must never hear descriptor vocabulary"*. `grep -n` puts `fn an_unpackable_record_is_refused_before_a_passphrase_is_minted` at **line 1928** exactly; its operand is the literal `"this is not a record of any class"` and it asserts `err.contains("not a form this container can place")` — so it is a genuine witness for the record-refusal surface, and the sentence now describes what it actually pins. A control test sits directly below it (`an_admissible_record_still_gets_its_passphrase_ceremony`, asserts the ceremony DOES fire for an admissible record), so the pin is not a false PASS. Re-run: `cargo nextest run --locked -E 'test(an_unpackable_record_is_refused_before_a_passphrase_is_minted)'` → `PASS [0.003s]`, 1 passed, 440 skipped. **The broadened claim was checked, not assumed:** every record class the message names is gate-excluded — a 12/24-word mnemonic is multi-token with no `(` and no `": "`; `text:`/`pass:`/`tx:` have a first token of `text:`/`pass:`/`tx:` and a `": "` key that is neither a BlueWallet header nor 8 hex; an md1/mk1/ms1/mt1 string is a single bech32 token that is not a base58check envelope. |

**Four of four land at the sites they name. One (edit 1) is half-propagated, and
one (edit 2) over-narrows.**

---

# The gate, re-derived against the adversarial set

The four tests, verbatim from §5.1 L812–822, as evaluated:

> **T1** an input whose **first token** is an identifier immediately followed by `(`;
> **T2** a **line** whose `": "` key is a BlueWallet header (`Name`/`Policy`/`Derivation`/`Format`) or an 8-hex fingerprint;
> **T3** a single token that begins with `[`, OR whose leading segment before any `/` is a 78-byte base58check payload;
> **T4** JSON with a descriptor field.

## (a) §4.5's fifteen rows, `--as` omitted — 15/15, the r17 Important is closed

Row set enumerated mechanically (`| input | verdict | why |` header at L551,
**15** data rows — the tool's count).

| # | §4.5 row | verdict | test | then | exit |
| --: | --- | :-: | --- | --- | :-: |
| 1 | bare `xpub…` | ACCEPT | T3 (whole token is the leading segment) | parses ⇒ "--as decides" | 2 |
| 2 | bare `zpub…` | ACCEPT | T3 | parses ⇒ "--as decides" | 2 |
| 3 | bare `Zpub…` | REFUSE | T3 (any version) | §6 step 4 ⇒ bare `Zpub`/`Ypub` row | 3 |
| 4 | bare `Ypub…` | REFUSE | T3 | §6 step 4 ⇒ same row | 3 |
| 5 | `[…/44'/0'/0']xpub…` | ACCEPT | T3 (`[`) | "--as decides" | 2 |
| 6 | `[…/49'/0'/0']xpub…` | ACCEPT | T3 (`[`) | "--as decides" | 2 |
| 7 | `[…/84'/0'/0']zpub…` | ACCEPT | T3 (`[`) | "--as decides" | 2 |
| 8 | `[…/86'/0'/0']xpub…` | REFUSE | T3 (`[`) | §6 step 4 ⇒ path-matches-no-script row | 3 |
| 9 | `[…/48'/0'/0'/2']xpub…` | REFUSE | T3 (`[`) | §6 step 4 ⇒ same row | 3 |
| 10 | `[…/84'/0'/1']zpub…` | REFUSE | T3 (`[`) | §6 step 4 ⇒ account ≠ 0 row | 3 |
| 11 | `[…/84/0/0]zpub…` | REFUSE | T3 (`[`) | §6 step 4 ⇒ path-matches-no-script row | 3 |
| 12 | `[4bbaa801]xpub…` | REFUSE | T3 (`[`) | §6 step 4 ⇒ fingerprint-no-path row | 3 |
| **13** | **`xpub…/<0;1>/*`** | **ACCEPT** | **T3 (leading segment) — FIXED** | parses ⇒ "--as decides" | **2** |
| 14 | `xpub…\n` | (device REFUSE) | T3 after §4.6's trim | parses ⇒ "--as decides" | 2 |
| 15 | bare `tpub…` | ACCEPT at device, `me` REFUSES promotion | T3 | §6 step 4 ⇒ bare `tpub` row | 3 |

**15/15 descriptor-shaped. r17's new-I1 is closed at the gate**, and with it the
consequence it named: §11 item 5's acceptance can no longer be satisfied by a
witness that dodges the defect.

## (b) The four format happy paths

| format | fires | grounded on |
| --- | --- | --- |
| 1 — BlueWallet | **T2** on the `Name: sh` line | the fork's own fixture (`nonstandard/parse_test.go:33`) opens with three `#` comment lines; T2 is per-line so the comments do not defeat it. The `": "` separator is exact: `parseBlueWalletDescriptor` does `strings.SplitN(l, ": ", 2)` and errors otherwise (`parse.go:91–94`), so a `Name:sh` file is a parser REFUSE anyway and the gate loses nothing. The four header names are the `switch key` arms at `parse.go:103–124`; the cosigner arm is `hex.DecodeString(key)` — `5A0804E3` is 8 hex, uppercase, and T2's "8-hex" covers it |
| 2 — plain BIP-380 | **T1** (`wsh` + `(`) | first token of `wsh(sortedmulti(2,…))` |
| 3 — `{label, descriptor}` JSON | **T4** | the fixture at `parse_test.go:19–24` is pretty-printed with a `"descriptor"` field. Checked for a T2 **false** fire: its lines key on `"label"`, `"blockheight"`, `"descriptor"`, `"devices"` — none a header, none 8 hex |
| 4 — promoted bare key | **T3**, now for all five use-site spellings | (a) above |

## (c) Records with parentheses and colons in their payloads — measured, `me 0.7.0`

Every row an actual invocation; message truncated to its first line.

| input | rc today | shipped class | gate | outcome under the spec |
| --- | :-: | --- | :-: | --- |
| `text: my wallet (2 of 3)` | 4 | RESERVED-prefix, not lowercase hex | **excluded** (first token `text:`) | exit 4 preserved — **r17's new-M3 closed** |
| `pass: hunter (2)` | 4 | RESERVED-prefix | excluded | exit 4 preserved |
| `text: note: hello` (colon in payload) | 4 | RESERVED-prefix | excluded — key is `text` on either the first-`": "` or last-`": "` reading | exit 4 preserved |
| `seed: abandon abandon abandoz` | 4 | "not a form this container can place" | excluded | exit 4 preserved |
| `tx: zz` via `--in` | 4 | RESERVED-prefix | excluded | exit 4 preserved (on argv it never reaches classification: rc=3, bearer-on-argv guard) |
| a mistyped bare mnemonic | 4 | "not a form this container can place" | excluded | exit 4 preserved, pinned test green |
| malformed `md1yq…` | 4 | "not a form this container can place" | excluded (bech32 charset, not a base58check envelope) | exit 4, and the message names md1 — correct vocabulary |
| `deadbeef: xpub…` | 4 | "not a form this container can place" | **T2** | exit 3, headerless-BlueWallet branch-1 error — r17's intended flip |

## (d) Miniscript, and the edge tokens

| input | gate | §6 route | exit | note |
| --- | :-: | --- | :-: | --- |
| `wsh(or_d(pk(A),and_v(v:pkh(B),older(52560))))` | **T1** (`wsh`) | step 3 ⇒ miniscript row | 3 | the realistic spelling; unambiguous |
| `or_d(pk(A),and_v(…))` bare | **depends on the character class** | miniscript row, or exit-4 record refusal | 3 or 4 | `or_d` carries `_`; §5.1 never defines "identifier" — **new-M2** |
| `v:pkh(A)` bare | excluded on any reading (first identifier is followed by `:`) | record refusal | 4 | not a standalone descriptor; recorded, not filed |
| `[` alone | **T3** (`[`) | step 4 fires (first non-whitespace `[`), no branch-4 row matches ⇒ the unparseable-file row with branch 4's error | 3 | one derivable answer |
| 77-byte base58check token (constructed, valid checksum) | excluded (payload ≠ 78) | shipped record refusal | 4 | **consistent with §6 step 4, which states the identical 78-byte test** — recorded, not filed. Same answer for a one-character-mistyped xpub (checksum fails) |
| 78-byte base58check token, non-key version | **T3** | step 4 ⇒ branch 4 ("ANY version", stated) | 3 | consistent |
| `xpub…/` (trailing slash) | **T3** (leading segment) | §6 step 4 does **not** fire — not a single base58check token ⇒ step 5's generic four-forms text | 3 | the divergence, **new-I2** |
| `Zpub…/<0;1>/*`, `tpub…/<0;1>/*` | **T3** (leading segment) | step 4 does not fire ⇒ step 5's generic text instead of the bare-`Zpub`/bare-`tpub` rows | 3 | same divergence; the text stays true, the specific remedy is lost |
| a bitcoin address | excluded | shipped record refusal, which names addresses | 4 | §6's address row stays reachable with `--as` present (§11 item 4); recorded, not filed |
| well-formed `md1…` / `mk1…` | classification SUCCEEDS; the gate never runs | — | 0 | packs |

## (e) Multi-record inputs — where the first-token narrowing breaks

| input | gate (r16, `(` anywhere) | gate (r17, first token) | §6's multi-record row (L1292) |
| --- | :-: | :-: | --- |
| `wsh(sortedmulti(…))` then a mnemonic | T1 | **T1** | reachable — whole input does not parse, a record does ⇒ MREC, exit 4 |
| a mnemonic then `wsh(sortedmulti(…))` | T1 | **none** | **unreachable** — not descriptor-shaped ⇒ the shipped generic refusal, exit 4 |
| a mnemonic then `Name: sh …` | T2 | **T2** | reachable (T2 is per line) |

Measured at `7bc5e8f`, both orderings, `me 0.7.0`: both are rc=4 with the
status-quo text today (`record 1 …` and `record 0 … Descriptors and addresses
are not yet classifiable here`), which is exactly what §6's row exists to
replace.

---

# NEW findings

## new-I1 (Important) — T1 is scoped per INPUT while T2 is scoped per LINE; the multi-record row is lost whenever the descriptor is not record 0, and this falsifies a cell of the settled 60-cell table

**Where.** §5.1 L812–814 (*"an **input** whose first token is an identifier
immediately followed by `(`"*) against L814–815 (*"a **line** whose `": "` key
is…"*) — the two clauses are eleven words apart and scoped differently — and
against §6 L1292's multi-record row.

**What it costs.** §6 L1292 states its own applicability in full: *"Applies ONLY
when the whole input does not parse as one descriptor"*. There is no positional
precondition, and the quoted text is parameterised on the record index
(*"record `N` is a wallet descriptor"*). §5.1's gate now adds one that §6 does
not know about: unless the descriptor's own first token opens the **input**, the
input is not descriptor-shaped, the whole-input re-read never happens, and the
shipped record refusal fires instead.

**It is a regression against settled work, not a new class.** r15's decision
table (r15 report, "The 10 input classes") defines class 10 as *"multi-record
incl. a descriptor"* with exemplar **`mnemonic + descriptor, two lines`** —
the mnemonic first — and its `omitted` cells in **both** builds read
`— · MREC · 4 · L1265`. r16 re-checked the gate against that exemplar and
recorded it as safe because *"class 10 contains a `(`-bearing descriptor line"*
— true of r16's whole-input test, false of r17's first-token test. So the two
`class 10 × omitted` cells now hold two answers: MREC (§6 L1292's split-naming
text) by §6, or NOREC (the shipped generic refusal) by §5.1. Same exit code, two
different refusal classes and two different texts.

**The table is 58/60 single-valued, not 60/60.** r17's verdict — *"the decision
table stays at 60/60 — this round moved neither"* — is falsified by r17's own
fold.

**Why it is the worse half of the pair.** A plain BIP-380 descriptor is the
commonest export form, and "seed on one line, wallet on the next" is the
plausible operator file. That operator now hears *"not a BIP-39 mnemonic, not an
md1/mk1/ms1/mt1 string… Descriptors and addresses are not yet classifiable
here — see `sysw::classify`"* (measured above) — no mention of `--as`, no mention
that a descriptor was recognised, the exact status quo §1 and §2.1 exist to
remove. Meanwhile the same file with a BlueWallet header instead of a descriptor
line DOES get the good message, because T2 is per-line. Nothing in the spec
explains that difference, and nothing in §6 predicts it.

**Not prescribing the fix**, but recording what the derivation shows: the two
clauses differ only in scope, and scoping T1 to a **line** keeps r17's new-M3
closed for the same reason it is closed now — the line's first token in
`text: my wallet (2 of 3)` is `text:`. That was checked, not assumed. Whatever
the repair, §6 L1292's applicability sentence and §5.1's gate must state the
same precondition, and r15's class-10 cells are the acceptance witness.

## new-I2 (Important) — r17's new-I1 repair stops at §5.1; §6's cause-selection step 4 still carries the old token test, while §5.1 claims the four tests are aligned with §6's steps

**Where.** §5.1 L818–821 (T3) versus §6 L1229–1233 (step 4), and §5.1 L811's
parenthetical *"(aligned with §6's cause-selection steps; corrected per R0 r16
and r17)"*.

**Measured, not read:** `grep -n '78 bytes\|78-byte\|base58check'` over the spec
returns exactly **two** hits — L819 and L1230 — and the fold moved one of them.
§6 step 4 still reads *"it is a single base58check token whose payload is 78
bytes"*, which is precisely the test r17 proved cannot see `xpub…/<0;1>/*`. r17
said so in the finding itself: *"any repair to T3 should be mirrored in §6 step
4, or the two diverge again."* They diverged.

**Three consequences, in ascending order.**

1. *The alignment claim is now false.* It is false for T3 (narrower in §6) and
   for T1 (§6 step 3 is still *"input contains `(`"*, wider). Only the T1 half is
   harmless — an input reaching cause-selection has already passed the gate, so a
   wider step 3 loses nothing.
2. *Specific refusals are lost for a real spelling.* A refusing origin-less key
   carrying a use-site path — `Zpub…/<0;1>/*`, `Ypub…/<0;1>/*`, `tpub…/<0;1>/*`,
   `ypub…/<0;1>/*` — now passes the gate but misses step 4, so it lands on step
   5's generic four-forms text instead of the bare-`Zpub`/`Ypub`, bare-`tpub` or
   `ypub`-family rows the spec built for exactly that key class. The generic text
   is true, so this alone is not what makes it Important — it is what r17 already
   ruled *"true, so it is not itself a finding"*.
3. *It invites the defect back.* §5.1 tells the implementer the tests are aligned
   with §6's steps, and §6 step 4 is itself NORMATIVE and more precisely worded.
   An implementer who writes the shape gate from §6 — which the sentence
   sanctions — reproduces r17's new-I1 exactly, and there is no reviewer left
   downstream of this spec to catch it. That is the "folds fail by incomplete
   propagation" shape: the facts are right at the site that was edited and stale
   at the site that was not.

## new-M1 (Minor) — §5.3's closing absolute is repaired for the neither-path site and still contradicted by the substitution's own replacement text

The clause now reads *"No refusal names a flag that refuses in the current
build; **a neither-path refusal** may DESCRIBE a different re-export's future
path"*. The exception is granted only to neither-path refusals — but under the
paragraph's own reading rule (*"naming the flag or otherwise — semantic, not
lexical, per R0 r14's new-M3"*) the **stock replacement the rule mandates**,
*"the scannable-plate path is not in this build — keep the export file; it packs
when the device update ships"* (L1040–1041), names the scannable-plate path in a
build where it refuses, and so does §5.1's window variant *"come back for the QR
plate later; nothing is lost by waiting"*. Neither is a neither-path refusal, so
neither is covered. The **principle** the fold added — *"describing routes
nothing"* — is the correct and sufficient distinction; it is the scoping to
neither-path refusals that leaves the absolute false. Stating the principle
unconditionally, or repairing *"names"* → *"offers"* / *"points the operator at"*,
closes it. Ruling unaffected; no refusal text changes.

## new-M2 (Minor) — "identifier" is undefined in T1, and descriptor/miniscript fragment names are not plain alphabetic

T1 turns on a lexical class the spec never states. Under `[A-Za-z]+` a bare
`or_d(…)`, `and_v(…)`, `multi_a(…)` or `sortedmulti_a(…)` fails the gate and
hears the record refusal; under `[A-Za-z][A-Za-z0-9_]*` it reaches §6's
miniscript row. The realistic input (`wsh(or_d(…))`) is unaffected because `wsh`
is plain alphabetic, and §6's miniscript row stays reachable with `--as` present
(§11 item 4), which is why this is Minor and not Important. One clause naming the
character class removes the ambiguity — and it is worth naming rather than
leaving to the implementer, because the wrapper-prefixed spelling `v:pkh(…)`
fails T1 on **every** reading, which is a decision the spec should be seen to
have made rather than inherited.

## new-N1 (Nit) — §9 item 7 still reads stale, having been flagged to this fold

§9 item 7 says §6's refusal texts *"have not been walked with the operator"*. It
predates the walk fold at `d0647f4`, and §6's rows now cite walk W5/W11/W13 and
F-419 as written from the walk. r17 flagged it *"for the fold to either narrow
or discharge"*; the fold did neither. The true residual is narrower: no
systematic row-by-row walk of §6 has been done.

---

# Standing sweeps

| sweep | method | result |
| --- | --- | --- |
| **quoted spans carry no internal identifiers** | multi-line extraction of all `*"…"*` spans, matched against `§\d｜F-\d{3}｜R0｜NEW-[A-Z]\d｜new-[A-Z]\d｜walk W\d｜conjunct \d｜EXIT_｜r1[0-7]\b｜carriage rule｜window substitution` (pattern widened to `r17`) | **45 spans, 0 violations** — count and result identical to r16 and r17. The fold added no quoted text; its new exemplars (`text: my wallet (2 of 3)`, the r17/r16 provenance labels) sit in backticks and prose, outside every operator-visible span |
| **substitution reach** | enumerate every refusal text routing to the descriptor path; check the exemption removes exactly the NEITHER-PATH one | **5 sites, 4 taking substitution, 1 exempt** — §5.3(a) L968, §5.3(a″) L1027, §6 L1273, §6 L1291, and the `multi`-form replacement (L1273, incorporated by reference at L1291). Unchanged from r17: the trigger sentence *"NEITHER-PATH refusals are exempt"* is byte-identical across the diff, and r17's L1267/L1285 are this round's L1273/L1291 — a uniform +6, matching the fold's net +6 lines. The new clause grants a permission, not a new exemption trigger, so reach does not move |
| **diff containment** | `git show --numstat 7bc5e8f`, `git show --numstat 7cb3daf` | fold: **1 file, 2 hunks, +18/−12**, both hunks briefed (§5.1's gate bullet, §5.3's closing absolute). Persist: 1 file, +355, report only. **Nothing outside the four edits changed** |
| **§4.5 row count** | mechanical count from the table header at L551 | **15** rows — the gate's claim *"all fifteen §4.5 rows"* is measured, not transcribed |
| **shape-test site count** | `grep -n '78 bytes\|78-byte\|base58check'`, `grep -n 'first token\|contains \`(\`'` | 2 sites each; **one of each pair moved** — new-I2 |
| **pinned test** | `cargo nextest run --locked -E 'test(an_unpackable_record_is_refused_before_a_passphrase_is_minted)'` | `PASS [0.003s]` — 1 passed, 440 skipped; line number, operand and assertion verified in source |

---

# Verdict

**0 Critical / 2 Important / 2 Minor / 1 Nit. The spec does not re-close GREEN.**

What did close: §4.5 is **15/15** under the gate, §4.7 conjunct 7's use-site set
is **5/5**, the four format happy paths each fire exactly one test, every
`seed:`/`text:`/`pass:`/`tx:`/md1/mistyped-mnemonic cell keeps its shipped exit-4
refusal with its precise text, and the walk lens remains complete. The pinned
test is a real witness with a real control.

What did not: the decision table is **58/60**, not 60/60 — r17's fold moved a
cell r17 believed it had not touched — and the token-test repair reached one of
its two sites.

**The recurrence is the finding underneath the findings.** Four consecutive
rounds have each corrected one disjunct of this gate and broken or half-fixed
another: r15 wrote it from §6's ordering, r16 restored the `[` disjunct, r17
repaired the token test and over-narrowed the parenthesis test, this fold
propagated neither correction into §6. r17 proposed the structural remedy and it
still applies — write the gate as a **derivation** (*descriptor-shaped ⇔ some
branch of §4's cascade could match*) with the four tests as its stated
consequence, so the next missing shape is a contradiction rather than an
omission; and make the exhaustive check a mechanical one, because it is small:
15 rows in §4.5, 4 happy paths, conjunct 7's 5 members, r15's 10 classes — and
**the check must include the multi-record class and both record orderings**,
which is the one axis every round so far has enumerated with the descriptor
first.

## What the spec's own text leaves open regardless of this round

Carried forward from r17, re-checked for staleness against this diff only; none
of these gate the GREEN and none was independently re-verified this round.

- **§9 residuals 1–7**, unchanged: nothing has run on hardware; the three
  admission-table cells have never been exercised (§9's own gate-that-never-ran
  note); change-chain and testnet address equality unmeasured; `md-cli` at repo
  HEAD vs published `md-codec` 0.42.0 not proven byte-identical; TinyGo build of
  a new `sysw.Classify` arm unchecked; the negative claims' search scope named
  and bounded. §9 item 7 additionally reads stale — new-N1.
- **Parked with S2** (F-418, needs the device on the bench): §11 item 1's
  `sysw.Classify` arm and item 6's on-device `ClassDescriptor` display; §6's
  `--as descriptor`-only refusal rows (§11 item 4).
- **Plan-phase items:** §7's vector file does not exist yet (49-row floor, 8-tag
  manifest, one sha256 pinned in both repos); F-414 (descriptor + other records
  in one container — the capability behind new-I1's row); F-416
  (`SPEC_systemwide_payloads` §5.6's `--in` amendment); F-413 (host-side
  version-byte normalisation); F-422 (the Specter question, awaiting an operator
  ruling).
- **Recorded, not filed** (plan-phase notes): a mistyped or truncated extended
  key hears the record refusal, because the gate and §6 step 4 agree on an exact
  78-byte payload; and r15's note that class 10 under an explicit `--as` gets the
  unparseable-file refusal rather than a message naming the record split.
