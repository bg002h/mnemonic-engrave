# R0 r0 — correctness/whole-document lens on the composer fold of `SPEC_systemwide_payloads.md`

**Artifact:** `design/SPEC_systemwide_payloads.md` as folded by commit `12e0659`
("fold: payload spec -- composer classes ... the CREATED Wallet Policy admission
row ... the three reserved prefixes").
**Lens:** is the folded document internally consistent and true, and did three
correct local edits make anything ELSE in the same document false?
**Mandate checked against:** `design/SPEC_wallet_policy_composer.md` §6a and §10
item 6 (closed 0C/0I; its rulings are not re-reviewed here).

**What I ran:** `git show 12e0659`; a full read of all 1574 lines of the folded
document; `git log -1` + full read of `gui/sysw_admit.go`, `sysw/record.go`,
`gui/sysw_admit_oracle_test.go`, `gui/sysw_unload.go` at fork
`/scratch/code/shibboleth/seedhammer` HEAD `169073c`; `grep -rn "seedEntryFlow"`
and `grep -rn "progWalletPolicy|progTransaction"` over the fork's non-test Go;
`git log -S'progWalletPolicy' -- gui/sysw_admit.go`; an `awk` cell-count over
every row of the §3.3.2 table; reads of composer §6a, §7d, §10; `grep` of
`design/FOLLOWUPS.md` for F-415 and `progTransaction`; `grep` of
`design/SPEC_engrave_transaction.md` for `tx:`/`3.3.2`; `grep` of
`crates/me-cli/src/sysw/coverage.rs` for class/program/prefix enumerations.

## VERDICT: 0C/3I/4M/5N

---

## I-1 — §3.3.2 still asserts a cell-perfect transcription that the fold's own new bullet says is false

`design/SPEC_systemwide_payloads.md:402-408`

> "**RULED AT FOLD, 2026-08-12 — the table is the normative RECORD; enforcement is
> per-site and structural (§13 D7).** The journeys review measured the
> implementation: the table is transcribed cell-perfectly into
> `gui/sysw_admit.go`, and its `admits()` has **zero non-test callers** — every
> consumption site instead names exactly one class in its `take`/`syswOffer`
> call, and each named class is a `•` in its program's row (checked cell by
> cell)."

Contradicted twenty lines above by the fold's own new bullet,
`design/SPEC_systemwide_payloads.md:384-386`:

> "The fork has carried a `progWalletPolicy` admission map since S2
> (`gui/sysw_admit.go`: `ClassDescriptor`, `ClassMDMK`) with no row here."

Measured at fork HEAD `169073c`: `progWalletPolicy` carries exactly
`{sysw.ClassDescriptor: true, sysw.ClassMDMK: true}` — 2 of the new row's 8
admitted cells — and `sysw.Class` has no `ClassKey`, `ClassHash` or `ClassNow`
member at all, so the three new COLUMNS are untranscribable today.

Severity: **Important.** This is not the expected-and-permitted "fork map lags
the table" mismatch; it is the document, inside one NORMATIVE section, stating
both that the transcription is cell-perfect and that a row is missing from it.
An implementer who reads line 404 at face value concludes composer Stage 2's
transcription work is already done, and the reachability bullet below it (M-1)
is measured over the same stale premise.

Remedy: one dated clause on line 404 — the cell-perfect measurement is as of
2026-08-12, and the Wallet Policy row plus the `Key`/`Hash`/`Now` columns created
2026-09-02 await the composer's device stages.

## I-2 — the NORMATIVE table now carries a ninth program; §1 decision 3 and §3.1 still apportion exactly eight

`design/SPEC_systemwide_payloads.md:362` (new row), against `:33-35`, `:236-241`,
`:6` and `:1261`.

The new row:

> `| Wallet Policy | • | • | • | | • | • | | • | • | • |`

Decision 3, `:33-35`:

> "3. **Scope is all eight non-Sealed-Payload programs**: Backup Wallet, BIP-39
> Password, Engrave Text, Account Xpub, Engrave Bundle, Engrave Single-Sig,
> Engrave Multisig, BIP-85 Child Seed."

§3.1, `:236-241`:

> "**"One seam, not eight" is therefore false as originally written, and the spec
> says so rather than keeping the slogan.** The seam covers 4 of the 8 programs.
> The other four do not share a helper at all… So the work is **one shared seam
> plus four individual wirings**, and each of the four gets its own admission
> test rather than inheriting one. Pretending otherwise would have under-scoped
> the implementation by half the programs."

Wallet Policy is in neither set. Measured in the fork: `seedEntryFlow` /
`seedEntryFlowTitled` / `seedEntryFlowResume` have no call site inside
`wallet_policy.go`, and `gui/sysw_admit_oracle_test.go:66` registers
`wallet_policy.go:walletPolicyFlow` under `progWalletPolicy` alone. So the row
admits three seed classes to a program for which this document names no entry
path at all — under a table whose own preamble (`:305-307`) says "**An
implementer transcribes it; nothing here is left to be derived.**" The two
program-count claims at `:6` ("eight programs, two repos") and `:1261` ("If NFC
becomes a first-class secret path for eight programs") are stale by the same
edit.

Severity: **Important.** Two normative sections (§1's operator-ruled scope, §3.1's
measured wiring apportionment) now disagree with §3.3.2's normative table about
how many programs this spec governs, and the disagreement is exactly the
under-scoping §3.1 exists to prevent.

Remedy: one clause — Wallet Policy is a ninth program that post-dates the
2026-08-11 scope list, and its seed step is the composer spec's §7d seating, not
§3.1's seam.

## I-3 — §5.3.1's normative classification-ORDER sentence still says "the two prefixes"; the next clause in the same paragraph reserves five

`design/SPEC_systemwide_payloads.md:616-622`

> "**Classification order is normative:** the two prefixes are matched **before**
> the existing sniffers in `Classify`. … The prefixes are also **reserved**: a
> record beginning `text:`, `pass:`, `key:`, `hash:` or `now:` that is not valid
> lowercase hex is `ClassUnknown` and refused, never silently treated as free
> text."

"The prefixes" in the second sentence is anaphoric to "the two prefixes" in the
first, so the paragraph now states that the two prefixes are five. What it
contradicts, in the fold's own new paragraph at `:593-597`:

> "The three composer prefixes follow the same rule as the first two (reserved,
> lowercase hex body, matched before the sniffers, a body that fails ANY rule is
> `ClassUnknown` and refused)"

and in the mandate, `SPEC_wallet_policy_composer.md` §6a:

> "All three follow `SPEC_systemwide_payloads.md` section 5.3: a reserved prefix,
> a lowercase-hex body, matched BEFORE the sniffers, and a prefixed record whose
> body is not valid hex is `ClassUnknown` and refused."

The composer defers to this sentence as the normative source of the
matched-before-the-sniffers rule for all three classes, and the sentence is
scoped to two.

Severity: **Important.** An internal contradiction inside a single normative
paragraph, on the one rule another closed spec cites this section for by
reference.

Remedy: "the two prefixes" → "the reserved prefixes" (one word pair).

## M-1 — §3.3.2's two dated measurement lists were not extended for the new row

`design/SPEC_systemwide_payloads.md:423-430` and `:431-438`

> "Cells with no consumption path today: `ClassCodex32Secret` everywhere (the
> inconsistency below); `ClassPassphrase` at the four seam programs …;
> `ClassMDMK` at Single-Sig and Multisig (the supplied-md1 path)."

> "§3.1's NORMATIVE seam signature returns `bip39.Mnemonic`, which cannot carry
> the `ClassCodex32Secret` this table admits to all four seam programs."

Measured: `progWalletPolicy`'s only consumption sites are `ClassMDMK` and
`ClassDescriptor` (`gui/sysw_admit_oracle_test.go:66-74`), so the new row adds
six cells with no consumption path (`Mnem`, `Cdx32`, `Passph`, `Key`, `Hash`,
`Now`), and `Cdx32` becomes a FIFTH unservable cell — one the composer actively
requires: §7d, "seeds — BIP-39 words or ms1, from the payload or typed".
Neither list says so.

Severity: **Minor.** Both are explicitly dated 2026-08-12 records and no rule
changes; the same clause that fixes I-1 can carry them.

## M-2 — §5.3's own enumeration and its Rust-first clause do not reach the three classes §5.3.1 now carries

`design/SPEC_systemwide_payloads.md:546-557`

> "### 5.3 Widened admission — NORMATIVE, Rust first … Today the public section
> admits **`ClassMDMK` only** … Two changes: 1. **A record class for free
> text** … 2. **Secret classes permitted in a plaintext container.** … Both are
> wire/admission behaviour. Per the Rust-primary rule they land in
> `mnemonic-engrave`'s Rust **with test vectors first**".

Three further classes now widen the same public-section admission, and the new
§3.3.1 rows point back here for them — "`**NEW** (§5.3; composer spec §6a)`"
(`:330-332`) — while §5.3 proper never mentions them and its "Both" leaves them
outside the section's stated Rust-first obligation.

Severity: **Minor.** Nothing is unowned in fact: the composer's §10 is titled
"Host work items (Rust first)" and its item 2 carries `me sysw pack`'s three
classes with vectors (§12 item 8, cross-language). The payload spec's own list is
merely incomplete.

## M-3 — PRE-EXISTING (not this fold): the "zero callers" claim about `admits()` is false at fork HEAD

`design/SPEC_systemwide_payloads.md:405` ("its `admits()` has **zero non-test
callers**") and `:1556` (§13 D7: "`admits()` has zero production callers").

Measured at fork HEAD `169073c`: `gui/sysw_unload.go:89`

```go
if admits(progTransaction, r.class) {
```

inside `syswPayloadHasTransaction`, whose own comment says the question is
"Asked through the ADMISSION TABLE rather than by listing classes here". This
was introduced by the Engrave Transaction cycle (`5623824`, 2026-08-25), not by
this fold.

Severity: **Minor, non-gating for this fold.** Recorded because I-1's remedy
edits the same sentence, so it is one edit rather than two.

## M-4 — the `progTransaction` note reassigns half of the follow-up the fold cites two bullets earlier

`design/SPEC_systemwide_payloads.md:396-398`

> "- **`progTransaction` has an admission map in the fork and NO row here.** Noted
> 2026-09-02 while creating the Wallet Policy row; it belongs to the
> transaction-engraving cycle's owner and is not created by this fold."

`design/FOLLOWUPS.md:14533-14538` (F-415, cited at `:384` as "F-415 named the
gap"):

> "The §3.3.2 admission table … lists `Descr` cells for Engrave Bundle and
> Engrave Multisig only — no Wallet Policy row, **no Engrave Transaction row** —
> while the code admits `ClassDescriptor` in `progWalletPolicy` too. … The
> reconciliation (add the missing rows, or remove the code cell) belongs to
> `SPEC_systemwide_payloads`' own next cycle, with the usual gate."

Measured: `design/SPEC_engrave_transaction.md` contains no occurrence of
`progTransaction`, `3.3.1` or `3.3.2`; that cycle shipped 2026-08-25; and no
entry in `design/FOLLOWUPS.md` mentions `progTransaction`. So the named owner has
no open vehicle, and F-415 is about to be closed with half its scope
unaddressed and unfiled. (Writing the note at all is mandated by composer §10
item 6 — "the missing `progTransaction` row is noted for its own owner" — so the
defect is the ownership sentence, not the decision to note it.)

Severity: **Minor.** Scheduling, not a rule.

Remedy: file a follow-up for the Transaction row (with `ClassMt`/`ClassTx`, see
N-5), or point the sentence at F-415's remaining half.

## N-1 — "stay refused" in a row created by this fold

`:393` — "`FreeText` and `Address` stay refused." Nothing "stays" in a row that
did not exist before line 362 was written. "are refused" is the true form.

## N-2 — the `ClassNow` paragraph drops the refusal half of the composer's rule

`:335-338` — "`ClassNow` is a LOWER BOUND on the present — the pack time — that
the device, which has no clock, echoes beside a time lock and never encodes into
anything". Composer §6a: "the record affects ONLY echoes **and refusals** (§6b),
never an encoded operand". §6b's date floor and below-bound refusal are omitted.
Not false (the load-bearing "never encodes" half is exact), and body rules are
delegated — so a nit.

## N-3 — "since S2" is early by nine days

`:385` — "The fork has carried a `progWalletPolicy` admission map since S2".
Measured: `progWalletPolicy: {sysw.ClassDescriptor: true, sysw.ClassMDMK: true}`
appears verbatim in `gui/sysw_admit.go` at `09c5f14` (2026-08-20), before S2;
what S2 added (`cde7545`, 2026-08-29) was the Descriptor cell's first CONSUMER,
which is what `gui/sysw_admit.go`'s own comment says ("added at S2 P3.2").

## N-4 — §5.3.1's heading and argument are about two classes; the block now governs five

`:559`, `:570`, `:576` — "The two new classes collide with EPD§6.4", "**Both new
classes violate both clauses.**", "Relaxing EPD§6.4 for two classes would weaken
the rule for all of them". The three composer bodies carry no interior space, no
hyphen and no LF, so they inherit the hex encoding without the collision that
justifies it, and the section does not say so. Harmless — hex is right for them
for the lowercasing and comparison reasons at `:601-606` — but the heading now
undersells its own scope.

## N-5 — PRE-EXISTING: §3.3.1 has no `ClassMt`/`ClassTx` rows

`sysw/record.go:39` and `:43` carry both, both documented there as NOT secret.
The fold's `progTransaction` note covers the missing admission ROW only, not the
two missing class rows. Same owner as M-4; not created by this fold.

---

## What the fold made false elsewhere

**Found — three sites, all recorded above:** §3.3.2's 2026-08-12 measurement
block (I-1, M-1, M-3), §1 decision 3 + §3.1 + `:6` + `:1261`'s program count
(I-2), and §5.3/§5.3.1's two-class enumerations (I-3, M-2, N-4).

**Grepped and clean:**

- `eight|Eight|8 programs|of the 8|nine|Nine` → 5 hits, all handled in I-2 (`:6`,
  `:33`, `:237`; `:512` and `:1261` are the 8-byte magic and the emulator line).
- `every class|all classes|each class|the classes|five classes|seven|ClassFreeText|prefixes`
  → no enumeration outside §3.3.1/§5.3.1 that the three new classes falsify.
- §3.3.1's "The secret column **extends** `seal/session.go:17`… It becomes those
  two plus `ClassPassphrase`" (`:341-344`) — still TRUE, precisely because the
  composer classes are non-secret; the fold's non-secret ruling is what keeps it
  so.
- §3.3.3's flag rules (`:468-487`) — F1/F2/F4 key on "admitted class is secret",
  never on a class list; the new row changes nothing there, and the fold's
  "F1 and F2 fire inside the composer's seed step" is the correct consequence.
- §3.3.2's "`ClassPassphrase` is admitted wherever a seed is" (`:366-372`) — the
  new row preserves the invariant (Mnem •, Cdx32 •, Passph •), and the
  justification is real: composer §7d, "per-seed passphrase as Multisig Build
  offers". Its cited list of three derive functions is not extended, which is
  cosmetic.
- §12 (`:1392-1527`) — no definition enumerates prefixes, classes or the table's
  column count. `[cliff]`, `[compared]`, `[identity]`, `[digest-shown]`,
  `[passphrase-bounds]`, `[mdmk-decode]` are all untouched by the fold.
- §13 (`:1528-1574`) — only D7 (M-3, pre-existing) touches `admits()`; no row
  enumerates classes or programs.
- §8/§8.3 (`:1248-1346`) and `crates/me-cli/src/sysw/coverage.rs` — no test id's
  text enumerates classes, prefixes or programs; the only `Class` mention in
  `coverage.rs` is `:201`'s `ClassMDMK` note. Nothing falsified.
- §5.6's `me sysw pack` surface (`:858-900`) — currently still TRUE and complete.
  **Watch item, not a finding:** composer §10 item 2 adds `--no-now` and an
  auto-append of `now:` to `me sysw pack`. When that ships, §5.6 — this
  document's NORMATIVE single source for that surface — becomes incomplete in
  exactly the shape F-416 was filed for ("two NORMATIVE contracts must not
  disagree silently"). §10 item 6 did not mandate a §5.6 edit, so this belongs to
  the composer cycle at ship, not to this fold.

## Claims in the new text checked

| # | Claim (site) | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | The three classes are NOT secret (`:334`) | **TRUE** | Composer §6a: "None is secret." |
| 2 | `key:<hex of "[fingerprint/path]xpub">` (`:330`) | **TRUE** | Composer §6a table: "hex of the UTF-8 text `[fingerprint/path]xpub` (BIP-380 key-origin notation…)" |
| 3 | `hash:<64 lowercase hex>`, a 32-byte sha256 digest (`:331`) | **TRUE** | Composer §6a: "the 32-byte digest itself, 64 lowercase hex"; "`hash:` MUST decode to exactly 32 bytes" |
| 4 | `now:<hex of "<seconds>[,<height>]">`, the pack time (`:332`) | **TRUE** | Composer §6a: "hex of the UTF-8 text `<unix-seconds>[,<block-height>]` … the PACK time and optional height" |
| 5 | `ClassNow` never encodes into anything (`:337`) | **TRUE** (incomplete, N-2) | Composer §6a: "never an encoded operand" — but "ONLY echoes **and refusals**" |
| 6 | Body rules and refusal lines live in composer §6a/§8n (`:338-340`) | **TRUE** | §6a is the body-rule table; §8n is titled "Host-side record refusals (`me sysw pack` stderr, §6a), one line each" |
| 7 | The row's ten cells (`:362`) | **TRUE, exact** | Composer §10 item 6: "Mnem •, Cdx32 •, Passph •, FreeText blank, Descr •, MDMK •, Addr blank, Key •, Hash •, Now •". `awk` cell count: every row of the table is 11 fields, header included — the fold added three cells to all ten rows and broke none |
| 8 | `gui/sysw_admit.go` carries `progWalletPolicy` with `ClassDescriptor`, `ClassMDMK` (`:385-386`) | **TRUE** | Fork HEAD `169073c`: `progWalletPolicy: {sysw.ClassDescriptor: true, sysw.ClassMDMK: true}` |
| 9 | …"since S2" (`:385`) | **FALSE (nit)** | See N-3: `09c5f14`, 2026-08-20 |
| 10 | The "NO seed class … least privilege" comment exists and is reversed (`:390-391`) | **TRUE** | `gui/sysw_admit.go:47-51` verbatim; composer §6a: "`gui/sysw_admit.go:47-51` ("NO seed class … least privilege"), which C12 deliberately reverses" |
| 11 | C12 supports seed admission (`:388`) | **TRUE** | Composer `:63`, C12: "seeds (BIP-39 words or ms1) are a key source, via the unsealed payload or the keyboard" |
| 12 | The three composer classes "no other program admits" (`:392-393`) | **TRUE** | Only row 362 carries `•` in Key/Hash/Now |
| 13 | F1 and F2 fire in the composer's seed step as in Multisig Build (`:393-395`) | **TRUE** | Composer §10 item 6 states it in those words; §3.3.3's F1/F2 key on secrecy, not on program |
| 14 | F-415 named the gap (`:384`) | **TRUE** | `design/FOLLOWUPS.md:14530` — and see M-4 for the half it also named |
| 15 | `progTransaction` has a map and no row here (`:396`) | **TRUE** | `gui/sysw_admit.go`: `progTransaction: {sysw.ClassMt: true, sysw.ClassTx: true}`; no Transaction row in §3.3.2 |
| 16 | …"belongs to the transaction-engraving cycle's owner" (`:397-398`) | **FALSE (M-4)** | F-415 assigns it to this document's own next cycle; `SPEC_engrave_transaction.md` never mentions §3.3.2 |
| 17 | The three prefixes follow the same rule as the first two (`:593-597`) | **TRUE** | Composer §6a: "All three follow `SPEC_systemwide_payloads.md` section 5.3: a reserved prefix, a lowercase-hex body, matched BEFORE the sniffers…" — but see I-3, the sentence it defers to still says "two" |
| 18 | `tx:` is the transaction cycle's prefix, governed by that cycle's spec (`:596-597`) | **TRUE** | `sysw/record.go:21` `TxPrefix = "tx:"`; `design/SPEC_engrave_transaction.md` §2.1 "One framed record under `tx:`" |
| 19 | The delegation of body rules to another spec conflicts with §5.3's "Rust first, with test vectors first"? | **NO CONFLICT** | Composer §10 is titled "Host work items (**Rust first**)"; item 2 puts the three classes' body rules and refusal lines in `me sysw pack`'s Rust `pack_with`, and §12 item 8 requires a cross-language vector set. The Rust-primary rule is honoured — §5.3's own sentence just does not reach them (M-2) |
| 20 | §12's "every rule below is defined HERE and nowhere else" vs the delegation | **NO CONFLICT** | §12 governs the six bracketed rules it names; none of them is a record-body rule, and §5.3.1 delegates rather than restating — which is §12's own prescribed shape |

## What I ran

```
git show 12e0659 [--stat]
sed -n '1,200p;201,500p;500,700p;700,1000p;1000,1350p;1347,1574p' design/SPEC_systemwide_payloads.md   # whole document
grep -n '^#|^##|^###|^####' design/SPEC_systemwide_payloads.md
awk 'NR>=352 && NR<=364 {gsub(/\|/,"|"); ...}'                      # cell count per table row -> 11/11 on all 12 lines
grep -n 'eight|Eight|8 programs|of the 8|nine|Nine' design/SPEC_systemwide_payloads.md
grep -n 'the two prefixes|two new classes|Both new classes|for two classes|Two changes' design/SPEC_systemwide_payloads.md
grep -n 'every class|all classes|each class|the classes|five classes|seven|ClassFreeText|prefixes' design/SPEC_systemwide_payloads.md
grep -n 'transcribed cell-perfectly|zero non-test callers|Reachability, recorded|zero production callers' design/SPEC_systemwide_payloads.md
grep -n 'prefix|Class|program' crates/me-cli/src/sysw/coverage.rs
sed -n '257,330p;806,840p;436,455p' design/SPEC_wallet_policy_composer.md
grep -n 'key:|hash:|now:|ClassKey|ClassHash|ClassNow' design/SPEC_wallet_policy_composer.md
grep -n -i 'passphrase' design/SPEC_wallet_policy_composer.md
sed -n '14528,14552p' design/FOLLOWUPS.md ; grep -n 'progTransaction' design/FOLLOWUPS.md   # -> no hits
grep -n 'progTransaction|3.3.2|3.3.1' design/SPEC_engrave_transaction.md                    # -> no hits
grep -n 'tx:' design/SPEC_engrave_transaction.md

# fork /scratch/code/shibboleth/seedhammer @ 169073c (read-only)
git log --oneline -1
cat gui/sysw_admit.go ; sed -n '1,60p' sysw/record.go ; sed -n '1,110p' gui/sysw_admit_oracle_test.go ; sed -n '70,100p' gui/sysw_unload.go
grep -rn "seedEntryFlow" --include="*.go" . | grep -v _test.go
grep -rn "progWalletPolicy|progTransaction" --include="*.go" .
grep -rn "ClassMt|ClassTx|ClassKey|ClassHash|ClassNow" sysw/*.go
git log --format='%h %ad %s' --date=short -S'progWalletPolicy' -- gui/sysw_admit.go
git show 09c5f14:gui/sysw_admit.go | grep -n progWalletPolicy
```

Not re-derived, per the brief: the table/glyph/cite gate complaints on this file
(all pre-existing and untouched), and the expected lag between the new row and
today's fork map.
