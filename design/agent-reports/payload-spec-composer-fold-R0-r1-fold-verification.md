# R1 fold verification — `44765d7` responding to R0 r0 correctness report

**Artifact:** `design/SPEC_systemwide_payloads.md`.
**BEFORE:** `12e0659` (original composer fold). **AFTER:** `44765d7` (= working
tree). **Report under verification:** `design/agent-reports/payload-spec-composer-fold-R0-r0-correctness.md`
(opus; persisted `bb49953`; 0C/3I/4M/5N on `12e0659`).
**Companion commit:** `72ac66d` (files `F-450` in `design/FOLLOWUPS.md`, the
M-4/N-5 response).
**Lens:** mechanical fold verification — did the fold fix each finding as
stated, and did it make anything else false? Not a fresh audit.

**What I ran:** `git show bb49953` (full report text); `git diff 12e0659..44765d7 -- design/SPEC_systemwide_payloads.md`;
`git show 72ac66d`; the 8 grep phrasings from the brief plus one extra
(`zero production callers`, the D7 mirror of M-3) against the AFTER file;
`awk` cell count over table rows 362-374 (all 11 fields, unchanged); against
`/scratch/code/shibboleth/seedhammer` at HEAD `169073c` (read-only): `git show
-s --format='%h %ci %s'` on `09c5f14`, `cde7545`, `5623824`; `sed`/`grep` on
`gui/sysw_unload.go:75-100`, `gui/sysw_admit_oracle_test.go:60-75`,
`sysw/record.go:27-43`, `gui/sysw_admit.go` (progWalletPolicy/progTransaction
maps); reads of composer spec §7d, §6b, §10, §12 item 8 in
`design/SPEC_wallet_policy_composer.md`.

## VERDICT: 11 FIXED / 1 PARTIAL / 0 NOT FIXED / 0 DECLINED — 0 regressions, 0 new defects

## Per-finding table

| id | title | fold's response (AFTER text, abridged) | verdict |
| --- | --- | --- | --- |
| I-1 | §3.3.2 asserted cell-perfect transcription while its own new bullet said a row was missing | `:414-424`: "AS OF 2026-08-12: the table was transcribed cell-perfectly... admits() had zero non-test callers... Two dated departures since... the Wallet Policy row and the Key/Hash/Now columns created 2026-09-02 are NOT yet transcribed (progWalletPolicy carries ClassDescriptor and ClassMDMK only, sysw.Class has no composer members yet)... and the Engrave Transaction cycle (2026-08-25) gave admits() one production caller, gui/sysw_unload.go:89..., so 'zero' is the 2026-08-12 measurement." | **FIXED** |
| I-2 | table has a 9th program row; §1 decision 3, §3.1, `:6`, `:1261` still say eight | Decision 3 `:33-38` adds "(Wallet Policy... is a NINTH program... 'Eight' elsewhere in this document is the 2026-08-11 count.)"; §3.1 `:238-241` adds the same note inline; `:6` → "eight programs as ruled in 2026-08 and a ninth since 2026-09-02"; `:1295` (was `:1261`) → "the eight (now nine) programs" | **FIXED** |
| I-3 | §5.3.1's "the two prefixes" contradicted its own new 3-prefix paragraph | `:649`: "the reserved prefixes are matched **before** the existing sniffers" (was "the two prefixes") | **FIXED** |
| M-1 | the two dated cell-reachability lists were not extended for the new row | `:449-455` new sentence: "six of its eight admitted cells have no consumption path today (Mnem, Cdx32, Passph, Key, Hash, Now; only Descr and MDMK are consumed...), and Cdx32 there is a FIFTH unservable-until-carried cell..." | **FIXED** |
| M-2 | §5.3's "Both are wire/admission behaviour" and Rust-first clause didn't reach the 3 composer classes | `:583-587`: "**All three** are wire/admission behaviour... (the composer classes: me sysw pack, composer spec §10 item 2, with the cross-language fixture of its §12 item 8)" | **FIXED** |
| M-3 | PRE-EXISTING: `admits()` "zero non-test/production callers" is false at fork HEAD (2 sites: `:405`≈now `:417`, and §13 D7 `:1556`≈now `:1590`) | I-1's edit (above) folds M-3's fact into the `:417` sentence, exactly as directed. The §13 D7 mirror at `:1590` ("`admits()` has zero production callers") is **untouched** — still unqualified, still false at fork HEAD | **PARTIAL** — fixed where directed, one mirrored occurrence left stale (see Propagation sweep) |
| M-4 | the `progTransaction` note reassigned F-415's remaining half to a nonexistent owner | `:396-403`: "...Filed as mnemonic-engrave F-450 with the payload spec's next cycle as owner; not created by this fold." | **FIXED** |
| N-1 | "stay refused" in a row that didn't previously exist | `:403`: "`FreeText` and `Address` **are** refused." | **FIXED** |
| N-2 | `ClassNow` paragraph dropped the refusal half of composer §6a's rule | `:344-345`: "...echoes beside a time lock — and uses for its lock-entry refusals (a date below the bound, the 2009 floor: composer §6b) — and never encodes into anything" | **FIXED** |
| N-3 | "since S2" is early by 9 days | `:392-393`: "since 2026-08-20 (`gui/sysw_admit.go` at `09c5f14`: ClassDescriptor, ClassMDMK; S2 added the Descriptor cell's first consumer on 2026-08-29)" | **FIXED** |
| N-4 | §5.3.1 heading/argument governs 2 classes but the section now covers 5 | Heading `:600`: "...(the three composer classes inherit the encoding, see the end of this section)"; new parenthetical at `:610-614` explaining why the composer classes don't collide but take the encoding anyway | **FIXED** |
| N-5 | PRE-EXISTING: §3.3.1 has no `ClassMt`/`ClassTx` rows | Folded into the same M-4 bullet and F-450 (`72ac66d`): "§3.3.1 has no `ClassMt`/`ClassTx` rows (`sysw/record.go` carries both, not secret)" plus F-450's own body naming both classes and line numbers | **FIXED** |

## Propagation sweep

| phrasing | hits in AFTER | verdict |
| --- | --- | --- |
| `the two prefixes` | 0 | removed, replaced with "the reserved prefixes" (I-3) |
| `Both are wire` | 0 | removed, replaced with "All three are wire" (M-2) |
| `transcribed cell-perfectly` | 1 (`:416`, "AS OF 2026-08-12: the table was transcribed cell-perfectly") | past tense, dated — correct |
| `zero non-test callers` | 1 (`:417`, "admits() **had** zero non-test callers") | past tense, dated — correct |
| **extra check** `zero production callers` | 1 (`:1590`, §13 D7: "`admits()` has zero production callers") | **NOT dated, present tense, still false at fork HEAD** — the M-3 mirror the fold left untouched |
| `stay refused` | 0 | removed, replaced with "are refused" (N-1) |
| `since S2` | 0 | removed, replaced with "since 2026-08-20 ... ; S2 added ... on 2026-08-29" (N-3) |
| `Two changes` | 1 (`:574`, "Two changes ruled 2026-08-11, and a third added 2026-09-02") | exact required text — correct |

**`eight`/`Eight` — full list, 8 hits:**

| line | text (abridged) | verdict |
| --- | --- | --- |
| `:6` | "eight programs as ruled in 2026-08 and a ninth since 2026-09-02, two repos" | dated, ninth noted — OK |
| `:33` | "Scope is all eight non-Sealed-Payload programs: [8 named]" | decision-3's original list; ninth immediately noted in the next 5 lines (`:35-38`) — OK |
| `:38` | "...not §3.1's seam. 'Eight' elsewhere in this document is the 2026-08-11 count.)" | meta-note governing interpretation of the term elsewhere, not itself a count claim — OK |
| `:240` | "**'One seam, not eight' is therefore false as originally written**... The seam covers 4 of the 8 programs ruled in decision 3 (Wallet Policy, the ninth, added 2026-09-02...)" | historical-claim heading + immediate ninth-noted correction — OK |
| `:450` | "six of **its eight** admitted cells" | about the Wallet Policy row's own admitted-cell count (8 of 10 columns), not the program count — not applicable |
| `:537` | "**8** bytes, ASCII, `MNEMSYSW`. **Eight** to match `MNEMBLOB`" | magic-number byte count, not program count — not applicable |
| `:624` | `now:<...[,<height>]...>` | substring match on "height", not the word "eight" at all — not applicable |
| `:1295` | "the eight (now nine) programs" | dated/updated correctly — OK |

All program-count occurrences of "eight" are either dated to 2026-08 with the
ninth noted nearby, or updated to "eight (now nine)". No stale bare "eight
programs" claim survives.

## Claims checked

1. **Decision 3's parenthesis vs composer §7d's seating** — TRUE. Composer
   `SPEC_wallet_policy_composer.md:443-445` (§7d): "seeds — BIP-39 words or
   ms1, from the payload or typed; a seed may fill several slots... per-seed
   passphrase as Multisig Build offers; scrubbed on exit (C14)." Decision 3's
   new text ("its seed step is the composer spec's own seating, not §3.1's
   seam") accurately points at this seating mechanism as distinct from §3.1's
   `seedEntryFlow` seam.
2. **§3.1's "neither of the two groups below"** — TRUE. §3.1's table (`:213-217`)
   names the 4-program seam (BIP-85 Child Seed, Account Xpub, Engrave
   Single-Sig, Engrave Multisig); the text at `:243-245` names "the other four"
   (Backup Wallet, BIP-39 Password, Engrave Text, Engrave Bundle) — 4+4=8,
   matching decision 3's list exactly. Wallet Policy appears in neither list.
3. **`gui/sysw_unload.go:89` is a production caller; Transaction cycle date** —
   TRUE, both halves. `gui/sysw_unload.go:88-89`: `for _, r := range
   s.records { if admits(progTransaction, r.class) {` inside
   `syswPayloadHasTransaction`, a non-test function. `git show -s --format='%h
   %ci %s' 5623824` in the fork → `5623824 2026-08-25 13:52:23 -0700 gui: the
   Engrave Transaction program...`.
4. **M-1's oracle-test evidence and the "FIFTH" count** — TRUE, both halves.
   `gui/sysw_admit_oracle_test.go:66-74` registers
   `{"wallet_policy.go", "walletPolicyFlow", []syswProgram{progWalletPolicy}}`
   with comment text naming exactly "ClassMDMK for the card route and
   ClassDescriptor for §5.2's re-encoded record" as the two consumed classes —
   matches "only Descr and MDMK are consumed" exactly. On the FIFTH count: the
   pre-existing (untouched) sentence two lines below explicitly scopes the
   prior unservable-Cdx32 claim to "all four seam programs" (the
   `bip39.Mnemonic`-typed seam signature can't carry `ClassCodex32Secret`) —
   Backup Wallet's Cdx32 is implied serviceable via its own typed menu ("Backup
   Wallet's typed menu already accepts M*1 strings"), so it is not a fifth
   pre-existing case. Adding Wallet Policy (whose only consumers are
   `ClassMDMK`/`ClassDescriptor`, confirmed above) as a program with an
   unservable Cdx32 cell makes 4+1 = 5. Correct.
5. **N-3's two dates** — TRUE, both. `git show -s --format=... 09c5f14` →
   `2026-08-20 20:26:34 -0700`; `gui/sysw_admit.go:52`:
   `progWalletPolicy: {sysw.ClassDescriptor: true, sysw.ClassMDMK: true},`
   verbatim, present at that commit. `git show -s --format=... cde7545` →
   `2026-08-29 13:04:00 -0700 gui: Wallet Policy takes a Descriptor record, and
   the cell finally fires` — S2's Descriptor-consumer commit.
6. **5.3's "Three composer record classes" citing composer §10 item 2 / §12
   item 8** — TRUE, both. §10 item 2: "`me sysw pack`: `key:`, `hash:`, `now:`
   classes with the §6a body rules and the §8n refusal lines from `pack_with`;
   the payload-wide single-`now:` rule..." §12 item 8: "**Record classes,
   lockstep.** A cross-language vector set: each `key:`, `hash:`, `now:`
   record (valid and each §6a malformation) classifies identically on the host
   and on the device..." Both say exactly what the fold cites them for.
7. **F-450's entry** — TRUE. `sysw/record.go:39` (`ClassMt`) and `:43`
   (`ClassTx`) confirmed present at fork HEAD, both under the "NOT secret"
   comment block. Owning phase is stated explicitly in the commit title and
   body: "owning phase: the payload spec's next cycle — a dated fold under
   that document's own gate; does NOT gate the wallet-policy composer." The
   spec's `progTransaction` bullet (`:396-403`) now reads "Filed as
   mnemonic-engrave F-450 with the payload spec's next cycle as owner", which
   matches F-450's stated owner and corrects M-4's prior false assignment to
   "the transaction-engraving cycle's owner."

## New defects introduced by the fold

None found. Every edit checked against fork/composer-spec source text was
accurate; the table's cell structure is unchanged (11 fields, all 13 rows,
`awk` count); no previously-true sentence was made false by an edit.

The one residual is **not introduced by this fold** — it is M-3's second site
(§13 D7, `:1590`, "`admits()` has zero production callers") left unqualified
while the report's own instruction was to fold M-3's fact into I-1's sentence
specifically (which the fold did, correctly). Since the report cited both
sites as instances of the same false claim, this is worth a follow-up note so
it isn't lost, but it is pre-existing and out of this fold's directed scope,
consistent with the report's own "Minor, non-gating for this fold" severity
call on M-3.

## What I ran

```
git show bb49953                                    # full R0 r0 report text
git diff 12e0659..44765d7 -- design/SPEC_systemwide_payloads.md
git show 72ac66d                                     # F-450 companion commit
grep -n "the two prefixes|Both are wire|transcribed cell-perfectly|zero non-test callers|zero production callers|stay refused|since S2|Two changes" design/SPEC_systemwide_payloads.md
grep -ni "eight" design/SPEC_systemwide_payloads.md
awk -F'|' 'NR>=362 && NR<=374 {print NR": "NF-2}' design/SPEC_systemwide_payloads.md   # 11 fields, all rows
sed -n '30,40p;190,255p;340,470p;570,660p;1280,1300p;1580,1595p' design/SPEC_systemwide_payloads.md
grep -n "^### 7d|^### 6b|Host work items|^### 10\b" design/SPEC_wallet_policy_composer.md
sed -n '428,470p;806,830p;870,880p' design/SPEC_wallet_policy_composer.md

# fork /scratch/code/shibboleth/seedhammer @ 169073c (read-only)
git log --oneline -1
git show -s --format='%h %ci %s' 09c5f14 cde7545 5623824
sed -n '75,100p' gui/sysw_unload.go
grep -n "progWalletPolicy|walletPolicyFlow" gui/sysw_admit_oracle_test.go
sed -n '60,75p' gui/sysw_admit_oracle_test.go
sed -n '1,60p' sysw/record.go | grep -n "ClassMt|ClassTx|Class("
grep -n "^\tClass" sysw/record.go
grep -n "progWalletPolicy|progTransaction" gui/sysw_admit.go
```

Not re-derived, per the brief: the composer spec's own rulings, the
pre-existing gate results on this file (table/glyph/cite counts), and the
fork facts already measured at `169073c` in the R0 r0 report (re-verified
independently above rather than re-derived from scratch).
