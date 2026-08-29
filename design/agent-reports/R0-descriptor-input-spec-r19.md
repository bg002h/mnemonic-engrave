# R0 round 19 — VERIFICATION of the r18 fold, and the closure round

**Target:** `design/SPEC_descriptor_input.md` at `3f2362c` ("spec: fold R0 r18 --
uniform per-line gate scope; step 4 mirrored"). `3f2362c` is HEAD; tree clean
(`git status --short` empty).
**Source of truth for what was required:**
`design/agent-reports/R0-descriptor-input-spec-r18.md` (0C/2I/2M/1N).
**Scope, as briefed:** (1) verify the two edits at their sites and re-derive the
cells r18 falsified, including the per-line scope's own new risk; (2) diff
containment, the standing sweeps, and a disposition of r18's two Minors and its
Nit; (3) if it holds, the full closure verdict. Everything from r1–r18 is taken
as settled. **No fresh audit was performed.**
**Reviewer:** independent context, opus tier. Read-only; nothing modified,
committed or pushed.
**Diff read in full:** `git diff 7bc5e8f..3f2362c`. The fold `3f2362c` touches
**one file, 2 hunks, +18/−12**; the range's other commit `f0e2c87` is the r18
report's own persist (+339, report only). Persist and fold are two commits, in
that order.
**Binaries / tools used:** `/home/bcg/.cargo/bin/me` (`me 0.7.0`), `python3`
(base58check arithmetic, the quoted-span sweep, the site-parity sweep), `git`,
`grep`/`awk` for the mechanical counts. Every count below is a tool's output,
not a reading.

---

## Counts

| severity | NEW this round | carried from r18, unfolded |
| --- | :-: | :-: |
| **Critical** | **0** | 0 |
| **Important** | **0** | 0 |
| Minor | 2 | 2 |
| Nit | 0 | 1 |

**0 Critical / 0 Important. The spec RE-CLOSES GREEN, with the walk lens
complete.** Both of r18's Importants are fixed at the sites they name, the two
cells r18 falsified re-derive to their settled answers, the per-line scope does
**not** catch a record line's payload (checked against the shipped classifier,
not assumed), and the decision table is back to **60/60**. The two new Minors and
the three carried items are recorded below; none gates.

---

# Disposition of r18's five findings

| finding | verdict | evidence |
| --- | --- | --- |
| **new-I1** — T1 scoped per INPUT while T2 was per LINE; the multi-record row lost whenever the descriptor is not record 0; falsifies r15's class-10 cells | **FIXED** | §5.1 L811–812 now reads *"by these tests, each applied to EVERY line of the input — uniform per-line scope, so a descriptor buried behind other records still opens the gate and reaches §6's multi-record row"*, and each disjunct was re-worded to match (*"a line whose first token…"*, *"a line whose `": "` key…"*, *"a line that is a single token beginning with `[`…"*). r15's class-10 exemplar re-derived below in **both** orderings: MREC · 4 in each. The repair went the direction r18's derivation pointed — widening the gate rather than narrowing §6 — so §6 L1298's applicability sentence needed no edit and got none |
| **new-I2** — the token-test repair stopped at §5.1; §6 step 4 still carried the old test | **FIXED, parity 2/2 measured** | Wrap-tolerant site sweep over the whitespace-flattened file: `78-byte base58check payload` → **2** hits; `leading segment before any \`/\`` → **2** hits. The two sites are §5.1's T3 and §6 step 4 (L1235), which now reads *"a single token whose leading segment before any `/` is a 78-byte base58check payload (an extended-key envelope, ANY version, with or without a use-site tail — mirrored from §5.1's gate per r18's new-I2)"*. Consequence 2 of the finding is discharged: `Zpub…/<0;1>/*` now fires step 4 and reaches the bare-`Zpub`/`Ypub` row instead of step 5's generic text. **Measured**, not read — real keys, base58check decoded: payload = **78 bytes**, checksum valid, and the leading segment is the bare key for all five §4.7 conjunct-7 use-site spellings (`absent`, `/*`, `/0/*`, `<0;1>`, `<0;1>/*`) → **5/5 fire at BOTH sites**, mainnet `xpub` and testnet `tpub` alike |
| **new-M1** — §5.3's closing absolute repaired for the neither-path site, still contradicted by the stock replacement text | **NOT ADDRESSED — carried** | The fold's 2 hunks are §5.1's gate bullet and §6 step 4; §5.3 was not touched. L1044–1046 is byte-identical to r18's L1040–1042 (uniform +6 line shift, see the sweep table). The finding stands exactly as written |
| **new-M2** — "identifier" is undefined in T1 | **NOT ADDRESSED — carried** | T1's only change is `input` → `line`. The ambiguity is unchanged in kind: a bare `or_d(…)`/`multi_a(…)` line fires under `[A-Za-z][A-Za-z0-9_]*` and not under `[A-Za-z]+`. Per-line scope neither widens nor narrows it — the same two readings, now evaluated per line |
| **new-N1** — §9 item 7 reads stale | **NOT ADDRESSED — carried** | §9 item 7 (L1610–1613) still reads *"The refusal texts in §6 have not been walked with the operator… It should be walked before the plan closes."* §6 cites walk W4/W5/W11/W13 and F-419 in its own rows. Now sharper than a staleness nit: it **contradicts this round's closure premise**, since the GREEN is being called with the walk lens complete. r18's narrowing — *"no systematic row-by-row walk of §6 has been done"* — is still the true residual |

**Both Importants fixed; the fold touched nothing else.** The two Minors and the
Nit were left for the plan phase, which is where Minors and Nits may live.

---

# The cell re-derivations

## 1. `mnemonic + descriptor`, mnemonic FIRST, `--as` absent — the r18 Important's own witness

Input, two lines: a valid 12-word mnemonic, then
`wsh(sortedmulti(2,[fp/84h/0h/0h]xpub…/<0;1>/*,[fp2/84h/0h/0h]xpub…/<0;1>/*))`.

| step | evaluation | answer |
| --- | --- | --- |
| record classification | line 0 → `Class::Mnemonic`; line 1 → `Class::Unknown` (measured: `me 0.7.0`, rc=**4**, *"record 1 … is not a form this container can place"*) | fails ⇒ the gate is consulted |
| the gate, **per line** | T1 on line 1: first token `wsh` is an identifier immediately followed by `(` | **fires** ⇒ descriptor-shaped |
| whole-input re-read | the two-line document parses under no branch (the mnemonic line breaks every one) | does NOT parse as one descriptor |
| does some individual record parse as one descriptor? | line 1 does | yes |
| §5.1 L841–843 ⇒ §6 L1298 | multi-record row: *"record `N` is a wallet descriptor. A descriptor is packed ALONE…"* | **MREC · `EXIT_INVALID` (4)** |

Descriptor-FIRST re-derives identically (T1 fires on line 0). **Both orderings
now reach the row**, which is precisely what r18 showed was lost. Build-invariant:
the row is flag- and build-independent, so `omitted, FULL` and `omitted, WIN`
both read `— · MREC · 4`, restoring r15's two class-10 cells verbatim.

**Plan-phase note, from "ask for a counterexample":** §11 item 4's test for this
row must use the **mnemonic-first** ordering. A descriptor-first test passes in
both the pre- and post-fold worlds and is therefore not a witness for this fix.

## 2. `seed: x (y)` on line 2 of a records file — the per-line parenthesis test must NOT catch a record's payload

Input: a valid 12-word mnemonic, then `seed: my wallet (2 of 3)`.

| test, evaluated per line | line 0 (mnemonic) | line 1 (`seed: my wallet (2 of 3)`) |
| --- | --- | --- |
| **T1** first token an identifier immediately followed by `(` | first token `abandon`, no `(` → no | first token is **`seed:`** — an identifier followed by `:`, not `(` → **no** |
| **T2** `": "` key is a BlueWallet header or 8 hex | no `": "` → no | key is `seed` — neither → no |
| **T3** the line is a single token, `[`-initial or 78-byte leading segment | 12 tokens → no | 5 tokens → no |
| **T4** the WHOLE input is JSON with a descriptor field | — | no |

**Gate CLOSED ⇒ the shipped record refusal stands, exit 4, record vocabulary.**
Measured today: rc=**4**, 0 stdout bytes, *"record 1 … is not a form this
container can place"*. **The per-line paren test does not catch it, and this is
not a coincidence of the exemplar:** T1 keys on the **line's first token**, and
the shipped classifier (`crates/me-cli/src/sysw/mod.rs:211`, read whole) admits
exactly six record shapes — `tx:`, `pass:`, `text:` prefixed, a BIP-39 mnemonic,
an `mt1` string, and a `seal`-validated `md`/`mk`/`ms` string. **None of the six
begins with `identifier(`**, so no well-formed record line can fire T1, whatever
its payload contains. Re-measured for the whole class at `me 0.7.0`, all rc=4,
0 stdout: `text: my wallet (2 of 3)` as record 1 (RESERVED-prefix refusal),
`seed: my wallet (2 of 3)` as record 1, `text: hello` as record 0.

## 3. The 60-cell decision table — verdict

**60/60, single-valued.** The two cells r18 falsified (`class 10 × omitted`, both
builds) are restored to `— · MREC · 4` by derivation 1. No other cell moves, and
the reason is mechanical rather than a survey: classes 1–8 are single-LINE
descriptor exemplars, for which per-line and whole-input scope are the same
predicate; class 9's exemplars (`hello world`, a mistyped mnemonic word) are
multi-token single lines that fire no test under either scope, so they keep
`NOREC · 4`; class 10 is derivation 1; and the empty/whitespace rows never reach
the gate. The `--as`-present columns never consult the gate at all (it runs only
when `--as` is absent), so 30 of the 60 cells are untouched by construction.

## 4. The four format happy paths, re-derived under per-line scope — 4/4

| format | fires | why per-line scope preserves it |
| --- | --- | --- |
| 1 — BlueWallet | **T2** on the `Name: sh` line | T2 was already per-line; the fixture's three leading `#` comment lines still do not defeat it |
| 2 — plain BIP-380 | **T1** on the descriptor line | single-line input: per-line ≡ whole-input |
| 3 — `{label, descriptor}` JSON | **T4**, whole-input | the fold re-worded T4 to *"a whole input that is JSON with a descriptor field"* — necessary, because the fork's fixture is pretty-printed and **no per-line test fires on it**: the `"descriptor"` line's first token is `"descriptor":` (fails T1) and its `": "` key is neither a header nor 8 hex (fails T2). See r19-M1 |
| 4 — promoted bare key | **T3**, all fifteen §4.5 rows | rows counted mechanically off the near-miss table: 17 pipe-lines − header − separator = **15**. Each is a single-token line, so *"a line that is a single token"* ≡ r17's *"a single token"* for every one of them; the `[`-initial rows (5–12) and the leading-segment rows (1–4, 13, 15) fire as they did at `7bc5e8f`, and row 14 fires after §4.6's trim |

---

# NEW findings

## r19-M1 (Minor) — *"uniform per-line scope"* over-claims by one disjunct: T4 is whole-input by its own words, and an implementer who honours the umbrella breaks format 3's happy path

**Where.** §5.1 L811–812, *"by these tests, each applied to EVERY line of the
input — uniform per-line scope"*, against the fourth disjunct at L823, *"or a
**whole input** that is JSON with a descriptor field"*.

**What it costs.** Two readings. (a) All four tests per line — then a
pretty-printed `{label, descriptor}` export fires nothing (checked line by line
above), the gate stays closed, and §11 item 5's acceptance for format 3 fails.
(b) Three per-line tests plus one whole-input test — everything holds. Reading
(b) is the correct one and the fold plainly intended it: it added the words *"a
whole input that is"* to that disjunct in the same edit. The specific clause
governs the general umbrella, so a single-valued answer exists and this is not
Important. But *"uniform"* is the word that makes the umbrella sound normative,
and it is the sentence an implementer will lift. One qualifier — *"each applied
to every line, except the JSON test, which reads the whole input"* — closes it.

**Same shape, smaller:** the predicate is still named *"the input is
DESCRIPTOR-SHAPED"* while its definition is now *"some LINE is descriptor-shaped,
or the whole input is descriptor-JSON"*. Recorded in the same finding; not a
second one.

## r19-M2 (Minor) — the gate is now per-LINE while §6's cause selection is still whole-INPUT, so the specific §6 rows are unreachable for exactly the buried keys the fold newly admits

**Where.** §5.1's four tests (per line) against §6 L1231–1237's five-step rule,
whose steps 2, 3 and 4 are all phrased over the **input** (*"input's first
non-comment line"*, *"input contains `(`"*, *"its first non-whitespace character
is `[`, or it is a single token…"*).

**The derivation.** Take a records file whose line 0 is a valid `md1…` record and
whose line 1 is a bare `Zpub…`. Classification fails (line 1 is `Class::Unknown`
— measured, rc=4 today). The gate now OPENS: T3 fires on line 1, whose leading
segment is a 78-byte base58check payload. The whole input does not parse as one
descriptor, and line 1 does not either (§4.5 row 3 is a REFUSE — `Zpub`'s version
maps to `48'/0'/0'/2'`, not in the promotion loop), so §5.1's multi-record
condition (*"AND some individual record does"*) is not met and MREC does not
fire. Cause selection then runs against the **whole input**: step 1 no, step 2 no
(the `md1` line has no `": "`), step 3 no, step 4 no (first non-whitespace
character is `m`, and a two-line input is not a single token) → **step 5's
generic four-forms text, exit 3** — when §6 holds a dedicated bare-`Zpub`/`Ypub`
row written for exactly this operator's mistake. The same loss applies to a
buried `tpub`, a buried `[fp]xpub…` with no path, and a buried account-≠-0 key:
gate open, MREC unavailable, specific row unreachable. Where the first line does
carry a `": "` — a `text:` record, say — step 2 fires instead and the refusal
names **BlueWallet** as the closest form for a file with no BlueWallet content.

**Why Minor and not Important.** Every outcome remains single-valued, truthful
and at a defined exit code — the leading sentence (*"this is not a wallet
descriptor in any of the four forms `me` reads"*) is true of the whole input, and
*"looks most like"* is a stated resemblance, not a claim. Nothing funds-related
moves: a buried but well-formed inadmissible descriptor (`sortedmulti(0,…)`)
PARSES, so MREC fires, the operator re-runs it alone, and conjunct 2's
anyone-can-spend refusal is reached. And the state is not new — it was already
reachable pre-fold through T2, which has been per-line since r16, and r18
recorded that route (its table (e), row 3) without filing it. **The fold widens
an existing entrance rather than opening one.** What is new is that the widening
lands on the key class §6 built its most specific rows for, which is worth a
plan-phase clause; whether cause selection should follow the line that opened the
gate is a design call for the plan, not a repair this review prescribes.

---

# Standing sweeps

| sweep | method | result |
| --- | --- | --- |
| **quoted spans carry no internal identifiers** | whitespace-flattened file, all `*"…"*` spans extracted, matched against `§\d｜F-\d{3}｜R0｜NEW-[A-Z]\d｜new-[A-Z]\d｜walk W\d｜conjunct \d｜EXIT_｜r1[0-9]\b｜carriage rule｜window substitution` (pattern widened to `r18`/`r19`) | **45 spans, 0 violations** — identical to r16, r17 and r18. The fold added no quoted text; its provenance labels (`r18's new-I1`, `mirrored from §5.1's gate per r18's new-I2`) sit in prose and parentheses, outside every operator-visible span |
| **substitution reach** | locate every §5.3/§6 substitution site at `7bc5e8f` and at `3f2362c` and compare | **unchanged, 5 sites, 4 taking substitution, 1 exempt.** Every site moved by exactly **+6** lines (1033→1037, 1037→1041, 1040→1044, 1273→1279, 1291→1297, 1292→1298), matching the fold's net +6 — a uniform shift, so no text in those regions changed. Hunk 1 (§5.1) is net +4 and hunk 2 (§6 step 4) net +2, which the intermediate site at 1033→1037 confirms |
| **diff containment** | `git show --numstat 3f2362c`, `git show --numstat f0e2c87`, hunk count | fold: **1 file, 2 hunks, +18/−12**, both hunks briefed (§5.1's gate bullet, §6's cause-selection step 4). Persist: 1 file, +339, report only. **Nothing outside the two edits changed** |
| **shape-test site parity** | wrap-tolerant regex over the flattened file | `78-byte base58check payload` **2/2**; `leading segment before any \`/\`` **2/2**; `first token is an identifier immediately followed by` **1** (§5.1 only — §6 step 3's wider *"input contains `(`"* is the deliberate counterpart, ruled harmless by r18 and unchanged); `each applied to EVERY line of the input` **1** |
| **§4.5 row count** | pipe-lines in the near-miss table minus header and separator | **15** — the gate's claim *"all fifteen §4.5 rows"* is measured, not transcribed |
| **T3 arithmetic** | base58check decode of a real `xpub` and a real `tpub`, then the leading segment of all five conjunct-7 use-site spellings | payload **78 bytes**, checksum valid, leading segment == the bare key **5/5** for both networks — so T3 and §6 step 4 fire on all ten constructions |
| **record classes vs T1** | `crates/me-cli/src/sysw/mod.rs:211` read whole | **6 admitted shapes, 0 beginning `identifier(`** — no well-formed record line can fire the per-line paren test |
| **§7 coverage manifest** | tag table summed | **8 tags, minima sum 51**, floor **49** physical rows with two permitted overlaps — internally consistent, and `promotion-near-miss`'s minimum of 15 matches §4.5's measured 15 |
| **shipped surface, status quo** | `me 0.7.0`, seven invocations via `--in` | all rc=**4**, 0 stdout bytes — the baseline every derivation above is measured against |

---

# Verdict

**0 Critical / 0 Important / 2 Minor / 0 Nit (new); 2 Minor + 1 Nit carried from
r18, unfolded.**

**The spec RE-CLOSES GREEN, and the walk lens is complete.** Both of r18's
Importants are fixed at the sites they name and nothing else moved: the gate's
four tests now apply per line (with the JSON test explicitly whole-input), a
descriptor buried behind other records opens the gate and reaches §6's
multi-record row in both orderings, and §6's cause-selection step 4 carries the
same leading-segment token test as the gate — parity 2/2, measured. The 60-cell
decision table is **60/60** single-valued, §4.5 is **15/15** under the gate,
conjunct 7's use-site set is **5/5** at both shape-test sites, the four format
happy paths each still fire exactly one test, and the per-line widening does
**not** reach a record line's payload — established against the shipped
classifier's six admitted shapes rather than against an exemplar.

This is the round that ends the four-round recurrence r18 named. It ends because
the fold repaired the **scope** rather than another disjunct, and because it
propagated to the second site instead of one: the two normative shape tests are
now textually parallel, so the next divergence would have to be introduced
deliberately.

## What the spec's own text leaves open — none of it gating

Re-checked for staleness against this diff only; not independently re-verified
this round except where a count is given.

- **§9 residuals 1–7**, unchanged by this fold: nothing has run on hardware; the
  three admission-table cells have never been exercised (§9's own
  gate-that-never-ran note); change-chain and testnet address equality are
  unmeasured; the published `md-codec` 0.42.0 tarball is not proven
  byte-identical to the tree's; a TinyGo build of a new `sysw.Classify` arm is
  unchecked; the negative claims' search scope is named and bounded. **§9 item 7
  additionally reads stale and now contradicts this closure** — r18's new-N1,
  carried; its true residual is *"no systematic row-by-row walk of §6 has been
  done"*.
- **Carried Minors from r18:** new-M1 (§5.3's closing absolute is still
  contradicted by the stock replacement text it mandates) and new-M2
  (*"identifier"* undefined in T1).
- **New Minors from this round:** r19-M1 (*"uniform per-line scope"* over-claims
  by one disjunct) and r19-M2 (per-line gate vs whole-input cause selection).
- **Parked with S2** (F-418, needs the device on the bench): §11 item 1's
  `sysw.Classify` arm, §11 item 6's on-device `ClassDescriptor` display, and
  §6's `--as descriptor`-only refusal rows within §11 item 4.
- **Plan-phase items:** §7's vector file does not exist yet (**49**-row floor,
  **8**-tag manifest summing to **51** tag-slots, one sha256 pinned in both
  repos); F-414 (descriptor + other records in one container — the capability
  behind the multi-record row); F-416 (`SPEC_systemwide_payloads` §5.6's `--in`
  amendment); F-413 (host-side version-byte normalisation); F-422 (the `/0/*`
  transform ruling — `design/FOLLOWUPS.md:14547`, owning phase *"descriptor-input
  plan, before S1 closes"*, so it is due before S1, not at the end).
- **Recorded, not filed** (plan-phase notes): §11 item 4's multi-record test must
  use the **mnemonic-first** ordering or it is not a witness for r18's new-I1; a
  mistyped or truncated extended key still hears the record refusal, because the
  gate and §6 step 4 agree on an exact 78-byte payload; r15's note that class 10
  under an explicit `--as` gets the unparseable-file refusal rather than a
  message naming the record split; and the gate's per-line scope is defined over
  lines, which for a multi-operand argv invocation means the record stream's
  LF-separated records (§4.6's *"the whole input"*), worth one clause in the plan.
