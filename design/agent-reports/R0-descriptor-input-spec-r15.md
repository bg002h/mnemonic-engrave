# R0 round 15 — the r14 fold, and the CLOSURE decision table

**Target:** `design/SPEC_descriptor_input.md` at `f1dd328` ("spec: fold R0 r14 --
admission precedes the window; substitution semantic"). `f1dd328` is HEAD; tree
clean (`git status --short` empty).
**Source of truth for what was required:**
`design/agent-reports/R0-descriptor-input-spec-r14.md` (0C/1I/3M/1N).
**Scope:** (1) fidelity of the fold to r14's five findings and defects the fold
itself introduced; (2) the full closure decision table over the five interacting
rules. r1–r14 measured results, sweeps, walk log, citation gate, rulings and
dispositions taken as settled.
**Reviewer:** independent context, opus tier. Read-only; nothing modified,
committed or pushed.
**Diff read in full:** `git show f1dd328` — one file, +27/−11, six hunks. Every
hunk dispositioned below.
**Binaries used:** `/home/bcg/.cargo/bin/me` (`me 0.7.0`), `cargo nextest`.

---

## Counts — NEW findings

| severity | count |
| --- | :-: |
| **Critical** | **0** |
| **Important** | **4** |
| Minor | 2 |
| Nit | 1 |

**The spec does NOT re-close GREEN this round, and the walk lens is not
complete.**

**Table verdict: NOT all cells single-valued.** Of the 60 cells derived below:

- **54 are single-valued** on tier, follower, exit code and §6 row.
- **4 have two derivable answers for the FOLLOWER and the EXIT CODE.**
  Classes 2 and 5 under `--as descriptor` in the S3-only window: §5.4 L1087–1091
  says *"its refusal is the window's, not admission's"* and §5.4 L1096–1099 says
  a full-tier `wsh(multi(…))` under `--as descriptor` *"meets a conjunct-1
  admission refusal"* — same section, nine lines apart, opposite answers
  (**new-I3**). Class 9 under `--as` omitted, either build: `EXIT_REFUSED` (3)
  with §6's descriptor text, or the shipped `EXIT_INVALID` (4) (**new-I4**).
- **2 are single-valued on the follower but two-valued on a sub-detail** —
  classes 1 and 2, window, `--as` omitted: the build-dead `--as descriptor` is
  marked (M6) or unmarked (the new one-carries ruling) (**new-M1**).
54 + 4 + 2 = 60. Separately, and cutting across that partition, **6 cells
resolve to text that is FALSE about the input or carries internal identifiers**:
class 5's four `--as md1`/omitted cells (**new-I1**; two of them also
**new-I2**), plus classes 2 and 5 under `--as descriptor` in the window on the
window reading of **new-I3**.

Two of this round's four Importants are **fold-introduced** (I1, I2) and sit in
the same clause the fold rewrote. One (I3) is the residue of r14's own new-I1 in
the one place the fold's ordering rule cannot see. One (I4) is a first-time
question: the table's non-descriptor boundary row, which no prior round asked.

**Good news for the cycle's record, and it is real: r14's central finding is
correctly and completely fixed for every class r14 constructed.** The ordering
sentence is right, it is well-motivated, and it silently closes a sixth cell
nobody had raised (an unparseable input under `--as descriptor` in the window —
see "Verified in passing"). The two fold-introduced Importants are both in one
sentence, both mechanical, and both one edit each.

---

## Disposition of r14's five

| r14 finding | verdict | evidence |
| --- | :-: | --- |
| **new-I1** (the window refusal's variant selector has no arm for an INADMISSIBLE input; nothing orders it against the admission refusal) | **FIXED for every class r14 constructed; NOT fixed for the one path-dependent conjunct** | The ordering paragraph (L838–844) states it: *"the §4.7 admission refusal PRECEDES the window refusal … The window refusal fires only for wallets at least one path admits, and its two arms partition exactly those, by md1-representability"*. Re-run of both r14 constructions below: both clean, both single-valued, no window text at any site. **But the rule quantifies over BOTH paths** (`no path admits` vs `at least one path admits`), and §4.7 has exactly one PATH-DEPENDENT conjunct — conjunct 1, the `multi` twins. A `multi` under `--as descriptor` fails conjunct 1 on the descriptor path while md1 admits it, so it is *"a wallet at least one path admits"* and the new rule does not disambiguate it. §5.4 then answers that cell **twice, nine lines apart, in opposite directions** (L1087–1091 the window's refusal; L1096–1099 a conjunct-1 admission refusal), and the window answer's arm is false about the input. Re-filed as **new-I3**. |
| **new-M1** (§5.1 cited item 5's "two tested cases" in the same commit that made it three) | **FIXED** | L818 now reads *"matching §11 item 5's tested cases across all four formats"* — the count is removed rather than corrected, which is the fix that cannot go stale again. §11 item 5 now says *"Four cases tested"*. `grep -n "item 5's.*case"` → one hit, no number. |
| **new-M2** (the absolute *"the choice text itself never offers a dead flag"* is false for the exactly-one-carries case) | **FIXED as a ruling; the ruling's own wording re-opens a smaller version of it** | The fold qualified the absolute to *"never offers a BUILD-dead flag unmarked"* (L768) and added the deliberate ruling at L770–775. The three cases are now disjoint by intent and the justification is sound (*"the operator who picks the input-dead value gets that path's own refusal, which names the working flag"* — true at both exemplar sites, checked below). The residue: the ruling's trigger is stated as *"exactly ONE value carries this particular input"*, and §5.4 defines "carries" to include the window — so in the S3-only window the trigger fires and instructs *"offers both, unmarked"*, against M6's *"marks that value inline"*. Filed **new-M1** (Minor: both sentences carry their own qualifier — BUILD-dead / input-dead — so a careful reader gets one answer). |
| **new-M3** (the `multi` + `/0/*` remedy's `sortedmulti` half is not executable in the window; the substitution is keyed lexically) | **FIXED in mechanism, BROKEN in execution** | §5.3 L1016–1018 is now semantic: *"every remedy … that ROUTES TO the descriptor path — naming the flag or otherwise (semantic, not lexical, per R0 r14's new-M3)"*. Verified the widening reaches **exactly one** new site and over-applies nowhere (sweep below). But the fold's two edits at the §6 row it was aimed at both misfire: the annotation was written INSIDE the operator-visible quote (**new-I1**), and the substitution it now triggers produces a false, ungrammatical tail (**new-I2**). |
| **new-N1** (r11 dropped from §5.4's provenance line) | **FIXED** | L1071–1073: *"(walk W13; R0 r9 I1, r10 new-I1, **r11 new-M1/M2/N1**, r12–r13: the three rules below, each stated once)"*. All three r11-originated pieces still present at L1077–1091. |

**Fidelity: 3 of 5 closed outright; 1 closed as a ruling with a Minor residue;
1 closed for its constructed classes but not for the class its remedy cannot
reach.**

### Re-run of r14's new-I1 construction A — hardened use-site, `--as descriptor`, S3-only window

**Result: the §4.7 admission refusal, at every site. No window text anywhere.
CLOSED.**

Input `wsh(sortedmulti(2, [fp/48h/0h/0h/2h]xpub…/<0;1>/*h,
[fp2/48h/0h/0h/2h]xpub…/<0;1>/*h))`, `me sysw pack --in wallet.txt --as descriptor`,
S3-only build.

| site | says | agrees? |
| --- | --- | :-: |
| §5.1 L838–841 (ordering) | *"A wallet no path admits gets its admission refusal regardless of flag and build"* | ✅ ADM |
| §5.1 L842–844 (window scope) | *"The window refusal fires only for wallets at least one path admits"* — conjunct 7 is path-independent, so no path admits | ✅ ADM |
| §5.4 L1075–1080 (TIER) | fails 2–7 → PARTIAL block | ✅ PARTIAL + ADM |
| §5.4 L1101–1112 (CARRIAGE) | governs `--as` omission only; not this invocation | ✅ (silent, correctly) |
| §6 L1261 (hardened row) | *"refused on both `--as` paths"* | ✅ ADM |
| §6 L1240 (window row) | now scoped out by L842–844 | ✅ does not fire |
| §11 item 5 case 4 | *"inadmissible with explicit `--as descriptor` in the window — the admission refusal, never the window text"* | ✅ ADM |

Message: §6 L1261's *"a hardened use-site step cannot be derived from an xpub
(BIP-32). The device would silently derive the UNhardened child and display
addresses for a wallet that cannot exist, so this is refused on both `--as`
paths."* at `EXIT_REFUSED` (3). r14's "one of them is false whichever wins" is
discharged: only one fires.

The same run is clean for every class r14 listed — non-consecutive multipath,
`/0/1/*`, bare fixed index, `wsh(KEY)`/`sh(KEY)`, the single-key wrappers, mixed
network, 21 keys, version bytes. All eight fail path-INDEPENDENT conjuncts, so
`no path admits` holds for all eight and L838–841 selects ADM for all eight.

### Re-run of r14's new-I1 construction B — `sortedmulti(0,…)`, `--as descriptor`, window

**Result: the anyone-can-spend admission refusal, directly. CLOSED, and the
fold's own reasoning names it.**

`wsh(sortedmulti(0, K1/<0;1>/*, K2/<0;1>/*))` fails conjunct 2 (path-independent)
→ no path admits → ADM at (3) → §6 L1248: *"threshold 0 means NO signature is
required: anyone who can see this script can spend from it… if it already holds
funds, treat them as at risk now."* The ordering paragraph cites this exact input
as its motivation (L840–842: *"`sortedmulti(0,…)` must hear "treat those funds as
at risk now", never "nothing is lost by waiting""*). Arm 1's *"nothing is lost by
waiting"* is unreachable for it. r14's funds-relevant stake is discharged.

### The semantic substitution — does it catch the `sortedmulti`-alternative clause, and does it over-apply?

**Catches it: YES. Over-applies: NO.** I enumerated every remedy in §5.3 and §6
and asked "does this route the operator to the `--as descriptor` output path?":

| remedy site | routes to descriptor path? | substituted in the window? |
| --- | :-: | :-: |
| §5.3(a), §5.3(a″) NORMATIVE blocks — *"names … `--as descriptor`, which carries that shape exactly"* | yes, by name | yes (lexical already) |
| §6 L1246 — *"Use `--as descriptor`, which carries `/0/*` exactly."* | yes, by name | yes (lexical already) |
| §6 L1264 — *"Use `--as descriptor`, which carries `<0;1>` exactly."* | yes, by name | yes (lexical already) |
| §6 L1246's `multi` replacement — *"or as a `sortedmulti` policy…"* | **yes, without naming the flag** (`sortedmulti` + `/0/*` is `--as descriptor`-only, §5.5 L1169) | **yes — NEW, the fold's target** ✅ |
| §6 L1244 `multi` row — *"`--as md1` encodes `multi` policies"* | no, routes to md1 | no ✅ |
| §6 version-bytes row — *"supply the full multisig descriptor: `sh(wsh(sortedmulti(…)))`…"* | no — a corrected descriptor STRING, carried by both paths | no ✅ |
| §6 bare-key rows — *"Supply the descriptor instead: `tr([…]xpub…/<0;1>/*)`"* | no — `tr(KEY)` is ✅/✅ in §5.5 | no ✅ |
| §6 single-key-wrapper and `wsh(KEY)`/`sh(KEY)` rows | no — the named forms are carried by both | no ✅ |

`grep -n 'descriptor path'` returns four in-spec uses (L766, L1016, L1017, L1246)
plus `me seal`'s at L1608; all four use it for the `--as descriptor` output path,
so the term the semantic rule keys on is consistently defined. The widening lands
on exactly the one clause it was written for. **Mechanism: closed.** Its two
execution defects are new-I1 and new-I2 below.

### Does the one-carries ruling compose with M6 and the carriage rule?

Three cases, and the fold intends them disjoint:

| case | rule | block? | marking |
| --- | --- | :-: | --- |
| descriptor path not shipped (build-dead) | M6, L766–769 | fires if something carries | `--as descriptor (not available in this build)` |
| neither value carries | carriage, L770–772 + §5.4 L1101–1112 | **does not fire** | n/a |
| exactly one carries because the other refuses THIS input | new, L772–777 | fires | both offered, **unmarked** |

Composition is sound wherever the cases are disjoint, and the ruling's
justification checks out at both exemplar sites: `wsh(multi(…/<0;1>/*))` with
`--as` omitted in a full build offers `--as descriptor`, and the operator who
picks it gets §6 L1244, which names `--as md1` ✅; `sortedmulti(…/0/*)`
post-window offers `--as md1`, and the operator who picks it gets §6 L1246,
which names `--as descriptor` ✅. Both refusals name the working flag, so the
journey rule's bar is genuinely not met and the ruling is right.

**Where they are not disjoint: the S3-only window.** There `--as descriptor` is
build-dead for every input, so for any md1-carried input BOTH case 1 and case 3
hold, and they give opposite marking instructions. Filed **new-M1**.

### The two text fixes

- **new-M1's citation:** fixed by deletion of the count, not by correcting it ✅.
- **new-N1's provenance:** r11 restored ✅.

---

# THE DECISION TABLE

## How each cell was derived

**The five rules, in the order they compose:**

1. **Cascade + cause rule** (§4.1, §6 L1195–1206) — runs first; no parse means no
   tier block and no `--as`-dependent follower.
2. **TIER** (§5.4 L1075–1091) — FULL iff conjuncts 2–7 hold AND ≥1 `--as` path
   admits the shape; PARTIAL otherwise. Flag-independent, build-independent.
3. **CARRIAGE** (§5.4 L1101–1112) — governs `--as` OMISSION only. Block at
   `EXIT_USAGE` (2) iff ≥1 value carries in this build; else the input's own
   refusal directly at (3).
4. **ORDERING** (§5.1 L838–844, new) — ADM precedes the window refusal; the
   window refusal fires only for wallets ≥1 path admits.
5. **FOLLOWER independence** (§5.4 L1093–1099) — tier picks lines, not outcomes.

**Notation.** Tier: `F` FULL block, `P` PARTIAL block, `—` no block.
Follower: `PACK`, `BLOCK` (§5.1 choice block), `WIN1`/`WIN2` (window refusal arm
1 / arm 2), `ADM` (§4.7 admission refusal), `5.3a`/`5.3a″`/`5.3m` (§5.3 refusal;
`m` = the `multi` replacement remedy), `CASC` (§6 unparseable row), `MREC` (§6
multi-record row), `NOREC` (shipped no-records refusal).
Build: **FULL** = both paths shipped; **WIN** = S3-only window.

## The 10 input classes

| # | class | exemplar |
| :-: | --- | --- |
| 1 | carried by both | `wsh(sortedmulti(2,K1/<0;1>/*,K2/<0;1>/*))` |
| 2 | md1-only, `multi` well-formed | `wsh(multi(2,K1/<0;1>/*,K2/<0;1>/*))` |
| 3 | descriptor-only, (a) | `wsh(sortedmulti(2,K1/0/*,K2/0/*))` |
| 4 | descriptor-only, (a″) | `wsh(sortedmulti(2,K1/<0;1>,K2/<0;1>))` |
| 5 | admitted, carried by NEITHER | `wsh(multi(2,K1/0/*,K2/0/*))` |
| 6 | inadmissible — underivable | `wsh(sortedmulti(2,K1/<0;1>/*h,K2/<0;1>/*h))` |
| 7 | inadmissible — unspendable | `wsh(sortedmulti(3,K1/<0;1>/*,K2/<0;1>/*))` |
| 8 | inadmissible — anyone-can-spend | `wsh(sortedmulti(0,K1/<0;1>/*,K2/<0;1>/*))` |
| 9 | non-descriptor, non-record | a mistyped BIP-39 word; `hello world` |
| 10 | multi-record incl. a descriptor | mnemonic + descriptor, two lines |

## The table

| # | `--as md1`, FULL | `--as descriptor`, FULL | omitted, FULL | `--as md1`, WIN | `--as descriptor`, WIN | omitted, WIN |
| :-: | --- | --- | --- | --- | --- | --- |
| **1** | F · PACK · **0** · — | F · PACK · **0** · — | F · BLOCK · **2** · L1235 | F · PACK · **0** · — | F · WIN1 · **3** · L1240 | F · BLOCK · **2** · L1235 **⚠M1** |
| **2** | F · PACK · **0** · — | F · ADM(c1) · **3** · L1244 | F · BLOCK · **2** · L1235 | F · PACK · **0** · — | F · **WIN1 3 (L1240) or ADM(c1) 3 (L1244)** **⚠I3** | F · BLOCK · **2** · L1235 **⚠M1** |
| **3** | F · 5.3a · **3** · L1246 | F · PACK · **0** · — | F · BLOCK · **2** · L1235 | F · 5.3a+sub · **3** · L1246 | F · WIN2 · **3** · L1240 | F · WIN2 · **3** · L1240 |
| **4** | F · 5.3a″ · **3** · L1264 | F · PACK · **0** · — | F · BLOCK · **2** · L1235 | F · 5.3a″+sub · **3** · L1264 | F · WIN2 · **3** · L1240 | F · WIN2 · **3** · L1240 |
| **5** | F · 5.3m · **3** · L1246 **⚠I1** | F · ADM(c1) · **3** · L1244 | F · 5.3m · **3** · L1246 **⚠I1** | F · 5.3m+sub · **3** · L1246 **⚠I1 I2** | F · **WIN2 3 (L1240) or ADM(c1) 3 (L1244)** **⚠I3** | F · 5.3m+sub · **3** · L1246 **⚠I1 I2** |
| **6** | P · ADM(c7) · **3** · L1261 | P · ADM(c7) · **3** · L1261 | P · ADM(c7) · **3** · L1261 | P · ADM(c7) · **3** · L1261 | P · ADM(c7) · **3** · L1261 | P · ADM(c7) · **3** · L1261 |
| **7** | P · ADM(c2) · **3** · L1247 | P · ADM(c2) · **3** · L1247 | P · ADM(c2) · **3** · L1247 | P · ADM(c2) · **3** · L1247 | P · ADM(c2) · **3** · L1247 | P · ADM(c2) · **3** · L1247 |
| **8** | P · ADM(c2) · **3** · L1248 | P · ADM(c2) · **3** · L1248 | P · ADM(c2) · **3** · L1248 | P · ADM(c2) · **3** · L1248 | P · ADM(c2) · **3** · L1248 | P · ADM(c2) · **3** · L1248 |
| **9** | — · CASC · **3** · L1232 | — · CASC · **3** · L1232 | — · **CASC 3 or NOREC 4** **⚠I4** | — · CASC · **3** · L1232 | — · CASC · **3** · L1232 | — · **CASC 3 or NOREC 4** **⚠I4** |
| **10** | — · CASC · **3** · L1232 **⚠note** | — · CASC · **3** · L1232 **⚠note** | — · MREC · **4** · L1265 | — · CASC · **3** · L1232 **⚠note** | — · CASC · **3** · L1232 **⚠note** | — · MREC · **4** · L1265 |

Empty file / whitespace-only, all six columns: `— · NOREC · 2 · L1233/L1234`.
**Measured, not derived** (see below): rc=2, 0 stdout bytes, both inputs.

⚠ markers: **I1–I4 / M1** are this round's findings. **⚠note** is recorded, not
filed — with `--as` present a multi-record file is read whole, so it gets the
unparseable-file refusal rather than a message naming the record split; truthful,
so no finding, but it is a plausible loop (the multi-record row's own remedy
sends the operator there) and belongs in a plan-phase note.

## Cell-by-cell agreement check

For every cell above I resolved §5.1, §5.3, §5.4, §6 and §11 independently and
compared. Sections agree on **56 of 60**. The exceptions:

**Class 1 & 2 × WIN × omitted (⚠M1).** §5.4's carriage rule and §6 L1235 agree
the block fires at 2 (md1 carries). §5.1 L766–769 (M6) says `--as descriptor` is
marked `(not available in this build)`. §5.1 L772–777 says the block *"still
offers both, unmarked"* because exactly one value carries. Two derivable answers
for the marking; one for tier, follower and exit code.

**Class 2 × WIN × `--as descriptor` (⚠I3).** Two derivable answers, from two
sentences in the same subsection:

- §5.4 L1087–1091 (TIER paragraph): *"A `multi` input in the window is FULL-tier:
  **its refusal is the window's, not admission's**"* → WIN1, §6 L1240.
- §5.4 L1096–1099 (FOLLOWER paragraph): *"(A full-tier `wsh(multi(…))` under
  `--as descriptor` **meets a conjunct-1 admission refusal**…)"* — unqualified by
  build, and in the window a `multi` under `--as descriptor` is full-tier → ADM,
  §6 L1244.

§5.1 L842–844 permits the window branch (md1 admits the shape) and does not
forbid the admission branch. §6 L1240 and §6 L1244 both fire from their own
checks. **Nothing in the spec selects between them**, and the two messages say
different things — one names `sortedmulti` vs `multi` and the working flag, the
other promises a QR plate that will never exist for this shape.

**Class 5 × WIN × `--as descriptor` (⚠I3).** Same two-way split, with arm 2 in
place of arm 1 and the same permanent falsity in its closing sentence.

**Class 9 × omitted (⚠I4).** §5.1 L806–812 sends the input into the cascade; §6
L1195–1206 step 5 selects the unparseable row; §6 L1192 makes it `EXIT_REFUSED`
(3). The shipped surface — **measured, and pinned by a green test** — is
`EXIT_INVALID` (4) with a different message. §6 L1233's own principle (*"records
the existing behaviour rather than silently regressing a tested surface"*) points
the other way. Two derivable answers, and the spec never chooses.

**Everything else agrees**, including the six cells of class 6 and the six of
class 7/8, which are flag- and build-invariant exactly as the ordering paragraph
now says.

## What is MEASURED in the table rather than derived

`me 0.7.0` has no `--as` flag (`me sysw pack --as md1 …` → `error: unexpected
argument '--as' found`, rc=2), so only the pre-parse rows have a shipped surface.
All five were run at `f1dd328`:

| input | rc | stdout bytes | stderr |
| --- | :-: | :-: | --- |
| empty file, `--in` | **2** | 0 | `me: no records in <file>: pass them on argv, with --in, or on stdin. An EMPTY input is what a FAILED upstream command leaves behind…` |
| whitespace-only file, `--in` | **2** | 0 | identical message |
| single descriptor line, `--in` | **4** | 0 | `record 0 … is not a form this container can place: … Descriptors and addresses are not yet classifiable here — see sysw::classify` |
| mnemonic + descriptor, `--in` | **4** | 0 | same, `record 1` |
| 14-line BlueWallet file, `--in` | **4** | 0 | same, `record 0` |
| mistyped BIP-39 word (`…abandoz`), `--in` | **4** | 0 | same, `record 0` |
| `hello world`, `--in` | **4** | 0 | same, `record 0` |

§6 L1233/L1234's `EXIT_USAGE` (2) and 0-stdout claims: **confirmed**. §6 L1265's
*"`EXIT_INVALID` (4), as today"*: **confirmed**. §5.1 L800–803's claim that a
BlueWallet file via `--in` dies on `record 0`: **confirmed**. The class-9 rows'
shipped answer (rc 4, record-vocabulary message): **confirmed**, and it is the
measurement new-I4 turns on.

---

# NEW findings

## new-I1 (Important) — the fold wrote three internal identifiers INSIDE an operator-visible refusal quote, against §6's NORMATIVE walk-W5 rule, in the same file r14 had just swept clean

**Where.** §6 L1246, the `multi`-form replacement remedy, inside the `*"…"*`
span. The span now ends:

> …*(a DIFFERENT policy — `me` will not rewrite it; this alternative needs the
> descriptor path, so the window substitution applies to it, r14's new-M3)."*

The closing `"*` comes **after** `r14's new-M3)`, so `window substitution`,
`r14` and `new-M3` are all inside the operator-visible text.

**What it violates.** §6 L1217–1221, NORMATIVE: *"Every quoted text below: leads
with the verdict; contains NO internal identifiers — no phase labels, no
F-numbers, no spec § references inside the quotes (those live in the row's
annotation, outside the quotes)"*. And §5.1 L829's *"contains no internal phase
labels (walk W5)"*, the rule the operator's own verdict ("convoluted") produced.
R0 r11's new-I1 was filed at Important severity for exactly this shape — a
directive left inside a verbatim span.

**Machine-verified, both directions.** I extracted all 45 `*"…"*` spans
multi-line-aware and searched them for `§\d|F-\d{3}|R0|NEW-[A-Z]\d|new-[A-Z]\d|
walk W\d|conjunct \d|EXIT_|r1[0-5]|carriage rule|window substitution`:

- at `f1dd328`: **one span matches**, L1246, with three hits
  (`['window substitution', 'r14', 'new-M3']`).
- at `b331c77` (pre-fold): `grep -c "r14's new-M3"` → **0**. The fold introduced
  it.
- r14's identical sweep at `b331c77` returned 0 hits and is confirmed still
  correct; the only other matches at `f1dd328` are four `BIP-3\d\d` citations
  (L985, L1232, L1249, L1252), which walk W9 requires — *"cite the BIPs, an
  authority the operator can check"* — and which r14's pattern did not include
  either. Span count unchanged at 45.

**The operator-visible consequence.** A `multi` + `/0/*` export — class 5, a cell
reached in **four of six** build/flag combinations — prints a refusal ending
`…so the window substitution applies to it, r14's new-M3).` §11 item 4 requires
*"a test that reaches it and asserts the **text**"*, so a conformant
implementation compiles that string into the binary and a conformant test pins
it.

**The fix is one edit and the row already has the right home for it:** the
annotation site `(R0 r5's NEW-I2.)` sits immediately outside the quote.

## new-I2 (Important) — the substitution the fold newly routes to that clause produces a tail that is ungrammatical and FALSE, and contradicts the same refusal's own second sentence

**Where.** §6 L1246's `multi` replacement, under §5.3 L1015–1020's window
substitution, in the S3-only window.

**Derivation.** §5.3's rule replaces the routing **clause** with *"the
scannable-plate path is not in this build — keep the export file; it packs when
the device update ships."* The fold's annotation says this clause is now in
scope. Substituting, the operator holding `wsh(multi(2,K1/0/*,K2/0/*))` in the
window reads:

> *"this is a `multi` policy, which only `--as md1` carries — and md1 cannot
> represent `/0/*`. No `me` path engraves this descriptor this release;
> re-export with `<0;1>/*` (works in every build), **or the scannable-plate path
> is not in this build — keep the export file; it packs when the device update
> ships.**"*

**Two defects, one clause.**

1. **False.** *"keep the export file; it packs when the device update ships"* is
   false for the file the operator is holding. `--as descriptor` refuses `multi`
   in **every** build — §5.5 L1171 marks it *"❌ device refuses (§4.3)"* with no
   build qualifier, and §10 declines the widening on the record, permanently
   (*"Widening the host's DESCRIPTOR-record path would break §7's invariant"*).
   `--as md1` refuses `/0/*` in every build. **No build ever engraves this file.**
   The same sentence says so eleven words earlier: *"No `me` path engraves this
   descriptor this release"*. The substituted text contradicts it and tells the
   operator to wait.
2. **Incoherent.** *"keep the export file"* is the opposite instruction to
   *"re-export"*, which is the remedy it was substituted into, and the
   replacement is a full independent clause dropped into an `or …` slot.

**Why this is Important and not the Minor r14 filed.** r14 rated the pre-fold
state Minor because *"the executable remedy is first in the same sentence and the
wasted path ends in variant 2's honest refusal"* — the operator who took the
`sortedmulti` route simply got refused again. The fold changed the outcome class:
the operator is now told, on the authority of the refusal itself, that the file
they hold packs after a future update. The action that produces is **do nothing
and wait**, for a file that will never pack. By the journey rule that is worse
than silence — the pre-fold text at least ended in a refusal that told the truth.

**The substitution is not wrong; its text is wrong here.** The replacement clause
was written for a site where the wallet genuinely does pack once the descriptor
path ships (§5.3(a)'s `sortedmulti` + `/0/*` — §5.5 L1169 ✅). Class 5 is the one
site where it is false, and it is the only site the fold newly bound it to.

## new-I3 (Important) — the ordering rule quantifies over BOTH paths, so it cannot see §4.7's one PATH-DEPENDENT conjunct; §5.4 then answers that cell TWICE, nine lines apart, in opposite directions — and the window answer's promise is permanently false

**Where.** §5.1 L838–844 (the new ordering paragraph), §5.4 L1087–1091 (the
`multi`-in-the-window parenthetical) and §5.4 L1096–1099 (the FOLLOWER
paragraph's own `multi` example), §6 L1240 (the window row) against §6 L1244
(the `multi` row), §5.5 L1171, §10 L1596's `multi` bullet.

**The gap.** The ordering paragraph's two branches are *"a wallet **no path**
admits"* → ADM, and *"wallets **at least one path** admits"* → the window
refusal. That partition is a quantification over both paths, and it is exactly
right for conjuncts 2–7, which §4.7 makes path-independent. **Conjunct 1 is not
path-independent** — it is the one conjunct with an md1-path widening (*"on the
`--as md1` path ONLY, the three `multi` twins"*). A `multi` therefore fails
admission on the descriptor path while satisfying *"at least one path admits"*,
so the new rule leaves it in the window branch and does not disambiguate it.

**And §5.4 already answers this exact cell twice, in opposite directions.** Both
sentences are parentheticals, both survived the fold untouched, and they are nine
lines apart:

| L1087–1091, in **The TIER** | L1096–1099, in **The FOLLOWER** |
| --- | --- |
| *"(A `multi` input in the window is FULL-tier: **its refusal is the window's, not admission's** — the wallet is derivable and spendable, and md1-packable when its use-site paths are md1-representable…)"* | *"(A full-tier `wsh(multi(…))` under `--as descriptor` **meets a conjunct-1 admission refusal**; the two prior attempts to couple tier to follower each manufactured a false claim.)"* |
| → §5.1's window refusal, §6 L1240 | → §4.7's admission refusal, §6 L1244 |

Neither carries a qualifier that separates them: the first says *"in the
window"* and does not name a flag; the second says *"under `--as descriptor`"*
and does not name a build. **In the window, under `--as descriptor`, a
full-tier `multi` satisfies both antecedents.** The FOLLOWER rule that would
normally arbitrate says only that the follower is *"decided independently by
§5's own logic"*, and §5's logic is these two sentences. The cell has two
derivable answers, and they print different messages.

This is the residue r14 predicted in a different form: r14 closed the ordering
question for the eight path-independent classes, and the one path-dependent
conjunct is where the pre-existing double answer was hiding — invisible until
the table forced both parentheticals to be read against the same input.

**Construction A — class 2, `wsh(multi(2,K1/<0;1>/*,K2/<0;1>/*))`,
`--as descriptor`, S3-only window.**

- Conjuncts 2–7 all hold; md1 admits the shape → **FULL tier**.
- Follower: **two answers** per the table above. Taking the window one (L1087–1091,
  the more specific sentence, and the one §5.1 L842–844 permits):
- Arm selection is by md1-representability. `<0;1>/*` is md1-representable
  (§5.5 L1171 ✅, chunk-set-id `0xd5e52`), so **arm 1** fires:
  *"Available now: --as md1 — me converts and packs in one step… **Your export
  file is all you need to come back for the QR plate later; nothing is lost by
  waiting.**"*
- **The second half is permanently false.** There is no later QR plate for a
  `multi`: the device's parser takes `sortedmulti` and not `multi`, and §10
  records the refusal to widen as a *decision*, not a gap. The operator is told
  to keep the file and come back for an artefact that will never exist.
- And §6 L1244 — the row that tells the truth, *"the device's descriptor parser
  accepts `sortedmulti` and not `multi`… `sortedmulti` differs from `multi` only
  in key ordering at spend time — it is not a synonym, so `me` will not rewrite
  it for you"* — is the one the ordering suppresses.

**Construction B — class 5, `wsh(multi(2,K1/0/*,K2/0/*))`, `--as descriptor`,
window.** Same route; `/0/*` is not md1-representable, so **arm 2** fires:
*"--as md1 cannot carry this wallet either — key `@0` uses `/0/*`. No path in
this build engraves this file. **It loses nothing by waiting: keep it, and it
packs the day the device update ships.**"* Also false, for the same reason,
compounded — this wallet fails on both paths permanently.

**Taking the OTHER answer does not dissolve the finding, it relocates it.** If
L1096–1099 governs and the cell is ADM(c1) → §6 L1244, then §5.4 L1087–1091's
*"its refusal is the window's, not admission's"* is a false statement in a
NORMATIVE section, §6 L1240's window row never fires for a `multi`, and the tier
justification built on it (*"stripping its identification would blind the
operator"*) is arguing for a follower it does not get. One of the two sentences
is wrong whichever way it is ruled — which is r14's own "no judgement about which
should win is needed" test, met again.

**Why it is Important and not a re-litigation of r14's new-I1.** r14's finding is
dispositioned FIXED above for all eight classes r14 enumerated, and I re-ran two
of them clean. Every one of those eight fails a path-independent conjunct.
Conjunct 1 is the single conjunct the ordering rule's vocabulary cannot express,
it is not on r14's list, and the defect it produces is the *same* one r14 named:
a cell with two derivable answers and an arm whose text is false about the input,
on the journey rule's own test. Per the standing rule that prescribed fixes are
not authoritative, the defect governs.

**Also unfalsifiable by the spec's own gate.** §11 item 5's sibling still pins
*"BOTH alternative variants… (an md1-representable input, and an (a)/(a″)-shaped
one)"*, and both constructions above satisfy that pin — construction A **is** an
md1-representable input and construction B **is** (a)-shaped. A conformant
implementation passes item 5 with arm 1 and arm 2 wired to `multi` inputs whose
promises it cannot keep. Filed as **new-M2** separately for the acceptance side.

**Not prescribing the fix.** Two things need saying, and the second does not go
away if the first is ruled toward admission: (1) which of §5.4's two
parentheticals governs a full-tier `multi` under `--as descriptor` in the window;
(2) a criterion in the window arms that distinguishes *"the descriptor path has
not shipped **yet**"* from *"the descriptor path will **never** carry this
shape"*, because both arms' closing sentences — *"nothing is lost by waiting"*,
*"it packs the day the device update ships"* — are promises about the former, and
the (a)/(a″) `multi` classes reach arm 2 by other routes too.

## new-I4 (Important) — every `--as`-absent classification failure is routed into §4's cascade, so a mistyped mnemonic gets a descriptor-vocabulary refusal at exit 3; this falsifies a shipped green test, unacknowledged

**Where.** §5.1 L806–812 (the discriminator), §6 L1195–1206 (the five-step cause
rule, step 5), §6 L1192 (*"Every refusal in this section is `EXIT_REFUSED` (3)
unless marked otherwise"*), §6 L1232 (the unparseable row) against §6 L1233 (the
empty-file row's stated principle) and §6 L1265 (*"`EXIT_INVALID` (4), as today"*).

**The derivation, and it has no branch.** §5.1 L807: *"When `--as` is absent and
record classification fails, `me` re-reads the whole input through §4's
cascade."* No qualifier — not "when the record looks like a descriptor", not
"when the cascade produces a better diagnostic". The discriminator then specifies
two outcomes: the whole input parses as one descriptor (→ block or carriage
refusal), or it does not and *"some individual record does"* (→ §6's multi-record
row, 4). **The third case — the whole input does not parse AND no individual
record is a descriptor — is not specified**, and §6's cause rule fills it
anyway: step 5, *"otherwise → 'not a descriptor in any form I know', listing the
four"* → §6 L1232 at `EXIT_REFUSED` (3).

**So a mistyped BIP-39 word gets the descriptor refusal.** Measured at
`f1dd328`, `me 0.7.0`:

```
$ me sysw pack --in typo.txt      # 'abandon ×11 abandoz'
rc=4
me: record 0 (records count from 0) is not a form this container can place: not a
BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`
record. Descriptors and addresses are not yet classifiable here — see sysw::classify
```

Under the spec that becomes `EXIT_REFUSED` (3) and *"this is not a wallet
descriptor in any of the four forms `me` reads: a BlueWallet `Key: value` setup
file, a plain BIP-380 descriptor, a `{"label":…,"descriptor":…}` JSON export, or
a single extended key…"* — a message about coordinator exports, handed to
somebody who mistyped one word of a seed phrase. The message that named the
record forms they actually used is gone.

**It breaks a shipped, green test.**
`crates/me-cli/tests/sysw_cli.rs:1928`,
`an_unpackable_record_is_refused_before_a_passphrase_is_minted`, packs the argv
operand `"this is not a record of any class"` and asserts
`err.contains("not a form this container can place")` on a `.failure()`. Run at
`f1dd328`:

```
cargo nextest run --locked -p mnemonic-engrave \
  -E 'test(an_unpackable_record_is_refused_before_a_passphrase_is_minted)'
  PASS [0.021s] mnemonic-engrave::sysw_cli an_unpackable_record_is_refused_…
  Summary [0.023s] 1 test run: 1 passed, 431 skipped
```

That operand is not a descriptor in any of the four forms, so the spec routes it
to L1232 and the assertion fails.

**Why it is Important.** It is a missing case in normative routing whose single
derivable answer regresses a tested surface at both the exit code and the text,
on the **most common wrong input `me sysw pack` receives** — and the spec's own
neighbouring row states the principle it breaks: §6 L1233 keeps the shipped
empty-file refusal *"rather than silently regressing a tested surface"*, and
§6 L1265 keeps `EXIT_INVALID` (4) *"as today"*. This row was given neither
treatment, and no §9 item records the change.

**The distinction that resolves it is already in the spec's vocabulary.** With
`--as` present the operator has declared descriptor intent, and L1232 at (3) is
the right answer (the table's class 9 columns 1, 2, 4, 5 — all correct). With
`--as` absent they have declared nothing, and the shipped record-vocabulary
refusal at (4) is the one that helps. §5.1's discriminator is the sentence that
does not draw the line.

---

# Minor

**new-M1 — in the S3-only window, the one-carries ruling and M6 give opposite
marking instructions for the same block, and the window is the first shipping
build.** §5.1 L772–777 (fold-written): *"when exactly ONE value carries this
particular input, the block still offers both, **unmarked**"*. §5.1 L766–769
(M6): *"In a build where the descriptor path has not shipped, the block **marks
that value inline** — `--as descriptor (not available in this build)` — so the
choice text never offers a BUILD-dead flag unmarked"*. The new rule's trigger is
*"carries"*, and §5.4 L1103–1105 defines carriage as *"admission, §5.3
representability, and **the window** all considered"* — so in the window,
`--as descriptor` does not carry ANY input, and every md1-carried input is an
"exactly one carries" input. Both rules fire; they disagree on the marking. Cells
1×WIN×omitted and 2×WIN×omitted, i.e. the S3 release's ordinary path. Minor
rather than Important because each sentence carries its own qualifier —
**BUILD**-dead in one, **input**-dead in the other — so the intended resolution
(M6 governs build-deadness, the new rule governs input-deadness) is derivable
from the text; and the operator who picks an unmarked `--as descriptor` in the
window still gets the window refusal, which names `--as md1`. One word: scope the
new rule's trigger to *"carries because the other value refuses this input"*.

**new-M2 — §11 item 5's sibling pins the two window arms by the property that
does not distinguish the failing cells, so the acceptance gate cannot fail on
new-I3.** Item 5 requires *"BOTH alternative variants tested (an
md1-representable input, and an (a)/(a″)-shaped one)"*. new-I3's construction A
IS md1-representable and construction B IS (a)-shaped, so both are admissible
witnesses for the pin the spec already has — a `multi` wired to either arm passes
item 5. There is no acceptance case anywhere in §11 for the `multi`-under-
`--as descriptor`-in-the-window cell that §5.4 L1087–1091 rules, even though the
fold added a fourth case to item 5 for the ordering it *did* state. This is the
closure-is-lens-closure second clause in its acceptance form: the gate cannot
execute against the defect. (Filed separately from new-I3 because it survives
whichever way new-I3 is ruled — the cell needs a pin either way.)

---

# Nit

**new-N1 — the ordering paragraph was inserted mid-sentence, leaving a dangling
"followed by" and md1-representability stated twice in adjacent paragraphs.**
§5.1 L828–860 now reads: the window-refusal code block, then the new paragraph
ending *"…and its two arms partition exactly those, **by md1-representability:**"*,
then a new paragraph beginning *"**followed by** ONE of two alternative clauses,
**decided by md1-representability** (walk W11…):"*. The *"followed by"* was
attached to the code block two paragraphs above and now attaches to nothing; the
colon on the inserted paragraph promises the arms that the next paragraph then
re-introduces with the same criterion. No behaviour is ambiguous — both
sentences name the same selector — but the passage reads as two half-sentences.
Moving the ordering paragraph to sit *after* the two arms, or deleting the
duplicated *"decided by md1-representability"*, fixes it.

---

# Verified in passing — recorded so a later round does not re-spend it

- **The ordering rule closes a sixth cell nobody had raised, correctly.** An
  input that does not parse, under `--as descriptor` in the window: §5.1 L830
  emits the window refusal *"AFTER the host-side parse"*, and L842–844 now scopes
  it to *"wallets at least one path admits"*. An unparseable input is not a
  wallet, so the window refusal cannot fire and the cascade refusal does — class
  9 and class 10, columns 4 and 5 of the table, all single-valued. Before the
  fold this cell had the same two-follower ambiguity r14 filed. Unclaimed credit.
- **Empty and whitespace inputs under `--as descriptor` in the window are
  single-valued too**, for the same reason: the no-records refusal precedes any
  parse, so it precedes the window refusal. rc=2, measured.
- **§6 still has exactly 34 data rows.** Counted mechanically at `f1dd328`; the
  fold edited one in place (L1246) and added none.
- **Quoted-span count unchanged at 45.** One new violating span (new-I1); the
  other four pattern matches are deliberate BIP citations required by walk W9.
- **The semantic substitution widens by exactly one site and over-applies
  nowhere** — the eight-row enumeration above. `descriptor path` is used
  consistently for the `--as descriptor` output path at all four in-spec sites.
- **Line widths in the fold's regions:** 78 (M6/one-carries), 75 (ordering para),
  85 (substitution), 86 (§11 item 5). The file has 138 prose lines over 78
  columns, so both outliers sit inside its own norm; r13's N1 was a 105-column
  line and does not recur. No finding.
- **The FULL/PARTIAL partition is unchanged and still exhaustive and disjoint**
  under the fold — the fold touched no tier text. Every class 6/7/8 cell is
  PARTIAL, every class 1–5 cell is FULL, in all six columns.
- **`wallet-id:` emission is consistent across the table.** Classes 1–5 are FULL
  and (a)/(a″) shapes get *"wallet-id: none"*; classes 6–8 are PARTIAL and print
  no `wallet-id:` line; classes 9–10 print no block at all.
- **Both walked journeys still compose.** Journey 1 (BlueWallet `sh` fixture,
  window, `--as descriptor`) is class 1 column 5 — WIN1, and arm 1 is TRUE for a
  `sortedmulti`, unaffected by new-I3, which needs a `multi`. Journey 2 (bare
  BIP-84 `zpub`, childless → (a′) materialises) is class 1 column 3 — BLOCK at 2
  with both values live in a full build, unaffected by new-M1, which needs the
  window.
- **No cross-document copy.** `grep -rn` over `design/` (excluding
  `agent-reports/`) for `as decides`, `carriage rule`, `window substitution`,
  `at least one .*admits` → 0 hits outside the spec. No implementation plan
  exists yet.
- **The shipped pre-parse surface is exactly what §6 records** — seven
  invocations measured at `f1dd328`, table above. §6 L1233, L1234 and L1265's
  exit codes and the 0-stdout rule all confirmed; §5.1 L800–803's BlueWallet
  `--in` claim confirmed.

---

# What would re-close the round

**new-I1** — move `this alternative needs the descriptor path, so the window
substitution applies to it, r14's new-M3` out of the `*"…"*` span and into the
row's existing annotation site next to `(R0 r5's NEW-I2.)`. One edit, mechanical,
and the span sweep in this report is the check.

**new-I2** — give class 5 a window tail that is true. The substitution's stock
replacement promises a future the `multi` form does not have; this site needs its
own, e.g. that neither form of this wallet is engravable in this build and the
`sortedmulti` re-export becomes available when the device update ships — or scope
the substitution rule so it does not reach a clause whose subject the descriptor
path refuses permanently.

**new-I3** — rule the `multi`-under-`--as descriptor`-in-the-window cell. §5.4
L1087–1091 sends it to the window refusal; §5.4 L1096–1099 sends it to the
conjunct-1 admission refusal; one of the two sentences has to change. Then, and
independently of that ruling, give the window arms a criterion that separates
*not yet* from *never*: both closing sentences (*"nothing is lost by waiting"*,
*"it packs the day the device update ships"*) are promises no build can keep for
a `multi`.

**new-I4** — say which refusal an `--as`-absent, non-descriptor, unclassifiable
input gets. The shipped answer is `EXIT_INVALID` (4) with the record-vocabulary
message, it is pinned by a green test, and §6's two neighbouring rows already
state the no-regression principle. One sentence in §5.1's discriminator.

The two Minors and the Nit are single-clause edits and can ride along: scope the
one-carries trigger to input-deadness; add the `multi`-in-window case to §11 item
5's sibling pin; un-dangle the *"followed by"*. Then a re-review scoped to *"did
the fold fix the four, and did it introduce a defect"*.

**What is closed and should not be re-opened:** r14's new-I1 for all eight
path-independent inadmissible classes (both constructions re-run clean here); the
ordering paragraph's motivation and its funds-safety argument; the semantic
substitution's mechanism and its one-site reach (enumerated); the one-carries
ruling itself and its justification (checked at both exemplar sites); r11's
provenance restoration; the §11 item-5 count fix; the 34-row count; the 45-span
count; the FULL/PARTIAL partition; and everything r14 listed as closed.

One line for the cycle's record: **the decision table found what fourteen
readings did not, and it found it by construction rather than by looking
harder.** new-I3 is two parentheticals nine lines apart in the same subsection —
both individually true-sounding, both read many times, contradictory only when a
single input is pushed through both. new-I4 is a rule with no branch, caught by
running the spec's own text against a test that already exists. A section-by-
section lens cannot reach either: they are not wrong sentences, they are cells
with two answers. Enumerating the cells is the lens; it should have been run
before r10.

---

# What the spec's own text leaves open (carried forward; the round that closes it inherits this list)

**§9 residuals (7), verified unchanged at `f1dd328`:** (1) nothing run on
hardware; (2) the three admission-table cells have never been exercised — *a gate
that has never executed*; (3) change addresses and testnet unmeasured in the
`--as md1` address equality; (4) the published `md-codec` 0.42.0 tarball not
byte-compared to the tree; (5) TinyGo compilation of a new `sysw.Classify` arm
unchecked; (6) two negative claims with named, narrower search scopes; (7) §6's
refusal texts *"have not been walked with the operator"* — still stated as open
even though the walk reached refusal text at W5/W11/W13; flagged for a scope
update by r11, r12, r13 and r14 and still not updated.

**Parked with S2 (F-418, S1 → S3 → S2):** §11 item 1 (the `Descriptor` classify
round trip), §11 item 4's `--as descriptor`-only refusal rows, and §11 item 6 (a
`ClassDescriptor` record loaded and displayed on a real device, the discharge of
§9 item 2). All three need the device on the bench.

**Named follow-ups the spec defers to:** F-413 (host-side SLIP-132
normalisation), F-414 (descriptor + other records in one container), F-416
(`--in`'s contract note in `SPEC_systemwide_payloads` §5.6), F-417 (md1 wire
extension seam), F-422 (**RULING WANTED**, owning phase *"descriptor-input plan,
before S1 closes"* — only an interim status-quo ruling is recorded), F-420/F-421
(cross-tool referrals, owning phase "with or after S1"), F-423 (plate packing,
fork-side, with S2).

**Newly surfaced by this round's table, not filed as findings:** class 10 under
any `--as` value gets the unparseable-file refusal rather than a message naming
the record split (the multi-record row is scoped to `--as` absent) — the operator
who half-follows the multi-record row's remedy is told their file is not a
descriptor, with no mention of the other records. Truthful, so no finding, but it
is a plausible loop worth a plan-phase note.
