# R0 round 1 — fold-verification pass (mechanical, not a fresh audit)

Checked: the fold responding to `design/agent-reports/mt-spec-R0-round0-{design,funds-safety,external-facts,coverage}.md` and `mt-spec-R0-round0-controller-triage.md`, against `design/SPEC_mt_v0_1.md`.

## IMPORTANT — the target moved during this check

The dispatch brief names two fold commits, `042cb65` and `1e74d4b`. **A third
commit, `fefe901` ("spec: plate layout for the string form is the user's, not
ours"), is now HEAD** (`git log --oneline -3`: `fefe901` → `1e74d4b` → `042cb65`).
It landed between my first read of the spec file and my second — i.e. mid-task,
committed by the operator/controller while this check was running. `git status`
is clean; this is not an uncommitted edit, it is a real commit on `master`.

`fefe901` deletes §3b's chars-per-plate table and the `mt string`-vs-`mt qr`
plate-count comparison (operator ruling: *"How many codex32 characters fit a
hand engraved plate? As many as a user wants. It is not our concern."*), closes
old open-question 11 as **OUT OF SCOPE** rather than **ANSWERED**, and moves the
legend-budget doubt box from the deleted §3b section into §5. It does **not**
touch §5's five fields, §6, §7, or §8.

Consequence for this report: I checked the spec as it exists at `fefe901`
(current `master` tip, 746 lines), since that is what "the current
`design/SPEC_mt_v0_1.md`" resolves to. Two things the brief calls "already
settled / known open" are stale as a direct result of this third commit, not an
error in the brief:
- The brief lists **§10.12** as a "KNOWN open" item. `fefe901` deleted it
  (verified: `grep -n "^12\."` inside §10 returns nothing; the section now runs
  1,2,3,4,5,6,7,8,9,10,11,14,13). It is now a **dangling reference** — see N-1.
- The brief lists **§10.11(b)** as "KNOWN open." `fefe901` replaced item 11's
  "ANSWERED, two residues (a)/(b)" text with "CLOSED — OUT OF SCOPE." The (b)
  residue no longer exists as text to be open.

Neither of these changes what the 40 R0 findings below verify against §5–§9
(unaffected by `fefe901`), but both feed directly into the "new defects" section.

## Verdict

**40 Critical/Important findings checked** (13C + 27I across the four lenses,
matching each report's own stated verdict counts).

| verdict | count | IDs |
| --- | --- | --- |
| FIXED | 13 | D-1, D-2, D-5, D-6, D-7, S-1, S-2, S-3, S-5, S-6, F-1, F-2, C-2 |
| PARTIALLY FIXED | 6 | D-4, S-8, I-4, I-6, I-7, I-8 |
| NOT FIXED | 11 | S-4, S-7, S-12, S-13, S-14, S-15, S-16, I-2, I-3, I-5, I-10 |
| DEFERRED (numbered open question) | 4 | D-3, C-1, C-3, I-1 |
| OBSOLETE (defect's section deleted) | 6 | D-8, S-9, S-10, S-11, I-9, I-11 |

**4 new defects found**, all Important, none Critical: N-1 (dangling §10.12
reference), N-2 (§4's mt-string-layout pointer is both mistargeted and stale),
N-3 (§7's Bearer mitigation is asserted for both verbs but unenforceable for
`mt string` under the new layout ruling — this is T-1's own defect class
recurring for the second verb), N-4 (§8 refusal 2's "the data needed to run it
always arrives with the payload" is PSBT-specific and false for `mt string`'s
raw-tx payload, despite §8's preamble claiming every refusal binds both verbs).

No Critical was left NOT FIXED. Of the 6 Criticals that were NOT already fixed
by earlier folds, 1 (S-4, no fee/balance refusal) is genuinely open and
unflagged; the funds-safety lens's other Criticals (S-1, S-2, S-3, S-5, S-6) are
FIXED, and the design/coverage lens's remaining Criticals (D-3, C-1, C-3) are
explicitly DEFERRED as blocking, numbered open questions rather than silently
dropped.

---

## Finding-by-finding

Severity as reported by the originating lens. Quotes are verbatim from the
current `design/SPEC_mt_v0_1.md` (line numbers as of `fefe901`).

### Lens 1 — design (`mt-spec-R0-round0-design.md`)

| ID | sev | one-line restatement | verdict |
| --- | --- | --- | --- |
| D-1 | C | §4's objective wasn't a total order; implicit tie-break ran toward the smallest, least legible module | **FIXED** |
| D-2 | C | §5's 5-field legend contradicted by §6a/§6c/§6d/§7, which still promised deleted fields | **FIXED** |
| D-3 | C | §4 selects a config the machine can't be told to cut; no section names the boundary artifact | **DEFERRED** (§10.9) |
| D-4 | I | No layering statement: mt-codec vs CLI vs firmware; boundary artifact unnamed; "manifest" undefined | **PARTIALLY FIXED** |
| D-5 | I | "k\*k tiling" isn't the tiling that produced §4's own table (2 qr, 6 qr aren't squares) | **FIXED** |
| D-6 | I | §4's "never leave redundancy unbought" contradicts §10.6, which asks whether to spend a plate on redundancy — "redundancy" means two different things | **FIXED** |
| D-7 | I | The objective's ECC-first term encodes an unstated damage model, and it's the wrong one for whole-plate loss | **FIXED** |
| D-8 | I | §8's refusals aren't scoped to a verb, so the §1a produce/present/engrave split leaks | **OBSOLETE** |

**D-1.** §4's objective now reads (lines 330–334):
> 1. minimise plates … 2. maximise ECC … 3. minimise symbol count … 4.
> TIE-BREAK: maximise MODULE SIZE … 5. then minimise QR version

with a correction box (340–358) explaining the old tie-break ran backwards and
citing the same 4-tied/41-tied numbers D-1 measured. This is a total order with
the tie-break running toward legibility, exactly D-1's request. Matches commit
`042cb65`'s claimed fold (T-6).

**D-2.** Old §6a/§6c/§6d are deleted outright (confirmed: `grep -n "§6c\|§6d"`
returns only §11's retrospective note at line 744, "Those sections are now out
of scope"). §7's table (545–551) now reads, e.g., "**not mitigated on the
plate**" for silent invalidation and "**cannot be fixed, and is NOT on the
plate**" for pinned fee — no row claims a field §5 doesn't carry. Matches T-1 in
the commit message.

**D-3.** §10.9 (685–692): *"How does the engraving reach the machine, and can
the machine engrave what §4 selects? … §4 may be selecting from a space the
machine cannot reach. **This blocks implementation and must close before
code.**"* Explicit numbered open question, marked blocking — DEFERRED per the
brief's own definition.

**D-4.** Three sub-claims, three different outcomes:
- "The manifest is undefined" — the word `manifest` now has **zero** occurrences
  in the spec (confirmed by grep against `fefe901`). §5's drop table replaced
  "survives in the manifest" language with `recoverable how` (line 452: *"fee
  rate and date | inputs − outputs, and the PSBT carries the input amounts"*).
  This sub-claim is moot because the concept it was about is gone — **OBSOLETE**
  in substance, folded into the PARTIALLY-FIXED verdict.
- "The artifact crossing the mt→machine boundary is never named" — still true,
  now explicitly DEFERRED via §10.9 (above).
- "Plate geometry has no named owner / does `mt` reuse the `me-preview` sidecar
  or re-derive geometry" — **still unaddressed**: `sidecar` and `me-preview` are
  zero occurrences in the current spec. §4 still re-derives its own plate
  constants (335–338) with no statement of relationship to `me bundle
  --preview`.

**D-5.** §4 line 329: `x rectangular tiling (across x rows)`, and the correction
box (342–346): *"The tiling is rectangular, not square. … `across x rows` is
what the reference implementation does."* Matches T-2.

**D-6.** §10 item 6 (668–670): *"~~How much fountain redundancy?~~ **CLOSED**,
operator ruling 2026-08-23: zero. `mt` protects against **plate damage** (ECC),
not **plate loss** (duplicate plates, the operator's choice)."* This explicitly
separates the two senses of "redundancy" D-6 flagged as conflated, and §3's
"Redundancy is zero" box (209–220) repeats the same split. §4's "never leave
redundancy unbought" (line 326) is no longer ambiguous once §10.6 has closed the
fountain-parts question to zero elsewhere.

**D-7.** §3 (217–220): *"§4 is right to spend leftover capacity on ECC. ECC
addresses marks, scratches and corrosion on a symbol, which is the failure this
artifact is being hardened against."* This is exactly the damage-model statement
D-7 said was missing, now written down alongside the explicit ECC/duplicate-plate
split.

**D-8.** Its citations (§1a, §8.3a, §8.3b, §6a's tier binding) no longer exist —
the whole four-tier amount ladder that D-8 was about is deleted. OBSOLETE by
subtraction. **But the same shape of defect recurs post-fold** — see N-4.

### Lens 2 — funds safety (`mt-spec-R0-round0-funds-safety.md`)

| ID | sev | one-line restatement | verdict |
| --- | --- | --- | --- |
| S-1 | C | Legacy inputs: sighash commits to no amount, so a fat-fingered `--i-certify-amounts` value pays away the input | **FIXED** |
| S-2 | C | Sighash flags never inspected; non-`ALL` makes the `TO` legend line a lie | **FIXED** |
| S-3 | C | Required locktime is inert when all inputs are final (`nSequence`) | **FIXED** |
| S-4 | C | Nothing checks `Σin − Σout`; the classic forgotten-change/absurd-fee loss is unguarded | **NOT FIXED** |
| S-5 | C | "No prevouts at all" tier passes every §8 refusal | **FIXED** |
| S-6 | C | §7/§6a/§6c/§6d promise legend content §5 doesn't carry | **FIXED** |
| S-7 | I | `include_mempool false` gives the opposite of the behaviour §6b argues for | **NOT FIXED** |
| S-8 | I | `gettxout` `null` conflates "spent" with "node can't see it" (syncing / wrong chain) | **PARTIALLY FIXED / DEFERRED** |
| S-9 | I | Tier 3 proves the previous tx exists, not that the outpoint ever confirmed | **OBSOLETE** |
| S-10 | I | `--i-certify-amounts` has no stated scope (one input vs whole tx) | **OBSOLETE** |
| S-11 | I | Tier evidence not carried into the presented PSBT, destroying the signer's independent check | **OBSOLETE** |
| S-12 | I | §8.4 can't be evaluated offline; a time-based locktime prints as an absurd block number | **NOT FIXED** |
| S-13 | I | Nothing checks dust / minimum relay feerate | **NOT FIXED** |
| S-14 | I | One `TO` line for N outputs; change back to the source wallet is unflagged | **NOT FIXED** |
| S-15 | I | The legend never says what the artifact *is* (no "Bitcoin transaction" / format-version field) | **NOT FIXED** |
| S-16 | I | Threat model starts at the finished plate; no payload-lifecycle or revocation story | **NOT FIXED** |

**S-1 / S-2.** §8 refusal 6 (604–611): *"Any input not signed with
`SIGHASH_ALL` (or taproot's `SIGHASH_DEFAULT`) → refuse. … `SIGHASH_SINGLE` and
`SIGHASH_ANYONECANPAY` are refused on the same grounds. **Additionally, any
legacy (non-segwit) input is refused**, because nothing in a legacy sighash
commits to the input amount…"* Both S-1's legacy-input hazard and S-2's
uninspected-sighash hazard are now a single named refusal. Matches "lens2
sighash" in the `042cb65` commit message.

**S-3.** §8 refusal 4 (592–600): *"Verifying 'actually timelocked' requires
reading `nSequence` … Under `--timelocked`, `mt` refuses unless **both** hold:
`nLockTime` is in the future, **and** at least one input is non-final."* Matches
T-4.

**S-4.** §8's nine refusals (566–618) contain no fee, balance, or
`Σin − Σout` check anywhere; §9 (626) only scopes out *choosing* a fee ("fee
estimation … are wallet decisions") which is a different claim from *refusing
an absurd one already chosen*. §6a still only checks unspentness, not value.
No open question in §10 names this gap either. S-4's original scenario (an
operator using `mt` itself to omit a change output) is now moot since `mt`
never builds transactions — but the underlying defect (a hand-built or
wallet-built transaction with a catastrophic fee reaches `mt` and nothing
refuses it) is unchanged and unmentioned.

**S-5.** §8 refusal 2 (572–578): *"The finalized PSBT carries each input's UTXO
record, so … **the data needed to run it always arrives with the payload.** A
PSBT whose UTXO records are missing is refused under (1)'s sibling rule: `mt`
requires the MIN form of §3."* The specific "raw hex, no prevouts, silently
passes" scenario S-5 described is closed for `mt qr` because the payload is now
mandatorily a MIN-form PSBT carrying UTXO records. (This refusal's applicability
to `mt string`'s raw-tx payload is a *different*, new problem — see N-4.)

**S-6.** Same evidence as D-2 above — §7's table no longer promises fields §5
doesn't carry, and §5's own drop table (448–452) states the correct recovery
path for each cut field.

**S-7.** Still explicitly open, in the spec's own words (526–531): *"**Known
limitation, from R0 lens 2, not yet resolved.** `false` also means a
mempool-spent input reads as *unspent*, which is the opposite of the caution
this section argues for."* This is disclosed but not fixed, and not filed as a
numbered §10 item (§10.5 covers a different sub-case — see S-8).

**S-8.** The "node still syncing" half of S-8 is explicitly deferred: §10 item 5
(664–667): *"Should `mt` require the node to be out of IBD before trusting
`gettxout`? §8.5's refusal cannot currently distinguish 'spent' from 'this node
does not know yet'…"* — DEFERRED for that half. The "wrong chain / signet node
answering confidently" half of S-8 (scenario B) has no mention anywhere
(`getblockchaininfo`, `chain`, `initialblockdownload` — zero hits) — that half
is **NOT FIXED**, hence the mixed verdict.

**S-9, S-10, S-11.** All three are about the deleted four-tier amount ladder:
S-9 about tier 3's "bound by txid but never confirmed" gap, S-10 about
`--i-certify-amounts`'s scope, S-11 about tier evidence not reaching a presented
PSBT. The tier system, the flag (`certify`: zero hits), and the `present` verb
(§9, line 625: *"PSBT presentation to a signing device — both removed"*) are all
gone. OBSOLETE.

**S-12.** §8 refusal 4 (581–600) still has no stated source for "the future" —
no chain-tip, no `--assume-height`, no IBD check referenced from this refusal —
and the legend field (line 407) is still literally `SPENDABLE AFTER BLOCK <n>`
with no representation for a Unix-timestamp-style `nLockTime` (≥500,000,000).
Neither of S-12's two scenarios (stale/offline tip, time-based locktime) is
mentioned anywhere in the current spec, including §10. (Distinct from S-3/T-4,
which is about the `nSequence` finality gap and is fixed — S-12 is about how
"future" gets decided at all.)

**S-13.** No dust or minimum-relay-feerate refusal exists; `dust`, `relay
policy`, `minrelay`, `standardness` are absent from §8. The nearest text (line
599, *"relay also depends on fee"*) is a caveat inside the `--immediate`
warning, not a check.

**S-14.** §5's legend table (403–409) still has exactly one `TO <truncated
addr> <amount>` line; §6 (481–486) still states destinations are "displayed,
never encoded" with no selection rule for N>1 outputs, and no mention anywhere
of flagging a change output that returns to the source wallet.

**S-15.** §5's five fields (405–409) are: bearer warning, wallet stub,
locktime, destination, plate index. None names the artifact type, "Bitcoin",
"transaction", or a format version (`grep -i "format version\|BITCOIN TX"`:
zero hits).

**S-16.** §7's table (545–551) has exactly four rows (Bearer, Pinned
destination, Pinned fee, Non-`ALL` sighash); none addresses the pre-engraving
lifecycle of the signed transaction (shell history, stdout, files, transport)
or names revocation (spending an input to void a copied plate) as the
operator's lever. §9 (623–637) likewise has no such item.

### Lens 3 — external facts (`mt-spec-R0-round0-external-facts.md`)

| ID | sev | one-line restatement | verdict |
| --- | --- | --- | --- |
| F-1 | C | `ur:bytes` is forbidden for production by BCR-2020-005 itself | **FIXED** |
| F-2 | I | "Sparrow/Keystone/Passport/Specter already read [UR]" overstates support — true only for `ur:psbt`, not the chosen `ur:bytes` | **FIXED** |

**F-1.** §1 ruling 4 and §3's correction box (127–141) rewrite the envelope to
`ur:psbt`, cite the exact BCR-2020-005 MUST-NOT sentence F-1 quoted, and record
having enumerated BCR-2020-006's 58 rows to confirm `psbt` is the only
transaction-shaped registered type. Matches T-3.

**F-2.** The overstated "already read"/"already consume" framing is gone —
`grep -n "Sparrow"` now returns exactly one hit, inside §10 item 2 (645–653),
which is the honest version of the claim: *"§3 does not currently claim
ecosystem readability for the multi-plate case"*, citing the same Sparrow
scan-failure report F-2 found, now used as a disclosed open risk rather than
supporting evidence for a claim.

### Lens 4 — coverage (`mt-spec-R0-round0-coverage.md`)

| ID | sev | one-line restatement | verdict |
| --- | --- | --- | --- |
| C-1 | C | No stated engraving path; §4 selects from a config space the machine can't reach | **DEFERRED** (§10.9) |
| C-2 | C | "k\*k tiling" contradicts the search that produced §4's own table | **FIXED** |
| C-3 | C | No CLI surface; §8's refusals don't say which verb they bind | **DEFERRED** (§10.10) |
| I-1 | I | `FROM WALLET <8 hex>` is mandatory with no specified input, no absent-case rule | **DEFERRED** (§10.4) |
| I-2 | I | Legend field templates unspecified: truncation rule, amount format, wrap/overflow | **NOT FIXED** |
| I-3 | I | One `TO` line, N outputs, no selection rule | **NOT FIXED** (= S-14) |
| I-4 | I | §8.4 refusal: no chain-tip source, no time-locked case, misses all-sequences-final | **PARTIALLY FIXED** |
| I-5 | I | The legend-only (zero-symbol) plate case is in the reference model, not in §4's prose | **NOT FIXED** |
| I-6 | I | §4's plate geometry (screw holes, `qrBorder`, real font pitch, stroke-quantized modules) isn't the machine's | **PARTIALLY FIXED** |
| I-7 | I | Recoverer's 2040 walk never written down: plate order, symbol order, plate↔fragment mapping, format id | **PARTIALLY FIXED** |
| I-8 | I | Bytes going into the UR unspecified: CBOR wrapping, bytewords style, QR mode/case | **PARTIALLY FIXED** |
| I-9 | I | `present` is a verb with no artifact | **OBSOLETE** |
| I-10 | I | No test vectors, no conformance surface | **NOT FIXED** |
| I-11 | I | "The manifest" is load-bearing and never specified | **OBSOLETE** |

**C-1.** Same evidence as D-3 — §10.9, marked "blocks implementation."

**C-2.** Same evidence as D-5 — §4's search space now reads `across x rows`
and the correction box calls out the exact defect C-2 named (2 qr / 6 qr rows
aren't squares).

**C-3.** §10 item 10 (693–698): *"**There is no CLI surface.** Two verbs (`mt
qr`, `mt string`) and two flags … are now named, but nothing specifies the
input convention … the output convention, or the exit codes … **Blocks
implementation.**"* Verb names and the two flags C-3 asked for now exist; I/O
convention and exit codes are explicitly DEFERRED, matching C-3's own
description of what's missing almost verbatim.

**I-1.** §10 item 4 (658–663): *"Where does `FROM WALLET <8 hex>` come from? It
is a mandatory legend field sized into §4's reservation, and nothing specifies
what supplies the md1 card, nor what the legend does when it is absent."*
Near-verbatim match to I-1's own wording. DEFERRED.

**I-2.** §5's `TO <truncated addr> <amount>` row (line 408) still gives no
truncation rule, amount-decimal rule, or wrap rule; `grep -i "truncat"` returns
only the field name itself, no rule. The `BEARER` field is still 41 characters
(line 405) against a stated ~35-char line width with no wrap specification.

**I-3.** Identical evidence to S-14.

**I-4.** Case 3 (all-sequences-final defeats an on-paper-future locktime) is now
explicitly fixed — see S-3 above, same refusal 4 text. Cases 1 (no chain-tip
source offline) and 2 (time-based `nLockTime` rendering) are unchanged and
unaddressed — see S-12. Mixed verdict because I-4 bundled a since-fixed case
with two that remain open.

**I-5.** §4's plate model (327–338) states the legend reservation ("6 lines
reserved on plate 1") and the objective, but nowhere states or accounts for the
case where the legend and a symbol don't co-fit plate 1 at all (a
zero-symbol first plate). No mention of this case anywhere in §4.

**I-6.** The font-pitch/4.25mm sub-issue is now disclosed and deferred: §5's
new box (411–434, moved there by `fefe901`) states *"§4's entire plate table
and this section's 6-line reservation both stand on it. Filed as §10.14…"* and
§10 item 14 (705–712) repeats it. But the screw-hole band and the fork's
additional `qrBorder` margin — `screw` and `qrBorder` are **zero** occurrences
in the current spec — are entirely unmentioned in §4's plate model (335–338:
`85 x 85mm, outerMargin 3mm => 79mm usable`, `quiet zone: 4 modules per side`,
nothing about a screw-hole band or a border beyond the quiet zone). Mixed
verdict: one of I-6's three sub-issues is filed, two are untouched.

**I-7.** The fountain-parameter-learning sub-issue is answered: §10 item 8
(676–684), *"~~How does a recoverer learn the fountain parameters?~~
**ANSWERED** from source by R0 lens 4…"* — this is lens 4's own answer, now
folded in. But I-7's other three sub-issues (plate order, symbol order within a
tiled plate, and the mapping between `PLATE n OF m` and the underlying UR
`seqNum`) are still nowhere in the spec, and the format-identifier gap is the
same open item as S-15.

**I-8.** The CBOR-wrapping ambiguity is effectively closed by the envelope
change itself: `psbt` is a registered UR type with a defined CBOR byte-string
form (unlike the ad hoc `bytes` type), so "does the message get CBOR-wrapped"
is now answered by the type choice rather than left to the implementer. The
bytewords-style and QR-mode/case questions remain phrased descriptively, not as
requirements — §3 (222, 224): *"Bytewords minimal is **exactly** 2 characters
per byte…"*, *"Uppercased, `ur:psbt/N-M/…` **is** fully QR-alphanumeric"* — read
as measurements of a specific encoding choice, not stated as a MUST that binds
an implementer. Mixed verdict.

**I-9.** §9 (625): *"Transaction construction, and PSBT presentation to a
signing device — both removed by operator ruling 2026-08-23."* The `present`
verb I-9 was about no longer exists. OBSOLETE.

**I-10.** `grep -c "test vector"` / `grep -c "\bvector\b"` / `grep -c
fixture`: all **zero** against current HEAD. (An earlier state of the file, at
`1e74d4b`, had one incidental mention inside the now-deleted §10 item 12 —
gone along with that item.) No vectors, no conformance directory, no fixture
of any kind exists in or is referenced from the spec.

**I-11.** Same evidence as D-4's manifest sub-point — `manifest`: zero
occurrences. OBSOLETE.

---

## New defects introduced by the fold

None of these were in the four round-0 reports. All four are internal
contradictions or dangling references of the kind the brief asked me to hunt
for specifically (§5/§7/§8/§10, the qr/string payload split).

### N-1 — §3b cites §10.12 for the fill-vs-balance chunking question; §10.12 no longer exists

**Severity: Important.**

`design/SPEC_mt_v0_1.md:292`:
> **A new `mt1` codec could choose to fill**, which would raise the ceiling —
> undecided, §10.12.

`fefe901` deleted the entire "12. Should `mt1` FILL its chunks rather than
balance them?" open question (visible in `git show fefe901`'s diff hunk, which
removes that block along with the old item 11 text and does not renumber or
relocate it). Section 10's current items run 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
14, 13 — no 12.

Positive control that the search scope is right: `grep -n "^12\."` inside the
`## 10. Open questions` block returns nothing, and `grep -n "§10\.1[0-9]"`
across the whole file finds every other `§10.1x` reference (§10.10, §10.11 —
wait, `§10.11` is not cited elsewhere — §10.12, §10.13, §10.14) resolving to a
real heading except this one. `§10.12` is the only `§10.N` citation in the
document that points at nothing.

This is exactly the class of defect the brief's own note about the "already
settled / known open" list flags: the brief itself (written before `fefe901`)
lists §10.12 as legitimately open. It no longer is — it's a broken pointer.

### N-2 — §4's pointer for "`mt string`'s layout" is both the wrong section and stale

**Severity: Important.**

`design/SPEC_mt_v0_1.md:316-319`:
> ## 4. Choosing the configuration — `mt qr` only
>
> **This section governs `mt qr` and nothing else.** `mt string`'s layout is
> undecided and is §10.10.

Two independent problems:
1. **Wrong target.** §10.10 (693–698) is entirely about CLI surface (verb
   names, flags, I/O convention, exit codes) — it contains no discussion of
   layout, font size, or plates. It was never about `mt string`'s layout.
2. **Stale claim.** `fefe901` (the ruling reviewed above) explicitly closed
   this question, and not as "undecided": §3b's new subsection (line 294,
   "Layout on steel is the user's, not `mt`'s") and §10 item 11 (699–703,
   "**CLOSED — OUT OF SCOPE**") both state the operator ruled it out of scope
   rather than left it open. §4's sentence still says "undecided" about a
   question the spec itself, three sections later, says is closed.

Positive control for the wrong-target claim: every other `§10.N` reference I
checked in §5, §6, §7, §8 (`§10.4`, `§10.5`, `§10.14`) points at a section that
actually discusses the topic named at the citing site — this is the one
citation where the target section's content doesn't match what the citing
sentence says it will find there.

### N-3 — §7's Bearer-hazard mitigation is asserted for both verbs, but `mt string`'s own layout ruling gives `mt` no way to guarantee it

**Severity: Important** (recurrence of T-1's defect class for the verb T-1
didn't originally cover).

§7's table preamble (542–543): *"**Every mitigation below names a field §5
actually engraves.**"* The Bearer row (547): *"…the `BEARER` line states it
plainly, and it **is the first line on the plate**."*

`fefe901`'s new §3b subsection (296–303) rules the opposite for `mt string`:
> **`mt string` emits a string. That is the whole of its output.** Font size,
> characters per plate, how many plates, what order they are laid out in,
> whether the string is cut by hand or by machine, and **whether anything is
> engraved beside it are all the user's decisions.** This spec does not
> constrain any of them…

So for `mt string`, `mt` has no mechanism that guarantees the `BEARER` warning
— or any of §5's five fields — appears on the plate at all, let alone as "the
first line." §7 makes an unqualified claim about "the plate" that only holds
for `mt qr`.

This is not a hypothetical reading: **the fold's own commit message for
`fefe901` says the same thing**, and says it is unresolved:
> the refusals reviewer because the ruling SHARPENS its question rather than
> removing it — `mt` has no mechanism to ensure a hand-cut plate carries the
> BEARER warning at all, while section 7 still claims that line as the bearer
> mitigation. That is now a real threat-model question for one of the two
> verbs, and it is theirs.

The commit message identifies the gap and defers it to "the refusals
reviewer" — but nothing in the spec text itself (§7's table, its preamble, or
§10) records this as an open question or qualifies the Bearer row to `mt qr`
only. A reader of the spec alone has no way to discover this; only the commit
message says so. Recommend either scoping §7's preamble/Bearer row to `mt qr`
explicitly, or filing it as a numbered §10 item the way every other
fold-introduced gap in this document is filed.

### N-4 — §8 refusal 2 claims prevout data "always arrives with the payload," which is false for `mt string`

**Severity: Important.**

§8's preamble (562–564): *"**Every refusal below binds BOTH verbs** unless it
names one — a hand-engraved plate is exactly as bearer, and exactly as
permanent, as a machine-engraved one."* Refusal 2 (572–578):
> **Script-invalid** → refuse. Real libbitcoinconsensus verification: … The
> finalized PSBT carries each input's UTXO record, so … **the data needed to
> run it always arrives with the payload.** A PSBT whose UTXO records are
> missing is refused under (1)'s sibling rule: `mt` requires the MIN form of
> §3.

§3b (261–262) states the opposite payload shape for the other verb: *"**The
payload is the raw signed transaction, NOT the PSBT** — deliberately…"* A raw
signed transaction carries outpoints (`txid:vout`) but **no UTXO records**
(value + scriptPubKey) — those exist only in a PSBT's `PSBT_IN_WITNESS_UTXO` /
`PSBT_IN_NON_WITNESS_UTXO` fields, which `mt string`'s payload, by its own
definition two paragraphs earlier, does not have.

So refusal 2's justification — "the data needed to run it always arrives with
the payload" — is true for `mt qr` and false for `mt string`. Either
libbitcoinconsensus verification silently cannot run for `mt string` (meaning
§8's own claim that "this section now carries the whole safety argument," line
560, is false for one of the two verbs on its one substantive cryptographic
check), or `mt string` needs a separately-specified prevout input the spec
never names. This is the same shape of defect design-lens finding D-8 flagged
in the pre-fold spec (refusals not scoped to a verb) — OBSOLETE against its
original citations (§1a, §8.3a/b, all deleted), but effectively reintroduced by
the new qr/string split that replaced them. (Refusal 1's wording,
`PSBT_IN_FINAL_SCRIPTSIG`/`PSBT_IN_FINAL_SCRIPTWITNESS`, has the same
PSBT-specific phrasing problem, but the underlying property — every input
carries real signature data — is structurally guaranteed for any raw
transaction that parses at all, so that one is a wording gap rather than a
capability gap; noted here for completeness, not scored separately.)

---

## Coverage note

Per the brief: citation gate (`./scripts/plan-cite-check.sh`) re-run against
current HEAD — 23/27 resolved, 4 dangling, all 4 the same cross-repo citations
already verified by hand (not re-resolved here). That gate checks code
citations only; it does not check internal `§N` cross-references, which is why
N-1 and N-2 are not things the gate would have caught.
